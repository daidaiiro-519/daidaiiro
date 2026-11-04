// Package application は、補完の業務領域「検索」のユースケース。トランザクションスクリプトで、1つのユースケースを
// 1つの手続きで最後まで行う。値の比べ方は、中核の業務領域のドメインサービス「検索の比較」を使う。
// 範囲は AI が渡す path の並びだけで、その外を探さない。JSON ドキュメントも操作の記録も変えない。
package application

import (
	"context"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Documents は、ポート。範囲の path のファイルを JSON ドキュメントとして読む。アダプター files が満たす。
type Documents interface {
	Read(ctx context.Context, path domain.Path) (domain.Document, error)
}

// Validator は、ポート。AI が渡した matches を JSON Schema で検証する。アダプター schemacheck が満たす。
type Validator interface {
	Check(ctx context.Context, schemaID string, content []byte) ([]domain.Violation, error)
}
