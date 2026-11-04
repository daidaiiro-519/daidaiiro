// Package application は、補完の業務領域「操作の記録の参照」のユースケース。トランザクションスクリプトで、
// 1つのユースケースを1つの手続きで最後まで行う。ドメインモデルは持たない（守る不変条件が無い。読むだけ）。
package application

import (
	"context"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// Records は、ポート。操作の記録を読む。アダプター recordstore が満たす。
type Records interface {
	// Operations は、path（空ならすべて）の操作を新しい順に、limit 件（0 なら全部）まで返す。
	Operations(ctx context.Context, paths []domain.Path, limit int) ([]Operation, error)
	// Operation は、command と ID の操作を、その change か judgment と一緒に返す。無ければ nil。
	Operation(ctx context.Context, command string, id int64) (*Operation, error)
}

// Operation は、操作の記録の1件。ビュー operations の列と、1件を指したときの中身を持つ。
type Operation struct {
	ID      int64
	Command string // create ・ update ・ validate
	Path    string
	At      string
	Detail  *Detail // 1件を指したときだけ
}

// Detail は、操作の中身。書く操作（create ・ update）なら change、検証（validate）なら judgment を持つ。
type Detail struct {
	SchemaID    string
	BeforeHash  string // 書く操作の、書く前のハッシュ。作成では空
	ContentHash string // 書いた後か、検証した内容のハッシュ
	Validation  string // 検証の記録の種類：validated か judged。書く操作では空
	Changes     []Change
	Judgments   []Judgment
}

// Change は、書く操作の中の、1つの instance location の前後の値。値は JSON の文字列で、無ければ空。
type Change struct {
	InstanceLocation, Op, Before, After string
}

// Judgment は、検証の中の、1つの rule と1つの instance location の組の結果。
type Judgment struct {
	Rule, InstanceLocation, Verdict, By, Reason string
}
