package cli

import (
	"encoding/json"
	"errors"
	"io"
	"regexp"
	"strconv"
	"strings"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
	record "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/record/application"
	rendering "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/rendering/application"
	search "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/search/application"
)

// output は、ログイベントを JSON の1行ずつ書く。instruction と next は error-codes.json の定型に値を入れて作る。
type output struct {
	encoder           *json.Encoder
	failed            bool
	codes             map[string]template
	violatedByMachine template
	violatedByAI      template
	promptNext        template
	createdNext       template
	updatedNext       template
}

// template は、error-codes.json の instruction の定型文と、コマンドごとの next の形。
type template struct {
	Instruction string            `json:"instruction"`
	Next        map[string]string `json:"next"`
}

// next は、コマンドの next の形を返す。semantic の有無と、--judgments ・ --matches の有無でも分け、無ければコマンド、次に * の形を使う。
func (t template) next(args arguments) string {
	key := args.Command
	switch {
	case args.Semantic:
		key = "semantic " + args.Command
	case args.Judgments != nil:
		key += " --judgments"
	case args.Matches != nil:
		key += " --matches"
	}
	for _, k := range []string{key, args.Command, "*"} {
		if next, ok := t.Next[k]; ok {
			return next
		}
	}
	return ""
}

func newOutput(w io.Writer) (*output, error) {
	content, err := publishedlanguage.Read("error-codes.json")
	if err != nil {
		return nil, err
	}
	var file struct {
		Codes     map[string]template `json:"codes"`
		ByMachine template            `json:"violated_by_machine"`
		ByAI      template            `json:"violated_by_ai"`
		Prompt    template            `json:"prompt"`
		Created   template            `json:"created"`
		Updated   template            `json:"updated"`
	}
	if err := json.Unmarshal(content, &file); err != nil {
		return nil, err
	}
	encoder := json.NewEncoder(w)
	encoder.SetEscapeHTML(false)
	return &output{encoder: encoder, codes: file.Codes, violatedByMachine: file.ByMachine, violatedByAI: file.ByAI, promptNext: file.Prompt, createdNext: file.Created, updatedNext: file.Updated}, nil
}

func (o *output) exit() int {
	if o.failed {
		return 1
	}
	return 0
}

type rejectedEvent struct {
	Event                   string `json:"event"`
	Code                    string `json:"code"`
	InstanceLocation        string `json:"instanceLocation"`
	AbsoluteKeywordLocation string `json:"absoluteKeywordLocation,omitempty"`
	Message                 string `json:"message"`
	Line                    int    `json:"line,omitempty"`
	Instruction             string `json:"instruction"`
	Next                    string `json:"next"`
}

// failures は、err を errors.Join の葉に分け、Failure ごとに rejected のログイベントを書く。
// Failure でない error と、書けなかった error を返す。
func (o *output) failures(args arguments, err error) error {
	o.failed = true
	var others []error
	for _, e := range leaves(err) {
		var f *domain.Failure
		if !errors.As(e, &f) {
			others = append(others, e)
			continue
		}
		values := fill(args, f.Path)
		values["{instanceLocation}"] = f.InstanceLocation
		values["{absoluteKeywordLocation}"] = f.AbsoluteKeywordLocation
		values["{message}"] = f.Message
		values["{line}"] = strconv.Itoa(f.Line)
		t := o.codes[string(f.Code)]
		if err := o.encoder.Encode(rejectedEvent{
			Event: "rejected", Code: string(f.Code), InstanceLocation: f.InstanceLocation,
			AbsoluteKeywordLocation: f.AbsoluteKeywordLocation, Message: f.Message, Line: f.Line,
			Instruction: replace(t.Instruction, values), Next: replace(t.next(args), values),
		}); err != nil {
			others = append(others, err)
		}
	}
	return errors.Join(others...)
}

// leaves は、errors.Join でまとめた error を1つずつに分ける。
func leaves(err error) []error {
	if joined, ok := err.(interface{ Unwrap() []error }); ok {
		var all []error
		for _, e := range joined.Unwrap() {
			all = append(all, leaves(e)...)
		}
		return all
	}
	return []error{err}
}

