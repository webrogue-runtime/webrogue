use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro]
pub fn event_encoders(_args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let enums = webrogue_events::enums().into_iter().map(enum_definition);
    let functions = webrogue_events::events().into_iter().map(event_encoder);
    quote! {
        #(#enums)*
        #(#functions)*
    }
    .into()
}

fn type_ident(ty: webrogue_events::FieldType) -> proc_macro2::TokenStream {
    match ty {
        webrogue_events::FieldType::Enum(r#enum) => {
            let ident = enum_ident(r#enum);
            quote!(#ident)
        }
        webrogue_events::FieldType::Raw(raw_type) => raw_ident(raw_type),
        webrogue_events::FieldType::Bytes(len) => quote!(&[u8; #len]),
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

fn event_encoder(event: webrogue_events::Event) -> proc_macro2::TokenStream {
    // let arg_names: Vec<_> = (0..event.fields.len())
    //     .map(|i| proc_macro2::Ident::new(&format!("arg{i}"), proc_macro2::Span::call_site()))
    //     .collect::<Vec<_>>();
    let arg_names = event
        .fields
        .iter()
        .map(|field| proc_macro2::Ident::new(&field.rust_name(), proc_macro2::Span::call_site()))
        .collect::<Vec<_>>();
    let arg_tys = event
        .fields
        .iter()
        .map(|field| type_ident(field.ty.clone()))
        .collect::<Vec<_>>();
    let args = arg_names.iter().zip(arg_tys.iter()).map(|(name, ty)| {
        quote! {
            #name: #ty
        }
    });

    let func_name = proc_macro2::Ident::new(&event.rust_name(), proc_macro2::Span::call_site());

    let writes = event
        .fields
        .iter()
        .zip(arg_names.iter())
        .map(|(field, arg)| {
            let offset = field.offset;
            let offset = quote!(#offset);

            let raw_write = |raw_type, arg: TokenStream| match raw_type {
                webrogue_events::RawType::U32 => {
                    quote! {
                        event_buffer[#offset.. #offset + 4].clone_from_slice(&#arg.to_le_bytes());
                    }
                }
                webrogue_events::RawType::U16 => {
                    quote! {
                        event_buffer[#offset.. #offset + 2].clone_from_slice(&#arg.to_le_bytes());
                    }
                }
                webrogue_events::RawType::Bool => {
                    quote! { event_buffer[#offset] = if #arg { 1u8 } else { 0u8 }; }
                }
                webrogue_events::RawType::U8 => quote! { event_buffer[#offset] = #arg; },
            };

            match field.ty.clone() {
                webrogue_events::FieldType::Enum(r#enum) => raw_write(
                    r#enum.ty,
                    quote! {
                        #arg.to_raw()
                    },
                ),
                webrogue_events::FieldType::Raw(raw_type) => raw_write(raw_type, quote!(#arg)),
                webrogue_events::FieldType::Bytes(len) => quote! {
                    event_buffer[#offset.. #offset + #len].clone_from_slice(#arg);
                },
            }
        });

    // event_buf.write_all(&(1u32).to_le_bytes())

    let event_size = event.size;
    let event_size = quote!(#event_size);
    let event_id = event.id as u32;
    let event_id = quote!(#event_id);

    quote! {
        pub fn #func_name(events_buffer: &mut Vec<u8> #(, #args)*) {
            let pos = events_buffer.len();
            events_buffer.resize(pos + #event_size, 2u8);
            let event_buffer = &mut events_buffer.as_mut_slice()[pos..];
            event_buffer[0..4].clone_from_slice(&#event_id.to_le_bytes());
            #(#writes)*
        }
    }
}

fn enum_definition(r#enum: webrogue_events::Enum) -> proc_macro2::TokenStream {
    let enum_name = enum_ident(r#enum.clone());
    let raw_type_ident = raw_ident(r#enum.ty);
    let cases = r#enum.cases.iter().map(|case| {
        let ident = proc_macro2::Ident::new(&case.rust_name(), proc_macro2::Span::call_site());
        quote! { #ident, }
    });
    let match_arms = r#enum.cases.iter().map(|case| {
        let enum_ident = enum_ident(r#enum.clone());
        let value = case.value;
        let case_ident = proc_macro2::Ident::new(&case.rust_name(), proc_macro2::Span::call_site());
        let value = match r#enum.ty {
            webrogue_events::RawType::U32 => {
                let value = value as u32;
                quote! { #value }
            }
            webrogue_events::RawType::U16 => {
                let value = value as u16;
                quote! { #value }
            }
            webrogue_events::RawType::Bool => unimplemented!(),
            webrogue_events::RawType::U8 => {
                let value = value as u8;
                quote! { #value }
            }
        };
        quote! { #enum_ident::#case_ident => #value, }
    });
    quote! {
        pub enum #enum_name {
            #(#cases)*
        }

        impl #enum_name {
            fn to_raw(self) -> #raw_type_ident {
                match self {
                     #(#match_arms)*
                }
            }
        }
    }
}
