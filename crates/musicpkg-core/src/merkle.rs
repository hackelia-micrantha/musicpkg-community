// SPDX-License-Identifier: MPL-2.0

//! RFC 9162 SHA-256 Merkle Tree Hash and inclusion-proof verification.
//!
//! MUSICPKG uses this construction for canonical ledger-record history and the
//! current-title state tree. Leaves are exact CDE object bytes.

use std::fmt;

use sha2::{Digest, Sha256};

pub const MAX_AUDIT_PATH: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidTreeSize,
    InvalidLeafIndex,
    InvalidProofLength,
    ResourceLimit,
    RootMismatch,
}

impl Error {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ResourceLimit => "RESOURCE_LIMIT",
            Self::InvalidTreeSize
            | Self::InvalidLeafIndex
            | Self::InvalidProofLength
            | Self::RootMismatch => "LEDGER_INCLUSION_INVALID",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InclusionProof {
    pub tree_size: u64,
    pub leaf_index: u64,
    pub audit_path: Vec<[u8; 32]>,
}

pub fn leaf_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([0u8]);
    hasher.update(data);
    hasher.finalize().into()
}

pub fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([1u8]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// RFC 9162 Merkle Tree Hash over exact leaf bytes.
pub fn rfc9162_root<T: AsRef<[u8]>>(leaves: &[T]) -> [u8; 32] {
    match leaves.len() {
        0 => Sha256::digest([]).into(),
        1 => leaf_hash(leaves[0].as_ref()),
        n => {
            let split = 1usize << (usize::BITS - (n - 1).leading_zeros() - 1);
            let left = rfc9162_root(&leaves[..split]);
            let right = rfc9162_root(&leaves[split..]);
            node_hash(&left, &right)
        }
    }
}

fn largest_power_of_two_less_than(n: u64) -> u64 {
    debug_assert!(n > 1);
    1u64 << (u64::BITS - (n - 1).leading_zeros() - 1)
}

/// Append sibling orientation from leaf to root.
///
/// `true` means the sibling is on the left of the current hash.
fn proof_shape(leaf_index: u64, tree_size: u64, out: &mut Vec<bool>) {
    if tree_size <= 1 {
        return;
    }
    let split = largest_power_of_two_less_than(tree_size);
    if leaf_index < split {
        proof_shape(leaf_index, split, out);
        out.push(false);
    } else {
        proof_shape(leaf_index - split, tree_size - split, out);
        out.push(true);
    }
}

pub fn inclusion_root(leaf: &[u8], proof: &InclusionProof) -> Result<[u8; 32], Error> {
    if proof.tree_size == 0 {
        return Err(Error::InvalidTreeSize);
    }
    if proof.leaf_index >= proof.tree_size {
        return Err(Error::InvalidLeafIndex);
    }
    if proof.audit_path.len() > MAX_AUDIT_PATH {
        return Err(Error::ResourceLimit);
    }

    let mut shape = Vec::with_capacity(MAX_AUDIT_PATH.min(proof.audit_path.len()));
    proof_shape(proof.leaf_index, proof.tree_size, &mut shape);
    if shape.len() != proof.audit_path.len() {
        return Err(Error::InvalidProofLength);
    }

    let mut current = leaf_hash(leaf);
    for (sibling, sibling_on_left) in proof.audit_path.iter().zip(shape) {
        current = if sibling_on_left {
            node_hash(sibling, &current)
        } else {
            node_hash(&current, sibling)
        };
    }
    Ok(current)
}

pub fn verify_inclusion(
    leaf: &[u8],
    proof: &InclusionProof,
    expected_root: &[u8; 32],
) -> Result<(), Error> {
    if inclusion_root(leaf, proof)? == *expected_root {
        Ok(())
    } else {
        Err(Error::RootMismatch)
    }
}
