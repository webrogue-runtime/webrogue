webrogue_events_macro::event_definitions!();

#[cfg(feature = "wasmtime")]
mod wasmtime {
    use super::*;

    webrogue_events_macro::event_wasmtime_encoders!();
}

#[cfg(feature = "web")]
mod web {
    use super::*;
    use wasm_bindgen::JsValue;

    webrogue_events_macro::event_js_encoders!();

    pub(crate) fn js_bytes(bytes: &[u8]) -> JsValue {
        js_sys::Uint8Array::from(bytes).into()
    }

    pub(crate) fn js_simple_event(tag: &str) -> JsValue {
        let object = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&object, &JsValue::from_str("tag"), &JsValue::from_str(tag));
        object.into()
    }

    pub(crate) fn js_tagged_event(tag: &str, value: JsValue) -> JsValue {
        let object = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&object, &JsValue::from_str("tag"), &JsValue::from_str(tag));
        let _ = js_sys::Reflect::set(&object, &JsValue::from_str("val"), &value);
        object.into()
    }
}
