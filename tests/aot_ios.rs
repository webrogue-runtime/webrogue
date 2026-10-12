#![cfg(all(target_os = "macos", feature = "compile", feature = "llvm"))]

use std::{
    io::Read,
    process::{Command, Output, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

const BUNDLE_ID: &str = "io.github.webrogue.tests.Stdio";

#[test]
fn ios_simulator_aot_app_forwards_guest_stdout_and_stderr() {
    let fixture_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/stdio");
    let temporary_dir = tempfile::tempdir().expect("create temporary directory");
    let mobile_fixture_dir = temporary_dir.path().join("fixture");
    std::fs::create_dir(&mobile_fixture_dir).expect("create iOS fixture directory");
    std::fs::copy(
        fixture_dir.join("main.wasm"),
        mobile_fixture_dir.join("main.wasm"),
    )
    .expect("copy iOS fixture guest");

    // Use a predictable bundle identifier. The Xcode project capitalizes the first character of
    // the final identifier component when it generates PRODUCT_BUNDLE_IDENTIFIER.
    let config = std::fs::read_to_string(fixture_dir.join("webrogue.json"))
        .expect("read stdio fixture config")
        .replace(
            "io.github.webrogue-runtime.tests.stdio",
            "io.github.webrogue.tests.stdio",
        );
    assert!(
        config.contains("io.github.webrogue.tests.stdio"),
        "iOS bundle identifier was not applied to the fixture"
    );
    std::fs::write(mobile_fixture_dir.join("webrogue.json"), config)
        .expect("write iOS fixture config");

    let artifacts =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("aot_artifacts_macos.zip");
    let build_dir = temporary_dir.path().join("ios-app");
    let compile_output = Command::new(env!("CARGO_BIN_EXE_webrogue"))
        .arg("compile")
        .arg("xcode")
        .arg(mobile_fixture_dir.join("webrogue.json"))
        .arg(&build_dir)
        .arg("ios")
        .arg("--simulator")
        .arg("--config")
        .arg("local")
        .env("WEBROGUE_ARTIFACTS_PATH", artifacts)
        .output()
        .expect("compile iOS Simulator AOT app");
    assert_success(&compile_output, "compile iOS Simulator AOT app");

    let app = build_dir.join("Output.app");
    assert!(app.is_dir(), "iOS compiler did not produce Output.app");
    let device = booted_or_available_iphone();

    let install_output = simctl(&[
        "install",
        &device,
        app.to_str().expect("app bundle path is UTF-8"),
    ]);
    assert_success(&install_output, "install iOS AOT app in Simulator");

    let mut launch = Command::new("xcrun")
        .args(["simctl", "launch", "--console", &device, BUNDLE_ID])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch iOS AOT app with console capture");

    let (sender, receiver) = mpsc::channel();
    let stdout_reader = read_console_stream(
        launch.stdout.take().expect("capture Simulator stdout"),
        true,
        sender.clone(),
    );
    let stderr_reader = read_console_stream(
        launch.stderr.take().expect("capture Simulator stderr"),
        false,
        sender.clone(),
    );
    drop(sender);

    let deadline = Instant::now() + Duration::from_secs(30);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut closed_streams = 0;
    while Instant::now() < deadline && closed_streams < 2 {
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok((is_stdout, Some(data))) => {
                if is_stdout {
                    stdout.extend(data);
                } else {
                    stderr.extend(data);
                }
            }
            Ok((_, None)) => closed_streams += 1,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        if contains_guest_messages(&stdout, &stderr) {
            break;
        }
        if launch
            .try_wait()
            .expect("check Simulator launch process")
            .is_some()
            && closed_streams == 2
        {
            break;
        }
    }

    let _ = simctl(&["terminate", &device, BUNDLE_ID]);
    if launch
        .try_wait()
        .expect("check Simulator launch process")
        .is_none()
    {
        let _ = launch.kill();
    }
    let _ = launch.wait();
    stdout_reader.join().expect("join Simulator stdout reader");
    stderr_reader.join().expect("join Simulator stderr reader");
    while let Ok((is_stdout, data)) = receiver.try_recv() {
        if let Some(data) = data {
            if is_stdout {
                stdout.extend(data);
            } else {
                stderr.extend(data);
            }
        }
    }
    let _ = simctl(&["shutdown", &device]);

    let stdout = String::from_utf8_lossy(&stdout);
    let stderr = String::from_utf8_lossy(&stderr);
    let combined = format!("{stdout}\n{stderr}");
    assert!(
        combined.contains("guest stdout"),
        "guest stdout not found in iOS Simulator console output:\n{combined}"
    );
    assert!(
        combined.contains("guest stderr"),
        "guest stderr not found in iOS Simulator console output:\n{combined}"
    );
}

fn booted_or_available_iphone() -> String {
    let output = simctl(&["list", "devices", "available"]);
    assert_success(&output, "list available iOS Simulators");
    let devices = String::from_utf8_lossy(&output.stdout);
    let (device, booted) = devices
        .lines()
        .find_map(|line| {
            let line = line.trim();
            if !line.starts_with("iPhone ") {
                return None;
            }
            let (_, rest) = line.split_once('(')?;
            let (device_id, _) = rest.split_once(')')?;
            Some((device_id.to_owned(), line.contains("(Booted)")))
        })
        .expect("no available iPhone simulator found");

    if !booted {
        let boot_output = simctl(&["boot", &device]);
        assert_success(&boot_output, "boot iOS Simulator");
    }
    let boot_status = simctl(&["bootstatus", &device, "-b"]);
    assert_success(&boot_status, "wait for iOS Simulator boot");
    device
}

fn read_console_stream<R: Read + Send + 'static>(
    mut stream: R,
    is_stdout: bool,
    sender: mpsc::Sender<(bool, Option<Vec<u8>>)>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) | Err(_) => {
                    let _ = sender.send((is_stdout, None));
                    break;
                }
                Ok(size) => {
                    if sender
                        .send((is_stdout, Some(buffer[..size].to_vec())))
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    })
}

fn contains_guest_messages(stdout: &[u8], stderr: &[u8]) -> bool {
    let output = String::from_utf8_lossy(stdout);
    let error = String::from_utf8_lossy(stderr);
    let combined = format!("{output}\n{error}");
    combined.contains("guest stdout") && combined.contains("guest stderr")
}

fn simctl(args: &[&str]) -> Output {
    Command::new("xcrun")
        .arg("simctl")
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("run simctl {:?}: {error}", args))
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
