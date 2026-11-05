// Package application は、補完の業務領域「描画」のユースケース。トランザクションスクリプトで、1つのユースケースを
// 1つの手続きで最後まで行う。テンプレートの選び方は、中核の業務領域のドメインサービス「レンダラー」を使う。
// 見た目は、依存の注入として使う側が渡すデザインシステムが決める。JSON ドキュメントも操作の記録も変えない。
package application

import (
	"context"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Documents は、ポート。範囲の path とデザインシステムのファイルを読む。アダプター files が満たす。
type Documents interface {
	Read(ctx context.Context, path domain.Path) (domain.Document, error)
}

// Schemas は、ポート。schemas の中から $id で Concrete を引く。アダプター files が満たす。
type Schemas interface {
	Lookup(ctx context.Context, id string) (domain.Concrete, error)
}

// Validator は、ポート。JSON をスキーマで機械的に検証する。アダプター schemacheck が満たす。
type Validator interface {
	Check(ctx context.Context, schemaID string, content []byte) ([]domain.Violation, error)
}

// Renderer は、ポート。テンプレートで描画し、全部が描けてから書き出す。描く前に、テンプレートが読む欄を集める。
// アダプター templates が満たす（テンプレートの構文木と JS の構文木は、アダプターが読む）。
type Renderer interface {
	// References は、テンプレートと、その中の <script> の JS が読む .Value と .Tokens の欄を集める。
	// テンプレートか JS の構文が誤っていれば template_invalid を返す。
	References(template domain.Template) ([]domain.Reference, error)
	Render(ctx context.Context, pages []domain.Page) error
}
