package application

import (
	"context"
	"errors"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Create は、ユースケース「作成」。Concrete に従うインスタンス（または Abstract に従う Concrete）を、渡した path に作る。
// 各ステップの担保：引数は機械的な検証（JSON Schema）、作成の prompt は意味的な検証（ドメインモデル）、
// 生成は AI、JSON ドキュメントの形と値は機械的な検証（JSON Schema）、作法は
// 機械的な検証（ドメインモデル）、ファイルを書くことと操作の記録への追記は処理（アダプター）。
type Create struct {
	Documents  Documents
	Schemas    Schemas
	Validator  Validator
	Records    Records
	AbstractID string // Abstract の $id。Abstract に従う Concrete を作るときは、作法も確かめる
}

// Prompt は、Concrete の x-prompt から作成の prompt を作って返す。
// $id の Concrete が schemas に無いか2つあれば、その error を返す。
func (u Create) Prompt(ctx context.Context, path domain.Path, schemaID string) (domain.Prompt, error) {
	concrete, err := u.Schemas.Lookup(ctx, schemaID)
	if err != nil {
		return nil, at(path, err)
	}
	return domain.NewCreatePrompt(path, concrete), nil
}

// Run は、AI が渡した JSON ドキュメントを検証し、path に書いて change を操作の記録に追記し、操作の ID と作成を返す。
// error のときは、ファイルも操作の記録も変えない。
func (u Create) Run(ctx context.Context, path domain.Path, content []byte) (int64, *domain.Write, error) {
	creation, err := u.check(ctx, path, content)
	if err != nil {
		return 0, nil, at(path, err)
	}
	var id int64
	err = u.Records.WithLock(ctx, func(ctx context.Context) error {
		if err := absent(ctx, u.Documents, path); err != nil {
			return err
		}
		// 追記を先に行い、確定はファイルを書けた後にする。書けなければ追記も取り消される
		if id, err = u.Records.AppendChange(ctx, creation); err != nil {
			return err
		}
		return u.Documents.Write(ctx, path, creation.Document().Content)
	})
	if err != nil {
		return 0, nil, at(path, err)
	}
	return id, creation, nil
}

// check は、JSON ドキュメントを Concrete で検証し、作法を確かめて、作成を返す。
func (u Create) check(ctx context.Context, path domain.Path, content []byte) (*domain.Write, error) {
	value, err := domain.ParseJSON(content)
	if err != nil {
		return nil, domain.OfArgument(err, "--json")
	}
	schemaID, ok := domain.SchemaIDOf(value)
	if !ok {
		return nil, &domain.Failure{
			Code: domain.SchemaNotFound, InstanceLocation: "/$schema",
			Message: "$schema に従う Concrete の $id が無い",
		}
	}
	concrete, err := u.Schemas.Lookup(ctx, schemaID)
	if err != nil {
		return nil, err
	}
	if schemaID != u.AbstractID {
		if err := satisfiesAbstract(ctx, u.Validator, u.AbstractID, concrete); err != nil {
			return nil, err
		}
	}
	violations, err := u.Validator.Check(ctx, schemaID, content)
	if err != nil {
		return nil, err
	}
	if err := domain.FailuresOf(domain.SchemaViolation, path.String(), violations); err != nil {
		return nil, domain.OfArgument(err, "--json")
	}
	if schemaID == u.AbstractID {
		if err := domain.FailuresOf(domain.ConventionViolation, path.String(), domain.Convention(value)); err != nil {
			return nil, err
		}
	}
	return domain.NewCreation(domain.WriteInput{
		Path: path, Content: content, Value: value, Concrete: concrete,
	})
}

// absent は、path にファイルが無ければ nil を返す。あれば file_exists の Failure を返す。
func absent(ctx context.Context, documents Documents, path domain.Path) error {
	_, err := documents.Read(ctx, path)
	if err == nil {
		return domain.Fail(domain.FileExists, path.String(), path.String()+" は既にある")
	}
	var f *domain.Failure
	if errors.As(err, &f) && f.Code == domain.FileNotFound {
		return nil
	}
	return err
}
