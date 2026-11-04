package domain

// procedureCommand は、procedure を返すコマンド。
type procedureCommand string

// procedure を返すコマンド。
const (
	CreateCommand procedureCommand = "create"
)

// Step は、procedure の step 1つ。1つのサブスキーマの keyword location ・ title ・ description ・ x-prompt を持つ。
type Step struct {
	KeywordLocation Pointer
	Title           string
	Description     string
	Prompt          map[string]any
}

// Procedure は、値オブジェクト procedure。AI が従う、生成の step と answer の形を持つ。
// JSON ドキュメントの内容は貼らず、path で渡す。next は、受け取った引数を引き継ぐ入力側のアダプターが作る。
type Procedure interface {
	Command() procedureCommand
	Path() Path
	Steps() []Step
	// AnswerSchemaID は、answer の形：AI が渡す JSON ドキュメントが従う Concrete の $id。
	AnswerSchemaID() string
	procedure()
}

type generation struct {
	command  procedureCommand
	path     Path
	steps    []Step
	answerID string
}

func (g generation) Command() procedureCommand { return g.command }
func (g generation) Path() Path                { return g.path }
func (g generation) Steps() []Step             { return g.steps }
func (g generation) AnswerSchemaID() string    { return g.answerID }
func (generation) procedure()                  {}

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
