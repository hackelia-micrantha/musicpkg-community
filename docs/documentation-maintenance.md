# Public documentation maintenance

**Adoption:** Micrantha shared `documentation-sync` default, unchanged. Follow the [shared skill](https://github.com/hackelia-micrantha/.github/blob/main/skills/documentation-sync/SKILL.md) for evidence reconciliation, bounded ambiguity decisions, and review. This document records the project-specific authority and test entry points; it does not copy the organization skill.

## Publication and authority

- This repository owns the **published, versioned** MUSICPKG wire-format, verification and interoperability specifications, fixtures, reference-conformance interfaces, and their public explanatory material.
- The private `hackelia-micrantha/musicpkg` repository owns unpublished engineering, security research, operational implementation and **staging**, but cannot silently redefine published public specifications. Review public contract/fixture changes as explicit versioned changes first.
- [UPSTREAM.md](../UPSTREAM.md) records selective publication, source provenance, and Apache-2.0 (specs, documentation, fixtures) versus MPL-2.0 (reference implementation) licensing. Do not mirror unpublished implementation, credentials, research corpora, or private test material into this repository.
- This contract's adoption does not authorize publication of a new protocol version, release, licensing change, or private-source transfer.

## Public claim and validation boundaries

Review [README](../README.md), [product contract](../PRODUCT.md), [specifications](../spec/), [RFCs](../rfcs/), [test vectors](../test-vectors/), [reference conformance](../crates/musicpkg-conformance/README.md), [security policy](../SECURITY.md), and [upstream provenance](../UPSTREAM.md) together when a change affects their published claims.

The current published baseline is **v0.2 Working Draft**. The strict reference-conformance gate is **49** cases, separated into **26 executable reference checks** and **23 specification/contract assertions**. Do not imply that all 49 are executable product-verifier cases. The published independent ledger-state fixture is an interoperability oracle, not a complete production ownership-state or package-level verification implementation. Full `musicpkg verify` remains tracked in [issue #3](https://github.com/hackelia-micrantha/musicpkg-community/issues/3).

For every user-facing statement, distinguish **normative specification**, **reference implementation/test**, **planned implementation**, and **published supported consumer behavior**. In particular, copyability of ciphertext is not ownership, owner credentials do not make plaintext impossible to extract at playback, forensic watermarking is not an authorization gate, and offline ownership goals must not be presented as a completed end-to-end player or transfer service.

## Existing exact-head validation

```sh
# Rust reference corpus, locked dependencies, formatting, tests, Clippy,
# and the strict 49-case (26 executable / 23 contract assertion) JSON report.
./ci/check

# Independent Go interoperability tests when interop/go or its vectors change.
cd interop/go
go mod download
go mod verify
go test -mod=readonly -count=1 ./...
go vet -mod=readonly ./...
```

[Public conformance CI](../.github/workflows/conformance.yml) runs the Rust gate for PRs and main pushes on GitHub-hosted runners. [Public Go interoperability CI](../.github/workflows/go-interop.yml) is **path-scoped**, not a full-repository docs check: run it or the documented local command only where its code/vectors change. Passing source/test CI does not constitute publication of a completed package verifier or production trust/ownership service.

## Maintenance and decisions

Review claims after public specification or fixture revisions, ownership/recovery/ledger semantic changes, conformance-accounting changes, security profile proposals, release/install path changes, or public/private promotion decisions. Preserve version labels and exact corpus accounting. Prefer tests or deterministic checks for count, vector, link and manifest drift; retain human review for normative/security/licensing claims.

Surface ambiguous changes using the shared skill's `DOC-Q*` human decision queue. This public project has no additional `documentation-sync` specialization at present.
