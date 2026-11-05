use quote::quote;

use crate::util::enum_ident;

pub fn event_js_encoders() -> proc_macro2::TokenStream {
    let enums = webrogue_events::enums().into_iter().map(enum_js_encoders);
    let events_impl = events_enum_js_encoders(webrogue_events::events());
    quote! {
        #(#enums)*

        #events_impl
    }
}

fn events_enum_js_encoders(events: Vec<webrogue_events::Event>) -> proc_macro2::TokenStream {
    let arms = events.into_iter().map(|event| {
        let name = proc_macro2::Ident::new(&event.rust_name(), proc_macro2::Span::call_site());
        let tag = kebab_case(&event.rust_name());

        if event.fields.is_empty() {
            return quote! {
                Event::#name => js_simple_event(#tag),
            };
        }

        let fields = event.fields.iter().map(|field| {
            proc_macro2::Ident::new(&field.rust_name(), proc_macro2::Span::call_site())
        });
        let pushes = event.fields.iter().map(|field| {
            let field_ident =
                proc_macro2::Ident::new(&field.rust_name(), proc_macro2::Span::call_site());
            let value = js_field_value(&field.ty, &field_ident);
            quote! { values.push(&#value); }
        });

        quote! {
            Event::#name(#(#fields, )*) => {
                let values = js_sys::Array::new();
                #(#pushes)*
                js_tagged_event(#tag, values.into())
            }
        }
    });

    quote! {
        impl Event {
            pub fn to_js(self) -> wasm_bindgen::JsValue {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}

fn enum_js_encoders(r#enum: webrogue_events::Enum) -> proc_macro2::TokenStream {
    let enum_name = enum_ident(r#enum.clone());
    let arms = r#enum.cases.iter().map(|case| {
        let case_ident = proc_macro2::Ident::new(&case.rust_name(), proc_macro2::Span::call_site());
        let js_name = kebab_case(&case.rust_name());
        quote! { #enum_name::#case_ident => #js_name, }
    });

    quote! {
        impl #enum_name {
            pub(crate) fn js_name(self) -> &'static str {
                match self {
                    #(#arms)*
                }
            }
        }
    }
}

fn js_field_value(
    ty: &webrogue_events::FieldType,
    field: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    use webrogue_events::{FieldType, RawType};

    match ty {
        FieldType::Enum(_) => quote! { wasm_bindgen::JsValue::from_str(#field.js_name()) },
        FieldType::Raw(RawType::Bool) => quote! { wasm_bindgen::JsValue::from_bool(#field) },
        FieldType::Raw(_) => quote! { wasm_bindgen::JsValue::from_f64(#field as f64) },
        FieldType::Bytes(_) => quote! { js_bytes(#field.as_slice()) },
    }
}

fn kebab_case(name: &str) -> String {
    // Rust identifiers cannot represent the WIT names of function keys exactly,
    // but their kebab-case form is simply `f` followed by the number.
    if let Some(rest) = name.strip_prefix('F')
        && !rest.is_empty()
        && rest.chars().all(|character| character.is_ascii_digit())
    {
        return format!("f{rest}");
    }

    let mut out = String::new();
    let mut previous = None;
    for character in name.chars() {
        if character.is_ascii_uppercase() {
            if !out.is_empty() {
                out.push('-');
            }
            out.push(character.to_ascii_lowercase());
        } else if character.is_ascii_digit() {
            if previous.is_some_and(|previous: char| previous.is_ascii_alphanumeric()) {
                out.push('-');
            }
            out.push(character);
        } else {
            out.push(character);
        }
        previous = Some(character);
    }
    out
}
