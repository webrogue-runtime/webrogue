use crate::types::*;
use std::io::Write;

pub fn write_to_file(file: &mut std::fs::File, parse_results: &ParseResults) {
    file.write(
        b"glGetStringData(i32, u32, u32) -> ()
glGetStringLen(i32, u32) -> (i32)
init_ptrs() -> ()
",
    )
    .unwrap();

    for command in parse_results.commands.clone() {
        if crate::common::EXCLUDED.contains(&command.name.as_str()) {
            continue;
        }
        file.write(
            format!(
                "{}({}) -> ({})\n",
                command.name,
                command
                    .params
                    .iter()
                    .map(|param| { param.ty.to_wasm_param_type() })
                    .collect::<Vec<_>>()
                    .join(", "),
                command.ret.to_wasm_param_type()
            )
            .as_bytes(),
        )
        .unwrap();
    }
}
