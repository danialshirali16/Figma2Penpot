use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(relative)
}

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "f2p-cli-test-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_f2p-cli"))
        .args(args)
        .output()
        .expect("run f2p-cli")
}

#[test]
fn convert_writes_archive_and_report() {
    let out = temp_dir("convert");
    let output = run(&[
        "convert",
        fixture("circle.fig").to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Validation: passed"), "{stdout}");

    let penpot = out.join("circle.penpot");
    let report = out.join("circle.report.json");
    assert!(penpot.exists());
    assert!(report.exists());
    let bytes = std::fs::read(&penpot).unwrap();
    assert!(bytes.starts_with(b"PK"));

    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    assert_eq!(report["counts"]["detected"], 2);
    assert_eq!(report["counts"]["converted"], 2);
    assert_eq!(report["validation"], "passed");
    assert_eq!(report["targetPenpot"], "2.18.5");

    let _ = std::fs::remove_dir_all(out);
}

#[test]
fn write_failure_reports_failure_without_a_partial_archive() {
    let out = temp_dir("write-failure");
    let archive = out.join("circle.penpot");
    std::fs::create_dir(&archive).unwrap();
    let output = run(&[
        "convert",
        fixture("circle.fig").to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--force",
    ]);
    assert!(!output.status.success());
    assert!(archive.is_dir());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join("circle.report.json")).unwrap()).unwrap();
    assert_eq!(report["status"], "failed");
    let leftovers: Vec<_> = std::fs::read_dir(&out)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp-"))
        .collect();
    assert!(leftovers.is_empty(), "temporary files were not cleaned up");
    let _ = std::fs::remove_dir_all(out);
}

#[test]
fn convert_refuses_to_overwrite_without_force() {
    let out = temp_dir("force");
    let source = fixture("circle.fig");
    let args = [
        "convert",
        source.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ];
    assert!(run(&args).status.success());
    let second = run(&args);
    assert!(!second.status.success());
    assert!(String::from_utf8_lossy(&second.stderr).contains("already exists"));

    let mut forced = args.to_vec();
    forced.push("--force");
    assert!(run(&forced).status.success());
    let _ = std::fs::remove_dir_all(out);
}

#[test]
fn convert_rejects_bad_input_without_panicking() {
    let output = run(&[
        "convert",
        fixture("README.md").to_str().unwrap(),
        "--out",
        std::env::temp_dir().to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a Figma file"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}
