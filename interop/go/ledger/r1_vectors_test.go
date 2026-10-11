// SPDX-License-Identifier: MPL-2.0

package ledger

import (
	"bytes"
	"crypto/aes"
	"crypto/cipher"
	"testing"

	"golang.org/x/crypto/argon2"
)

// R1 is a public known-answer probe only. It does not accept arbitrary
// untrusted KDF cost fields or implement a user-facing recovery verifier.
func TestCryptoR1Argon2idRecoveryBundle(t *testing.T) {
	v := fixture(t, "crypto.json")
	password := hexField(t, v, "r1_bundle", "passphrase_utf8_nfc")
	salt := hexField(t, v, "r1_bundle", "salt")
	nonce := hexField(t, v, "r1_bundle", "nonce")
	rr := hexField(t, v, "r1_bundle", "rr")
	if len(salt) != 16 || len(nonce) != 12 || len(rr) != 32 {
		t.Fatal("invalid public R1 salt, nonce or root length")
	}

	// RFC 9106 Argon2id second recommended profile, version 0x13:
	// 64 MiB, three passes, four lanes, 32-byte result.
	if argon2.Version != 0x13 {
		t.Fatal("unexpected Argon2 version")
	}
	key := argon2.IDKey(password, salt, 3, 65536, 4, 32)
	expectBytes(t, "R1 Argon2id key", key, hexField(t, v, "r1_bundle", "argon2id_output"))

	// Reconstruct the nine-field AAD instead of trusting the fixture's CBOR.
	// The public bytes are already NFC-normalized UTF-8; this is not a text UI.
	aad := cborParts(0x89,
		cborText("MUSICPKG recovery-bundle R1"),
		cborMajor(0, 1),  // version
		cborMajor(0, 1),  // profile
		cborMajor(0, 2),  // Argon2 type=id
		cborMajor(0, 19), // Argon2 version
		cborBytes(salt),
		cborMajor(0, 65536),
		cborMajor(0, 3),
		cborMajor(0, 4),
	)
	expectBytes(t, "R1 recovery AAD", aad, hexField(t, v, "r1_bundle", "aad_cbor"))

	ciphertext, err := aesSeal(key, nonce, rr, aad)
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "R1 encrypted recovery root", ciphertext, hexField(t, v, "r1_bundle", "ciphertext"))
	block, err := aes.NewCipher(key)
	if err != nil {
		t.Fatal(err)
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		t.Fatal(err)
	}
	opened, err := gcm.Open(nil, nonce, ciphertext, aad)
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "R1 opened recovery root", opened, rr)

	badAAD := append([]byte(nil), aad...)
	badAAD[len(badAAD)-1] ^= 1
	if _, err := gcm.Open(nil, nonce, ciphertext, badAAD); err == nil {
		t.Fatal("R1 opened with altered KDF-profile AAD")
	}
	damaged := append([]byte(nil), ciphertext...)
	damaged[len(damaged)-1] ^= 1
	if _, err := gcm.Open(nil, nonce, damaged, aad); err == nil {
		t.Fatal("R1 opened with altered authentication tag")
	}
	wrongPassword := append([]byte(nil), password...)
	wrongPassword[0] ^= 1
	wrongKey := argon2.IDKey(wrongPassword, salt, 3, 65536, 4, 32)
	if bytes.Equal(key, wrongKey) {
		t.Fatal("different passphrase produced identical R1 key")
	}
	wrongBlock, err := aes.NewCipher(wrongKey)
	if err != nil {
		t.Fatal(err)
	}
	wrongGCM, err := cipher.NewGCM(wrongBlock)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := wrongGCM.Open(nil, nonce, ciphertext, aad); err == nil {
		t.Fatal("R1 opened with wrong passphrase")
	}
}
