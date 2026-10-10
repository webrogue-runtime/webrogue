use std::sync::{Arc, Mutex};

use wasmtime::component::{HasData, Linker, ResourceTable, WasmList};
use wasmtime::{AsContext as _, AsContextMut as _};

use crate::event_sink::EventStream;

pub(crate) mod generated {
    wasmtime::component::bindgen!({
        path: "wit/webrogue-gfx.wit",
        world: "gfx",
        with: {
            "webrogue:gfx/windowing.window": super::Window,
            "webrogue:gfx/vulkan.renderer": super::VulkanRenderer,
        },
        imports: {
            "webrogue:gfx/windowing.[method]window.event-stream": trappable | store,
            "webrogue:gfx/vulkan.[method]renderer.create-device-memory-blob": trappable | store,
            default: trappable,
        },
    });
}

pub trait GFXView: Send {
    fn gfx_ctx(&mut self) -> GFXCtxView<'_>;
}

struct GFX;

impl HasData for GFX {
    type Data<'a> = GFXCtxView<'a>;
}

pub struct GFXCtxView<'a> {
    pub ctx: &'a mut System,
    pub table: &'a mut ResourceTable,
}

pub fn add_to_linker<T>(linker: &mut Linker<T>) -> wasmtime::Result<()>
where
    T: GFXView + 'static,
{
    generated::webrogue::gfx::windowing::add_to_linker::<_, GFX>(linker, GFXView::gfx_ctx)?;
    generated::webrogue::gfx::cpu_rendering::add_to_linker::<_, GFX>(linker, GFXView::gfx_ctx)?;
    generated::webrogue::gfx::vulkan::add_to_linker::<_, GFX>(linker, GFXView::gfx_ctx)?;
    generated::webrogue::gfx::device_info::add_to_linker::<_, GFX>(linker, GFXView::gfx_ctx)?;

    // Bindgen normally lifts lists into Vecs. These blob functions need the
    // guest buffer's backing memory directly, so register typed WasmList
    // callbacks instead of the generated Vec-based wrappers.
    linker.allow_shadowing(true);
    let result: wasmtime::Result<()> = (|| {
        let mut root = linker.root();
        let mut vulkan = root.instance("webrogue:gfx/vulkan")?;
        vulkan.func_wrap("[method]renderer.create-shmem", create_shmem::<T>)?;
        vulkan.func_wrap(
            "[method]renderer.register-blob",
            register_blob_with_memory::<T>,
        )?;
        wasmtime::Result::Ok(())
    })();
    linker.allow_shadowing(false);
    result?;

    Ok(())
}

pub trait AbstractSystem: Send + Sync {
    fn make_window(&self) -> Window;
    fn pump(&self);
    fn get_virgl_context(&self) -> Option<Arc<Mutex<webrogue_virgl::ContextContainer>>>;
}

#[derive(Clone)]
pub struct System(Arc<dyn AbstractSystem>);

impl System {
    pub fn new(system: Arc<dyn AbstractSystem>) -> Self {
        Self(system)
    }
}

pub trait AbstractWindow: Send + Sync {
    fn get_size(&self) -> (u32, u32);
    fn get_gl_size(&self) -> (u32, u32);
    fn get_vk_id(&self) -> Option<u32>;
    fn present_pixels(&self, pixels: &[u32]) -> anyhow::Result<bool>;
    fn get_event_stream(&self) -> EventStream;
}

pub struct Window(Arc<dyn AbstractWindow>);

impl Window {
    pub fn new(window: Arc<dyn AbstractWindow>) -> Self {
        Self(window)
    }
}

pub struct VulkanRenderer {
    id: u32,
}

fn create_shmem<T>(
    mut store: wasmtime::StoreContextMut<'_, T>,
    (self_, buf): (wasmtime::component::Resource<VulkanRenderer>, WasmList<u8>),
) -> wasmtime::Result<(u32,)>
where
    T: GFXView + 'static,
{
    let (buf_ptr, buf_len) = {
        let buf = buf.as_le_slice(store.as_context());
        (buf.as_ptr().cast_mut(), buf.len())
    };
    wasmtime::ensure!(buf_len != 0, "create-shmem: buffer must not be empty");
    let host = store.data_mut().gfx_ctx();
    let renderer_id = host.table.get(&self_)?.id;
    let Some(virgl_context) = host.ctx.0.get_virgl_context() else {
        wasmtime::bail!("get_virgl_context() failed")
    };

    let virgl_context = virgl_context.lock().unwrap();

    Ok((virgl_context.create_blob(renderer_id, buf_ptr, buf_len, 0),))
}

