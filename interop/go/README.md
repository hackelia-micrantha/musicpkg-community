# Go interoperability probe (v0.2)

A standalone Go probe of the public MUSICPKG v0.2 vectors. It uses Go's
standard library and a pinned `golang.org/x/crypto/argon2` dependency for
Argon2id (RFC 9106). It does not import or execute Rust or private code.

All Go implementation helpers live in `*_test.go` files. No importable
cryptographic or ownership-verification API is published by this probe.

From `interop/go` run:

```sh
go test -count=1 ./...
go vet ./...
```

The probe uses only the committed public `test-vectors/v0.2/crypto.json`
and `ledger-state.json`. It independently checks:

- ISSUE and TRANSFER record hashing, output-ID deterministic CBOR/hashes;
- TRANSFER seller/recipient signing inputs and Ed25519 signatures;
- track HKDF-SHA256 derivation and AES-256-GCM media-record authentication;
- O1 Ed25519/X25519 public keys and canonical owner-key-set hashes;
- recovery-envelope HKDF/AES-256-GCM known-answer ciphertext;
- R1 Argon2id (64 MiB / 3 passes / 4 lanes), deterministic KDF/AAD,
  AES-256-GCM passphrase recovery and altered-passphrase/tag/AAD rejection;
- RFC 9180 E1 X25519 and H1 P-256 HPKE KEM shared secret, key schedule,
  AES-GCM seal/open, contextual AAD, and H1 owner grant signature;
- publisher COSE Sig_structure and ownership-proof signature inputs;
- RFC 9162 3-leaf Merkle tree root.

Selected negative mutations test wrong transfer signature domains, corrupted
signatures/transfer bodies, altered media-record/HPKE AAD, wrong publisher
signature domains, and mutated Merkle leaves.

**Evidence limits:** These are narrowly scoped known-answer probes, not a
general CBOR parser, production HPKE/R1 implementation, passphrase NFC text UI,
strict Ed25519 weak-key policy,
ownership-state transitions, package validator or 49-case implementation
parity. In particular, `VerifyVectorSignature` must not be used for
production authorization. Neither `PACKAGE_VALID` nor
`CURRENT_OWNER_VALID` is established. The current **26 executable / 23
contract assertions** remain unchanged.

Go source is MPL-2.0; public fixture/spec licenses remain unchanged.
