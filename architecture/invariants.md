# Candidate v0.2 Architecture Invariants

Status: **candidate architecture frozen for RFC v0.2 serialization/protocol work**

This document freezes the ownership/trust model. RFC v0.2 may still choose exact algorithms, encodings, suite identifiers, field widths, parser limits, and test-vector bytes, but it MUST NOT silently change these invariants without reopening architecture review.

## Product contract

```text
Buy once -> own -> play on your devices -> back up -> prove ownership -> optionally transfer canonical title.

COPYING != OWNING
```

A completed purchase is ownership, not a renewable service permission.

## Consumer invariants

1. **Offline permanence.** Existing legitimate purchases remain playable without publisher, retailer, license server, or live ownership-ledger service.
2. **Arbitrary encrypted backup.** Copying `.musicpkg` bytes is allowed; package possession alone is insufficient for plaintext playback.
3. **Owner-controlled devices.** The purchaser controls ordinary post-purchase device enrollment. The publisher is not a device-domain controller.
4. **Recoverability.** Loss of a phone/laptop/TPM does not destroy ownership when purchaser recovery material survives.
5. **No mandatory vendor escrow/backdoor.** Recovery remains purchaser-controlled and publicly implementable.
6. **Pseudonymous ownership proof.** Civil identity/payment data are not required in the package or shared title state.
7. **No remote revocation of completed sale.** Mutable publisher/business policy cannot silently turn an owned package into an expired license.

## Publisher invariants

1. **Artifact copying is not usable ownership.** A copied protected package alone remains ciphertext.
2. **Authentic issuance is independently verifiable.** Publisher/package provenance and issuance are signed/bound.
3. **Title is independently verifiable.** Historical and fresh current-title state can be checked without a publisher playback server.
4. **Canonical transfer is single-spend.** For transferable titles, conflicting spends of one current output cannot both finalize under the ledger's consensus assumptions.
5. **Operational credentials can be hardened.** Hardware-backed device grants may make key cloning materially harder.
6. **Plaintext/output control is bounded.** MUSICPKG does not claim that an authorized owner cannot capture final PCM/analog output.
7. **Forensic evidence is optional/probabilistic.** Watermark evidence may identify issuance lineage but is never playback authorization or proof of human guilt.

## Authority separation

MUSICPKG has distinct authority classes:

```text
publisher issuance authority
per-purchase owner authority
purchaser Recovery Root
replaceable device playback authority
canonical title authority
forensic evidence/provenance authority
```

These MUST NOT collapse into a single vendor-controlled service.

## Package architecture

The logical package separates an immutable publisher-signed content/issuance core from a mutable authenticated ownership overlay/local state:

```text
IMMUTABLE CORE
  content manifest
  encrypted/watermarked audio
  original publisher issuance/provenance
  original historical ledger evidence

EVOLVING OWNERSHIP OVERLAY
  current owner binding
  current owner content-key envelope
  transfer/rekey evidence
  optional cached current-title proof

OWNER-LOCAL STATE
  recovery envelope
  device grants
  local hardening policy
```

Physical ZIP/member layout is an RFC v0.2 detail.

## Media encryption

- `K_album` is random per issued package.
- Per-track keys are independently derived with domain separation.
- Bounded media records use authenticated encryption and are authenticated before plaintext reaches the decoder.
- CENC's separation of encrypted media from key-management systems is retained conceptually, but confidentiality-only CTR/CBC is not the baseline.
- Partial/pattern encryption is not required without a concrete decoder/hardware need.

Exact mandatory media AEAD/nonce framing is RFC v0.2 work; AES-256-GCM is the leading mandatory profile candidate.

## Owner authority and recovery

Each title tenure/acquisition uses fresh independently generated owner key material. Owner keys are NOT deterministically derived from a library seed.

The owner key set structurally separates:

```text
signing authority        -> ownership proof/title transitions
key-encryption authority -> receives K_album envelopes
```

Exact algorithms are RFC v0.2 work.

A random 256-bit offline Recovery Root (`RR`) protects package-local encrypted recovery envelopes containing independently generated per-purchase owner private material.

Recovery profiles:

- R1: Argon2id-protected Recovery Root bundle;
- R2: threshold/Shamir recovery, e.g. 2-of-3, preferred high-assurance;
- R3: optional compartmented roots by collection/epoch.

The Recovery Root is not a daily playback key and is not sent to the publisher.

## Device grants

A software device-grant path is part of the portability/preservation baseline.

Optional Hardware Device Grant H1 is frozen at architecture level as:

```text
P-256 ECDH / HPKE-compatible DHKEM
HKDF-SHA-256
AES-256-GCM
```

Hardware attestation and PCR/measured-boot binding are optional assurance profiles, not playback/recovery requirements.

Device keys are disposable operational state. Hardware loss -> owner recovery/new grant, not ownership loss.

## Playback boundary

Baseline flow:

