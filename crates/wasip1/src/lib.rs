use cap_rand::Rng as _;
use cap_rand::SeedableRng as _;
use rand::Rng as _;

mod fs;
mod stdout;

struct RandomCtx {}

impl cap_rand::RngCore for RandomCtx {
    fn next_u32(&mut self) -> u32 {
        rand::random()
    }

    fn next_u64(&mut self) -> u64 {
        rand::random()
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
        rand::rng().fill_bytes(dst);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), cap_rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

pub fn make_ctx<VFSHandle: webrogue_wrapp::IVFSHandle + 'static>(
    handle: VFSHandle,
    config: &webrogue_wrapp::config::Config,
    persistent_dir: &std::path::Path,
) -> anyhow::Result<wasi_common::WasiCtx> {
    #[cfg(not(target_arch = "wasm32"))]
    let mut wasi_ctx = {
        let mut builder = wasi_common::sync::WasiCtxBuilder::new();
        // builder.inherit_stdio();
        // builder.stdout(Box::new(stdout::STDOutFile {}));
        // builder.stderr(Box::new(stdout::STDOutFile {}));
        builder.build()
    };
    #[cfg(target_arch = "wasm32")]
    let mut wasi_ctx = {
        let random = Box::new(cap_rand::rngs::StdRng::from_seed(
            cap_rand::thread_rng(cap_rand::ambient_authority()).r#gen(),
        ));
        let clocks = wasi_common::WasiClocks::new();
        let sched = Box::new(Sched {});
        let table = wasi_common::Table::new();
        wasi_common::WasiCtx::new(random, clocks, sched, table)
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
                    let home_dir = wasi_common::sync::dir::Dir::from_cap_std(
                        wasi_common::sync::Dir::open_ambient_dir(
                            real_path,
                            wasi_common::sync::ambient_authority(),
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

struct Sched {}

#[async_trait::async_trait]
impl wasi_common::WasiSched for Sched {
    async fn poll_oneoff<'a>(
        &self,
        poll: &mut wasi_common::Poll<'a>,
    ) -> Result<(), wasi_common::Error> {
        todo!()
    }
    async fn sched_yield(&self) -> Result<(), wasi_common::Error> {
        todo!()
    }
    async fn sleep(&self, duration: std::time::Duration) -> Result<(), wasi_common::Error> {
        todo!()
    }
}
