// SPDX-License-Identifier: MPL-2.0

use crate::cbor::validate_cde;
use minicbor::Encoder;
use sha2::{Digest, Sha256};
use std::fmt;

const ISSUER_ID_DOMAIN: &[u8] = b"MUSICPKG issuer v1";
const OUTPUT_ID_DOMAIN: &str = "MUSICPKG output-id v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidIssuerPublicKeyLength,
    InvalidRecordHashLength,
    InvalidCbor,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIssuerPublicKeyLength => write!(f, "invalid issuer public key length"),
            Self::InvalidRecordHashLength => write!(f, "invalid record hash length"),
            Self::InvalidCbor => write!(f, "object is not one canonical deterministic CBOR item"),
        }
    }
}

impl std::error::Error for Error {}

pub fn hash_cde(cbor: &[u8]) -> Result<[u8; 32], Error> {
    validate_cde(cbor).map_err(|_| Error::InvalidCbor)?;
    Ok(Sha256::digest(cbor).into())
}

pub fn issuer_id(issuer_ed25519_public: &[u8]) -> Result<[u8; 32], Error> {
    if issuer_ed25519_public.len() != 32 {
        return Err(Error::InvalidIssuerPublicKeyLength);
    }
    let mut h = Sha256::new();
    h.update(ISSUER_ID_DOMAIN);
    h.update([0]);
    h.update(issuer_ed25519_public);
    Ok(h.finalize().into())
}

pub fn output_id(record_hash: &[u8]) -> Result<[u8; 32], Error> {
    if record_hash.len() != 32 {
        return Err(Error::InvalidRecordHashLength);
    }
    let mut e = Encoder::new(Vec::new());
    e.array(3).expect("Vec CBOR writer");
    e.str(OUTPUT_ID_DOMAIN).expect("Vec CBOR writer");
    e.bytes(record_hash).expect("Vec CBOR writer");
    e.u8(0).expect("Vec CBOR writer");
    Ok(Sha256::digest(e.into_writer()).into())
}
