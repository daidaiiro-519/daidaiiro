package domain

import (
	"bytes"
	"encoding/json"
	"slices"
	"strconv"
)

// Previous は、前回の意味的な検証。ある JSON ドキュメントの、直前の意味的な検証（AI の判定を受け取った検証）が残した
// 内容 ・ Concrete のハッシュ ・ judgment を持つ。変更点の基準で、機械的な検証だけでは動かない。
type Previous struct {
	content      []byte
	concreteHash Hash
	judgments    []Judgment
}

// NewPrevious は、操作の記録から読んだ前回の検証を返す。Concrete のハッシュの形が違えば error を返す。
func NewPrevious(content []byte, concreteHash string, judgments []Judgment) (*Previous, error) {
	h, err := ParseHash(concreteHash)
	if err != nil {
		return nil, err
	}
	return &Previous{content: bytes.Clone(content), concreteHash: h, judgments: slices.Clone(judgments)}, nil
}

// ChangedLocations は、ドメインサービス「変更点を求める」。
// 2つの JSON ドキュメントを JSON Pointer ごとに比べ、値が違う instance location の集まりを返す。
// 片方にしか無い instance location も変更点に入り、子の値が変われば親の値も変わる。数は書いたままの文字列で比べる。
func ChangedLocations(before, after any) map[string]bool {
	left, right := map[string]string{}, map[string]string{}
	flatten(before, nil, left)
	flatten(after, nil, right)
	changed := map[string]bool{}
	for location, value := range left {
		if other, ok := right[location]; !ok || other != value {
			changed[location] = true
		}
	}
	for location := range right {
		if _, ok := left[location]; !ok {
			changed[location] = true
		}
	}
	return changed
}

// flatten は、JSON の値を木の全部の instance location ごとの JSON 文字列にして、into に入れる。変更点を求めるときの比べる元になる。
func flatten(value any, at []string, into map[string]string) {
	encoded, _ := json.Marshal(value) // ParseJSON が読んだ値（map ・ slice ・ string ・ json.Number ・ bool ・ nil）は、必ず JSON にできる
	into[PointerOf(at...).String()] = string(encoded)
	switch v := value.(type) {
	case map[string]any:
		for key, child := range v {
			flatten(child, append(append([]string(nil), at...), key), into)
		}
	case []any:
		for i, child := range v {
			flatten(child, append(append([]string(nil), at...), strconv.Itoa(i)), into)
		}
	}
}

// Inherited は、ドメインサービス「変更点を求める」の、前回の検証から引き継ぐ AI の judgment を返す。
// Concrete のハッシュが前回と違うとき、または前回の検証が無いときは、すべての instance location が対象になり、引き継がない。
// それ以外は、instance location も rule の related_fields も変更点に含まない AI の judgment を引き継ぐ。
func Inherited(previous *Previous, current any, concrete Concrete) []Judgment {
	if previous == nil || previous.concreteHash.String() != concrete.Hash().String() {
		return nil
	}
	before, err := ParseJSON(previous.content)
	if err != nil {
		return nil // 前回の内容は検証を通った JSON なので、読めないなら引き継ぐものは無い
	}
	changed := ChangedLocations(before, current)
	var inherited []Judgment
	for _, j := range previous.judgments {
		if j.By() != ByAI || changed[j.InstanceLocation().String()] {
			continue
		}
		rule, ok := concrete.Rule(j.Rule().ID())
		if !ok || touches(rule, changed) {
			continue
		}
		inherited = append(inherited, j)
	}
	return inherited
}

// touches は、rule の related_fields のどれかが変更点に含まれるかを返す。含まれる rule の AI の judgment は引き継がない。
func touches(rule Rule, changed map[string]bool) bool {
	for _, field := range rule.RelatedFields() {
		if changed[field.String()] {
			return true
		}
	}
	return false
}

// Changed は、ドメインサービス「変更点を求める」。前回の検証が無いとき、または Concrete のハッシュが前回と違うときは
// すべての instance location が対象なので nil を返す。それ以外は、前回の検証の JSON ドキュメントといまの値を比べた変更点を返す。
func Changed(previous *Previous, current any, concrete Concrete) map[string]bool {
	if previous == nil || previous.concreteHash.String() != concrete.Hash().String() {
		return nil
	}
	before, err := ParseJSON(previous.content)
	if err != nil {
		return nil // 前回の内容は検証を通った JSON なので、読めないならすべてを対象にする
	}
	return ChangedLocations(before, current)
}

// Violated は、前回の検証が、いまの内容（current）の検証なら、その violated の judgment を返す。内容が違えば nil。
func (p *Previous) Violated(current Hash) []Judgment {
	if p == nil || HashOf(p.content).String() != current.String() {
		return nil
	}
	var violated []Judgment
	for _, j := range p.judgments {
		if j.Verdict() == Violated {
			violated = append(violated, j)
		}
	}
	return violated
}
