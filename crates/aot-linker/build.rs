use std::str::FromStr as _;

fn main() {
    let crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    let dst = cmake::Config::new(crate_manifest_dir)
        .define("LLVM_ENABLE_PROJECTS", "lld")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("LLVM_TARGETS_TO_BUILD", "")
        .define("LLVM_ENABLE_LIBXML2", "OFF")
        .define("LLVM_ENABLE_ZLIB", "OFF")
        .define("LLVM_ENABLE_ZSTD", "OFF")
        .always_configure(false)
        .build();
    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    // println!("cargo:rustc-link-lib=static=stdc++");
    // println!("cargo:rustc-link-lib=static=c++");
    // let link_type = "static";
    // let libs = "lldCommon lldCOFF lldELF lldMachO lldMinGW lldWasm LLVMSupport LLVMCodeGen LLVMCore LLVMDebugInfoDWARF LLVMDemangle LLVMMC LLVMOption LLVMTarget LLVMTargetParser";

    // println!("cargo:rustc-link-lib={}=lldAsLib", link_type);
    // for lib in libs.split(' ') {
    //     println!("cargo:rustc-link-lib={}={}", link_type, lib);
    // }
   
    // println!("cargo::rustc-link-arg=-lstdc++");
    let deps_path = dst.join("build/lldAsLib_deps.txt");
    let deps = std::fs::read_to_string(deps_path).unwrap();

    for dep in deps.split(';') {
        println!("cargo:rustc-link-lib=static={}", dep);
    }

    println!("cargo:rustc-link-lib=dylib=c++");
}
