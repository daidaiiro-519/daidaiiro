package domain

import "strings"

// fontWeights は、fontWeight の名前から数への対応（Design Tokens Format Module 2025.10 の 8.4 の表）。
var fontWeights = map[string]string{
	"thin": "100", "hairline": "100", "extra-light": "200", "ultra-light": "200", "light": "300",
	"normal": "400", "regular": "400", "book": "400", "medium": "500", "semi-bold": "600", "demi-bold": "600",
	"bold": "700", "extra-bold": "800", "ultra-bold": "800", "black": "900", "heavy": "900",
	"extra-black": "950", "ultra-black": "950",
}

// colorFunctions は、hex が使えない色の、色空間から CSS Color 4 の関数への対応。無い色空間は color(<色空間> …) で書く。
var colorFunctions = map[string]string{
	"hsl": "hsl", "hwb": "hwb", "lab": "lab", "lch": "lch", "oklab": "oklab", "oklch": "oklch",
}

// CSSOf は、ドメインサービス「レンダラー」の、翻訳ツールとしての決まり。トークンの型から、値を CSS の文字列にする。
// dimension と duration は数と単位、color は hex（alpha が 1 でないときは CSS Color 4 の関数）、fontFamily は空白を含む名前を
// 引用符で囲んで「, 」でつなぎ、fontWeight は数、cubicBezier は cubic-bezier(…)、number は数。
func CSSOf(tokenType string, value any) string {
	switch tokenType {
	case "dimension", "duration":
		object, _ := value.(map[string]any) // 形は design-tokens.schema.json が決めている
		unit, _ := object["unit"].(string)
		return jsonText(object["value"]) + unit
	case "color":
		return colorCSS(value)
	case "fontFamily":
		names, ok := value.([]any)
		if !ok {
			names = []any{value}
		}
		quoted := make([]string, 0, len(names))
		for _, n := range names {
			name, _ := n.(string) // 形は design-tokens.schema.json が決めている
			if strings.ContainsAny(name, " \t") {
				name = `"` + strings.ReplaceAll(name, `"`, `\"`) + `"`
			}
			quoted = append(quoted, name)
		}
		return strings.Join(quoted, ", ")
	case "fontWeight":
		if name, ok := value.(string); ok {
			return fontWeights[name]
		}
		return jsonText(value)
	case "cubicBezier":
		points, _ := value.([]any) // 形は design-tokens.schema.json が決めている
		texts := make([]string, 0, len(points))
		for _, p := range points {
			texts = append(texts, jsonText(p))
		}
		return "cubic-bezier(" + strings.Join(texts, ", ") + ")"
	}
	return jsonText(value)
}

func colorCSS(value any) string {
	object, _ := value.(map[string]any) // 形は design-tokens.schema.json が決めている
	alpha, hasAlpha := object["alpha"]
	opaque := !hasAlpha || jsonText(alpha) == "1"
	if hex, ok := object["hex"].(string); ok && opaque {
		return hex
	}
	space, _ := object["colorSpace"].(string)
	components, _ := object["components"].([]any)
	texts := make([]string, 0, len(components))
	for _, c := range components {
		if text, ok := c.(string); ok {
			texts = append(texts, text) // none
			continue
		}
		texts = append(texts, jsonText(c))
	}
	body := strings.Join(texts, " ")
	if !opaque {
		body += " / " + jsonText(alpha)
	}
	if function, ok := colorFunctions[space]; ok {
		return function + "(" + body + ")"
	}
	return "color(" + space + " " + body + ")"
}
