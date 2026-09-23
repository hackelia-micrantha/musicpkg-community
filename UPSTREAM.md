# Upstream and provenance

`musicpkg-community` is the public interoperability surface for MUSICPKG. It is maintained from the private canonical engineering repository through deliberate, reviewed promotion rather than repository mirroring.

## Initial v0.2 publication

- Private canonical engineering repository: `hackelia-micrantha/musicpkg`
- Reviewed source snapshot: `a79e60e7976cb635b8959b451dd7b017e075caff`
- Published profile: MUSICPKG Working Draft v0.2
- Public surface: product contract, architecture invariants, RFC, normative specs, CDDL, and conformance fixtures
- Intentionally excluded: private research corpus, attack tooling, operational infrastructure, unpublished experiments, credentials, private CI/evidence, and implementation details not required for interoperability

## Authority model

The repositories have different authorities:

- `musicpkg-community` is authoritative for **published** wire-format, verification, and conformance contracts.
- `musicpkg` is authoritative for private engineering implementation, unpublished design/security work, adversarial research, and publication staging.

An implementation does not silently redefine the published format. If implementation evidence exposes a specification defect, the public contract is revised explicitly, reviewed, versioned, and accompanied by updated fixtures when behavior changes.

## Promotion policy

Only material required for independent implementation, verification, or useful public review is promoted. Unknown or ambiguously classified material stays private until reviewed.

A conforming producer, verifier, or player must be implementable from the public repository without access to the private repository or a Micrantha-operated service.

Public and private repositories are not bidirectional mirrors. Public contributions to normative behavior are resolved in the public contract first, then consumed and revalidated by the private implementation.


## Licensing boundary

Published specifications, RFCs, schemas, conformance fixtures, and documentation are Apache-2.0. Public reference implementation source is MPL-2.0 unless an individual file declares another SPDX identifier.

The existing private Rust `musicpkg-core` is MPL-2.0. Promotion may preserve that license; copying code into this repository does not relicense it under the repository's Apache-2.0 documentation/specification default. Normative behavior remains defined by the public specification and fixtures, not by the reference implementation.

## Conformance report v2 publication — 2026-09-22

- Private reviewed source: `hackelia-micrantha/musicpkg@b5d762e000b7a2c35f8cf458b56b4a7889648355`, `spec/conformance-report-v2.md` (blob `c4df128870a05a92148905f7e817f44879160ec8`).
- Promoted artifact: `spec/conformance-report-v2.md`, an Apache-2.0 public report-format contract.
- Verification: the private merged-main conformance gate passed in [run 35791608914](https://github.com/hackelia-micrantha/musicpkg/actions/runs/35791608914).
- Public scope of this publication is the **report contract only**. It does not publish or claim independent public execution of the Rust reference harness, crypto/ledger verification, or product workflows. That implementation and its public-only CI remain tracked by #3.
- The incompatible pre-merge `musicpkg-conformance-report-v1` draft was never promoted as a stable public interface.

## Executable public conformance promotion — 2026-09-23

- Reviewed private implementation source: `hackelia-micrantha/musicpkg@b5d762e000b7a2c35f8cf458b56b4a7889648355` (the exact merged-main harness verification baseline).
- Curated public source: `Cargo.toml`, `Cargo.lock`, `crates/musicpkg-core/`, `crates/musicpkg-conformance/`, and the portable `ci/check` command. The Rust crate manifests already declare MPL-2.0. Core Rust files acquired explicit `SPDX-License-Identifier: MPL-2.0` headers in this promotion without otherwise changing implementation behavior; the conformance source retained existing SPDX headers.
- The three public fixture files have byte-identical Git blobs to that reviewed source: `crypto.json` `a8d92b99f2a43b6f9d17304ba5256205e84eaddb`, `negative.json` `6caad6f1cf7b8a2773e2fb4f3de6b259adffece8`, `scenarios.json` `fa79bb178c533643fad0bb66c74e10057f543a4b`. No private test vector, research corpus, privileged runner configuration, credential, or operational tooling was mirrored.
- The public CI is an independent GitHub-hosted Cargo gate, intentionally not the private repository's Nix/JIT runner workflow. Source and fixture evaluation have no runtime private-repo or vendor-service dependency; fetching locked third-party Rust crates at build time may require ordinary internet access.
- The public harness is a **reference conformance implementation**, not a normative protocol definition or a complete package verifier. Current evidence comprises 26 implementation checks plus 23 labeled contract assertions; `musicpkg verify`, parser, canonical ownership ledger, and playback are not claimed as implemented.
