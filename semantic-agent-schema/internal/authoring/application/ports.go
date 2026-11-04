// Package application は、中核の業務領域「作成と検証」のユースケース。
// ポートを通して読み書きし、判断はドメインモデルに任せる。
package application

import (
	"context"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Documents は、ポート。path のファイルを JSON ドキュメントとして読む。
type Documents interface {
	Read(ctx context.Context, path domain.Path) (domain.Document, error)
}

// Schemas は、ポート。schemas の中から $id で Concrete を引く。
type Schemas interface {
	Lookup(ctx context.Context, id string) (domain.Concrete, error)
}

// Validator は、ポート。JSON をスキーマで機械的に検証し、JSON Schema の違反を返す。
type Validator interface {
	Check(ctx context.Context, schemaID string, content []byte) ([]domain.Violation, error)
}

// Records は、ポート。操作の記録から前回の検証を読み、検証を追記する。
type Records interface {
	LastValidation(ctx context.Context, path domain.Path) (*domain.Previous, error)
	AppendValidation(ctx context.Context, v *domain.Validation) (int64, error)
}
