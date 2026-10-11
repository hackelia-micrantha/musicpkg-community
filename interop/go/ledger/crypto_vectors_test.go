// SPDX-License-Identifier: MPL-2.0

package ledger

import (
	"bytes"
	"crypto/aes"
	"crypto/cipher"
	"crypto/ecdh"
	"crypto/ed25519"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/binary"
	"testing"
)

// HKDF, CBOR concatenation, and Merkle logic are independently implemented here
// rather than calling Rust/reference implementation internals.
func hkdf32(ikm, salt, info []byte) []byte {
	mac := hmac.New(sha256.New, salt)
	mac.Write(ikm)
	prk := mac.Sum(nil)
	mac = hmac.New(sha256.New, prk)
	mac.Write(info)
	mac.Write([]byte{1})
	return mac.Sum(nil)
}
func cborParts(prefix byte, parts ...[]byte) []byte {
	b := []byte{prefix}
	for _, p := range parts {
		b = append(b, p...)
	}
	return b
}
func aesSeal(key, nonce, plain, aad []byte) ([]byte, error) {
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}
	return gcm.Seal(nil, nonce, plain, aad), nil
}
func merkleHash(leaves [][]byte) [32]byte {
	if len(leaves) == 0 {
		return sha256.Sum256(nil)
	}
	if len(leaves) == 1 {
		return sha256.Sum256(append([]byte{0}, leaves[0]...))
	}
	split := 1
	for split*2 < len(leaves) {
		split *= 2
	}
	left, right := merkleHash(leaves[:split]), merkleHash(leaves[split:])
	data := append(append([]byte{1}, left[:]...), right[:]...)
	return sha256.Sum256(data)
}
func TestCryptoTrackRecord(t *testing.T) {
	v := fixture(t, "crypto.json")
	pkg := hexField(t, v, "track_record", "package_id")
	track := hexField(t, v, "track_record", "track_id")
	salt := sha256.Sum256(append([]byte("MUSICPKG track salt v1\x00"), pkg...))
	expectBytes(t, "track salt", salt[:], hexField(t, v, "track_record", "hkdf_salt"))
	info := cborParts(0x83, cborText("MUSICPKG track key v1"), cborBytes(pkg), cborBytes(track))
	expectBytes(t, "track info", info, hexField(t, v, "track_record", "hkdf_info_cbor"))
	key := hkdf32(hexField(t, v, "track_record", "k_album"), salt[:], info)
	expectBytes(t, "track key", key, hexField(t, v, "track_record", "k_track"))
	nonce := make([]byte, 12)
	binary.BigEndian.PutUint64(nonce[4:], 0)
	expectBytes(t, "media nonce", nonce, hexField(t, v, "track_record", "nonce"))
	plain := hexField(t, v, "track_record", "plaintext")
	aad := cborParts(0x86, cborText("MUSICPKG media record v1"), cborBytes(pkg), cborBytes(track), cborMajor(0, 0), cborMajor(0, len(plain)), cborBytes(hexField(t, v, "track_record", "media_hash")))
	expectBytes(t, "record AAD", aad, hexField(t, v, "track_record", "aad_cbor"))
	ciphertext, err := aesSeal(key, nonce, plain, aad)
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "record ciphertext", ciphertext, hexField(t, v, "track_record", "ciphertext_and_tag"))
	block, _ := aes.NewCipher(key)
	gcm, _ := cipher.NewGCM(block)
	badAAD := append([]byte(nil), aad...)
	badAAD[len(badAAD)-1] ^= 1
	if _, err := gcm.Open(nil, nonce, ciphertext, badAAD); err == nil {
		t.Fatal("tampered AAD accepted")
	}
}
func TestCryptoOwnerO1(t *testing.T) {
	v := fixture(t, "crypto.json")
	edSeed := hexField(t, v, "owner_o1", "ed25519_seed")
	edPub := ed25519.NewKeyFromSeed(edSeed).Public().(ed25519.PublicKey)
	expectBytes(t, "Ed25519 public", edPub, hexField(t, v, "owner_o1", "ed25519_public"))
	xKey, err := ecdh.X25519().NewPrivateKey(hexField(t, v, "owner_o1", "x25519_private"))
	if err != nil {
		t.Fatal(err)
	}
	xPub := xKey.PublicKey().Bytes()
	expectBytes(t, "X25519 public", xPub, hexField(t, v, "owner_o1", "x25519_public"))
	keys := cborParts(0xa3, []byte{1, 1, 2}, cborBytes(edPub), []byte{3}, cborBytes(xPub))
	expectBytes(t, "owner key-set CDE", keys, hexField(t, v, "owner_o1", "owner_key_set_cbor"))
	h := sha256.Sum256(keys)
	expectBytes(t, "owner key-set hash", h[:], hexField(t, v, "owner_o1", "owner_key_set_hash"))
}
func TestCryptoRecoveryEnvelope(t *testing.T) {
	v := fixture(t, "crypto.json")
	pkg := hexField(t, v, "track_record", "package_id")
	title := hexField(t, fixture(t, "ledger-state.json"), "issue", "title_id")
	ownerHash := hexField(t, v, "owner_o1", "owner_key_set_hash")
	info := cborParts(0x83, cborText("MUSICPKG recovery key v1"), cborBytes(title), cborBytes(ownerHash))
	key := hkdf32(hexField(t, v, "recovery_envelope", "rr"), pkg, info)
	expectBytes(t, "recovery key", key, hexField(t, v, "recovery_envelope", "k_recovery"))
	aad := cborParts(0x84, cborText("MUSICPKG recovery-envelope v1"), cborBytes(pkg), cborBytes(title), cborBytes(ownerHash))
	expectBytes(t, "recovery AAD", aad, hexField(t, v, "recovery_envelope", "aad_cbor"))
	nonce := hexField(t, v, "recovery_envelope", "nonce")
	plain := hexField(t, v, "recovery_envelope", "owner_private_material")
	cipherText, err := aesSeal(key, nonce, plain, aad)
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "recovery ciphertext", cipherText, hexField(t, v, "recovery_envelope", "ciphertext"))
}
func TestCryptoPublisherAndOwnershipProof(t *testing.T) {
	v := fixture(t, "crypto.json")
	challenge := hexField(t, v, "ownership_proof", "challenge_cbor")
	proofInput := cborParts(0x82, cborText("MUSICPKG ownership-proof v1"), challenge)
	expectBytes(t, "proof signing input", proofInput, hexField(t, v, "ownership_proof", "signing_input_cbor"))
	if !VerifyVectorSignature(hexField(t, v, "owner_o1", "ed25519_public"), proofInput, hexField(t, v, "ownership_proof", "signature")) {
		t.Fatal("ownership signature invalid")
	}
	pub := hexField(t, v, "publisher_cose_sign1", "issuer_public")
	issuerID := sha256.Sum256(append([]byte("MUSICPKG issuer v1\x00"), pub...))
	expectBytes(t, "issuer ID", issuerID[:], hexField(t, v, "publisher_cose_sign1", "issuer_id"))
	kid := sha256.Sum256(pub)
	expectBytes(t, "issuer kid", kid[:16], hexField(t, v, "publisher_cose_sign1", "kid"))
	sigInput := cborParts(0x84, cborText("Signature1"), cborBytes(hexField(t, v, "publisher_cose_sign1", "protected_cbor")), []byte{0x40}, cborBytes(hexField(t, v, "publisher_cose_sign1", "payload_cbor")))
	expectBytes(t, "COSE signing input", sigInput, hexField(t, v, "publisher_cose_sign1", "sig_structure_cbor"))
	if !VerifyVectorSignature(pub, sigInput, hexField(t, v, "publisher_cose_sign1", "signature")) {
		t.Fatal("publisher signature invalid")
	}
	if VerifyVectorSignature(pub, proofInput, hexField(t, v, "publisher_cose_sign1", "signature")) {
		t.Fatal("publisher signature accepted wrong domain")
	}
}
func TestCryptoMerkleThreeLeaves(t *testing.T) {
	v := fixture(t, "crypto.json")
	obj := v["merkle_three_leaves"].(map[string]any)
	raw := obj["leaf_data_cbor"].([]any)
	leafHashes := obj["leaf_hashes"].([]any)
	leaves := make([][]byte, len(raw))
	for i, value := range raw {
		leaves[i] = mustHex(t, value.(string))
		hash := sha256.Sum256(append([]byte{0}, leaves[i]...))
		expectBytes(t, "leaf hash", hash[:], mustHex(t, leafHashes[i].(string)))
	}
	root := merkleHash(leaves)
	expectBytes(t, "3-leaf root", root[:], hexField(t, v, "merkle_three_leaves", "root"))
	changed := append([][]byte(nil), leaves...)
	changed[2] = append([]byte(nil), leaves[2]...)
	changed[2][len(changed[2])-1] ^= 1
	bad := merkleHash(changed)
	if bytes.Equal(root[:], bad[:]) {
		t.Fatal("mutated Merkle root not detected")
	}
}
func mustHex(t *testing.T, value string) []byte {
	t.Helper()
	return hexField(t, map[string]any{"value": value}, "value")
}
