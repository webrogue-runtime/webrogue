use quote::quote;

#[proc_macro]
pub fn event_encoders(_args: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let functions = webrogue_events::all().into_iter().map(event_encoder);
    quote!( #(#functions)* ).into()
}

fn type_ident(ty: webrogue_events::FieldType) -> proc_macro2::TokenStream {
    match ty {
        webrogue_events::FieldType::U32 => quote!(u32),
        webrogue_events::FieldType::Bool => quote!(bool),
        webrogue_events::FieldType::U8 => quote!(u8),
    }
}

fn event_encoder(event: webrogue_events::Event) -> proc_macro2::TokenStream {
    // let arg_names: Vec<_> = (0..event.fields.len())
    //     .map(|i| proc_macro2::Ident::new(&format!("arg{i}"), proc_macro2::Span::call_site()))
    //     .collect::<Vec<_>>();

    let arg_names = event
        .fields
        .iter()
        .map(|field| proc_macro2::Ident::new(field.name, proc_macro2::Span::call_site()))
        .collect::<Vec<_>>();
    let arg_tys = event
        .fields
        .iter()
        .map(|field| type_ident(field.ty))
        .collect::<Vec<_>>();
    let args = arg_names.iter().zip(arg_tys.iter()).map(|(name, ty)| {
        quote! {
            #name: #ty
        }
    });

    let func_name = proc_macro2::Ident::new(event.name, proc_macro2::Span::call_site());

    let writes = event
        .fields
        .iter()
        .zip(arg_names.iter())
        .map(|(field, arg)| {
            let offset = field.offset;
            let offset = quote!(#offset);
            match field.ty {
                webrogue_events::FieldType::U32 => {
                    quote! {
                        event_buffer[#offset.. #offset + 4].clone_from_slice(&#arg.to_le_bytes());
                    }
                }
                webrogue_events::FieldType::Bool => {
                    quote! { event_buffer[#offset] = if #arg { 1u8 } else { 0u8 }; }
                }
                webrogue_events::FieldType::U8 => quote! { event_buffer[#offset] = #arg },
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
