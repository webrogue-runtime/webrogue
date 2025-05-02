use std::str::FromStr as _;

fn download_sdl_source(output_path: &std::path::Path, version: &str) {
    if output_path.is_dir() {
        return;
    }
    let archive_bytes = reqwest::blocking::get(format!(
        "https://github.com/libsdl-org/SDL/archive/refs/tags/release-{}.zip",
        version
    ))
    .unwrap()
    .bytes()
    .unwrap();
    zip_extract::extract(std::io::Cursor::new(archive_bytes), output_path, true).unwrap();
}

fn main() {
    let crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    #[cfg(feature = "sdl3")]
    download_sdl_source(&crate_manifest_dir.join("SDL3"), "3.2.10");

    #[cfg(feature = "cmake")]
    {
        let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
        let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

        let mut cmake_cfg = cmake::Config::new(crate_manifest_dir);
        if os == "windows" {
            cmake_cfg
                .static_crt(true)
                .define("CMAKE_C_FLAGS_DEBUG", "/Zi /Ob0 /Od /RTC1")
                .define("CMAKE_C_FLAGS_RELEASE", "/O2 /Ob2 /DNDEBUG")
                .define("CMAKE_C_FLAGS_MINSIZEREL", "/O1 /Ob1 /DNDEBUG")
                .define("CMAKE_C_FLAGS_RELWITHDEBINFO", "/Zi /O2 /Ob1 /DNDEBUG");
        }
        if os == "linux" && env == "musl" {
            cmake_cfg.define("SDL_X11_SHARED", "OFF");
            cmake_cfg.define("CMAKE_FIND_LIBRARY_SUFFIXES", ".a");
        }

        #[cfg(feature = "sdl3")]
        cmake_cfg.define("WEBROGUE_GFX_SDL_VERSION", "3");
        cmake_cfg
            .define("SDL_CMAKE_DEBUG_POSTFIX", "")
            .define("SDL_OPENGL", "OFF")
            .define("SDL_OPENGLES", "ON");

        let dst = cmake_cfg.build();
        println!(
            "cargo:rustc-link-search=native={}",
            dst.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=wrgfxfallback");
        if os == "windows" {
            #[cfg(feature = "sdl3")]
            println!("cargo:rustc-link-lib=static=SDL3-static");
        } else {
            #[cfg(feature = "sdl3")]
            println!("cargo:rustc-link-lib=static=SDL3");
        }
        if os == "macos" {
            #[cfg(feature = "sdl3")]
            {
                println!("cargo:rustc-link-lib=framework=AudioToolbox");
                println!("cargo:rustc-link-lib=framework=Carbon");
                println!("cargo:rustc-link-lib=framework=Cocoa");
                println!("cargo:rustc-link-lib=framework=CoreAudio");
                println!("cargo:rustc-link-lib=framework=CoreFoundation");
                println!("cargo:rustc-link-lib=framework=CoreHaptics");
                println!("cargo:rustc-link-lib=framework=CoreVideo");
                println!("cargo:rustc-link-lib=framework=ForceFeedback");
                println!("cargo:rustc-link-lib=framework=GameController");
                println!("cargo:rustc-link-lib=framework=IOKit");
                println!("cargo:rustc-link-lib=framework=Metal");
                println!("cargo:rustc-link-lib=framework=QuartzCore");
                println!("cargo:rustc-link-lib=framework=UniformTypeIdentifiers");

                println!("cargo:rustc-link-lib=framework=AVFoundation");
                println!("cargo:rustc-link-lib=framework=CoreMedia");
            }
        }
    }
    #[cfg(feature = "cc")]
    {
        cc::Build::new()
            .file("webrogue_gfx_ffi_sdl.c")
            .file("webrogue_gfx_ffi_sdl_events.c")
            .define("WEBROGUE_GFX_SDL_VERSION", "2")
            .include("SDL/include")
            .compile("wrgfxfallback");
    }
}
