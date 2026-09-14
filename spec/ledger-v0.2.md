# MUSICPKG v0.2 Ownership Ledger Profile

Status: Working Draft

This document specifies the deterministic state transition and finalized-proof surface that v0.2 implementations must share. It intentionally does not define a peer-to-peer consensus transport or invent a new BFT protocol.

## 1. State model

Canonical ownership state is a map:

```text
State: title_id -> ownership-output
```

There is at most one current output for a `title_id`.

Each accepted state-changing record consumes zero or one current output and creates exactly one new current output:

| Record | Consumes | Creates |
|---|---|---|
| ISSUE | none | initial output |
| TRANSFER | current title output | new-owner output |
| INHERIT | current title output | heir output |
| REISSUE | current title output | same owner, new package output |
| REKEY | current title output | fresh owner-key output |

## 2. Common validation

Before applying any record:

1. CDE/CDDL validation MUST succeed.
2. `record_hash = SHA-256(CDE(record))`.
3. every referenced owner/issuer signature MUST validate over the exact signing input defined in `crypto-v0.2.md`;
4. identifiers and hashes MUST match referenced immutable objects;
5. all integer increments MUST be checked for overflow;
6. the transition MUST be deterministic from the current state and record bytes alone.

Timestamps are informational and MUST NOT decide canonical ordering or transition validity.

## 3. ISSUE

An ISSUE record is valid only when:

- `title_id` is not already present in State;
- publisher issuance COSE is valid under accepted issuer trust policy for ledger admission;
- record `asset_id`, `title_id`, `package_id`, initial owner key set, transfer policy, and owner-envelope hash exactly match the publisher issuance;
- the issuance binds this `ledger_id`/genesis;
- the package content manifest and initial envelope commitments are valid.

Output:

```text
sequence = 0
owner = issue-record.owner-key-set
package_id = issue-record.package_id
owner_envelope_hash = issue-record.initial-owner-envelope-hash
producing_record_hash = record_hash
output_id = OutputID(record_hash, 0)
```

## 4. TRANSFER

Let `input = State[title_id]`.

A TRANSFER is valid only when:

- `input.output_id == transfer_body.input_output_id`;
- input `asset_id`, `title_id`, and `package_id` equal the body values;
- `input.transfer_policy.transferable == true`;
- seller signature validates under `input.owner-key-set.Ed25519`;
- recipient signature validates under the new recipient OwnerKeySet Ed25519 key;
- `new_owner_envelope_hash` commits to an E1 envelope whose context binds the same transfer ID, input output ID, title/package, and new owner key-set hash;
- the input has not already been consumed in canonical state.

State update:

```text
delete State[title_id]
State[title_id] = new output
```

New output:

```text
asset_id = input.asset_id
title_id = input.title_id
package_id = input.package_id
sequence = input.sequence + 1
owner_key_set = transfer_body.new_owner
owner_envelope_hash = transfer_body.new_owner_envelope_hash
transfer_policy = input.transfer_policy
producing_record_hash = record_hash
output_id = OutputID(record_hash, 0)
```

A signed proposal that has not been included in finalized canonical state is `TRANSFER_NOT_FINAL` and does not change title.

## 5. INHERIT

INHERIT uses the same state/output rules as TRANSFER except:

- `input.transfer_policy.inheritable` MUST be true;
- authority evidence is validated under the inheritance/recovery policy accepted for the title;
- heir acceptance validates under the new heir OwnerKeySet.

v0.2 does not define civil probate law. A deployment may require external legal evidence before admitting a technically valid INHERIT transition.

## 6. REISSUE

REISSUE is valid only when:

- referenced input is the current output;
- `input.transfer_policy.reissuable == true`;
- owner authorization validates under the current owner key;
- authorized reissuer signature validates;
- the new publisher issuance is authentic and preserves `asset_id` and `title_id` while binding the new `package_id`;
- new owner envelope hash is valid for the current owner and new package.

New output preserves owner key set and title policy but changes `package_id` and owner-envelope hash. Sequence increments by one.

REISSUE is the mechanism for optional new media/watermark/package bytes; ordinary TRANSFER does not require it.

## 7. REKEY / RECOVER

REKEY is valid only when:

- referenced input is current;
- new OwnerKeySet is fresh and proves possession through its acceptance signature;
- authority evidence satisfies the accepted old-owner/recovery profile;
- new owner envelope is bound to the existing `package_id`, `title_id`, input output, and new OwnerKeySet.

