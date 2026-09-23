// SPDX-License-Identifier: MPL-2.0

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Payload},
};
use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use minicbor::Encoder;
use sha2::Sha256;
use std::fmt;
use zeroize::Zeroizing;

const RECOVERY_KEY_DOMAIN: &str = "MUSICPKG recovery key v1";
const RECOVERY_ENVELOPE_DOMAIN: &str = "MUSICPKG recovery-envelope v1";
const R1_DOMAIN: &str = "MUSICPKG recovery-bundle R1";
const R1_MEMORY_KIB: u32 = 65_536;
const R1_ITERATIONS: u32 = 3;
const R1_LANES: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidLength(&'static str),
    Hkdf,
    Argon2,
    Aead,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(name) => write!(f, "invalid {name} length"),
            Self::Hkdf => write!(f, "HKDF expansion failed"),
            Self::Argon2 => write!(f, "Argon2id derivation failed"),
            Self::Aead => write!(f, "AEAD authentication failed"),
        }
    }
}

impl std::error::Error for Error {}

fn require_len(input: &[u8], expected: usize, name: &'static str) -> Result<(), Error> {
    if input.len() == expected {
        Ok(())
    } else {
        Err(Error::InvalidLength(name))
    }
}

pub fn derive_recovery_key(
    rr: &[u8],
    package_id: &[u8],
    title_id: &[u8],
    owner_key_set_hash: &[u8],
) -> Result<[u8; 32], Error> {
    require_len(rr, 32, "Recovery Root")?;
    require_len(package_id, 16, "package_id")?;
    require_len(title_id, 32, "title_id")?;
    require_len(owner_key_set_hash, 32, "owner_key_set_hash")?;

    let mut enc = Encoder::new(Vec::new());
    enc.array(3).expect("Vec CBOR writer");
    enc.str(RECOVERY_KEY_DOMAIN).expect("Vec CBOR writer");
    enc.bytes(title_id).expect("Vec CBOR writer");
    enc.bytes(owner_key_set_hash).expect("Vec CBOR writer");
    let info = enc.into_writer();

    let hkdf = Hkdf::<Sha256>::new(Some(package_id), rr);
    let mut out = [0u8; 32];
    hkdf.expand(&info, &mut out).map_err(|_| Error::Hkdf)?;
    Ok(out)
}

pub fn recovery_aad(
    package_id: &[u8],
    title_id: &[u8],
    owner_key_set_hash: &[u8],
) -> Result<Vec<u8>, Error> {
    require_len(package_id, 16, "package_id")?;
    require_len(title_id, 32, "title_id")?;
    require_len(owner_key_set_hash, 32, "owner_key_set_hash")?;

    let mut enc = Encoder::new(Vec::new());
    enc.array(4).expect("Vec CBOR writer");
    enc.str(RECOVERY_ENVELOPE_DOMAIN).expect("Vec CBOR writer");
    enc.bytes(package_id).expect("Vec CBOR writer");
    enc.bytes(title_id).expect("Vec CBOR writer");
    enc.bytes(owner_key_set_hash).expect("Vec CBOR writer");
    Ok(enc.into_writer())
}

pub fn open_recovery_envelope(
    rr: &[u8],
    package_id: &[u8],
    title_id: &[u8],
    owner_key_set_hash: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    require_len(nonce, 12, "recovery nonce")?;
    let key = Zeroizing::new(derive_recovery_key(
        rr,
        package_id,
        title_id,
        owner_key_set_hash,
    )?);
    let aad = recovery_aad(package_id, title_id, owner_key_set_hash)?;
    let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(|_| Error::Aead)?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| Error::Aead)?;
    if plaintext.len() != 64 {
        return Err(Error::InvalidLength("owner private material"));
    }
    Ok(Zeroizing::new(plaintext))
}

pub fn derive_r1_key(passphrase_bytes: &[u8], salt: &[u8]) -> Result<[u8; 32], Error> {
    require_len(salt, 16, "R1 salt")?;
    let params =
        Params::new(R1_MEMORY_KIB, R1_ITERATIONS, R1_LANES, Some(32)).map_err(|_| Error::Argon2)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; 32];
    argon2
        .hash_password_into(passphrase_bytes, salt, &mut out)
        .map_err(|_| Error::Argon2)?;
    Ok(out)
}

pub fn r1_aad(salt: &[u8]) -> Result<Vec<u8>, Error> {
    require_len(salt, 16, "R1 salt")?;
    let mut enc = Encoder::new(Vec::new());
    enc.array(9).expect("Vec CBOR writer");
    enc.str(R1_DOMAIN).expect("Vec CBOR writer");
    enc.u8(1).expect("Vec CBOR writer");
    enc.u8(1).expect("Vec CBOR writer");
    enc.u8(2).expect("Vec CBOR writer"); // Argon2id
    enc.u8(0x13).expect("Vec CBOR writer");
    enc.bytes(salt).expect("Vec CBOR writer");
    enc.u32(R1_MEMORY_KIB).expect("Vec CBOR writer");
    enc.u32(R1_ITERATIONS).expect("Vec CBOR writer");
    enc.u32(R1_LANES).expect("Vec CBOR writer");
    Ok(enc.into_writer())
}

pub fn open_r1_bundle(
    passphrase_bytes: &[u8],
    salt: &[u8],
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    require_len(nonce, 12, "R1 nonce")?;
    let key = Zeroizing::new(derive_r1_key(passphrase_bytes, salt)?);
    let aad = r1_aad(salt)?;
    let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(|_| Error::Aead)?;
    let rr = cipher
        .decrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: ciphertext,
                aad: &aad,
            },
        )
        .map_err(|_| Error::Aead)?;
    if rr.len() != 32 {
        return Err(Error::InvalidLength("Recovery Root"));
    }
    Ok(Zeroizing::new(rr))
}
