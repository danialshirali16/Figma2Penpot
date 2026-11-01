use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Baseline {
    schema_version: u32,
    fixtures: Vec<BaselineFixture>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BaselineFixture {
    fixture: String,
    detected: u64,
    converted: u64,
    approximated: u64,
    skipped: u64,
    unsupported: u64,
    failed: u64,
    status: String,
    wall_ms: u64,
    peak_rss_mb: f64,
}

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(relative)
}

fn temp_dir() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "f2p-baseline-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn conversion_counts_and_status_match_the_checked_in_baseline() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/benchmarks/baseline.json");
    let baseline: Baseline = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(baseline.schema_version, 1);
    assert!(!baseline.fixtures.is_empty());
    let out = temp_dir();

    for expected in &baseline.fixtures {
        let input = fixture(&expected.fixture);
        let output = Command::new(env!("CARGO_BIN_EXE_f2p-cli"))
            .args([
                "convert",
                input.to_str().unwrap(),
                "--out",
                out.to_str().unwrap(),
                "--force",
            ])
            .output()
            .expect("run f2p-cli");
        assert!(
            output.status.success(),
            "{} failed: {}",
            expected.fixture,
            String::from_utf8_lossy(&output.stderr)
        );
        let stem = input.file_stem().unwrap().to_string_lossy();
        let report_path = out.join(format!("{stem}.report.json"));
        let archive_path = out.join(format!("{stem}.penpot"));
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&report_path).unwrap()).unwrap();
        let counts = &report["counts"];
        assert_eq!(
            counts["detected"], expected.detected,
            "{} detected",
            expected.fixture
        );
        assert_eq!(
            counts["converted"], expected.converted,
            "{} converted",
            expected.fixture
        );
        assert_eq!(
            counts["approximated"], expected.approximated,
            "{} approximated",
            expected.fixture
        );
        assert_eq!(
            counts["skipped"], expected.skipped,
            "{} skipped",
            expected.fixture
        );
        assert_eq!(
            counts["unsupported"], expected.unsupported,
            "{} unsupported",
            expected.fixture
        );
        assert_eq!(
            counts["failed"], expected.failed,
            "{} failed",
            expected.fixture
        );
        assert_eq!(
            report["status"], expected.status,
            "{} status",
            expected.fixture
        );
        assert_eq!(
            report["validation"], "passed",
            "{} validation",
            expected.fixture
        );
        let bytes = std::fs::read(&archive_path).unwrap();
        penpot_writer::validate_archive(&bytes).unwrap_or_else(|error| {
            panic!("{} archive failed validation: {error}", expected.fixture)
        });

        // The reference numbers are documentation; CI asserts correctness, not speed.
        assert!(expected.wall_ms > 0, "{} wallMs", expected.fixture);
        assert!(expected.peak_rss_mb > 0.0, "{} peakRssMb", expected.fixture);
    }
    let _ = std::fs::remove_dir_all(out);
}
