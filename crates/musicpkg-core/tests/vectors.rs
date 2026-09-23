// SPDX-License-Identifier: MPL-2.0

use std::fs;
use std::path::PathBuf;

use musicpkg_core::media::{
    derive_track_key, media_record_aad, media_record_nonce, open_media_record,
};
use serde_json::Value;

fn vector_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-vectors/v0.2/crypto.json")
}

fn vectors() -> Value {
    let bytes = fs::read(vector_file()).expect("read committed crypto vectors");
    serde_json::from_slice(&bytes).expect("parse crypto vectors")
}

fn h(s: &str) -> Vec<u8> {
    hex::decode(s).expect("valid vector hex")
}

#[test]
fn track_vector_derives_exact_key_nonce_aad_and_plaintext() {
    let v = vectors();
    let t = &v["track_record"];

    let package_id = h(t["package_id"].as_str().unwrap());
    let track_id = h(t["track_id"].as_str().unwrap());
    let k_album = h(t["k_album"].as_str().unwrap());
    let media_hash = h(t["media_hash"].as_str().unwrap());
    let record_index = t["record_index"].as_u64().unwrap();
    let plaintext = h(t["plaintext"].as_str().unwrap());
    let ciphertext = h(t["ciphertext_and_tag"].as_str().unwrap());

    assert_eq!(
        hex::encode(derive_track_key(&k_album, &package_id, &track_id).unwrap()),
        t["k_track"].as_str().unwrap()
    );
    assert_eq!(
        hex::encode(media_record_nonce(record_index)),
        t["nonce"].as_str().unwrap()
    );
    let aad = media_record_aad(
        &package_id,
        &track_id,
        record_index,
        plaintext.len() as u64,
        &media_hash,
    )
    .unwrap();
    assert_eq!(hex::encode(&aad), t["aad_cbor"].as_str().unwrap());
    assert_eq!(
        open_media_record(
            &k_album,
            &package_id,
            &track_id,
            record_index,
            plaintext.len() as u64,
            &media_hash,
            &ciphertext,
        )
        .unwrap(),
        plaintext
    );
}

#[test]
fn owner_o1_vector_derives_exact_public_keys_cbor_and_hash() {
    use musicpkg_core::owner::{OwnerKeySet, owner_key_set_hash};

    let v = vectors();
    let o = &v["owner_o1"];
    let ed_seed = h(o["ed25519_seed"].as_str().unwrap());
    let x_private = h(o["x25519_private"].as_str().unwrap());

    let keys = OwnerKeySet::from_private_material(&ed_seed, &x_private).unwrap();
    assert_eq!(
        hex::encode(keys.ed25519_public()),
        o["ed25519_public"].as_str().unwrap()
    );
    assert_eq!(
        hex::encode(keys.x25519_public()),
        o["x25519_public"].as_str().unwrap()
    );

    let cbor = keys.to_cde();
    assert_eq!(
        hex::encode(&cbor),
        o["owner_key_set_cbor"].as_str().unwrap()
    );
    assert_eq!(
        hex::encode(owner_key_set_hash(&keys)),
        o["owner_key_set_hash"].as_str().unwrap()
    );
}

#[test]
fn recovery_vectors_open_exact_owner_material_and_recovery_root() {
    use musicpkg_core::recovery::{
        derive_r1_key, derive_recovery_key, open_r1_bundle, open_recovery_envelope, r1_aad,
        recovery_aad,
    };

    let v = vectors();
    let r = &v["recovery_envelope"];
    let package_id = h("000102030405060708090a0b0c0d0e0f");
    let title_id = h("404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f");
    let owner_hash = h(v["owner_o1"]["owner_key_set_hash"].as_str().unwrap());
    let rr = h(r["rr"].as_str().unwrap());

    assert_eq!(
        hex::encode(derive_recovery_key(&rr, &package_id, &title_id, &owner_hash).unwrap()),
        r["k_recovery"].as_str().unwrap()
    );
    assert_eq!(
        hex::encode(recovery_aad(&package_id, &title_id, &owner_hash).unwrap()),
        r["aad_cbor"].as_str().unwrap()
    );
    assert_eq!(
        hex::encode(
            open_recovery_envelope(
                &rr,
                &package_id,
                &title_id,
                &owner_hash,
                &h(r["nonce"].as_str().unwrap()),
                &h(r["ciphertext"].as_str().unwrap()),
            )
            .unwrap()
        ),
        r["owner_private_material"].as_str().unwrap()
    );

    let b = &v["r1_bundle"];
    let passphrase = h(b["passphrase_utf8_nfc"].as_str().unwrap());
    let salt = h(b["salt"].as_str().unwrap());
    assert_eq!(
        hex::encode(derive_r1_key(&passphrase, &salt).unwrap()),
        b["argon2id_output"].as_str().unwrap()
    );
    assert_eq!(
        hex::encode(r1_aad(&salt).unwrap()),
        b["aad_cbor"].as_str().unwrap()
    );
    assert_eq!(
        hex::encode(
            open_r1_bundle(
                &passphrase,
                &salt,
                &h(b["nonce"].as_str().unwrap()),
                &h(b["ciphertext"].as_str().unwrap()),
            )
            .unwrap()
        ),
        b["rr"].as_str().unwrap()
    );
}

