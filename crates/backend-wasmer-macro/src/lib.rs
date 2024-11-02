use proc_macro::TokenStream;

#[proc_macro]
pub fn make_funcs(item: TokenStream) -> TokenStream {
    let imports = webrogue_macro_common::parse_macro_input!(item as webrogue_macro_common::Imports);

    let mut result = format!(
        "
{}fn make_imports() -> webrogue_backend_wasmer::Imports {{
    webrogue_backend_wasmer::Imports {{
        f: Box::new(|
            import_object: &mut webrogue_backend_wasmer::wasmer::Imports, 
            store: &mut webrogue_backend_wasmer::wasmer::Store, 
            env: webrogue_backend_wasmer::wasmer::FunctionEnv<webrogue_backend_wasmer::Env>
        | {{
    ",
        if imports.is_public { "pub " } else { "" }
    );

    let mut fn_counter = 0;

    for (i, imported_module) in imports.modules.iter().enumerate() {
        result += &format!("{}\n{{\n", imported_module.attributes.join("\n"));
        for import in imported_module.funcs.clone() {
            let static_func_name = format!("imported_fn_{}", fn_counter);
            fn_counter += 1;
            let in_args = import
                .args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    let ty = match arg {
                        webrogue_macro_common::ValueType::U32 => "i32",
                        webrogue_macro_common::ValueType::U64 => "i64",
                        webrogue_macro_common::ValueType::I32 => "i32",
                        webrogue_macro_common::ValueType::I64 => "i64",
                        webrogue_macro_common::ValueType::F32 => "f32",
                        webrogue_macro_common::ValueType::F64 => "f64",
                    };
                    format!(", arg{}: {}", i, ty)
                })
                .collect::<Vec<_>>()
                .join("");

            let out_args = import
                .args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    let conv = match arg {
                        webrogue_macro_common::ValueType::U32 => " as u32",
                        webrogue_macro_common::ValueType::U64 => " as u64",
                        webrogue_macro_common::ValueType::I32 => "",
                        webrogue_macro_common::ValueType::I64 => "",
                        webrogue_macro_common::ValueType::F32 => "",
                        webrogue_macro_common::ValueType::F64 => "",
                    };
                    format!("arg{}{},\n", i, conv)
                })
                .collect::<Vec<_>>()
                .join("");

            let out_ty = match import.ret.clone() {
                Some(ty) => {
                    format!(
                        " -> {}",
                        match ty {
                            webrogue_macro_common::ValueType::U32 => "i32",
                            webrogue_macro_common::ValueType::U64 => "i64",
                            webrogue_macro_common::ValueType::I32 => "i32",
                            webrogue_macro_common::ValueType::I64 => "i64",
                            webrogue_macro_common::ValueType::F32 => "f32",
                            webrogue_macro_common::ValueType::F64 => "f64",
                        }
                    )
                }
                None => "".to_string(),
            };

            let out_conv = match import.ret {
                Some(webrogue_macro_common::ValueType::U32) => " as i32",
                Some(webrogue_macro_common::ValueType::U64) => " as i64",
                Some(webrogue_macro_common::ValueType::I32) => "",
                Some(webrogue_macro_common::ValueType::I64) => "",
                Some(webrogue_macro_common::ValueType::F32) => "",
                Some(webrogue_macro_common::ValueType::F64) => "",
                _ => "",
            };

            result += &format!(
                "
fn {}(mut env: webrogue_backend_wasmer::wasmer::FunctionEnvMut<webrogue_backend_wasmer::Env>{}){} {{
    let mut context = env.data_mut().context.lock().unwrap();
    let ptr = context
        .context_vec
        .get_raw::<{}::Context>({});
    {}::{}(
        &mut context.memory_factory,
        unsafe {{ &mut *ptr }},
        {}
    ){}
}}

import_object.define(
    \"{}\",
    \"{}\",
    webrogue_backend_wasmer::wasmer::Function::new_typed_with_env(
        store,
        &env,
        {},
    ),
);
    ",
                static_func_name,
                in_args,
                out_ty,
                imported_module.rust_module,
                i,
                imported_module.rust_module,
                import.func_name,
                out_args,
                out_conv,
                imported_module.module_name,
                import.func_name,
                static_func_name,
            );
        }
        result += "}"
    }

    result += "
        })
    }
}   ";
    result += &webrogue_macro_common::make_context_fn(&imports);

    result.parse().unwrap()
}
