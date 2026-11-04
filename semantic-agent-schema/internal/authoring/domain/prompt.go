package domain

import "errors"

// commandName は、書く操作と prompt のコマンド。
type commandName string

// 書く操作と prompt のコマンド。
const (
	CreateCommand   commandName = "create"
	UpdateCommand   commandName = "update"
	ValidateCommand commandName = "validate"
)

// Step は、prompt の step 1つ。1つのサブスキーマの title ・ description ・ x-prompt を持つ。
// 位置は、作成ではスキーマの側の keyword location、更新と判定ではインスタンスの側の instance location で持ち、もう一方は nil。
// 判定では、判定する rule の id も持つ（step 1つが、rule と instance location の組1つ）。
type Step struct {
	KeywordLocation  Pointer
	InstanceLocation Pointer
	Title            string
	Description      string
	XPrompt          map[string]any
	Rule             string
}

// Prompt は、値オブジェクト prompt。コマンドが x-prompt から組んで AI に返す、生成の step と response の形を持つ。
// JSON ドキュメントの内容は貼らず、path で渡す。next は、受け取った引数を引き継ぐ入力側のアダプターが作る。
type Prompt interface {
	Command() commandName
	Path() Path
	Steps() []Step
	// Hash は、判定の prompt で、判定する JSON ドキュメントの内容のハッシュ。作成と更新では nil。
	Hash() Hash
	// ResponseSchemaID は、response の形：作成では AI が渡す JSON ドキュメントが従う Concrete の $id、更新では JSON Patch のスキーマの $id、
	// 判定では judgment-response のスキーマの $id。
	ResponseSchemaID() string
	prompt()
}

type generation struct {
	command    commandName
	path       Path
	hash       Hash
	steps      []Step
	responseID string
}

func (g generation) Command() commandName     { return g.command }
func (g generation) Path() Path               { return g.path }
func (g generation) Steps() []Step            { return g.steps }
func (g generation) Hash() Hash               { return g.hash }
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

// NewJudgmentPrompt は、ドメインサービス「prompt を作る」。いまの JSON ドキュメントと Concrete を一緒にたどり、判定の prompt を返す。
// x-prompt の rules の1件と、そのサブスキーマに当たる instance location の組ごとに step を作る。changed が nil なら
// すべての組を、そうでなければ instance location か rule の related_fields が変更点に含まれる組だけを入れる（ドメインサービス「変更点を求める」）。
// response の形は responseID（judgment-response のスキーマの $id）。JSON ドキュメントの内容は貼らず、path ・ 内容のハッシュ ・
// instance location で渡す。AI は judgments にハッシュを写し、受け取るときに、判定した内容がいまの内容と同じかを照らす。
func NewJudgmentPrompt(document Document, value any, concrete Concrete, changed map[string]bool, responseID string) Prompt {
	var steps []Step
	walkInstance(concrete.structure(), value, nil, func(sub map[string]any, _ any, at []string) {
		xPrompt, _ := sub["x-prompt"].(map[string]any) // 無ければ rule も無い
		location := PointerOf(at...)
		for _, item := range promptRules(sub) {
			ruleObject, _ := item.(map[string]any) // 形は Abstract が決める
			id, _ := ruleObject["id"].(string)
			rule, ok := concrete.Rule(id)
			if !ok || (changed != nil && !changed[location.String()] && !touches(rule, changed)) {
				continue
			}
			title, _ := sub["title"].(string)             // 無ければ空
			description, _ := sub["description"].(string) // 無ければ空
			steps = append(steps, Step{
				InstanceLocation: location, Title: title, Description: description, XPrompt: xPrompt, Rule: id,
			})
		}
	})
	return generation{command: ValidateCommand, path: document.Path, hash: document.Hash, steps: steps, responseID: responseID}
}

// MatchResponse は、集約「検証」の不変条件として、response の rule と instance location の組が、prompt の組と同じかを確かめる。
// 足りない組 ・ 余分な組 ・ 重なる組があれば、組ごとの response_mismatch の Failure をまとめて返す。
func MatchResponse(path Path, p Prompt, judgments []Judgment) error {
	want := map[string]bool{}
	for _, step := range p.Steps() {
		want[pairKey(step.Rule, step.InstanceLocation.String())] = true
	}
	var failures []error
	got := map[string]bool{}
	for _, j := range judgments {
		key := pairKey(j.Rule().ID(), j.InstanceLocation().String())
		switch {
		case got[key]:
			failures = append(failures, mismatch(path, j.InstanceLocation().String(), "rule "+j.Rule().ID()+" の judgment が2つある"))
		case !want[key]:
			failures = append(failures, mismatch(path, j.InstanceLocation().String(), "rule "+j.Rule().ID()+" は prompt に無い組"))
		}
		got[key] = true
	}
	for _, step := range p.Steps() {
		if !got[pairKey(step.Rule, step.InstanceLocation.String())] {
			failures = append(failures, mismatch(path, step.InstanceLocation.String(), "rule "+step.Rule+" の judgment が無い"))
		}
	}
	return errors.Join(failures...)
}

func pairKey(rule, location string) string { return rule + "\x00" + location }

func mismatch(path Path, location, message string) error {
	return &Failure{Code: ResponseMismatch, Path: path.String(), InstanceLocation: location, Message: message}
}
