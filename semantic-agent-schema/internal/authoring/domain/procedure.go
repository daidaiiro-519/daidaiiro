package domain

// commandName は、書く操作と procedure のコマンド。
type commandName string

// 書く操作と procedure のコマンド。
const (
	CreateCommand commandName = "create"
	UpdateCommand commandName = "update"
)

// Step は、procedure の step 1つ。1つのサブスキーマの title ・ description ・ x-prompt を持つ。
// 位置は、作成ではスキーマの側の keyword location、更新ではインスタンスの側の instance location で持ち、もう一方は nil。
type Step struct {
	KeywordLocation  Pointer
	InstanceLocation Pointer
	Title            string
	Description      string
	Prompt           map[string]any
}

// Procedure は、値オブジェクト procedure。AI が従う、生成の step と answer の形を持つ。
// JSON ドキュメントの内容は貼らず、path で渡す。next は、受け取った引数を引き継ぐ入力側のアダプターが作る。
type Procedure interface {
	Command() commandName
	Path() Path
	// Hash は、更新する JSON ドキュメントのいまのハッシュ。作成では nil。
	Hash() Hash
	Steps() []Step
	// AnswerSchemaID は、answer の形：作成では AI が渡す JSON ドキュメントが従う Concrete の $id、更新では JSON Patch のスキーマの $id。
	AnswerSchemaID() string
	procedure()
}

type generation struct {
	command  commandName
	path     Path
	hash     Hash
	steps    []Step
	answerID string
}

func (g generation) Command() commandName   { return g.command }
func (g generation) Path() Path             { return g.path }
func (g generation) Hash() Hash             { return g.hash }
func (g generation) Steps() []Step          { return g.steps }
func (g generation) AnswerSchemaID() string { return g.answerID }
func (generation) procedure()               {}

// NewCreateProcedure は、ドメインサービス「procedure を作る」。Concrete のサブスキーマから、作成の procedure を返す。
// title ・ description ・ x-prompt のどれかを持つサブスキーマを、keyword location の順に step にする。同じ入力なら同じ procedure になる。
func NewCreateProcedure(path Path, concrete Concrete) Procedure {
	var steps []Step
	for _, sub := range concrete.Subschemas() {
		if sub.Title == "" && sub.Description == "" && sub.Prompt == nil {
			continue
		}
		steps = append(steps, Step{
			KeywordLocation: sub.KeywordLocation, Title: sub.Title,
			Description: sub.Description, Prompt: sub.Prompt,
		})
	}
	return generation{command: CreateCommand, path: path, steps: steps, answerID: concrete.ID()}
}

// NewUpdateProcedure は、ドメインサービス「procedure を作る」。いまの JSON ドキュメントと Concrete を一緒にたどり、
// 更新の procedure を返す。title ・ description ・ x-prompt のどれかを持つサブスキーマに当たる instance location を、
// たどった順に step にする。answer の形は patchID（JSON Patch のスキーマの $id）。JSON ドキュメントの内容は貼らず、path とハッシュで渡す。
func NewUpdateProcedure(document Document, value any, concrete Concrete, patchID string) Procedure {
	var steps []Step
	walkInstance(concrete.structure(), value, nil, func(sub map[string]any, _ any, at []string) {
		title, _ := sub["title"].(string)             // 無ければ空
		description, _ := sub["description"].(string) // 無ければ空
		prompt, _ := sub["x-prompt"].(map[string]any) // 無ければ nil
		if title == "" && description == "" && prompt == nil {
			return
		}
		steps = append(steps, Step{InstanceLocation: PointerOf(at...), Title: title, Description: description, Prompt: prompt})
	})
	return generation{command: UpdateCommand, path: document.Path, hash: document.Hash, steps: steps, answerID: patchID}
}
