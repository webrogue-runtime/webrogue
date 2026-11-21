use std::{cell::RefCell, collections::BTreeMap};

use wasm_bindgen::prelude::*;
use webrogue_gfx::{EventStream, SurfaceKind, Window, events::Event};

struct HostWindow {
    window: Window,
    events: EventStream,
    last_size: Option<(u32, u32)>,
    /// Canvas created for this window by `can_create_surfaces`.
    webgpu_canvas_id: u32,
}

thread_local! {
    static WINDOWS: RefCell<BTreeMap<u32, HostWindow>> = const { RefCell::new(BTreeMap::new()) };
    static NEXT_HANDLE: RefCell<u32> = const { RefCell::new(1) };
}

fn system() -> webrogue_gfx::System {
    crate::SYSTEM
        .lock()
        .unwrap()
        .clone()
        .expect("webrogue gfx system is not initialized")
}

fn with_window<R>(handle: u32, f: impl FnOnce(&HostWindow) -> R) -> Option<R> {
    WINDOWS.with(|windows| windows.borrow().get(&handle).map(f))
}

fn with_window_mut<R>(handle: u32, f: impl FnOnce(&mut HostWindow) -> R) -> Option<R> {
    WINDOWS.with(|windows| windows.borrow_mut().get_mut(&handle).map(f))
}

fn parse_surface_kind(surface_kind: &str) -> Result<SurfaceKind, JsValue> {
    match surface_kind {
        "software" => Ok(SurfaceKind::Software),
        "vulkan" => Ok(SurfaceKind::Vulkan),
        "webgpu" => Ok(SurfaceKind::WebGPU),
        other => Err(JsValue::from_str(&format!("unknown surface kind: {other}"))),
    }
}

#[wasm_bindgen(js_name = gfxMakeWindow)]
pub fn gfx_make_window(surface_kind: &str) -> Result<u32, JsValue> {
    let surface_kind = parse_surface_kind(surface_kind)?;
    let window = system()
        .make_window(surface_kind)
        .map_err(|error| JsValue::from_str(&format!("{error:#}")))?;
    let webgpu_canvas_id = window.webgpu_canvas_id();
    let events = window.get_event_stream();
    let handle = NEXT_HANDLE.with(|next| {
        let mut next = next.borrow_mut();
        let handle = *next;
        *next += 1;
        handle
    });
    WINDOWS.with(|windows| {
        windows.borrow_mut().insert(
            handle,
            HostWindow {
                window,
                events,
                last_size: None,
                webgpu_canvas_id,
            },
        );
    });
    Ok(handle)
}

#[wasm_bindgen(js_name = gfxDropWindow)]
pub fn gfx_drop_window(handle: u32) {
    let canvas_id = WINDOWS.with(|windows| {
        windows
            .borrow_mut()
            .remove(&handle)
            .map(|host| host.webgpu_canvas_id)
    });
    if let Some(canvas_id) = canvas_id {
        // The canvas lives on the main thread; remove it there so that a later
        // window can create a fresh canvas with a different context type.
        let Ok(port) = wasm_thread::worker_payload().dyn_into::<web_sys::MessagePort>() else {
            return;
        };
        let message = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &message,
            &JsValue::from_str("kind"),
            &JsValue::from_str("drop"),
        );
        let _ = js_sys::Reflect::set(
            &message,
            &JsValue::from_str("canvas"),
            &JsValue::from(canvas_id),
        );
        let _ = port.post_message(&message);
    }
}

#[wasm_bindgen(js_name = gfxWindowWebgpuCanvasId)]
pub fn gfx_window_webgpu_canvas_id(handle: u32) -> u32 {
    with_window(handle, |host| host.webgpu_canvas_id).unwrap_or(u32::MAX)
}

#[wasm_bindgen(js_name = gfxWindowPhysicalSize)]
pub fn gfx_window_physical_size(handle: u32) -> Vec<u32> {
    with_window(handle, |host| {
        let (width, height) = host.window.get_physical_size();
        vec![width, height]
    })
    .unwrap_or_else(|| vec![0, 0])
}

#[wasm_bindgen(js_name = gfxWindowLogicalSize)]
pub fn gfx_window_logical_size(handle: u32) -> Vec<u32> {
    with_window(handle, |host| {
        let (width, height) = host.window.get_logical_size();
        vec![width, height]
    })
    .unwrap_or_else(|| vec![0, 0])
}

#[wasm_bindgen(js_name = gfxWindowVulkanId)]
pub fn gfx_window_vulkan_id(handle: u32) -> u32 {
    with_window(handle, |host| host.window.get_vk_id().unwrap_or(0)).unwrap_or(0)
}

#[wasm_bindgen(js_name = gfxWindowSetMouseGrab)]
pub fn gfx_window_set_mouse_grab(handle: u32, enabled: bool) -> bool {
    with_window(handle, |host| host.window.set_mouse_grab(enabled)).unwrap_or(false)
}

#[wasm_bindgen(js_name = gfxPresentPixels)]
pub fn gfx_present_pixels(handle: u32, pixels: &[u32]) {
    with_window(handle, |host| {
        if let Err(error) = host.window.present_pixels(pixels) {
            web_sys::console::error_1(&JsValue::from_str(&format!(
                "failed to present pixels: {error:#}"
            )));
        }
    });
}

#[wasm_bindgen(js_name = gfxWindowNextEvent)]
pub async fn gfx_window_next_event(handle: u32) -> Option<JsValue> {
    loop {
        let event = with_window_mut(handle, |host| {
            if let Some(event) = host.events.try_recv() {
                return Some(event);
            }

            let size = host.window.get_physical_size();
            if size.0 != 0 && size.1 != 0 && Some(size) != host.last_size {
                host.last_size = Some(size);
                return Some(Event::Resized);
            }

            None
        });
        match event {
            Some(Some(event)) => return Some(event.to_js()),
            Some(None) => {}
            // The window was closed.
            None => return None,
        }
        crate::js_sleep(8).await;
    }
}
