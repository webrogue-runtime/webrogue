use crate::types::*;

fn simple_type_rust_to_wasm(ty: GLType, name: String) -> Option<String> {
    match ty {
        GLType::Float => Some(name),
        GLType::UInt => Some(format!("{}.into()", name)),
        GLType::Int => Some(format!("{}.into()", name)),
        GLType::U8 => Some(format!("{}.into()", name)),
        GLType::I64 => Some(format!("{}", name)),
        GLType::U64 => Some(format!("{}", name)),
        GLType::Void => Some(name),
        GLType::OpaqueSync => Some(format!("gl.register_opaque_sync_object({})", name)),
        _ => None,
    }
}

fn map_param_name(original: String) -> String {
    match original.as_str() {
        "type" => "_type".to_owned(),
        "ref" => "_ref".to_owned(),
        _ => original,
    }
}

pub fn get_as_str(parse_results: &ParseResults) -> String {
    let mut result = r#"use crate::{env_wrapper::EnvWrapper, memory_handle::MemoryHandle};

// DO NOT EDIT! This file is generated automatically

#[allow(non_snake_case)]
#[rustfmt::skip]
pub fn add_to_imports(
    exports: &mut wasmer::Exports,
    store: &mut wasmer::StoreMut<'_>,
    env: &wasmer::FunctionEnv<EnvWrapper>,
) {
"#
    .to_owned();
    for command in parse_results.commands.clone() {
        if crate::common::EXCLUDED.contains(&command.name.as_str()) {
            continue;
        }
        if crate::common::MANUAL_IMPL.contains(&command.name.as_str()) {
            continue;
        }
        let mut import_args = vec![];
        let mut ffi_args = vec![];
        let mut converts = vec![];
        let mut writes = vec![];
        let mut ffi_arg_types = vec![];

        let mut is_memory_mut = None;
        let mut is_gl_mut = false;

        for param in command.params.clone() {
            let mapped_name = map_param_name(param.name.clone());
            import_args.push(format!(
                "\n            {}: {},",
                mapped_name,
                param.ty.to_wasm_param_type()
            ));

            ffi_arg_types.push(format!(
                "    {}: {},\n",
                mapped_name,
                param.ty.to_rust_type()
            ));

            let converted_param = match param.ty.clone() {
                GLType::Float => {
                    format!(
                        "                let converted_{} = {};",
                        mapped_name, mapped_name
                    )
                }
                GLType::UInt => format!(
                    "                let converted_{} = {};",
                    mapped_name, mapped_name
                ),
                GLType::Int => format!(
                    "                let converted_{} = {};",
                    mapped_name, mapped_name
                ),
                GLType::U8 => format!(
                    "                let converted_{} = {} as u8;",
                    mapped_name, mapped_name
                ),
                GLType::U64 => format!(
                    "                let converted_{} = {};",
                    mapped_name, mapped_name
                ),
                GLType::I64 => format!(
                    "                let converted_{} = {};",
                    mapped_name, mapped_name
                ),
                GLType::ISizeT => {
                    format!(
                        "                let converted_{} = {} as isize;",
                        mapped_name, mapped_name
                    )
                }
                GLType::Void => panic!(),
                GLType::OpaqueSync => format!(
                    "                let converted_{} = gl.resolve_opaque_sync_object({});",
                    mapped_name, mapped_name
                ),
                GLType::Ptr(inner, is_const) => {
                    let inner_ty = (*inner).clone();
                    if is_const {
                        if is_memory_mut == None {
                            is_memory_mut = Some(false)
                        }
                    } else {
                        is_memory_mut = Some(true)
                    }
                    let rust_type = match inner_ty.clone() {
                        GLType::Void => "u8".to_owned(),
                        _ => inner_ty.to_rust_type(),
                    };
                    let mut len_param = param
                        .len_name
                        .or(Some("UNKNOWN".to_owned()))
                        .clone()
                        .unwrap();
                    if (len_param == format!("COMPSIZE({})", param.name)
                        || len_param == "UNKNOWN"
                        || len_param == "COMPSIZE()")
                        && inner_ty == GLType::I8
                        && is_const
                    {
                        len_param = format!(
                            "crate::utils::guest_strlen(&memory, {} as u64) + 1",
                            mapped_name
                        );
                    } else {
                        if len_param.starts_with("COMPSIZE") {
                            is_gl_mut = true;
                            len_param = format!(
                                "crate::compsize::{}_{}_compsize(&mut gl, {})",
                                command.name,
                                mapped_name,
                                len_param[9..len_param.len() - 1]
                                    .split(",")
                                    .map(|s| {
                                        let mapped_name = map_param_name(s.to_owned());
                                        if let Some(param) = command
                                            .params
                                            .iter()
                                            .filter(|param_candidate| param_candidate.name == s)
                                            .next()
                                        {
                                            if let Some(group) = param.group.clone() {
                                                format!(
                                                    "crate::ffi::GLEnumGroup{}::from_raw({})",
                                                    group, mapped_name
                                                )
                                                .to_owned()
                                            } else {
                                                mapped_name
                                            }
                                        } else {
                                            mapped_name
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join(",")
                            )
                        }
                    }
                    if !is_const {
                        if inner_ty.to_wasm_mem_type() == rust_type {
                            writes.push(format!(
                                "
                memory.write_slice::<{}>({} as u64, &vec_{})?;
",
                                inner_ty.to_wasm_mem_type(),
                                mapped_name,
                                mapped_name,
                            ))
                        } else {
                            writes.push(format!(
                            "
                memory.write_slice::<{}>({} as u64, &vec_{}.iter().map(|v| *v as _).collect::<Vec<_>>())?;
",
                            inner_ty.to_wasm_mem_type(),
                            mapped_name,
                            mapped_name,
                        ))
                        }
                    }
                    let read = if inner_ty.to_wasm_mem_type() == rust_type {
                        format!(
                            "let mut vec_{}: Vec<{}> = memory.read_vec::<{}>({} as u64, len_{} as u64)?;", 
                            mapped_name,
                            rust_type,
                            inner_ty.to_wasm_mem_type(),
                            mapped_name,
                            mapped_name,
                        )
                    } else {
                        format!(
                            "let mut vec_{}: Vec<{}> = memory.read_vec::<{}>({} as u64, len_{} as u64)?.iter().map(|v| *v as {}).collect::<Vec<_>>();",
                            mapped_name,
                            rust_type,
                            inner_ty.to_wasm_mem_type(),
                            mapped_name,
                            mapped_name,
                            rust_type,
                        )
                    };
                    format!(
                        r#"
                let len_{} = ({}) as usize;
                {}
                let converted_{} = vec_{}.as_mut_ptr() as {};
"#,
                        mapped_name,
                        len_param,
                        read,
                        mapped_name,
                        mapped_name,
                        param.ty.to_rust_type(),
                    )
                }
                GLType::I8 => panic!(),
            };

            converts.push(converted_param);
            ffi_args.push(format!(
                "\n                        converted_{},",
                mapped_name
            ));
        }
        let memory_init = match is_memory_mut {
            None => "",
            Some(_) => {
                r#"                let memory =
                    MemoryHandle::new(store.data().lazy.get().unwrap().memory.clone(), &store);
"#
            }
        };
        result += &format!(
            r#"    exports.insert(
    "{}",
    wasmer::Function::new_typed_with_env(
        store,
        &env,
        move |
            mut store: wasmer::FunctionEnvMut<EnvWrapper>,{}
        | -> Result<{}, wasmer::RuntimeError> {{
                let gl_arc = store.data_mut().gl.clone();
                let {}gl = gl_arc.{}().unwrap();
{}{}
                let result = unsafe {{
                    (gl.proc_addresses.{})({}
                    ) 
                }};
{}
                Ok({})
            }},
        ),
    );
"#,
            command.name,
            import_args.join(""),
            match command.ret {
                GLType::Void => "()".to_owned(),
                _ => command.ret.to_wasm_param_type(),
            },
            if is_gl_mut { "mut " } else { "" },
            if is_gl_mut { "write" } else { "read" },
            memory_init,
            converts.join("\n"),
            command.name,
            ffi_args.join(""),
            writes.join("\n"),
            match simple_type_rust_to_wasm(command.ret.clone(), "result".to_owned()) {
                None => {
                    dbg!(command.ret.clone());
                    format!(
                        "compile_error!(\"unsupported return type in {}\")",
                        command.name
                    )
                }
                Some(v) => v,
            }
        );
    }
    result += r#"
}"#;
    result
}
