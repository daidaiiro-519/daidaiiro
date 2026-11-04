// Package application は、中核の業務領域「作成と検証」のユースケース。
// ポートを通して読み書きし、判断はドメインモデルに任せる。
package application

import (
	"context"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Documents は、ポート。path のファイルを JSON ドキュメントとして読み書きする。アダプター files が満たす。
type Documents interface {
	Read(ctx context.Context, path domain.Path) (domain.Document, error)
	// Write は、一時ファイルに書いて置き換える。
	Write(ctx context.Context, path domain.Path, content []byte) error
}

// Schemas は、ポート。schemas の中から $id で Concrete を引く。アダプター files が満たす。
type Schemas interface {
	Lookup(ctx context.Context, id string) (domain.Concrete, error)
}

// Validator は、ポート。JSON をスキーマで機械的に検証し、JSON Schema の違反を返す。アダプター schemacheck が満たす。
type Validator interface {
	Check(ctx context.Context, schemaID string, content []byte) ([]domain.Violation, error)
	// CheckWith は、まだ書いていないスキーマ（更新した後の Concrete）のバイト列で検証する。
	CheckWith(ctx context.Context, schema, content []byte) ([]domain.Violation, error)
}

// Patcher は、ポート。JSON Patch を JSON ドキュメントに適用する。アダプター patching が満たす。
type Patcher interface {
	Apply(document, patch []byte) ([]byte, error)
}

// Records は、ポート。操作の記録から前回の検証を読み、検証と change を追記する。アダプター recordstore が満たす。
type Records interface {
	// WithLock は、書き込みのロックの中で f を行い、f が error を返さなければ追記を確定する。
	WithLock(ctx context.Context, f func(ctx context.Context) error) error
	// AppendChange は、WithLock の中で、書く操作の change を追記し、操作の ID を返す。
	AppendChange(ctx context.Context, w *domain.Write) (int64, error)
	// LastJudged は、path の前回の意味的な検証を返す。無ければ nil。
	LastJudged(ctx context.Context, path domain.Path) (*domain.Previous, error)
	AppendValidation(ctx context.Context, v *domain.Validation) (int64, error)
}
