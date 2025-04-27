#[cfg(not(feature = "build-llvm"))]
fn main() {}

#[cfg(feature = "build-llvm")]
fn main() {
    use std::str::FromStr as _;

    let crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    let mut cmake_cfg = cmake::Config::new(crate_manifest_dir);
    cmake_cfg
        .define("LLVM_ENABLE_PROJECTS", "lld")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("LLVM_ENABLE_LIBXML2", "OFF")
        .define("LLVM_ENABLE_ZLIB", "OFF")
        .define("LLVM_ENABLE_ZSTD", "OFF")
        .define("LLVM_ENABLE_TERMINFO", "OFF")
        .define("LLVM_TARGETS_TO_BUILD", "")
        .profile("Release")
        .always_configure(false);
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "windows" {
        cmake_cfg
            .static_crt(true)
            .define("CMAKE_C_FLAGS_DEBUG", "/Zi /Ob0 /Od /RTC1")
            .define("CMAKE_C_FLAGS_RELEASE", "/O2 /Ob2 /DNDEBUG")
            .define("CMAKE_C_FLAGS_MINSIZEREL", "/O1 /Ob1 /DNDEBUG")
            .define("CMAKE_C_FLAGS_RELWITHDEBINFO", "/Zi /O2 /Ob1 /DNDEBUG")
            .define("CMAKE_CXX_FLAGS_DEBUG", "/Zi /Ob0 /Od /RTC1")
            .define("CMAKE_CXX_FLAGS_RELEASE", "/O2 /Ob2 /DNDEBUG")
            .define("CMAKE_CXX_FLAGS_MINSIZEREL", "/O1 /Ob1 /DNDEBUG")
            .define("CMAKE_CXX_FLAGS_RELWITHDEBINFO", "/Zi /O2 /Ob1 /DNDEBUG")
            .define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded")
            .define("LLVM_DISABLE_ASSEMBLY_FILES", "ON");
    }
    let dst = cmake_cfg.build();
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib64").display()
    );

    let deps_path = dst.join("build/lldAsLib_deps.txt");
    let deps = std::fs::read_to_string(deps_path).unwrap();

    for dep in deps.split(';') {
        println!("cargo:rustc-link-lib=static={}", dep);
    }
    if target_os == "linux" {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else if target_os == "macos" {
        println!("cargo:rustc-link-lib=dylib=c++");
    }
}
