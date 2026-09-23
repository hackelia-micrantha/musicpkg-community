// SPDX-License-Identifier: MPL-2.0

use musicpkg_core::cose::{publisher_sig_structure, verify_publisher_signature};
use musicpkg_core::hpke_profile::{open_device_h1, open_owner_e1};
use musicpkg_core::ids::issuer_id;
use musicpkg_core::media::{
    derive_track_key, media_record_aad, media_record_nonce, open_media_record, seal_media_record,
};
use musicpkg_core::owner::{OwnerKeySet, owner_key_set_hash};
use musicpkg_core::recovery::{
    derive_r1_key, derive_recovery_key, open_r1_bundle, open_recovery_envelope, r1_aad,
    recovery_aad,
};
use musicpkg_core::signatures::{
    device_grant_signing_input, ownership_proof_signing_input, verify_ed25519,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::fixture::{self, FixtureError};
use crate::report::{CaseResult, compared, fixture_defect};

type RecoveryContext = (Vec<u8>, Vec<u8>, Vec<u8>);

enum Failure {
    Fixture(FixtureError),
    Implementation(String),
}

impl From<FixtureError> for Failure {
    fn from(value: FixtureError) -> Self {
        Self::Fixture(value)
    }
}

fn imp(message: impl Into<String>) -> Failure {
    Failure::Implementation(message.into())
}

fn check_eq(actual: &[u8], expected: &[u8], label: &str) -> Result<(), Failure> {
    if actual == expected {
        Ok(())
    } else {
        Err(imp(format!("{label} mismatch")))
    }
}

fn case(id: &str, check: Result<(), Failure>) -> CaseResult {
    match check {
        Ok(()) => compared("positive", id, "OK", "OK"),
        Err(Failure::Fixture(error)) => fixture_defect("positive", id, error.to_string()),
        Err(Failure::Implementation(detail)) => {
            let mut result = compared("positive", id, "OK", "IMPLEMENTATION_MISMATCH");
            result.detail = Some(detail);
            result
        }
    }
}

pub fn run(crypto: &Value) -> Vec<CaseResult> {
    vec![
        case("track-record", check_track_record(crypto)),
        case("owner-o1", check_owner_o1(crypto)),
        case("owner-envelope-e1", check_owner_envelope(crypto)),
        case("device-grant-h1", check_device_grant(crypto)),
        case("recovery-envelope", check_recovery_envelope(crypto)),
        case("r1-bundle", check_r1_bundle(crypto)),
        case("ownership-proof", check_ownership_proof(crypto)),
        case("publisher-cose-sign1", check_publisher(crypto)),
        case("merkle-three-leaves", check_merkle(crypto)),
    ]
}

fn check_track_record(v: &Value) -> Result<(), Failure> {
    let k_album = fixture::hex_at(v, "/track_record/k_album")?;
    let package = fixture::hex_at(v, "/track_record/package_id")?;
    let track = fixture::hex_at(v, "/track_record/track_id")?;
    let media_hash = fixture::hex_at(v, "/track_record/media_hash")?;
    let index = fixture::req_u64(v, "/track_record/record_index")?;
    let plaintext = fixture::hex_at(v, "/track_record/plaintext")?;
    let ciphertext = fixture::hex_at(v, "/track_record/ciphertext_and_tag")?;

    let key = derive_track_key(&k_album, &package, &track).map_err(|e| imp(e.to_string()))?;
    check_eq(
        &key,
        &fixture::hex_at(v, "/track_record/k_track")?,
        "track key",
    )?;
    check_eq(
        &media_record_nonce(index),
        &fixture::hex_at(v, "/track_record/nonce")?,
        "record nonce",
    )?;
    let aad = media_record_aad(&package, &track, index, plaintext.len() as u64, &media_hash)
        .map_err(|e| imp(e.to_string()))?;
    check_eq(
        &aad,
        &fixture::hex_at(v, "/track_record/aad_cbor")?,
        "record AAD",
    )?;
    let opened = open_media_record(
        &k_album,
        &package,
        &track,
        index,
        plaintext.len() as u64,
        &media_hash,
        &ciphertext,
    )
    .map_err(|e| imp(e.to_string()))?;
    check_eq(&opened, &plaintext, "record plaintext")?;
    let sealed = seal_media_record(&k_album, &package, &track, index, &media_hash, &plaintext)
        .map_err(|e| imp(e.to_string()))?;
    check_eq(&sealed, &ciphertext, "record ciphertext")
}

fn check_owner_o1(v: &Value) -> Result<(), Failure> {
    let keys = OwnerKeySet::from_private_material(
        &fixture::hex_at(v, "/owner_o1/ed25519_seed")?,
        &fixture::hex_at(v, "/owner_o1/x25519_private")?,
    )
    .map_err(|e| imp(e.to_string()))?;
    check_eq(
        keys.ed25519_public(),
        &fixture::hex_at(v, "/owner_o1/ed25519_public")?,
        "owner Ed25519 public",
    )?;
    check_eq(
        keys.x25519_public(),
        &fixture::hex_at(v, "/owner_o1/x25519_public")?,
        "owner X25519 public",
    )?;
    check_eq(
        &keys.to_cde(),
        &fixture::hex_at(v, "/owner_o1/owner_key_set_cbor")?,
        "owner key CBOR",
    )?;
    check_eq(
        &owner_key_set_hash(&keys),
        &fixture::hex_at(v, "/owner_o1/owner_key_set_hash")?,
        "owner key hash",
    )
}

fn check_owner_envelope(v: &Value) -> Result<(), Failure> {
    let opened = open_owner_e1(
        &fixture::hex_at(v, "/owner_o1/x25519_private")?,
        &fixture::hex_at(v, "/owner_envelope_e1/enc")?,
        &fixture::hex_at(v, "/owner_envelope_e1/context_cbor")?,
        &fixture::hex_at(v, "/owner_envelope_e1/ciphertext")?,
    )
    .map_err(|e| imp(e.to_string()))?;
    check_eq(
        opened.as_slice(),
        &fixture::hex_at(v, "/track_record/k_album")?,
        "owner envelope plaintext",
    )
}

fn check_device_grant(v: &Value) -> Result<(), Failure> {
    let scalar = fixture::req_u64(v, "/device_grant_h1/recipient_private_scalar_vector_only")?;
    let mut private = [0u8; 32];
    private[24..].copy_from_slice(&scalar.to_be_bytes());
    let body = fixture::hex_at(v, "/device_grant_h1/body_cbor")?;
    let enc = fixture::hex_at(v, "/device_grant_h1/enc")?;
    let ciphertext = fixture::hex_at(v, "/device_grant_h1/ciphertext")?;
    let opened =
        open_device_h1(&private, &enc, &body, &ciphertext).map_err(|e| imp(e.to_string()))?;
    check_eq(
        opened.as_slice(),
        &fixture::hex_at(v, "/track_record/k_album")?,
        "H1 grant plaintext",
    )?;
    let input =
        device_grant_signing_input(&body, &enc, &ciphertext).map_err(|e| imp(e.to_string()))?;
    check_eq(
        &input,
        &fixture::hex_at(v, "/device_grant_h1/owner_signature_input_cbor")?,
        "H1 signing input",
    )?;
    verify_ed25519(
        &fixture::hex_at(v, "/owner_o1/ed25519_public")?,
        &input,
        &fixture::hex_at(v, "/device_grant_h1/owner_signature")?,
    )
    .map_err(|e| imp(e.to_string()))
}

fn recovery_context(v: &Value) -> Result<RecoveryContext, Failure> {
    Ok((
        hex::decode("000102030405060708090a0b0c0d0e0f").map_err(|e| imp(e.to_string()))?,
        hex::decode("404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f")
            .map_err(|e| imp(e.to_string()))?,
        fixture::hex_at(v, "/owner_o1/owner_key_set_hash")?,
    ))
}

fn check_recovery_envelope(v: &Value) -> Result<(), Failure> {
    let (package, title, owner_hash) = recovery_context(v)?;
    let rr = fixture::hex_at(v, "/recovery_envelope/rr")?;
    let key =
        derive_recovery_key(&rr, &package, &title, &owner_hash).map_err(|e| imp(e.to_string()))?;
    check_eq(
        &key,
        &fixture::hex_at(v, "/recovery_envelope/k_recovery")?,
        "recovery key",
    )?;
    let aad = recovery_aad(&package, &title, &owner_hash).map_err(|e| imp(e.to_string()))?;
    check_eq(
        &aad,
        &fixture::hex_at(v, "/recovery_envelope/aad_cbor")?,
        "recovery AAD",
    )?;
    let opened = open_recovery_envelope(
        &rr,
        &package,
        &title,
        &owner_hash,
        &fixture::hex_at(v, "/recovery_envelope/nonce")?,
        &fixture::hex_at(v, "/recovery_envelope/ciphertext")?,
    )
    .map_err(|e| imp(e.to_string()))?;
    check_eq(
        opened.as_slice(),
        &fixture::hex_at(v, "/recovery_envelope/owner_private_material")?,
        "recovery plaintext",
    )
}

fn check_r1_bundle(v: &Value) -> Result<(), Failure> {
    let passphrase = fixture::hex_at(v, "/r1_bundle/passphrase_utf8_nfc")?;
    let salt = fixture::hex_at(v, "/r1_bundle/salt")?;
    let key = derive_r1_key(&passphrase, &salt).map_err(|e| imp(e.to_string()))?;
    check_eq(
        &key,
        &fixture::hex_at(v, "/r1_bundle/argon2id_output")?,
        "R1 Argon2id",
    )?;
    let aad = r1_aad(&salt).map_err(|e| imp(e.to_string()))?;
    check_eq(&aad, &fixture::hex_at(v, "/r1_bundle/aad_cbor")?, "R1 AAD")?;
    let rr = open_r1_bundle(
        &passphrase,
        &salt,
        &fixture::hex_at(v, "/r1_bundle/nonce")?,
        &fixture::hex_at(v, "/r1_bundle/ciphertext")?,
    )
    .map_err(|e| imp(e.to_string()))?;
    check_eq(
        rr.as_slice(),
        &fixture::hex_at(v, "/r1_bundle/rr")?,
        "R1 Recovery Root",
    )
}

fn check_ownership_proof(v: &Value) -> Result<(), Failure> {
    let challenge = fixture::hex_at(v, "/ownership_proof/challenge_cbor")?;
    let input = ownership_proof_signing_input(&challenge).map_err(|e| imp(e.to_string()))?;
    check_eq(
        &input,
        &fixture::hex_at(v, "/ownership_proof/signing_input_cbor")?,
        "ownership signing input",
    )?;
    verify_ed25519(
        &fixture::hex_at(v, "/owner_o1/ed25519_public")?,
        &input,
        &fixture::hex_at(v, "/ownership_proof/signature")?,
    )
    .map_err(|e| imp(e.to_string()))
}

fn check_publisher(v: &Value) -> Result<(), Failure> {
    let public = fixture::hex_at(v, "/publisher_cose_sign1/issuer_public")?;
    check_eq(
        &issuer_id(&public).map_err(|e| imp(e.to_string()))?,
        &fixture::hex_at(v, "/publisher_cose_sign1/issuer_id")?,
        "issuer id",
    )?;
    let protected = fixture::hex_at(v, "/publisher_cose_sign1/protected_cbor")?;
    let payload = fixture::hex_at(v, "/publisher_cose_sign1/payload_cbor")?;
    let structure = publisher_sig_structure(&protected, &payload);
    check_eq(
        &structure,
        &fixture::hex_at(v, "/publisher_cose_sign1/sig_structure_cbor")?,
        "COSE Sig_structure",
    )?;
    verify_publisher_signature(
        &public,
        &protected,
        &payload,
        &fixture::hex_at(v, "/publisher_cose_sign1/signature")?,
    )
    .map_err(|e| imp(e.to_string()))
}

pub(crate) fn leaf_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([0u8]);
    hasher.update(data);
    hasher.finalize().into()
}

fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([1u8]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

pub(crate) fn rfc9162_root(leaves: &[Vec<u8>]) -> [u8; 32] {
    match leaves.len() {
        0 => Sha256::digest([]).into(),
        1 => leaf_hash(&leaves[0]),
        n => {
            let split = 1usize << ((usize::BITS - (n - 1).leading_zeros() - 1) as usize);
            let left = rfc9162_root(&leaves[..split]);
            let right = rfc9162_root(&leaves[split..]);
            node_hash(&left, &right)
        }
    }
}

fn check_merkle(v: &Value) -> Result<(), Failure> {
    let leaf_values = fixture::req_array(v, "/merkle_three_leaves/leaf_data_cbor")?;
    let expected_hash_values = fixture::req_array(v, "/merkle_three_leaves/leaf_hashes")?;
    if leaf_values.len() != expected_hash_values.len() {
        return Err(imp("Merkle leaf/hash count mismatch"));
    }
    let mut leaves = Vec::with_capacity(leaf_values.len());
    for (index, value) in leaf_values.iter().enumerate() {
        let raw = value
            .as_str()
            .ok_or_else(|| FixtureError(format!("merkle leaf {index} must be hex string")))?;
        let leaf = hex::decode(raw)
            .map_err(|e| FixtureError(format!("invalid merkle leaf {index}: {e}")))?;
        let expected_raw = expected_hash_values[index]
            .as_str()
            .ok_or_else(|| FixtureError(format!("merkle hash {index} must be hex string")))?;
        let expected = hex::decode(expected_raw)
            .map_err(|e| FixtureError(format!("invalid merkle hash {index}: {e}")))?;
        check_eq(&leaf_hash(&leaf), &expected, "Merkle leaf hash")?;
        leaves.push(leaf);
    }
    check_eq(
        &rfc9162_root(&leaves),
        &fixture::hex_at(v, "/merkle_three_leaves/root")?,
        "Merkle root",
    )
}
