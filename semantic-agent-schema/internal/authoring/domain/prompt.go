package domain

// commandName は、書く操作と prompt のコマンド。
type commandName string

// 書く操作と prompt のコマンド。
const (
	CreateCommand commandName = "create"
	UpdateCommand commandName = "update"
)

// Step は、prompt の step 1つ。1つのサブスキーマの title ・ description ・ x-prompt を持つ。
// 位置は、作成ではスキーマの側の keyword location、更新ではインスタンスの側の instance location で持ち、もう一方は nil。
type Step struct {
	KeywordLocation  Pointer
	InstanceLocation Pointer
	Title            string
	Description      string
	XPrompt          map[string]any
}

// Prompt は、値オブジェクト prompt。コマンドが x-prompt から組んで AI に返す、生成の step と response の形を持つ。
// JSON ドキュメントの内容は貼らず、path で渡す。next は、受け取った引数を引き継ぐ入力側のアダプターが作る。
type Prompt interface {
	Command() commandName
	Path() Path
	Steps() []Step
	// ResponseSchemaID は、response の形：作成では AI が渡す JSON ドキュメントが従う Concrete の $id、更新では JSON Patch のスキーマの $id。
	ResponseSchemaID() string
	prompt()
}

type generation struct {
	command    commandName
	path       Path
	steps      []Step
	responseID string
}

func (g generation) Command() commandName     { return g.command }
func (g generation) Path() Path               { return g.path }
func (g generation) Steps() []Step            { return g.steps }
func (g generation) ResponseSchemaID() string { return g.responseID }
func (generation) prompt()                    {}

// NewCreatePrompt は、ドメインサービス「prompt を作る」。Concrete のサブスキーマの x-prompt から、作成の prompt を返す。
// title ・ description ・ x-prompt のどれかを持つサブスキーマを、keyword location の順に step にする。同じ入力なら同じ prompt になる。
func NewCreatePrompt(path Path, concrete Concrete) Prompt {
	var steps []Step
	for _, sub := range concrete.Subschemas() {
		if sub.Title == "" && sub.Description == "" && sub.XPrompt == nil {
			continue
		}
		steps = append(steps, Step{
			KeywordLocation: sub.KeywordLocation, Title: sub.Title,
			Description: sub.Description, XPrompt: sub.XPrompt,
		})
	}
	return generation{command: CreateCommand, path: path, steps: steps, responseID: concrete.ID()}
}

// NewUpdatePrompt は、ドメインサービス「prompt を作る」。いまの JSON ドキュメントと Concrete を一緒にたどり、
// 更新の prompt を返す。title ・ description ・ x-prompt のどれかを持つサブスキーマに当たる instance location を、
// たどった順に step にする。response の形は patchID（JSON Patch のスキーマの $id）。JSON ドキュメントの内容は貼らず、path で渡す。
func NewUpdatePrompt(document Document, value any, concrete Concrete, patchID string) Prompt {
	var steps []Step
	walkInstance(concrete.structure(), value, nil, func(sub map[string]any, _ any, at []string) {
		title, _ := sub["title"].(string)              // 無ければ空
		description, _ := sub["description"].(string)  // 無ければ空
		xPrompt, _ := sub["x-prompt"].(map[string]any) // 無ければ nil
		if title == "" && description == "" && xPrompt == nil {
			return
		}
		steps = append(steps, Step{InstanceLocation: PointerOf(at...), Title: title, Description: description, XPrompt: xPrompt})
	})
	return generation{command: UpdateCommand, path: document.Path, steps: steps, responseID: patchID}
}
