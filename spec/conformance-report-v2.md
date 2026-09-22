# MUSICPKG Conformance Report v2

`musicpkg-conformance-report-v2` is the stable machine-readable output contract
for running the MUSICPKG v0.2 conformance corpus.

## Report

```json
{
  "schema": "musicpkg-conformance-report-v2",
  "fixture_version": "musicpkg-v0.2-draft",
  "summary": {
    "total": 49,
    "implementation_passed": 26,
    "contract_assertions_passed": 23,
    "implementation_mismatch": 0,
    "contract_assertion_mismatch": 0,
    "fixture_defect": 0
  },
  "cases": []
}
```

Each evaluated case contains `id`, `group`, `status`, and `evidence`.
`expected`, `actual`, and `detail` are present when useful for diagnosis.
`evidence` is omitted when a fixture defect prevents reliable evaluation.

The two evidence classes are:

- `implementation` — the case executes reference implementation behavior;
- `contract_assertion` — the case checks a frozen specification/fixture invariant
  but does not prove that the corresponding production parser, ledger, state
  machine, or product workflow exists.

The stable statuses are:

- `pass` — the evaluated behavior/assertion matches its committed contract;
- `implementation_mismatch` — executable reference behavior differs from the
  expected top-level result;
- `contract_assertion_mismatch` — a declarative contract assertion differs from
  the frozen expected result;
- `fixture_defect` — fixture/harness input cannot be evaluated reliably.

A fixture defect takes precedence over mismatches for process exit status because
the run itself is not trustworthy.

## Exit status

| Exit | Meaning |
| ---: | --- |
| `0` | all implementation checks and contract assertions passed |
| `1` | one or more implementation or contract-assertion mismatches |
| `2` | one or more fixture/harness defects |

## Conformance groups

The v0.2 report contains 49 checks:

- `positive`: 9 deterministic cryptographic/interoperability implementation checks;
- `negative`: 25 mutation/error checks, currently 17 implementation checks and
  8 contract assertions for parser/ledger/state behavior not yet implemented;
- `scenario`: 15 product/failure contract assertions.

Therefore a clean v0.2 run currently reports **26 implementation passes** and
**23 contract assertions passed**. It must not be described as 49 executable
implementation-conformance passes.

The report format is implementation-neutral. A second implementation may emit
this schema without using Rust or `musicpkg-core`. As reference APIs replace
contract-only stand-ins, individual cases may move from `contract_assertion` to
`implementation` without changing their normative fixture expectation.

## Authority

The published specification and fixtures define expected behavior. The report
records both executable implementation evidence and separately identified
contract assertions; neither makes implementation behavior normative. When an
implementation and the published contract differ, the discrepancy must be
resolved explicitly rather than normalizing the fixture to the implementation.
