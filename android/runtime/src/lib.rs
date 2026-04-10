#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    use winit_android::EventLoopBuilderExtAndroid as _;

    let mut event_loop_builder = winit::event_loop::EventLoopBuilder::default();
    event_loop_builder.with_android_app(app.clone());

    let event_loop = event_loop_builder.build().unwrap();

    let mut gfx_builder = webrogue_gfx_winit::SimpleWinitBuilder::with_event_loop(event_loop);
    // Currently there are no means to recreate surfaces
    // destroyed on [onStop]: https://developer.android.com/reference/android/app/Activity#onStop()
    // so we simply exit and hope Android will restart this activity
    gfx_builder = gfx_builder.with_on_hide(Box::new(|| {
        use ndk_sys::quick_exit;
        unsafe {
            quick_exit(42);
        }
    }));

    #[cfg(feature = "runner")]
    {
        use std::{ffi::CString, str::FromStr as _};
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

        webrogue_wasmtime::Runtime::new(&data_dir)
            .run_builder(
                webrogue_wasmtime::GFXInitParams::new(gfx_builder),
                vfs_builder,
            )
            .unwrap();
    };

    #[cfg(feature = "launcher")]
    {
        use std::str::FromStr as _;
        use webrogue_gfx::IBuilder as _;

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
        let default_value = env.new_string("default_value").unwrap();
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

        // Keep in sync with android/runtime/launcher/src/lib.rs
        #[derive(serde::Serialize, serde::Deserialize)]
        struct LaunchIntentData {
            pub storage_path: String,
            pub sdp_offer: String,
        }

        let launch_intent_data: LaunchIntentData =
            serde_json::from_str(data.to_str().unwrap()).unwrap();

        let vm = env.get_java_vm().unwrap();
        let activity = env
            .new_global_ref(unsafe { jni::objects::JObject::from_raw(activity) })
            .unwrap();

        gfx_builder
            .run(
                move |winit_system| {
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap()
                        .block_on(async {
                            use std::sync::{Arc, Mutex};

                            use webrogue_hub_debuggee::{
                                HubDebuggeeGFX, HubDebuggeeWinitSystemGFX,
                            };

                            webrogue_hub_debuggee::HubDebuggee::new(
                                std::path::PathBuf::from_str(&launch_intent_data.storage_path)
                                    .unwrap(),
                                HubDebuggeeGFX::WinitSystem(HubDebuggeeWinitSystemGFX {
                                    gfx_system: Arc::new(Mutex::new(Some(winit_system))),
                                }),
                            )
                            .launch(
                                launch_intent_data.sdp_offer,
                                Box::new(|sdp_answer| {
                                    on_sdp_answer(sdp_answer, vm, activity);
                                }),
                            )
                            .await
                            .unwrap();
                        });
                },
                Some(true),
            )
            .unwrap();
    };
}

#[cfg(all(target_os = "android", feature = "launcher"))]
fn on_sdp_answer(sdp_answer: String, vm: jni::JavaVM, activity: jni::objects::GlobalRef) {
    let mut env = vm.attach_current_thread().unwrap();
    let intent_class = env.find_class("android/content/Intent").unwrap();
    let class_name = env.new_string("dev.webrogue.launcher.DEBUG_EVENT").unwrap();
    let intent = env
        .new_object(
            &intent_class,
            "(Ljava/lang/String;)V",
            &[(&class_name).into()],
        )
        .unwrap();

    let key = env.new_string("data").unwrap();
    let value = env.new_string(sdp_answer).unwrap();
    env.call_method(
        &intent,
        "putExtra",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[(&key).into(), (&value).into()],
    )
    .unwrap();
    let package_name = env.new_string("dev.webrogue.launcher").unwrap();
    // env.call_method(
    //     &intent,
    //     "setPackage",
    //     "(Ljava/lang/String;)Landroid/content/Intent;",
    //     &[(&package_name).into()],
    // )
    // .unwrap();
    let component_class = env
        .new_string("dev.webrogue.launcher.DebugEventBroadcastReceiver")
        .unwrap();
    let component_name = env
        .new_object(
            "android/content/ComponentName",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[(&package_name).into(), (&component_class).into()],
        )
        .unwrap();
    env.call_method(
        &intent,
        "setComponent",
        "(Landroid/content/ComponentName;)Landroid/content/Intent;",
        &[(&component_name).into()],
    )
    .unwrap();
    env.call_method(
        &activity,
        "sendBroadcast",
        "(Landroid/content/Intent;)V",
        &[(&intent).into()],
    )
    .unwrap();
}
