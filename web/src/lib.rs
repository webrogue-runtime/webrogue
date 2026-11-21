mod host;
mod vfs_host;
mod wrapp;

use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use wasm_bindgen::{JsCast, closure::Closure, prelude::*};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Element, HtmlCanvasElement, ImageBitmap, ImageBitmapRenderingContext, MessageEvent, MessagePort,
};
use webrogue_gfx::System;
use webrogue_gfx_winit::{ProxiedWinitBuilder, WindowRegistry, WinitProxy};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::WindowId,
};
use winit_web::WindowAttributesWeb;

use crate::wrapp::SharedArchive;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Size used for a canvas whose container has not been laid out yet.
const DEFAULT_CANVAS_WIDTH: u32 = 960;
const DEFAULT_CANVAS_HEIGHT: u32 = 720;

pub(crate) static SYSTEM: Mutex<Option<System>> = Mutex::new(None);

static ARCHIVE: Mutex<Option<SharedArchive>> = Mutex::new(None);

thread_local! {
    /// Canvases of live windows, keyed by the id stored in the corresponding
    /// `HostWindow`.
    ///
    /// Each window needs its own canvas: a canvas can only have one kind of
    /// rendering context, and software windows use a 2D context while WebGPU
    /// windows use a `bitmaprenderer` context.
    static CANVASES: RefCell<BTreeMap<u32, HtmlCanvasElement>> = RefCell::new(BTreeMap::new());
    /// Main-thread end of the channel that the guest worker uses to send
    /// WebGPU frames and canvas lifecycle messages.
    static WEBGPU_PORT: RefCell<Option<MessagePort>> = const { RefCell::new(None) };
}

fn message_canvas_id(data: &JsValue) -> Option<u32> {
    js_sys::Reflect::get(data, &JsValue::from_str("canvas"))
        .ok()?
        .as_f64()
        .map(|id| id as u32)
}

fn message_kind(data: &JsValue) -> Option<String> {
    js_sys::Reflect::get(data, &JsValue::from_str("kind"))
        .ok()?
        .as_string()
}

/// Receives frames and canvas lifecycle messages from the guest worker.
fn install_webgpu_port(port: &MessagePort) {
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(|event: MessageEvent| {
        let data = event.data();
        let Some(kind) = message_kind(&data) else {
            return;
        };
        match kind.as_str() {
            // The guest finished rendering a frame. Show it on the window canvas.
            "frame" => {
                let Some(canvas_id) = message_canvas_id(&data) else {
                    return;
                };
                let Ok(bitmap) = js_sys::Reflect::get(&data, &JsValue::from_str("bitmap")) else {
                    return;
                };
                if !bitmap.is_instance_of::<ImageBitmap>() {
                    return;
                }
                let bitmap: ImageBitmap = bitmap.unchecked_into();
                CANVASES.with(|canvases| {
                    let canvases = canvases.borrow();
                    let Some(canvas) = canvases.get(&canvas_id) else {
                        return;
                    };
                    // `transferFromImageBitmap` replaces the canvas bitmap but
                    // keeps the `width`/`height` attributes, so the canvas
                    // would clip (or stretch) frames rendered at another size.
                    // Keep the backing store in sync with the guest instead.
                    // The CSS size stays independent: the canvas fills its
                    // container.
                    let (width, height) = (bitmap.width(), bitmap.height());
                    if canvas.width() != width || canvas.height() != height {
                        canvas.set_width(width);
                        canvas.set_height(height);
                    }
                    let Ok(Some(context)) = canvas.get_context("bitmaprenderer") else {
                        return;
                    };
                    let Ok(context) = context.dyn_into::<ImageBitmapRenderingContext>() else {
                        return;
                    };
                    context.transfer_from_image_bitmap(&bitmap);
                });
            }
            // The window was closed. Drop its canvas so a later window gets a
            // fresh element that can use a different kind of context.
            "drop" => {
                let Some(canvas_id) = message_canvas_id(&data) else {
                    return;
                };
                CANVASES.with(|canvases| {
                    let canvas = canvases.borrow_mut().remove(&canvas_id);
                    if let Some(canvas) = canvas
                        && let Some(parent) = canvas.parent_node()
                    {
                        let _ = parent.remove_child(&canvas);
                    }
                });
            }
            _ => {}
        }
    });
    port.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
}

