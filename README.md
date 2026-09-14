# MUSICPKG Community

MUSICPKG is an open digital music ownership format built around a simple product contract:

> Buy a music file once. Own it. Play it on your devices. Back it up. Prove it is yours. Transfer ownership legitimately when the title permits it.

The central distinction is:

```text
COPYING != OWNING
```

This repository is the public interoperability surface for MUSICPKG. A conforming producer, verifier, or player must be implementable from the material published here without requiring access to private Micrantha repositories or vendor services.

## Repository boundary

`hackelia-micrantha/musicpkg-community` is authoritative for **published** MUSICPKG wire-format and interoperability contracts:

- RFCs and protocol specifications;
- deterministic schemas and cryptographic profiles;
- container and parser rules;
- ownership and transfer semantics;
- ledger/checkpoint verification semantics;
- machine-readable verification states and errors;
- positive, negative, and product/failure test vectors;
- public conformance tooling and reference-verifier interfaces.

`hackelia-micrantha/musicpkg` remains the private canonical engineering repository for implementation, unpublished design work, security research, adversarial testing, operational tooling, and publication staging.

Published contracts do not silently change to match an implementation. If implementation evidence exposes a specification defect, the public contract is revised explicitly and versioned.

## Status

The current public baseline is **MUSICPKG v0.2 Working Draft**, published from the reviewed v0.2 interoperability corpus. Start with [`rfcs/0000-musicpkg.md`](rfcs/0000-musicpkg.md), the normative [`spec/`](spec/) documents, and [`test-vectors/v0.2/`](test-vectors/v0.2/).

## Security model

MUSICPKG aims to provide durable offline ownership, owner-controlled recovery and device enrollment, authentic publisher issuance, and meaningful resistance to trivial artifact redistribution without placing a publisher or service in the ordinary playback path.

It does **not** claim that a general-purpose endpoint can make plaintext or analog capture impossible. Watermarking is forensic evidence, not authorization.

## Relationship to the private repository

This repository is deliberately curated rather than blindly mirrored. Public interoperability contracts and fixtures are promoted here after review; private operational data, attack tooling, unpublished experiments, and sensitive implementation details remain outside the public surface.

See [`UPSTREAM.md`](UPSTREAM.md) for the promotion and provenance contract.

## License

The published specification, documentation, schemas, and test vectors are licensed under Apache License 2.0. Reference implementation source is licensed under MPL-2.0 unless a file states otherwise. See [`LICENSES/README.md`](LICENSES/README.md) for the subtree and SPDX rules.
