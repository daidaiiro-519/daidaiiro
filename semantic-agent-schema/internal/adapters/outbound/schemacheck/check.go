// Package schemacheck は、アダプター。ポート Validator を JSON Schema の実装で満たし、違反をライブラリの出力のまま
// instance location ・ absolute keyword location ・ message へ変換する。独自の判定も独自の文も足さない。
// x-prompt を持つサブスキーマがインスタンスのどの位置に当たるかも、ライブラリの検証が決めた組をそのまま返す。
package schemacheck

import (
	"bytes"
	"context"
	"errors"
	"net/url"
	"slices"
	"strings"

	"github.com/santhosh-tekuri/jsonschema/v6"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
)

// Lookup は、公開された言葉に無い $id のスキーマ（schemas の中の Concrete）のバイト列を返す。
type Lookup func(ctx context.Context, id string) ([]byte, error)

// Checker は、公開された言葉と、Lookup で引いた Concrete で JSON を検証する。
type Checker struct {
	lookup Lookup
}

// New は、Checker を返す。lookup が nil なら、公開された言葉だけで検証する。
func New(lookup Lookup) *Checker {
	return &Checker{lookup: lookup}
}

// loader は、Compile が知らない $id を引く。引けなかったときの error を残す。
type loader struct {
	load func(id string) ([]byte, error)
	err  error
}

// Load は、url の $id のスキーマを読んだ値を返す。
func (l *loader) Load(url string) (any, error) {
	if l.load == nil {
		l.err = domain.Fail(domain.SchemaNotFound, "", url)
		return nil, l.err
	}
	content, err := l.load(url)
	if err != nil {
		l.err = err
		return nil, err
	}
	return jsonschema.UnmarshalJSON(bytes.NewReader(content))
}

// Check は、content を schemaID のスキーマで検証し、JSON Schema の違反を返す。
// JSON として読めなければ invalid_json、スキーマを引けなければその error を返す。
func (c *Checker) Check(ctx context.Context, schemaID string, content []byte) ([]domain.Violation, error) {
	violations, _, err := c.check(ctx, schemaID, content, nil)
	return violations, err
}

// Evaluate は、content を schemaID のスキーマで検証し、JSON Schema の違反と、x-prompt を持つサブスキーマが当たったインスタンスの
// 位置の組を返す。組は、検証のたびにライブラリが呼ぶ拡張項目の SchemaExt が集めたもので、keyword location は schemaID の
// スキーマ（resource）の中の位置にする。ほかの resource のサブスキーマの組は返さない。
func (c *Checker) Evaluate(ctx context.Context, schemaID string, content []byte) ([]domain.Violation, []domain.Evaluation, error) {
	violations, located, err := c.check(ctx, schemaID, content, nil)
	if err != nil {
		return nil, nil, err
	}
	var evaluations []domain.Evaluation
	for _, l := range located {
		resource, fragment, _ := strings.Cut(l.schema.Location, "#")
		if resource != schemaID {
			continue
		}
		decoded, err := url.PathUnescape(fragment)
		if err != nil {
			return nil, nil, err
		}
		keyword, err := domain.NewPointer(decoded)
		if err != nil {
			return nil, nil, err
		}
		evaluations = append(evaluations, domain.Evaluation{KeywordLocation: keyword, InstanceLocation: domain.PointerOf(l.at...)})
	}
	return violations, evaluations, nil
}

// located は、検証で、x-prompt を持つサブスキーマ schema が、インスタンスの位置 at（参照トークンの並び）に当たったこと。
type located struct {
	schema *jsonschema.Schema
	at     []string
}

// xPrompt は、x-prompt を持つサブスキーマの SchemaExt。検証の振る舞いは足さず、当たったインスタンスの位置を into に足すだけ。
type xPrompt struct {
	schema *jsonschema.Schema
	into   *[]located
}

// Validate は、このサブスキーマが当たったインスタンスの位置を記録する。
func (x *xPrompt) Validate(ctx *jsonschema.ValidatorContext, _ any) {
	*x.into = append(*x.into, located{schema: x.schema, at: slices.Clone(ctx.ValueLocation())})
}

// check は、prepare（無ければ nil）で Compiler に resource を足してから、content を schemaID のスキーマで検証する。
// JSON Schema の違反と、x-prompt を持つサブスキーマが当たった位置を返す。
func (c *Checker) check(ctx context.Context, schemaID string, content []byte, prepare func(*jsonschema.Compiler) error) ([]domain.Violation, []located, error) {
	value, err := jsonschema.UnmarshalJSON(bytes.NewReader(content))
	if err != nil {
		return nil, nil, domain.Fail(domain.InvalidJSON, "", err.Error())
	}
	l := &loader{}
	if c.lookup != nil {
		l.load = func(id string) ([]byte, error) { return c.lookup(ctx, id) }
	}
	var into []located
	compiler, err := newCompiler(l, &into)
	if err != nil {
		return nil, nil, err
	}
	if prepare != nil {
		if err := prepare(compiler); err != nil {
			return nil, nil, err
		}
	}
	schema, err := compiler.Compile(schemaID)
	if err != nil {
		return nil, nil, compileError(err, l.err)
	}
	// Compile もメタスキーマでの検証で位置を足すので、content の検証で当たった位置だけを残す
	into = nil
	err = schema.Validate(value)
	if err == nil {
		return nil, into, nil
	}
	var verr *jsonschema.ValidationError
	if !errors.As(err, &verr) {
		return nil, nil, domain.Fail(domain.SchemaViolation, "", err.Error())
	}
	return leaves(verr, nil, nil), into, nil
}

