use quote::quote;

pub fn enum_ident(r#enum: webrogue_events::Enum) -> proc_macro2::Ident {
    let name = r#enum.rust_name();
    proc_macro2::Ident::new(&name, proc_macro2::Span::call_site())
}

pub fn events_case(event: webrogue_events::Event) -> proc_macro2::TokenStream {
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

fn type_ident(ty: webrogue_events::FieldType) -> proc_macro2::TokenStream {
    match ty {
        webrogue_events::FieldType::Enum(r#enum) => {
            let ident = enum_ident(r#enum);
            quote!(#ident)
        }
        webrogue_events::FieldType::Raw(raw_type) => raw_ident(raw_type),
        webrogue_events::FieldType::Bytes(_) => quote!(Vec<u8>),
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