fn register_blob_with_memory<T>(
    mut store: wasmtime::StoreContextMut<'_, T>,
    (self_, res_id, buf): (
        wasmtime::component::Resource<VulkanRenderer>,
        u32,
        WasmList<u8>,
    ),
) -> wasmtime::Result<()>
where
    T: GFXView + 'static,
{
    let (buf_ptr, buf_len) = {
        let buf = buf.as_le_slice(store.as_context());
        (buf.as_ptr().cast_mut(), buf.len())
    };
    let host = store.data_mut().gfx_ctx();
    let renderer_id = host.table.get(&self_)?.id;
    let Some(virgl_context) = host.ctx.0.get_virgl_context() else {
        wasmtime::bail!("get_virgl_context() failed")
    };
    virgl_context
        .lock()
        .unwrap()
        .register_blob(renderer_id, res_id.into(), buf_ptr, buf_len);
    Ok(())
}

pub trait AbstractBuilder {
    fn run<Output>(
        self,
        body_fn: impl FnOnce(System) -> Output + Send + 'static,
        vulkan_requirement: Option<bool>,
    ) -> anyhow::Result<Output>
    where
        Output: Send + 'static;
}

impl<'a> generated::webrogue::gfx::windowing::Host for GFXCtxView<'a> {
    fn close_window(
        &mut self,
        _window: wasmtime::component::Resource<Window>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl<T> generated::webrogue::gfx::windowing::HostWindowWithStore<T> for GFX {
    fn event_stream(
        mut host: wasmtime::component::Access<T, Self>,
        self_: wasmtime::component::Resource<Window>,
    ) -> wasmtime::Result<
        wasmtime::component::StreamReader<generated::webrogue::gfx::windowing::WindowEvent>,
    > {
        let window = &host.get().table.get(&self_)?.0.clone();

        Ok(wasmtime::component::StreamReader::new(
            host.as_context_mut(),
            window.get_event_stream(),
        )?)
    }
}

impl<'a> generated::webrogue::gfx::windowing::HostWindow for GFXCtxView<'a> {
    fn get_logical_size(
        &mut self,
        self_: wasmtime::component::Resource<Window>,
    ) -> wasmtime::Result<(u32, u32)> {
        Ok(self.table.get(&self_)?.0.get_size())
    }

    fn get_physical_size(
        &mut self,
        self_: wasmtime::component::Resource<Window>,
    ) -> wasmtime::Result<(u32, u32)> {
        Ok(self.table.get(&self_)?.0.get_gl_size())
    }

    fn drop(&mut self, rep: wasmtime::component::Resource<Window>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }

    fn new(&mut self) -> wasmtime::Result<wasmtime::component::Resource<Window>> {
        let window = self.ctx.0.make_window();
        Ok(self.table.push(window)?)
    }

    fn get_vulkan_id(
        &mut self,
        self_: wasmtime::component::Resource<Window>,
    ) -> wasmtime::Result<u32> {
        let Some(id) = self.table.get(&self_)?.0.get_vk_id() else {
            wasmtime::bail!("get_vulkan_id failed")
        };
        Ok(id)
    }
}

impl<'a> generated::webrogue::gfx::device_info::Host for GFXCtxView<'a> {
    fn get_os_family(&mut self) -> wasmtime::Result<u8> {
        let os_family = cfg_select! {
            target_os = "linux" => {
                1
            }
            target_os = "windows" => {
                2
            }
            target_os = "macos" => {
                3
            }
            target_os = "android" => {
                4
            }
            target_os = "ios" => {
                5
            }
            _ => {
                0
            }
        };
        Ok(os_family)
    }
}

impl<'a> generated::webrogue::gfx::cpu_rendering::Host for GFXCtxView<'a> {
    fn present_pixels(
        &mut self,
        window: wasmtime::component::Resource<Window>,
        buf: Vec<u32>,
    ) -> wasmtime::Result<()> {
        self.table
            .get(&window)?
            .0
            .present_pixels(&buf)
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(())
    }
}

impl<'a> generated::webrogue::gfx::vulkan::Host for GFXCtxView<'a> {
    fn check_presence(&mut self) -> wasmtime::Result<bool> {
        Ok(self.ctx.0.get_virgl_context().is_some())
    }
}

impl<'a> generated::webrogue::gfx::vulkan::HostRenderer for GFXCtxView<'a> {
    fn new(
        &mut self,
        name: Vec<u8>,
    ) -> wasmtime::Result<wasmtime::component::Resource<VulkanRenderer>> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let id = virgl_context.lock().unwrap().create_renderer(&name);
        if id == 0 {
            wasmtime::bail!("failed to create Vulkan renderer")
        }
        Ok(self.table.push(VulkanRenderer { id })?)
    }

    fn create_shmem(
        &mut self,
        _self_: wasmtime::component::Resource<VulkanRenderer>,
        _buf: Vec<u8>,
    ) -> std::result::Result<u32, wasmtime::Error> {
        wasmtime::bail!(
            "renderer.create-shmem must use the custom lifting binding installed by add_to_linker"
        )
    }

    fn register_blob(
        &mut self,
        _self_: wasmtime::component::Resource<VulkanRenderer>,
        _res_id: u32,
        _buf: Vec<u8>,
    ) -> wasmtime::Result<()> {
        wasmtime::bail!(
            "renderer.register-blob must use the custom lifting binding installed by add_to_linker"
        )
    }

    fn drop(&mut self, rep: wasmtime::component::Resource<VulkanRenderer>) -> wasmtime::Result<()> {
        let renderer = self.table.delete(rep)?;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context.lock().unwrap().destroy_renderer(renderer.id);
        Ok(())
    }

    fn resource_unref(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        res_id: u32,
    ) -> wasmtime::Result<()> {
        let renderer_id = self.table.get(&self_)?.id;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context
            .lock()
            .unwrap()
            .resource_unref(renderer_id, res_id);
        Ok(())
    }

    fn create_sync(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        value: u64,
    ) -> wasmtime::Result<u32> {
        let renderer_id = self.table.get(&self_)?.id;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let sync_id = virgl_context
            .lock()
            .unwrap()
            .sync_create(renderer_id, value);
        Ok(sync_id)
    }

    fn sync_unref(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        sync_id: u32,
    ) -> wasmtime::Result<()> {
        let renderer_id = self.table.get(&self_)?.id;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context
            .lock()
            .unwrap()
            .sync_unref(renderer_id, sync_id);
        Ok(())
    }

    fn sync_read(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        sync_id: u32,
    ) -> wasmtime::Result<u64> {
        let renderer_id = self.table.get(&self_)?.id;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let value = virgl_context
            .lock()
            .unwrap()
            .sync_read(renderer_id, sync_id);
        Ok(value)
    }

    fn sync_write(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        sync_id: u32,
        value: u64,
    ) -> wasmtime::Result<()> {
        let renderer_id = self.table.get(&self_)?.id;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context
            .lock()
            .unwrap()
            .sync_write(renderer_id, sync_id, value);
        Ok(())
    }

    fn submit_cmd(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        headers: Vec<u8>,
        cmds: Vec<u8>,
        syncs: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let renderer_id = self.table.get(&self_)?.id;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let words = |bytes: &[u8]| -> Vec<u32> {
            bytes
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect()
        };
        virgl_context.lock().unwrap().submit_cmd(
            renderer_id,
            &words(&headers),
            &words(&cmds),
            &words(&syncs),
        );
        Ok(())
    }

    fn sync_wait(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        flags: u32,
        timeout: u32,
        syncs: Vec<u32>,
    ) -> wasmtime::Result<i32> {
        let renderer_id = self.table.get(&self_)?.id;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let result = virgl_context
            .lock()
            .unwrap()
            .sync_wait(renderer_id, flags, timeout, &syncs);
        Ok(result)
    }

    fn get_max_timeline_count(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
    ) -> wasmtime::Result<u32> {
        self.table.get(&self_)?;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let result = virgl_context.lock().unwrap().get_max_timeline_count();
        Ok(result)
    }

    fn get_capset(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        id: u32,
        version: u32,
    ) -> wasmtime::Result<Vec<u8>> {
        self.table.get(&self_)?;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };

        let data = virgl_context.lock().unwrap().get_capset(id, version);
        Ok(data)
    }

