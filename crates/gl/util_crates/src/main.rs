mod c_guest;
mod c_guest_loader;
mod common;
mod parse;
mod proc_addresses;
mod rust_ffi;
mod rust_wasm_imports_imps;
mod types;
mod wr_defs;

use std::io::Write;

fn run_flavor(
    f: impl Fn(&mut std::fs::File, &types::ParseResults),
    path: &str,
    commands: &types::ParseResults,
) {
    let mut file: std::fs::File = std::fs::File::create(path).unwrap();
    f(&mut file, commands);
}

// TODO get rid of run_macro
fn run_macro(
    preamble: &str,
    f: impl Fn(&types::ParseResults) -> String,
    path: &str,
    commands: &types::ParseResults,
) {
    let mut file: std::fs::File = std::fs::File::create(path).unwrap();
    file.write(preamble.as_bytes()).unwrap();
    file.write(f(commands).as_bytes()).unwrap();
}

fn main() {
    let parse_results = parse::parse();

    run_flavor(
        c_guest::write_to_file,
        "../../../examples/libs/GLES2/gl2.inc",
        &parse_results,
    );

    run_flavor(
        wr_defs::write_to_file,
        "../../../crates/gl/defs.in",
        &parse_results,
    );

    run_flavor(
        c_guest_loader::write_to_file,
        "../../../examples/libs/webrogue_gfx/webrogue_gl_loader.c",
        &parse_results,
    );

    run_macro(
        "#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unreachable_patterns)]

// DO NOT EDIT! This file is generated automatically

",
        rust_ffi::get_as_str,
        "../../../crates/gl/src/ffi.rs",
        &parse_results,
    );

    run_macro(
        "#![allow(non_snake_case)]

pub use crate::context::Context;
pub use crate::mainual_impl::*;

// DO NOT EDIT! This file is generated automatically

",
        rust_wasm_imports_imps::get_as_str,
        "../../../crates/gl/src/api.rs",
        &parse_results,
    );

    run_macro(
        "#![allow(non_snake_case)]
",
        proc_addresses::get_as_str,
        "../../../crates/gl/src/proc_addresses.rs",
        &parse_results,
    );
}
