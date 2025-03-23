use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::HashSet;
use std::str::FromStr;
use wiggle_generate::config::Asyncness;
use wiggle_generate::names;
use wiggle_generate::CodegenSettings;

pub fn link_module(
    module: &witx::Module,
    target_path: Option<&syn::Path>,
    settings: &CodegenSettings,
) -> proc_macro2::TokenStream {
    let module_ident = names::module(&module.name);

    let send_bound = if settings.async_.contains_async(module) {
        quote! { + Send, T: Send }
    } else {
        quote! {}
    };

    let mut bodies = Vec::new();
    let mut bounds = HashSet::new();
    for f in module.funcs() {
        let asyncness = settings.async_.get(module.name.as_str(), f.name.as_str());
        bodies.push(generate_func(&module, &f, target_path, asyncness));
        let bound = wiggle_generate::func_bounds(module, &f, settings);
        for b in bound {
            bounds.insert(b);
        }
    }

    let ctx_bound = if let Some(target_path) = target_path {
        let bounds = bounds
            .into_iter()
            .map(|b| quote!(#target_path::#module_ident::#b));
        quote!( #(#bounds)+* #send_bound )
    } else {
        let bounds = bounds.into_iter();
        quote!( #(#bounds)+* #send_bound )
    };

    let func_name = quote::format_ident!("add_{}_to_linker", module_ident);

    let u = if settings.mutable {
        quote!(&mut U)
    } else {
        quote!(&U)
    };
    quote! {
        pub fn #func_name<U>(
            imports: &mut imports::Imports,
            get_cx: impl Fn(&mut context::Store) -> #u + Send + Sync + Copy + 'static,
        )
            where
                U: #ctx_bound #send_bound
        {
            #(#bodies)*
        }
    }
    .into()
}

fn generate_func(
    module: &witx::Module,
    func: &witx::InterfaceFunc,
    target_path: Option<&syn::Path>,
    asyncness: Asyncness,
) -> proc_macro2::TokenStream {
    let module_str = module.name.as_str();
    let module_ident = names::module(&module.name);

    let field_str = func.name.as_str();
    let field_ident = names::func(&func.name);

    let (params, results) = func.wasm_signature();

    let arg_names = (0..params.len())
        .map(|i| Ident::new(&format!("arg{i}"), Span::call_site()))
        .collect::<Vec<_>>();
    let arg_tys = params
        .iter()
        .map(|ty| names::wasm_type(*ty))
        .collect::<Vec<_>>();
    let arg_decls = arg_names
        .iter()
        .zip(arg_tys.iter())
        .enumerate()
        .map(|(i, (name, ty))| {
            let num = proc_macro2::TokenStream::from_str(&format!("{}", i)).unwrap();
            quote! {
                let #name = ArgGetter::<#ty>::get(#num);
            }
        })
        .collect::<Vec<_>>();

    let ret_ty = match results.len() {
        0 => quote!(()),
        1 => names::wasm_type(results[0]),
        _ => unimplemented!(),
    };

    let await_ = if asyncness.is_sync() {
        quote!()
    } else {
        quote!(.await)
    };

    let abi_func = if let Some(target_path) = target_path {
        quote!( #target_path::#module_ident::#field_ident )
    } else {
        quote!( #field_ident )
    };

    let body = quote! {
        let mut memory = wiggle::GuestMemory::Dynamic(Box::new(memory::DynamicMemory {}));
        let ret = #abi_func(u, &mut memory #(, #arg_names)*)#await_.unwrap();
        RetSetter::<#ret_ty>::set(ret);
    };

    match asyncness {
        Asyncness::Async => {
            todo!()
        }

        Asyncness::Blocking { block_with } => {
            quote! {
                imports.add_fn(
                    #module_str,
                    #field_str,
                    Box::new(move |store| {
                        let u = get_cx(store);
                        #(#arg_decls)*
                        #block_with(async {#body}).unwrap();
                    })
                );
            }
        }

        Asyncness::Sync => {
            quote! {
                imports.add_fn(
                    #module_str,
                    #field_str,
                    Box::new(move |store| {
                        let u = get_cx(store);
                        #(#arg_decls)*
                        #body
                    })
                );
            }
        }
    }
}
