#![cfg(all(feature = "internal", feature = "run"))]

use std::process::{Command, Output};

const CPU_RENDERING_WIDTH: u32 = 32;
const CPU_RENDERING_HEIGHT: u32 = 24;
const EXPECTED_RESOURCE: &[u8] = b"bundled asset payload\n";

#[test]
fn runner_forwards_guest_stdout_and_stderr() {
    let fixture_dir = fixture_dir("stdio");
    let temporary_dir = tempfile::tempdir().expect("create temporary directory");
    let persistent_dir = temporary_dir.path().join("persistent");

    let output = run_app(&fixture_dir.join("webrogue.json"), &persistent_dir);

    assert_success(&output, "run stdio fixture");
    assert_eq!(output.stdout, b"guest stdout\n");
    assert_eq!(output.stderr, b"guest stderr\n");
}

#[test]
fn persistent_storage_survives_between_runs() {
    let fixture_dir = fixture_dir("persistence");
    let temporary_dir = tempfile::tempdir().expect("create temporary directory");
    let persistent_dir = temporary_dir.path().join("persistent");

    for (expected_stdout, expected_count) in [(b"count: 1\n" as &[u8], 1), (b"count: 2\n", 2)] {
        let output = run_app(&fixture_dir.join("webrogue.json"), &persistent_dir);
        assert_success(&output, "run persistence fixture");
        assert_eq!(output.stdout, expected_stdout);
        assert!(output.stderr.is_empty());
        assert_eq!(
            std::fs::read_to_string(persistent_dir.join("state/count.txt"))
                .expect("read persisted counter")
                .trim(),
            expected_count.to_string(),
        );
    }
}

#[test]
fn cpu_rendering_matches_golden_image() {
    let fixture_dir = fixture_dir("cpu_rendering");
    let temporary_dir = tempfile::tempdir().expect("create temporary directory");
    let capture = temporary_dir.path().join("rendered.ppm");
    let persistent_dir = temporary_dir.path().join("persistent");

    let output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("internal")
        .arg("run")
        .arg(fixture_dir.join("webrogue.json"))
        .arg("--persistent-dir")
        .arg(&persistent_dir)
        .arg("--capture-ppm")
        .arg(&capture)
        .arg("--window-width")
        .arg(CPU_RENDERING_WIDTH.to_string())
        .arg("--window-height")
        .arg(CPU_RENDERING_HEIGHT.to_string())
        .output()
        .expect("run CPU-rendering fixture");
    assert_success(&output, "run CPU-rendering fixture");
    assert!(output.stdout.is_empty(), "unexpected guest stdout");
    assert!(output.stderr.is_empty(), "unexpected guest stderr");

    assert_eq!(
        std::fs::read(capture).expect("read captured PPM"),
        include_bytes!("fixtures/cpu_rendering/golden.ppm"),
        "captured CPU-rendered image differs from golden",
    );
}

#[test]
fn resources_are_available_from_webrogue_json() {
    let fixture_dir = fixture_dir("resources");
    let temporary_dir = tempfile::tempdir().expect("create temporary directory");
    let persistent_dir = temporary_dir.path().join("persistent");

    let output = run_app(&fixture_dir.join("webrogue.json"), &persistent_dir);
    assert_resource_output(&output, "run resource fixture from webrogue.json");
}

#[test]
fn resources_are_packaged_in_wrapp() {
    let fixture_dir = fixture_dir("resources");
    let temporary_dir = tempfile::tempdir().expect("create temporary directory");
    let wrapp = temporary_dir.path().join("resources.wrapp");
    let persistent_dir = temporary_dir.path().join("persistent");

    let pack_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("pack")
        .arg("--config")
        .arg(fixture_dir.join("webrogue.json"))
        .arg("--output")
        .arg(&wrapp)
        .output()
        .expect("pack resource fixture");
    assert_success(&pack_output, "pack resource fixture");

    let output = run_app(&wrapp, &persistent_dir);
    assert_resource_output(&output, "run resource fixture from WRAPP");
}

fn fixture_dir(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn run_app(app: &std::path::Path, persistent_dir: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("internal")
        .arg("run")
        .arg(app)
        .arg("--persistent-dir")
        .arg(persistent_dir)
        .output()
        .expect("run Webrogue fixture")
}

fn assert_resource_output(output: &Output, operation: &str) {
    assert_success(output, operation);
    assert_eq!(output.stdout, EXPECTED_RESOURCE);
    assert!(output.stderr.is_empty(), "unexpected guest stderr");
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
