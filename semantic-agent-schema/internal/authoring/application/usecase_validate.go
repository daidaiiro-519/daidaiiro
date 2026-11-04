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
	document, err := u.Documents.Read(ctx, path)
	if err != nil {
		return 0, nil, err
	}
	value, err := domain.ParseJSON(document.Content)
	if err != nil {
		return 0, nil, err
	}
	schemaID, ok := domain.SchemaIDOf(value)
	if !ok {
		return 0, nil, &domain.Failure{
			Code: domain.SchemaNotFound, InstanceLocation: "/$schema",
			Message: "$schema に従う Concrete の $id が無い",
		}
	}
	concrete, err := u.Schemas.Lookup(ctx, schemaID)
	if err != nil {
		return 0, nil, err
	}
	// インスタンスを検証する前に、その Concrete を Abstract で検証する。違反は Concrete のファイルの error にする
	if schemaID != u.AbstractID {
		if err := satisfiesAbstract(ctx, u.Validator, u.AbstractID, concrete); err != nil {
			return 0, nil, err
		}
	}
	// 機械的な検証（JSON Schema）。違反は error にせず、violated の judgment として数える
	violations, err := u.Validator.Check(ctx, schemaID, document.Content)
	if err != nil {
		return 0, nil, err
	}
	// 機械的な検証（ドメインモデル）。JSON ドキュメントが Concrete なら、作法を満たすかを確かめる
	var convention []domain.Violation
	if schemaID == u.AbstractID {
		convention = domain.Convention(value)
	}
	previous, err := u.Records.LastValidation(ctx, path)
	if err != nil {
		return 0, nil, err
	}
	validation := domain.NewValidation(domain.ValidationInput{
		Document: document, Value: value, Concrete: concrete,
		Violations: violations, Convention: convention, Previous: previous,
	})
	id, err := u.Records.AppendValidation(ctx, validation)
	if err != nil {
		return 0, nil, err
	}
	return id, validation, nil
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
