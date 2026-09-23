// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

use musicpkg_conformance::{CaseStatus, EvidenceClass, run_all};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-vectors/v0.2")
}

#[test]
fn run_all_reports_the_complete_v02_fixture_set() {
    let report = run_all(&fixtures());
    assert_eq!(report.schema, "musicpkg-conformance-report-v2");
    assert_eq!(report.cases.len(), 49);
    assert_eq!(report.summary.total, 49);
    assert_eq!(report.summary.implementation_passed, 26);
    assert_eq!(report.summary.contract_assertions_passed, 23);
    assert_eq!(report.summary.implementation_mismatch, 0);
    assert_eq!(report.summary.contract_assertion_mismatch, 0);
    assert_eq!(report.summary.fixture_defect, 0);
    assert!(
        report
            .cases
            .iter()
            .all(|case| case.status == CaseStatus::Pass)
    );

    let positive = report
        .cases
        .iter()
        .filter(|case| case.group == "positive")
        .count();
    let negative = report
        .cases
        .iter()
        .filter(|case| case.group == "negative")
        .count();
    let scenario = report
        .cases
        .iter()
        .filter(|case| case.group == "scenario")
        .count();
    assert_eq!((positive, negative, scenario), (9, 25, 15));

    let implementation = report
        .cases
        .iter()
        .filter(|case| case.evidence == Some(EvidenceClass::Implementation))
        .count();
    let contract_assertions = report
        .cases
        .iter()
        .filter(|case| case.evidence == Some(EvidenceClass::ContractAssertion))
        .count();
    assert_eq!((implementation, contract_assertions), (26, 23));

    assert!(
        report
            .cases
            .iter()
            .filter(|case| case.group == "scenario")
            .all(|case| case.evidence == Some(EvidenceClass::ContractAssertion))
    );

    for id in [
        "unknown-mandatory-profile",
        "zip-path-traversal",
        "zip-duplicate-path",
        "checkpoint-threshold-short",
        "checkpoint-duplicate-validator",
        "validator-rotation-hash-mismatch",
        "stale-but-valid-current-title",
        "double-transfer",
    ] {
        let case = report
            .cases
            .iter()
            .find(|case| case.group == "negative" && case.id == id)
            .expect("contract-only negative case exists");
        assert_eq!(case.evidence, Some(EvidenceClass::ContractAssertion));
    }
}