// CheckWith は、content を、まだ書いていないスキーマ schema（バイト列）で検証し、JSON Schema の違反を返す。
// schema の $id は、schemas の中の同じ $id の Concrete より先に使う。
func (c *Checker) CheckWith(ctx context.Context, schema, content []byte) ([]domain.Violation, error) {
	value, err := jsonschema.UnmarshalJSON(bytes.NewReader(schema))
	if err != nil {
		return nil, domain.Fail(domain.InvalidJSON, "", err.Error())
	}
	object, _ := value.(map[string]any) // $id が無ければ空の $id になり、Compile が error を返す
	id, _ := object["$id"].(string)
	violations, _, err := c.check(ctx, id, content, func(compiler *jsonschema.Compiler) error { return compiler.AddResource(id, value) })
	return violations, err
}

// newCompiler は、公開された言葉を resource にし、拡張項目の語彙を登録した Compiler を返す。
// x-prompt を持つサブスキーマは、検証で当たったインスタンスの位置を into に足す。
func newCompiler(l *loader, into *[]located) (*jsonschema.Compiler, error) {
	vocabulary, err := compileVocabulary()
	if err != nil {
		return nil, err
	}
	compiler := jsonschema.NewCompiler()
	// format を注釈でなく違反として検証する。Abstract が、ライブラリが確かめられる名前だけを書かせる
	compiler.AssertFormat()
	if err := addPublished(compiler); err != nil {
		return nil, err
	}
	compiler.RegisterVocabulary(&jsonschema.Vocabulary{
		URL:    publishedlanguage.VocabularyID,
		Schema: vocabulary,
		// 拡張項目は検証の振る舞いを足さない。形は vocabulary.schema.json が決める。x-prompt を持つサブスキーマだけ、
		// 自分の *Schema（Location を持つ）を引いて、当たった位置を記録する
		Compile: func(ctx *jsonschema.CompilerContext, object map[string]any) (jsonschema.SchemaExt, error) {
			if _, ok := object["x-prompt"].(map[string]any); !ok {
				return nil, nil
			}
			return &xPrompt{schema: ctx.Enqueue(nil), into: into}, nil
		},
	})
	compiler.UseLoader(l)
	return compiler, nil
}

func compileVocabulary() (*jsonschema.Schema, error) {
	compiler := jsonschema.NewCompiler()
	if err := addPublished(compiler); err != nil {
		return nil, err
	}
	return compiler.Compile(publishedlanguage.VocabularyID)
}

func addPublished(compiler *jsonschema.Compiler) error {
	for _, name := range publishedlanguage.Schemas {
		content, err := publishedlanguage.Read(name)
		if err != nil {
			return err
		}
		value, err := jsonschema.UnmarshalJSON(bytes.NewReader(content))
		if err != nil {
			return err
		}
		object, _ := value.(map[string]any) // 公開された言葉のスキーマは、どれも $id を持つオブジェクト
		id, _ := object["$id"].(string)
		if err := compiler.AddResource(id, value); err != nil {
			return err
		}
	}
	return nil
}

// compileError は、Compile の error を返す。引けなかった $id があればその error、
// $ref の先の Concrete が Abstract に違反していれば、違反の葉ごとの schema_violation をまとめた error にする。
func compileError(err, loadErr error) error {
	if loadErr != nil {
		return loadErr
	}
	var serr *jsonschema.SchemaValidationError
	var verr *jsonschema.ValidationError
	if !errors.As(err, &serr) || !errors.As(serr.Err, &verr) {
		return domain.Fail(domain.SchemaViolation, "", err.Error())
	}
	var failures []error
	for _, v := range leaves(verr, nil, nil) {
		failures = append(failures, &domain.Failure{
			Code: domain.SchemaViolation, InstanceLocation: v.InstanceLocation.String(),
			AbsoluteKeywordLocation: v.AbsoluteKeywordLocation, Message: v.Message,
		})
	}
	return errors.Join(failures...)
}

// leaves は、Causes を葉までたどり、葉ごとに違反を返す。
// propertyNames の違反の instance location は、親（その違反を Causes に持つ違反）の instance location で置き換える。
// enclosing は親の instance location、fixed は置き換えた後の instance location。
func leaves(e *jsonschema.ValidationError, enclosing, fixed []string) []domain.Violation {
	location := e.InstanceLocation
	keywords := e.ErrorKind.KeywordPath()
	switch {
	case fixed != nil:
		location = fixed
	case len(keywords) > 0 && keywords[0] == "propertyNames":
		location = enclosing
		fixed = enclosing
	}
	if len(e.Causes) == 0 {
		return []domain.Violation{{
			InstanceLocation:        domain.PointerOf(location...),
			AbsoluteKeywordLocation: e.SchemaURL + domain.PointerOf(keywords...).String(),
			Message:                 message(e),
		}}
	}
	var violations []domain.Violation
	for _, cause := range e.Causes {
		violations = append(violations, leaves(cause, slices.Clone(location), fixed)...)
	}
	return violations
}

// message は、ライブラリが出した葉の違反の文から、先頭の「at '<instance location>': 」を除いた文を返す。
func message(e *jsonschema.ValidationError) string {
	text := e.Error()
	if strings.HasPrefix(text, "at '") {
		if i := strings.Index(text, "': "); i >= 0 {
			return text[i+3:]
		}
	}
	return text
}
