// SPDX-License-Identifier: MPL-2.0

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use musicpkg_conformance::{CaseStatus, Report, run_all};

fn default_fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-vectors/v0.2")
}

fn usage() -> &'static str {
    "usage: musicpkg-conformance [--fixtures PATH] [--json]"
}

fn parse_args() -> Result<(PathBuf, bool), String> {
    let mut fixtures = default_fixtures();
    let mut json = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fixtures" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--fixtures requires a path".to_owned())?;
                fixtures = PathBuf::from(value);
            }
            "--json" => json = true,
            "-h" | "--help" => return Err(usage().to_owned()),
            other => return Err(format!("unknown argument: {other}\n{}", usage())),
        }
    }
    Ok((fixtures, json))
}

fn render_human(report: &Report) -> String {
    let verdict = if report.exit_code() == 0 {
        "PASS"
    } else {
        "FAIL"
    };
    let mut out = format!(
        "MUSICPKG v0.2 conformance: {verdict}\n{} implementation passes; {} contract assertions passed; {} implementation mismatches; {} contract assertion mismatches; {} fixture defects\n",
        report.summary.implementation_passed,
        report.summary.contract_assertions_passed,
        report.summary.implementation_mismatch,
        report.summary.contract_assertion_mismatch,
        report.summary.fixture_defect,
    );
    for case in &report.cases {
        if case.status != CaseStatus::Pass {
            out.push_str(&format!("{:?} {}:{}", case.status, case.group, case.id));
            if let Some(detail) = &case.detail {
                out.push_str(&format!(" — {detail}"));
            }
            out.push('\n');
        }
    }
    out
}

fn main() -> ExitCode {
    let (fixtures, json) = match parse_args() {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::from(2);
        }
    };
    let report = run_all(&fixtures);
    if json {
        match serde_json::to_string_pretty(&report) {
            Ok(encoded) => println!("{encoded}"),
            Err(error) => {
                eprintln!("serialize conformance report: {error}");
                return ExitCode::from(2);
            }
        }
    } else {
        print!("{}", render_human(&report));
    }
    ExitCode::from(report.exit_code() as u8)
}
