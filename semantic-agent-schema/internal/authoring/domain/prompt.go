package domain

import (
	"encoding/json"
	"errors"
	"strings"
)

// commandName は、書く操作と prompt のコマンド。
type commandName string

// 書く操作と prompt のコマンド。
const (
	CreateCommand   commandName = "create"
	UpdateCommand   commandName = "update"
	ValidateCommand commandName = "validate"
)

// Step は、prompt の step 1つ。作成では、1つのサブスキーマの keyword location ・ title ・ description ・ x-prompt を持つ。
// 更新と判定では、インスタンスの側の instance location と、当たるサブスキーマの keyword location を持ち（更新で violated の
// judgment があれば、その instance location）、title ・ description ・
// x-prompt はサブスキーマごとに1回だけ置く（Prompt.Subschemas）。判定では、判定する rule の id も持つ
// （step 1つが、rule と instance location の組1つ）。
type Step struct {
	KeywordLocation  Pointer
	InstanceLocation Pointer
	Title            string
	Description      string
	XPrompt          json.RawMessage // スキーマに書いた順のまま
	Rule             string
}

// Prompt は、値オブジェクト prompt。コマンドが x-prompt から組んで AI に返す、step と response の形を持つ。
// JSON ドキュメントの内容は貼らず、path で渡す。next は、受け取った引数を引き継ぐ入力側のアダプターが作る。
type Prompt interface {
	Command() commandName
	Path() Path
	Steps() []Step
	// Subschemas は、更新と判定で、step が指すサブスキーマ（keyword location ごとに1回だけ）を返す。作成では nil。
	Subschemas() []Step
	// Hash は、判定の prompt で、判定する JSON ドキュメントの内容のハッシュ。作成と更新では nil。
	Hash() Hash
	// ResponseSchemaID は、response の形：作成では AI が渡す JSON ドキュメントが従う Concrete の $id、更新では JSON Patch のスキーマの $id、
	// 判定では judgment-response のスキーマの $id。
	ResponseSchemaID() string
	// ResponseSchema は、作成で、response の形（Concrete）の中身を返す。更新と判定では nil（公開された言葉のスキーマ）。
	ResponseSchema() map[string]any
	// Violations は、更新で、直す対象の violated の judgment を返す。
	Violations() []Judgment
	// ForConcrete は、対象の JSON ドキュメントが Concrete（Abstract に従う）なら true。
	ForConcrete() bool
	prompt()
}

type generation struct {
	command     commandName
	path        Path
	hash        Hash
	steps       []Step
	subschemas  []Step
	responseID  string
	response    map[string]any
	violations  []Judgment
	forConcrete bool
}

func (g generation) Command() commandName           { return g.command }
func (g generation) Path() Path                     { return g.path }
func (g generation) Steps() []Step                  { return g.steps }
func (g generation) Subschemas() []Step             { return g.subschemas }
func (g generation) Hash() Hash                     { return g.hash }
func (g generation) ResponseSchemaID() string       { return g.responseID }
func (g generation) ResponseSchema() map[string]any { return g.response }
func (g generation) Violations() []Judgment         { return g.violations }
func (g generation) ForConcrete() bool              { return g.forConcrete }
func (generation) prompt()                          {}

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
	return generation{command: CreateCommand, path: path, steps: steps, responseID: concrete.ID(), response: concrete.structure()}
}

// subschemaIndex は、keyword location ごとに1回だけサブスキーマを集める。
type subschemaIndex struct {
	seen map[string]bool
	list []Step
}

func (x *subschemaIndex) add(concrete Concrete, keyword Pointer) {
	if x.seen == nil {
		x.seen = map[string]bool{}
	}
	if x.seen[keyword.String()] {
		return
	}
	x.seen[keyword.String()] = true
	sub := subschemaAt(concrete, keyword)
	x.list = append(x.list, Step{KeywordLocation: keyword, Title: sub.Title, Description: sub.Description, XPrompt: sub.XPrompt})
}

// UpdateInput は、更新の prompt を作るために application が読んだもの。
type UpdateInput struct {
	Document    Document
	Value       any
	Concrete    Concrete
	PatchID     string
	Violations  []Judgment // 直す対象の violated の judgment（いまの内容の、直前の検証のもの）
	ForConcrete bool
}

