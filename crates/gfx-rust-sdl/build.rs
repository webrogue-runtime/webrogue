fn main() {
    let root = std::env::current_dir().unwrap();
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_dir = root.join(out_dir);
    let schema_dir = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let target = std::env::var("TARGET").unwrap();
    let target_os = target.as_str().splitn(3, '-').nth(2).unwrap();
    if target_os == "darwin" {
        download_macos_fat(schema_dir);
    };
    if target_os.contains("windows") {
        download_windows_x64(schema_dir);
    };
}

fn download_macos_fat(output_path: &std::path::Path) {
    println!(
        "cargo:rerun-if-changed={}",
        output_path.join("libEGL.dylib").to_string_lossy()
    );
    println!(
        "cargo:rerun-if-changed={}",
        output_path.join("libGLESv2.dylib").to_string_lossy()
    );
    if output_path.join("libGLESv2.dylib").exists() && output_path.join("libEGL.dylib").exists() {
        return;
    }
    let archive_bytes = reqwest::blocking::get(
        "https://github.com/webrogue-runtime/angle-builder/releases/latest/download/macos_fat.zip",
    )
    .unwrap()
    .bytes()
    .unwrap();
    zip_extract::extract(std::io::Cursor::new(archive_bytes), &output_path, true).unwrap();
}

fn download_windows_x64(output_path: &std::path::Path) {
    println!(
        "cargo:rerun-if-changed={}",
        output_path.join("libEGL.dll").to_string_lossy()
    );
    println!(
        "cargo:rerun-if-changed={}",
        output_path.join("libGLESv2.dll").to_string_lossy()
    );
    if output_path.join("libGLESv2.dll").exists() && output_path.join("libEGL.dll").exists() {
        return;
    }
    let archive_bytes = reqwest::blocking::get(
        "https://github.com/webrogue-runtime/angle-builder/releases/latest/download/windows_x64.zip",
    )
    .unwrap()
    .bytes()
    .unwrap();
    zip_extract::extract(std::io::Cursor::new(archive_bytes), &output_path, true).unwrap();
}
