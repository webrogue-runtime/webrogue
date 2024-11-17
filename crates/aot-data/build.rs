fn main() {
    #[cfg(not(feature = "skip-compiling"))]
    {
        use std::str::FromStr;

        let out_dir = std::path::PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();

        let wasm_path = std::env::var("WEBROGUE_AOT_PATH").expect("WEBROGUE_AOT_PATH");
        println!("cargo::rerun-if-env-changed=WEBROGUE_AOT_PATH");
        println!("cargo::rerun-if-changed={}", wasm_path);

        let obj_path = out_dir.join("wr_aot.o");

        let target = match (
            std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_OS").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_VENDOR").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_ENV").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_ABI").unwrap().as_str(),
        ) {
            ("unix", "linux", "x86_64", "unknown", "gnu", "") => "x86_64-linux-gnu",
            // aarch64-linux-gnu
            // x86_64-apple-darwin
            // arm64-apple-darwin
            // x86_64-windows-gnu
            (family, os, arch, vendor, env, abi) => {
                panic!(
                    "Unknown system\n(\"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\")\n",
                    family, os, arch, vendor, env, abi
                )
            }
        };

        webrogue_aot_compiler::compile_wasm_file(wasm_path.into(), obj_path.clone(), target)
            .expect("webrogue_aot_compiler failed");

        cc::Build::new().object(obj_path).compile("wr_aot");
        println!("cargo:rustc-link-lib=static=wr_aot");
        // panic!("");
    }
}
