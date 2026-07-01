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
        use jni::objects::JString;
        use std::str::FromStr as _;
        use webrogue_gfx::IBuilder as _;

        let activity = app.activity_as_ptr() as jni::sys::jobject;

        let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr() as *mut jni::sys::JavaVM) };
        let mut scope = jni::ScopeToken::default();
        let mut env = unsafe {
            vm.attach_current_thread_guard(|| jni::vm::AttachConfig::new(), &mut scope)
                .unwrap()
        };

        let activity = unsafe { jni::objects::JObject::from_raw(env.borrow_env_mut(), activity) };

        let intent = env
            .borrow_env_mut()
            .call_method(
                &activity,
                jni::jni_str!("getIntent"),
                jni::jni_sig!(() -> android.content.Intent),
                &[],
            )
            .unwrap()
            .l()
            .unwrap();

        let extra = env
            .borrow_env_mut()
            .call_method(
                intent,
                jni::jni_str!("getExtras"),
                jni::jni_sig!(() -> android.os.Bundle),
                &[],
            )
            .unwrap()
            .l()
            .unwrap();

        let key = env.borrow_env_mut().new_string("data").unwrap();
        let default_value = env.borrow_env_mut().new_string("default_value").unwrap();
        let data_object = env
            .borrow_env_mut()
            .call_method(
                extra,
                jni::jni_str!("getString"),
                jni::jni_sig!((key: JString, default_value: JString) -> JString),
                &[(&key).into(), (&default_value).into()],
            )
            .unwrap()
            .l()
            .unwrap();

        let data = JString::cast_local(env.borrow_env_mut(), data_object).unwrap();

        // Keep in sync with android/runtime/launcher/src/lib.rs
        #[derive(serde::Serialize, serde::Deserialize)]
        struct LaunchIntentData {
            pub storage_path: String,
            pub sdp_offer: String,
        }

        let launch_intent_data: LaunchIntentData =
            serde_json::from_str(&data.try_to_string(env.borrow_env_mut()).unwrap()).unwrap();

        let vm = env.borrow_env_mut().get_java_vm().unwrap();
        let activity = env.borrow_env_mut().new_global_ref(activity).unwrap();

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
                false,
            )
            .unwrap();
    };
}

#[cfg(all(target_os = "android", feature = "launcher"))]
fn on_sdp_answer(
    sdp_answer: String,
    vm: jni::JavaVM,
    activity: jni::objects::Global<jni::objects::JObject<'_>>,
) {
    let mut scope = jni::ScopeToken::default();
    let mut env = unsafe {
        vm.attach_current_thread_guard(|| jni::vm::AttachConfig::new(), &mut scope)
            .unwrap()
    };
    let intent_class = env
        .borrow_env_mut()
        .find_class(jni::jni_str!("android/content/Intent"))
        .unwrap();
    let class_name = env
        .borrow_env_mut()
        .new_string("dev.webrogue.launcher.DEBUG_EVENT")
        .unwrap();
    let intent = env
        .borrow_env_mut()
        .new_object(
            &intent_class,
            jni::jni_sig!((className: JString)),
            &[(&class_name).into()],
        )
        .unwrap();

    let key = env.borrow_env_mut().new_string("data").unwrap();
    let value = env.borrow_env_mut().new_string(sdp_answer).unwrap();
    env.borrow_env_mut()
        .call_method(
            &intent,
            jni::jni_str!("putExtra"),
            jni::jni_sig!((key: JString, value: JString) -> android.content.Intent),
            &[(&key).into(), (&value).into()],
        )
        .unwrap();
    let package_name = env
        .borrow_env_mut()
        .new_string("dev.webrogue.launcher")
        .unwrap();
    // env.borrow_env_mut().call_method(
    //     &intent,
    //     "setPackage",
    //     "(Ljava/lang/String;)Landroid/content/Intent;",
    //     &[(&package_name).into()],
    // )
    // .unwrap();
    let component_class = env
        .borrow_env_mut()
        .new_string("dev.webrogue.launcher.DebugEventBroadcastReceiver")
        .unwrap();
    let component_name = env
        .borrow_env_mut()
        .new_object(
            jni::jni_str!("android/content/ComponentName"),
            jni::jni_sig!((packageName: JString, componentClass: JString)),
            &[(&package_name).into(), (&component_class).into()],
        )
        .unwrap();
    env.borrow_env_mut()
        .call_method(
            &intent,
            jni::jni_str!("setComponent"),
            jni::jni_sig!((componentName: android.content.ComponentName) -> android.content.Intent),
            &[(&component_name).into()],
        )
        .unwrap();
    env.borrow_env_mut()
        .call_method(
            &activity,
            jni::jni_str!("sendBroadcast"),
            jni::jni_sig!((intent: android.content.Intent)),
            &[(&intent).into()],
        )
        .unwrap();
}
