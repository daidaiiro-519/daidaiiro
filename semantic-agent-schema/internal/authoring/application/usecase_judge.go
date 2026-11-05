package application

import (
	"context"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Judge は、ユースケース「意味的な検証」。x-prompt の rule に照らして変更点の意味を AI に判定させ、操作の記録に残す。
// 各ステップの担保：引数は機械的な検証（JSON Schema）、変更点を求めることは機械的な検証（ドメインモデル）、
// 判定の prompt は意味的な検証（ドメインモデル）、判定は AI、response の形は機械的な検証（JSON Schema）、
// response の組が prompt の組と同じかは機械的な検証（ドメインモデル）、追記は処理（アダプター）。
// prompt は保存しない。response を受け取ったときに、同じ JSON ドキュメントと前回の検証から同じ prompt を作り直して照らす。
type Judge struct {
	Documents  Documents
	Schemas    Schemas
	Validator  Validator
	Records    Records
	AbstractID string // Abstract の $id
	ResponseID string // 判定の response のスキーマ（judgment-response.schema.json）の $id
}

// JudgmentRequest は、判定の prompt を求めた結果。Prompt が nil なら判定する組が無く、判定は要らないので、Validation を
// 機械的な検証として操作の記録に追記した（ID）。それ以外は、Validation は counts を数えるための検証で、追記しない。
type JudgmentRequest struct {
	Prompt     domain.Prompt
	Validation *domain.Validation
	ID         int64
}

// Prompt は、機械的な検証をしてから、判定の prompt と、counts を数えるための検証（追記しない）を返す。
// all が false なら変更点に当たる rule だけを、true ならすべての rule とすべての instance location を判定の対象にする。
// 判定する組が無ければ prompt を返さず、検証を操作の記録に追記する。
func (u Judge) Prompt(ctx context.Context, path domain.Path, all bool) (JudgmentRequest, error) {
	c, previous, err := u.read(ctx, path, all)
	if err != nil {
		return JudgmentRequest{}, at(path, err)
	}
	prompt := domain.NewJudgmentPrompt(c.document, c.concrete, c.evaluations, changed(previous, c, all), u.ResponseID)
	validation := domain.NewValidation(c.input(previous))
	if len(prompt.Steps()) > 0 {
		return JudgmentRequest{Prompt: prompt, Validation: validation}, nil
	}
	id, err := u.Records.AppendValidation(ctx, validation)
	if err != nil {
		return JudgmentRequest{}, at(path, err)
	}
	return JudgmentRequest{Validation: validation, ID: id}, nil
}

// Run は、AI が渡した判定の response を検証し、prompt の組と同じなら、judgment と JSON ドキュメントを操作の記録に追記する。
// response が違反なら、操作の記録に追記しない。
func (u Judge) Run(ctx context.Context, path domain.Path, response []byte, all bool) (int64, *domain.Validation, error) {
	validation, err := u.judged(ctx, path, response, all)
	if err != nil {
		return 0, nil, at(path, err)
	}
	id, err := u.Records.AppendValidation(ctx, validation)
	if err != nil {
		return 0, nil, at(path, err)
	}
	return id, validation, nil
}

// judged は、判定の response を形と hash と prompt の組で照らし、AI の judgment を持つ検証を返す。追記はしない。
func (u Judge) judged(ctx context.Context, path domain.Path, response []byte, all bool) (*domain.Validation, error) {
	c, previous, err := u.read(ctx, path, all)
	if err != nil {
		return nil, err
	}
	value, err := domain.ParseJSON(response)
	if err != nil {
		return nil, domain.OfArgument(err, "--judgments")
	}
	violations, err := u.Validator.Check(ctx, u.ResponseID, response)
	if err != nil {
		return nil, err
	}
	if err := domain.FailuresOf(domain.SchemaViolation, path.String(), violations); err != nil {
		return nil, domain.OfArgument(err, "--judgments")
	}
	hash, judgments, err := responseJudgments(value)
	if err != nil {
		return nil, err
	}
	// 判定した内容が、いまの内容と同じかを照らす。prompt を返した後に変わっていれば、判定は古い内容に対するもの
	if err := domain.MatchResponseHash(path, hash, c.document.Hash); err != nil {
		return nil, err
	}
	prompt := domain.NewJudgmentPrompt(c.document, c.concrete, c.evaluations, changed(previous, c, all), u.ResponseID)
	if err := domain.MatchResponse(path, prompt, judgments); err != nil {
		return nil, err
	}
	in := c.input(previous)
	in.Judged = judgments
	return domain.NewValidation(in), nil
}

// read は、機械的な検証をし、前回の意味的な検証を返す。all なら、すべてを判定し直すので、引き継ぐ前回の検証は無い（nil）。
func (u Judge) read(ctx context.Context, path domain.Path, all bool) (checked, *domain.Previous, error) {
	c, err := mechanical(ctx, u.Documents, u.Schemas, u.Validator, u.AbstractID, path)
	if err != nil {
		return checked{}, nil, err
	}
	if all {
		return c, nil, nil
	}
	previous, err := u.Records.LastJudged(ctx, path)
	if err != nil {
		return checked{}, nil, err
	}
	return c, previous, nil
}

// changed は、判定の対象の変更点を返す。nil はすべての instance location。
func changed(previous *domain.Previous, c checked, all bool) map[string]bool {
	if all {
		return nil
	}
	return domain.Changed(previous, c.value, c.concrete)
}

// responseJudgments は、形を検証した judgments の JSON から、判定した内容のハッシュと AI の judgment を返す。
func responseJudgments(value any) (domain.Hash, []domain.Judgment, error) {
	object, _ := value.(map[string]any) // 形は judgment-response.schema.json が決めている
	hashText, _ := object["hash"].(string)
	hash, err := domain.ParseHash(hashText)
	if err != nil {
		return nil, nil, err
	}
	items, _ := object["judgments"].([]any)
	judgments := make([]domain.Judgment, 0, len(items))
	for _, item := range items {
		j, _ := item.(map[string]any)
		rule, _ := j["rule"].(string)
		location, _ := j["instanceLocation"].(string)
		verdict, _ := j["verdict"].(string)
		reason, _ := j["reason"].(string)
		judgment, err := domain.NewJudgment(rule, location, verdict, string(domain.ByAI), reason)
		if err != nil {
			return nil, nil, err
		}
		judgments = append(judgments, judgment)
	}
	return hash, judgments, nil
}
