package application

import (
	"context"
	"errors"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Validate は、ユースケース「検証」。範囲の JSON ドキュメントが、スキーマと不変条件を満たすか、
// 意味的な検証がどこまで済んでいるかを、JSON ドキュメントごとの counts で返す。
type Validate struct {
	Documents  Documents
	Schemas    Schemas
	Validator  Validator
	Records    Records
	AbstractID string // Abstract の $id。Concrete に従うインスタンスと、Abstract に従う Concrete を分ける
}

// ValidateResult は、JSON ドキュメント1つの結果。Err が nil なら、Validation を操作の記録に追記した。
type ValidateResult struct {
	ID         int64
	Validation *domain.Validation
	Err        error
}

// Run は、範囲の JSON ドキュメントを1つずつ検証し、JSON ドキュメントごとの結果を返す。
func (u Validate) Run(ctx context.Context, paths []domain.Path) []ValidateResult {
	results := make([]ValidateResult, 0, len(paths))
	for _, path := range paths {
		id, validation, err := u.one(ctx, path)
		results = append(results, ValidateResult{ID: id, Validation: validation, Err: at(path, err)})
	}
	return results
}

func (u Validate) one(ctx context.Context, path domain.Path) (int64, *domain.Validation, error) {
	c, err := mechanical(ctx, u.Documents, u.Schemas, u.Validator, u.AbstractID, path)
	if err != nil {
		return 0, nil, err
	}
	previous, err := u.Records.LastJudged(ctx, path)
	if err != nil {
		return 0, nil, err
	}
	validation := domain.NewValidation(c.input(previous))
	id, err := u.Records.AppendValidation(ctx, validation)
	if err != nil {
		return 0, nil, err
	}
	return id, validation, nil
}

// checked は、機械的な検証の結果。JSON ドキュメント ・ その値 ・ Concrete ・ JSON Schema の違反 ・ 作法の違反を持つ。
type checked struct {
	document    domain.Document
	value       any
	concrete    domain.Concrete
	violations  []domain.Violation
	convention  []domain.Violation
	forConcrete bool // JSON ドキュメントが Abstract に従う Concrete か
}

// input は、検証を作る入力を返す。previous は、引き継ぐ AI の judgment の出どころ（無ければ nil）。
func (c checked) input(previous *domain.Previous) domain.ValidationInput {
	return domain.ValidationInput{
		Document: c.document, Value: c.value, Concrete: c.concrete,
		Violations: c.violations, Convention: c.convention, Previous: previous, ForConcrete: c.forConcrete,
	}
}

// mechanical は、JSON ドキュメントを読み、機械的な検証をする。インスタンスを Concrete で、Concrete を Abstract で
// 検証し（JSON Schema）、JSON ドキュメントが Concrete なら作法を確かめる（ドメインモデル）。違反は error にせず、結果に持つ。
func mechanical(ctx context.Context, documents Documents, schemas Schemas, validator Validator, abstractID string, path domain.Path) (checked, error) {
	document, err := documents.Read(ctx, path)
	if err != nil {
		return checked{}, err
	}
	value, err := domain.ParseJSON(document.Content)
	if err != nil {
		return checked{}, err
	}
	schemaID, ok := domain.SchemaIDOf(value)
	if !ok {
		return checked{}, &domain.Failure{
			Code: domain.SchemaNotFound, InstanceLocation: "/$schema",
			Message: "$schema に従う Concrete の $id が無い",
		}
	}
	concrete, err := schemas.Lookup(ctx, schemaID)
	if err != nil {
		return checked{}, err
	}
	// インスタンスを検証する前に、その Concrete を Abstract で検証する。違反は Concrete のファイルの error にする
	if schemaID != abstractID {
		if err := satisfiesAbstract(ctx, validator, abstractID, concrete); err != nil {
			return checked{}, err
		}
	}
	violations, err := validator.Check(ctx, schemaID, document.Content)
	if err != nil {
		return checked{}, err
	}
	var convention []domain.Violation
	if schemaID == abstractID {
		convention = domain.Convention(value)
	}
	return checked{
		document: document, value: value, concrete: concrete, violations: violations, convention: convention,
		forConcrete: schemaID == abstractID,
	}, nil
}

// satisfiesAbstract は、機械的な検証（JSON Schema）。Concrete を Abstract で検証し、違反があれば Concrete の path を持つ
// schema_violation の error を返す。
func satisfiesAbstract(ctx context.Context, validator Validator, abstractID string, concrete domain.Concrete) error {
	violations, err := validator.Check(ctx, abstractID, concrete.Content())
	if err != nil {
		return err
	}
	var path string
	if concrete.Path() != nil {
		path = concrete.Path().String()
	}
	return domain.FailuresOf(domain.SchemaViolation, path, violations)
}

// at は、path の無い Failure に、検証している JSON ドキュメントの path を入れる。
func at(path domain.Path, err error) error {
	if err == nil {
		return nil
	}
	for _, e := range leaves(err) {
		var f *domain.Failure
		if errors.As(e, &f) && f.Path == "" {
			f.Path = path.String()
		}
	}
	return err
}

// leaves は、errors.Join でまとめた error を1つずつに分ける。
func leaves(err error) []error {
	if joined, ok := err.(interface{ Unwrap() []error }); ok {
		var all []error
		for _, e := range joined.Unwrap() {
			all = append(all, leaves(e)...)
		}
		return all
	}
	return []error{err}
}
