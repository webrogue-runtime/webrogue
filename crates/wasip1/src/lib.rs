#![cfg_attr(target_arch = "wasm32", feature(stdarch_wasm_atomic_wait))]
mod fs;
mod stdout;

pub fn make_ctx<VFSHandle: webrogue_wrapp::IVFSHandle + 'static>(
    handle: VFSHandle,
    config: &webrogue_wrapp::config::Config,
    persistent_dir: &std::path::Path,
) -> anyhow::Result<webrogue_wasi_common::WasiCtx> {
    #[cfg(not(target_arch = "wasm32"))]
    let mut wasi_ctx = {
        let mut builder = webrogue_wasi_common::sync::WasiCtxBuilder::new();
        // builder.inherit_stdio();
        // builder.stdout(Box::new(stdout::STDOutFile {}));
        // builder.stderr(Box::new(stdout::STDOutFile {}));
        builder.build()
    };
    #[cfg(target_arch = "wasm32")]
    let mut wasi_ctx = {
        use rand::SeedableRng as _;

        let random = Box::new(rand::rngs::StdRng::from_seed(rand::random()));
        let clocks = webrogue_wasi_common::WasiClocks::new();
        let sched = Box::new(Sched {});
        let table = webrogue_wasi_common::Table::new();
        webrogue_wasi_common::WasiCtx::new(random, clocks, sched, table)
    };

    let app_dir = fs::Dir::root(handle);
    wasi_ctx.push_preopened_dir(Box::new(app_dir), "/")?;

    if let Some(filesystem) = &config.filesystem {
        if let Some(persistent) = &filesystem.persistent {
            for persistent in persistent {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // TODO check if this check enough
                    anyhow::ensure!(
                        !persistent.name.contains("/")
                            && !persistent.name.contains("\\")
                            && persistent.name != ".."
                            && persistent.name != ".",
                        "Persistent directory name is invalid: {}",
                        persistent.name
                    );
                    let real_path = persistent_dir.join(&persistent.name);
                    if !real_path.is_dir() {
                        std::fs::create_dir_all(&real_path)?;
                    }
                    let home_dir = webrogue_wasi_common::sync::dir::Dir::from_cap_std(
                        webrogue_wasi_common::sync::Dir::open_ambient_dir(
                            real_path,
                            webrogue_wasi_common::sync::ambient_authority(),
                        )?,
                    );
                    wasi_ctx.push_preopened_dir(Box::new(home_dir), &persistent.mapped_path)?;
                }

                #[cfg(target_arch = "wasm32")]
                {
                    todo!()
                }
            }
        }
    };

    wasi_ctx.push_preopened_dir(fs::make_dev_root(), "/dev")?;

    if let Some(env) = &config.env {
        for (key, value) in env.iter() {
            wasi_ctx.push_env(key, value)?;
        }
    }

    Ok(wasi_ctx)
}

#[cfg(target_arch = "wasm32")]
struct Sched {}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait]
impl webrogue_wasi_common::WasiSched for Sched {
    async fn poll_oneoff<'a>(
        &self,
        poll: &mut webrogue_wasi_common::Poll<'a>,
    ) -> Result<(), webrogue_wasi_common::Error> {
        todo!()
    }
    async fn sched_yield(&self) -> Result<(), webrogue_wasi_common::Error> {
        todo!()
    }
    async fn sleep(
        &self,
        duration: std::time::Duration,
    ) -> Result<(), webrogue_wasi_common::Error> {
        blocking_sleep(duration.as_millis() as i64);
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub fn blocking_sleep(millis: i64) {
    // We create a dummy variable to "wait" on
    let mut dummy: i32 = 0;
    let ptr = &mut dummy as *mut i32;

    // Convert milliseconds to nanoseconds for the WASM intrinsic
    let timeout_ns = millis * 1_000_000;

    unsafe {
        // 0: The pointer to wait on
        // 0: The "expected" value (it matches, so it will sleep)
        // timeout_ns: How long to sleep before timing out
        core::arch::wasm32::memory_atomic_wait32(ptr, 0, timeout_ns);
    }
}

fn make_runtime() -> tokio::runtime::Runtime {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::Builder::new()
        .name("wasi-io".to_owned())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            tx.send(runtime).unwrap();
        })
        .unwrap();
    rx.recv().unwrap()
}

lazy_static::lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = make_runtime();
}

pub fn run_in_executor<F: std::future::Future>(
    future: F,
) -> wasmtime_internal_core::error::Result<F::Output> {
    Ok(RUNTIME.block_on(future))
}
