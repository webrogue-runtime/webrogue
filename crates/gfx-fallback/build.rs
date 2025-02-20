use std::str::FromStr as _;

fn download_sdl_source(output_path: &std::path::Path) {
    if output_path.is_dir() {
        return;
    }
    let archive_bytes = reqwest::blocking::get(
        "https://github.com/libsdl-org/SDL/archive/refs/tags/release-2.30.9.zip",
    )
    .unwrap()
    .bytes()
    .unwrap();
    zip_extract::extract(std::io::Cursor::new(archive_bytes), &output_path, true).unwrap();
}

fn main() {
    let crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    download_sdl_source(&crate_manifest_dir.join("SDL"));

    #[cfg(feature = "cmake")]
    {
        let dst = cmake::Config::new(crate_manifest_dir)
            .define("SDL_CMAKE_DEBUG_POSTFIX", "")
            .build();
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=wrgfxfallback");
        println!("cargo:rustc-link-lib=static=SDL2");
        println!("cargo:rustc-link-lib=static=aemu-base");
        println!("cargo:rustc-link-lib=static=logging-base");
        println!("cargo:rustc-link-lib=static=gles2_dec");
        println!("cargo:rustc-link-lib=static=GLSnapshot");
        println!("cargo:rustc-link-lib=static=apigen-codec-common");

        #[cfg(target_os = "linux")]
        println!("cargo:rustc-link-lib=dylib=stdc++");
        #[cfg(target_os = "macos")]
        println!("cargo:rustc-link-lib=dylib=c++");
    }
    #[cfg(feature = "cc")]
    {
        cc::Build::new()
            .file("webrogue_gfx_ffi_sdl2.c")
            .include("SDL/include")
            .compile("wr_aot");
    }
}
