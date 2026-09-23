// SPDX-License-Identifier: MPL-2.0

use crate::signatures::{Error, sign_ed25519, verify_ed25519};
use minicbor::Encoder;

pub fn publisher_sig_structure(protected_cbor: &[u8], payload_cbor: &[u8]) -> Vec<u8> {
    let mut e = Encoder::new(Vec::new());
    e.array(4).expect("Vec CBOR writer");
    e.str("Signature1").expect("Vec CBOR writer");
    e.bytes(protected_cbor).expect("Vec CBOR writer");
    e.bytes(&[]).expect("Vec CBOR writer");
    e.bytes(payload_cbor).expect("Vec CBOR writer");
    e.into_writer()
}

pub fn verify_publisher_signature(
    issuer_public: &[u8],
    protected_cbor: &[u8],
    payload_cbor: &[u8],
    signature: &[u8],
) -> Result<(), Error> {
    let structure = publisher_sig_structure(protected_cbor, payload_cbor);
    verify_ed25519(issuer_public, &structure, signature)
}

pub fn sign_publisher(
    issuer_seed: &[u8],
    protected_cbor: &[u8],
    payload_cbor: &[u8],
) -> Result<[u8; 64], Error> {
    let structure = publisher_sig_structure(protected_cbor, payload_cbor);
    sign_ed25519(issuer_seed, &structure)
}