/// Returns the message port connected to the main thread, for the guest worker.
///
/// It is created by the main thread and transferred to the worker as its
/// payload (see `wasm_thread::Builder::spawn_async`).
#[wasm_bindgen(js_name = workerPayload)]
pub fn worker_payload() -> JsValue {
    wasm_thread::worker_payload()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = webrogueWorker, js_name = runGuest)]
    fn run_guest(bytes: &[u8]) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = webrogueWorker, js_name = sleep)]
    async fn sleep(ms: u32);
}

pub(crate) async fn js_sleep(ms: u32) {
    sleep(ms).await;
}

#[wasm_bindgen]
pub async fn start(wrapp_url: String, worker_script_url: String) -> Result<(), JsValue> {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    console_error_panic_hook::set_once();

    let bytes = fetch_bytes(&wrapp_url).await?;
    let archive =
        wrapp::Archive::open(bytes).map_err(|error| JsValue::from_str(&format!("{error:#}")))?;
    *ARCHIVE.lock().unwrap() = Some(Arc::new(archive));

    let event_loop = winit::event_loop::EventLoopBuilder::default()
        .build()
        .map_err(|error| JsValue::from_str(&format!("{error}")))?;
    event_loop
        .run_app(App::new(worker_script_url))
        .map_err(|error| JsValue::from_str(&format!("{error}")))?;

    Ok(())
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no browser window"))?;
    let response = JsFuture::from(window.fetch_with_str(url)).await?;
    let response: web_sys::Response = response.dyn_into()?;
    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "unable to fetch {url} (HTTP {})",
            response.status()
        )));
    }
    let buffer = JsFuture::from(response.array_buffer()?).await?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

struct App {
    proxy: Option<WinitProxy>,
    window_registry: WindowRegistry,
    worker_script_url: String,
}