```text
verify package/title evidence
 -> open device grant
 -> K_album / K_track
 -> AEAD-authenticated bounded media record
 -> decoder
 -> PCM
 -> OS audio
 -> DAC/analog
```

Players should minimize key/plaintext lifetime, avoid plaintext temp files, use sandbox/least privilege, and fail closed on verification errors.

Platform capture restrictions/protected-media paths are optional hardening only. A dedicated secure decoder/DAC may keep digital PCM off the host but cannot prevent analog capture.

## Provenance and metadata

- C2PA is a **recommended producer provenance profile**, not a core playback requirement. C2PA trust-list changes cannot invalidate ownership/playback.
- DDEX concepts/identifiers may be carried as compact metadata, but full DDEX message choreography is not a player requirement.
- External identifiers are descriptive claims, not cryptographic object identities.

## Watermarking

- Watermarking is optional, versioned, replaceable forensic evidence.
- It carries opaque fingerprint/code material, never purchaser PII.
- Watermark extraction is not performed as a playback authorization step.
- Commercial profiles may use collusion-resistant fingerprint codes.
- A base transfer does not rewrite the original watermark; it remains origin-issuance evidence.
- Optional `REISSUE` may create a new current-owner-specific watermark without making publisher availability a transfer requirement.

## Ownership verification

Base v0.2 verification uses pseudonymous per-title owner keys and standard challenge signatures; zero-knowledge proof is deferred.

Verification predicates are separate:

```text
PACKAGE_VALID
HISTORICAL_OWNER_VALID
CURRENT_OWNER_VALID
CURRENT_STATE_UNKNOWN
NOT_CURRENT_OWNER
```

A fresh current-title check is required only for title-sensitive operations. `CURRENT_STATE_UNKNOWN` MUST NOT disable ordinary playback.

## Transfer / re-ownership

Transfer remains in scope but may be disabled by a title's transfer policy.

For a transferable title:

1. recipient generates a fresh independent OwnerKeySet and recovery envelope;
2. current owner rewraps existing `K_album` to the recipient key;
3. sender signs spending of the current title output;
4. recipient signs acceptance/binding of the new owner key/envelope;
5. canonical title state finalizes one transition;
6. media ciphertext is unchanged.

Payment, pricing, tax, royalties, KYC, listings, and fair-exchange escrow are outside the base protocol.

A finalized transfer guarantees unique canonical title, not deletion/forgetting of old plaintext/keys.

Technical canonical transfer does not itself establish a legal resale/right under copyright law.

## Ledger

The ledger exists only where canonical transferable title is required.

It combines:

```text
append-only Merkle history
+
deterministic UTXO-like ownership state
```

Supported transition classes include:

```text
ISSUE
TRANSFER
INHERIT
REISSUE
REKEY / RECOVER
```

A naïve 2-of-3 signature rule is not Byzantine-safe.

If Byzantine-safe transferable title is claimed, production target is a mature BFT protocol with conventional quorum intersection; for `f=1`, architecture target is 4 validators with 3-of-4 finality plus actual locking/finality semantics.

The byte-level checkpoint/state proof is RFC v0.2 work. The consensus transport/library may remain deployment-specific provided its finalized checkpoints satisfy the spec's safety/verifier contract.

If transferable title is absent, the architecture SHOULD collapse to a simpler signed transparency log.

## Failure semantics

If publisher/store disappears:

- existing playback works;
- recovery/device migration works;
- historical publisher verification remains possible from preserved evidence.

If all ledger operators disappear:

- existing playback works;
- historical bundled proofs verify;
- recovery works;
- new transfers/rekeys/fresh title queries may stop.

If every device and all purchaser recovery material are lost:

- recovery may be impossible by design; there is no mandatory publisher backdoor.

If a prior owner retained plaintext after transfer:

- protocol cannot erase that history; canonical title still transfers.

## Legal / commerce boundary

The base format is not:

- a payment network;
- cryptocurrency;
- a marketplace;
- a general smart-contract platform;
- a legal determination of digital first-sale rights;
- a promise of exclusive physical/plaintext possession after resale.

These boundaries are deliberate architecture constraints informed by prior art and the product contract.

## RFC v0.2 may choose without reopening architecture

The RFC may now freeze:

- exact CDDL schemas;
- deterministic CBOR/COSE rules;
- owner/publisher/validator crypto suite IDs;
- HPKE suite IDs;
- media record binary framing and nonce/AAD construction;
- package ZIP constraints/MIME type;
- recovery bundle/share encoding;
- title/checkpoint/state-proof byte formats;
- validator-set rotation proof format;
- parser/resource limits;
- conformance classes;
- exact error states;
- positive/negative byte test vectors.

Any change that reintroduces publisher playback authority, makes fresh ledger state a playback lease, removes purchaser-controlled recovery, equates transfer with cryptographic deletion, or weakens `COPYING != OWNING` requires architecture review rather than an RFC-only edit.
