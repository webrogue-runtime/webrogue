use lazy_static::lazy_static;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
    window::WindowBuilder,
};
use webrogue_launcher::{LauncherConfig, MailboxInternal};
use wry::WebView;

#[cfg(target_os = "android")]
use wry::prelude::ndk;

fn init_logging() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Info)
            .with_tag("webrogue_launcher_logger"),
    );
}

#[cfg(target_os = "android")]
fn stop_unwind<F: FnOnce() -> T, T>(f: F) -> T {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("attempt to unwind out of `rust` with err: {:?}", err);
            std::process::abort()
        }
    }
}

#[cfg(target_os = "android")]
fn _start_app() {
    stop_unwind(|| main());
}

lazy_static! {
    static ref ANDROID_CACHE_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
}

#[cfg(target_os = "android")]
lazy_static! {
    static ref JVM: Mutex<Option<jni::JavaVM>> = Mutex::new(None);
    static ref ACTIVITY: Mutex<Option<jni::objects::GlobalRef>> = Mutex::new(None);
}

#[cfg(target_os = "android")]
unsafe fn android_setup(
    package: &str,
    mut env: jni::JNIEnv,
    looper: &ndk::looper::ThreadLooper,
    activity: jni::objects::GlobalRef,
) {
    use log::error;
    use std::str::FromStr;

    init_logging();

    *JVM.lock().unwrap() = Some(env.get_java_vm().unwrap());
    *ACTIVITY.lock().unwrap() = Some(activity.clone());

    let path: anyhow::Result<PathBuf> = (|| {
        let file = env
            .call_method(&activity, "getCacheDir", "()Ljava/io/File;", &[])?
            .l()?;

        let path_obj = env
            .call_method(&file, "getPath", "()Ljava/lang/String;", &[])?
            .l()?
            .into();
        let path_string = env.get_string(&path_obj)?.to_str()?.to_owned();
        Ok(PathBuf::from_str(&path_string)?)
    })();

    match path {
        Ok(path) => *ANDROID_CACHE_DIR.lock().unwrap() = Some(path),
        Err(err) => {
            error!("Err {}", err);
            panic!()
        }
    };

    wry::android_setup(package, env, looper, activity)
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn start_app() {
    #[cfg(target_os = "android")]
    {
        tao::android_binding!(
            dev_webrogue,
            launcher,
            // LauncherActivity,
            WryActivity,
            android_setup,
            _start_app
        );
        wry::android_binding!(dev_webrogue, launcher);
    }
}

pub fn main() {
    init_logging();
    let event_loop = EventLoop::new();
    let event_loop_proxy = event_loop.create_proxy();

    let mut webview = None;
    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                webview = Some(build_webview(event_loop, event_loop_proxy.clone()).unwrap());
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested { .. },
                ..
            } => {
                webview.take();
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(_) => {
                if let Some(webview) = webview.as_ref() {
                    webview.1.proxy_wake_up(&webview.0);
                }
            }
            _ => (),
        }
    });
}

lazy_static! {
    static ref SDP_ANSWER_SENDER: Mutex<Option<tokio::sync::mpsc::UnboundedSender<String>>> =
        Mutex::new(None);
}

#[cfg(target_os = "android")]
#[no_mangle]
unsafe extern "C" fn Java_dev_webrogue_launcher_DebugEventBroadcastReceiver_onData<'local>(
    mut env: jni::JNIEnv<'local>,
    class: jni::objects::JClass<'local>,
    data: jni::objects::JString,
) {
    let data = env.get_string(&data).unwrap().to_str().unwrap().to_owned();
    SDP_ANSWER_SENDER
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .send(data);
}