// fill は、定型の {command} ・ {path} ・ {paths} ・ {schemas} ・ {schema} ・ {range} ・ {all} ・ {limit} ・ {operation} ・ {designSystem} ・ {out} に入れる値を返す。
func fill(args arguments, path string) map[string]string {
	var schemas, paths []string
	for _, s := range args.Schemas {
		schemas = append(schemas, "--schemas "+quote(s))
	}
	for _, p := range args.Paths {
		paths = append(paths, quote(p))
	}
	limit, operation := "", ""
	if args.Limit != nil {
		limit = " --limit " + strconv.Itoa(*args.Limit)
	}
	if args.Operation != "" {
		operation = " --operation " + quote(args.Operation)
	}
	all := ""
	if args.All {
		all = " --all"
	}
	command := args.Command
	if args.Semantic {
		command = "semantic " + command
	}
	ranges := ""
	for _, r := range args.Range {
		ranges += " --range " + quote(r)
	}
	return map[string]string{
		"{command}":      command,
		"{path}":         quote(path),
		"{paths}":        strings.Join(paths, " "),
		"{schemas}":      strings.Join(schemas, " "),
		"{schema}":       quote(args.Schema),
		"{range}":        ranges,
		"{all}":          all,
		"{limit}":        limit,
		"{operation}":    operation,
		"{designSystem}": quote(args.DesignSystem),
		"{out}":          quote(args.Out),
	}
}

func replace(text string, values map[string]string) string {
	pairs := make([]string, 0, 2*len(values))
	for k, v := range values {
		pairs = append(pairs, k, v)
	}
	// 空の {…} で残る空白を除く：末尾の空白と、続いた空白
	return strings.TrimSpace(strings.ReplaceAll(strings.NewReplacer(pairs...).Replace(text), "  ", " "))
}

type judgmentEvent struct {
	Rule             string `json:"rule"`
	InstanceLocation string `json:"instanceLocation"`
	Verdict          string `json:"verdict"`
	By               string `json:"by"`
	Reason           string `json:"reason"`
}

type countsEvent struct {
	Verified   int `json:"verified"`
	Violated   int `json:"violated"`
	Unverified int `json:"unverified"`
	NotJudged  int `json:"not-judged"`
}

type validatedEvent struct {
	Event       string          `json:"event"`
	ID          int64           `json:"id"`
	Path        string          `json:"path"`
	SchemaID    string          `json:"schemaId"`
	ContentHash string          `json:"contentHash"`
	Counts      countsEvent     `json:"counts"`
	Judgments   []judgmentEvent `json:"judgments"`
	Instruction string          `json:"instruction,omitempty"`
	Next        string          `json:"next,omitempty"`
}

// validation は、validated（判定の response なら judged）のログイベントを書く。violated の judgment があれば、instruction と next を付け、終了コードを 1 にする。
func (o *output) validation(args arguments, id int64, v *domain.Validation) error {
	counts := v.Counts()
	name := "validated"
	if args.Judgments != nil {
		name = "judged"
	}
	event := validatedEvent{
		Event: name, ID: id, Path: v.Document().Path.String(),
		SchemaID: v.Concrete().ID(), ContentHash: v.Document().Hash.String(),
		Counts: countsEvent{
			Verified: counts.Verified, Violated: counts.Violated,
			Unverified: counts.Unverified, NotJudged: counts.NotJudged,
		},
		Judgments: []judgmentEvent{},
	}
	var violated []string
	byMachine := false
	for _, j := range v.Judgments() {
		event.Judgments = append(event.Judgments, judgmentEvent{
			Rule: j.Rule().ID(), InstanceLocation: j.InstanceLocation().String(),
			Verdict: string(j.Verdict()), By: string(j.By()), Reason: j.Reason(),
		})
		if j.Verdict() == domain.Violated {
			violated = append(violated, j.InstanceLocation().String())
			byMachine = byMachine || j.By() == domain.Machine
		}
	}
	if counts.Violated > 0 {
		o.failed = true
		// 機械的な検証の違反は位置と理由が分かっているので JSON Patch で直す。AI の判定の違反だけなら、
		// 同じ x-prompt の prompt に従って直すため、更新の prompt を依頼する
		t := o.violatedByAI
		if byMachine {
			t = o.violatedByMachine
		}
		values := fill(args, event.Path)
		values["{instanceLocation}"] = strings.Join(violated, `", "`) // 定型が引用符で囲むので、間を閉じて開く
		event.Instruction = replace(t.Instruction, values)
		event.Next = replace(t.next(args), values)
	}
	return o.encoder.Encode(event)
}

