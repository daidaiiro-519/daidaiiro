package domain

import (
	"errors"
	"slices"
	"strings"
)

// Token は、デザイントークン1つ。tokens の中の位置 ・ 決まった型 ・ 別名を解いた値を持つ。
// Inherited は、型を自分の $type でなく、グループか別名の先から受け継いだとき true。
type Token struct {
	Location  Pointer
	Type      string
	Value     any
	Alias     bool
	Inherited bool
}

// ResolveTokens は、ドメインサービス「レンダラー」の、デザイントークンの機械的な検証。JSON Schema で書けない決まり
// （グループから受け継ぐ $type ・ 別名の先が在るか ・ 輪になっていないか ・ 別名の先の型が合うか）を確かめ、
// 別名を解いた値の木（グループは名前から子への map、トークンは値。$ で始まる性質は除く）と、トークンの並びを返す。
// 別名の違反は token_alias_invalid、型が決まらないトークンは schema_violation を返す。
// 位置は tokens の中の JSON Pointer で、呼ぶ側がデザインシステムの中の位置にする。
func ResolveTokens(tokens any) (any, []Token, error) {
	r := &resolver{tokens: map[string]rawToken{}}
	r.collect(tokens, nil, "")
	var failures []error
	resolved := map[string]Token{}
	for _, key := range sortedTokenKeys(r.tokens) {
		token, err := r.resolve(key, nil)
		if err != nil {
			failures = append(failures, err)
			continue
		}
		resolved[key] = token
	}
	if err := errors.Join(failures...); err != nil {
		return nil, nil, err
	}
	var list []Token
	for _, key := range sortedTokenKeys(r.tokens) {
		list = append(list, resolved[key])
	}
	return tree(tokens, nil, resolved), list, nil
}

// rawToken は、別名を解く前のトークン。
type rawToken struct {
	location Pointer
	ownType  string
	inherit  string
	value    any
}

type resolver struct {
	tokens map[string]rawToken // 名前（a.b.c）からトークン
}

// collect は、グループをたどり、トークンを名前で集める。グループの $type を子へ受け継ぐ。
func (r *resolver) collect(node any, at []string, inherited string) {
	object, ok := node.(map[string]any)
	if !ok {
		return
	}
	if t, ok := object["$type"].(string); ok {
		inherited = t
	}
	for _, key := range sortedNames(object) {
		child, ok := object[key].(map[string]any)
		if !ok || (strings.HasPrefix(key, "$") && key != "$root") {
			continue
		}
		path := append(slices.Clone(at), key)
		if _, isToken := child["$value"]; isToken {
			ownType, _ := child["$type"].(string) // 無ければ空
			r.tokens[strings.Join(path, ".")] = rawToken{
				location: PointerOf(path...), ownType: ownType, inherit: inherited, value: child["$value"],
			}
			continue
		}
		r.collect(child, path, inherited)
	}
}

// resolve は、名前のトークンの型と、別名を解いた値を返す。visiting は、たどっている別名の名前の並び（輪を見つけるため）。
func (r *resolver) resolve(key string, visiting []string) (Token, error) {
	raw := r.tokens[key]
	at := PointerOf(append(strings.Split(key, "."), "$value")...)
	target, isAlias := aliasOf(raw.value)
	if !isAlias {
		t := raw.ownType
		if t == "" {
			t = raw.inherit
		}
		if t == "" {
			return Token{}, &Failure{Code: SchemaViolation, InstanceLocation: raw.location.String(), Message: key + " の型が決まらない（$type もグループの $type も無い）"}
		}
		return Token{Location: raw.location, Type: t, Value: raw.value, Inherited: raw.ownType == ""}, nil
	}
	if slices.Contains(visiting, key) {
		return Token{}, &Failure{Code: TokenAliasInvalid, InstanceLocation: at.String(), Message: "別名が輪になっている：" + strings.Join(append(visiting, key), " → ")}
	}
	if _, ok := r.tokens[target]; !ok {
		return Token{}, &Failure{Code: TokenAliasInvalid, InstanceLocation: at.String(), Message: "{" + target + "} は在るトークンでない（無いか、グループを指す）"}
	}
	resolved, err := r.resolve(target, append(slices.Clone(visiting), key))
	if err != nil {
		var f *Failure
		if errors.As(err, &f) && f.Code == TokenAliasInvalid && len(visiting) == 0 {
			// 別名の先の違反は、先のトークンが自分で出すので、ここでは別名の位置だけを示す
			return Token{}, &Failure{Code: TokenAliasInvalid, InstanceLocation: at.String(), Message: "{" + target + "} を解けない：" + f.Message}
		}
		return Token{}, err
	}
	declared := raw.ownType
	if declared == "" {
		declared = raw.inherit
	}
	if declared != "" && declared != resolved.Type {
		return Token{}, &Failure{Code: TokenAliasInvalid, InstanceLocation: at.String(), Message: key + " の型 " + declared + " と、別名の先 {" + target + "} の型 " + resolved.Type + " が違う"}
	}
	return Token{Location: raw.location, Type: resolved.Type, Value: resolved.Value, Alias: true, Inherited: raw.ownType == ""}, nil
}

// aliasOf は、値が {group.token} の形の別名なら、その名前を返す。
func aliasOf(value any) (string, bool) {
	text, ok := value.(string)
	if !ok || len(text) < 3 || !strings.HasPrefix(text, "{") || !strings.HasSuffix(text, "}") || strings.ContainsAny(text[1:len(text)-1], "{}") {
		return "", false
	}
	return text[1 : len(text)-1], true
}

// tree は、別名を解いた値の木を返す。テンプレートには、この木を .Tokens として渡す。
func tree(node any, at []string, resolved map[string]Token) any {
	object, ok := node.(map[string]any)
	if !ok {
		return nil
	}
	out := map[string]any{}
	for key, child := range object {
		if strings.HasPrefix(key, "$") && key != "$root" {
			continue
		}
		path := append(slices.Clone(at), key)
		if token, ok := resolved[strings.Join(path, ".")]; ok {
			out[key] = token.Value
			continue
		}
		out[key] = tree(child, path, resolved)
	}
	return out
}

func sortedTokenKeys(tokens map[string]rawToken) []string {
	keys := make([]string, 0, len(tokens))
	for k := range tokens {
		keys = append(keys, k)
	}
	slices.Sort(keys)
	return keys
}
