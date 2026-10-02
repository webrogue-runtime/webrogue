use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

use wasmtime::{
    component::{HasData, Linker, ResourceTable},
    AsContext, AsContextMut as _, Memory, SharedMemory,
};

use crate::event_sink::EventStream;

pub(crate) mod generated {
    wasmtime::component::bindgen!({
        path: "wit/webrogue-gfx.wit",
        world: "gfx",
        with: {
            "webrogue:gfx/windowing.window": super::Window,
            "webrogue:gfx/vulkan.linear-memory-marker": super::LinearMemoryMarker,
        },
        imports: {
            "webrogue:gfx/vulkan.[static]linear-memory-marker.create": trappable | store,
            "webrogue:gfx/vulkan.create-blob": trappable | store,
            "webrogue:gfx/vulkan.register-blob": trappable | store,
            "webrogue:gfx/windowing.[method]window.event-stream": trappable | store,
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

#[derive(Clone)]
pub enum LinearMemoryMarker {
    Unshared(Memory),
    Shared(SharedMemory),
}

impl LinearMemoryMarker {
    fn get_data(&self, store: impl AsContext) -> (*mut u8, usize) {
        match self {
            LinearMemoryMarker::Unshared(memory) => {
                (memory.data_ptr(&store), memory.data_size(&store))
            }
            LinearMemoryMarker::Shared(shared_memory) => (
                shared_memory.data().as_ptr() as *const u8 as *mut u8,
                shared_memory.data_size(),
            ),
        }
    }

    fn get_range(&self, offset: u32, size: u32, store: impl AsContext) -> Option<*mut u8> {
        let (ptr, len) = self.get_data(&store);
        if len < (offset + size) as usize {
            return None;
        }
        return Some(unsafe { ptr.add(offset as usize) });
    }
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
        window: wasmtime::component::Resource<Window>,
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

    fn resource_unref(&mut self, res_id: u32) -> wasmtime::Result<()> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context.lock().unwrap().resource_unref(res_id);
        Ok(())
    }

    fn create_sync(&mut self, value: u64) -> wasmtime::Result<u32> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let sync_id = virgl_context.lock().unwrap().sync_create(value);
        Ok(sync_id)
    }

    fn sync_unref(&mut self, sync_id: u32) -> wasmtime::Result<()> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context.lock().unwrap().sync_unref(sync_id);
        Ok(())
    }

    fn sync_read(&mut self, sync_id: u32) -> wasmtime::Result<u64> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let value = virgl_context.lock().unwrap().sync_read(sync_id);
        Ok(value)
    }

    fn sync_write(&mut self, sync_id: u32, value: u64) -> wasmtime::Result<()> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context.lock().unwrap().sync_write(sync_id, value);
        Ok(())
    }

    fn submit_cmd(
        &mut self,
        headers: Vec<u8>,
        cmds: Vec<u8>,
        syncs: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let words = |bytes: &[u8]| -> Vec<u32> {
            bytes
                .chunks_exact(4)
                .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
                .collect()
        };
        virgl_context
            .lock()
            .unwrap()
            .submit_cmd(&words(&headers), &words(&cmds), &words(&syncs));
        Ok(())
    }

    fn sync_wait(&mut self, flags: u32, timeout: u32, syncs: Vec<u32>) -> wasmtime::Result<i32> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let result = virgl_context
            .lock()
            .unwrap()
            .sync_wait(flags, timeout, &syncs);
        Ok(result)
    }

    fn get_max_timeline_count(&mut self) -> wasmtime::Result<u32> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let result = virgl_context.lock().unwrap().get_max_timeline_count();
        Ok(result)
    }

    fn get_capset(&mut self, id: u32, version: u32) -> wasmtime::Result<Vec<u8>> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };

        let data = virgl_context.lock().unwrap().get_capset(id, version);
        Ok(data)
    }

    fn context_init(&mut self, capset_id: u32) -> wasmtime::Result<()> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context.lock().unwrap().context_init(capset_id);
        Ok(())
    }

    fn create_renderer(&mut self, name: Vec<u8>) -> wasmtime::Result<()> {
        let Some(virgl_context) = self.ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        virgl_context.lock().unwrap().create_renderer(&name);
        Ok(())
    }
}

impl<T> generated::webrogue::gfx::vulkan::HostWithStore<T> for GFX {
    fn create_blob(
        mut host: wasmtime::component::Access<T, Self>,
        blob_id: u64,
        buf_ptr: u32,
        buf_len: u32,
        memory: wasmtime::component::Resource<LinearMemoryMarker>,
    ) -> wasmtime::Result<u32> {
        let Some(virgl_context) = host.get().ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let memory = host.get().table.get(&memory)?.clone();

        let virgl_context = virgl_context.lock().unwrap();
        if blob_id != 0 {
            Ok(virgl_context.create_blob(std::ptr::null(), buf_len as usize, blob_id))
        } else {
            let Some(ptr) = memory.get_range(buf_ptr, buf_len, host.as_context()) else {
                wasmtime::bail!("create_blob: outside of linear memory")
            };
            Ok(virgl_context.create_blob(ptr, buf_len as usize, blob_id))
        }
    }

    fn register_blob(
        mut host: wasmtime::component::Access<T, Self>,
        res_id: u32,
        buf_ptr: u32,
        buf_len: u32,
        memory: wasmtime::component::Resource<LinearMemoryMarker>,
    ) -> wasmtime::Result<()> {
        let Some(virgl_context) = host.get().ctx.0.get_virgl_context() else {
            wasmtime::bail!("get_virgl_context() failed")
        };
        let memory = host.get().table.get(&memory)?.clone();
        let Some(ptr) = memory.get_range(buf_ptr, buf_len, host.as_context()) else {
            wasmtime::bail!("register_blob: outside of linear memory")
        };
        virgl_context
            .lock()
            .unwrap()
            .register_blob(res_id.into(), ptr, buf_len as usize);
        Ok(())
    }
}

impl<T> generated::webrogue::gfx::vulkan::HostLinearMemoryMarkerWithStore<T> for GFX {
    fn create(
        mut host: wasmtime::component::Access<T, Self>,
        ptr: u32,
        data: Vec<u8>,
    ) -> wasmtime::Result<Option<wasmtime::component::Resource<LinearMemoryMarker>>> {
        let instances = host.as_context_mut().debug_all_instances();
        let mut memories = Vec::new();
        let mut present_indices = BTreeSet::new();
        let mut buffer = vec![0u8; data.len()];
        for instance in instances {
            for i in 0.. {
                if let Some(memory) = instance.debug_memory(host.as_context_mut(), i) {
                    if memory
                        .read(host.as_context_mut(), ptr as usize, &mut buffer)
                        .is_err()
                    {
                        continue;
                    }
                    if data != buffer {
                        continue;
                    }

                    if present_indices.insert(memory.debug_index_in_store()) {
                        memories.push(LinearMemoryMarker::Unshared(memory));
                    }
                } else {
                    break;
                }
            }
        }

        if memories.len() != 1 {
            return Ok(None);
        }
        let memory = memories.pop().unwrap();

        Ok(Some(host.get().table.push(memory)?))
    }
}

impl<'a> generated::webrogue::gfx::vulkan::HostLinearMemoryMarker for GFXCtxView<'a> {
    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<LinearMemoryMarker>,
    ) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }
}
