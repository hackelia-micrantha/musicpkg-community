// SPDX-License-Identifier: MPL-2.0

package ledger

import (
	"bytes"
	"crypto/ed25519"
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func fixture(t *testing.T, name string) map[string]any {
	t.Helper()
	data, err := os.ReadFile(filepath.Join("..", "..", "..", "test-vectors", "v0.2", name))
	if err != nil {
		t.Fatalf("read published fixture: %v", err)
	}
	var v map[string]any
	if err := json.Unmarshal(data, &v); err != nil {
		t.Fatalf("parse fixture: %v", err)
	}
	return v
}

func hexField(t *testing.T, v map[string]any, path ...string) []byte {
	t.Helper()
	var item any = v
	for _, key := range path {
		obj, ok := item.(map[string]any)
		if !ok {
			t.Fatalf("not an object at %q", key)
		}
		item, ok = obj[key]
		if !ok {
			t.Fatalf("missing fixture field %q", key)
		}
	}
	s, ok := item.(string)
	if !ok {
		t.Fatalf("hex fixture field is not a string: %v", path)
	}
	b, err := hex.DecodeString(s)
	if err != nil {
		t.Fatalf("invalid hex at %v: %v", path, err)
	}
	return b
}

func expectBytes(t *testing.T, label string, actual, expected []byte) {
	t.Helper()
	if !bytes.Equal(actual, expected) {
		t.Errorf("%s mismatch: got %x want %x", label, actual, expected)
	}
}

func fixed32(t *testing.T, b []byte) [32]byte {
	t.Helper()
	if len(b) != 32 {
		t.Fatalf("expected 32-byte hash, got %d", len(b))
	}
	return [32]byte(b)
}

func TestPublicLedgerRecordHashesAndOutputIDs(t *testing.T) {
	v := fixture(t, "ledger-state.json")
	for _, entry := range []struct{ name, record string }{
		{"issue", "issue_record_cbor"},
		{"transfer", "transfer_record_cbor"},
	} {
		t.Run(entry.name, func(t *testing.T) {
			record := hexField(t, v, entry.name, entry.record)
			hash := RecordHash(record)
			expectBytes(t, "SHA-256 record hash", hash[:], hexField(t, v, entry.name, "record_hash"))
			expectBytes(t, "CDE output ID domain", OutputIDDomainCDE(hash), hexField(t, v, entry.name, "output_id_domain_cbor"))
			id := OutputID(hash)
			expectBytes(t, "output ID", id[:], hexField(t, v, entry.name, "output_id"))
		})
	}
}

func TestPublicTransferSigningInputsAndSignatures(t *testing.T) {
	v := fixture(t, "ledger-state.json")
	crypto := fixture(t, "crypto.json")
	body := hexField(t, v, "transfer", "transfer_body_cbor")
	seller := TransferSigningInput(SellerDomain, body)
	recipient := TransferSigningInput(RecipientDomain, body)
	expectBytes(t, "seller domain-separated CDE", seller, hexField(t, v, "transfer", "seller_signing_input_cbor"))
	expectBytes(t, "recipient domain-separated CDE", recipient, hexField(t, v, "transfer", "recipient_signing_input_cbor"))
	sellerKey := hexField(t, crypto, "owner_o1", "ed25519_public")
	recipientKey := hexField(t, v, "transfer", "new_owner_o1", "ed25519_public")
	sellerSig := hexField(t, v, "transfer", "seller_signature")
	recipientSig := hexField(t, v, "transfer", "recipient_signature")
	if !VerifyVectorSignature(sellerKey, seller, sellerSig) {
		t.Fatal("published seller signature failed")
	}
	if !VerifyVectorSignature(recipientKey, recipient, recipientSig) {
		t.Fatal("published recipient signature failed")
	}
	if VerifyVectorSignature(sellerKey, recipient, sellerSig) {
		t.Error("seller signature accepted under recipient domain")
	}
	if VerifyVectorSignature(recipientKey, seller, recipientSig) {
		t.Error("recipient signature accepted under seller domain")
	}
	mutated := append([]byte(nil), body...)
	mutated[len(mutated)-1] ^= 1
	if VerifyVectorSignature(sellerKey, TransferSigningInput(SellerDomain, mutated), sellerSig) {
		t.Error("seller signature accepted mutated transfer body")
	}
	badSig := append([]byte(nil), recipientSig...)
	badSig[0] ^= 1
	if VerifyVectorSignature(recipientKey, recipient, badSig) {
		t.Error("mutated recipient signature accepted")
	}
	if VerifyVectorSignature(make([]byte, ed25519.PublicKeySize-1), recipient, recipientSig) {
		t.Error("short key accepted")
	}
}

func TestPublishedOutputIDKnownAnswer(t *testing.T) {
	record, _ := hex.DecodeString("fa47cdd64887fcb1087ca1638bf7be6977eb7b85b205cd7c06ce8e42946571d4")
	expected, _ := hex.DecodeString("62a4534874897bfb1e105c06326f66d5f23d067a177b8dee435cc423f68aa21d")
	id := OutputID(fixed32(t, record))
	expectBytes(t, "ISSUE known-answer output ID", id[:], expected)
}
