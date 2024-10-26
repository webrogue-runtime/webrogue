pub use anyhow::Result;
use clap::Parser;
use webrogue_runtime::WasiFactory;

#[cfg(feature = "backend_wasmtime")]
use webrogue_backend_wasmtime::{make_funcs, Backend};

#[cfg(feature = "_backend_wasmer")]
use webrogue_backend_wasmer::{make_funcs, Backend};

#[cfg(feature = "backend_v8")]
use webrogue_backend_v8::{make_funcs, Backend};

#[cfg(feature = "wasi_sync")]
fn make_wasi_factory() -> impl webrogue_runtime::WasiFactory {
    webrogue_wasi_sync::WasiFactory::new()
}

make_funcs!({
    "wasi_snapshot_preview1": {
        module: webrogue_wasi::wasi_snapshot_preview1
    },
    "webrogue_gl": {
        attribute: "#[cfg(feature = \"gl\")]",
        module: webrogue_gl::api
    },
    "webrogue_gfx": {
        attribute: "#[cfg(feature = \"_gfx\")]",
        module: webrogue_gfx
    }
});

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

pub fn main() -> Result<()> {
    let lifecycle = webrogue_runtime::Lifecycle::new();

    let wasi_factory = make_wasi_factory();
    let mut wasi = wasi_factory.make();

    wasi_factory.add_actual_dir(&mut wasi, std::env::current_dir()?, "/");

    let args = Cli::parse();
    let reader = webrogue_runtime::wrapp::WrappHandle::from_file_path(args.path)?;

    webrogue_std_stream_os::bind_streams(&mut wasi);

    let backend = Backend::new();

    #[cfg(feature = "_gfx")]
    let mut webrogue_gfx_context =
        webrogue_gfx::Context::new(Box::new(webrogue_gfx_rust_sdl::make_system));
    #[cfg(feature = "gl")]
    let mut webrogue_gl_context = webrogue_gl::api::Context::new(&mut webrogue_gfx_context);
    lifecycle.run(
        backend,
        make_imports(),
        make_context_vec(
            &mut wasi,
            #[cfg(feature = "gl")]
            &mut webrogue_gl_context,
            #[cfg(feature = "_gfx")]
            &mut webrogue_gfx_context,
        ),
        reader,
    )?;
    #[cfg(feature = "gl")]
    drop(webrogue_gl_context);
    #[cfg(feature = "_gfx")]
    drop(webrogue_gfx_context);
    drop(wasi);

    Ok(())
}
