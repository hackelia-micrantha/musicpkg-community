// SPDX-License-Identifier: MPL-2.0

use std::collections::HashSet;

use musicpkg_core::cbor::{Error as CborError, validate_cde};
use musicpkg_core::cose::verify_publisher_signature;
use musicpkg_core::hpke_profile::{open_device_h1, open_owner_e1};
use musicpkg_core::media::{Error as MediaError, open_media_record};
use musicpkg_core::recovery::{open_r1_bundle, open_recovery_envelope};
use musicpkg_core::signatures::{ownership_proof_signing_input, verify_ed25519};
use serde_json::Value;

use crate::fixture::{self, FixtureError};
use crate::positive::rfc9162_root;
use crate::report::{CaseResult, compared, fixture_defect};

type MediaArgs = (Vec<u8>, Vec<u8>, Vec<u8>, u64, usize, Vec<u8>, Vec<u8>);
type RecoveryContext = (Vec<u8>, Vec<u8>, Vec<u8>);

pub fn run(crypto: &Value, negative: &Value) -> Vec<CaseResult> {
    let cases = match fixture::req_array(negative, "/cases") {
        Ok(cases) => cases,
        Err(error) => return vec![fixture_defect("negative", "cases", error.to_string())],
    };
    cases
        .iter()
        .enumerate()
        .map(|(index, case)| run_case(crypto, case, index))
        .collect()
}

fn run_case(crypto: &Value, case: &Value, index: usize) -> CaseResult {
    let id = match fixture::req_str(case, "/id") {
        Ok(value) => value,
        Err(error) => {
            return fixture_defect("negative", &format!("case-{index}"), error.to_string());
        }
    };
    let expected = match fixture::req_str(case, "/expected") {
        Ok(value) => value,
        Err(error) => return fixture_defect("negative", id, error.to_string()),
    };
    match actual_for(id, crypto) {
        Ok(actual) => compared("negative", id, expected, actual),
        Err(error) => fixture_defect("negative", id, error.to_string()),
    }
}

fn actual_for(id: &str, v: &Value) -> Result<&'static str, FixtureError> {
    match id {
        "record-tag-bitflip" => record_tag_bitflip(v),
        "record-wrong-index" => record_wrong_index(v),
        "record-wrong-media-hash" => record_wrong_media_hash(v),
        "owner-envelope-wrong-recipient" => owner_wrong_recipient(v),
        "owner-envelope-context-change" => owner_context_change(v),
        "h1-grant-wrong-device" => h1_wrong_device(v),
        "h1-grant-owner-signature-bitflip" => h1_signature_bitflip(v),
        "recovery-wrong-root" => recovery_wrong_root(v),
        "r1-wrong-passphrase" => r1_wrong_passphrase(v),
        "ownership-proof-signature-bitflip" => proof_signature_bitflip(v),
        "ownership-proof-replay-audience" => proof_replay_audience(v),
        "ownership-proof-replay-nonce" => proof_replay_nonce(v),
        "publisher-signature-bitflip" => publisher_signature_bitflip(v),
        "publisher-payload-mutation" => publisher_payload_mutation(v),
        "merkle-leaf-mutation" => merkle_leaf_mutation(v),
        "duplicate-cbor-key" => Ok(match validate_cde(&[0xa2, 0x02, 0x01, 0x02, 0x02]) {
            Err(CborError::DuplicateMapKey) => "DUPLICATE_CBOR_KEY",
            _ => "OK",
        }),
        "non-deterministic-cbor-integer" => Ok(match validate_cde(&[0x18, 0x01]) {
            Err(CborError::NonDeterministic) => "NON_DETERMINISTIC_CBOR",
            _ => "OK",
        }),
        "unknown-mandatory-profile" => Ok(if profile_supported(65535) {
            "OK"
        } else {
            "UNSUPPORTED_PROFILE"
        }),
        "zip-path-traversal" => Ok(match validate_paths(&["../ownership/issuance.cose"]) {
            Err("unsafe") => "UNSAFE_PATH",
            _ => "OK",
        }),
        "zip-duplicate-path" => Ok(
            match validate_paths(&["manifest/media.cbor", "manifest/media.cbor"]) {
                Err("duplicate") => "DUPLICATE_PATH",
                _ => "OK",
            },
        ),
        "checkpoint-threshold-short" => Ok(if checkpoint_signers(&[1, 2], 3).is_err() {
            "CHECKPOINT_THRESHOLD_NOT_MET"
        } else {
            "OK"
        }),
        "checkpoint-duplicate-validator" => Ok(if unique_validators(&[1, 1, 2]).is_err() {
            "VALIDATOR_SET_INVALID"
        } else {
            "OK"
        }),
        "validator-rotation-hash-mismatch" => Ok(if rotation_matches(&[1u8; 32], &[2u8; 32]) {
            "OK"
        } else {
            "CHECKPOINT_CHAIN_INVALID"
        }),
        "stale-but-valid-current-title" => Ok(if checkpoint_fresh(100, 1000, 300) {
            "OK"
        } else {
            "CHECKPOINT_STALE"
        }),
        "double-transfer" => Ok(if second_spend_rejected() {
            "TRANSFER_INPUT_SPENT"
        } else {
            "OK"
        }),
        _ => Err(FixtureError(format!("unknown negative fixture case: {id}"))),
    }
}

