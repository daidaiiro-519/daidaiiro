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
	// View は、ルートスキーマの x-view（テンプレートの種類の名前）を返す。無ければ空。
	View() string
	// Subschemas は、properties と items でたどれるサブスキーマを、keyword location の順に返す。
	Subschemas() []Subschema
	// structure は、Concrete を読んだルートスキーマを返す。
	structure() map[string]any
}

// Subschema は、Concrete の1つのサブスキーマの、AI が読む部分。位置は、ルートスキーマからの keyword location で持つ。
type Subschema struct {
	KeywordLocation Pointer
	Title           string
	Description     string
	XPrompt         map[string]any // x-prompt。無ければ nil
}

type schema struct {
	id      string
	path    Path
	content []byte
	hash    Hash
	rules   map[string]Rule
	subs    []Subschema
	root    map[string]any
}

func (s schema) ID() string      { return s.id }
func (s schema) Path() Path      { return s.path }
func (s schema) Content() []byte { return s.content }
func (s schema) Hash() Hash      { return s.hash }
func (s schema) Rule(id string) (Rule, bool) {
	r, ok := s.rules[id]
	return r, ok
}
func (s schema) Subschemas() []Subschema   { return s.subs }
func (s schema) structure() map[string]any { return s.root }
func (s schema) View() string {
	view, _ := s.root["x-view"].(string) // 無ければ空
	return view
}

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
	var subs []Subschema
	walkStructure(root, nil, &subs)
	return schema{id: id, path: path, content: content, hash: HashOf(content), rules: rules, subs: subs, root: root}, nil
}

// walkStructure は、ドメインサービス「サブスキーマを引く」の、Concrete の側。ルートスキーマから JSON Schema の構造
// （properties と items）だけをたどり、keyword location ごとにサブスキーマを集める。
func walkStructure(subschema map[string]any, at []string, into *[]Subschema) {
	sub := Subschema{KeywordLocation: PointerOf(at...)}
	sub.Title, _ = subschema["title"].(string)              // 無ければ空
	sub.Description, _ = subschema["description"].(string)  // 無ければ空
	sub.XPrompt, _ = subschema["x-prompt"].(map[string]any) // 無ければ nil
	*into = append(*into, sub)
	if properties, ok := subschema["properties"].(map[string]any); ok {
		for _, name := range sortedNames(properties) {
			if child, ok := properties[name].(map[string]any); ok {
				walkStructure(child, append(slices.Clone(at), "properties", name), into)
			}
		}
	}
	if items, ok := subschema["items"].(map[string]any); ok {
		walkStructure(items, append(slices.Clone(at), "items"), into)
	}
}

func sortedNames(object map[string]any) []string {
	names := make([]string, 0, len(object))
	for name := range object {
		names = append(names, name)
	}
	slices.Sort(names)
	return names
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