impl App {
    fn new(worker_script_url: String) -> Self {
        Self {
            proxy: None,
            window_registry: WindowRegistry::new(),
            worker_script_url,
        }
    }
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        let (builder, proxy) = ProxiedWinitBuilder::new(event_loop.create_proxy());
        let builder = builder.with_window_attributes(|window_attributes| {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            // Every window gets its own canvas. A canvas has a single kind of
            // rendering context, so sharing one between a software window (2D)
            // and a WebGPU window (`bitmaprenderer`) would not work.
            //
            // The id is owned by the window (see `WinitWindow`), so a native
            // window that is recreated replaces its canvas under the same id
            // and the guest's cached id stays valid.
            let canvas_id = webrogue_gfx_winit::CURRENT_CANVAS_ID.load(Ordering::Relaxed);
            let canvas = document
                .create_element("canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();
            canvas.set_id(&format!("webrogue-canvas-{canvas_id}"));
            canvas.set_class_name("webrogue-canvas");

            let container = document.get_element_by_id("webrogue-canvases");
            let parent = container
                .clone()
                .or_else(|| document.body().map(|body| body.unchecked_into::<Element>()))
                .unwrap();

            // The canvas fills its container instead of using a fixed size.
            // The web backend derives the window size from the canvas' client
            // size, so this is what makes the guest follow layout changes.
            //
            // `height: 100%` only resolves when the container has a height of
            // its own. Otherwise it falls back to `auto`, and a canvas with
            // `height: auto` keeps its intrinsic aspect ratio (the ratio of its
            // `width`/`height` attributes), which would freeze the aspect ratio
            // of the first frame. Use the viewport height in that case.
            let has_own_height = container.is_some() && parent.client_height() > 0;
            let (initial_width, initial_height) = if has_own_height {
                (
                    u32::try_from(parent.client_width()).unwrap_or(0),
                    u32::try_from(parent.client_height()).unwrap_or(0),
                )
            } else {
                let viewport_width = window
                    .inner_width()
                    .ok()
                    .and_then(|value| value.as_f64())
                    .unwrap_or(0.0);
                let viewport_height = window
                    .inner_height()
                    .ok()
                    .and_then(|value| value.as_f64())
                    .unwrap_or(0.0);
                (viewport_width as u32, viewport_height as u32)
            };
            canvas.set_width(if initial_width == 0 {
                DEFAULT_CANVAS_WIDTH
            } else {
                initial_width
            });
            canvas.set_height(if initial_height == 0 {
                DEFAULT_CANVAS_HEIGHT
            } else {
                initial_height
            });
            let style = canvas.style();
            let _ = style.set_property("display", "block");
            let _ = style.set_property("width", "100%");
            let _ = style.set_property("height", if has_own_height { "100%" } else { "100svh" });

            parent.append_child(&canvas).unwrap();

            CANVASES.with(|canvases| {
                let previous = canvases.borrow_mut().insert(canvas_id, canvas.clone());
                // Drop the canvas of a previous incarnation of this window.
                if let Some(previous) = previous
                    && let Some(parent) = previous.parent_node()
                {
                    let _ = parent.remove_child(&previous);
                }
            });

            // canvas.transfer_control_to_offscreen().unwrap();

            window_attributes.with_platform_attributes(Box::new(
                WindowAttributesWeb::default().with_canvas(Some(canvas)),
            ))
        });

        if self.proxy.replace(proxy.clone()).is_some() {
            panic!("can_create_surfaces called twice");
        }

        proxy.can_create_surfaces(event_loop, &mut self.window_registry);

        let system = builder.to_system(Some(false)).unwrap();
        *SYSTEM.lock().unwrap() = Some(system);

        let message_channel = web_sys::MessageChannel::new().unwrap();
        let main_port = message_channel.port1();
        install_webgpu_port(&main_port);
        WEBGPU_PORT.with(|slot| *slot.borrow_mut() = Some(main_port));

        // {

        // }
        // message_channel.port1().post_message_with_transferable(message, transferable)

        #[cfg(target_arch = "wasm32")]
        match wasm_thread::Builder::new()
            .worker_script_url(self.worker_script_url.clone())
            .spawn_async(
                move || async move {
                    let message_port = wasm_thread::worker_payload()
                        .dyn_into::<web_sys::MessagePort>()
                        .unwrap();
                    // message_port.add_event_listener_with_callback(type_, listener);
                    let Some(bytes) = ARCHIVE
                        .lock()
                        .unwrap()
                        .as_ref()
                        .and_then(|archive| archive.main_wasm())
                    else {
                        web_sys::console::error_1(&JsValue::from_str(
                            "WRAPP archive does not contain main.wasm",
                        ));
                        return;
                    };

                    let bytes = match webrogue_wasm_processing::compose(bytes) {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            web_sys::console::error_1(&JsValue::from_str(&format!(
                                "unable to compose the guest component: {error:#}"
                            )));
                            return;
                        }
                    };

                    if let Err(error) = JsFuture::from(run_guest(&bytes)).await {
                        web_sys::console::error_1(&JsValue::from_str("guest stopped:"));
                        web_sys::console::error_1(&error);
                    }
                    drop(message_port);
                },
                Some(
                    wasm_bindgen::prelude::Upcast::<JsValue>::upcast(&message_channel.port2())
                        .clone(),
                ),
            ) {
            Err(error) => {
                web_sys::console::error_1(&JsValue::from_str(&format!(
                    "unable to spawn the guest worker: {error}"
                )));
            }
            Ok(_) => {}
        }
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.destroy_surfaces(event_loop, &mut self.window_registry);
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.window_event(&mut self.window_registry, window_id, event);
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &dyn ActiveEventLoop,
        _device_id: Option<DeviceId>,
        event: DeviceEvent,
    ) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.device_event(&mut self.window_registry, event);
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.proxy_wake_up(event_loop, &mut self.window_registry);
        }
    }
}
