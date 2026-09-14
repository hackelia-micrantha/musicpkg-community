# MUSICPKG v0.2 Verification and Conformance

Status: Working Draft

This document defines the observable verification state machine for conforming players, ownership verifiers, and publisher/title verifiers.

## 1. Verification principles

A conforming implementation MUST distinguish:

- malformed/unsupported input;
- cryptographic invalidity/tampering;
- lack of the required owner/device key;
- valid historical ownership with unknown current title;
- current canonical ownership;
- policy/legal questions outside the protocol.

An implementation MUST NOT collapse `CURRENT_STATE_UNKNOWN` into `INVALID` and MUST NOT require fresh ledger access for ordinary playback.

## 2. Package verification phases

### Phase P0 — Safe container

Validate the ZIP/container rules from `container-v0.2.md` before processing security-sensitive content.

Failure classes:

```text
MALFORMED_CONTAINER
UNSAFE_PATH
DUPLICATE_PATH
UNSUPPORTED_ZIP_FEATURE
RESOURCE_LIMIT
```

### Phase P1 — Structured metadata

Parse CDE CBOR/COSE against `musicpkg-v0.2.cddl`.

Failure classes:

```text
MALFORMED_CBOR
NON_DETERMINISTIC_CBOR
DUPLICATE_CBOR_KEY
SCHEMA_VIOLATION
UNSUPPORTED_PROFILE
```

### Phase P2 — Immutable media core

Verify object references and exact hashes/sizes from `manifest/content.cbor`.

Failure classes:

```text
OBJECT_MISSING
OBJECT_SIZE_MISMATCH
OBJECT_HASH_MISMATCH
MEDIA_DESCRIPTOR_MISMATCH
```

### Phase P3 — Publisher issuance

Verify COSE_Sign1, protected headers, issuer ID, content manifest binding, initial owner binding, ledger identity, and transfer policy.

Failure classes:

```text
ISSUER_KEY_UNKNOWN
ISSUER_SIGNATURE_INVALID
ISSUANCE_BINDING_INVALID
UNSUPPORTED_ISSUER_PROFILE
```

`ISSUER_KEY_UNKNOWN` means the signature cannot be trusted under the verifier's issuer-trust policy; it is distinct from a mathematically invalid signature.

### Phase P4 — Bundled ledger issuance evidence

Verify genesis/ledger identity, validator set/proof chain, finalized checkpoint signatures, threshold, record inclusion proof, ISSUE record, and package/owner/watermark commitments.

Failure classes:

```text
LEDGER_ID_MISMATCH
LEDGER_GENESIS_MISMATCH
VALIDATOR_SET_INVALID
CHECKPOINT_SIGNATURE_INVALID
CHECKPOINT_THRESHOLD_NOT_MET
CHECKPOINT_CHAIN_INVALID
LEDGER_INCLUSION_INVALID
ISSUE_RECORD_MISMATCH
```

No network operation is required by this phase.

### Phase P5 — Playback authority

The player establishes either:

- durable owner authority sufficient to open E1 and/or enroll a device; or
- an authorized S1/H1 device grant.

Failure classes:

```text
KEY_NOT_HELD
DEVICE_GRANT_INVALID
OWNER_SIGNATURE_INVALID
HPKE_OPEN_FAILED
RECOVERY_REQUIRED
```

`KEY_NOT_HELD` is an authorization/possession result, not evidence that the package is malformed or counterfeit.

### Phase P6 — Track authentication/decryption

Validate track index invariants, derive `K_track`, and authenticate each AES-GCM record before release.

Failure classes:

```text
TRACK_INDEX_INVALID
RECORD_SEQUENCE_INVALID
AEAD_AUTH_FAILED
TRACK_HASH_MISMATCH
```

A conforming player MUST stop releasing further plaintext from the affected track after authentication failure.

## 3. Package verification result

Machine-readable result:

```text
PackageResult {
    package_valid: bool
    issuer_signature: valid | invalid | key_unknown
    ledger_issuance: valid | invalid
    authority: owner | device | not_held | recovery_required
    playback: allowed | denied
    current_title: not_checked | current | not_current | unknown
    errors: [ErrorCode]
}
```

`playback=allowed` requires P0–P6 success for the material being played and valid owner/device authority. It does not require current-title freshness.

## 4. Ownership verification levels

### PACKAGE_VALID

Establishes:

- package/manifest integrity;
- authentic publisher issuance under the verifier's issuer trust policy;
- valid bundled ledger issuance evidence.

It says nothing about current claimant key control.

### HISTORICAL_OWNER_VALID

Additionally verifies:

- an authentic ownership output from issuance/history;
- a nonce-bound owner challenge signature under that output's owner key.

This proves claimant control of a cryptographic owner key associated with a real historical title state.

### CURRENT_OWNER_VALID

Additionally verifies a fresh finalized current-title proof:

- same `title_id`;
- included `ownership-output` in current state tree;
- output binding to claimant owner key;
- finalized checkpoint accepted under validator-set proof chain;
- challenge bound to `checkpoint_id` when requested.

### CURRENT_STATE_UNKNOWN

Returned when historical/package/key evidence is valid but sufficiently fresh canonical state cannot be obtained or verified.

This state MUST NOT disable ordinary offline playback.

### NOT_CURRENT_OWNER

Returned when fresh canonical state proves a different current output for the same `title_id` or the claimed output has been replaced by a later finalized transition.

This result affects transfer/marketplace/current-title claims, not historical authenticity.

## 5. Ownership challenge requirements

A verifier requesting proof MUST generate at least 32 random nonce bytes.

The challenge MUST bind:

