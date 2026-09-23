// SPDX-License-Identifier: MPL-2.0

use serde_json::{Value, json};

use crate::fixture;
use crate::report::{CaseResult, compared, fixture_defect};

pub fn run(scenarios: &Value) -> Vec<CaseResult> {
    let values = match fixture::req_array(scenarios, "/scenarios") {
        Ok(values) => values,
        Err(error) => return vec![fixture_defect("scenario", "scenarios", error.to_string())],
    };
    values
        .iter()
        .enumerate()
        .map(|(index, value)| run_case(value, index))
        .collect()
}

fn run_case(value: &Value, index: usize) -> CaseResult {
    let id = match fixture::req_str(value, "/id") {
        Ok(id) => id,
        Err(error) => {
            return fixture_defect("scenario", &format!("scenario-{index}"), error.to_string());
        }
    };
    let expected = match fixture::req(value, "/expected") {
        Ok(expected) if expected.is_object() => expected,
        Ok(_) => {
            return fixture_defect("scenario", id, "scenario expected value must be an object");
        }
        Err(error) => return fixture_defect("scenario", id, error.to_string()),
    };
    let oracle = match oracle(id) {
        Some(oracle) => oracle,
        None => {
            return fixture_defect(
                "scenario",
                id,
                format!("unknown scenario fixture case: {id}"),
            );
        }
    };
    if expected == &oracle {
        compared(
            "scenario",
            id,
            "SCENARIO_CONTRACT_VALID",
            "SCENARIO_CONTRACT_VALID",
        )
    } else {
        let mut result = compared(
            "scenario",
            id,
            "SCENARIO_CONTRACT_VALID",
            "SCENARIO_CONTRACT_MISMATCH",
        );
        result.detail =
            Some("fixture scenario expectation differs from the v0.2 contract oracle".into());
        result
    }
}

fn oracle(id: &str) -> Option<Value> {
    Some(match id {
        "offline-existing-purchase" => {
            json!({"playback":"allowed","current_title":"not_checked","error":"OK"})
        }
        "publisher-disappears" => {
            json!({"playback":"allowed","migration":"allowed","recovery":"allowed","publisher_online_dependency":false})
        }
        "ledger-disappears" => {
            json!({"playback":"allowed","historical_issuance":"valid","current_title":"unknown","new_transfer":"unavailable"})
        }
        "copied-package-no-authority" => {
            json!({"package_valid":true,"playback":"denied","error":"KEY_NOT_HELD"})
        }
        "lost-device-surviving-owner-authority" => {
            json!({"new_device_grant":"allowed_without_publisher_or_ledger","old_grant_required":false,"ownership_lost":false})
        }
        "total-device-loss-with-r1" => {
            json!({"recover_rr":true,"recover_owner_key":true,"new_device_grant":true,"publisher_required":false,"ledger_required_for_playback_recovery":false})
        }
        "wrong-r1-passphrase" => json!({"recovery":"denied","error":"RECOVERY_AUTH_FAILED"}),
        "historical-owner-offline" => {
            json!({"historical_owner":"valid","current_title":"unknown","playback_if_local_authority_valid":"allowed"})
        }
        "current-owner-proof" => json!({"ownership_result":"CURRENT_OWNER_VALID"}),
        "canonical-transfer" => {
            json!({"A_current_title":false,"B_current_title":true,"title_id_preserved":true,"package_id_preserved":true,"media_reencrypted":false,"publisher_participation_required":false})
        }
        "signed-transfer-not-final" => {
            json!({"B_current_title":false,"result":"TRANSFER_NOT_FINAL"})
        }
        "double-transfer-race" => {
            json!({"first_finalized":"current title","second":"TRANSFER_INPUT_SPENT","both_final":false})
        }
        "transfer-does-not-prove-erasure" => {
            json!({"B_current_title":true,"protocol_claims_A_deleted_plaintext":false,"protocol_claims_A_cannot_play_retained_plaintext":false})
        }
        "rekey-after-owner-key-compromise" => {
            json!({"title_id_preserved":true,"package_id_preserved":true,"current_owner_key_replaced":true,"media_reencrypted":false})
        }
        "vendor-policy-change-after-sale" => {
            json!({"ordinary_playback_revoked":false,"network_renewal_required":false})
        }
        _ => return None,
    })
}
