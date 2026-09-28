// SPDX-License-Identifier: MPL-2.0

use musicpkg_core::merkle::{
    Error, InclusionProof, leaf_hash, node_hash, rfc9162_root, verify_inclusion,
};

fn fixture_leaves() -> Vec<Vec<u8>> {
    [
        "82667265636f726400",
        "82667265636f726401",
        "82667265636f726402",
    ]
    .into_iter()
    .map(|value| hex::decode(value).unwrap())
    .collect()
}

fn expected_root() -> [u8; 32] {
    hex::decode("1438146ce0a4f3becd4088235578d2eaaa595fbefea0d5fe1795f0443c781e81")
        .unwrap()
        .try_into()
        .unwrap()
}

#[test]
fn public_three_leaf_vector_matches_rfc9162_root() {
    let leaves = fixture_leaves();
    assert_eq!(
        hex::encode(leaf_hash(&leaves[0])),
        "cbb7e91733bdb5310c522f2d8879f9839443ce4a58e784a3a72435534b4410d4"
    );
    assert_eq!(rfc9162_root(&leaves), expected_root());
}

#[test]
fn single_leaf_tree_uses_an_empty_audit_path() {
    let leaf = b"single";
    let root = leaf_hash(leaf);
    let proof = InclusionProof {
        tree_size: 1,
        leaf_index: 0,
        audit_path: vec![],
    };

    assert_eq!(verify_inclusion(leaf, &proof, &root), Ok(()));

    let mut with_extra_sibling = proof;
    with_extra_sibling.audit_path.push([0; 32]);
    assert_eq!(
        verify_inclusion(leaf, &with_extra_sibling, &root),
        Err(Error::InvalidProofLength)
    );
}

#[test]
fn inclusion_proofs_follow_the_non_power_of_two_tree_shape() {
    let leaves = fixture_leaves();
    let hashes = leaves
        .iter()
        .map(|leaf| leaf_hash(leaf))
        .collect::<Vec<_>>();
    let left_subtree = node_hash(&hashes[0], &hashes[1]);
    let root = expected_root();

    let proofs = [
        InclusionProof {
            tree_size: 3,
            leaf_index: 0,
            audit_path: vec![hashes[1], hashes[2]],
        },
        InclusionProof {
            tree_size: 3,
            leaf_index: 1,
            audit_path: vec![hashes[0], hashes[2]],
        },
        InclusionProof {
            tree_size: 3,
            leaf_index: 2,
            audit_path: vec![left_subtree],
        },
    ];

    for (leaf, proof) in leaves.iter().zip(&proofs) {
        assert_eq!(verify_inclusion(leaf, proof, &root), Ok(()));
    }
}

#[test]
fn mutated_leaf_cannot_reuse_a_valid_inclusion_proof() {
    let mut leaves = fixture_leaves();
    let hashes = leaves
        .iter()
        .map(|leaf| leaf_hash(leaf))
        .collect::<Vec<_>>();
    let proof = InclusionProof {
        tree_size: 3,
        leaf_index: 1,
        audit_path: vec![hashes[0], hashes[2]],
    };
    *leaves[1].last_mut().unwrap() = 9;

    assert_eq!(
        verify_inclusion(&leaves[1], &proof, &expected_root()),
        Err(Error::RootMismatch)
    );
    assert_eq!(Error::RootMismatch.code(), "LEDGER_INCLUSION_INVALID");
}

#[test]
fn malformed_proof_shape_fails_closed() {
    let leaves = fixture_leaves();
    let hashes = leaves
        .iter()
        .map(|leaf| leaf_hash(leaf))
        .collect::<Vec<_>>();

    assert_eq!(
        verify_inclusion(
            &leaves[0],
            &InclusionProof {
                tree_size: 0,
                leaf_index: 0,
                audit_path: vec![],
            },
            &expected_root(),
        ),
        Err(Error::InvalidTreeSize)
    );
    assert_eq!(
        verify_inclusion(
            &leaves[0],
            &InclusionProof {
                tree_size: 3,
                leaf_index: 3,
                audit_path: vec![],
            },
            &expected_root(),
        ),
        Err(Error::InvalidLeafIndex)
    );
    assert_eq!(
        verify_inclusion(
            &leaves[0],
            &InclusionProof {
                tree_size: 3,
                leaf_index: 0,
                audit_path: vec![hashes[1]],
            },
            &expected_root(),
        ),
        Err(Error::InvalidProofLength)
    );
    assert_eq!(
        verify_inclusion(
            &leaves[0],
            &InclusionProof {
                tree_size: 3,
                leaf_index: 0,
                audit_path: vec![[0; 32]; 65],
            },
            &expected_root(),
        ),
        Err(Error::ResourceLimit)
    );
}
