use crate::types::*;
use std::io::Write;

pub fn write_to_file(file: &mut std::fs::File, parse_results: &ParseResults) {
    for command in parse_results.commands.clone() {
        if crate::common::EXCLUDED.contains(&command.name.as_str()) {
            continue;
        }
        if crate::common::C_MANUAL_IMPL.contains(&command.name.as_str()) {
            continue;
        }
        let params_with_types = command
            .params
            .iter()
            .map(|param| param.ty.to_c_type() + " " + param.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let params_without_types = command
            .params
            .iter()
            .map(|param| param.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        file.write(
            format!(
                r#"
__attribute__((import_name("{}")))
__attribute__((import_module("webrogue_gl")))
{} imported_{}({});

{} {}({}) {{
    {}imported_{}({});
}}
        "#,
                command.name,
                command.ret.to_c_type(),
                command.name,
                params_with_types,
                command.ret.to_c_type(),
                command.name,
                params_with_types,
                if command.ret == GLType::Void {
                    ""
                } else {
                    "return "
                },
                command.name,
                params_without_types
            )
            .as_bytes(),
        )
        .unwrap();
    }
}
