# RFC 0000 — Portable Owner-Bound Music Package Format (MUSICPKG)

**Status:** Working Draft 0.2  
**Intended maturity:** Experimental / pre-standardization  
**Date:** September 2026

## Abstract

MUSICPKG is an open portable format for purchased digital music. A completed purchase creates durable cryptographic ownership rather than a renewable playback lease. The owner receives encrypted media, publisher issuance evidence, purchaser-controlled recovery authority, and sufficient offline proof to continue ordinary playback without a publisher, retailer, license server, subscription, or ownership-ledger service.

The central invariant is:

```text
COPYING != OWNING
```

Copying a `.musicpkg` artifact alone does not create another usable owned copy. An owner may make arbitrary encrypted backups and authorize their own devices. When a title permits transfer, canonical ownership can move to another owner without requiring the publisher to remain in the playback path.

MUSICPKG does not claim to prevent capture after legitimate playback reaches plaintext/PCM or the analog boundary.

## 1. Requirements language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** are to be interpreted as described by RFC 2119 and RFC 8174 when, and only when, they appear in all capitals.

## 2. Product contract

A conforming ecosystem preserves these user-visible properties:

1. A completed purchase is an ownership event, not a recurring permission lease.
2. Ordinary playback MUST work offline after legitimate enrollment.
3. Publisher, retailer, ledger, or network disappearance MUST NOT disable existing legitimate playback.
4. Copying only the protected artifact MUST NOT reveal playable plaintext.
5. The owner controls ordinary post-purchase device enrollment.
6. Device loss MUST NOT destroy ownership when valid purchaser recovery material exists.
7. Ownership MAY be independently proven without purchaser civil identity in the portable artifact or shared ledger.
8. Where transfer is enabled, canonical title MAY move to a new owner and the prior title state is consumed.
9. Transfer MUST NOT be represented as proof that a prior owner erased historical plaintext, keys, or recordings.
10. Publisher verification of issuance/title MUST NOT create publisher control over ordinary playback.

The canonical plain-language contract is maintained in `PRODUCT.md`.

## 3. Terminology and identifiers

### Asset

The recording/release/edition identity. `asset_id` is a 32-byte MUSICPKG cryptographic identifier and is distinct from external descriptive identifiers such as ISRC/ISWC/GRid/UPC.

### Title

One transferable ownership lineage. `title_id` is a 32-byte identifier that survives ordinary transfer and normally survives reissue.

### Package

One concrete encrypted media issuance. `package_id` is a random 16-byte identifier. It normally remains stable across transfer but changes on a `REISSUE` that creates new media/package bytes.

### OwnerKeySet

A fresh per-title owner credential. Profile O1 contains an Ed25519 signing public key and X25519 HPKE public key.

### Recovery Root

A purchaser-controlled random 256-bit offline recovery secret (`RR`). It does not generate owner keys and is not used for routine playback.

### Device grant

Replaceable operational authority allowing an authorized device to recover `K_album`. Hardware-backed grants are optional hardening, not the durable ownership root.

## 4. Architecture

```text
publisher issuance
        |
        v
immutable encrypted media core
        |
        +-- initial owner envelope
        +-- package-local recovery envelope
        +-- bundled historical ledger proof
        |
        v
current owner authority
        |
        +-- device grants
        +-- ownership proof
        +-- optional canonical transfer
```

The following are deliberately separate predicates:

- package/media integrity;
- publisher provenance/trust;
- historical issuance validity;
- possession of owner/device authority;
- current canonical title;
- forensic watermark evidence.

Failure of an optional provenance/watermark/current-title service MUST NOT silently become playback revocation.

## 5. Physical container

The physical `.musicpkg` representation is ZIP-compatible according to `spec/container-v0.2.md`.

Core objects include:

```text
manifest/media.cbor
manifest/content.cbor
ownership/issuance.cose
ownership/owner-envelope.cbor
ownership/recovery-envelope.cbor
ledger/genesis.cbor
ledger/issuance-record.cbor
ledger/issuance-proof.cbor
ledger/checkpoint.cbor
audio/<track-id>.flac.enc
audio/<track-id>.index.cbor
```

Security identity derives from hashes/signatures over logical object bytes, not ZIP headers or archive byte canonicalization.

ZIP path traversal, duplicate paths, symlinks/special files, ZIP encryption, unsupported compression/features, and resource-limit violations MUST be rejected before security-sensitive parsing.

Mutable device/current-title state SHOULD live outside the archival package in a sidecar/player store.

## 6. Structured encoding

Security-sensitive structures use CBOR with RFC 8949 Core Deterministic Encoding and the CDDL in `spec/musicpkg-v0.2.cddl`.

Conforming implementations MUST reject:

