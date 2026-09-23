// SPDX-License-Identifier: MPL-2.0

use crate::cbor::validate_cde;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use minicbor::Encoder;
use std::fmt;
use zeroize::Zeroizing;

const DEVICE_GRANT_DOMAIN: &str = "MUSICPKG device-grant signature v1";
const OWNERSHIP_PROOF_DOMAIN: &str = "MUSICPKG ownership-proof v1";
const TRANSFER_SELLER_DOMAIN: &str = "MUSICPKG transfer seller v1";
const TRANSFER_RECIPIENT_DOMAIN: &str = "MUSICPKG transfer recipient v1";
const INHERIT_AUTHORITY_DOMAIN: &str = "MUSICPKG inherit authority v1";
const INHERIT_HEIR_DOMAIN: &str = "MUSICPKG inherit heir v1";
const REISSUE_OWNER_DOMAIN: &str = "MUSICPKG reissue owner v1";
const REISSUE_ISSUER_DOMAIN: &str = "MUSICPKG reissue issuer v1";
const REKEY_AUTHORITY_DOMAIN: &str = "MUSICPKG rekey authority v1";
const REKEY_NEW_OWNER_DOMAIN: &str = "MUSICPKG rekey new-owner v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidPublicKey,
    InvalidSignature,
    VerificationFailed,
    InvalidCbor,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPublicKey => write!(f, "invalid Ed25519 public key"),
            Self::InvalidSignature => write!(f, "invalid Ed25519 signature"),
            Self::VerificationFailed => write!(f, "Ed25519 verification failed"),
            Self::InvalidCbor => write!(f, "embedded CBOR is not one canonical deterministic item"),
        }
    }
}

impl std::error::Error for Error {}

pub fn sign_ed25519(seed: &[u8], message: &[u8]) -> Result<[u8; 64], Error> {
    let seed_bytes =
        Zeroizing::new(<[u8; 32]>::try_from(seed).map_err(|_| Error::InvalidPublicKey)?);
    let key = SigningKey::from_bytes(&seed_bytes);
    Ok(key.sign(message).to_bytes())
}

pub fn verify_ed25519(public_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), Error> {
    let pk_bytes: [u8; 32] = public_key.try_into().map_err(|_| Error::InvalidPublicKey)?;
    let key = VerifyingKey::from_bytes(&pk_bytes).map_err(|_| Error::InvalidPublicKey)?;
    let sig_bytes: [u8; 64] = signature.try_into().map_err(|_| Error::InvalidSignature)?;
    let sig = Signature::from_bytes(&sig_bytes);
    key.verify(message, &sig)
        .map_err(|_| Error::VerificationFailed)
}

fn body_signing_input(domain: &str, body_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    validate_cde(body_cbor).map_err(|_| Error::InvalidCbor)?;
    let mut e = Encoder::new(Vec::new());
    e.array(2).expect("Vec CBOR writer");
    e.str(domain).expect("Vec CBOR writer");
    let mut out = e.into_writer();
    out.extend_from_slice(body_cbor);
    Ok(out)
}

pub fn device_grant_signing_input(
    body_cbor: &[u8],
    enc: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, Error> {
    validate_cde(body_cbor).map_err(|_| Error::InvalidCbor)?;
    let mut e = Encoder::new(Vec::new());
    e.array(4).expect("Vec CBOR writer");
    e.str(DEVICE_GRANT_DOMAIN).expect("Vec CBOR writer");
    let mut out = e.into_writer();
    out.extend_from_slice(body_cbor);
    let mut e = Encoder::new(out);
    e.bytes(enc).expect("Vec CBOR writer");
    e.bytes(ciphertext).expect("Vec CBOR writer");
    Ok(e.into_writer())
}

pub fn ownership_proof_signing_input(challenge_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    body_signing_input(OWNERSHIP_PROOF_DOMAIN, challenge_cbor)
}

pub fn transfer_seller_signing_input(body_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    body_signing_input(TRANSFER_SELLER_DOMAIN, body_cbor)
}

pub fn transfer_recipient_signing_input(body_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    body_signing_input(TRANSFER_RECIPIENT_DOMAIN, body_cbor)
}

pub fn inherit_authority_signing_input(body_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    body_signing_input(INHERIT_AUTHORITY_DOMAIN, body_cbor)
}

pub fn inherit_heir_signing_input(body_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    body_signing_input(INHERIT_HEIR_DOMAIN, body_cbor)
}

pub fn reissue_owner_signing_input(body_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    body_signing_input(REISSUE_OWNER_DOMAIN, body_cbor)
}

pub fn reissue_issuer_signing_input(body_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    body_signing_input(REISSUE_ISSUER_DOMAIN, body_cbor)
}

pub fn rekey_authority_signing_input(body_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    body_signing_input(REKEY_AUTHORITY_DOMAIN, body_cbor)
}

pub fn rekey_new_owner_signing_input(body_cbor: &[u8]) -> Result<Vec<u8>, Error> {
    body_signing_input(REKEY_NEW_OWNER_DOMAIN, body_cbor)
}