    fn context_init(
        &mut self,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        capset_id: u32,
    ) -> wasmtime::Result<()> {
        let renderer_id = self.table.get(&self_)?.id;
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context
            .lock()
            .unwrap()
            .context_init(renderer_id, capset_id);
        Ok(())
    }
}

impl<T> generated::webrogue::gfx::vulkan::HostRendererWithStore<T> for GFX {
    fn create_device_memory_blob(
        mut host: wasmtime::component::Access<T, Self>,
        self_: wasmtime::component::Resource<VulkanRenderer>,
        blob_id: u64,
        size: u64,
    ) -> wasmtime::Result<u32> {
        let Some(virgl_context) = host.get().ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let renderer_id = host.get().table.get(&self_)?.id;

        wasmtime::ensure!(
            blob_id != 0,
            "create_device_memory_blob_with_memory: blob_id can't be zero"
        );
        wasmtime::ensure!(
            size != 0,
            "create_device_memory_blob_with_memory: size can't be zero"
        );
        let virgl_context = virgl_context.lock().unwrap();
        let size = size.try_into().map_err(|_| {
            wasmtime::format_err!("create_device_memory_blob_with_memory: error converting size")
        })?;

        Ok(virgl_context.create_blob(renderer_id, std::ptr::null(), size, blob_id))
    }
}
