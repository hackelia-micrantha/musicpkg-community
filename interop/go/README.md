# Go interoperability probe (v0.2)

A standalone **standard-library-only** Go probe of the public MUSICPKG
v0.2 vectors. It does not import or execute Rust or private-repository code.

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
- publisher COSE Sig_structure and ownership-proof signature inputs;
- RFC 9162 3-leaf Merkle tree root.

Selected negative mutations test wrong transfer signature domains, corrupted
signatures/transfer bodies, altered media-record AAD, wrong publisher
signature domains, and mutated Merkle leaves.

**Evidence limits:** These are narrowly scoped known-answer probes, not a
general CBOR parser, HPKE, R1 Argon2id, strict Ed25519 weak-key policy,
ownership-state transitions, package validator or 49-case implementation
parity. In particular, `VerifyVectorSignature` must not be used for
production authorization. Neither `PACKAGE_VALID` nor
`CURRENT_OWNER_VALID` is established. The current **26 executable / 23
contract assertions** remain unchanged.

Go source is MPL-2.0; public fixture/spec licenses remain unchanged.
