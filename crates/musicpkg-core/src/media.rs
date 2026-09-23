// SPDX-License-Identifier: MPL-2.0

use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, Payload},
};
use hkdf::Hkdf;
use minicbor::Encoder;
use sha2::{Digest, Sha256};
use std::fmt;
use zeroize::Zeroizing;

const TRACK_SALT_DOMAIN: &[u8] = b"MUSICPKG track salt v1";
const TRACK_KEY_DOMAIN: &str = "MUSICPKG track key v1";
const MEDIA_RECORD_DOMAIN: &str = "MUSICPKG media record v1";

pub const MAX_RECORD_PLAINTEXT: u64 = 1_048_576;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidLength(&'static str),
    Hkdf,
    Aead,
    RecordTooLarge,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(name) => write!(f, "invalid {name} length"),
            Self::Hkdf => write!(f, "HKDF expansion failed"),
            Self::Aead => write!(f, "AEAD authentication failed"),
            Self::RecordTooLarge => write!(f, "media record exceeds v0.2 plaintext limit"),
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

fn encoder() -> Encoder<Vec<u8>> {
    Encoder::new(Vec::new())
}

fn finish(enc: Encoder<Vec<u8>>) -> Vec<u8> {
    enc.into_writer()
}

pub fn derive_track_key(
    k_album: &[u8],
    package_id: &[u8],
    track_id: &[u8],
) -> Result<[u8; 32], Error> {
    require_len(k_album, 32, "K_album")?;
    require_len(package_id, 16, "package_id")?;
    require_len(track_id, 16, "track_id")?;

    let mut salt_input = Vec::with_capacity(TRACK_SALT_DOMAIN.len() + 1 + package_id.len());
    salt_input.extend_from_slice(TRACK_SALT_DOMAIN);
    salt_input.push(0);
    salt_input.extend_from_slice(package_id);
    let salt = Sha256::digest(&salt_input);

    let mut enc = encoder();
    enc.array(3).expect("Vec CBOR writer");
    enc.str(TRACK_KEY_DOMAIN).expect("Vec CBOR writer");
    enc.bytes(package_id).expect("Vec CBOR writer");
    enc.bytes(track_id).expect("Vec CBOR writer");
    let info = finish(enc);

    let hkdf = Hkdf::<Sha256>::new(Some(&salt), k_album);
    let mut out = [0u8; 32];
    hkdf.expand(&info, &mut out).map_err(|_| Error::Hkdf)?;
    Ok(out)
}

pub fn media_record_nonce(record_index: u64) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[4..].copy_from_slice(&record_index.to_be_bytes());
    nonce
}

pub fn media_record_aad(
    package_id: &[u8],
    track_id: &[u8],
    record_index: u64,
    plaintext_length: u64,
    media_hash: &[u8],
) -> Result<Vec<u8>, Error> {
    require_len(package_id, 16, "package_id")?;
    require_len(track_id, 16, "track_id")?;
    require_len(media_hash, 32, "media_hash")?;

    let mut enc = encoder();
    enc.array(6).expect("Vec CBOR writer");
    enc.str(MEDIA_RECORD_DOMAIN).expect("Vec CBOR writer");
    enc.bytes(package_id).expect("Vec CBOR writer");
    enc.bytes(track_id).expect("Vec CBOR writer");
    enc.u64(record_index).expect("Vec CBOR writer");
    enc.u64(plaintext_length).expect("Vec CBOR writer");
    enc.bytes(media_hash).expect("Vec CBOR writer");
    Ok(finish(enc))
}

pub fn seal_media_record(
    k_album: &[u8],
    package_id: &[u8],
    track_id: &[u8],
    record_index: u64,
    media_hash: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, Error> {
    let plaintext_length = u64::try_from(plaintext.len()).map_err(|_| Error::RecordTooLarge)?;
    if plaintext_length > MAX_RECORD_PLAINTEXT {
        return Err(Error::RecordTooLarge);
    }
    let k_track = Zeroizing::new(derive_track_key(k_album, package_id, track_id)?);
    let aad = media_record_aad(
        package_id,
        track_id,
        record_index,
        plaintext_length,
        media_hash,
    )?;
    let nonce_bytes = media_record_nonce(record_index);
    let cipher = Aes256Gcm::new_from_slice(k_track.as_ref()).map_err(|_| Error::Aead)?;
    cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| Error::Aead)
}

pub fn open_media_record(
    k_album: &[u8],
    package_id: &[u8],
    track_id: &[u8],
    record_index: u64,
    plaintext_length: u64,
    media_hash: &[u8],
    ciphertext_and_tag: &[u8],
) -> Result<Vec<u8>, Error> {
    if plaintext_length > MAX_RECORD_PLAINTEXT {
        return Err(Error::RecordTooLarge);
    }
    let k_track = Zeroizing::new(derive_track_key(k_album, package_id, track_id)?);
    let aad = media_record_aad(
        package_id,
        track_id,
        record_index,
        plaintext_length,
        media_hash,
    )?;
    let nonce_bytes = media_record_nonce(record_index);
    let cipher = Aes256Gcm::new_from_slice(k_track.as_ref()).map_err(|_| Error::Aead)?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: ciphertext_and_tag,
                aad: &aad,
            },
        )
        .map_err(|_| Error::Aead)?;
    if plaintext.len() as u64 != plaintext_length {
        return Err(Error::InvalidLength("plaintext"));
    }
    Ok(plaintext)
}
