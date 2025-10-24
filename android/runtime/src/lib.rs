#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    use std::{ffi::CString, str::FromStr};
    use winit_android::EventLoopBuilderExtAndroid as _;

    let mut event_loop_builder = winit::event_loop::EventLoopBuilder::default();
    event_loop_builder.with_android_app(app.clone());

    let event_loop = event_loop_builder.build().unwrap();

    let asset: ndk::asset::OpenedFileDescriptor = app
        .asset_manager()
        .open(&CString::from_str("aot.swrapp").unwrap())
        .unwrap()
        .open_file_descriptor()
        .unwrap();

    let vfs_builder = webrogue_wasmtime::WrappVFSBuilder::from_file_part(
        std::fs::File::from(asset.fd),
        asset.offset as u64,
        asset.size as u64,
    )
    .unwrap();

    let data_dir = app.internal_data_path().unwrap();

    let gfx_builder = webrogue_gfx_winit::WinitBuilder::default().with_event_loop(event_loop);

    #[cfg(feature = "launcher")]
    webrogue_wasmtime::run_jit_builder(gfx_builder, vfs_builder, &data_dir).unwrap();
    #[cfg(feature = "runner")]
    webrogue_wasmtime::run_aot_builder(gfx_builder, vfs_builder, &data_dir).unwrap();
}
