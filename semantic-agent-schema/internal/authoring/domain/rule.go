package domain

import "slices"

// Rule は、値オブジェクト rule。judgment がどの決まりに照らした結果かを表す。
// x-prompt の rules の1件なら、id と related_fields（インスタンスのルートからの絶対の JSON Pointer）を持つ。
// 機械的な検証の rule は、JSON Schema の absolute keyword location か、作法の検査の名前を id にする。
type Rule interface {
	ID() string
	RelatedFields() []Pointer
	rule()
}

type promptRule struct {
	id      string
	related []Pointer
}

func (r promptRule) ID() string               { return r.id }
func (r promptRule) RelatedFields() []Pointer { return slices.Clone(r.related) }
func (promptRule) rule()                      {}

// ConventionRule は、作法の検査（ドメインサービス バリデーター）の judgment の rule の id。
const ConventionRule = "convention"

// NewRule は、id と related_fields から rule を返す。id が空なら schema_violation の Failure を返す。
func NewRule(id string, relatedFields []Pointer) (Rule, error) {
	if id == "" {
		return nil, Fail(SchemaViolation, "", "rule の id が空")
	}
	return promptRule{id: id, related: slices.Clone(relatedFields)}, nil
}
