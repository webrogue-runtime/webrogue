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
}
