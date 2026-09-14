# MUSICPKG Product Contract

## North star

MUSICPKG is a digital music ownership format.

The intended consumer experience is simple:

> Buy a music file once. Own it. Play it on your devices. Back it up. Prove it is yours. Transfer ownership legitimately if the title permits it.

The intended publisher experience is equally simple:

> Sell a legitimate copy once, receive the purchase payment, retain cryptographic proof of issuance and current canonical ownership, and make ordinary file copying materially less useful without remaining in the playback path.

MUSICPKG is not intended to be subscription DRM, rental, token-gated streaming, or a remotely revocable license system.

## Product model

The product consists of four core concepts:

```text
1. Media
   encrypted purchased music

2. Ownership
   cryptographic title controlled by one owner

3. Devices
   owner-authorized playback devices

4. Registry
   canonical issuance/transfer history when transferable title is supported
```

Supporting technologies such as C2PA, DDEX metadata, trusted hardware, watermarking, and BFT consensus exist to support those four concepts. They are not the product themselves.

## Purchase flow

A purchase should behave conceptually as follows:

```text
Publisher
    |
    | one-time sale
    v
signed MUSICPKG artifact
    +
initial ownership authority
    |
    v
Consumer
```

The publisher receives the sale payment at issuance. Continued playback does not require further publisher participation.

After purchase the consumer may:

- play the music offline;
- authorize their own phone, computer, stereo, or other conforming device;
- make arbitrary encrypted backups;
- migrate to replacement devices;
- preserve the purchase if the retailer or publisher disappears;
- independently prove legitimate ownership;
- transfer canonical title to another person when transfer is supported.

## Core ownership invariant

The system is built around this distinction:

```text
COPYING != OWNING
```

More precisely:

```text
copy .musicpkg
    !=
obtain playback authority

copy ciphertext + metadata
    !=
obtain playback authority

possess valid owner authority
    =
ability to authorize playback devices

valid ownership transfer
    =
new canonical owner
```

The protected artifact may be backed up or copied freely because artifact possession alone does not establish ownership authority.

## Consumer contract

A legitimate owner should be able to rely on all of the following properties.

### Permanent use

A valid purchase does not expire because a retailer, publisher, account service, license server, or ledger service disappears.

### Offline playback

Ordinary playback does not require a network connection or periodic license renewal.

### Owner-controlled devices

The owner, rather than the publisher or retailer, authorizes ordinary playback devices after purchase.

Routine device credentials should be non-exportable where practical, but hardware-bound credentials are operational grants rather than the permanent ownership root.

### Backup and migration

The owner may create arbitrary encrypted backups and recover the purchase onto replacement hardware using owner-controlled recovery material.

### Verifiable ownership

The owner can prove control of a cryptographically valid ownership credential without requiring personal information to be embedded in the media, package, or shared ownership ledger.

### Legitimate transfer

When transfer is permitted, the current owner can transfer canonical title to a new owner. The registry records that the previous title state was consumed and that the new owner holds the current title.

### Privacy

The portable artifact, ownership chain, and watermark should use opaque identifiers and cryptographic commitments rather than names, email addresses, payment details, or reusable customer identities.

## Publisher contract

A publisher should be able to rely on the following properties.

### One-time sale

The normal commercial transaction is a sale. The publisher receives the purchase payment at issuance and does not need to remain an authorization intermediary for playback.

### Authentic issuance

A conforming implementation can verify that a package was legitimately issued and has not had its authenticated content or ownership evidence silently modified.

### Copy resistance at the artifact layer

Copying the purchased `.musicpkg` artifact by itself does not create another usable copy of the music.

This is the primary redistribution-resistance guarantee.

### Ownership verification

A publisher or other verifier can establish that:

- a package is genuine;
- its issuance descends from an authorized publisher;
- its ownership record is valid;
- a claimant controls the relevant ownership credential;
- the canonical title has or has not subsequently been transferred, when fresh ledger state is available.

The verifier need not learn the purchaser's civil identity merely to verify cryptographic ownership.

### Canonical transferable title

When resale or gifting is supported, the registry prevents two conflicting transfers of the same canonical ownership state from both becoming final, subject to the ledger's stated consensus assumptions.

### Forensic evidence

A commercial profile may embed privacy-preserving forensic fingerprint evidence into the audio. If such evidence survives redistribution, it may link a leaked copy to an authenticated issuance lineage.

This is evidence and deterrence, not a perfect copy-prevention guarantee.

## Device model

The owner controls a durable recovery/ownership authority and authorizes replaceable device credentials:

```text
              Owner authority
                   |
        +----------+----------+
        |          |          |
      phone      laptop     stereo
```

A desirable implementation makes routine device keys non-exportable through TPM, Secure Enclave, StrongBox, smart-card, or equivalent facilities.

The design SHOULD minimize the exportability of playback authority while preserving owner-controlled recovery, migration, and inheritance.

A consumer should not normally need to manage or expose a reusable raw decryption key.

## Transfer / re-ownership

Suppose Alice owns a transferable title and transfers it to Bob:

```text
Before:
    Title X -> Alice

Alice authorizes transfer
        |
        v
canonical ownership registry
        |
        v
After:
    old Alice state -> spent
    Title X -> Bob
```

Bob receives current ownership authority and appropriate access to the encrypted media.

Conforming software should subsequently report Bob as the current canonical owner.

### Transfer limitation

Canonical transfer cannot prove that Alice deleted a previously captured plaintext recording or every historical secret she may have legitimately possessed.

Therefore MUSICPKG guarantees unique canonical transferable title, not cryptographic forgetting by a previous owner.

## What MUSICPKG deliberately does not guarantee

MUSICPKG cannot guarantee that a determined legitimate listener cannot capture the final waveform.

Once legitimate playback reaches plaintext audio, an attacker controlling the endpoint may potentially record through:

- process instrumentation;
- memory/PCM capture;
- virtual audio devices;
- a hostile kernel;
- digital loopback;
- analog recording.

Hardware-backed credentials, hardened players, and protected playback facilities raise attack cost but do not eliminate this boundary.

Forensic watermarking exists to provide possible attribution after this boundary is crossed.

## Product boundary

The following technologies are supporting layers and MUST NOT replace the ownership contract:

- C2PA provides provenance, not ownership authority;
- DDEX provides music-domain identifiers/metadata, not playback authority;
- trusted hardware protects operational credentials, not permanent ownership;
- watermarking provides forensic evidence, not authorization;
- the ledger establishes canonical title, not permission to continue listening to an already valid owned copy;
- publisher services facilitate sale and issuance, not perpetual permission.

## Product success criteria

A MUSICPKG implementation succeeds when the following user story works:

1. Alice buys an album once.
2. Alice receives a portable encrypted file and ownership authority.
3. Alice authorizes her phone, laptop, and stereo.
4. Alice copies the encrypted file to backup storage without creating additional owners.
5. The retailer disappears and Alice continues playing the album.
6. Alice replaces a device and restores playback without publisher permission.
7. A third party can verify that Alice holds valid ownership without needing her payment history.
8. If the title supports transfer, Alice can give or sell canonical title to Bob.
9. Bob becomes the current canonical owner.
10. Simple copying of Alice's protected artifact does not create another usable owned copy.
11. Publisher verification remains possible without the publisher being a permanent playback landlord.

## Concise promise

For consumers:

> Permanent, recoverable, vendor-independent ownership of purchased music.

For publishers:

> Copying the purchased artifact does not equal copying usable music, while authentic issuance and canonical ownership remain independently verifiable.

For both:

> The publisher sells the music; the consumer owns the purchase.