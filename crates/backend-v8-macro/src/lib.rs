use proc_macro::TokenStream;

#[proc_macro]
pub fn make_funcs(item: TokenStream) -> TokenStream {
    let imports = webrogue_macro_common::parse_macro_input!(item as webrogue_macro_common::Imports);
    let mut result = format!(
        "
{}fn make_imports(
) -> webrogue_backend_v8::Imports {{
    webrogue_backend_v8::Imports {{
        f: Box::new(|
            scope: &mut webrogue_backend_v8::v8::HandleScope<webrogue_backend_v8::v8::Context>, 
            imports: webrogue_backend_v8::v8::Local<webrogue_backend_v8::v8::Object>
        | {{
",
        if imports.is_public { "pub " } else { "" }
    );

    for (i, imported_module) in imports.modules.iter().enumerate() {
        result += &format!("{}\n{{\n", imported_module.attributes.join("\n"));
        for import in imported_module.funcs.clone() {
            let args = import
                .args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    format!(
                        "                args.get({}).{}",
                        i,
                        match arg {
                            webrogue_macro_common::ValueType::U32 => "uint32_value(scope).unwrap()",
                            webrogue_macro_common::ValueType::U64 =>
                                "to_big_int(scope).unwrap().u64_value().0",
                            webrogue_macro_common::ValueType::I32 => "int32_value(scope).unwrap()",
                            webrogue_macro_common::ValueType::I64 =>
                                "to_big_int(scope).unwrap().i64_value().0",
                            webrogue_macro_common::ValueType::F32 =>
                                "number_value(scope).unwrap() as f32",
                            webrogue_macro_common::ValueType::F64 => "number_value(scope).unwrap()",
                        }
                    )
                })
                .collect::<Vec<_>>()
                .join(",\n");
            let ret = match import.ret {
                Some(ret) => match ret {
                    webrogue_macro_common::ValueType::U32 => "rv.set_uint32(ret);",
                    webrogue_macro_common::ValueType::U64 => {
                        "rv.set(v8::BigInt::new_from_u64(scope, ret).into());"
                    }
                    webrogue_macro_common::ValueType::I32 => "rv.set_int32(ret);",
                    webrogue_macro_common::ValueType::I64 => {
                        "rv.set(v8::BigInt::new_from_I64(scope, ret).into());"
                    }
                    webrogue_macro_common::ValueType::F32 => "rv.set_double(ret as f64);",
                    webrogue_macro_common::ValueType::F64 => "rv.set_double(ret);",
                },
                None => "",
            };
            result += &format!(
                "
    let func = webrogue_backend_v8::v8::Function::new(
        scope,
        |scope: &mut webrogue_backend_v8::v8::HandleScope,
         args: webrogue_backend_v8::v8::FunctionCallbackArguments,
         mut rv: webrogue_backend_v8::v8::ReturnValue| {{
            let data = scope.get_data(0);
            let webrogue_context = data as *mut webrogue_backend_v8::Context;
            let memory_factory = &mut unsafe {{ &mut *webrogue_context }}.memory_factory;
            let ctx = unsafe {{
                &mut (*webrogue_context)
                    .context_vec
                    .get::<{}::Context>({})
            }};

            let ret = {}::{}(
                memory_factory,
                ctx,
                {}
            );
            {}
        }},
    )
    .unwrap();
    webrogue_backend_v8::register_import(scope, imports, \"{}\", \"{}\", func.into());
            ",
                imported_module.rust_module,
                i,
                imported_module.rust_module,
                import.func_name,
                args,
                ret,
                imported_module.module_name,
                import.func_name,
            );
        }
        result += "}"
    }
    result += "
        })
    }
}
    ";
    result += &webrogue_macro_common::make_context_fn(&imports);
    result.parse().unwrap()
}
