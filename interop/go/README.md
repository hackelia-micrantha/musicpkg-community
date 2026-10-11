# Go interoperability probe (v0.2)

A standalone **standard-library-only** Go probe of public ISSUE/TRANSFER
record hashing, output-ID deterministic CBOR, seller/recipient signing inputs,
and Ed25519 test-vector signatures.

From `interop/go` run:

```sh
go test ./...
```

Tests read only the committed public `test-vectors/v0.2/ledger-state.json`
and `crypto.json`, never private files or Rust subprocess/FFI calls. Negative
probes cover switched signature domains, mutated bodies, corrupt signatures,
and invalid public-key lengths.

**Evidence limits:** This is not a general-purpose CBOR parser, strict
Ed25519 weak-key policy, COSE verifier, ownership state machine, package
validator, or implementation of the 49-case v0.2 conformance harness.
The existing **26 executable / 23 contract assertions** remain unchanged.
Neither `PACKAGE_VALID` nor `CURRENT_OWNER_VALID` is established. In
particular `VerifyVectorSignature` must not be used for authorization.

Go source is MPL-2.0; public fixture/spec licenses remain unchanged.
