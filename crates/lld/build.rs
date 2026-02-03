#[cfg(not(feature = "link-llvm"))]
fn main() {}

#[cfg(feature = "link-llvm")]
fn main() {
    use std::env;
    use std::path::PathBuf;
    use std::str::FromStr as _;

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let (lib_prefix, lib_suffix) = if target_os == "windows" {
        ("", ".lib")
    } else {
        ("lib", ".a")
    };
    let target_triple = env::var("TARGET").unwrap();
    let lib_name = "webroguelld";
    let lib_dir = PathBuf::from_str(&env::var("OUT_DIR").unwrap()).unwrap();
    let lib = lib_dir.join(format!("{}{}{}", lib_prefix, lib_name, lib_suffix));
    if !lib.exists() {
        let mut response = reqwest::blocking::get(format!("https://github.com/webrogue-runtime/webrogue-lld-builder/releases/download/latest_build/{}", target_triple)).unwrap().error_for_status().unwrap();
        response
            .copy_to(&mut std::fs::File::create(&lib).unwrap())
            .unwrap();
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static={}", lib_name);
    if target_os == "linux" {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target_os == "macos" {
        println!("cargo:rustc-link-lib=dylib=c++");
    }
}
