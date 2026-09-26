// Package core は層の内側である。外側の層へ依存しない。
package core

import "errors"

// ErrEmpty は名前が空であることを表す。
var ErrEmpty = errors.New("名前が空である")

// Layer は名前を1つ持つ層である。欄は私用にしてある。
type Layer struct {
	label string
}

// NewLayer は名前から層を組む。名前が空のときは ErrEmpty を返す。
func NewLayer(label string) (Layer, error) {
	if label == "" {
		return Layer{}, ErrEmpty
	}
	return Layer{label: label}, nil
}

// Label は層の名前を返す。
func (l Layer) Label() string { return l.label }

// Name はこの層の名前を返す。
func Name() string { return "core" }
