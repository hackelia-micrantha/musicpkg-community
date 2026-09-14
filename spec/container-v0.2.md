# MUSICPKG v0.2 Container Profile

Status: Working Draft

MUSICPKG uses a ZIP-compatible physical container with a small, deterministic logical object model. ZIP is only the transport envelope; MUSICPKG cryptographic identity comes from signed/hashes of logical object bytes, never from ZIP central-directory bytes.

## 1. File extension and media type

Extension:

```text
.musicpkg
```

Provisional media type for experiments:

```text
application/vnd.musicpkg+zip
```

No IANA registration is claimed by this working draft.

## 2. Required logical objects

A v0.2 package contains at least:

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

Optional objects include:

```text
provenance/c2pa.*
watermark/descriptor.cbor
artwork/*
metadata/*
```

Mutable device-local state MUST NOT be required inside the archival `.musicpkg` artifact. Device grants SHOULD be stored in a player-owned sidecar/state store. If a tool chooses to embed local state in a working copy, it MUST reside beneath `local/` and MUST NOT be covered by the publisher content-manifest commitment.

## 3. ZIP profile

v0.2 requirements:

- single-disk ZIP only;
- standard ZIP or ZIP64 allowed;
- ZIP encryption MUST NOT be used;
- compression method MUST be `STORE` (method 0) for all security-sensitive/normative objects;
- producers SHOULD use `STORE` for every object to minimize parser ambiguity and resource-amplification risk;
- archive entry names are UTF-8 and use `/` as the only separator;
- symlink/device/special-file entries are prohibited;
- duplicate logical paths are prohibited;
- absolute paths are prohibited;
- empty path segments other than a terminal directory marker are prohibited;
- `.` and `..` path segments are prohibited;
- backslash (`\`), NUL, and Windows drive-prefix forms are prohibited;
- path comparison for duplicate detection is exact UTF-8 byte comparison after validating the above rules; implementations MUST NOT apply locale-specific case folding;
- multi-disk/spanned archives are prohibited;
- a parser MUST reconcile local-header and central-directory names/sizes and reject conflicts rather than choosing one representation.

ZIP CRC values are transport diagnostics only. MUSICPKG integrity comes from cryptographic object hashes and AEAD/signatures.

## 4. Resource limits

Conforming general-purpose v0.2 players MUST enforce limits before allocating/decompressing/parsing attacker-controlled data.

Baseline limits:

```text
maximum archive entries                 4096
maximum UTF-8 path length               255 bytes
maximum single CBOR/COSE metadata file  16 MiB
maximum artwork/auxiliary object        256 MiB
maximum track count                     1024
maximum media record plaintext          1 MiB
maximum ledger proof chain entries      256
maximum validator set size              64
maximum ownership challenge audience    1024 UTF-8 bytes
```

A player MAY impose lower implementation limits and SHOULD expose a distinct `RESOURCE_LIMIT` error rather than treating such input as a cryptographic failure.

Large encrypted track objects may exceed 4 GiB through ZIP64. Their expected exact sizes are committed in `manifest/content.cbor`; readers SHOULD stream/hash rather than allocate whole objects.

## 5. Security-sensitive CBOR

The following objects MUST use the CDE rules in `spec/crypto-v0.2.md` and conform to `spec/musicpkg-v0.2.cddl`:

- media descriptor;
- content manifest;
- owner/recovery envelopes;
- watermark descriptor;
- ledger genesis/records/proofs/checkpoints;
- ownership/title structures.

A parser MUST reject:

- indefinite lengths;
- duplicate map keys;
- non-shortest integer/length encodings;
- map keys out of deterministic order;
- values outside the CDDL constraints;
- trailing bytes after the expected top-level data item;
- unknown fields in closed v0.2 structures.

## 6. Manifest construction order

To avoid hash/signature cycles, producers build a package in this order:

```text
1. choose package_id / asset_id / title_id / track_ids
2. create optional watermark descriptor/commitment
3. create manifest/media.cbor
4. media_hash = SHA-256(exact media.cbor bytes)
5. derive track keys and encrypt authenticated records using media_hash in AAD
6. create authenticated track indexes
7. hash all immutable package objects covered by content manifest
8. create manifest/content.cbor
9. content_manifest_hash = SHA-256(exact content.cbor bytes)
10. create owner content-key envelope
11. create publisher issuance payload and COSE_Sign1
12. create ledger ISSUE record/proof/checkpoint bundle
13. create recovery envelope
14. write final ZIP container
```

No hash field is allowed to depend recursively on the object that contains it.

## 7. Content-manifest object set

`manifest/content.cbor` MUST commit at least:

- `manifest/media.cbor`;
- every encrypted audio object;
- every track index;
- watermark descriptor when present;
- immutable artwork/metadata whose authenticity is part of the purchase.

It MUST NOT include:

- itself;
- `ownership/issuance.cose`;
- current owner envelopes created after issuance;
- recovery envelopes that may be rotated;
- mutable device grants;
- fresh current-title proof bundles.

The publisher issuance separately signs `content_manifest_hash`, initial owner binding/envelope hash, ledger identity, and policy. This separation permits owner recovery/transfer overlays to evolve without modifying the immutable media core.

## 8. Object hashing

For each `object-reference` in the content manifest:

```text
hash = SHA-256(exact uncompressed object bytes)
size = exact uncompressed object byte length
```

The archive compression/ZIP header representation has no cryptographic meaning.

Before using a security-sensitive object, a player MUST verify its committed hash/size whenever that object is covered by the content manifest.

For large track objects this verification MAY be streamed concurrently with reading, but plaintext from a media record MUST NOT be released until:

1. the package/issuance chain is otherwise valid;
2. the record itself authenticates successfully;
3. the implementation has a strategy that ultimately detects a whole-object hash mismatch.

A strict verifier MAY hash the entire encrypted track before playback; a streaming player MAY combine per-record AEAD verification with a running whole-object hash and treat a final object-hash mismatch as package tampering.

## 9. Track object framing

An encrypted track object is only the concatenation of AES-GCM ciphertext records in ascending record-index order. It contains no unauthenticated per-record header.

For each index entry:

```text
ciphertext_length = plaintext_length + 16
```

Validation rules:

- first record index is 0;
- record indices increase by exactly 1;
- first ciphertext/plaintext offsets are zero;
- offsets are contiguous;
- every non-final record SHOULD have the configured maximum plaintext size;
- final plaintext offset + final plaintext length equals track plaintext byte length from media descriptor;
- final ciphertext offset + final ciphertext length equals encrypted track object size;
- index entry count equals `record_count` in the track descriptor;
- no record number may repeat under a track key.

The index object itself is immutable and committed by the content manifest.

## 10. Initial package verification order

A player opening a package SHOULD use this order to fail early and avoid unnecessary key operations:

```text
safe ZIP parse
 -> CDE/CDDL parse of core manifests
 -> verify content-manifest references/path constraints
 -> verify publisher issuance COSE signature
 -> verify package/manifest/initial-owner/ledger bindings
 -> verify bundled ledger issuance/checkpoint evidence
 -> establish owner or device authority
 -> open K_album envelope
 -> verify track index
 -> derive K_track
 -> authenticate/decrypt bounded records
 -> decode FLAC
```

Fresh current-title state is deliberately absent from the ordinary playback path.

## 11. Mutation model

The immutable publisher media core SHOULD remain byte-for-byte stable across ordinary ownership transfers.

Mutable/evolving state includes:

- current owner envelope;
- current title proof/checkpoint cache;
- recovery envelope rotation;
- device grants;
- transfer/rekey history references.

Implementations MAY represent this state in sidecar files/databases rather than rewriting the archival ZIP. A future standardized mutable-overlay packaging profile may be defined, but v0.2 interoperability does not require ZIP mutation for transfer or device enrollment.
