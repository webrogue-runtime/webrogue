use quote::quote;

#[proc_macro]
pub fn event_encoders(_args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let enums = webrogue_events::enums().into_iter().map(enum_definition);
    let events_enum = events_enum(webrogue_events::events());
    quote! {
        #(#enums)*

        #events_enum
    }
    .into()
}

fn events_enum(events: Vec<webrogue_events::Event>) -> proc_macro2::TokenStream {
    fn events_case(event: webrogue_events::Event) -> proc_macro2::TokenStream {
        let name = proc_macro2::Ident::new(&event.rust_name(), proc_macro2::Span::call_site());
        if event.fields.is_empty() {
            quote! {
                #name,
            }
        } else {
            let fields = event.fields.into_iter().map(|field| type_ident(field.ty));
            quote! {
                #name(#(#fields, )*),
            }
        }
    }

    let cases = events.clone().into_iter().map(events_case);

    fn match_case(event: webrogue_events::Event) -> proc_macro2::TokenStream {
        let name = proc_macro2::Ident::new(&event.rust_name(), proc_macro2::Span::call_site());
        if event.fields.is_empty() {
            quote! {
                Event::#name => crate::interface::generated::webrogue::gfx::windowing::WindowEvent::#name
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
                Event::#name(#(#fields_in, )*) => crate::interface::generated::webrogue::gfx::windowing::WindowEvent::#name((#(#fields_out, )*))
            }
        }
    }

    let match_cases = events.into_iter().map(match_case);

    quote! {
        #[derive(Clone)]
        pub enum Event {
            #(#cases)*
        }

        impl Event {
            pub(crate) fn to_component_type(self) -> crate::interface::generated::webrogue::gfx::windowing::WindowEvent {
                match self {
                    #(#match_cases, )*
                }
            }
        }
    }
}

fn type_ident(ty: webrogue_events::FieldType) -> proc_macro2::TokenStream {
    match ty {
        webrogue_events::FieldType::Enum(r#enum) => {
            let ident = enum_ident(r#enum);
            quote!(#ident)
        }
        webrogue_events::FieldType::Raw(raw_type) => raw_ident(raw_type),
        webrogue_events::FieldType::Bytes(len) => quote!(Vec<u8>),
    }
}

fn raw_ident(raw_type: webrogue_events::RawType) -> proc_macro2::TokenStream {
    match raw_type {
        webrogue_events::RawType::U32 => quote!(u32),
        webrogue_events::RawType::U16 => quote!(u16),
        webrogue_events::RawType::Bool => quote!(bool),
        webrogue_events::RawType::U8 => quote!(u8),
    }
}

fn enum_ident(r#enum: webrogue_events::Enum) -> proc_macro2::Ident {
    let name = r#enum.rust_name();
    proc_macro2::Ident::new(&name, proc_macro2::Span::call_site())
}

fn enum_definition(r#enum: webrogue_events::Enum) -> proc_macro2::TokenStream {
    let enum_name = enum_ident(r#enum.clone());
    let cases = r#enum.cases.iter().map(|case| {
        let ident = proc_macro2::Ident::new(&case.rust_name(), proc_macro2::Span::call_site());
        quote! { #ident, }
    });
    let match_arms = r#enum.cases.iter().map(|case| {
        let enum_ident = enum_ident(r#enum.clone());
        let case_ident = proc_macro2::Ident::new(&case.rust_name(), proc_macro2::Span::call_site());
        quote! { #enum_ident::#case_ident => crate::interface::generated::webrogue::gfx::windowing::#enum_ident::#case_ident, }
    });
    quote! {
        #[derive(PartialEq, Clone)]
        pub enum #enum_name {
            #(#cases)*
        }

        impl #enum_name {
            pub(crate) fn to_component_type(self) -> crate::interface::generated::webrogue::gfx::windowing::#enum_name {
                match self {
                    #(#match_arms)*
                }
            }
        }
    }
}
