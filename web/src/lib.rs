#![feature(mpmc_channel)]

use std::sync::atomic::AtomicBool;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use webrogue_gfx::{IBuilder, ISystem, IWindow};
mod app;
mod reader;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_value(x: &JsValue);
}

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
    log("log from main thread");

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

    JsFuture::from(opfs_root.remove_entry("current_wrapp")).await;

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
        .spawn(|| {
            log("log 2 from worker");
        })
        .unwrap()
        .join_async()
        .await
        .unwrap();

    let event_loop = winit::event_loop::EventLoopBuilder::default()
        .build()
        .unwrap();

    event_loop.run_app(app::App::new()).unwrap();

    // let builder = webrogue_gfx_winit::SimpleWinitBuilder::with_default_event_loop().unwrap();

    // builder
    //     .run(
    //         |system| {
    //             let window = system.make_window();
    //             let gl_size = window.get_gl_size();
    //             log(&format!("gl_size: {}x{}", gl_size.0, gl_size.1));
    //         },
    //         Some(false),
    //     )
    //     .unwrap();
}
