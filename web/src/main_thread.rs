//! Code in this module is special because it run on the main thread. Be careful with atomics

use std::sync::atomic::AtomicBool;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlCanvasElement;
use webrogue_gfx_winit::{WindowRegistry, WinitProxy};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};
use winit_web::WindowAttributesWeb;

lazy_static::lazy_static! {
    static ref INITIALIZED: AtomicBool = AtomicBool::new(false);
}

#[wasm_bindgen]
pub async fn start(wasm_bindgen_shim_url: String, wrapp_url: String) {
    let initialized = INITIALIZED.fetch_or(true, std::sync::atomic::Ordering::SeqCst);
    if initialized {
        return;
    }
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let response = JsFuture::from(window.fetch_with_str(&wrapp_url))
        .await
        .unwrap()
        .dyn_into::<web_sys::Response>()
        .unwrap();

    // let persisted = JsFuture::from(window.navigator().storage().persist().unwrap())
    //     .await
    //     .unwrap();

    // if !persisted.is_truthy() {
    //     panic!("not persisted")
    // }

    let opfs_root = JsFuture::from(window.navigator().storage().get_directory())
        .await
        .unwrap()
        .dyn_into::<web_sys::FileSystemDirectoryHandle>()
        .unwrap();

    let _ = JsFuture::from(opfs_root.remove_entry("current_wrapp")).await;

    let file_handle = JsFuture::from(opfs_root.get_file_handle_with_options("current_wrapp", &{
        let options = web_sys::FileSystemGetFileOptions::new();
        options.set_create(true);
        options
    }))
    .await
    .unwrap()
    .dyn_into::<web_sys::FileSystemFileHandle>()
    .unwrap();

    let writable = JsFuture::from(file_handle.create_writable_with_options(&{
        let options = web_sys::FileSystemCreateWritableOptions::new();
        options.set_keep_existing_data(true);
        options
    }))
    .await
    .unwrap()
    .dyn_into::<web_sys::WritableStream>()
    .unwrap();

    JsFuture::from(response.body().unwrap().pipe_to(&writable))
        .await
        .unwrap();

    // Needed to initialize wasm_bindgen_shim_url stored in wasm_thread crate statically
    wasm_thread::Builder::new()
        .wasm_bindgen_shim_url(wasm_bindgen_shim_url)
        .spawn(|| {})
        .unwrap()
        .join_async()
        .await
        .unwrap();

    let event_loop = winit::event_loop::EventLoopBuilder::default()
        .build()
        .unwrap();

    event_loop.run_app(App::new()).unwrap();
}

pub struct App {
    pub proxy: Option<WinitProxy>,
    pub window_registry: WindowRegistry,
}

impl ApplicationHandler for App {
    fn can_create_surfaces(&mut self, event_loop: &dyn winit::event_loop::ActiveEventLoop) {
        let (builder, proxy) =
            webrogue_gfx_winit::ProxiedWinitBuilder::new(event_loop.create_proxy());
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
        let is_called_twice = self.proxy.replace(proxy).is_some();
        if is_called_twice {
            panic!("can_create_surfaces called twice")
        }
        let _ = wasm_thread::Builder::new()
            .name("wasi-main".to_owned())
            .spawn(|| crate::run::main(builder))
            .unwrap();
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.destroy_surfaces(event_loop);
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

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        if let Some(proxy) = self.proxy.as_ref() {
            proxy.proxy_wake_up(event_loop, &mut self.window_registry);
        }
    }
}

impl App {
    pub fn new() -> Self {
        App {
            proxy: None,
            window_registry: webrogue_gfx_winit::WindowRegistry::new(),
        }
    }
}
