fn main() {
    // #[cfg(target_arch = "aarch64")]
    #[cfg(feature = "runner")]
    {
        use std::str::FromStr as _;

        let crate_manifest_dir =
            std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

        println!("cargo:rustc-link-lib=dylib=webrogue_aot");
        println!(
            "cargo:rustc-link-search=native={}",
            crate_manifest_dir
                .join("runner")
                .join("src")
                .join("main")
                .join("jniLibs")
                .join(match arch.as_str() {
                    "aarch64" => "arm64-v8a",
                    arch => arch,
                })
                .display()
        );
    }
}
