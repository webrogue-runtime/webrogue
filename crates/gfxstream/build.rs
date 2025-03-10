use std::str::FromStr as _;

fn main() {
    let _crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    #[cfg(feature = "cmake")]
    {
        let dst = cmake::Config::new(_crate_manifest_dir).build();
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=webrogue_gfxstream");

        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-lib=dylib=stdc++");
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=dylib=c++");
    }
    #[cfg(feature = "cc")]
    {
        let external_dir = _crate_manifest_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("external");
        let gfx_src_dir = external_dir.join("gfxstream");
        let gfx_host_src_dir = gfx_src_dir.join("host");
        let aemu_src_dir = external_dir.join("aemu");
        cc::Build::new()
            .cpp(true)
            .std("c++17")
            .file("webrogue_gfxstream.cpp")
            .file(gfx_host_src_dir.join("gl").join("glsnapshot").join("GLSnapshot.cpp"))
            .file(gfx_host_src_dir.join("gl").join("gles2_dec").join("GLESv2Decoder.cpp"))
            .file(gfx_host_src_dir.join("gl").join("gles2_dec").join("gles2_server_context.cpp"))
            .file(gfx_host_src_dir.join("gl").join("gles2_dec").join("gles2_dec.cpp"))
            .file(gfx_host_src_dir.join("apigen-codec-common").join("ChecksumCalculatorThreadInfo.cpp"))
            .file(gfx_host_src_dir.join("apigen-codec-common").join("ChecksumCalculator.cpp"))
            .file(aemu_src_dir.join("host-common").join("vm_operations.cpp"))
            .file(aemu_src_dir.join("host-common").join("crash_reporter.cpp"))
            .file(aemu_src_dir.join("base").join("Tracing.cpp"))
            .file(aemu_src_dir.join("base").join("Stream.cpp"))
            .include(gfx_host_src_dir.clone())
            .include(gfx_src_dir.join("include"))
            .include(gfx_host_src_dir.join("apigen-codec-common"))
            .include(gfx_host_src_dir.join("gl").join("glsnapshot"))
            .include(aemu_src_dir.join("base").join("include"))
            .include(aemu_src_dir.join("host-common").join("include"))
            .compile("webrogue_gfxstream");
    }
}
