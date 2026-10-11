// SPDX-License-Identifier: MPL-2.0

package ledger

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/ecdh"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/binary"
	"fmt"
	"testing"
)

// The HPKE routines here are test-only, independent RFC 9180 known-answer
// probes. They do not implement a public HPKE API or validate parsed packages.
func hpkeU16(n uint16) []byte {
	out := make([]byte, 2)
	binary.BigEndian.PutUint16(out, n)
	return out
}

func hpkeExtract(salt, ikm []byte) []byte {
	mac := hmac.New(sha256.New, salt)
	mac.Write(ikm)
	return mac.Sum(nil)
}

func hpkeExpand(prk, info []byte, size int) []byte {
	var result, previous []byte
	for counter := byte(1); len(result) < size; counter++ {
		mac := hmac.New(sha256.New, prk)
		mac.Write(previous)
		mac.Write(info)
		mac.Write([]byte{counter})
		previous = mac.Sum(nil)
		result = append(result, previous...)
	}
	return result[:size]
}

func hpkeLabeledExtract(suite, salt []byte, label string, ikm []byte) []byte {
	input := append([]byte("HPKE-v1"), suite...)
	input = append(input, []byte(label)...)
	input = append(input, ikm...)
	return hpkeExtract(salt, input)
}

func hpkeLabeledExpand(suite, prk []byte, label string, info []byte, size int) []byte {
	input := append(hpkeU16(uint16(size)), []byte("HPKE-v1")...)
	input = append(input, suite...)
	input = append(input, []byte(label)...)
	input = append(input, info...)
	return hpkeExpand(prk, input, size)
}

func hpkeProbe(kemID uint16, ephemeralPrivate, recipientPublic, info []byte) (enc, shared, key, nonce []byte, err error) {
	var curve ecdh.Curve
	switch kemID {
	case 0x0020:
		curve = ecdh.X25519()
	case 0x0010:
		curve = ecdh.P256()
	default:
		return nil, nil, nil, nil, fmt.Errorf("unsupported test KEM 0x%x", kemID)
	}
	skE, err := curve.NewPrivateKey(ephemeralPrivate)
	if err != nil {
		return nil, nil, nil, nil, err
	}
	pkR, err := curve.NewPublicKey(recipientPublic)
	if err != nil {
		return nil, nil, nil, nil, err
	}
	enc = skE.PublicKey().Bytes()
	dh, err := skE.ECDH(pkR)
	if err != nil {
		return nil, nil, nil, nil, err
	}
	kemSuite := append([]byte("KEM"), hpkeU16(kemID)...)
	eaePRK := hpkeLabeledExtract(kemSuite, nil, "eae_prk", dh)
	kemContext := append(append([]byte(nil), enc...), recipientPublic...)
	shared = hpkeLabeledExpand(kemSuite, eaePRK, "shared_secret", kemContext, 32)

	// Base mode 0; KDF HKDF-SHA256=0x0001; AEAD AES-256-GCM=0x0002.
	suite := append(append([]byte("HPKE"), hpkeU16(kemID)...), 0, 1, 0, 2)
	pskIDHash := hpkeLabeledExtract(suite, nil, "psk_id_hash", nil)
	infoHash := hpkeLabeledExtract(suite, nil, "info_hash", info)
	context := append(append([]byte{0}, pskIDHash...), infoHash...)
	secret := hpkeLabeledExtract(suite, shared, "secret", nil)
	key = hpkeLabeledExpand(suite, secret, "key", context, 32)
	nonce = hpkeLabeledExpand(suite, secret, "base_nonce", context, 12)
	return enc, shared, key, nonce, nil
}

func hpkeAEADOpen(key, nonce, ciphertext, aad []byte) ([]byte, error) {
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return nil, err
	}
	return gcm.Open(nil, nonce, ciphertext, aad)
}

