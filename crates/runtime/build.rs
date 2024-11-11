// use wasmer;

#[cfg(feature = "aot")]
use std::str::FromStr;

#[cfg(feature = "aot")]
fn run_cmd(command: &mut std::process::Command) {
    let output = command.output().unwrap();
    if !output.status.success() {
        unsafe {
            panic!(
                "{}\n\n{}",
                std::str::from_utf8_unchecked(&output.stdout),
                std::str::from_utf8_unchecked(&output.stderr),
            )
        }
    }
}

fn main() {
    #[cfg(feature = "aot")]
    {
        let crate_manifest_dir =
            std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

        let workspace_manifest_dir = crate_manifest_dir
            .clone()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        let out_dir = std::path::PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();
        let target_dir = out_dir
            // .clone()
            // .join("target")
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        // let _ = std::fs::create_dir(target_dir.clone());

        let make_cargo_run = |args| {
            let mut binding = std::process::Command::new(std::env::var("CARGO").unwrap());
            let cargo_run = binding
                .arg("run")
                .arg("--package=wasmer-cli")
                .arg("--release")
                .arg("--manifest-path")
                .arg(
                    workspace_manifest_dir
                        .join("external")
                        .join("wasmer")
                        .join("Cargo.toml"),
                )
                .arg("--target-dir")
                .arg(target_dir)
                .arg("--features=cranelift");
            run_cmd(cargo_run.args(args));
        };

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
                    r#"Unknown system
("{}", "{}", "{}", "{}", "{}", "{}") => "",
"#,
                    family, os, arch, vendor, env, abi
                )
            }
        };
        let wasm_path = std::env::var("WEBROGUE_AOT_PATH").expect("WEBROGUE_AOT_PATH");
        println!("cargo::rerun-if-env-changed=WEBROGUE_AOT_PATH");
        println!("cargo::rerun-if-changed={}", wasm_path);
        let common_args = vec![
            &wasm_path,
            "--prefix",
            "wr_aot",
            "--target",
            target,
            "--cranelift",
        ];

        let obj_path = out_dir.join("wr_aot.o");
        // panic!("{}", target_dir.clone().display());
        let binding = obj_path.clone();
        let mut args = vec!["create-obj", "-o", binding.to_str().unwrap()];
        args.append(&mut common_args.clone());
        args.push("--enable-all");
        make_cargo_run(args);

        // let header_path = out_dir.join("wr_aot.h");
        // let binding = header_path.clone();
        // let mut args = vec!["gen-c-header", "-o", binding.to_str().unwrap()];
        // args.append(&mut common_args.clone());
        // make_cargo_run(args);

        cc::Build::new().object(obj_path).compile("wr_aot");
    }
}