- protocol version;
- purpose code;
- verifier audience;
- nonce;
- asset/title/output IDs;
- checkpoint ID for a current-owner proof;
- an expiry where interactive verification is time-sensitive.

Purpose codes are application-scoped. Reusing a proof obtained for one audience/purpose in another MUST fail due to the signed challenge binding.

## 6. Publisher verification

A publisher or marketplace can verify ownership without becoming a playback authority:

```text
verify package/issuance
 -> verify current-title proof when freshness matters
 -> send fresh ownership challenge
 -> verify claimant signature
```

No civil identity/account lookup is required by the MUSICPKG protocol.

A business MAY separately require identity/payment/account information for a transaction, but that data is outside the ownership proof.

## 7. Transfer preconditions

Before accepting a transfer proposal, recipient/marketplace SHOULD verify:

1. immutable package/issuance validity;
2. seller is `CURRENT_OWNER_VALID` at a sufficiently fresh checkpoint;
3. seller transfer policy permits transfer;
4. recipient's new OwnerKeySet is syntactically valid;
5. recipient proves control of its new signing key;
6. proposed E1 envelope hash is bound in the transfer body;
7. seller and recipient signatures validate;
8. input output remains unspent at finalization.

Final ownership changes only when the transfer record is finalized by canonical ledger state.

A signed-but-unfinalized transfer proposal is not title.

## 8. Freshness

MUSICPKG does not define a global maximum age for a current-title checkpoint because deployment availability and transaction risk vary.

A verifier that needs current title MUST apply an explicit freshness policy and report its decision. It MUST distinguish:

```text
checkpoint cryptographically valid but too old for local policy
```

from:

```text
checkpoint cryptographically invalid
```

Recommended error/status:

```text
CHECKPOINT_STALE
```

This status is non-fatal for ordinary playback.

## 9. Required error taxonomy

Conforming implementations MUST expose stable machine-readable categories at least equivalent to:

```text
OK
MALFORMED_CONTAINER
UNSAFE_PATH
DUPLICATE_PATH
UNSUPPORTED_ZIP_FEATURE
RESOURCE_LIMIT
MALFORMED_CBOR
NON_DETERMINISTIC_CBOR
DUPLICATE_CBOR_KEY
SCHEMA_VIOLATION
UNSUPPORTED_PROFILE
OBJECT_MISSING
OBJECT_SIZE_MISMATCH
OBJECT_HASH_MISMATCH
MEDIA_DESCRIPTOR_MISMATCH
ISSUER_KEY_UNKNOWN
ISSUER_SIGNATURE_INVALID
ISSUANCE_BINDING_INVALID
LEDGER_ID_MISMATCH
LEDGER_GENESIS_MISMATCH
VALIDATOR_SET_INVALID
CHECKPOINT_SIGNATURE_INVALID
CHECKPOINT_THRESHOLD_NOT_MET
CHECKPOINT_CHAIN_INVALID
CHECKPOINT_STALE
LEDGER_INCLUSION_INVALID
ISSUE_RECORD_MISMATCH
KEY_NOT_HELD
RECOVERY_REQUIRED
RECOVERY_AUTH_FAILED
DEVICE_GRANT_INVALID
OWNER_SIGNATURE_INVALID
HPKE_OPEN_FAILED
TRACK_INDEX_INVALID
RECORD_SEQUENCE_INVALID
AEAD_AUTH_FAILED
TRACK_HASH_MISMATCH
HISTORICAL_OWNER_VALID
CURRENT_OWNER_VALID
CURRENT_STATE_UNKNOWN
NOT_CURRENT_OWNER
TRANSFER_POLICY_DENIED
TRANSFER_SIGNATURE_INVALID
TRANSFER_INPUT_SPENT
TRANSFER_NOT_FINAL
```

Implementations MAY expose more detailed diagnostics, but MUST NOT leak key material or secret-dependent internal values in error messages/logs.

## 10. Conformance classes

### `musicpkg-package-producer`

Creates CDE objects, authenticated media, publisher issuance, initial E1 envelope, and initial ledger evidence conforming to v0.2.

### `musicpkg-player`

Implements P0–P6, at least software S1/owner-authority playback, and all mandatory crypto profiles required for ordinary playback.

### `musicpkg-h1-player`

A `musicpkg-player` additionally implementing Hardware Device Grant H1.

### `musicpkg-ownership-verifier`

Verifies historical/current title evidence and owner challenge signatures without needing to decrypt audio.

### `musicpkg-ledger-validator`

Validates deterministic ownership state transitions and participates in a deployment's finalized checkpoint protocol.

### `musicpkg-recovery-tool`

Implements R1 bundle recovery plus package-local recovery-envelope reconstruction/re-enrollment.

### `musicpkg-forensic-verifier`

Implements one or more watermark detector profiles and cryptographic commitment/issuance correlation. It is deliberately separate from playback conformance.

## 11. Negative conformance requirements

The conformance suite MUST include at least:

- duplicate ZIP path;
- path traversal;
- duplicate CBOR key;
- non-deterministic CBOR encoding;
- unknown mandatory profile;
- mutated media descriptor;
- mutated ciphertext object;
- forged publisher signature;
- wrong owner E1 key;
- wrong HPKE context/AAD;
- invalid recovery passphrase/tag;
- copied H1 grant without matching hardware key;
- reordered/missing media record;
- invalid record tag;
- forged ISSUE record;
- invalid Merkle proof;
- insufficient checkpoint signatures;
- duplicate validator signature;
- wrong validator set after rotation;
- stale-but-valid current checkpoint;
- replayed ownership proof to another nonce/audience/purpose;
- transfer from already-spent input;
- conflicting transfer proposal where only one can finalize.

Each vector MUST specify the exact expected top-level result/error category.
