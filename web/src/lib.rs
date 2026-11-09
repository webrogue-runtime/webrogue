mod host;
mod vfs_host;
mod wrapp;

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlCanvasElement;
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

pub(crate) static SYSTEM: Mutex<Option<System>> = Mutex::new(None);

static ARCHIVE: Mutex<Option<SharedArchive>> = Mutex::new(None);

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
            let canvas = document
                .get_element_by_id("webrogue-runtime-canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();

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

        #[cfg(target_arch = "wasm32")]
        match wasm_thread::Builder::new()
            .worker_script_url(self.worker_script_url.clone())
            .spawn_async(move || async move {
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
            }) {
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
