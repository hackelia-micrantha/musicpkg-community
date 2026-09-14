# Contributing

MUSICPKG Community accepts contributions to the public specification, interoperability fixtures, conformance behavior, and reference tooling.

## Design constraints

Changes must preserve the product contract unless an RFC explicitly proposes an architecture revision:

- completed purchase is ownership, not a renewable playback lease;
- ordinary playback remains offline-capable after legitimate enrollment;
- publisher, retailer, or ledger disappearance does not revoke an existing legitimate purchase;
- `COPYING != OWNING` remains the core artifact/authority invariant;
- purchaser-controlled recovery and device enrollment remain vendor-independent;
- fresh current-title state is distinct from historical package/playback validity;
- transfer does not claim cryptographic deletion of a previous owner's historical plaintext or keys;
- publisher verification must not become publisher playback authority;
- watermarking remains forensic evidence, not authorization.

## Specification changes

Normative behavior changes should include:

1. the RFC/specification change;
2. CDDL/schema changes where applicable;
3. positive and negative fixture updates;
4. explicit compatibility/versioning impact;
5. a clear statement when the change alters a frozen architecture invariant rather than only serialization/interoperability detail.

Implementation findings that contradict the current specification should be filed as specification defects rather than patched into one implementation silently.

## Public/private boundary

Do not submit private infrastructure, credentials, publisher operational data, unpublished attack corpora, customer data, or material copied from the private engineering repository unless it has been deliberately reviewed for public release.
