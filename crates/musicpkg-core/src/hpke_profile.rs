// SPDX-License-Identifier: MPL-2.0

use hpke::{
    Deserializable, Kem as KemTrait, OpModeR, OpModeS, Serializable,
    aead::AesGcm256,
    kdf::HkdfSha256,
    kem::{DhP256HkdfSha256, X25519HkdfSha256},
    setup_receiver, setup_sender,
};
use std::fmt;
use zeroize::Zeroizing;

const OWNER_E1_INFO: &[u8] = b"MUSICPKG owner-envelope E1";
const DEVICE_S1_INFO: &[u8] = b"MUSICPKG device-grant S1";
const DEVICE_H1_INFO: &[u8] = b"MUSICPKG device-grant H1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidKey,
    InvalidEncappedKey,
    Setup,
    Seal,
    Open,
    InvalidPlaintextLength,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey => write!(f, "invalid HPKE key"),
            Self::InvalidEncappedKey => write!(f, "invalid HPKE encapsulated key"),
            Self::Setup => write!(f, "HPKE setup failed"),
            Self::Seal => write!(f, "HPKE seal failed"),
            Self::Open => write!(f, "HPKE open failed"),
            Self::InvalidPlaintextLength => write!(f, "invalid HPKE plaintext length"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedEnvelope {
    pub enc: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

fn seal<Kem>(
    public_key: &[u8],
    info: &[u8],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<SealedEnvelope, Error>
where
    Kem: KemTrait,
{
    if plaintext.len() != 32 {
        return Err(Error::InvalidPlaintextLength);
    }
    let pk = <Kem as KemTrait>::PublicKey::from_bytes(public_key).map_err(|_| Error::InvalidKey)?;
    let (encapped, mut ctx) = setup_sender::<AesGcm256, HkdfSha256, Kem>(&OpModeS::Base, &pk, info)
        .map_err(|_| Error::Setup)?;
    let ciphertext = ctx.seal(plaintext, aad).map_err(|_| Error::Seal)?;
    Ok(SealedEnvelope {
        enc: encapped.to_bytes().to_vec(),
        ciphertext,
    })
}

fn open<Kem>(
    private_key: &[u8],
    enc: &[u8],
    info: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error>
where
    Kem: KemTrait,
{
    let sk =
        <Kem as KemTrait>::PrivateKey::from_bytes(private_key).map_err(|_| Error::InvalidKey)?;
    let encapped =
        <Kem as KemTrait>::EncappedKey::from_bytes(enc).map_err(|_| Error::InvalidEncappedKey)?;
    let mut ctx =
        setup_receiver::<AesGcm256, HkdfSha256, Kem>(&OpModeR::Base, &sk, &encapped, info)
            .map_err(|_| Error::Setup)?;
    let plaintext = ctx.open(ciphertext, aad).map_err(|_| Error::Open)?;
    if plaintext.len() != 32 {
        return Err(Error::InvalidPlaintextLength);
    }
    Ok(Zeroizing::new(plaintext))
}

pub fn seal_owner_e1(
    recipient_x25519_public: &[u8],
    aad: &[u8],
    k_album: &[u8],
) -> Result<SealedEnvelope, Error> {
    seal::<X25519HkdfSha256>(recipient_x25519_public, OWNER_E1_INFO, aad, k_album)
}

pub fn open_owner_e1(
    recipient_x25519_private: &[u8],
    enc: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    open::<X25519HkdfSha256>(
        recipient_x25519_private,
        enc,
        OWNER_E1_INFO,
        aad,
        ciphertext,
    )
}

pub fn seal_device_s1(
    recipient_x25519_public: &[u8],
    aad: &[u8],
    k_album: &[u8],
) -> Result<SealedEnvelope, Error> {
    seal::<X25519HkdfSha256>(recipient_x25519_public, DEVICE_S1_INFO, aad, k_album)
}

pub fn open_device_s1(
    recipient_x25519_private: &[u8],
    enc: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    open::<X25519HkdfSha256>(
        recipient_x25519_private,
        enc,
        DEVICE_S1_INFO,
        aad,
        ciphertext,
    )
}

pub fn seal_device_h1(
    recipient_p256_public: &[u8],
    aad: &[u8],
    k_album: &[u8],
) -> Result<SealedEnvelope, Error> {
    seal::<DhP256HkdfSha256>(recipient_p256_public, DEVICE_H1_INFO, aad, k_album)
}

pub fn open_device_h1(
    recipient_p256_private: &[u8],
    enc: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    open::<DhP256HkdfSha256>(recipient_p256_private, enc, DEVICE_H1_INFO, aad, ciphertext)
}
