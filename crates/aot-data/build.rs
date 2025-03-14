fn main() {
    #[cfg(feature = "compiling")]
    {
        extern crate cc;
        extern crate webrogue_aot_compiler;
        use std::str::FromStr;

        let out_dir = std::path::PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();

        let wrapp_path = std::env::var("WEBROGUE_AOT_PATH").expect("WEBROGUE_AOT_PATH");
        println!("cargo::rerun-if-env-changed=WEBROGUE_AOT_PATH");
        println!("cargo::rerun-if-changed={}", wrapp_path);

        let obj_path = out_dir.join("wr_aot.o");

        let (target, is_pic) = match (
            std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_OS").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_VENDOR").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_ENV").unwrap().as_str(),
            std::env::var("CARGO_CFG_TARGET_ABI").unwrap().as_str(),
        ) {
            ("unix", "linux", "x86_64", "unknown", "gnu", "") => {
                (webrogue_aot_compiler::Target::X86_64LinuxGNU, false) // TODO check is_pic
            }
            ("unix", "macos", "x86_64", "apple", "", "") => {
                (webrogue_aot_compiler::Target::x86_64AppleDarwin, false) // TODO check is_pic
            }
            (family, os, arch, vendor, env, abi) => {
                panic!(
                    "Unknown system\n(\"{}\", \"{}\", \"{}\", \"{}\", \"{}\", \"{}\")\n",
                    family, os, arch, vendor, env, abi
                )
            }
        };

        webrogue_aot_compiler::compile_wrapp_to_object(
            wrapp_path.into(),
            obj_path.clone(),
            target,
            is_pic,
        )
        .expect("webrogue_aot_compiler failed");

        cc::Build::new().object(obj_path).compile("wr_aot");
        println!("cargo:rustc-link-lib=static=wr_aot");
    }
}
