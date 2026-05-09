use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
    str::FromStr as _,
};

fn main() {
    let crate_manifest_dir =
        PathBuf::from_str(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap();

    const ASSETS: [(&str, &str); 2] = [
        (
            "webview_assets/main.js",
            "http://webrogue.dev/webrogue-hub-webview-assets/launcher/main.js",
        ),
        (
            "webview_assets/style.css",
            "http://webrogue.dev/webrogue-hub-webview-assets/launcher/style.css",
        ),
    ];
    let assets = ASSETS
        .iter()
        .map(|(filename, url)| (crate_manifest_dir.join(filename), url))
        .collect::<Vec<_>>();

    let mut has_missing = false;

    for (filename, _url) in &assets {
        println!("cargo::rerun-if-changed={}", filename.to_str().unwrap());
        has_missing |= !filename.exists();
    }
    if !has_missing {
        return;
    }

    for (filename, url) in &assets {
        create_dir_all(filename.parent().unwrap()).unwrap();
        let response = reqwest::blocking::get(**url).unwrap();
        assert!(response.status().is_success());
        let data = response.bytes().unwrap().to_vec();
        File::create(filename).unwrap().write_all(&data).unwrap();
    }
}