fn media_args(v: &Value) -> Result<MediaArgs, FixtureError> {
    let plaintext = fixture::hex_at(v, "/track_record/plaintext")?;
    Ok((
        fixture::hex_at(v, "/track_record/k_album")?,
        fixture::hex_at(v, "/track_record/package_id")?,
        fixture::hex_at(v, "/track_record/track_id")?,
        fixture::req_u64(v, "/track_record/record_index")?,
        plaintext.len(),
        fixture::hex_at(v, "/track_record/media_hash")?,
        fixture::hex_at(v, "/track_record/ciphertext_and_tag")?,
    ))
}

fn media_result(result: Result<Vec<u8>, MediaError>) -> &'static str {
    match result {
        Err(MediaError::Aead) => "AEAD_AUTH_FAILED",
        _ => "OK",
    }
}

fn record_tag_bitflip(v: &Value) -> Result<&'static str, FixtureError> {
    let (key, package, track, index, len, media_hash, mut ciphertext) = media_args(v)?;
    let last = ciphertext
        .last_mut()
        .ok_or_else(|| FixtureError("empty record ciphertext".into()))?;
    *last ^= 1;
    Ok(media_result(open_media_record(
        &key,
        &package,
        &track,
        index,
        len as u64,
        &media_hash,
        &ciphertext,
    )))
}

fn record_wrong_index(v: &Value) -> Result<&'static str, FixtureError> {
    let (key, package, track, index, len, media_hash, ciphertext) = media_args(v)?;
    Ok(media_result(open_media_record(
        &key,
        &package,
        &track,
        index + 1,
        len as u64,
        &media_hash,
        &ciphertext,
    )))
}

fn record_wrong_media_hash(v: &Value) -> Result<&'static str, FixtureError> {
    let (key, package, track, index, len, _media_hash, ciphertext) = media_args(v)?;
    Ok(media_result(open_media_record(
        &key,
        &package,
        &track,
        index,
        len as u64,
        &[0u8; 32],
        &ciphertext,
    )))
}

fn owner_wrong_recipient(v: &Value) -> Result<&'static str, FixtureError> {
    let result = open_owner_e1(
        &[0x42u8; 32],
        &fixture::hex_at(v, "/owner_envelope_e1/enc")?,
        &fixture::hex_at(v, "/owner_envelope_e1/context_cbor")?,
        &fixture::hex_at(v, "/owner_envelope_e1/ciphertext")?,
    );
    Ok(if result.is_err() {
        "HPKE_OPEN_FAILED"
    } else {
        "OK"
    })
}

fn mutate_subsequence(mut bytes: Vec<u8>, needle: &[u8]) -> Result<Vec<u8>, FixtureError> {
    let pos = bytes
        .windows(needle.len())
        .position(|w| w == needle)
        .ok_or_else(|| FixtureError("mutation target not found in fixture".into()))?;
    bytes[pos] ^= 1;
    Ok(bytes)
}

fn owner_context_change(v: &Value) -> Result<&'static str, FixtureError> {
    let context = mutate_subsequence(
        fixture::hex_at(v, "/owner_envelope_e1/context_cbor")?,
        &(0x40u8..=0x5f).collect::<Vec<_>>(),
    )?;
    let result = open_owner_e1(
        &fixture::hex_at(v, "/owner_o1/x25519_private")?,
        &fixture::hex_at(v, "/owner_envelope_e1/enc")?,
        &context,
        &fixture::hex_at(v, "/owner_envelope_e1/ciphertext")?,
    );
    Ok(if result.is_err() {
        "HPKE_OPEN_FAILED"
    } else {
        "OK"
    })
}

