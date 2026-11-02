use quote::quote;

use crate::util::{enum_ident, events_case};

pub fn event_definitions() -> proc_macro2::TokenStream {
    let enums = webrogue_events::enums().into_iter().map(enum_definition);
    let events_enum = events_enum(webrogue_events::events());
    quote! {
        #(#enums)*

        #events_enum
    }
}

fn enum_definition(r#enum: webrogue_events::Enum) -> proc_macro2::TokenStream {
    let enum_name = enum_ident(r#enum.clone());
    let cases = r#enum.cases.iter().map(|case| {
        let ident = proc_macro2::Ident::new(&case.rust_name(), proc_macro2::Span::call_site());
        quote! { #ident, }
    });
    quote! {
        #[derive(Debug, PartialEq, Clone)]
        pub enum #enum_name {
            #(#cases)*
        }
    }
}

fn events_enum(events: Vec<webrogue_events::Event>) -> proc_macro2::TokenStream {
    let cases = events.into_iter().map(events_case);

    quote! {
        #[derive(Clone)]
        pub enum Event {
            #(#cases)*
        }
    }
}
