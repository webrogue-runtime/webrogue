use proc_macro::TokenStream;

#[proc_macro]
pub fn make_funcs(item: TokenStream) -> TokenStream {
    let imports = webrogue_macro_common::parse_macro_input!(item as webrogue_macro_common::Imports);

    let mut result = "".to_owned();

    result += "
fn make_imports() -> webrogue_backend_web::Imports {
    let mut result = webrogue_backend_web::Imports::new();
";
    for (i, imported_module) in imports.modules.iter().enumerate() {
        result += &format!("{}\n{{\n", imported_module.attributes.join("\n"));
        for import in imported_module.funcs.clone() {
            let args = import
                .args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    return format!(
                        "unsafe {{ webrogue_backend_web::ffi::wr_rs_em_js_getArg{}({}) }},",
                        arg.rust_type_str().to_uppercase(),
                        i
                    );
                })
                .collect::<Vec<_>>()
                .join("");
            let ret: String = match import.ret {
                None => "".to_owned(),
                Some(ret) => {
                    format!(
                        "unsafe {{ webrogue_backend_web::ffi::wr_rs_em_js_setResult{}(result) }}",
                        ret.rust_type_str().to_uppercase()
                    )
                }
            };
            result += &format!(
                r##"
        result.add_fn(
            "{}",
            "{}",
            "{}",
            Box::new({{
                move |context| {{
                    let result = {}::{}(
                        &mut context.memory_factory,
                        unsafe {{ context.context_vec.get::<{}::Context>({}) }},
                        {}
                    );
                    {}
                }}
            }})
        );
    "##,
                imported_module.module_name,
                import.func_name,
                import
                    .ret
                    .and_then(|ret| match ret {
                        webrogue_macro_common::ValueType::U32 => Some("uint32_t"),
                        webrogue_macro_common::ValueType::U64 => Some("uint64_t"),
                        webrogue_macro_common::ValueType::I32 => Some("int32_t"),
                        webrogue_macro_common::ValueType::I64 => Some("int64_t"),
                        webrogue_macro_common::ValueType::F32 => Some("float"),
                        webrogue_macro_common::ValueType::F64 => Some("double"),
                    })
                    .unwrap_or("void"),
                imported_module.rust_module,
                import.func_name,
                imported_module.rust_module,
                i,
                args,
                ret
            );
        }
        result += "}";
    }
    result += r"
    result
}
            ";

    result += &webrogue_macro_common::make_context_fn(&imports);

    result.parse().unwrap()
}
