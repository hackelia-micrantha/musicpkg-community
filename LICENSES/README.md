# License boundaries

MUSICPKG Community intentionally uses a mixed-license layout.

| Material | Default license |
| --- | --- |
| `README.md`, `PRODUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`, `UPSTREAM.md` | Apache-2.0 |
| `architecture/` | Apache-2.0 |
| `rfcs/` | Apache-2.0 |
| `spec/` | Apache-2.0 |
| `test-vectors/` | Apache-2.0 |
| reference implementation source (`crates/`, `reference/`, or equivalent) | MPL-2.0 |

The root [`LICENSE`](../LICENSE) contains the Apache License 2.0 text. [`MPL-2.0.txt`](MPL-2.0.txt) contains the Mozilla Public License 2.0 text for reference source.

Reference source files must include `SPDX-License-Identifier: MPL-2.0`. An explicit per-file SPDX identifier overrides the subtree default. Publication from the private repository preserves the source file's existing license; repository movement never implies relicensing.

The specification and conformance fixtures are normative. Reference implementations are non-normative evidence of interoperability and must not silently redefine the public contract.