fn build_webview(
    event_loop: &EventLoopWindowTarget<()>,
    event_loop_proxy: EventLoopProxy<()>,
) -> anyhow::Result<(WebView, TaoMailbox)> {
    let window = WindowBuilder::new().build(event_loop)?;

    let dir = ANDROID_CACHE_DIR.lock().unwrap().clone();
    Ok(webrogue_launcher::build_webview(
        &window,
        false,
        Arc::new(LauncherConfigImpl {
            storage_path: dir
                .unwrap_or_else(std::env::temp_dir)
                .join("server_storage"),
        }),
        |internal| TaoMailbox {
            event_loop_proxy,
            internal,
        },
    )?)
}

struct LauncherConfigImpl {
    storage_path: std::path::PathBuf,
}

#[async_trait::async_trait]
impl LauncherConfig for LauncherConfigImpl {
    fn storage_path(&self) -> std::path::PathBuf {
        self.storage_path.clone()
    }

    async fn launch(
        &self,
        sdp_offer: String,
        on_sdp_answer: Box<dyn FnOnce(String) + Send>,
    ) -> anyhow::Result<()> {
        #[cfg(target_os = "android")]
        {
            {
                let jvm_lock = JVM.lock().unwrap();
                let jvm = jvm_lock.as_ref().unwrap();
                let mut env = jvm.attach_current_thread().unwrap();

                let activity_lock = ACTIVITY.lock().unwrap();
                let activity = activity_lock.as_ref().unwrap().clone();

                let intent_class = env.find_class("android/content/Intent").unwrap();
                // let native_activity_class = env
                //     .find_class("dev/webrogue/launcher/LauncherActivity")
                //     .unwrap();

                let class_name =
                    env.new_string("android.app.NativeActivity".to_owned().replace('/', "."))?;
                let native_activity_class = env
                    .call_method(
                        activity.clone(),
                        "getAppClass",
                        "(Ljava/lang/String;)Ljava/lang/Class;",
                        &[(&class_name).into()],
                    )?
                    .l()
                    .unwrap();

                let intent = env
                    .new_object(
                        intent_class,
                        "(Landroid/content/Context;Ljava/lang/Class;)V",
                        &[
                            jni::objects::JValue::Object(activity.as_obj().into()),
                            (&native_activity_class).into(),
                        ],
                    )
                    .unwrap();

                // Keep in sync with android/runtime/src/lib.rs
                #[derive(serde::Serialize, serde::Deserialize)]
                struct LaunchIntentData {
                    pub storage_path: String,
                    pub sdp_offer: String,
                }

                let key = env.new_string("data").unwrap();
                let value = env
                    .new_string(
                        serde_json::to_string(&LaunchIntentData {
                            storage_path: self.storage_path.to_str().unwrap().to_owned(),
                            sdp_offer: sdp_offer,
                        })
                        .unwrap(),
                    )
                    .unwrap();
                env.call_method(
                    &intent,
                    "putExtra",
                    "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                    &[(&key).into(), (&value).into()],
                )
                .unwrap();

                env.call_method(
                    &intent,
                    "setFlags",
                    "(I)Landroid/content/Intent;",
                    &[(0x10000000 | 0x08000000).into()], // FLAG_ACTIVITY_NEW_TASK | FLAG_ACTIVITY_MULTIPLE_TASK
                )
                .unwrap();

                env.call_method(
                    &activity,
                    "startActivity",
                    "(Landroid/content/Intent;)V",
                    &[(&intent).into()],
                )
                .unwrap();
            }

            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            *SDP_ANSWER_SENDER.lock().unwrap() = Some(tx);
            let sdp_answer = rx.recv().await.unwrap();
            on_sdp_answer(sdp_answer);
        };
        Ok(())
    }
}

#[derive(Clone)]
struct TaoMailbox {
    event_loop_proxy: EventLoopProxy<()>,
    internal: MailboxInternal,
}

impl webrogue_launcher::Mailbox for TaoMailbox {
    fn wake_up(&self) {
        let _ = self.event_loop_proxy.send_event(());
    }
}

impl TaoMailbox {
    fn proxy_wake_up(&self, webview: &WebView) {
        self.internal.proxy_wake_up(webview);
    }
}
