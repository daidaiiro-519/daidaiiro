package domain

import (
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
// 更新と判定では、インスタンスの側の instance location と、当たるサブスキーマの keyword location を持ち、title ・ description ・
// x-prompt はサブスキーマごとに1回だけ置く（Prompt.Subschemas）。判定では、判定する rule の id も持つ
// （step 1つが、rule と instance location の組1つ）。
type Step struct {
	KeywordLocation  Pointer
	InstanceLocation Pointer
	Title            string
	Description      string
	XPrompt          map[string]any
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

func (x *subschemaIndex) add(keyword Pointer, sub map[string]any) {
	if x.seen == nil {
		x.seen = map[string]bool{}
	}
	if x.seen[keyword.String()] {
		return
	}
	x.seen[keyword.String()] = true
	title, _ := sub["title"].(string)              // 無ければ空
	description, _ := sub["description"].(string)  // 無ければ空
	xPrompt, _ := sub["x-prompt"].(map[string]any) // 無ければ nil
	x.list = append(x.list, Step{KeywordLocation: keyword, Title: title, Description: description, XPrompt: xPrompt})
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
// 更新の prompt を返す。title ・ description ・ x-prompt のどれかを持つサブスキーマに当たる instance location を、
// たどった順に step にし、サブスキーマは keyword location ごとに1回だけ置く。直す対象の violated の judgment も持つ。
// response の形は JSON Patch のスキーマ。JSON ドキュメントの内容は貼らず、path で渡す。
func NewUpdatePrompt(in UpdateInput) Prompt {
	var steps []Step
	var index subschemaIndex
	walkInstance(in.Concrete.structure(), in.Value, nil, nil, func(sub map[string]any, _ any, at, keyword []string) {
		title, _ := sub["title"].(string)              // 無ければ空
		description, _ := sub["description"].(string)  // 無ければ空
		xPrompt, _ := sub["x-prompt"].(map[string]any) // 無ければ nil
		if title == "" && description == "" && xPrompt == nil {
			return
		}
		k := PointerOf(keyword...)
		index.add(k, sub)
		steps = append(steps, Step{InstanceLocation: PointerOf(at...), KeywordLocation: k})
	})
	return generation{
		command: UpdateCommand, path: in.Document.Path, steps: steps, subschemas: index.list,
		responseID: in.PatchID, violations: in.Violations, forConcrete: in.ForConcrete,
	}
}

// NewJudgmentPrompt は、ドメインサービス「prompt を作る」。いまの JSON ドキュメントと Concrete を一緒にたどり、判定の prompt を返す。
// x-prompt の rules の1件と、そのサブスキーマに当たる instance location の組ごとに step を作る。changed が nil なら
// すべての組を、そうでなければ instance location か rule の related_fields が変更点に含まれる組だけを入れる（ドメインサービス「変更点を求める」）。
// forConcrete（JSON ドキュメントが Concrete で、concrete が Abstract）なら、Abstract のルートの rule を、Concrete の中の
// x-prompt の位置（ルートと、properties ・ items でたどるサブスキーマの …/x-prompt）ごとに当てる。
// サブスキーマは keyword location ごとに1回だけ置く。response の形は responseID（judgment-response のスキーマの $id）。
// JSON ドキュメントの内容は貼らず、path ・ 内容のハッシュ ・ instance location で渡す。
func NewJudgmentPrompt(document Document, value any, concrete Concrete, changed map[string]bool, responseID string, forConcrete bool) Prompt {
	var steps []Step
	var index subschemaIndex
	add := func(sub map[string]any, location, keyword Pointer) {
		for _, item := range promptRules(sub) {
			ruleObject, _ := item.(map[string]any) // 形は Abstract が決める
			id, _ := ruleObject["id"].(string)
			rule, ok := concrete.Rule(id)
			if !ok || (changed != nil && !changed[location.String()] && !touches(rule, changed)) {
				continue
			}
			index.add(keyword, sub)
			steps = append(steps, Step{InstanceLocation: location, KeywordLocation: keyword, Rule: id})
		}
	}
	if forConcrete {
		root := concrete.structure()
		document, _ := value.(map[string]any) // Concrete はオブジェクト（Abstract が決める）
		for _, location := range xPromptLocations(document) {
			add(root, location, PointerOf())
		}
	} else {
		walkInstance(concrete.structure(), value, nil, nil, func(sub map[string]any, _ any, at, keyword []string) {
			add(sub, PointerOf(at...), PointerOf(keyword...))
		})
	}
	return generation{
		command: ValidateCommand, path: document.Path, hash: document.Hash, steps: steps, subschemas: index.list,
		responseID: responseID, forConcrete: forConcrete,
	}
}

// xPromptLocations は、Concrete の中の x-prompt の位置（ルートと、properties ・ items でたどるサブスキーマの …/x-prompt）を返す。
// x-prompt を書いてよいのはこれらの位置だけ（Abstract が決める）なので、位置は機械的に決まる。
func xPromptLocations(root map[string]any) []Pointer {
	var subs []Subschema
	walkStructure(root, nil, &subs)
	var locations []Pointer
	for _, sub := range subs {
		if sub.XPrompt != nil {
			locations = append(locations, PointerOf(append(tokensOf(sub.KeywordLocation), "x-prompt")...))
		}
	}
	return locations
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
