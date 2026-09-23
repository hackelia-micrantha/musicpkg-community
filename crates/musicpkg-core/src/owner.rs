// SPDX-License-Identifier: MPL-2.0

use ed25519_dalek::SigningKey;
use minicbor::Encoder;
use sha2::{Digest, Sha256};
use std::fmt;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::Zeroizing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidLength(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(name) => write!(f, "invalid {name} length"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerKeySet {
    ed25519_public: [u8; 32],
    x25519_public: [u8; 32],
}

impl OwnerKeySet {
    pub fn from_private_material(
        ed25519_seed: &[u8],
        x25519_private: &[u8],
    ) -> Result<Self, Error> {
        let ed_seed = Zeroizing::new(
            <[u8; 32]>::try_from(ed25519_seed).map_err(|_| Error::InvalidLength("ed25519_seed"))?,
        );
        let x_private = Zeroizing::new(
            <[u8; 32]>::try_from(x25519_private)
                .map_err(|_| Error::InvalidLength("x25519_private"))?,
        );

        let signing = SigningKey::from_bytes(&ed_seed);
        let x_secret = StaticSecret::from(*x_private);
        let x_public = X25519PublicKey::from(&x_secret);

        Ok(Self {
            ed25519_public: signing.verifying_key().to_bytes(),
            x25519_public: x_public.to_bytes(),
        })
    }

    pub fn ed25519_public(&self) -> &[u8; 32] {
        &self.ed25519_public
    }

    pub fn x25519_public(&self) -> &[u8; 32] {
        &self.x25519_public
    }

    pub fn to_cde(&self) -> Vec<u8> {
        let mut enc = Encoder::new(Vec::new());
        enc.map(3).expect("Vec CBOR writer");
        enc.u8(1).expect("Vec CBOR writer");
        enc.u8(1).expect("Vec CBOR writer");
        enc.u8(2).expect("Vec CBOR writer");
        enc.bytes(&self.ed25519_public).expect("Vec CBOR writer");
        enc.u8(3).expect("Vec CBOR writer");
        enc.bytes(&self.x25519_public).expect("Vec CBOR writer");
        enc.into_writer()
    }
}

pub fn owner_key_set_hash(keys: &OwnerKeySet) -> [u8; 32] {
    Sha256::digest(keys.to_cde()).into()
}