type stepEvent struct {
	KeywordLocation  string         `json:"keywordLocation,omitempty"`
	InstanceLocation *string        `json:"instanceLocation,omitempty"`
	Title            string         `json:"title,omitempty"`
	Description      string         `json:"description,omitempty"`
	XPrompt          map[string]any `json:"x-prompt,omitempty"`
	Rule             string         `json:"rule,omitempty"`
}

type promptEvent struct {
	Path     string            `json:"path"`
	Hash     string            `json:"hash,omitempty"`
	Steps    []stepEvent       `json:"steps"`
	Response map[string]string `json:"response"`
	Next     string            `json:"next"`
}

type requestedEvent struct {
	Event  string       `json:"event"`
	Prompt promptEvent  `json:"prompt"`
	Counts *countsEvent `json:"counts,omitempty"`
}

// requested は、prompt を持つログイベント（作成なら generation_requested、更新なら update_requested、判定なら
// judgment_requested）を書く。判定では、counted（追記しない検証）の counts も書く。
// prompt の next は、error-codes.json の prompt の形に、受け取った引数を入れて作る。
func (o *output) requested(args arguments, p domain.Prompt, counted *domain.Validation) error {
	prompt := promptEvent{
		Path: p.Path().String(), Steps: []stepEvent{},
		Response: map[string]string{"$ref": p.ResponseSchemaID()},
		Next:     replace(o.promptNext.next(args), fill(args, p.Path().String())),
	}
	if p.Hash() != nil {
		prompt.Hash = p.Hash().String()
	}
	for _, step := range p.Steps() {
		e := stepEvent{Title: step.Title, Description: step.Description, XPrompt: step.XPrompt, Rule: step.Rule}
		if step.KeywordLocation != nil {
			e.KeywordLocation = "#" + step.KeywordLocation.String()
		}
		if step.InstanceLocation != nil {
			location := step.InstanceLocation.String()
			e.InstanceLocation = &location
		}
		prompt.Steps = append(prompt.Steps, e)
	}
	var event string
	switch p.Command() {
	case domain.CreateCommand:
		event = "generation_requested"
	case domain.UpdateCommand:
		event = "update_requested"
	case domain.ValidateCommand:
		event = "judgment_requested"
	}
	requested := requestedEvent{Event: event, Prompt: prompt}
	if counted != nil {
		c := counted.Counts()
		requested.Counts = &countsEvent{Verified: c.Verified, Violated: c.Violated, Unverified: c.Unverified, NotJudged: c.NotJudged}
		o.failed = o.failed || c.Violated > 0
	}
	return o.encoder.Encode(requested)
}

type writtenEvent struct {
	Event       string `json:"event"`
	ID          int64  `json:"id"`
	Path        string `json:"path"`
	SchemaID    string `json:"schemaId"`
	ContentHash string `json:"contentHash"`
	Next        string `json:"next"`
}

// written は、書く操作のログイベント（作成なら created、更新なら updated）を書く。next は、書いたあとの次のアクション（意味的な検証の依頼）。
func (o *output) written(args arguments, id int64, w *domain.Write) error {
	path := w.Document().Path.String()
	event := writtenEvent{
		ID: id, Path: path, SchemaID: w.Concrete().ID(), ContentHash: w.Document().Hash.String(),
	}
	switch w.Command() {
	case domain.CreateCommand:
		event.Event = "created"
		event.Next = replace(o.createdNext.next(args), fill(args, path))
	case domain.UpdateCommand:
		event.Event = "updated"
		event.Next = replace(o.updatedNext.next(args), fill(args, path))
	case domain.ValidateCommand:
		// 書く操作のコマンドは create か update だけで、ここには来ない
	}
	return o.encoder.Encode(event)
}

var plain = regexp.MustCompile(`^[A-Za-z0-9_./:@%+=,-]+$`)

// quote は、シェルでそのまま渡せない値を単一引用符で囲む。
func quote(value string) string {
	if value == "" || plain.MatchString(value) {
		return value
	}
	return "'" + strings.ReplaceAll(value, "'", `'\''`) + "'"
}