#[test]
fn hpke_vectors_open_exact_album_key_for_e1_and_h1() {
    use musicpkg_core::hpke_profile::{open_device_h1, open_owner_e1};

    let v = vectors();
    let expected = h(v["track_record"]["k_album"].as_str().unwrap());

    let e1 = &v["owner_envelope_e1"];
    let owner_private = h(v["owner_o1"]["x25519_private"].as_str().unwrap());
    let e1_plain = open_owner_e1(
        &owner_private,
        &h(e1["enc"].as_str().unwrap()),
        &h(e1["context_cbor"].as_str().unwrap()),
        &h(e1["ciphertext"].as_str().unwrap()),
    )
    .unwrap();
    assert_eq!(e1_plain.as_slice(), expected.as_slice());

    let h1 = &v["device_grant_h1"];
    let scalar = h1["recipient_private_scalar_vector_only"].as_u64().unwrap();
    let mut h1_private = [0u8; 32];
    h1_private[24..].copy_from_slice(&scalar.to_be_bytes());
    let h1_plain = open_device_h1(
        &h1_private,
        &h(h1["enc"].as_str().unwrap()),
        &h(h1["body_cbor"].as_str().unwrap()),
        &h(h1["ciphertext"].as_str().unwrap()),
    )
    .unwrap();
    assert_eq!(h1_plain.as_slice(), expected.as_slice());
}

#[test]
fn signature_vectors_reconstruct_exact_inputs_and_verify() {
    use musicpkg_core::cose::{publisher_sig_structure, verify_publisher_signature};
    use musicpkg_core::signatures::{
        device_grant_signing_input, ownership_proof_signing_input, verify_ed25519,
    };

    let v = vectors();
    let owner_public = h(v["owner_o1"]["ed25519_public"].as_str().unwrap());

    let d = &v["device_grant_h1"];
    let device_input = device_grant_signing_input(
        &h(d["body_cbor"].as_str().unwrap()),
        &h(d["enc"].as_str().unwrap()),
        &h(d["ciphertext"].as_str().unwrap()),
    )
    .unwrap();
    assert_eq!(
        hex::encode(&device_input),
        d["owner_signature_input_cbor"].as_str().unwrap()
    );
    verify_ed25519(
        &owner_public,
        &device_input,
        &h(d["owner_signature"].as_str().unwrap()),
    )
    .unwrap();

    let p = &v["ownership_proof"];
    let proof_input =
        ownership_proof_signing_input(&h(p["challenge_cbor"].as_str().unwrap())).unwrap();
    assert_eq!(
        hex::encode(&proof_input),
        p["signing_input_cbor"].as_str().unwrap()
    );
    verify_ed25519(
        &owner_public,
        &proof_input,
        &h(p["signature"].as_str().unwrap()),
    )
    .unwrap();

    let c = &v["publisher_cose_sign1"];
    let protected = h(c["protected_cbor"].as_str().unwrap());
    let payload = h(c["payload_cbor"].as_str().unwrap());
    let signature = h(c["signature"].as_str().unwrap());
    let sig_structure = publisher_sig_structure(&protected, &payload);
    assert_eq!(
        hex::encode(&sig_structure),
        c["sig_structure_cbor"].as_str().unwrap()
    );
    verify_publisher_signature(
        &h(c["issuer_public"].as_str().unwrap()),
        &protected,
        &payload,
        &signature,
    )
    .unwrap();
}

