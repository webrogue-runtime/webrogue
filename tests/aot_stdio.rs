#![cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64"),
    any(target_env = "gnu", target_env = "musl"),
    feature = "compile",
    feature = "llvm"
))]

use std::process::{Command, Output};

#[test]
fn aot_compiled_headless_app_forwards_guest_stdout_and_stderr() {
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stdio");
    let temporary_dir = tempfile::tempdir().expect("create temporary directory");
    let wrapp = temporary_dir.path().join("stdio.wrapp");
    let executable = temporary_dir.path().join("stdio-app");
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

    let pack_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("pack")
        .arg("--config")
        .arg(fixture_dir.join("webrogue.json"))
        .arg("--output")
        .arg(&wrapp)
        .output()
        .expect("pack AOT stdio fixture");
    assert_success(&pack_output, "pack AOT stdio fixture");

    let artifacts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("aot_artifacts.zip");
    let compile_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("compile")
        .arg("linux")
        .arg(&wrapp)
        .arg(&executable)
        .arg("--libc")
        .arg(libc)
        .arg("--arch")
        .arg(arch)
        .env("WEBROGUE_ARTIFACTS_PATH", artifacts)
        .output()
        .expect("compile AOT stdio fixture");
    assert_success(&compile_output, "compile AOT stdio fixture");

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

fn assert_success(output: &Output, operation: &str) {
    assert!(
        output.status.success(),
        "{operation} failed with {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
