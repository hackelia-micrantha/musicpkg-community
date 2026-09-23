// SPDX-License-Identifier: MPL-2.0

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-vectors/v0.2")
}

struct TempFixtureDir(PathBuf);

impl TempFixtureDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "musicpkg-conformance-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create fixture tempdir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFixtureDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_fixtures() -> TempFixtureDir {
    let dir = TempFixtureDir::new();
    for name in ["crypto.json", "negative.json", "scenarios.json"] {
        fs::copy(fixture_source().join(name), dir.path().join(name)).expect("copy fixture");
    }
    dir
}

fn run_json(fixtures: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_musicpkg-conformance"))
        .arg("--fixtures")
        .arg(fixtures)
        .arg("--json")
        .output()
        .expect("run conformance CLI")
}

fn report(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("CLI emits JSON report")
}

#[test]
fn valid_fixture_set_exits_zero() {
    let output = run_json(&fixture_source());
    assert_eq!(output.status.code(), Some(0));
    let value = report(&output);
    assert_eq!(value["schema"], "musicpkg-conformance-report-v2");
    assert_eq!(value["summary"]["total"], 49);
    assert_eq!(value["summary"]["implementation_passed"], 26);
    assert_eq!(value["summary"]["contract_assertions_passed"], 23);
    assert_eq!(value["summary"]["implementation_mismatch"], 0);
    assert_eq!(value["summary"]["contract_assertion_mismatch"], 0);
}

#[test]
fn implementation_mismatched_expectation_exits_one() {
    let dir = copy_fixtures();
    let path = dir.path().join("negative.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["cases"][0]["expected"] = json!("WRONG_EXPECTATION");
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    let output = run_json(dir.path());
    assert_eq!(output.status.code(), Some(1));
    let value = report(&output);
    assert_eq!(value["summary"]["implementation_mismatch"], 1);
    assert_eq!(value["summary"]["contract_assertion_mismatch"], 0);
}

#[test]
fn contract_assertion_mismatch_exits_one_without_claiming_implementation_failure() {
    let dir = copy_fixtures();
    let path = dir.path().join("scenarios.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["scenarios"][0]["expected"]["playback"] = json!("denied");
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    let output = run_json(dir.path());
    assert_eq!(output.status.code(), Some(1));
    let value = report(&output);
    assert_eq!(value["summary"]["implementation_mismatch"], 0);
    assert_eq!(value["summary"]["contract_assertion_mismatch"], 1);
}

#[test]
fn missing_fixtures_exit_two() {
    let dir = TempFixtureDir::new();
    let output = run_json(dir.path());
    assert_eq!(output.status.code(), Some(2));
    let value = report(&output);
    assert!(value["summary"]["fixture_defect"].as_u64().unwrap() > 0);
}

#[test]
fn malformed_fixture_field_exits_two_without_panicking() {
    let dir = copy_fixtures();
    let path = dir.path().join("crypto.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["track_record"]["k_album"] = json!(7);
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    let output = run_json(dir.path());
    assert_eq!(output.status.code(), Some(2));
    let value = report(&output);
    assert!(value["summary"]["fixture_defect"].as_u64().unwrap() > 0);
    assert!(!String::from_utf8_lossy(&output.stderr).contains("panicked"));
}

#[test]
fn human_output_is_concise_and_separates_evidence_classes() {
    let output = Command::new(env!("CARGO_BIN_EXE_musicpkg-conformance"))
        .arg("--fixtures")
        .arg(fixture_source())
        .output()
        .expect("run conformance CLI");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("MUSICPKG v0.2 conformance: PASS"));
    assert!(stdout.contains("26 implementation passes"));
    assert!(stdout.contains("23 contract assertions passed"));
    assert!(stdout.contains("0 implementation mismatches"));
    assert!(stdout.contains("0 contract assertion mismatches"));
    assert!(stdout.contains("0 fixture defects"));
}

#[test]
fn unknown_negative_case_is_fixture_defect() {
    let dir = copy_fixtures();
    let path = dir.path().join("negative.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["cases"][0]["id"] = json!("future-negative-case");
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    let output = run_json(dir.path());
    assert_eq!(output.status.code(), Some(2));
    let value = report(&output);
    assert_eq!(value["summary"]["fixture_defect"], 1);
}

#[test]
fn unknown_scenario_is_fixture_defect() {
    let dir = copy_fixtures();
    let path = dir.path().join("scenarios.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["scenarios"][0]["id"] = json!("future-scenario");
    fs::write(&path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();

    let output = run_json(dir.path());
    assert_eq!(output.status.code(), Some(2));
    let value = report(&output);
    assert_eq!(value["summary"]["fixture_defect"], 1);
}
