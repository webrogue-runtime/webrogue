mod definition;
mod util;
mod wasmtime;
mod web;

#[proc_macro]
pub fn event_definitions(_args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    definition::event_definitions().into()
}

#[proc_macro]
pub fn event_wasmtime_encoders(_args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    wasmtime::event_wasmtime_encoders().into()
}

#[proc_macro]
pub fn event_js_encoders(_args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    web::event_js_encoders().into()
}