The output preserves `asset_id`, `title_id`, `package_id`, and transfer policy, increments sequence, and replaces owner key/envelope.

A recovery operation that only rotates the offline Recovery Root and package recovery envelope without changing the public OwnerKeySet is local package maintenance and does not require a ledger REKEY transition.

## 8. Current-state Merkle tree

To compute `state_root`:

1. collect all current `ownership-output` values;
2. sort them strictly by ascending 32-byte `title_id`;
3. reject duplicate title IDs;
4. encode each output with CDE;
5. compute the RFC 9162 SHA-256 Merkle Tree Hash over those encoded leaves.

`output_count` equals the leaf count.

A current-title inclusion proof therefore proves that a specific output is present in the finalized current state.

To establish `NOT_CURRENT_OWNER` for a historical claimant, a verifier normally obtains the finalized current output for the same `title_id` and proves that output's inclusion. v0.2 does not define a separate sparse-tree non-membership proof.

## 9. Record history tree

The records tree contains every accepted ledger record in canonical finalized order.

Leaves are exact `CDE(ledger-record)` bytes. The tree hash and audit path algorithm are RFC 9162 SHA-256 Merkle Tree Hash.

`records_count` is cumulative.

The issuance bundle carried with a package contains the ISSUE record and audit proof necessary to establish inclusion in a finalized historical checkpoint.

## 10. Checkpoint chain

`checkpoint_id = SHA-256(CDE(checkpoint-payload))`.

At height 0, `previous_checkpoint_id` is 32 zero bytes. At height `h > 0`, it MUST equal the previous finalized checkpoint ID.

A finalized checkpoint verifier checks:

- ledger ID;
- checkpoint link;
- records root/count consistency with supplied proof;
- state root/count for current-title proofs;
- validator-set hash;
- protocol epoch;
- unique validator signatures;
- threshold satisfaction;
- validator-set transition proof when applicable.

A wire verifier validates finalized evidence. It does not need to implement proposer election, networking, timeouts, or BFT voting transport.

## 11. Validator sets

Validators in a set are strictly sorted by `validator_id`; duplicate IDs or public keys are invalid.

For a Byzantine-safe production deployment claiming tolerance `f`, governance SHOULD require at least:

```text
n >= 3f + 1
threshold >= 2f + 1
```

For the current f=1 target this means 4 validators and threshold 3.

The threshold check alone MUST NOT be advertised as consensus safety; deployments must use a mature consensus/locking/finality protocol consistent with their claimed fault model.

## 12. Validator rotation

If checkpoint `h` commits `next_validator_set_hash`, the next validator set becomes eligible at `h+1` only when:

```text
SHA-256(CDE(next_validator_set)) == checkpoint[h].next_validator_set_hash
```

A historical proof crossing rotations includes an ordered `validator-proof-chain` of finalized old-set checkpoint + next-set objects.

Verifier algorithm:

```text
trusted_set = genesis.validator_set
for transition in chain:
    verify transition.checkpoint under trusted_set
    require threshold finality
    require H(CDE(transition.next_set)) == checkpoint.next_validator_set_hash
    trusted_set = transition.next_set
verify target checkpoint under trusted_set
```

An implementation may cache previously verified validator sets/checkpoints.

## 13. Genesis trust

A package binds `ledger_genesis_hash` in publisher issuance. The ledger proof bundle supplies `ledger-genesis` and the verifier requires:

```text
SHA-256(CDE(ledger-genesis)) == issuance.ledger_genesis_hash
```

This prevents replacing the intended ledger with an attacker-created ledger without also forging publisher issuance.

## 14. Ledger failure semantics

If current ledger infrastructure is unavailable:

- bundled historical issuance proof remains verifiable;
- existing legitimate playback remains available;
- device enrollment/recovery remains available from owner authority;
- current-title freshness may become `CURRENT_STATE_UNKNOWN`;
- new canonical TRANSFER/INHERIT/REISSUE/REKEY operations cannot finalize.

That degraded state is intentional and MUST NOT be converted into remote revocation of owned media.

## 15. Consensus implementation boundary

MUSICPKG v0.2 standardizes:

- deterministic record validity;
- current ownership state transition;
- state/history roots;
- checkpoint bytes/signatures;
- validator-set rotation evidence;
- proof verification.

It does NOT standardize:

- validator network transport;
- mempool/gossip;
- proposer selection;
- view-change packets;
- payment settlement;
- a cryptocurrency or VM.

A deployment profile may select a mature BFT implementation whose finalized checkpoints satisfy this wire contract.
