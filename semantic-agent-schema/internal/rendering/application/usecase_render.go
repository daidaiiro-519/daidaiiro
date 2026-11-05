package application

import (
	"context"
	"encoding/json"
	"errors"
	"path"
	"strings"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Render は、ユースケース「描画」。範囲のインスタンスを、デザインシステムで HTML にする。
// 担保：引数は機械的な検証（JSON Schema）、デザインシステムを契約のスキーマで、インスタンスを Concrete で検証するのは
// 機械的な検証（JSON Schema）、x-view でテンプレートを選ぶことと、テンプレートと JS が読む欄を Concrete とデザイントークンに
// 照らすことは機械的な検証（ドメインサービス「レンダラー」）、構文木を読むことと描画と書き出しは処理（アダプター）。
type Render struct {
	Documents      Documents
	Schemas        Schemas
	Validator      Validator
	Renderer       Renderer
	AbstractID     string // Abstract の $id
	DesignSystemID string // デザインシステムのスキーマ（design-system.schema.json）の $id
	DesignTokensID string // デザイントークンのスキーマ（design-tokens.schema.json）の $id
}

// RenderInput は、描画の引数。
type RenderInput struct {
	Paths        []domain.Path
	DesignSystem domain.Path
	Out          domain.Path // 書き出す先のディレクトリ
}

// Rendered は、書き出した HTML 1つ。元のインスタンスの path と、書き出した path を持つ。
type Rendered struct {
	Path, Out domain.Path
}

// Run は、デザインシステムとインスタンスを検証し、テンプレートを選んで、全部が描けてから書き出す。
// rejected のときは、1つも書き出さない。
func (u Render) Run(ctx context.Context, in RenderInput) ([]Rendered, error) {
	designSystem, err := u.designSystem(ctx, in.DesignSystem)
	if err != nil {
		return nil, err
	}
	pages := make([]domain.Page, 0, len(in.Paths))
	rendered := make([]Rendered, 0, len(in.Paths))
	var failures []error
	for _, p := range in.Paths {
		page, err := u.page(ctx, p, in.Out, designSystem)
		if err != nil {
			failures = append(failures, err)
			continue
		}
		pages = append(pages, page)
		rendered = append(rendered, Rendered{Path: p, Out: page.Out})
	}
	if err := errors.Join(failures...); err != nil {
		return nil, err
	}
	if err := overlaps(in, rendered); err != nil {
		return nil, err
	}
	if err := u.Renderer.Render(ctx, pages); err != nil {
		return nil, err
	}
	return rendered, nil
}

// designSystem は、デザインシステムを読み、契約のスキーマで検証して、テンプレートのファイルの中身と tokens を返す。
type designSystem struct {
	templates map[string]domain.Template
	tokens    any
}

// designSystem は、デザインシステムのファイルを読んで契約のスキーマで検証し、テンプレートのファイルと tokens を読んで返す。
func (u Render) designSystem(ctx context.Context, p domain.Path) (designSystem, error) {
	document, err := u.Documents.Read(ctx, p)
	if err != nil {
		return designSystem{}, err
	}
	value, err := domain.ParseJSON(document.Content)
	if err != nil {
		return designSystem{}, at(p, err)
	}
	violations, err := u.Validator.Check(ctx, u.DesignSystemID, document.Content)
	if err != nil {
		return designSystem{}, at(p, err)
	}
	if err := domain.FailuresOf(domain.SchemaViolation, p.String(), violations); err != nil {
		return designSystem{}, err
	}
	object, _ := value.(map[string]any) // 形は design-system.schema.json が決めている
	items, _ := object["templates"].(map[string]any)
	templates := map[string]domain.Template{}
	for name, item := range items {
		relative, _ := item.(string) // 形は design-system.schema.json が決めている
		template, err := u.template(ctx, p, relative)
		if err != nil {
			return designSystem{}, err
		}
		templates[name] = template
	}
	tokens, err := u.tokens(ctx, p, object["tokens"])
	if err != nil {
		return designSystem{}, err
	}
	return designSystem{templates: templates, tokens: tokens}, nil
}

// tokens は、デザイントークンの、JSON Schema で書けない決まり（受け継ぐ型 ・ 別名）を確かめ、別名を解いた値の木を返す。
// グループから型を受け継いだトークンの値は、その型の形（design-tokens.schema.json の $defs）で検証する。
func (u Render) tokens(ctx context.Context, p domain.Path, tokens any) (any, error) {
	resolved, list, err := domain.ResolveTokens(tokens)
	if err != nil {
		return nil, inTokens(p, err)
	}
	var failures []error
	for _, token := range list {
		if !token.Inherited || token.Alias {
			continue // 自分の $type の値は design-tokens.schema.json が、別名の値は先のトークンが検証する
		}
		content, err := json.Marshal(token.Value)
		if err != nil {
			return nil, err
		}
		violations, err := u.Validator.Check(ctx, u.DesignTokensID+"#/$defs/"+token.Type, content)
		if err != nil {
			return nil, at(p, err)
		}
		for _, v := range violations {
			failures = append(failures, &domain.Failure{
				Code: domain.SchemaViolation, Path: p.String(),
				InstanceLocation:        "/tokens" + token.Location.String() + "/$value" + v.InstanceLocation.String(),
				AbsoluteKeywordLocation: v.AbsoluteKeywordLocation, Message: v.Message,
			})
		}
	}
	if err := errors.Join(failures...); err != nil {
		return nil, err
	}
	return resolved, nil
}

// inTokens は、デザイントークンの中の位置の Failure を、デザインシステムの中の位置にする。
func inTokens(p domain.Path, err error) error {
	for _, e := range unjoin(err) {
		var f *domain.Failure
		if errors.As(e, &f) {
			f.Path = p.String()
			f.InstanceLocation = "/tokens" + f.InstanceLocation
		}
	}
	return err
}

// unjoin は、errors.Join で束ねた error を 1 つずつに分ける。束ねていなければ、その error 1 つを返す。
func unjoin(err error) []error {
	if joined, ok := err.(interface{ Unwrap() []error }); ok {
		return joined.Unwrap()
	}
	return []error{err}
}

// template は、デザインシステムのファイルのあるディレクトリからの相対の path で、テンプレートのファイルを読む。
// working tree の外を指せば path_outside、読めなければ file_not_found を返す。
func (u Render) template(ctx context.Context, designSystem domain.Path, relative string) (domain.Template, error) {
	p, err := domain.NewPath(path.Join(path.Dir(designSystem.String()), relative))
	if err != nil {
		return domain.Template{}, at(designSystem, err)
	}
	document, err := u.Documents.Read(ctx, p)
	if err != nil {
		return domain.Template{}, err
	}
	return domain.Template{Path: p, Text: string(document.Content)}, nil
}

// page は、インスタンスを読み、Concrete で検証し、x-view でテンプレートを選んで、書き出す HTML 1つを返す。
// 書き出す path は、--out の下に、元の path の .json を .html にしたもの。
func (u Render) page(ctx context.Context, p domain.Path, out domain.Path, ds designSystem) (domain.Page, error) {
	document, err := u.Documents.Read(ctx, p)
	if err != nil {
		return domain.Page{}, err
	}
	value, err := domain.ParseJSON(document.Content)
	if err != nil {
		return domain.Page{}, at(p, err)
	}
	schemaID, ok := domain.SchemaIDOf(value)
	if !ok {
		return domain.Page{}, &domain.Failure{
			Code: domain.SchemaNotFound, Path: p.String(), InstanceLocation: "/$schema",
			Message: "$schema に従う Concrete の $id が無い",
		}
	}
	concrete, err := u.Schemas.Lookup(ctx, schemaID)
	if err != nil {
		return domain.Page{}, at(p, err)
	}
	if schemaID != u.AbstractID {
		violations, err := u.Validator.Check(ctx, u.AbstractID, concrete.Content())
		if err != nil {
			return domain.Page{}, at(p, err)
		}
		var concretePath string
		if concrete.Path() != nil {
			concretePath = concrete.Path().String()
		}
		if err := domain.FailuresOf(domain.SchemaViolation, concretePath, violations); err != nil {
			return domain.Page{}, err
		}
	}
	violations, err := u.Validator.Check(ctx, schemaID, document.Content)
	if err != nil {
		return domain.Page{}, at(p, err)
	}
	if err := domain.FailuresOf(domain.SchemaViolation, p.String(), violations); err != nil {
		return domain.Page{}, err
	}
	template, err := domain.SelectTemplate(p, concrete, ds.templates)
	if err != nil {
		return domain.Page{}, err
	}
	// 描く前に、テンプレートと JS が読む欄を、Concrete とデザイントークンに静的に照らす
	references, err := u.Renderer.References(template)
	if err != nil {
		return domain.Page{}, err
	}
	if err := domain.CheckReferences(template, concrete, ds.tokens, references); err != nil {
		return domain.Page{}, err
	}
	target, err := domain.NewPath(path.Join(out.String(), strings.TrimSuffix(p.String(), ".json")+".html"))
	if err != nil {
		return domain.Page{}, at(p, err)
	}
	return domain.Page{Out: target, Template: template, Value: value, Tokens: ds.tokens}, nil
}

// overlaps は、書き出す先が入力と重なれば out_overlap を返す。重なるのは、書き出す path が範囲の path か
// デザインシステムの path と同じとき、または --out が範囲の path かデザインシステムの path と同じとき。
func overlaps(in RenderInput, rendered []Rendered) error {
	inputs := map[string]bool{in.DesignSystem.String(): true}
	for _, p := range in.Paths {
		inputs[p.String()] = true
	}
	if inputs[in.Out.String()] {
		return domain.Fail(domain.OutOverlap, in.Out.String(), "--out "+in.Out.String()+" が入力と同じ path")
	}
	for _, r := range rendered {
		if inputs[r.Out.String()] {
			return domain.Fail(domain.OutOverlap, r.Out.String(), r.Out.String()+" は入力の path")
		}
	}
	return nil
}

// at は、path の無い Failure に、その path を入れる。
func at(p domain.Path, err error) error {
	var f *domain.Failure
	if errors.As(err, &f) && f.Path == "" {
		f.Path = p.String()
	}
	return err
}
