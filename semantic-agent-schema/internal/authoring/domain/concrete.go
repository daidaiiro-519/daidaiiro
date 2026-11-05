package domain

import (
	"encoding/json"
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
	// ruleLocation は、id の rule を置いたサブスキーマの keyword location を返す。無ければ false を返す。
	ruleLocation(id string) (Pointer, bool)
}

// Subschema は、Concrete の1つのサブスキーマの、AI が読む部分。位置は、ルートスキーマからの keyword location で持つ。
type Subschema struct {
	KeywordLocation Pointer
	Title           string
	Description     string
	XPrompt         json.RawMessage // x-prompt。スキーマに書いた順のまま持つ。無ければ nil
}

type schema struct {
	id      string
	path    Path
	content []byte
	hash    Hash
	rules   map[string]Rule
	ruleAt  map[string]Pointer
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
func (s schema) ruleLocation(id string) (Pointer, bool) {
	p, ok := s.ruleAt[id]
	return p, ok
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
	ruleAt := map[string]Pointer{}
	walkSubschemas(root, nil, func(subschema map[string]any, at []string) {
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
				ruleAt[ruleID] = PointerOf(at...)
			}
		}
	})
	return schema{id: id, path: path, content: content, hash: HashOf(content), rules: rules, ruleAt: ruleAt, subs: structureSubschemas(root, content), root: root}, nil
}

// structureSubschemas は、ルートスキーマから JSON Schema の構造（properties と items）でたどれるサブスキーマを、
// keyword location の順に集める。x-prompt は、キーの順を変えずに取り出すため、書いたままのバイト列から取る。
func structureSubschemas(root map[string]any, content json.RawMessage) []Subschema {
	var subs []Subschema
	walkSubschemas(root, nil, func(subschema map[string]any, at []string) {
		if !inStructure(at) {
			return
		}
		sub := Subschema{KeywordLocation: PointerOf(at...)}
		sub.Title, _ = subschema["title"].(string)             // 無ければ空
		sub.Description, _ = subschema["description"].(string) // 無ければ空
		if _, ok := subschema["x-prompt"].(map[string]any); ok {
			sub.XPrompt = member(rawAt(content, at), "x-prompt")
		}
		subs = append(subs, sub)
	})
	return subs
}

// inStructure は、keyword location の参照トークンが、properties とその名前、items だけでできているかを返す。
func inStructure(at []string) bool {
	for i := 0; i < len(at); i++ {
		switch at[i] {
		case "properties":
			i++ // 次のトークンは名前
			if i >= len(at) {
				return false
			}
		case "items":
		default:
			return false
		}
	}
	return true
}

// member は、書いたままの JSON オブジェクトから、name のメンバーの値を書いたままのバイト列で返す。無ければ nil。
func member(raw json.RawMessage, name string) json.RawMessage {
	var object map[string]json.RawMessage
	if json.Unmarshal(raw, &object) != nil {
		return nil // ParseJSON が読めた JSON の一部なので、オブジェクトでなければ、そのメンバーは無い
	}
	return object[name]
}

// subschemaAt は、keyword location のサブスキーマの、AI が読む部分を返す。x-prompt は書いたままのバイト列から取り出す。
// 無ければ keyword location だけを持つ Subschema。
func subschemaAt(c Concrete, keyword Pointer) Subschema {
	tokens := tokensOf(keyword)
	sub := Subschema{KeywordLocation: keyword}
	object := schemaAt(c.structure(), tokens)
	sub.Title, _ = object["title"].(string)             // 無ければ空
	sub.Description, _ = object["description"].(string) // 無ければ空
	if _, ok := object["x-prompt"].(map[string]any); ok {
		sub.XPrompt = member(rawAt(c.Content(), tokens), "x-prompt")
	}
	return sub
}

// schemaAt は、ルートスキーマから参照トークンの並びでたどったサブスキーマを返す。無ければ nil。
func schemaAt(root map[string]any, tokens []string) map[string]any {
	var value any = root
	for _, token := range tokens {
		switch v := value.(type) {
		case map[string]any:
			value = v[token]
		case []any:
			i, err := strconv.Atoi(token)
			if err != nil || i < 0 || i >= len(v) {
				return nil
			}
			value = v[i]
		default:
			return nil
		}
	}
	object, _ := value.(map[string]any) // サブスキーマでなければ nil
	return object
}

// rawAt は、書いたままの JSON から、参照トークンの並びでたどった値を書いたままのバイト列で返す。無ければ nil。
func rawAt(raw json.RawMessage, tokens []string) json.RawMessage {
	for _, token := range tokens {
		var array []json.RawMessage
		if json.Unmarshal(raw, &array) != nil {
			raw = member(raw, token)
			continue
		}
		i, err := strconv.Atoi(token)
		if err != nil || i < 0 || i >= len(array) {
			return nil
		}
		raw = array[i]
	}
	return raw
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
