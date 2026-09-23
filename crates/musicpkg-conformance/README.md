# musicpkg-conformance

`musicpkg-conformance` is the MUSICPKG v0.2 interoperability harness. It
consumes only the committed `test-vectors/v0.2` fixtures and emits the
`musicpkg-conformance-report-v2` result contract.

`v2` is intentionally incompatible with the pre-merge `v1` draft: `v1`
aggregated successful checks as implementation passes, while `v2` explicitly
separates executable implementation evidence from contract assertions. The
pre-merge `v1` draft must not be treated as a stable public interface.

## Run

```bash
cargo run -p musicpkg-conformance -- --fixtures test-vectors/v0.2
cargo run -p musicpkg-conformance -- --fixtures test-vectors/v0.2 --json
```

Exit status is part of the contract:

- `0` — all executable checks and contract assertions match the published fixtures;
- `1` — at least one implementation check or contract assertion mismatches;
- `2` — fixture or harness input is missing, malformed, or internally inconsistent.

The current v0.2 corpus contains 49 checks: 9 positive cryptographic vectors,
25 negative mutation/error cases, and 15 product/failure scenarios.

## Evidence boundary

A clean v0.2 run currently reports **26 implementation passes** and **23
contract assertions passed**.

The implementation evidence consists of the 9 positive cryptographic vectors
plus 17 negative cases that execute `musicpkg-core` behavior. The remaining 8
negative parser/ledger/state checks use bounded harness-local stand-ins and the
15 product/failure scenarios compare fixture expectations to the frozen v0.2
contract oracle. Those 23 checks are useful specification assertions, but they
are explicitly **not** evidence that the corresponding production parser,
ledger/state machine, or end-to-end product workflow exists.

Every evaluated case therefore carries an `evidence` value of either
`implementation` or `contract_assertion`. Mismatches are likewise reported as
`implementation_mismatch` or `contract_assertion_mismatch`.

As real parser/ledger/state APIs land, individual checks should migrate from
`contract_assertion` to `implementation` rather than adding duplicate fixtures.

The public specification and fixtures are normative. This Rust implementation
must not silently redefine them; a mismatch is either an implementation defect,
a contract-assertion defect, or an explicitly reviewed specification change.

Public promotion must preserve the `v2` evidence semantics and may not publish
the discarded `v1` draft as an equivalent schema.

## License

MPL-2.0. Source files carry `SPDX-License-Identifier: MPL-2.0`.
