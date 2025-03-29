mod fs;
mod fs_builder;

pub fn make_ctx(wrapp: webrogue_wrapp::WrappHandle) -> anyhow::Result<wasi_common::WasiCtx> {
    let mut builder = wasi_common::sync::WasiCtxBuilder::new();
    builder.inherit_stdio();
    let mut wasi_ctx = builder.build();
    let app_dir = fs::Dir::root(wrapp);
    wasi_ctx.push_preopened_dir(Box::new(app_dir), "/app")?;

    // wasi_ctx.push_env("SUPERTUXKART_DATADIR", "/app")?;
    // wasi_ctx.push_env("HOME", "/home/someone/wr_home")?;
    // let home_dir = wasi_common::sync::dir::Dir::from_cap_std(
    //     wasi_common::sync::Dir::open_ambient_dir("/", wasi_common::sync::ambient_authority())?,
    // );
    // wasi_ctx.push_preopened_dir(Box::new(home_dir), "/")?;

    Ok(wasi_ctx)
}
