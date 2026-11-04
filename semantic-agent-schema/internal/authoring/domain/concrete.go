package domain

import (
	"slices"
	"strconv"
)

// Concrete は、値オブジェクト Concrete。スキーマとして読むときの Concrete（Concrete に従う JSON ドキュメントなら Abstract）を表す。
// $id ・ 内容のハッシュ ・ サブスキーマごとの rule を持つ。
type Concrete interface {
	ID() string
	// Path は、schemas の中のファイルの path。公開された言葉の Abstract なら nil。
	Path() Path
	Content() []byte
	Hash() Hash
	// Rule は、id の rule を返す。無ければ false を返す。
	Rule(id string) (Rule, bool)
	concrete()
}

type schema struct {
	id      string
	path    Path
	content []byte
	hash    Hash
	rules   map[string]Rule
}

func (s schema) ID() string      { return s.id }
func (s schema) Path() Path      { return s.path }
func (s schema) Content() []byte { return s.content }
func (s schema) Hash() Hash      { return s.hash }
func (s schema) Rule(id string) (Rule, bool) {
	r, ok := s.rules[id]
	return r, ok
}
func (schema) concrete() {}

// NewConcrete は、Concrete のバイト列から Concrete を返す。path は schemas の中のファイルの path で、
// 公開された言葉の Abstract なら nil を渡す。JSON として読めなければ invalid_json、
// $id の無いオブジェクトなら schema_violation の Failure を返す。
func NewConcrete(path Path, content []byte) (Concrete, error) {
	value, err := ParseJSON(content)
	if err != nil {
		return nil, err
	}
	root, ok := value.(map[string]any)
	id, _ := root["$id"].(string) // root が nil でも、型の確かめは false になるだけ
	if !ok || id == "" {
		return nil, Fail(SchemaViolation, "", "Concrete に $id が無い")
	}
	rules := map[string]Rule{}
	walkSubschemas(root, nil, func(subschema map[string]any, _ []string) {
		for _, item := range promptRules(subschema) {
			rule, _ := item.(map[string]any) // 形は Abstract が決める。違う形の rule は飛ばす
			ruleID, _ := rule["id"].(string)
			if _, seen := rules[ruleID]; seen || ruleID == "" {
				continue
			}
			var related []Pointer
			fields, _ := rule["related_fields"].([]any) // 無ければ related_fields は空
			for _, f := range fields {
				text, _ := f.(string) // 形は Abstract が決める。文字列でなければ NewPointer が拒む
				if p, err := NewPointer(text); err == nil {
					related = append(related, p)
				}
			}
			r, err := NewRule(ruleID, related)
			if err == nil {
				rules[ruleID] = r
			}
		}
	})
	return schema{id: id, path: path, content: content, hash: HashOf(content), rules: rules}, nil
}

// promptRules は、サブスキーマの x-prompt の rules を返す。無ければ nil を返す。
func promptRules(subschema map[string]any) []any {
	prompt, ok := subschema["x-prompt"].(map[string]any)
	if !ok {
		return nil
	}
	rules, _ := prompt["rules"].([]any) // 配列でなければ rule は無い
	return rules
}

// サブスキーマを値に持つ JSON Schema 2020-12 のキーワード。
var (
	schemaMapKeywords    = []string{"$defs", "properties", "patternProperties", "dependentSchemas"}
	schemaArrayKeywords  = []string{"allOf", "anyOf", "oneOf", "prefixItems"}
	schemaSingleKeywords = []string{
		"additionalProperties", "items", "contains", "not", "if", "then", "else",
		"propertyNames", "unevaluatedItems", "unevaluatedProperties", "contentSchema",
	}
)

// walkSubschemas は、ルートスキーマから、サブスキーマを値に持つキーワードだけをたどり、各サブスキーマで visit を呼ぶ。
// const ・ default ・ examples の中の値は、サブスキーマでないのでたどらない。
func walkSubschemas(subschema map[string]any, at []string, visit func(map[string]any, []string)) {
	visit(subschema, at)
	for _, keyword := range schemaMapKeywords {
		children, ok := subschema[keyword].(map[string]any)
		if !ok {
			continue
		}
		names := make([]string, 0, len(children))
		for name := range children {
			names = append(names, name)
		}
		slices.Sort(names)
		for _, name := range names {
			if child, ok := children[name].(map[string]any); ok {
				walkSubschemas(child, append(slices.Clone(at), keyword, name), visit)
			}
		}
	}
	for _, keyword := range schemaArrayKeywords {
		children, ok := subschema[keyword].([]any)
		if !ok {
			continue
		}
		for i, item := range children {
			if child, ok := item.(map[string]any); ok {
				walkSubschemas(child, append(slices.Clone(at), keyword, strconv.Itoa(i)), visit)
			}
		}
	}
	for _, keyword := range schemaSingleKeywords {
		if child, ok := subschema[keyword].(map[string]any); ok {
			walkSubschemas(child, append(slices.Clone(at), keyword), visit)
		}
	}
}
