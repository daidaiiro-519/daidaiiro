package application

import (
	"context"
	"encoding/json"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Update は、ユースケース「更新」。JSON ドキュメント（インスタンスか Concrete）を JSON Patch で書き換える。
// 各ステップの担保：引数は機械的な検証（JSON Schema）、更新の prompt は意味的な検証（ドメインモデル）、
// JSON Patch の作成は AI、JSON Patch の形と、適用した結果の Concrete での検証と、Concrete を更新するときの範囲のインスタンスの
// 検証は機械的な検証（JSON Schema）、x-decided-by: human の値と楽観ロックのハッシュと作法は機械的な検証（ドメインモデル）、
// 追記と書き込みは処理（アダプター）。ハッシュの確認 ・ 追記 ・ 書き込みは、この順にロックの中で行う。
type Update struct {
	Documents  Documents
	Schemas    Schemas
	Validator  Validator
	Patcher    Patcher
	Records    Records
	AbstractID string // Abstract の $id。Concrete を更新するときは、作法と範囲のインスタンスも確かめる
	PatchID    string // JSON Patch のスキーマ（patch.schema.json）の $id。更新の prompt の response の形
}

// Prompt は、いまの JSON ドキュメントと Concrete から、更新の prompt を作って返す。いまの内容の直前の検証に violated の
// judgment があれば、直す対象として prompt に入れる。
func (u Update) Prompt(ctx context.Context, path domain.Path) (domain.Prompt, error) {
	document, value, concrete, err := u.read(ctx, path)
	if err != nil {
		return nil, at(path, err)
	}
	last, err := u.Records.LastValidation(ctx, path)
	if err != nil {
		return nil, at(path, err)
	}
	return domain.NewUpdatePrompt(domain.UpdateInput{
		Document: document, Value: value, Concrete: concrete, PatchID: u.PatchID,
		Violations: last.Violated(document.Hash), ForConcrete: concrete.ID() == u.AbstractID,
	}), nil
}

// UpdateInput は、更新の引数。
type UpdateInput struct {
	Path  domain.Path
	Patch []byte
	Range []domain.Path // Concrete を更新するときの、範囲のインスタンス
	// RangeGiven は、--range を渡したとき true。値なしの --range（Range が空）は、従うインスタンスが無いことを示す
	RangeGiven bool
}

// Run は、最初にいまのファイルを読んでハッシュを控え、JSON Patch を適用して検証し、ロックの中で書く直前のハッシュが
// 控えたハッシュと同じかを確かめてから（楽観ロック）、change を追記してファイルを書く。
// 操作の ID と書く操作を返す。error のときは、ファイルも操作の記録も変えない。
func (u Update) Run(ctx context.Context, in UpdateInput) (int64, *domain.Write, error) {
	write, err := u.check(ctx, in)
	if err != nil {
		return 0, nil, at(in.Path, err)
	}
	var id int64
	err = u.Records.WithLock(ctx, func(ctx context.Context) error {
		current, err := u.Documents.Read(ctx, in.Path)
		if err != nil {
			return err
		}
		if err := domain.MatchHash(in.Path, write.BeforeHash(), current.Hash); err != nil {
			return err
		}
		if id, err = u.Records.AppendChange(ctx, write); err != nil {
			return err
		}
		return u.Documents.Write(ctx, in.Path, write.Document().Content)
	})
	if err != nil {
		return 0, nil, at(in.Path, err)
	}
	return id, write, nil
}

// check は、JSON Patch を検証して適用し、結果を Concrete で検証して、書く操作を返す。
func (u Update) check(ctx context.Context, in UpdateInput) (*domain.Write, error) {
	before, err := u.Documents.Read(ctx, in.Path)
	if err != nil {
		return nil, err
	}
	beforeValue, err := domain.ParseJSON(before.Content)
	if err != nil {
		return nil, err
	}
	if _, err := domain.ParseJSON(in.Patch); err != nil {
		return nil, domain.OfArgument(err, "--patch")
	}
	violations, err := u.Validator.Check(ctx, u.PatchID, in.Patch)
	if err != nil {
		return nil, err
	}
	if err := domain.FailuresOf(domain.SchemaViolation, in.Path.String(), violations); err != nil {
		return nil, domain.OfArgument(err, "--patch")
	}
	content, err := u.apply(before.Content, in.Patch)
	if err != nil {
		return nil, err
	}
	value, concrete, err := u.satisfies(ctx, in, content)
	if err != nil {
		return nil, err
	}
	return domain.NewUpdate(domain.WriteInput{
		Path: in.Path, Content: content, Value: value, Concrete: concrete,
	}, before, beforeValue)
}

// apply は、JSON Patch の operation を順に1つずつ適用する。適用できない operation があれば、その path を
// instance location に持つ patch_not_applicable の Failure を返す。
func (u Update) apply(document, patch []byte) ([]byte, error) {
	value, err := domain.ParseJSON(patch)
	if err != nil {
		return nil, err
	}
	operations, _ := value.([]any) // 形は patch.schema.json が配列と決めている
	for _, operation := range operations {
		single, err := json.Marshal([]any{operation})
		if err != nil {
			return nil, err
		}
		if document, err = u.Patcher.Apply(document, single); err != nil {
			path, _ := operation.(map[string]any)["path"].(string) // 形は patch.schema.json が決めている
			return nil, &domain.Failure{Code: domain.PatchNotApplicable, InstanceLocation: path, Message: err.Error()}
		}
	}
	return document, nil
}

// satisfies は、適用した結果を Concrete で検証する。結果が Concrete なら、作法と、範囲のインスタンスを新しい Concrete で検証する。
func (u Update) satisfies(ctx context.Context, in UpdateInput, content []byte) (any, domain.Concrete, error) {
	value, err := domain.ParseJSON(content)
	if err != nil {
		return nil, nil, err
	}
	schemaID, ok := domain.SchemaIDOf(value)
	if !ok {
		return nil, nil, &domain.Failure{
			Code: domain.SchemaNotFound, InstanceLocation: "/$schema",
			Message: "$schema に従う Concrete の $id が無い",
		}
	}
	concrete, err := u.Schemas.Lookup(ctx, schemaID)
	if err != nil {
		return nil, nil, err
	}
	if schemaID != u.AbstractID {
		if err := satisfiesAbstract(ctx, u.Validator, u.AbstractID, concrete); err != nil {
			return nil, nil, err
		}
	}
	violations, err := u.Validator.Check(ctx, schemaID, content)
	if err != nil {
		return nil, nil, err
	}
	if err := domain.FailuresOf(domain.SchemaViolation, in.Path.String(), violations); err != nil {
		return nil, nil, err
	}
	if schemaID != u.AbstractID && in.RangeGiven {
		return nil, nil, &domain.Failure{
			Code: domain.ArgumentsInvalid, InstanceLocation: "/range",
			Message: "--range は Concrete を更新するときだけ渡す。" + in.Path.String() + " はインスタンス",
		}
	}
	if schemaID == u.AbstractID {
		if err := domain.FailuresOf(domain.ConventionViolation, in.Path.String(), domain.Convention(value)); err != nil {
			return nil, nil, err
		}
		concreteID, _ := value.(map[string]any)["$id"].(string) // 作法と Abstract を満たした Concrete は $id を持つ
		if err := u.rangeSatisfies(ctx, in, content, concreteID); err != nil {
			return nil, nil, err
		}
	}
	return value, concrete, nil
}

// rangeSatisfies は、Concrete を更新するときに、範囲のインスタンスを新しい Concrete で検証する。
// --range が無ければ range_required を返す。値なしの --range なら、従うインスタンスは無いので確かめない。インスタンスの $schema が Concrete の $id と違うときと、違反があるときは、
// そのインスタンスの path を持つ range_violation を返す。
func (u Update) rangeSatisfies(ctx context.Context, in UpdateInput, concrete []byte, concreteID string) error {
	if !in.RangeGiven {
		return domain.Fail(domain.RangeRequired, in.Path.String(), "Concrete の更新に --range が無い")
	}
	for _, path := range in.Range {
		instance, err := u.Documents.Read(ctx, path)
		if err != nil {
			return err
		}
		value, err := domain.ParseJSON(instance.Content)
		if err != nil {
			return at(path, err)
		}
		if id, _ := domain.SchemaIDOf(value); id != concreteID { // $schema が無ければ空の $id と比べて違う
			return &domain.Failure{
				Code: domain.RangeViolation, Path: path.String(), InstanceLocation: "/$schema",
				Message: path.String() + " の $schema " + id + " が、更新する Concrete の $id " + concreteID + " と違う",
			}
		}
		violations, err := u.Validator.CheckWith(ctx, concrete, instance.Content)
		if err != nil {
			return at(path, err)
		}
		if err := domain.FailuresOf(domain.RangeViolation, path.String(), violations); err != nil {
			return err
		}
	}
	return nil
}

// read は、JSON ドキュメントを読み、その $schema の Concrete を引く。
func (u Update) read(ctx context.Context, path domain.Path) (domain.Document, any, domain.Concrete, error) {
	document, err := u.Documents.Read(ctx, path)
	if err != nil {
		return domain.Document{}, nil, nil, err
	}
	value, err := domain.ParseJSON(document.Content)
	if err != nil {
		return domain.Document{}, nil, nil, err
	}
	schemaID, ok := domain.SchemaIDOf(value)
	if !ok {
		return domain.Document{}, nil, nil, &domain.Failure{
			Code: domain.SchemaNotFound, InstanceLocation: "/$schema",
			Message: "$schema に従う Concrete の $id が無い",
		}
	}
	concrete, err := u.Schemas.Lookup(ctx, schemaID)
	if err != nil {
		return domain.Document{}, nil, nil, err
	}
	return document, value, concrete, nil
}