#[test]
fn signature_vectors_reproduce_exact_ed25519_signatures_and_issuer_id() {
    use musicpkg_core::cose::sign_publisher;
    use musicpkg_core::ids::issuer_id;
    use musicpkg_core::signatures::{
        device_grant_signing_input, ownership_proof_signing_input, sign_ed25519,
    };

    let v = vectors();
    let owner_seed = h(v["owner_o1"]["ed25519_seed"].as_str().unwrap());

    let d = &v["device_grant_h1"];
    let device_input = device_grant_signing_input(
        &h(d["body_cbor"].as_str().unwrap()),
        &h(d["enc"].as_str().unwrap()),
        &h(d["ciphertext"].as_str().unwrap()),
    )
    .unwrap();
    assert_eq!(
        hex::encode(sign_ed25519(&owner_seed, &device_input).unwrap()),
        d["owner_signature"].as_str().unwrap()
    );

    let p = &v["ownership_proof"];
    let proof_input =
        ownership_proof_signing_input(&h(p["challenge_cbor"].as_str().unwrap())).unwrap();
    assert_eq!(
        hex::encode(sign_ed25519(&owner_seed, &proof_input).unwrap()),
        p["signature"].as_str().unwrap()
    );

    let c = &v["publisher_cose_sign1"];
    let issuer_seed = h(c["issuer_seed_vector_only"].as_str().unwrap());
    let issuer_public = h(c["issuer_public"].as_str().unwrap());
    assert_eq!(
        hex::encode(issuer_id(&issuer_public).unwrap()),
        c["issuer_id"].as_str().unwrap()
    );
    assert_eq!(
        hex::encode(
            sign_publisher(
                &issuer_seed,
                &h(c["protected_cbor"].as_str().unwrap()),
                &h(c["payload_cbor"].as_str().unwrap()),
            )
            .unwrap()
        ),
        c["signature"].as_str().unwrap()
    );
}

#[test]
fn strict_cbor_accepts_cde_and_rejects_ambiguous_encodings() {
    use musicpkg_core::cbor::{Error as CborError, validate_cde};

    let v = vectors();
    validate_cde(&h(v["owner_o1"]["owner_key_set_cbor"].as_str().unwrap())).unwrap();
    validate_cde(&h(v["ownership_proof"]["challenge_cbor"].as_str().unwrap())).unwrap();

    assert_eq!(
        validate_cde(&[0x18, 0x01]),
        Err(CborError::NonDeterministic)
    );
    assert_eq!(
        validate_cde(&[0xa2, 0x02, 0x01, 0x02, 0x02]),
        Err(CborError::DuplicateMapKey)
    );
    assert_eq!(
        validate_cde(&[0xa2, 0x02, 0x01, 0x01, 0x01]),
        Err(CborError::NonDeterministic)
    );
    assert_eq!(
        validate_cde(&[0x9f, 0x01, 0xff]),
        Err(CborError::NonDeterministic)
    );
    assert_eq!(validate_cde(&[0x01, 0x02]), Err(CborError::TrailingData));
}

#[test]
fn media_record_rejects_plaintext_length_above_v02_limit_before_open() {
    use musicpkg_core::media::{Error as MediaError, MAX_RECORD_PLAINTEXT, open_media_record};

    let v = vectors();
    let r = &v["track_record"];
    let result = open_media_record(
        &h(r["k_album"].as_str().unwrap()),
        &h(r["package_id"].as_str().unwrap()),
        &h(r["track_id"].as_str().unwrap()),
        0,
        MAX_RECORD_PLAINTEXT + 1,
        &h(r["media_hash"].as_str().unwrap()),
        &[],
    );
    assert_eq!(result, Err(MediaError::RecordTooLarge));
}

#[test]
fn signing_inputs_reject_noncanonical_or_trailing_embedded_cbor() {
    use musicpkg_core::signatures::{
        Error as SignatureError, device_grant_signing_input, ownership_proof_signing_input,
    };

    assert_eq!(
        ownership_proof_signing_input(&[0x01, 0x02]),
        Err(SignatureError::InvalidCbor)
    );
    assert_eq!(
        device_grant_signing_input(&[0x18, 0x01], &[0u8; 32], &[0u8; 48]),
        Err(SignatureError::InvalidCbor)
    );
}