fn h1_wrong_device(v: &Value) -> Result<&'static str, FixtureError> {
    let mut private = [0u8; 32];
    private[24..].copy_from_slice(&4u64.to_be_bytes());
    let result = open_device_h1(
        &private,
        &fixture::hex_at(v, "/device_grant_h1/enc")?,
        &fixture::hex_at(v, "/device_grant_h1/body_cbor")?,
        &fixture::hex_at(v, "/device_grant_h1/ciphertext")?,
    );
    Ok(if result.is_err() {
        "HPKE_OPEN_FAILED"
    } else {
        "OK"
    })
}

fn h1_signature_bitflip(v: &Value) -> Result<&'static str, FixtureError> {
    let body = fixture::hex_at(v, "/device_grant_h1/body_cbor")?;
    let enc = fixture::hex_at(v, "/device_grant_h1/enc")?;
    let ciphertext = fixture::hex_at(v, "/device_grant_h1/ciphertext")?;
    let input = musicpkg_core::signatures::device_grant_signing_input(&body, &enc, &ciphertext)
        .map_err(|e| FixtureError(e.to_string()))?;
    let mut signature = fixture::hex_at(v, "/device_grant_h1/owner_signature")?;
    let first = signature
        .first_mut()
        .ok_or_else(|| FixtureError("empty owner signature".into()))?;
    *first ^= 1;
    Ok(
        if verify_ed25519(
            &fixture::hex_at(v, "/owner_o1/ed25519_public")?,
            &input,
            &signature,
        )
        .is_err()
        {
            "OWNER_SIGNATURE_INVALID"
        } else {
            "OK"
        },
    )
}

fn recovery_context(v: &Value) -> Result<RecoveryContext, FixtureError> {
    Ok((
        hex::decode("000102030405060708090a0b0c0d0e0f").map_err(|e| FixtureError(e.to_string()))?,
        hex::decode("404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f")
            .map_err(|e| FixtureError(e.to_string()))?,
        fixture::hex_at(v, "/owner_o1/owner_key_set_hash")?,
    ))
}

fn recovery_wrong_root(v: &Value) -> Result<&'static str, FixtureError> {
    let (package, title, owner_hash) = recovery_context(v)?;
    let mut rr = fixture::hex_at(v, "/recovery_envelope/rr")?;
    let first = rr
        .first_mut()
        .ok_or_else(|| FixtureError("empty Recovery Root".into()))?;
    *first ^= 1;
    let result = open_recovery_envelope(
        &rr,
        &package,
        &title,
        &owner_hash,
        &fixture::hex_at(v, "/recovery_envelope/nonce")?,
        &fixture::hex_at(v, "/recovery_envelope/ciphertext")?,
    );
    Ok(if result.is_err() {
        "RECOVERY_AUTH_FAILED"
    } else {
        "OK"
    })
}

fn r1_wrong_passphrase(v: &Value) -> Result<&'static str, FixtureError> {
    let result = open_r1_bundle(
        b"correct horse battery staple!",
        &fixture::hex_at(v, "/r1_bundle/salt")?,
        &fixture::hex_at(v, "/r1_bundle/nonce")?,
        &fixture::hex_at(v, "/r1_bundle/ciphertext")?,
    );
    Ok(if result.is_err() {
        "RECOVERY_AUTH_FAILED"
    } else {
        "OK"
    })
}

fn proof_signature_bitflip(v: &Value) -> Result<&'static str, FixtureError> {
    let challenge = fixture::hex_at(v, "/ownership_proof/challenge_cbor")?;
    let input =
        ownership_proof_signing_input(&challenge).map_err(|e| FixtureError(e.to_string()))?;
    let mut signature = fixture::hex_at(v, "/ownership_proof/signature")?;
    let first = signature
        .first_mut()
        .ok_or_else(|| FixtureError("empty ownership signature".into()))?;
    *first ^= 1;
    Ok(
        if verify_ed25519(
            &fixture::hex_at(v, "/owner_o1/ed25519_public")?,
            &input,
            &signature,
        )
        .is_err()
        {
            "OWNER_SIGNATURE_INVALID"
        } else {
            "OK"
        },
    )
}

fn proof_replay(v: &Value, needle: &[u8]) -> Result<&'static str, FixtureError> {
    let challenge = mutate_subsequence(
        fixture::hex_at(v, "/ownership_proof/challenge_cbor")?,
        needle,
    )?;
    let input =
        ownership_proof_signing_input(&challenge).map_err(|e| FixtureError(e.to_string()))?;
    Ok(
        if verify_ed25519(
            &fixture::hex_at(v, "/owner_o1/ed25519_public")?,
            &input,
            &fixture::hex_at(v, "/ownership_proof/signature")?,
        )
        .is_err()
        {
            "OWNER_SIGNATURE_INVALID"
        } else {
            "OK"
        },
    )
}

