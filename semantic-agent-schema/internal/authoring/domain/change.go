package domain

import (
	"encoding/json"
	"slices"
)

// op は、change の種類。
type op string

// change の種類。
const (
	Add     op = "add"
	Remove  op = "remove"
	Replace op = "replace"
)

// Change は、値オブジェクト change。書く操作の中の、1つの instance location の前後の値を持つ。
// Before と After は JSON の文字列で、無いときは空。
type Change interface {
	InstanceLocation() Pointer
	Op() op
	Before() string
	After() string
	change()
}

type fieldChange struct {
	location      Pointer
	op            op
	before, after string
}

func (c fieldChange) InstanceLocation() Pointer { return c.location }
func (c fieldChange) Op() op                    { return c.op }
func (c fieldChange) Before() string            { return c.before }
func (c fieldChange) After() string             { return c.after }
func (fieldChange) change()                     {}

// changesOf は、集約 JSON ドキュメントの不変条件として、書くたびに instance location ごとの change を出す。
// 前の JSON ドキュメントが無ければ（作成）、すべての instance location が add になる。
func changesOf(before any, hasBefore bool, after any) []Change {
	left, right := map[string]string{}, map[string]string{}
	if hasBefore {
		flatten(before, nil, left)
	}
	flatten(after, nil, right)
	var changes []Change
	for _, location := range sortedKeys(left, right) {
		b, inBefore := left[location]
		a, inAfter := right[location]
		pointer := jsonPointer{location}
		switch {
		case inBefore && !inAfter:
			changes = append(changes, fieldChange{location: pointer, op: Remove, before: b})
		case !inBefore && inAfter:
			changes = append(changes, fieldChange{location: pointer, op: Add, after: a})
		case b != a:
			changes = append(changes, fieldChange{location: pointer, op: Replace, before: b, after: a})
		}
	}
	return changes
}

func sortedKeys(maps ...map[string]string) []string {
	seen := map[string]bool{}
	var keys []string
	for _, m := range maps {
		for k := range m {
			if !seen[k] {
				seen[k] = true
				keys = append(keys, k)
			}
		}
	}
	slices.Sort(keys)
	return keys
}

// jsonText は、値を JSON の文字列にする。
func jsonText(value any) string {
	encoded, _ := json.Marshal(value) // ParseJSON が読んだ値と、Concrete の default は、必ず JSON にできる
	return string(encoded)
}