func TestCryptoOwnerEnvelopeE1HPKE(t *testing.T) {
	v := fixture(t, "crypto.json")
	ephemeral := hexField(t, v, "owner_envelope_e1", "sender_ephemeral_private_vector_only")
	recipient := hexField(t, v, "owner_envelope_e1", "recipient_public")
	expectBytes(t, "E1 recipient matches O1 owner", recipient, hexField(t, v, "owner_o1", "x25519_public"))
	enc, shared, key, nonce, err := hpkeProbe(0x0020, ephemeral, recipient, []byte("MUSICPKG owner-envelope E1"))
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "E1 enc", enc, hexField(t, v, "owner_envelope_e1", "enc"))
	expectBytes(t, "E1 shared secret", shared, hexField(t, v, "owner_envelope_e1", "shared_secret"))
	expectBytes(t, "E1 HPKE key", key, hexField(t, v, "owner_envelope_e1", "hpke_key"))
	expectBytes(t, "E1 base nonce", nonce, hexField(t, v, "owner_envelope_e1", "hpke_base_nonce"))
	album := hexField(t, v, "track_record", "k_album")
	aad := hexField(t, v, "owner_envelope_e1", "context_cbor")
	ciphertext, err := aesSeal(key, nonce, album, aad)
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "E1 sealed album key", ciphertext, hexField(t, v, "owner_envelope_e1", "ciphertext"))
	opened, err := hpkeAEADOpen(key, nonce, ciphertext, aad)
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "E1 opened album key", opened, album)
	mutated := append([]byte(nil), aad...)
	mutated[len(mutated)-1] ^= 1
	if _, err := hpkeAEADOpen(key, nonce, ciphertext, mutated); err == nil {
		t.Fatal("E1 opened ciphertext with altered owner-envelope AAD")
	}
}

func TestCryptoDeviceGrantH1HPKE(t *testing.T) {
	v := fixture(t, "crypto.json")
	body := v["device_grant_h1"].(map[string]any)
	ephemeralScalar := byte(body["sender_ephemeral_private_scalar_vector_only"].(float64))
	recipientScalar := byte(body["recipient_private_scalar_vector_only"].(float64))
	ephemeral := make([]byte, 32)
	ephemeral[31] = ephemeralScalar
	private := make([]byte, 32)
	private[31] = recipientScalar
	skR, err := ecdh.P256().NewPrivateKey(private)
	if err != nil {
		t.Fatal(err)
	}
	recipient := hexField(t, v, "device_grant_h1", "recipient_public_uncompressed")
	expectBytes(t, "H1 recipient P-256 public", skR.PublicKey().Bytes(), recipient)
	enc, shared, key, nonce, err := hpkeProbe(0x0010, ephemeral, recipient, []byte("MUSICPKG device-grant H1"))
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "H1 enc", enc, hexField(t, v, "device_grant_h1", "enc"))
	expectBytes(t, "H1 shared secret", shared, hexField(t, v, "device_grant_h1", "shared_secret"))
	expectBytes(t, "H1 HPKE key", key, hexField(t, v, "device_grant_h1", "hpke_key"))
	expectBytes(t, "H1 base nonce", nonce, hexField(t, v, "device_grant_h1", "hpke_base_nonce"))
	album := hexField(t, v, "track_record", "k_album")
	aad := hexField(t, v, "device_grant_h1", "body_cbor")
	ciphertext, err := aesSeal(key, nonce, album, aad)
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "H1 sealed album key", ciphertext, hexField(t, v, "device_grant_h1", "ciphertext"))
	opened, err := hpkeAEADOpen(key, nonce, ciphertext, aad)
	if err != nil {
		t.Fatal(err)
	}
	expectBytes(t, "H1 opened album key", opened, album)
	grantInput := cborParts(0x84,
		cborText("MUSICPKG device-grant signature v1"),
		aad,
		cborBytes(enc),
		cborBytes(ciphertext),
	)
	expectBytes(t, "H1 owner signature input", grantInput, hexField(t, v, "device_grant_h1", "owner_signature_input_cbor"))
	ownerPub := hexField(t, v, "owner_o1", "ed25519_public")
	ownerSignature := hexField(t, v, "device_grant_h1", "owner_signature")
	if !VerifyVectorSignature(ownerPub, grantInput, ownerSignature) {
		t.Fatal("H1 owner device grant signature invalid")
	}
	badInput := append([]byte(nil), grantInput...)
	badInput[len(badInput)-1] ^= 1
	if VerifyVectorSignature(ownerPub, badInput, ownerSignature) {
		t.Fatal("H1 owner signature accepted ciphertext mutation")
	}
	badAAD := append([]byte(nil), aad...)
	badAAD[len(badAAD)-1] ^= 1
	if _, err := hpkeAEADOpen(key, nonce, ciphertext, badAAD); err == nil {
		t.Fatal("H1 opened ciphertext with altered grant body")
	}
}