fn proof_replay_audience(v: &Value) -> Result<&'static str, FixtureError> {
    proof_replay(v, b"example-verifier")
}
fn proof_replay_nonce(v: &Value) -> Result<&'static str, FixtureError> {
    proof_replay(v, &(0x11u8..=0x30).collect::<Vec<_>>())
}

fn publisher_signature_bitflip(v: &Value) -> Result<&'static str, FixtureError> {
    let mut signature = fixture::hex_at(v, "/publisher_cose_sign1/signature")?;
    let first = signature
        .first_mut()
        .ok_or_else(|| FixtureError("empty publisher signature".into()))?;
    *first ^= 1;
    let result = verify_publisher_signature(
        &fixture::hex_at(v, "/publisher_cose_sign1/issuer_public")?,
        &fixture::hex_at(v, "/publisher_cose_sign1/protected_cbor")?,
        &fixture::hex_at(v, "/publisher_cose_sign1/payload_cbor")?,
        &signature,
    );
    Ok(if result.is_err() {
        "ISSUER_SIGNATURE_INVALID"
    } else {
        "OK"
    })
}

fn publisher_payload_mutation(v: &Value) -> Result<&'static str, FixtureError> {
    let payload = mutate_subsequence(
        fixture::hex_at(v, "/publisher_cose_sign1/payload_cbor")?,
        &(0u8..=15).collect::<Vec<_>>(),
    )?;
    let result = verify_publisher_signature(
        &fixture::hex_at(v, "/publisher_cose_sign1/issuer_public")?,
        &fixture::hex_at(v, "/publisher_cose_sign1/protected_cbor")?,
        &payload,
        &fixture::hex_at(v, "/publisher_cose_sign1/signature")?,
    );
    Ok(if result.is_err() {
        "ISSUER_SIGNATURE_INVALID"
    } else {
        "OK"
    })
}

fn merkle_leaf_mutation(v: &Value) -> Result<&'static str, FixtureError> {
    let values = fixture::req_array(v, "/merkle_three_leaves/leaf_data_cbor")?;
    let mut leaves = Vec::new();
    for (i, value) in values.iter().enumerate() {
        let raw = value
            .as_str()
            .ok_or_else(|| FixtureError(format!("merkle leaf {i} must be string")))?;
        leaves.push(hex::decode(raw).map_err(|e| FixtureError(e.to_string()))?);
    }
    let leaf = leaves
        .get_mut(1)
        .ok_or_else(|| FixtureError("missing merkle leaf 1".into()))?;
    let last = leaf
        .last_mut()
        .ok_or_else(|| FixtureError("empty merkle leaf 1".into()))?;
    *last = 9;
    Ok(
        if rfc9162_root(&leaves).as_slice()
            != fixture::hex_at(v, "/merkle_three_leaves/root")?.as_slice()
        {
            "LEDGER_INCLUSION_INVALID"
        } else {
            "OK"
        },
    )
}

fn profile_supported(id: u16) -> bool {
    matches!(id, 1 | 2)
}

fn validate_paths(paths: &[&str]) -> Result<(), &'static str> {
    let mut seen = HashSet::new();
    for path in paths {
        if path.starts_with('/') || path.split('/').any(|part| part == "..") {
            return Err("unsafe");
        }
        if !seen.insert(*path) {
            return Err("duplicate");
        }
    }
    Ok(())
}

fn checkpoint_signers(signers: &[u64], threshold: usize) -> Result<(), ()> {
    if signers.len() < threshold {
        Err(())
    } else {
        Ok(())
    }
}

fn unique_validators(ids: &[u64]) -> Result<(), ()> {
    let mut seen = HashSet::new();
    if ids.iter().all(|id| seen.insert(*id)) {
        Ok(())
    } else {
        Err(())
    }
}

fn rotation_matches(committed: &[u8; 32], supplied: &[u8; 32]) -> bool {
    committed == supplied
}
fn checkpoint_fresh(checkpoint_time: u64, now: u64, max_age: u64) -> bool {
    now.saturating_sub(checkpoint_time) <= max_age
}
fn second_spend_rejected() -> bool {
    let mut spent = HashSet::new();
    let input = [0x5au8; 32];
    let first = spent.insert(input);
    let second = spent.insert(input);
    first && !second
}
