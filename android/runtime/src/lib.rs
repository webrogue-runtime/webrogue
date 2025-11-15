#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    use std::{ffi::CString, str::FromStr};
    use winit_android::EventLoopBuilderExtAndroid as _;

    let mut event_loop_builder = winit::event_loop::EventLoopBuilder::default();
    event_loop_builder.with_android_app(app.clone());

    let event_loop = event_loop_builder.build().unwrap();

    #[cfg(feature = "runner")]
    let vfs_builder = {
        let asset: ndk::asset::OpenedFileDescriptor = app
            .asset_manager()
            .open(&CString::from_str("aot.swrapp").unwrap())
            .unwrap()
            .open_file_descriptor()
            .unwrap();

        webrogue_wasmtime::WrappVFSBuilder::from_file_part(
            std::fs::File::from(asset.fd),
            asset.offset as u64,
            asset.size as u64,
        )
        .unwrap()
    };

    #[cfg(feature = "launcher")]
    let vfs_builder = {
        let activity = app.activity_as_ptr() as jni::sys::jobject;

        let vm =
            unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as *mut jni::sys::JavaVM) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();

        let intent = env
            .call_method(
                unsafe { jni::objects::JObject::from_raw(activity) },
                "getIntent",
                "()Landroid/content/Intent;",
                &[],
            )
            .unwrap()
            .l()
            .unwrap();

        let extra = env
            .call_method(intent, "getExtras", "()Landroid/os/Bundle;", &[])
            .unwrap()
            .l()
            .unwrap();

        let key = env.new_string("data").unwrap();
        let default_value = env.new_string("data").unwrap();
        let data = env
            .call_method(
                extra,
                "getString",
                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
                &[(&key).into(), (&default_value).into()],
            )
            .unwrap()
            .l()
            .unwrap();

        let data = env.get_string((&data).into()).unwrap();

        serde_json::from_str::<webrogue_wasmtime::RealVFSBuilder>(data.to_str().unwrap()).unwrap()
    };

    let data_dir = app.internal_data_path().unwrap();

    let mut gfx_builder = webrogue_gfx_winit::SimpleWinitBuilder::default();
    gfx_builder = gfx_builder.with_event_loop(event_loop);
    // Currently there are no means to recreate surfaces
    // destroyed on [onStop]: https://developer.android.com/reference/android/app/Activity#onStop()
    // so we simply exit and hope Android will restart this activity
    gfx_builder = gfx_builder.with_on_hide(Box::new(|| {
        use ndk_sys::quick_exit;
        unsafe {
            quick_exit(42);
        }
    }));
    #[cfg(feature = "launcher")]
    webrogue_wasmtime::run_jit_builder(gfx_builder, vfs_builder, &data_dir).unwrap();
    #[cfg(feature = "runner")]
    webrogue_wasmtime::run_aot_builder(gfx_builder, vfs_builder, &data_dir).unwrap();
}
