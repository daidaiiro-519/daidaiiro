package application

import (
	"context"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Log は、ユースケース「操作の記録の参照」。誰が（どのコマンドが）、いつ、何をしたかを返す。操作の記録は変えない。
// 担保：引数は機械的な検証（JSON Schema）、操作の記録を読むことは処理（アダプター）。
type Log struct {
	Records Records
}

// List は、絞る条件（path と件数）に合う操作を、新しい順に返す。
func (u Log) List(ctx context.Context, paths []domain.Path, limit int) ([]Operation, error) {
	return u.Records.Operations(ctx, paths, limit)
}

// One は、command と ID の操作を、その change か judgment と一緒に返す。無ければ nil を返す。
func (u Log) One(ctx context.Context, command string, id int64) (*Operation, error) {
	return u.Records.Operation(ctx, command, id)
}
