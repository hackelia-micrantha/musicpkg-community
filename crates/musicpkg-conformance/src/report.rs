// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

pub const REPORT_SCHEMA: &str = "musicpkg-conformance-report-v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    Implementation,
    ContractAssertion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    Pass,
    ImplementationMismatch,
    ContractAssertionMismatch,
    FixtureDefect,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseResult {
    pub id: String,
    pub group: String,
    pub status: CaseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<EvidenceClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub total: usize,
    pub implementation_passed: usize,
    pub contract_assertions_passed: usize,
    pub implementation_mismatch: usize,
    pub contract_assertion_mismatch: usize,
    pub fixture_defect: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    pub schema: String,
    pub fixture_version: String,
    pub summary: Summary,
    pub cases: Vec<CaseResult>,
}

impl Report {
    pub fn from_cases(fixture_version: impl Into<String>, cases: Vec<CaseResult>) -> Self {
        let mut summary = Summary {
            total: cases.len(),
            ..Summary::default()
        };
        for case in &cases {
            match (case.status, case.evidence) {
                (CaseStatus::Pass, Some(EvidenceClass::Implementation)) => {
                    summary.implementation_passed += 1;
                }
                (CaseStatus::Pass, Some(EvidenceClass::ContractAssertion)) => {
                    summary.contract_assertions_passed += 1;
                }
                (CaseStatus::ImplementationMismatch, _) => summary.implementation_mismatch += 1,
                (CaseStatus::ContractAssertionMismatch, _) => {
                    summary.contract_assertion_mismatch += 1;
                }
                (CaseStatus::FixtureDefect, _) => summary.fixture_defect += 1,
                (CaseStatus::Pass, None) => summary.fixture_defect += 1,
            }
        }
        Self {
            schema: REPORT_SCHEMA.to_owned(),
            fixture_version: fixture_version.into(),
            summary,
            cases,
        }
    }

    pub fn exit_code(&self) -> i32 {
        if self.summary.fixture_defect > 0 {
            2
        } else if self.summary.implementation_mismatch > 0
            || self.summary.contract_assertion_mismatch > 0
        {
            1
        } else {
            0
        }
    }
}

pub(crate) fn compared(
    group: &str,
    id: &str,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> CaseResult {
    let expected = expected.into();
    let actual = actual.into();
    CaseResult {
        id: id.to_owned(),
        group: group.to_owned(),
        status: if expected == actual {
            CaseStatus::Pass
        } else {
            CaseStatus::ImplementationMismatch
        },
        evidence: Some(EvidenceClass::Implementation),
        expected: Some(expected),
        actual: Some(actual),
        detail: None,
    }
}

pub(crate) fn fixture_defect(group: &str, id: &str, detail: impl Into<String>) -> CaseResult {
    CaseResult {
        id: id.to_owned(),
        group: group.to_owned(),
        status: CaseStatus::FixtureDefect,
        evidence: None,
        expected: None,
        actual: None,
        detail: Some(detail.into()),
    }
}
