#![cfg(all(
    target_os = "linux",
    target_arch = "x86_64",
    feature = "compile",
    feature = "llvm"
))]

use std::{
    process::{Command, Output},
    thread,
    time::{Duration, Instant},
};

const APPLICATION_ID: &str = "io.github.webrogue.tests.stdio";
const DEBUG_APPLICATION_ID: &str = "io.github.webrogue.tests.stdio.dev";

#[test]
fn android_aot_app_forwards_guest_stdout_and_stderr_to_logcat() {
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stdio");
    let temporary_dir = tempfile::tempdir().expect("create temporary directory");
    let mobile_fixture_dir = temporary_dir.path().join("fixture");
    std::fs::create_dir(&mobile_fixture_dir).expect("create Android fixture directory");
    std::fs::copy(
        fixture_dir.join("main.wasm"),
        mobile_fixture_dir.join("main.wasm"),
    )
    .expect("copy Android fixture guest");

    // Android application IDs cannot contain hyphens. Keep the desktop fixture's ID intact and
    // give this temporary WRAPP a valid package ID instead.
    let config = std::fs::read_to_string(fixture_dir.join("webrogue.json"))
        .expect("read stdio fixture config");
    let config = config.replace("io.github.webrogue-runtime.tests.stdio", APPLICATION_ID);
    assert!(
        config.contains(APPLICATION_ID),
        "Android-safe application ID was not applied to the fixture"
    );
    std::fs::write(mobile_fixture_dir.join("webrogue.json"), config)
        .expect("write Android fixture config");

    let wrapp = temporary_dir.path().join("stdio.wrapp");
    let pack_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("pack")
        .arg("--config")
        .arg(mobile_fixture_dir.join("webrogue.json"))
        .arg("--output")
        .arg(&wrapp)
        .output()
        .expect("pack Android AOT stdio fixture");
    assert_success(&pack_output, "pack Android AOT stdio fixture");

    let artifacts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("aot_artifacts.zip");
    let build_dir = temporary_dir.path().join("android-app");
    let apk = temporary_dir.path().join("stdio.apk");
    let compile_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("compile")
        .arg("android")
        .arg(&wrapp)
        .arg(&build_dir)
        .arg("--arch")
        .arg("aarch64,x86_64")
        .arg("--output")
        .arg(&apk)
        .env("WEBROGUE_ARTIFACTS_PATH", artifacts)
        .output()
        .expect("compile dual-ABI Android AOT app");
    assert_success(&compile_output, "compile dual-ABI Android AOT app");
    assert!(apk.is_file(), "Android compiler did not produce the APK");

    let install_output = adb(&["install", "-r", apk.to_str().expect("APK path is UTF-8")]);
    assert_success(&install_output, "install Android AOT app");

    let clear_log_output = adb(&["logcat", "-c"]);
    assert_success(&clear_log_output, "clear Android logcat");

    let launch_output = adb(&[
        "shell",
        "am",
        "start",
        "-n",
        &format!("{DEBUG_APPLICATION_ID}/android.app.NativeActivity"),
    ]);
    assert_success(&launch_output, "launch Android AOT app");

    let deadline = Instant::now() + Duration::from_secs(30);
    let mut logs = String::new();
    let mut app_was_running = false;
    while Instant::now() < deadline {
        let log_output = adb(&["logcat", "-d", "-v", "raw"]);
        assert_success(&log_output, "read Android logcat");
        logs = String::from_utf8_lossy(&log_output.stdout).into_owned();
        if logs.contains("guest stdout") && logs.contains("guest stderr") {
            break;
        }
        let pid_output = adb(&["shell", "pidof", DEBUG_APPLICATION_ID]);
        let app_is_running = pid_output.status.success() && !pid_output.stdout.is_empty();
        app_was_running |= app_is_running;
        if app_was_running && !app_is_running {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }

    let _ = adb(&["shell", "am", "force-stop", DEBUG_APPLICATION_ID]);
    assert!(
        logs.lines().any(|line| line.contains("guest stdout")),
        "guest stdout not found in Android logcat; relevant device logs:\n{}",
        relevant_logs(&logs)
    );
    assert!(
        logs.lines().any(|line| line.contains("guest stderr")),
        "guest stderr not found in Android logcat; relevant device logs:\n{}",
        relevant_logs(&logs)
    );
}

fn relevant_logs(logs: &str) -> String {
    logs.lines()
        .filter(|line| {
            line.contains(DEBUG_APPLICATION_ID)
                || line.contains("guest stdout")
                || line.contains("guest stderr")
                || line.contains("Fatal signal")
                || line.contains("Abort message:")
                || line.contains("page-aligned")
                || line.contains("memory image")
        })
        .take(100)
        .collect::<Vec<_>>()
        .join("\n")
}

fn adb(args: &[&str]) -> Output {
    Command::new("adb")
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("run adb {:?}: {error}", args))
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
