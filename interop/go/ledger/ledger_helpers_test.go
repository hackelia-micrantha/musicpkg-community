// SPDX-License-Identifier: MPL-2.0

// Package ledger provides implementation-independent primitives for comparing
// MUSICPKG v0.2 public vectors. It is not a parser or ownership verifier.
package ledger

import (
	"crypto/ed25519"
	"crypto/sha256"
)

func cborMajor(major byte, n int) []byte {
	switch {
	case n < 24:
		return []byte{(major << 5) | byte(n)}
	case n <= 0xff:
		return []byte{(major << 5) | 24, byte(n)}
	case n <= 0xffff:
		return []byte{(major << 5) | 25, byte(n >> 8), byte(n)}
	default:
		return []byte{(major << 5) | 26, byte(n >> 24), byte(n >> 16), byte(n >> 8), byte(n)}
	}
}

func cborText(s string) []byte {
	return append(cborMajor(3, len(s)), []byte(s)...)
}

func cborBytes(b []byte) []byte {
	return append(cborMajor(2, len(b)), b...)
}

// RecordHash hashes exact CDE record bytes. The caller must validate CDE
// and the ledger-record schema before treating its result as trusted.
func RecordHash(recordCDE []byte) [32]byte {
	return sha256.Sum256(recordCDE)
}

// OutputIDDomainCDE builds ["MUSICPKG output-id v1", record_hash, 0].
func OutputIDDomainCDE(recordHash [32]byte) []byte {
	result := []byte{0x83}
	result = append(result, cborText("MUSICPKG output-id v1")...)
	result = append(result, cborBytes(recordHash[:])...)
	return append(result, 0x00)
}

func OutputID(recordHash [32]byte) [32]byte {
	return sha256.Sum256(OutputIDDomainCDE(recordHash))
}

// TransferSigningInput encodes [domain, transfer-body]. The body must already
// be validated as exactly one canonical CBOR item by an upstream parser.
func TransferSigningInput(domain string, transferBodyCDE []byte) []byte {
	result := []byte{0x82}
	result = append(result, cborText(domain)...)
	return append(result, transferBodyCDE...)
}

const (
	SellerDomain    = "MUSICPKG transfer seller v1"
	RecipientDomain = "MUSICPKG transfer recipient v1"
)

// VerifyVectorSignature is a test-vector probe, not production authorization.
// Strict weak-key/COSE/admission policy must be enforced separately.
func VerifyVectorSignature(publicKey, message, signature []byte) bool {
	return len(publicKey) == ed25519.PublicKeySize &&
		len(signature) == ed25519.SignatureSize &&
		ed25519.Verify(publicKey, message, signature)
}