- indefinite lengths;
- duplicate keys;
- non-shortest integer/length encodings;
- maps not in deterministic key order;
- trailing data after an expected object;
- fields outside closed v0.2 schemas;
- unknown mandatory algorithm/profile identifiers.

Signed publisher issuance uses COSE_Sign1.

## 7. Mandatory cryptographic profiles

The exact constructions are in `spec/crypto-v0.2.md`.

| Purpose | v0.2 profile |
|---|---|
| Hash | SHA-256 |
| Publisher/owner/validator signature | Ed25519 |
| Owner envelope E1 | HPKE Base: X25519 / HKDF-SHA256 / AES-256-GCM |
| Software device grant S1 | HPKE Base: X25519 / HKDF-SHA256 / AES-256-GCM |
| Hardware device grant H1 | HPKE Base: P-256 / HKDF-SHA256 / AES-256-GCM |
| Track KDF | HKDF-SHA256 |
| Media records | AES-256-GCM |
| Recovery envelope | HKDF-SHA256 + AES-256-GCM |
| R1 recovery backup | Argon2id + AES-256-GCM |
| History/state Merkle trees | RFC 9162 SHA-256 Merkle Tree Hash |

H1 is an optional hardware conformance class; S1/software-capable playback remains required for open preservation/interoperability.

## 8. Media encryption

For each package:

```text
K_album = CSPRNG(32 bytes)
```

Each track derives an independent `K_track` from `K_album`, `package_id`, and `track_id` as specified in `crypto-v0.2.md`.

Tracks are divided into authenticated records of at most 1 MiB plaintext. AES-GCM nonce for record `i` is:

```text
0x00000000 || uint64_be(i)
```

under the unique per-track key.

Record AAD binds at least:

- protocol/domain string;
- package ID;
- track ID;
- record index;
- plaintext length;
- `media_hash`.

A player MUST authenticate an entire record before releasing its plaintext to the codec.

The encrypted track file is a concatenation of ciphertext+tag records. An authenticated committed index supplies offsets/lengths.

## 9. Immutable media core

`manifest/media.cbor` describes media identity/track framing before encryption. Its SHA-256 digest (`media_hash`) is bound into every media-record AAD.

`manifest/content.cbor` is created after encryption and commits exact hashes/sizes of immutable media objects.

Publisher issuance signs `content_manifest_hash` rather than being recursively included by it. This ordering eliminates manifest/signature hash cycles and permits ownership/recovery overlays to evolve without changing the encrypted media core.

## 10. Publisher issuance

`ownership/issuance.cose` is COSE_Sign1 with inline deterministic-CBOR issuance payload.

The payload binds at least:

- package/asset/title IDs;
- content-manifest hash;
- issuer identity;
- initial OwnerKeySet;
- initial owner-envelope hash;
- transfer policy;
- ledger ID/genesis hash;
- issuance time (informational);
- watermark commitment when present.

The v0.2 COSE protected header requires EdDSA (`alg=-8`) and deterministic issuer `kid` as specified in `crypto-v0.2.md`.

Publisher signature validity and publisher trust are separate: implementations MUST verify the signature; ecosystems define which publisher keys they trust.

## 11. Owner authority and E1 envelope

Each acquisition/title tenure SHOULD use fresh unlinkable O1 owner keys.

The issuer/previous owner encrypts the 32-byte `K_album` to the destination owner X25519 public key using E1 HPKE. The envelope AAD binds the package/title plus issuance/transfer/rekey context.

The owner private key is not sent to the publisher.

## 12. Recovery

Every owner key is independently random. Owner keys MUST NOT be deterministically derived from one collection seed in v0.2.

A purchaser may maintain a random 32-byte `RR`. Each package contains an authenticated recovery envelope that encrypts its exact O1 private material under a package-specific key derived from `RR`.

Normal playback/device migration SHOULD avoid loading `RR` when a surviving authorized device can enroll another device directly.

### R1

R1 is the mandatory portable passphrase-backup profile for `RR` and uses RFC 9106 Argon2id second-recommended parameters:

```text
memory       64 MiB
iterations   3
lanes        4
salt         128 bits
output       256 bits
```

The Argon2 output encrypts `RR` using AES-256-GCM.

Threshold/compartmented recovery remains an optional architecture extension; it is not required for v0.2 wire interoperability.

## 13. Device grants

The owner may authorize devices without publisher participation.

### S1

Portable software profile using X25519 HPKE.

### H1

Commodity hardware profile using P-256 HPKE-compatible ECDH so the device private key can remain non-exportable in TPM/Secure Enclave/StrongBox/PIV-like facilities.

Attestation and measured-boot/PCR binding are OPTIONAL and MUST NOT gate ordinary ownership, migration, recovery, or preservation.

The owner signs each device grant. Copying grant metadata to another machine is insufficient without the matching device key.

