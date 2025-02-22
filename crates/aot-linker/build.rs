#[cfg(not(feature = "build-llvm"))]
fn main() {
    panic!("build-llvm is disabled");
}

#[cfg(feature = "build-llvm")]
use std::str::FromStr as _;
#[cfg(feature = "build-llvm")]
fn main() {
    let crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    let dst = cmake::Config::new(crate_manifest_dir)
        .define("LLVM_ENABLE_PROJECTS", "lld")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("LLVM_ENABLE_LIBXML2", "OFF")
        .define("LLVM_ENABLE_ZLIB", "OFF")
        .define("LLVM_ENABLE_ZSTD", "OFF")
        .define("LLVM_ENABLE_TERMINFO", "OFF")
        .always_configure(false)
        .build();
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );

    let deps_path = dst.join("build/lldAsLib_deps.txt");
    let deps = std::fs::read_to_string(deps_path).unwrap();

    for dep in deps.split(';') {
        println!("cargo:rustc-link-lib=static={}", dep);
    }
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=dylib=stdc++");
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=dylib=c++");
}
