# Security Policy

## Reporting vulnerabilities

Please report security vulnerabilities privately through GitHub Security Advisories for this repository rather than opening a public issue.

Include affected specification/profile versions, relevant package or fixture inputs, expected behavior, observed behavior, and the security impact you have identified.

## Security boundary

MUSICPKG provides cryptographic protection for encrypted artifacts, authenticated media records, ownership/key binding, recovery-envelope integrity, publisher/owner signatures, and finalized title-state evidence under the stated ledger assumptions.

It does not claim universal prevention of plaintext or waveform capture after legitimate playback reaches a compromised general-purpose endpoint, PCM path, DAC, or analog output. Hardware-backed credentials and secure playback facilities are cost-raising controls; forensic watermarking is probabilistic evidence rather than authorization.

A current-title service or publisher service becoming unavailable must not silently revoke ordinary playback of an otherwise valid owned package.
