package application

import (
	"context"
	"errors"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Search は、ユースケース「検索」。範囲の JSON ドキュメントから、条件か問いに合う instance location を得る。
// 担保：引数は機械的な検証（JSON Schema）、条件で探すことは機械的な検証（ドメインサービス「検索の比較」）、
// 問いの prompt は path とハッシュだけを渡し、問いで探すのは AI、matches の形は機械的な検証（JSON Schema）、
// matches の出どころが範囲の中でハッシュがいまと同じかの確かめは、この手続きが行う。
type Search struct {
	Documents  Documents
	Validator  Validator
	ResponseID string // matches のスキーマ（search-response.schema.json）の $id
}

// Match は、見つけた instance location 1つ。出どころの path と、その内容のハッシュを持つ。
type Match struct {
	Path             domain.Path
	Hash             domain.Hash
	InstanceLocation domain.Pointer
	Reason           string // 問いで探したときの、AI が書いた理由。条件で探したときは空
}

// Where は、範囲の各 JSON ドキュメントを読み、すべての条件を満たすものの、条件の instance location を返す。
func (u Search) Where(ctx context.Context, paths []domain.Path, conditions []domain.Condition) ([]Match, error) {
	var matches []Match
	for _, path := range paths {
		document, value, err := u.read(ctx, path)
		if err != nil {
			return nil, err
		}
		if !domain.Matches(value, conditions) {
			continue
		}
		for _, c := range conditions {
			matches = append(matches, Match{Path: path, Hash: document.Hash, InstanceLocation: c.InstanceLocation})
		}
	}
	return matches, nil
}

// Prompt は、範囲の各 JSON ドキュメントの path と内容のハッシュを返す。中身は貼らず、AI が path から読む。
func (u Search) Prompt(ctx context.Context, paths []domain.Path) ([]domain.Document, error) {
	documents := make([]domain.Document, 0, len(paths))
	for _, path := range paths {
		document, _, err := u.read(ctx, path)
		if err != nil {
			return nil, err
		}
		documents = append(documents, document)
	}
	return documents, nil
}

// Matches は、AI が問いで探して渡した matches を JSON Schema で検証し、出どころが範囲の中にあり、ハッシュがいまの
// 内容と同じで、instance location がその JSON ドキュメントにあるかを確かめて返す。
func (u Search) Matches(ctx context.Context, paths []domain.Path, content []byte) ([]Match, error) {
	value, err := domain.ParseJSON(content)
	if err != nil {
		return nil, domain.OfArgument(err, "--matches")
	}
	violations, err := u.Validator.Check(ctx, u.ResponseID, content)
	if err != nil {
		return nil, err
	}
	if err := domain.FailuresOf(domain.SchemaViolation, "", violations); err != nil {
		return nil, domain.OfArgument(err, "--matches")
	}
	inRange := map[string]bool{}
	for _, p := range paths {
		inRange[p.String()] = true
	}
	object, _ := value.(map[string]any) // 形は search-response.schema.json が決めている
	items, _ := object["matches"].([]any)
	matches := make([]Match, 0, len(items))
	var failures []error
	for _, item := range items {
		m, err := u.match(ctx, item, inRange)
		if err != nil {
			failures = append(failures, err)
			continue
		}
		matches = append(matches, m)
	}
	if err := errors.Join(failures...); err != nil {
		return nil, err
	}
	return matches, nil
}

// match は、matches の1件の出どころを確かめる。範囲の外か、instance location が無ければ response_mismatch、
// ハッシュが違えば hash_mismatch を返す。
func (u Search) match(ctx context.Context, item any, inRange map[string]bool) (Match, error) {
	m, _ := item.(map[string]any) // 形は search-response.schema.json が決めている
	pathText, _ := m["path"].(string)
	hashText, _ := m["hash"].(string)
	locationText, _ := m["instanceLocation"].(string)
	reason, _ := m["reason"].(string)
	if !inRange[pathText] {
		return Match{}, domain.Fail(domain.ResponseMismatch, pathText, pathText+" は範囲の外")
	}
	path, err := domain.NewPath(pathText)
	if err != nil {
		return Match{}, err
	}
	hash, err := domain.ParseHash(hashText)
	if err != nil {
		return Match{}, err
	}
	location, err := domain.NewPointer(locationText)
	if err != nil {
		return Match{}, err
	}
	document, value, err := u.read(ctx, path)
	if err != nil {
		return Match{}, err
	}
	if err := domain.MatchResponseHash(path, hash, document.Hash); err != nil {
		return Match{}, err
	}
	if _, ok := domain.ValueAt(value, location); !ok {
		return Match{}, &domain.Failure{
			Code: domain.ResponseMismatch, Path: pathText, InstanceLocation: locationText,
			Message: pathText + " に " + locationText + " が無い",
		}
	}
	return Match{Path: path, Hash: hash, InstanceLocation: location, Reason: reason}, nil
}

// read は、範囲の JSON ドキュメントを読む。JSON として読めなければ invalid_json を返す。
func (u Search) read(ctx context.Context, path domain.Path) (domain.Document, any, error) {
	document, err := u.Documents.Read(ctx, path)
	if err != nil {
		return domain.Document{}, nil, err
	}
	value, err := domain.ParseJSON(document.Content)
	if err != nil {
		var f *domain.Failure
		if errors.As(err, &f) {
			f.Path = path.String()
		}
		return domain.Document{}, nil, err
	}
	return document, value, nil
}
