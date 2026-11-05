package domain

// Page は、描画で書き出す HTML 1つ。書き出す path ・ テンプレート ・ テンプレートに渡す .Value（インスタンス）と
// .Tokens（デザイントークン）を持つ。テンプレートに渡すのは、この2つだけ。
type Page struct {
	Out      Path
	Template string
	Value    any
	Tokens   any
}

// SelectTemplate は、ドメインサービス「レンダラー」。Concrete のルートスキーマの x-view で、デザインシステムの
// テンプレートを選んで返す。x-view が無いか、その名前のテンプレートがデザインシステムに無ければ template_missing を返す。
func SelectTemplate(path Path, concrete Concrete, templates map[string]string) (string, error) {
	view := concrete.View()
	if view == "" {
		return "", &Failure{Code: TemplateMissing, Path: path.String(), Message: concrete.ID() + " のルートスキーマに x-view が無い"}
	}
	template, ok := templates[view]
	if !ok {
		return "", &Failure{Code: TemplateMissing, Path: path.String(), Message: "x-view " + view + " のテンプレートがデザインシステムに無い"}
	}
	return template, nil
}
