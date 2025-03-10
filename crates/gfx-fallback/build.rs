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
        #[cfg(target_os = "macos")]
        {
            println!("cargo:rustc-link-lib=framework=Quartz");
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=IOKit");
            println!("cargo:rustc-link-lib=framework=GameController");
            println!("cargo:rustc-link-lib=framework=ForceFeedback");
            println!("cargo:rustc-link-lib=framework=CoreVideo");
            println!("cargo:rustc-link-lib=framework=CoreHaptics");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=CoreAudio");
            println!("cargo:rustc-link-lib=framework=Cocoa");
            println!("cargo:rustc-link-lib=framework=Carbon");
            println!("cargo:rustc-link-lib=framework=AudioToolbox");
        }
    }
    #[cfg(feature = "cc")]
    {
        cc::Build::new()
            .file("webrogue_gfx_ffi_sdl2.c")
            .file("webrogue_gfx_ffi_sdl2_events.c")
            .include("SDL/include")
            .compile("wrgfxfallback");
    }
}
