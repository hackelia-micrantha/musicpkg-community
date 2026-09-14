# MUSICPKG v0.2 Test Vectors

Status: working interoperability vectors

## Positive vectors

`crypto.json` contains deterministic vectors for:

- RFC 8949 deterministic-CBOR object bytes used by the test cases;
- OwnerKeySet O1 and `owner_key_set_hash`;
- track HKDF-SHA256 derivation;
- AES-256-GCM media-record nonce/AAD/ciphertext;
- E1 RFC 9180 HPKE X25519/HKDF-SHA256/AES-256-GCM;
- H1 RFC 9180 HPKE P-256/HKDF-SHA256/AES-256-GCM;
- H1 device-grant owner signature;
- package recovery key/envelope;
- R1 Argon2id + AES-256-GCM Recovery Root bundle;
- ownership challenge Ed25519 signature;
- publisher COSE_Sign1 Ed25519 Sig_structure/signature;
- RFC 9162 three-leaf Merkle Tree Hash example.

Private keys, deterministic HPKE ephemeral keys/scalars, Recovery Root values, nonces, and passphrases in these files are **test-only** and MUST NOT be reused in production.

## Vector generation/verification

The positive vector set was generated with independent standard primitive implementations and then round-trip checked for:

- AES-GCM media record open;
- X25519 E1 HPKE open;
- P-256 H1 HPKE open;
- recovery envelope open;
- R1 Recovery Root decrypt;
- owner/device Ed25519 signature verification;
- publisher COSE Sig_structure Ed25519 verification.

A reference implementation should reproduce the exact public/intermediate/output bytes in `crypto.json` from the provided deterministic vector inputs.

HPKE ephemeral private values are supplied only to make sender output deterministic for cross-implementation tests; production HPKE senders MUST generate ephemeral keys according to RFC 9180 requirements.

## Negative vectors

`negative.json` defines mutation cases and their required top-level error/result class, including:

- AEAD mutation/context mismatch;
- wrong HPKE recipient/context;
- invalid owner/publisher signatures;
- recovery authentication failures;
- ownership-proof replay/context changes;
- invalid Merkle proof;
- deterministic-CBOR violations;
- path traversal/duplicate ZIP paths;
- validator threshold/rotation errors;
- stale-but-valid checkpoint handling;
- double-transfer rejection.

Some container/ledger negatives are semantic mutation recipes rather than full binary package archives in this draft. A reference conformance harness should materialize them as concrete fixtures.

## Product/failure scenarios

`scenarios.json` captures cross-component requirements that should not be lost in byte-level crypto testing:

- offline playback;
- publisher/retailer disappearance;
- ledger disappearance;
- copied package without authority;
- device replacement;
- total-disaster recovery;
- historical-vs-current ownership;
- finalized transfer;
- signed-but-unfinalized transfer;
- double-transfer race;
- non-guarantee of historical plaintext deletion;
- rekey after compromise;
- no post-sale vendor policy revocation.

## Next implementation target

The reference conformance harness should consume these fixtures and expose the error/status taxonomy in `spec/verification-v0.2.md`.