#[test]
fn hpke_production_seal_roundtrips_e1_s1_and_h1() {
    use musicpkg_core::hpke_profile::{
        open_device_h1, open_device_s1, open_owner_e1, seal_device_h1, seal_device_s1,
        seal_owner_e1,
    };

    let v = vectors();
    let album_key = h(v["track_record"]["k_album"].as_str().unwrap());

    let x_private = h(v["owner_o1"]["x25519_private"].as_str().unwrap());
    let x_public = h(v["owner_o1"]["x25519_public"].as_str().unwrap());
    let e1_aad = h(v["owner_envelope_e1"]["context_cbor"].as_str().unwrap());

    let e1 = seal_owner_e1(&x_public, &e1_aad, &album_key).unwrap();
    let e1_open = open_owner_e1(&x_private, &e1.enc, &e1_aad, &e1.ciphertext).unwrap();
    assert_eq!(e1_open.as_slice(), album_key.as_slice());

    let s1_body = h(v["owner_o1"]["owner_key_set_cbor"].as_str().unwrap());
    let s1 = seal_device_s1(&x_public, &s1_body, &album_key).unwrap();
    let s1_open = open_device_s1(&x_private, &s1.enc, &s1_body, &s1.ciphertext).unwrap();
    assert_eq!(s1_open.as_slice(), album_key.as_slice());

    let h1 = &v["device_grant_h1"];
    let h1_public = h(h1["recipient_public_uncompressed"].as_str().unwrap());
    let scalar = h1["recipient_private_scalar_vector_only"].as_u64().unwrap();
    let mut h1_private = [0u8; 32];
    h1_private[24..].copy_from_slice(&scalar.to_be_bytes());
    let h1_body = h(h1["body_cbor"].as_str().unwrap());
    let sealed_h1 = seal_device_h1(&h1_public, &h1_body, &album_key).unwrap();
    let h1_open =
        open_device_h1(&h1_private, &sealed_h1.enc, &h1_body, &sealed_h1.ciphertext).unwrap();
    assert_eq!(h1_open.as_slice(), album_key.as_slice());
}

#[test]
fn media_record_seal_reproduces_exact_vector_ciphertext() {
    use musicpkg_core::media::seal_media_record;

    let v = vectors();
    let r = &v["track_record"];
    let sealed = seal_media_record(
        &h(r["k_album"].as_str().unwrap()),
        &h(r["package_id"].as_str().unwrap()),
        &h(r["track_id"].as_str().unwrap()),
        r["record_index"].as_u64().unwrap(),
        &h(r["media_hash"].as_str().unwrap()),
        &h(r["plaintext"].as_str().unwrap()),
    )
    .unwrap();
    assert_eq!(
        hex::encode(sealed),
        r["ciphertext_and_tag"].as_str().unwrap()
    );
}

#[test]
fn transition_signing_inputs_match_v02_domains() {
    use musicpkg_core::signatures::{
        inherit_authority_signing_input, inherit_heir_signing_input, reissue_issuer_signing_input,
        reissue_owner_signing_input, rekey_authority_signing_input, rekey_new_owner_signing_input,
        transfer_recipient_signing_input, transfer_seller_signing_input,
    };

    let body = hex::decode("a10101").unwrap();
    type SigningInputFn = fn(&[u8]) -> Result<Vec<u8>, musicpkg_core::signatures::Error>;
    let cases: &[(SigningInputFn, &str)] = &[
        (
            transfer_seller_signing_input,
            "82781b4d55534943504b47207472616e736665722073656c6c6572207631a10101",
        ),
        (
            transfer_recipient_signing_input,
            "82781e4d55534943504b47207472616e7366657220726563697069656e74207631a10101",
        ),
        (
            inherit_authority_signing_input,
            "82781d4d55534943504b4720696e686572697420617574686f72697479207631a10101",
        ),
        (
            inherit_heir_signing_input,
            "8278184d55534943504b4720696e68657269742068656972207631a10101",
        ),
        (
            reissue_owner_signing_input,
            "8278194d55534943504b472072656973737565206f776e6572207631a10101",
        ),
        (
            reissue_issuer_signing_input,
            "82781a4d55534943504b47207265697373756520697373756572207631a10101",
        ),
        (
            rekey_authority_signing_input,
            "82781b4d55534943504b472072656b657920617574686f72697479207631a10101",
        ),
        (
            rekey_new_owner_signing_input,
            "82781b4d55534943504b472072656b6579206e65772d6f776e6572207631a10101",
        ),
    ];
    for (make, expected) in cases {
        assert_eq!(hex::encode(make(&body).unwrap()), *expected);
    }
}

#[test]
fn core_hash_and_output_id_match_v02_derivation() {
    use musicpkg_core::ids::{hash_cde, output_id};

    let v = vectors();
    let owner_cbor = h(v["owner_o1"]["owner_key_set_cbor"].as_str().unwrap());
    assert_eq!(
        hex::encode(hash_cde(&owner_cbor).unwrap()),
        v["owner_o1"]["owner_key_set_hash"].as_str().unwrap()
    );

    let record_hash: Vec<u8> = (0u8..32).collect();
    assert_eq!(
        hex::encode(output_id(&record_hash).unwrap()),
        "5aac6b5bc7d5bb6ee8f8e05134a0a78542e4e5bac7f464ee1b82f18244240a3e"
    );
}
