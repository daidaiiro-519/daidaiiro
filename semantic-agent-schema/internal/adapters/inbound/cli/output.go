package cli

import (
	"encoding/json"
	"errors"
	"io"
	"regexp"
	"strings"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
)

// output は、ログイベントを JSON の1行ずつ書く。instruction と next は error-codes.json の定型に値を入れて作る。
type output struct {
	encoder  *json.Encoder
	failed   bool
	codes    map[string]template
	violated template
}

// template は、error-codes.json の instruction の定型文と next の形。
type template struct {
	Instruction string `json:"instruction"`
	Next        string `json:"next"`
}

func newOutput(w io.Writer) (*output, error) {
	content, err := publishedlanguage.Read("error-codes.json")
	if err != nil {
		return nil, err
	}
	var file struct {
		Codes    map[string]template `json:"codes"`
		Violated template            `json:"violated"`
	}
	if err := json.Unmarshal(content, &file); err != nil {
		return nil, err
	}
	encoder := json.NewEncoder(w)
	encoder.SetEscapeHTML(false)
	return &output{encoder: encoder, codes: file.Codes, violated: file.Violated}, nil
}

func (o *output) exit() int {
	if o.failed {
		return 1
	}
	return 0
}

type errorEvent struct {
	Event                   string `json:"event"`
	Code                    string `json:"code"`
	InstanceLocation        string `json:"instanceLocation,omitempty"`
	AbsoluteKeywordLocation string `json:"absoluteKeywordLocation,omitempty"`
	Message                 string `json:"message"`
	Instruction             string `json:"instruction"`
	Next                    string `json:"next"`
}

// failures は、err を errors.Join の葉に分け、Failure ごとに error のログイベントを書く。
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
		values := fill(args, f.Path, "")
		values["{instanceLocation}"] = f.InstanceLocation
		values["{absoluteKeywordLocation}"] = f.AbsoluteKeywordLocation
		values["{message}"] = f.Message
		t := o.codes[string(f.Code)]
		if err := o.encoder.Encode(errorEvent{
			Event: "error", Code: string(f.Code), InstanceLocation: f.InstanceLocation,
			AbsoluteKeywordLocation: f.AbsoluteKeywordLocation, Message: f.Message,
			Instruction: replace(t.Instruction, values), Next: replace(t.Next, values),
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

// fill は、定型の {command} ・ {path} ・ {paths} ・ {schemas} ・ {human} ・ {hash} に入れる値を返す。
func fill(args arguments, path, hash string) map[string]string {
	var schemas, paths []string
	for _, s := range args.Schemas {
		schemas = append(schemas, "--schemas "+quote(s))
	}
	for _, p := range args.Paths {
		paths = append(paths, quote(p))
	}
	human := ""
	if args.Human {
		human = " --human"
	}
	return map[string]string{
		"{command}": args.Command,
		"{path}":    quote(path),
		"{paths}":   strings.Join(paths, " "),
		"{schemas}": strings.Join(schemas, " "),
		"{human}":   human,
		"{hash}":    hash,
	}
}

func replace(text string, values map[string]string) string {
	pairs := make([]string, 0, 2*len(values))
	for k, v := range values {
		pairs = append(pairs, k, v)
	}
	return strings.NewReplacer(pairs...).Replace(text)
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

type validationEvent struct {
	Event       string          `json:"event"`
	ID          int64           `json:"id"`
	Actor       string          `json:"actor"`
	Path        string          `json:"path"`
	SchemaID    string          `json:"schemaId"`
	ContentHash string          `json:"contentHash"`
	Counts      countsEvent     `json:"counts"`
	Judgments   []judgmentEvent `json:"judgments"`
	Instruction string          `json:"instruction,omitempty"`
	Next        string          `json:"next,omitempty"`
}

// validation は、検証のログイベントを書く。violated の judgment があれば、instruction と next を付け、終了コードを 1 にする。
func (o *output) validation(args arguments, id int64, v *domain.Validation) error {
	counts := v.Counts()
	event := validationEvent{
		Event: "validation", ID: id, Actor: string(v.Actor().Kind()), Path: v.Document().Path.String(),
		SchemaID: v.Concrete().ID(), ContentHash: v.Document().Hash.String(),
		Counts: countsEvent{
			Verified: counts.Verified, Violated: counts.Violated,
			Unverified: counts.Unverified, NotJudged: counts.NotJudged,
		},
		Judgments: []judgmentEvent{},
	}
	var violated []string
	for _, j := range v.Judgments() {
		event.Judgments = append(event.Judgments, judgmentEvent{
			Rule: j.Rule().ID(), InstanceLocation: j.InstanceLocation().String(),
			Verdict: string(j.Verdict()), By: string(j.By()), Reason: j.Reason(),
		})
		if j.Verdict() == domain.Violated {
			violated = append(violated, j.InstanceLocation().String())
		}
	}
	if counts.Violated > 0 {
		o.failed = true
		values := fill(args, event.Path, event.ContentHash)
		values["{instanceLocation}"] = strings.Join(violated, ", ")
		event.Instruction = replace(o.violated.Instruction, values)
		event.Next = replace(o.violated.Next, values)
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
