package domain

import (
	"errors"
	"strconv"
	"strings"
)

// Template は、デザインシステムのテンプレート1つ。テンプレートのファイルの path と中身を持つ。
type Template struct {
	Path Path
	Text string
}

// Page は、描画で書き出す HTML 1つ。書き出す path ・ テンプレート ・ テンプレートに渡す .Value（インスタンス）と
// .Tokens（デザイントークン）を持つ。テンプレートに渡すのは、この2つだけ。
type Page struct {
	Out      Path
	Template Template
	Value    any
	Tokens   any
}

// Reference は、テンプレートか、テンプレートの中の JS が読む欄1つ。Root は Value か Tokens で、Segments はその下の並び。
// Line は、テンプレートのファイルの中の行。
type Reference struct {
	Root     string
	Segments []Segment
	Line     int
}

// Segment は、欄の並びの1つ。Element が true なら配列の要素（range や for…of の1つ）、そうでなければ Name のプロパティ。
type Segment struct {
	Name    string
	Element bool
}

// String は、欄の並びを .Value.a[].b の形で返す。
func (r Reference) String() string {
	var b strings.Builder
	b.WriteString("." + r.Root)
	for _, s := range r.Segments {
		if s.Element {
			b.WriteString("[]")
			continue
		}
		b.WriteString("." + s.Name)
	}
	return b.String()
}

// SelectTemplate は、ドメインサービス「レンダラー」。Concrete のルートスキーマの x-view で、デザインシステムの
// テンプレートを選んで返す。x-view が無いか、その名前のテンプレートがデザインシステムに無ければ template_missing を返す。
func SelectTemplate(path Path, concrete Concrete, templates map[string]Template) (Template, error) {
	view := concrete.View()
	if view == "" {
		return Template{}, &Failure{Code: TemplateMissing, Path: path.String(), Message: concrete.ID() + " のルートスキーマに x-view が無い"}
	}
	template, ok := templates[view]
	if !ok {
		return Template{}, &Failure{Code: TemplateMissing, Path: path.String(), Message: "x-view " + view + " のテンプレートがデザインシステムに無い"}
	}
	return template, nil
}

// CheckReferences は、ドメインサービス「レンダラー」の機械的な検証。描く前に、テンプレートが読む欄を静的に照らす。
// .Value の欄は Concrete の properties と items をたどり、.Tokens の欄はデザイントークンの値をたどる。
// properties を持つサブスキーマ（とオブジェクトのデザイントークン）の下で名前が無ければ、欄ごとの
// template_reference_missing をまとめて返す。properties を持たないサブスキーマ（配列 ・ 文字列など）の下は、
// 組み込みの読み方（length など）があるので照らさない。値が無い任意の欄は、ここでは通る。
func CheckReferences(template Template, concrete Concrete, tokens any, references []Reference) error {
	var failures []error
	seen := map[string]bool{}
	var root map[string]any // Value の欄を照らすときに、最初の1回だけ Concrete から取る
	for _, r := range references {
		missing := -1 // 無い欄の、Segments の中の位置。-1 なら照らせた
		switch r.Root {
		case "Value":
			if root == nil {
				root = concrete.structure()
			}
			missing = inSchema(root, r.Segments)
		case "Tokens":
			missing = inTokens(tokens, r.Segments)
		default:
			missing = 0
		}
		if missing < 0 {
			continue
		}
		// 無い欄の下を読む欄は、同じ無い欄として1つにまとめる
		first := Reference{Root: r.Root, Segments: r.Segments[:min(missing+1, len(r.Segments))], Line: r.Line}
		key := first.String() + "@" + strconv.Itoa(r.Line)
		if seen[key] {
			continue
		}
		seen[key] = true
		failures = append(failures, &Failure{
			Code: TemplateReferenceMissing, Path: template.Path.String(), Line: r.Line,
			Message: first.String() + " は " + map[string]string{"Value": "Concrete " + concrete.ID(), "Tokens": "デザイントークン"}[r.Root] + " に無い",
		})
	}
	return errors.Join(failures...)
}

// inSchema は、欄の並びを Concrete の properties と items でたどり、無い名前の位置を返す。照らせたなら -1。
func inSchema(subschema map[string]any, steps []Segment) int {
	for i, s := range steps {
		if s.Element {
			items, ok := subschema["items"].(map[string]any)
			if !ok {
				return -1 // 配列の要素の形が決まっていないので、照らせない
			}
			subschema = items
			continue
		}
		properties, ok := subschema["properties"].(map[string]any)
		if !ok {
			return -1 // properties を持たないサブスキーマの下は、組み込みの読み方があるので照らさない
		}
		child, ok := properties[s.Name].(map[string]any)
		if !ok {
			return i
		}
		subschema = child
	}
	return -1
}

// inTokens は、欄の並びをデザイントークンの値でたどり、無い名前の位置を返す。照らせたなら -1。
func inTokens(value any, steps []Segment) int {
	for i, s := range steps {
		switch v := value.(type) {
		case map[string]any:
			if s.Element {
				return -1
			}
			child, ok := v[s.Name]
			if !ok {
				return i
			}
			value = child
		case []any:
			if !s.Element || len(v) == 0 {
				return -1 // 配列の組み込みの読み方か、要素の無い配列なので照らさない
			}
			value = v[0]
		default:
			return -1 // 文字列や数の下は、組み込みの読み方があるので照らさない
		}
	}
	return -1
}