Hardware protects operational credentials; it does not guarantee that `K_album`, decoded FLAC, PCM, or output cannot be captured by a privileged authorized endpoint.

## 14. Ownership ledger

The ownership ledger exists to provide historical issuance evidence and, for transferable titles, one canonical current title state. It is not a license server.

The deterministic v0.2 state machine is specified in `spec/ledger-v0.2.md`.

Record classes:

```text
ISSUE
TRANSFER
INHERIT
REISSUE
REKEY/RECOVER
```

Canonical state maps each `title_id` to at most one current `ownership-output`.

A transfer consumes the current output exactly once and creates a new output. This provides title-layer double-spend prevention under the deployment's consensus assumptions.

## 15. Ledger finality boundary

MUSICPKG standardizes:

- record validity;
- deterministic state transitions;
- history/state Merkle roots;
- checkpoint representation/signatures;
- validator-set rotation evidence;
- proof verification.

It does not define validator networking/proposer/view-change packets. A transfer-enabled deployment SHOULD use a mature BFT replicated-state-machine protocol consistent with its claimed fault tolerance.

For one Byzantine fault, the current production target is at least 4 validators with a 3-of-4 finalization threshold plus actual locking/finality semantics. Signature count alone is not consensus.

If transferable title is not offered, an ecosystem MAY use a simpler append-only transparency-log profile rather than full canonical title consensus.

## 16. Offline ledger proof

A purchased package carries sufficient historical evidence to verify that its ISSUE record was committed to a finalized checkpoint and to validate the ledger genesis/validator history needed for that checkpoint.

Ordinary playback uses this bundled proof and does not need current network state.

A fresh current-state query is only required for operations that need current title, such as resale, inheritance, title inspection, reissue, or rekey governance.

## 17. Ownership verification

`spec/verification-v0.2.md` defines the machine-readable verification states.

A verifier can ask the owner to sign a fresh challenge binding:

- purpose;
- verifier audience;
- random nonce;
- asset/title/output IDs;
- checkpoint ID when current title is required;
- expiry when relevant.

The proof uses the pseudonymous per-title Ed25519 owner key and does not require purchaser name/email/payment information.

Verification states distinguish at least:

```text
PACKAGE_VALID
HISTORICAL_OWNER_VALID
CURRENT_OWNER_VALID
CURRENT_STATE_UNKNOWN
NOT_CURRENT_OWNER
```

`CURRENT_STATE_UNKNOWN` MUST NOT invalidate ordinary offline playback.

## 18. Transfer / re-ownership

For a TRANSFER:

1. recipient creates a fresh OwnerKeySet;
2. seller/authorized implementation creates a new E1 `K_album` envelope to the recipient;
3. transfer body commits the current input output and new owner/envelope;
4. seller signs transfer authorization;
5. recipient signs acceptance/key possession;
6. canonical ledger finality consumes the seller output and creates the recipient output.

The encrypted media normally remains unchanged. No publisher participation is required.

Payment, escrow, marketplace listing, taxation, royalties, and legal authorization to resell are outside the base protocol. A protocol-valid title transfer does not itself establish a jurisdictional digital-first-sale right.

Transfer does not prove deletion of historical plaintext or old keys.

## 19. Reissue and rekey

`REISSUE` creates a new `package_id`/publisher issuance while preserving asset/title lineage, typically when new media/watermark/package bytes are needed.

`REKEY/RECOVER` replaces public owner key material for the same package/title after compromise/recovery without re-encrypting the media.

Rotating only the purchaser Recovery Root and local/package recovery envelopes does not require a ledger transition when the public OwnerKeySet remains unchanged.

## 20. Provenance and music metadata

C2PA Content Credentials are a recommended producer profile for source/mastering/publisher provenance where available. C2PA trust-list state MUST NOT gate MUSICPKG playback.

Music metadata SHOULD align conceptually with DDEX entities/identifiers, but ordinary players are not required to implement DDEX B2B choreography.

External industry identifiers are descriptive metadata; MUSICPKG internal cryptographic IDs/hashes govern protocol identity.

## 21. Forensic watermark

A producer MAY embed a versioned per-issuance forensic fingerprint before lossless encoding/encryption.

The watermark:

- MUST NOT be the root playback authorization mechanism;
- MUST NOT require purchaser PII in the waveform;
- SHOULD use opaque issuance identifiers/commitments;
- MAY use collusion-resistant coding in commercial anti-piracy profiles;
- MUST be treated as probabilistic evidence subject to removal, overwrite, forgery, collusion, neural transformation, false positive, and physical-path attacks.

Ordinary playback need not run a watermark detector.

## 22. Failure semantics

### Publisher/retailer disappears

Existing legitimate playback, backups, migration, and recovery continue.

### Ledger unavailable

Existing playback and bundled historical proof continue. Current-title freshness may be unknown and new title-changing transactions cannot finalize.

