use std::str::FromStr as _;

fn main() {
    let crate_manifest_dir =
        std::path::PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    {
        let dst = cmake::Config::new(
            crate_manifest_dir,
        )
        .build();
        // println!(
        //     "cargo:rustc-link-search=native={}",
        //     dst.join("lib").display()
        // );
        // println!("cargo:rustc-link-lib=static=wrgfxfallback");
        // println!("cargo:rustc-link-lib=static=SDL2");
    }
}
