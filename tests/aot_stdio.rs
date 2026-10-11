#![cfg(all(
    any(
        all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64"),
            any(target_env = "gnu", target_env = "musl")
        ),
        all(
            target_os = "windows",
            any(target_arch = "x86_64", target_arch = "aarch64")
        ),
        all(
            target_os = "macos",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )
    ),
    feature = "compile",
    feature = "llvm"
))]

use std::process::{Command, Output};

#[test]
fn aot_compiled_headless_app_forwards_guest_stdout_and_stderr() {
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stdio");
    let temporary_dir = tempfile::tempdir().expect("create temporary directory");
    let wrapp = temporary_dir.path().join("stdio.wrapp");

    let pack_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("pack")
        .arg("--config")
        .arg(fixture_dir.join("webrogue.json"))
        .arg("--output")
        .arg(&wrapp)
        .output()
        .expect("pack AOT stdio fixture");
    assert_success(&pack_output, "pack AOT stdio fixture");

    let artifacts_name = if cfg!(target_os = "macos") {
        "aot_artifacts_macos.zip"
    } else {
        "aot_artifacts.zip"
    };
    let artifacts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(artifacts_name);
    let executable = compile_aot_fixture(&wrapp, temporary_dir.path(), &artifacts);

    let data_dir = temporary_dir.path().join("data");
    std::fs::create_dir(&data_dir).expect("create isolated app data directory");
    let mut run = Command::new(&executable);
    run.env("XDG_DATA_HOME", data_dir)
        .env_remove("WAYLAND_DISPLAY");
    let output = run.output().expect("launch AOT stdio fixture");

    assert_success(&output, "run AOT stdio fixture");
    assert_eq!(output.stdout, b"guest stdout\n");
    assert_eq!(output.stderr, b"guest stderr\n");
}

#[cfg(target_os = "linux")]
fn compile_aot_fixture(
    wrapp: &std::path::Path,
    temporary_dir: &std::path::Path,
    artifacts: &std::path::Path,
) -> std::path::PathBuf {
    let executable = temporary_dir.join("stdio-app");
    let libc = if cfg!(target_env = "musl") {
        "musl"
    } else {
        "glibc"
    };
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    };
    let compile_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("compile")
        .arg("linux")
        .arg(wrapp)
        .arg(&executable)
        .arg("--libc")
        .arg(libc)
        .arg("--arch")
        .arg(arch)
        .env("WEBROGUE_ARTIFACTS_PATH", artifacts)
        .output()
        .expect("compile Linux AOT stdio fixture");
    assert_success(&compile_output, "compile Linux AOT stdio fixture");
    executable
}

#[cfg(target_os = "windows")]
fn compile_aot_fixture(
    wrapp: &std::path::Path,
    temporary_dir: &std::path::Path,
    artifacts: &std::path::Path,
) -> std::path::PathBuf {
    let executable = temporary_dir.join("stdio-app.exe");
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    };
    let compile_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("compile")
        .arg("windows")
        .arg(wrapp)
        .arg(&executable)
        .arg("--console")
        .arg("--arch")
        .arg(arch)
        .env("WEBROGUE_ARTIFACTS_PATH", artifacts)
        .output()
        .expect("compile Windows AOT stdio fixture");
    assert_success(&compile_output, "compile Windows AOT stdio fixture");
    executable
}

#[cfg(target_os = "macos")]
fn compile_aot_fixture(
    wrapp: &std::path::Path,
    temporary_dir: &std::path::Path,
    artifacts: &std::path::Path,
) -> std::path::PathBuf {
    let build_dir = temporary_dir.join("macos-app");
    let compile_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("compile")
        .arg("xcode")
        .arg(wrapp)
        .arg(&build_dir)
        .arg("macos")
        .arg("--config")
        .arg("local")
        .env("WEBROGUE_ARTIFACTS_PATH", artifacts)
        .output()
        .expect("compile macOS AOT stdio fixture");
    assert_success(&compile_output, "compile macOS AOT stdio fixture");

    std::fs::read_dir(build_dir.join("Output.app/Contents/MacOS"))
        .expect("find macOS AOT executable")
        .map(|entry| entry.expect("read macOS app bundle entry").path())
        .find(|path| path.is_file())
        .expect("macOS app bundle contains no executable")
}

fn assert_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed with {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
