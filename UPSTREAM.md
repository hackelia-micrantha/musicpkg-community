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
