// Package schemacheck は、アダプター。ポート Validator を JSON Schema の実装で満たし、違反をライブラリの出力のまま
// instance location ・ absolute keyword location ・ message へ変換する。独自の判定も独自の文も足さない。message には、違反した keyword を直接持つサブスキーマの
// description を、スキーマの作者の文のまま続ける。
// サブスキーマがインスタンスのどの位置に当たるかも、ライブラリの検証が決めた組をそのまま返す。
package schemacheck

import (
	"bytes"
	"context"
	"errors"
	"net/url"
	"slices"
	"strconv"
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
	docs map[string]any // 読んだスキーマの値を $id ごとに残す。違反の description を引くため
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
	value, err := jsonschema.UnmarshalJSON(bytes.NewReader(content))
	if err == nil {
		l.docs[url] = value
	}
	return value, err
}

// Check は、content を schemaID のスキーマで検証し、JSON Schema の違反を返す。
// JSON として読めなければ invalid_json、スキーマを引けなければその error を返す。
func (c *Checker) Check(ctx context.Context, schemaID string, content []byte) ([]domain.Violation, error) {
	violations, _, err := c.check(ctx, schemaID, content, nil, false)
	return violations, err
}

// Evaluate は、content を schemaID のスキーマで検証し、JSON Schema の違反と、サブスキーマが当たったインスタンスの
// 位置の組を返す。組は、JSON Schema の注釈と同じに集める（JSON Schema 2020-12 Core 7.7 と 10.2.1.2）。
// 検証に通ったサブスキーマの組だけを返し、検証に落ちたサブスキーマの下の組は返さない。anyOf は、通った枝すべての組を返す。
// keyword location は schemaID のスキーマ（resource）の中の位置にする。ほかの resource のサブスキーマの組は返さない。
// ルートスキーマの検証に落ちたとき（違反があるとき）は、直す対象を示すため、通ったサブスキーマすべての組を返す。
func (c *Checker) Evaluate(ctx context.Context, schemaID string, content []byte) ([]domain.Violation, []domain.Evaluation, error) {
	violations, located, err := c.check(ctx, schemaID, content, nil, true)
	if err != nil {
		return nil, nil, err
	}
	var evaluations []domain.Evaluation
	seen := map[string]bool{}
	for _, l := range located {
		id := l.schema.Location + "\x00" + domain.PointerOf(l.at...).String()
		if seen[id] {
			continue
		}
		seen[id] = true
		_, fragment, _ := strings.Cut(l.schema.Location, "#")
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

// located は、サブスキーマ schema が、インスタンスの位置 at（参照トークンの並び）に当たり、検証に通ったこと。
type located struct {
	schema *jsonschema.Schema
	at     []string
}

func (l located) key() memoKey { return memoKey{l.schema, strings.Join(l.at, marker)} }

// pairSet は、重ならない組の並び。同じ組へ至る道が複数あるので、足すときに重なりを除く。
type pairSet struct {
	list []located
	seen map[memoKey]bool
}

func (s *pairSet) add(pairs ...located) {
	if s.seen == nil {
		s.seen = map[memoKey]bool{}
	}
	for _, p := range pairs {
		if k := p.key(); !s.seen[k] {
			s.seen[k] = true
			s.list = append(s.list, p)
		}
	}
}

// outcome は、サブスキーマをインスタンスの位置に当てた結果。ok なら、通ったサブスキーマとその下の、組を持つ。
type outcome struct {
	ok    bool
	pairs []located
}

type memoKey struct {
	schema *jsonschema.Schema
	at     string
}

// frame は、いま検証し直しているサブスキーマ。raw は、検証し直しのために足した参照トークンの、位置の並びの中の添え字。
type frame struct {
	key   memoKey
	raw   int
	pairs pairSet
}

// marker は、ValidatorContext.Validate がサブスキーマを検証し直すときに位置の並びへ足す参照トークン。
const marker = "\x00"

// collector は、ライブラリが呼ぶ拡張項目の SchemaExt が組を集めるための状態。
// ライブラリは、拡張項目を、サブスキーマの検証に落ちていても呼ぶ。そこで拡張項目の中でそのサブスキーマの検証をやり直し、
// 通ったときだけ組にする。やり直しの中で呼ばれる子の拡張項目の組は、通った親にだけ足す。結果は（サブスキーマ, 位置）ごとに覚え、
// 同じ検証をやり直さない。anyOf は通る枝で検証を打ち切るので、打ち切られた枝も検証し直す。
type collector struct {
	target string // 組を返す resource の $id
	active bool   // 検証の間だけ true。Compile がメタスキーマで検証するときは集めない
	frames []frame
	memo   map[memoKey]*outcome
	// visited は、検証がサブスキーマに当たった組すべて（落ちたものも含む）
	visited []located
}

// logical は、位置の並びから、検証し直しのために足した参照トークンを除く。
func (c *collector) logical(raw []string) []string {
	at := make([]string, 0, len(raw))
	for i, token := range raw {
		if !slices.ContainsFunc(c.frames, func(f frame) bool { return f.raw == i }) {
			at = append(at, token)
		}
	}
	return at
}

// pairs は、検証の結果の組を返す。ルートスキーマが通ったときは、ルートの組（通ったサブスキーマの下だけ）。
// 落ちたときは、検証が当たったサブスキーマすべての組（落ちたものも含む）。直す対象の位置を示すため。
func (c *collector) pairs(root *jsonschema.Schema) []located {
	if out := c.memo[memoKey{root, ""}]; out != nil && out.ok {
		return out.pairs
	}
	return c.visited
}

// visit は、ライブラリの拡張項目の呼び出し。s が、いまの位置の値 v に当たったときに呼ばれる。
func (c *collector) visit(ctx *jsonschema.ValidatorContext, s *jsonschema.Schema, v any) {
	if resource, _, _ := strings.Cut(s.Location, "#"); !c.active || resource != c.target {
		return // 組を返すのは target のサブスキーマだけなので、ほかの resource の検証し直しはしない
	}
	at := c.logical(ctx.ValueLocation())
	key := memoKey{s, strings.Join(at, marker)}
	if slices.ContainsFunc(c.frames, func(f frame) bool { return f.key == key }) {
		return // 検証し直している、そのサブスキーマ自身の呼び出し
	}
	if len(c.frames) == 0 {
		c.visited = append(c.visited, located{schema: s, at: at})
	}
	out := c.outcomeOf(ctx, s, v, at, key)
	if out.ok && len(c.frames) > 0 {
		c.frames[len(c.frames)-1].pairs.add(out.pairs...)
	}
}

func (c *collector) outcomeOf(ctx *jsonschema.ValidatorContext, s *jsonschema.Schema, v any, at []string, key memoKey) *outcome {
	if out, ok := c.memo[key]; ok {
		return out
	}
	c.frames = append(c.frames, frame{key: key, raw: len(ctx.ValueLocation())})
	err := ctx.Validate(s, v, []string{marker})
	f := c.frames[len(c.frames)-1]
	c.frames = c.frames[:len(c.frames)-1]
	out := &outcome{}
	c.memo[key] = out
	if err != nil {
		return out
	}
	out.ok = true
	f.pairs.add(located{schema: s, at: at})
	for _, branch := range s.AnyOf {
		if b := c.outcomeOf(ctx, branch, v, at, memoKey{branch, key.at}); b.ok {
			f.pairs.add(b.pairs...)
		}
	}
	out.pairs = f.pairs.list
	return out
}

// applied は、すべてのサブスキーマの SchemaExt。検証の振る舞いは足さず、組を集めるだけ。
type applied struct {
	schema *jsonschema.Schema
	into   *collector
}

// Validate は、このサブスキーマが当たったことを collector へ渡す。
func (x *applied) Validate(ctx *jsonschema.ValidatorContext, v any) {
	x.into.visit(ctx, x.schema, v)
}

// check は、prepare（無ければ nil）で Compiler に resource を足してから、content を schemaID のスキーマで検証する。
// JSON Schema の違反を返す。collect なら、組も返す。
func (c *Checker) check(ctx context.Context, schemaID string, content []byte, prepare func(*jsonschema.Compiler, map[string]any) error, collect bool) ([]domain.Violation, []located, error) {
	value, err := jsonschema.UnmarshalJSON(bytes.NewReader(content))
	if err != nil {
		return nil, nil, domain.Fail(domain.InvalidJSON, "", err.Error())
	}
	docs := map[string]any{}
	l := &loader{docs: docs}
	if c.lookup != nil {
		l.load = func(id string) ([]byte, error) { return c.lookup(ctx, id) }
	}
	into := &collector{target: schemaID, memo: map[memoKey]*outcome{}}
	compiler, err := newCompiler(l, into, docs)
	if err != nil {
		return nil, nil, err
	}
	if prepare != nil {
		if err := prepare(compiler, docs); err != nil {
			return nil, nil, err
		}
	}
	schema, err := compiler.Compile(schemaID)
	if err != nil {
		return nil, nil, compileError(err, l.err, docs)
	}
	into.active = collect
	err = schema.Validate(value)
	pairs := into.pairs(schema)
	if err == nil {
		return nil, pairs, nil
	}
	var verr *jsonschema.ValidationError
	if !errors.As(err, &verr) {
		return nil, nil, domain.Fail(domain.SchemaViolation, "", err.Error())
	}
	return leaves(verr, nil, nil, docs), pairs, nil
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
	violations, _, err := c.check(ctx, id, content, func(compiler *jsonschema.Compiler, docs map[string]any) error {
		docs[id] = value
		return compiler.AddResource(id, value)
	}, false)
	return violations, err
}

// newCompiler は、公開された言葉を resource にし、拡張項目の語彙を登録した Compiler を返す。
// サブスキーマは、検証で当たったことを into へ渡す。
func newCompiler(l *loader, into *collector, docs map[string]any) (*jsonschema.Compiler, error) {
	vocabulary, err := compileVocabulary()
	if err != nil {
		return nil, err
	}
	compiler := jsonschema.NewCompiler()
	// format を注釈でなく違反として検証する。Abstract が、ライブラリが確かめられる名前だけを書かせる
	compiler.AssertFormat()
	if err := addPublished(compiler, docs); err != nil {
		return nil, err
	}
	compiler.RegisterVocabulary(&jsonschema.Vocabulary{
		URL:    publishedlanguage.VocabularyID,
		Schema: vocabulary,
		// 拡張項目は検証の振る舞いを足さない。形は vocabulary.schema.json が決める。サブスキーマごとに、自分の *Schema（Location を持つ）を引いて、当たった位置を記録する
		Compile: func(ctx *jsonschema.CompilerContext, _ map[string]any) (jsonschema.SchemaExt, error) {
			return &applied{schema: ctx.Enqueue(nil), into: into}, nil
		},
	})
	compiler.UseLoader(l)
	return compiler, nil
}

func compileVocabulary() (*jsonschema.Schema, error) {
	compiler := jsonschema.NewCompiler()
	if err := addPublished(compiler, map[string]any{}); err != nil {
		return nil, err
	}
	return compiler.Compile(publishedlanguage.VocabularyID)
}

func addPublished(compiler *jsonschema.Compiler, docs map[string]any) error {
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
		docs[id] = value
		if err := compiler.AddResource(id, value); err != nil {
			return err
		}
	}
	return nil
}

// compileError は、Compile の error を返す。引けなかった $id があればその error、
// $ref の先の Concrete が Abstract に違反していれば、違反の葉ごとの schema_violation をまとめた error にする。
func compileError(err, loadErr error, docs map[string]any) error {
	if loadErr != nil {
		return loadErr
	}
	var serr *jsonschema.SchemaValidationError
	var verr *jsonschema.ValidationError
	if !errors.As(err, &serr) || !errors.As(serr.Err, &verr) {
		return domain.Fail(domain.SchemaViolation, "", err.Error())
	}
	var failures []error
	for _, v := range leaves(verr, nil, nil, docs) {
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
func leaves(e *jsonschema.ValidationError, enclosing, fixed []string, docs map[string]any) []domain.Violation {
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
		absolute := e.SchemaURL + domain.PointerOf(keywords...).String()
		text := message(e)
		if d := description(docs, absolute); d != "" {
			text += "（" + d + "）"
		}
		return []domain.Violation{{
			InstanceLocation:        domain.PointerOf(location...),
			AbsoluteKeywordLocation: absolute,
			Message:                 text,
		}}
	}
	var violations []domain.Violation
	for _, cause := range e.Causes {
		violations = append(violations, leaves(cause, slices.Clone(location), fixed, docs)...)
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

// description は、違反した keyword の絶対の位置（$id#JSON Pointer）の、keyword を直接持つサブスキーマの description を返す。
// 親のサブスキーマへは上らない。Abstract のサブスキーマでも Concrete のサブスキーマでも同じ。無ければ空。
func description(docs map[string]any, absolute string) string {
	id, fragment, _ := strings.Cut(absolute, "#")
	node, ok := docs[id]
	if !ok {
		return ""
	}
	if unescaped, err := url.PathUnescape(fragment); err == nil {
		fragment = unescaped
	}
	tokens := strings.Split(strings.TrimPrefix(fragment, "/"), "/")
	for _, token := range tokens[:len(tokens)-1] {
		if token == "" {
			continue
		}
		token = strings.NewReplacer("~1", "/", "~0", "~").Replace(token)
		switch parent := node.(type) {
		case map[string]any:
			node = parent[token]
		case []any:
			i, err := strconv.Atoi(token)
			if err != nil || i < 0 || i >= len(parent) {
				return ""
			}
			node = parent[i]
		default:
			return ""
		}
	}
	object, _ := node.(map[string]any)
	d, _ := object["description"].(string)
	return d
}
