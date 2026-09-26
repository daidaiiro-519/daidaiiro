// Package adapter は層の外側である。内側の層だけを呼ぶ。
package adapter

import (
	"fmt"

	"example.com/layered/internal/core"
)

// LabelOf は内側の層を組んで、その名前を返す。
func LabelOf(input string) (string, error) {
	l, err := core.NewLayer(input)
	if err != nil {
		return "", fmt.Errorf("層を組めない: %w", err)
	}
	return l.Label(), nil
}
