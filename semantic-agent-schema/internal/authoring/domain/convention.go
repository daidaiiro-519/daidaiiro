package domain

import (
	"slices"
	"strconv"
	"strings"
)

// Convention は、ドメインサービス バリデーター。JSON Schema で書けない Concrete の作法を確かめ、違反を返す。
//   - ルートスキーマの $schema の const が $id と同じ
//   - rule の id が重ならない
//   - 例の related のキーが related_fields にある（違反の位置は、同じ rule の related_fields。例が正しく、
//     related_fields に足りないことが多いため。同じ rule の違反は1件にまとめ、例の位置と足りないキーを message に並べる）
//
// Abstract の違反（キーワードが無い ・ 型が違う）は JSON Schema が出すので、ここでは数えない。
func Convention(concrete any) []Violation {
	root, ok := concrete.(map[string]any)
	if !ok {
		return nil
	}
	var violations []Violation
	if v, ok := schemaConst(root); ok {
		violations = append(violations, v)
	}
	seen := map[string]bool{}
	walkSubschemas(root, nil, func(subschema map[string]any, at []string) {
		violations = append(violations, rulesOf(subschema, at, seen)...)
	})
	return violations
}

// schemaConst は、ルートスキーマの $schema の const が $id と違えば、その違反を返す。
func schemaConst(root map[string]any) (Violation, bool) {
	properties, _ := root["properties"].(map[string]any) // 無ければ確かめるものが無い
	schema, _ := properties["$schema"].(map[string]any)
	constValue, hasConst := schema["const"].(string)
	id, hasID := root["$id"].(string)
	if !hasConst || !hasID || constValue == id {
		return Violation{}, false
	}
	return Violation{
		InstanceLocation: PointerOf("properties", "$schema", "const"),
		Message:          "$schema の const が $id と違う：" + constValue + " ≠ " + id,
	}, true
}

// rulesOf は、サブスキーマの x-prompt の rules に作法を当てる。rule の id の重なりと、例の related のキーが related_fields に無いことを違反にする。seen は、Concrete 全体で出た rule の id。
func rulesOf(subschema map[string]any, at []string, seen map[string]bool) []Violation {
	var violations []Violation
	for i, item := range promptRules(subschema) {
		rule, ok := item.(map[string]any)
		if !ok {
			continue
		}
		ruleAt := append(slices.Clone(at), "x-prompt", "rules", strconv.Itoa(i))
		if id, ok := rule["id"].(string); ok {
			if seen[id] {
				violations = append(violations, Violation{
					InstanceLocation: PointerOf(append(slices.Clone(ruleAt), "id")...),
					Message:          "rule の id が重なる：" + id,
				})
			}
			seen[id] = true
		}
		fields := map[string]bool{}
		related, _ := rule["related_fields"].([]any) // 無ければ related_fields は空
		for _, f := range related {
			if name, ok := f.(string); ok {
				fields[name] = true
			}
		}
		var missing []string // 例の位置ごとの、related_fields に無いキー
		for _, side := range []string{"bad", "good"} {
			examples, _ := rule[side].([]any) // 無ければ例は無い
			for j, e := range examples {
				example, _ := e.(map[string]any)
				values, _ := example["related"].(map[string]any)
				var keys []string
				for key := range values {
					if !fields[key] {
						keys = append(keys, key)
					}
				}
				if len(keys) == 0 {
					continue
				}
				slices.Sort(keys)
				at := PointerOf(append(slices.Clone(ruleAt), side, strconv.Itoa(j), "related")...)
				missing = append(missing, strconv.Quote(at.String())+" の "+strings.Join(keys, " ・ "))
			}
		}
		if len(missing) > 0 {
			violations = append(violations, Violation{
				InstanceLocation: PointerOf(append(slices.Clone(ruleAt), "related_fields")...),
				Message: "related_fields に、例の related のキーが無い（" + strings.Join(missing, "、") +
					"）。rule の文がその値を読むなら related_fields に足し、読まないなら例の related から除く",
			})
		}
	}
	return violations
}