### Device lost/reset

The operational grant is lost; owner recovery/enrollment creates a replacement.

### Recovery Root lost

If no surviving owner/device authority or alternate recovery exists, ownership may become unrecoverable. MUSICPKG cannot manufacture lost cryptographic authority.

### Recovery Root stolen

A full RR plus packages can compromise the associated recovery scope. Offline storage, threshold recovery, compartmenting, and root rotation reduce exposure but cannot remove this fundamental recovery trade-off.

## 23. Security boundary

Cryptographically enforceable properties include:

- encrypted-artifact confidentiality without valid authority;
- authenticated media integrity;
- signatures/key/package binding;
- package-local recovery-envelope integrity;
- finalized title-state validity under the ledger assumptions.

Cost-raising properties include:

- non-exportable device keys;
- bounded buffers;
- sandboxing/debugger/core-dump restrictions;
- OS capture restrictions/protected media paths;
- optional secure decoder/DAC hardware.

Probabilistic/detectable properties include forensic watermark attribution.

MUSICPKG does not guarantee prevention of:

- privileged PCM/output capture;
- virtual audio capture on general-purpose desktops;
- analog recording;
- intentional sharing of full recovered authority;
- extraction from every possible compromised implementation;
- perfect watermark survival;
- proof that a previous owner deleted plaintext after transfer.

## 24. Privacy

The package/shared ledger MUST NOT require purchaser name, email, payment information, or reusable account identity.

Fresh per-title OwnerKeySets SHOULD be used to reduce cross-library correlation.

Ownership proofs reveal the title/output and pseudonymous key material necessary for the requested proof. They do not prove civil identity unless an external application deliberately binds that identity.

Zero-knowledge ownership proofs are deferred from the mandatory v0.2 profile; per-title pseudonymous keys plus nonce-bound signatures provide the baseline privacy model.

## 25. Verification and errors

Conforming tools MUST expose stable machine-readable verification/error categories equivalent to those in `spec/verification-v0.2.md`.

Implementations MUST distinguish malformed/tampered input, unsupported profiles, absent keys, historical validity, current-title unknown/stale state, and actual cryptographic invalidity.

## 26. Conformance classes

v0.2 defines:

- `musicpkg-package-producer`;
- `musicpkg-player`;
- `musicpkg-h1-player`;
- `musicpkg-ownership-verifier`;
- `musicpkg-ledger-validator`;
- `musicpkg-recovery-tool`;
- `musicpkg-forensic-verifier`.

The exact requirements are in `spec/verification-v0.2.md`.

## 27. Test vectors

Deterministic positive vectors are in:

```text
test-vectors/v0.2/crypto.json
```

They cover:

- deterministic CBOR bytes/hashes;
- track HKDF/AES-GCM;
- O1 keys;
- E1 X25519 HPKE;
- H1 P-256 HPKE;
- recovery envelope;
- R1 Argon2id/AES-GCM;
- device-grant signature;
- ownership-proof signature;
- publisher COSE Sign1 signature;
- RFC 9162 Merkle hash behavior.

Negative cases and expected error classes are in:

```text
test-vectors/v0.2/negative.json
```

Secrets/private scalars in test vectors are test-only values and MUST NOT be used in production.

## 28. IANA considerations

This working draft requests no IANA action.

`application/vnd.musicpkg+zip` is provisional experimental documentation only and is not claimed as a registered media type.

## 29. Normative/reference specifications

- RFC 2119 — Key words for use in RFCs to Indicate Requirement Levels.
- RFC 8174 — Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words.
- RFC 5869 — HKDF.
- RFC 8032 — Ed25519 / EdDSA.
- RFC 8949 — CBOR and Core Deterministic Encoding.
- RFC 9052 — COSE structures.
- RFC 9053 — COSE algorithms.
- RFC 9106 — Argon2.
- RFC 9162 — Certificate Transparency v2 Merkle Tree Hash/proof construction reused by MUSICPKG.
- RFC 9180 — HPKE.

## 30. v0.2 implementation artifacts

The interoperability contract is split deliberately:

```text
rfcs/0000-musicpkg.md          normative architecture/behavior overview
spec/musicpkg-v0.2.cddl        exact structured schemas
spec/crypto-v0.2.md            exact crypto/signing/hash constructions
spec/container-v0.2.md         physical package/parser/framing rules
spec/ledger-v0.2.md            deterministic title state/checkpoint rules
spec/verification-v0.2.md      verifier state/error/conformance model
test-vectors/v0.2/*.json       positive/negative deterministic vectors
```

If these artifacts conflict with higher-level `PRODUCT.md` / `architecture/invariants.md` ownership guarantees, the conflict is a specification defect and MUST be resolved before claiming stable v0.2 interoperability.
