use std::{cell::RefCell, collections::BTreeMap};

use wasm_bindgen::prelude::*;
use webrogue_gfx::{EventStream, Window, events::Event};

struct HostWindow {
    window: Window,
    events: EventStream,
    last_size: Option<(u32, u32)>,
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

#[wasm_bindgen(js_name = gfxMakeWindow)]
pub fn gfx_make_window() -> u32 {
    let window = system().make_window();
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
            },
        );
    });
    handle
}

#[wasm_bindgen(js_name = gfxCloseWindow)]
pub fn gfx_close_window(handle: u32) {
    WINDOWS.with(|windows| {
        windows.borrow_mut().remove(&handle);
    });
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
