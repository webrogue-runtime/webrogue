use std::{io::Read, sync::Arc};

mod context;
mod ffi;
mod imports;
mod memory;
mod threads;
use anyhow::Context as _;
use ffi::{ArgGetter, RetSetter};
use webrogue_wrapp::IVFSHandle as _;

extern "C" {
    fn wr_rs_sleep(ms: u32);
    fn wr_reset_timer();
    // fn wr_get_timer() -> u64;
}

#[no_mangle]
extern "C" fn wr_rs_exported_fn(func_i: u32, context: *mut std::ffi::c_void) {
    let context_ref = unsafe { (context as *mut crate::context::Context).as_mut().unwrap() };
    let func = &context_ref.imports.funcs[func_i as usize];
    func(context_ref.store.as_mut().unwrap());
}

#[no_mangle]
extern "C" fn wr_rs_exported_async_fn(func_i: u32, context: *mut std::ffi::c_void) {
    let context_ref = unsafe { (context as *mut crate::context::Context).as_mut().unwrap() };
    let func = &context_ref.imports.funcs[func_i as usize];
    func(context_ref.store.as_mut().unwrap());
}

pub struct WrSyncSched {
    pub actual: Box<dyn wasi_common::WasiSched>,
}

#[wiggle::async_trait]
impl wasi_common::WasiSched for WrSyncSched {
    async fn poll_oneoff<'a>(
        &self,
        poll: &mut wasi_common::sched::Poll<'a>,
    ) -> Result<(), wasi_common::snapshots::preview_1::types::Error> {
        self.actual.poll_oneoff(poll).await
    }
    async fn sched_yield(&self) -> Result<(), wasi_common::snapshots::preview_1::types::Error> {
        self.actual.sched_yield().await
    }
    async fn sleep(
        &self,
        duration: std::time::Duration,
    ) -> Result<(), wasi_common::snapshots::preview_1::types::Error> {
        unsafe {
            wr_rs_sleep(duration.as_millis() as u32);
        }
        Ok(())
    }
}

fn exec_func(func_name: &str) {
    unsafe {
        let mut func_name = func_name.as_bytes().to_vec();
        func_name.push(0);
        ffi::wr_exec_func(func_name.as_ptr());
    }
}

pub fn main(wrapp_data: Option<&'static [u8]>) -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let mut imports = imports::Imports::new();
    // add_imports(&mut imports);
    // wasi_factory.sleep = Some(webrogue_wasi_sync::Sleep {
    //     f: Arc::new(|duration| unsafe {
    //         wr_rs_sleep(duration.as_millis() as u32);
    //     }),
    // });

    imports.add_initialozer(|imports| {
        add_webrogue_gfx_to_linker(imports, |store| &mut store.gfx);
        add_wasi_snapshot_preview1_to_linker(imports, |store| &mut store.preview1_ctx);
        imports.add_fn(
            "wasi",
            "thread-spawn",
            Box::new(|store| {
                let new_store = store.clone();
                let start_arg = ArgGetter::<i32>::get(0);
                let val = store.threads.spawn(new_store, start_arg).unwrap();
                RetSetter::<i32>::set(val);
            }),
        );
    });

    let (config, wrapp_handle) = if let Some(wrapp_data) = wrapp_data {
        let mut builder = webrogue_wrapp::WrappVFSBuilder::from_static_slice(wrapp_data)?;
        let config = builder.config()?.clone();
        let wrapp_handle = builder.build()?;
        (config, wrapp_handle)
    } else {
        let mut builder = webrogue_wrapp::WrappVFSBuilder::from_file_path(
            std::path::PathBuf::from("main.wrapp"),
        )?;
        let config = builder.config()?.clone();
        let wrapp_handle = builder.build()?;
        (config, wrapp_handle)
    };

    let persistent_path = std::path::PathBuf::from("/data")
        .join(".webrogue")
        .join(&config.id)
        .join("persistent");

    let mut wasm_bytes = Vec::new();
    wrapp_handle
        .open_file("/app/main.wasm")
        .with_context(|| {
            anyhow::anyhow!("Unable to open file specified as \"main\" in webrogue.json")
        })?
        .ok_or(anyhow::anyhow!("/app/main.wasm not found"))?
        .read_to_end(&mut wasm_bytes)?;

    let mut jsonptr = imports.to_json().as_bytes().to_vec();
    jsonptr.push(0);
    let mut context = context::Context::new(imports);
    let threads = threads::ThreadsContext::new(context.imports.clone());
    context.store = Some(context::Store {
        gfx: webrogue_gfx::GFXInterface::new(Arc::new(webrogue_gfx::GFXSystem::new(None))),
        preview1_ctx: webrogue_wasip1::make_ctx(wrapp_handle, &config, &persistent_path)?,
        threads: Arc::new(threads),
    });
    let mut limits = None;
    {
        let parser = wasmparser::Parser::new(0);
        for payload in parser.parse_all(&wasm_bytes) {
            match payload? {
                wasmparser::Payload::ImportSection(imports) => {
                    for entry in imports {
                        let entry = entry?;
                        if let wasmparser::TypeRef::Memory(memory) = entry.ty {
                            if memory.shared {
                                if let Some(max) = memory.maximum {
                                    limits = Some((memory.initial as u32, max as u32))
                                }
                            }
                        }
                    }
                }
                wasmparser::Payload::End(_) => break,
                _ => {}
            }
        }
    }
    unsafe {
        if let Some(limits) = limits {
            ffi::wr_make_shared_memory(limits.0, limits.1);
        }
        ffi::wr_init_wasm_module(
            ((&mut context) as *mut context::Context) as *mut std::ffi::c_void,
            jsonptr.as_ptr(),
            wasm_bytes.as_ptr(),
            wasm_bytes.len() as u32,
        );
        exec_func("_start");
    }
    drop(context);

    unsafe { wr_reset_timer() };

    Ok(())
}

webrogue_web_macro::wr_web_integration!({
    target: webrogue_gfx,
    witx: ["../gfx/witx/webrogue_gfx.witx"]
});

webrogue_web_macro::wr_web_integration!({
    target: wasi_common::snapshots::preview_1,
    witx: ["../../external/wasmtime/crates/wasi-common/witx/preview1/wasi_snapshot_preview1.witx"],
    block_on: *
});

#[no_mangle]
extern "C" fn rust_main() {
    match main(None) {
        Err(e) => {
            panic!("{}", e.to_string())
        }
        Ok(_) => {}
    }
}

#[no_mangle]
extern "C" fn rust_main_slice(size: u32, data: *const u8) {
    match main(Some(unsafe {
        std::slice::from_raw_parts(data, size as usize)
    })) {
        Err(e) => {
            panic!("{}", e.to_string())
        }
        Ok(_) => {}
    }
}
