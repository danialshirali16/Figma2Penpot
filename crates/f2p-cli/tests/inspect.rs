use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(relative)
}

#[test]
fn inspect_prints_verified_metadata() {
    let output = Command::new(env!("CARGO_BIN_EXE_f2p-cli"))
        .args(["inspect", fixture("circle.fig").to_str().unwrap()])
        .output()
        .expect("run f2p-cli");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fig-kiwi (version 101)"), "{stdout}");
    assert!(stdout.contains("Nodes:           5 total"), "{stdout}");
    assert!(stdout.contains("Page 1"), "{stdout}");
    assert!(stdout.contains("Internal Only Canvas"), "{stdout}");
    assert!(
        stdout.contains("Internal Only Canvas") && stdout.contains("[internal]"),
        "{stdout}"
    );
}

#[test]
fn inspect_reports_bad_input_without_a_panic() {
    let output = Command::new(env!("CARGO_BIN_EXE_f2p-cli"))
        .args(["inspect", fixture("README.md").to_str().unwrap()])
        .output()
        .expect("run f2p-cli");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:"), "{stderr}");
    assert!(stderr.contains("not a Figma file"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}
