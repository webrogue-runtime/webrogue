use lazy_static::lazy_static;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};
use webrogue_wrapp::RealVFSBuilder;
use wry::WebView;

#[cfg(target_os = "android")]
use wry::prelude::ndk;

fn init_logging() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Trace)
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
unsafe fn android_setup(
    package: &str,
    mut env: jni::JNIEnv,
    looper: &ndk::looper::ThreadLooper,
    activity: jni::objects::GlobalRef,
) {
    use log::error;
    use std::str::FromStr;

    init_logging();

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
            WryActivity,
            android_setup, // pass the wry::android_setup function to tao which will invoke when the event loop is created
            _start_app
        );
        wry::android_binding!(dev_webrogue, launcher);
    }
}

pub fn main() {
    init_logging();
    let event_loop = EventLoop::new();

    let mut webview = None;
    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                webview = Some(build_webview(event_loop).unwrap());
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested { .. },
                ..
            } => {
                webview.take();
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}

fn build_webview(event_loop: &EventLoopWindowTarget<()>) -> anyhow::Result<WebView> {
    let window = WindowBuilder::new().build(&event_loop)?;

    let dir = ANDROID_CACHE_DIR.lock().unwrap().clone();
    Ok(webrogue_launcher::build_webview(
        &window,
        false,
        Arc::new(ServerConfigImpl {
            storage_path: dir
                .unwrap_or_else(|| std::env::temp_dir())
                .join("server_storage"),
        }),
    )?)
}

struct ServerConfigImpl {
    storage_path: std::path::PathBuf,
}

impl webrogue_launcher::ServerConfig for ServerConfigImpl {
    fn storage_path(&self) -> std::path::PathBuf {
        self.storage_path.clone()
    }

    fn run(&self, mut vfs_builder: RealVFSBuilder) -> anyhow::Result<()> {
        todo!()
    }
}
