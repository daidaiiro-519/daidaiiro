package domain

import (
	"slices"
	"strconv"
	"strings"
)

// Evaluation は、機械的な検証（JSON Schema）で、サブスキーマが、インスタンスのどの位置に当たったかの組。
// KeywordLocation は検証に使ったスキーマ（インスタンスなら Concrete、Concrete なら Abstract）の中のサブスキーマの位置、
// InstanceLocation はそのサブスキーマが当たったインスタンスの位置。rule は、それを置いたサブスキーマが当たるインスタンスの位置に
// 当たる。組を決めるのは JSON Schema の実装で、ポート Validator を通して受け取る。ドメインモデルはスキーマの構造をたどらない。
type Evaluation struct {
	KeywordLocation  Pointer
	InstanceLocation Pointer
}

// sortedEvaluations は、組を instance location の順（親が先、配列の要素は番号の順）、同じ位置なら keyword location の順に並べ、
// 重なる組を除いて返す。JSON Schema の実装が組を集める順に頼らず、同じ入力なら同じ prompt にするため。
func sortedEvaluations(evaluations []Evaluation) []Evaluation {
	sorted := slices.Clone(evaluations)
	slices.SortFunc(sorted, func(a, b Evaluation) int {
		if c := compareLocations(a.InstanceLocation, b.InstanceLocation); c != 0 {
			return c
		}
		return strings.Compare(a.KeywordLocation.String(), b.KeywordLocation.String())
	})
	return slices.CompactFunc(sorted, func(a, b Evaluation) bool {
		return a.InstanceLocation.String() == b.InstanceLocation.String() && a.KeywordLocation.String() == b.KeywordLocation.String()
	})
}

// compareLocations は、2つの JSON Pointer を参照トークンごとに比べる。どちらも数のトークンは数として比べ、短い方（親）を先にする。
func compareLocations(a, b Pointer) int {
	left, right := tokensOf(a), tokensOf(b)
	for i := 0; i < len(left) && i < len(right); i++ {
		m, errM := strconv.Atoi(left[i])
		n, errN := strconv.Atoi(right[i])
		var c int
		if errM == nil && errN == nil {
			c = m - n
		} else {
			c = strings.Compare(left[i], right[i])
		}
		if c != 0 {
			return c
		}
	}
	return len(left) - len(right)
}