type changeEvent struct {
	InstanceLocation string `json:"instanceLocation"`
	Op               string `json:"op"`
	Before           string `json:"before,omitempty"`
	After            string `json:"after,omitempty"`
}

type listedEvent struct {
	Event       string          `json:"event"`
	Command     string          `json:"command"`
	ID          int64           `json:"id"`
	Path        string          `json:"path"`
	At          string          `json:"at"`
	SchemaID    string          `json:"schemaId,omitempty"`
	BeforeHash  string          `json:"beforeHash,omitempty"`
	ContentHash string          `json:"contentHash,omitempty"`
	Validation  string          `json:"validation,omitempty"`
	Changes     []changeEvent   `json:"changes,omitempty"`
	Judgments   []judgmentEvent `json:"judgments,omitempty"`
}

// listed は、操作の記録の1件を listed のログイベントで書く。1件を指したときは、その change か judgment も書く。
func (o *output) listed(op record.Operation) error {
	event := listedEvent{Event: "listed", Command: op.Command, ID: op.ID, Path: op.Path, At: op.At}
	if d := op.Detail; d != nil {
		event.SchemaID, event.BeforeHash, event.ContentHash, event.Validation = d.SchemaID, d.BeforeHash, d.ContentHash, d.Validation
		for _, c := range d.Changes {
			event.Changes = append(event.Changes, changeEvent{InstanceLocation: c.InstanceLocation, Op: c.Op, Before: c.Before, After: c.After})
		}
		for _, j := range d.Judgments {
			event.Judgments = append(event.Judgments, judgmentEvent{
				Rule: j.Rule, InstanceLocation: j.InstanceLocation, Verdict: j.Verdict, By: j.By, Reason: j.Reason,
			})
		}
	}
	return o.encoder.Encode(event)
}

type documentEvent struct {
	Path string `json:"path"`
	Hash string `json:"hash"`
}

type searchPromptEvent struct {
	Question  string            `json:"question"`
	Documents []documentEvent   `json:"documents"`
	Response  map[string]string `json:"response"`
	Next      string            `json:"next"`
}

type searchRequestedEvent struct {
	Event  string            `json:"event"`
	Prompt searchPromptEvent `json:"prompt"`
}

// searchRequested は、問いで探す prompt を search_requested のログイベントで書く。範囲の path と内容のハッシュだけを渡す。
func (o *output) searchRequested(args arguments, documents []domain.Document) error {
	prompt := searchPromptEvent{
		Question: args.Question, Documents: []documentEvent{},
		Response: map[string]string{"$ref": publishedlanguage.SearchResponseID},
		Next:     replace(o.promptNext.next(args), fill(args, "")),
	}
	for _, d := range documents {
		prompt.Documents = append(prompt.Documents, documentEvent{Path: d.Path.String(), Hash: d.Hash.String()})
	}
	return o.encoder.Encode(searchRequestedEvent{Event: "search_requested", Prompt: prompt})
}

type matchEvent struct {
	Path             string `json:"path"`
	Hash             string `json:"hash"`
	InstanceLocation string `json:"instanceLocation"`
	Reason           string `json:"reason,omitempty"`
}

type foundEvent struct {
	Event   string       `json:"event"`
	Range   []string     `json:"range"`
	Matches []matchEvent `json:"matches"`
}

// found は、探した範囲と、見つけた instance location を found のログイベントで書く。
func (o *output) found(paths []domain.Path, matches []search.Match) error {
	event := foundEvent{Event: "found", Range: []string{}, Matches: []matchEvent{}}
	for _, p := range paths {
		event.Range = append(event.Range, p.String())
	}
	for _, m := range matches {
		event.Matches = append(event.Matches, matchEvent{
			Path: m.Path.String(), Hash: m.Hash.String(), InstanceLocation: m.InstanceLocation.String(), Reason: m.Reason,
		})
	}
	return o.encoder.Encode(event)
}

type renderedEvent struct {
	Event string `json:"event"`
	Path  string `json:"path"`
	Out   string `json:"out"`
}

// rendered は、書き出した HTML 1つを rendered のログイベントで書く。
func (o *output) rendered(r rendering.Rendered) error {
	return o.encoder.Encode(renderedEvent{Event: "rendered", Path: r.Path.String(), Out: r.Out.String()})
}
