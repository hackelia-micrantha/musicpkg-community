// SPDX-License-Identifier: MPL-2.0

//! MUSICPKG v0.2 conformance harness.

mod fixture;
mod negative;
mod positive;
mod report;
mod scenario;

use std::path::Path;

use report::fixture_defect;
pub use report::{CaseResult, CaseStatus, EvidenceClass, Report, Summary};

const CONTRACT_ONLY_NEGATIVE_CASES: &[&str] = &[
    "unknown-mandatory-profile",
    "zip-path-traversal",
    "zip-duplicate-path",
    "checkpoint-threshold-short",
    "checkpoint-duplicate-validator",
    "validator-rotation-hash-mismatch",
    "stale-but-valid-current-title",
    "double-transfer",
];

fn is_contract_assertion(case: &CaseResult) -> bool {
    case.group == "scenario"
        || (case.group == "negative" && CONTRACT_ONLY_NEGATIVE_CASES.contains(&case.id.as_str()))
}

fn classify_evidence(cases: &mut [CaseResult]) {
    for case in cases {
        if !is_contract_assertion(case) || case.status == CaseStatus::FixtureDefect {
            continue;
        }
        case.evidence = Some(EvidenceClass::ContractAssertion);
        if case.status == CaseStatus::ImplementationMismatch {
            case.status = CaseStatus::ContractAssertionMismatch;
        }
    }
}

pub fn run_all(fixtures_dir: &Path) -> Report {
    let crypto = match fixture::read_json(&fixtures_dir.join("crypto.json")) {
        Ok(value) => value,
        Err(error) => {
            return Report::from_cases(
                "unknown",
                vec![fixture_defect("fixture", "crypto.json", error.to_string())],
            );
        }
    };
    let negative = match fixture::read_json(&fixtures_dir.join("negative.json")) {
        Ok(value) => value,
        Err(error) => {
            return Report::from_cases(
                "unknown",
                vec![fixture_defect(
                    "fixture",
                    "negative.json",
                    error.to_string(),
                )],
            );
        }
    };
    let scenarios = match fixture::read_json(&fixtures_dir.join("scenarios.json")) {
        Ok(value) => value,
        Err(error) => {
            return Report::from_cases(
                "unknown",
                vec![fixture_defect(
                    "fixture",
                    "scenarios.json",
                    error.to_string(),
                )],
            );
        }
    };

    let crypto_version = match fixture::version(&crypto) {
        Ok(value) => value.to_owned(),
        Err(error) => {
            return Report::from_cases(
                "unknown",
                vec![fixture_defect(
                    "fixture",
                    "crypto.version",
                    error.to_string(),
                )],
            );
        }
    };
    for (name, value) in [("negative", &negative), ("scenarios", &scenarios)] {
        match fixture::version(value) {
            Ok(version) if version == crypto_version => {}
            Ok(version) => {
                return Report::from_cases(
                    crypto_version,
                    vec![fixture_defect(
                        "fixture",
                        name,
                        format!("fixture version mismatch: {version}"),
                    )],
                );
            }
            Err(error) => {
                return Report::from_cases(
                    crypto_version,
                    vec![fixture_defect("fixture", name, error.to_string())],
                );
            }
        }
    }

    let mut cases = positive::run(&crypto);
    cases.extend(negative::run(&crypto, &negative));
    cases.extend(scenario::run(&scenarios));
    classify_evidence(&mut cases);
    Report::from_cases(crypto_version, cases)
}
