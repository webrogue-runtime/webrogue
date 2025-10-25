use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};
use wry::{WebView};

fn init_logging() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default()
            .with_min_level(log::Level::Trace)
            .with_tag("webrogue_launcher"),
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

#[no_mangle]
#[inline(never)]
pub extern "C" fn start_app() {
    #[cfg(target_os = "android")]
    {
        tao::android_binding!(
            dev_webrogue,
            launcher,
            WryActivity,
            wry::android_setup, // pass the wry::android_setup function to tao which will invoke when the event loop is created
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

    Ok(webrogue_launcher::build_webview(&window, false)?)
}
