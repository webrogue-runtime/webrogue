use quote::quote;

use crate::util::enum_ident;

pub fn event_wasmtime_encoders() -> proc_macro2::TokenStream {
    let enums = webrogue_events::enums().into_iter().map(enum_encoders);
    let events_impl = events_enum_encoders(webrogue_events::events());
    quote! {
        #(#enums)*

        #events_impl
    }
}

fn events_enum_encoders(events: Vec<webrogue_events::Event>) -> proc_macro2::TokenStream {
    fn match_case(event: webrogue_events::Event) -> proc_macro2::TokenStream {
        let name = proc_macro2::Ident::new(&event.rust_name(), proc_macro2::Span::call_site());
        if event.fields.is_empty() {
            quote! {
                Event::#name => crate::wasmtime::generated::webrogue::gfx::windowing::WindowEvent::#name
            }
        } else {
            let fields_in = event.fields.clone().into_iter().map(|field| {
                proc_macro2::Ident::new(&field.rust_name(), proc_macro2::Span::call_site())
            });
            let fields_out = event.fields.into_iter().map(|field| {
                let name =
                    proc_macro2::Ident::new(&field.rust_name(), proc_macro2::Span::call_site());
                match field.ty {
                    webrogue_events::FieldType::Enum(_) => {
                        quote! { #name.to_component_type() }
                    }
                    webrogue_events::FieldType::Raw(_) | webrogue_events::FieldType::Bytes(_) => {
                        quote! { #name }
                    }
                }
            });
            quote! {
                Event::#name(#(#fields_in, )*) => crate::wasmtime::generated::webrogue::gfx::windowing::WindowEvent::#name((#(#fields_out, )*))
            }
        }
    }

    let match_cases = events.into_iter().map(match_case);

    quote! {
        impl Event {
            pub(crate) fn to_component_type(self) -> crate::wasmtime::generated::webrogue::gfx::windowing::WindowEvent {
                match self {
                    #(#match_cases, )*
                }
            }
        }
    }
}

fn enum_encoders(r#enum: webrogue_events::Enum) -> proc_macro2::TokenStream {
    let enum_name = enum_ident(r#enum.clone());
    let match_arms = r#enum.cases.iter().map(|case| {
        let enum_ident = enum_ident(r#enum.clone());
        let case_ident = proc_macro2::Ident::new(&case.rust_name(), proc_macro2::Span::call_site());
        quote! { #enum_ident::#case_ident => crate::wasmtime::generated::webrogue::gfx::windowing::#enum_ident::#case_ident, }
    });
    quote! {
        impl #enum_name {
            pub(crate) fn to_component_type(self) -> crate::wasmtime::generated::webrogue::gfx::windowing::#enum_name {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}
