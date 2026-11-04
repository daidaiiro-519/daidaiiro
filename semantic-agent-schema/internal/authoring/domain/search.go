package domain

import "strings"

// Condition は、検索の条件1つ。instance location の値が、条件の値と同じであること。
type Condition struct {
	InstanceLocation Pointer
	Value            any // ParseJSON で読んだ値
}

// Matches は、ドメインサービス「検索の比較」。JSON ドキュメントの値が、すべての条件で、その instance location に
// 条件と同じ値を持つかを返す。比べるのは JSON の文字列で、数は書いたままの文字列で比べる。
func Matches(value any, conditions []Condition) bool {
	for _, c := range conditions {
		found, ok := ValueAt(value, c.InstanceLocation)
		if !ok || jsonText(found) != jsonText(c.Value) {
			return false
		}
	}
	return true
}

// ValueAt は、instance location の値を返す。無ければ false を返す。
func ValueAt(value any, location Pointer) (any, bool) {
	if location.String() == "" {
		return value, true
	}
	for _, token := range strings.Split(location.String()[1:], "/") {
		token = strings.ReplaceAll(strings.ReplaceAll(token, "~1", "/"), "~0", "~")
		switch v := value.(type) {
		case map[string]any:
			child, ok := v[token]
			if !ok {
				return nil, false
			}
			value = child
		case []any:
			i, ok := arrayIndex(token, len(v))
			if !ok {
				return nil, false
			}
			value = v[i]
		default:
			return nil, false
		}
	}
	return value, true
}

// arrayIndex は、配列の参照トークン（先頭に 0 を付けない10進数）を添字にする。範囲の外なら false を返す。
func arrayIndex(token string, length int) (int, bool) {
	if token == "" || (len(token) > 1 && token[0] == '0') {
		return 0, false
	}
	i := 0
	for _, r := range token {
		if r < '0' || r > '9' {
			return 0, false
		}
		i = i*10 + int(r-'0')
		if i >= length {
			return 0, false
		}
	}
	return i, true
}
