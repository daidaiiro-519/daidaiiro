package core

import (
	"errors"
	"testing"
)

func TestNewLayerRejectsEmpty(t *testing.T) {
	if _, err := NewLayer(""); !errors.Is(err, ErrEmpty) {
		t.Fatalf("空の名前を受け取った: %v", err)
	}
}

func TestNewLayerKeepsLabel(t *testing.T) {
	l, err := NewLayer("core")
	if err != nil {
		t.Fatalf("組めない: %v", err)
	}
	if l.Label() != "core" {
		t.Fatalf("名前が違う: %q", l.Label())
	}
}
