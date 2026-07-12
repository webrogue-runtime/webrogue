use std::str::FromStr as _;

fn main() {
    let _crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    println!(
        "cargo:rerun-if-changed={}",
        _crate_manifest_dir
            .join("witx")
            .join("webrogue_gfx.witx")
            .display()
    );

    #[cfg(feature = "static-moltenvk")]
    link_moltenvk();
}

#[cfg(feature = "static-moltenvk")]
fn link_moltenvk() {
    use std::{env, fs::File, path::PathBuf, str::FromStr};

    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if os != "macos" {
        return;
    }

    let out_dir = PathBuf::from_str(&env::var("OUT_DIR").unwrap()).unwrap();

    let tar_path = out_dir.join("MoltenVK-macos.tar");
    if !tar_path.exists() {
        let mut response = reqwest::blocking::get(
            "https://github.com/KhronosGroup/MoltenVK/releases/latest/download/MoltenVK-macos.tar",
        )
        .unwrap()
        .error_for_status()
        .unwrap();
        response
            .copy_to(&mut std::fs::File::create(&tar_path).unwrap())
            .unwrap();
    }

    let dir_path = out_dir.join("MoltenVK");

    if !dir_path.exists() {
        let mut tar = tar::Archive::new(File::open(tar_path).unwrap());
        tar.unpack(&out_dir).unwrap();
    }

    println!(
        "cargo:rustc-link-search=native={}",
        dir_path
            .join("MoltenVK")
            .join("static")
            .join("MoltenVK.xcframework")
            .join("macos-arm64_x86_64")
            .display()
    );
    println!("cargo:rustc-link-lib=static=MoltenVK");
    println!("cargo:rustc-link-lib=framework=IOKit");
    println!("cargo:rustc-link-lib=framework=IOSurface");
    println!("cargo:rustc-link-lib=framework=Metal");
}
