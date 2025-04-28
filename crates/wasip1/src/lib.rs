mod fs;
mod stdout;

pub fn make_ctx<
    FilePosition: webrogue_wrapp::IFilePosition + 'static,
    FileReader: webrogue_wrapp::IFileReader + 'static,
    VFSHandle: webrogue_wrapp::IVFSHandle<FilePosition, FileReader> + 'static,
>(
    handle: VFSHandle,
    config: &webrogue_wrapp::config::Config,
    persistent_dir: &std::path::PathBuf,
) -> anyhow::Result<wasi_common::WasiCtx> {
    let mut builder = wasi_common::sync::WasiCtxBuilder::new();
    // builder.inherit_stdio();
    builder.stdout(Box::new(stdout::STDOutFile {}));
    builder.stderr(Box::new(stdout::STDOutFile {}));
    let mut wasi_ctx = builder.build();

    let app_dir = fs::Dir::root(handle);
    wasi_ctx.push_preopened_dir(Box::new(app_dir), "/")?;

    if let Some(filesystem) = &config.filesystem {
        if let Some(persistent) = &filesystem.persistent {
            for persistent in persistent {
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
        }
    };

    if let Some(env) = &config.env {
        for (key, value) in env.iter() {
            wasi_ctx.push_env(key, value)?;
        }
    }

    Ok(wasi_ctx)
}