// NewUpdatePrompt は、ドメインサービス「prompt を作る」。いまの JSON ドキュメントと Concrete を一緒にたどり、
// 更新の prompt を返す。直す対象の violated の judgment があれば、その instance location ごとに step を作り、keyword location は
// その rule を置いたサブスキーマ（x-prompt の rule でなければ、その位置か、たどれる一番深い親の位置のサブスキーマ）にする。violated の judgment が無ければ、title ・ description ・ x-prompt の
// どれかを持つサブスキーマに当たる instance location を、たどった順に step にする。サブスキーマは keyword location ごとに1回だけ置く。
// response の形は JSON Patch のスキーマ。JSON ドキュメントの内容は貼らず、path で渡す。
func NewUpdatePrompt(in UpdateInput) Prompt {
	var steps []Step
	var index subschemaIndex
	keywords := map[string]Pointer{}
	walkInstance(in.Concrete.structure(), in.Value, nil, nil, func(sub map[string]any, _ any, at, keyword []string) {
		k := PointerOf(keyword...)
		keywords[PointerOf(at...).String()] = k
		title, _ := sub["title"].(string)              // 無ければ空
		description, _ := sub["description"].(string)  // 無ければ空
		xPrompt, _ := sub["x-prompt"].(map[string]any) // 無ければ nil
		if len(in.Violations) > 0 || (title == "" && description == "" && xPrompt == nil) {
			return
		}
		index.add(in.Concrete, k)
		steps = append(steps, Step{InstanceLocation: PointerOf(at...), KeywordLocation: k})
	})
	seen := map[string]bool{}
	for _, v := range in.Violations {
		location := v.InstanceLocation()
		if seen[location.String()] {
			continue
		}
		seen[location.String()] = true
		// AI の判定の違反は、その rule を置いたサブスキーマ。機械的な検証の違反は、その位置か一番深い親のサブスキーマ
		k, ok := in.Concrete.ruleLocation(v.Rule().ID())
		if !ok {
			k = keywordAt(keywords, location)
		}
		index.add(in.Concrete, k)
		steps = append(steps, Step{InstanceLocation: location, KeywordLocation: k})
	}
	return generation{
		command: UpdateCommand, path: in.Document.Path, steps: steps, subschemas: index.list,
		responseID: in.PatchID, violations: in.Violations, forConcrete: in.ForConcrete,
	}
}

// keywordAt は、instance location か、その一番深い親のうち、たどれた位置の keyword location を返す。ルートは必ずたどれる。
func keywordAt(keywords map[string]Pointer, location Pointer) Pointer {
	tokens := tokensOf(location)
	for i := len(tokens); i > 0; i-- {
		if k, ok := keywords[PointerOf(tokens[:i]...).String()]; ok {
			return k
		}
	}
	return PointerOf()
}

// NewJudgmentPrompt は、ドメインサービス「prompt を作る」。機械的な検証で x-prompt を持つサブスキーマが当たったインスタンスの
// 位置の組（evaluations）から、判定の prompt を返す。そのサブスキーマの x-prompt の rules の1件と、当たった instance location の
// 組ごとに step を作る。インスタンスでも Concrete（concrete が Abstract）でも同じ決め方で、どの rule も、それを置いたサブスキーマが
// 当たる位置にだけ当たる。changed が nil ならすべての組を、そうでなければ instance location か rule の related_fields が変更点に
// 含まれる組だけを入れる（ドメインサービス「変更点を求める」）。サブスキーマは keyword location ごとに1回だけ置く。
// response の形は responseID（judgment-response のスキーマの $id）。JSON ドキュメントの内容は貼らず、path ・ 内容のハッシュ ・
// instance location で渡す。
func NewJudgmentPrompt(document Document, concrete Concrete, evaluations []Evaluation, changed map[string]bool, responseID string) Prompt {
	steps, subschemas := judgmentSteps(concrete, evaluations, changed)
	return generation{
		command: ValidateCommand, path: document.Path, hash: document.Hash, steps: steps, subschemas: subschemas,
		responseID: responseID,
	}
}

// judgmentSteps は、判定する rule と instance location の組を step で返し、step が指すサブスキーマを keyword location ごとに
// 1回だけ返す。changed が nil ならすべての組を、それ以外は instance location か rule の related_fields が変更点に当たる組を返す。
func judgmentSteps(concrete Concrete, evaluations []Evaluation, changed map[string]bool) ([]Step, []Step) {
	var steps []Step
	var index subschemaIndex
	for _, e := range sortedEvaluations(evaluations) {
		for _, item := range promptRules(schemaAt(concrete.structure(), tokensOf(e.KeywordLocation))) {
			ruleObject, _ := item.(map[string]any) // 形は Abstract が決める
			id, _ := ruleObject["id"].(string)
			rule, ok := concrete.Rule(id)
			if !ok || (changed != nil && !changed[e.InstanceLocation.String()] && !touches(rule, changed)) {
				continue
			}
			index.add(concrete, e.KeywordLocation)
			steps = append(steps, Step{InstanceLocation: e.InstanceLocation, KeywordLocation: e.KeywordLocation, Rule: id})
		}
	}
	return steps, index.list
}

// tokensOf は、JSON Pointer を、エスケープを戻した参照トークンの並びにする。
func tokensOf(p Pointer) []string {
	if p.String() == "" {
		return nil
	}
	var tokens []string
	for _, token := range strings.Split(p.String()[1:], "/") {
		tokens = append(tokens, strings.ReplaceAll(strings.ReplaceAll(token, "~1", "/"), "~0", "~"))
	}
	return tokens
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
