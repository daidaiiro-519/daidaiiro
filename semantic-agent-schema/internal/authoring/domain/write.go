package domain

import (
	"bytes"
	"encoding/json"
	"errors"
	"slices"
	"strconv"
)

// Write は、集約「JSON ドキュメント」を書く操作（作成か更新）。書いた後の JSON ドキュメント ・ 書く前のハッシュ ・
// 従う Concrete ・ instance location ごとの change を持つ。
// 不変条件：x-decided-by: human の値は、このコマンドでは default（更新では前の値）から変えない。人が決める値は、人がファイルを直接書く。
// 書くたびに instance location ごとの change を出す。
type Write struct {
	command    commandName
	document   Document
	beforeHash Hash
	concrete   Concrete
	changes    []Change
}

// WriteInput は、書く操作を作るために application が読んだもの。
type WriteInput struct {
	Path     Path
	Content  []byte // 書いた後の JSON ドキュメントのバイト列。書くときは字下げをそろえる（更新では、書く前のファイルの字下げ）
	Value    any    // Content を ParseJSON で読んだ値
	Concrete Concrete
}

// NewCreation は、作成を返す。x-decided-by: human のサブスキーマに当たるインスタンスの値が
// default と違えば（default が無ければ、どの値も）、その instance location ごとの decided_by_human の Failure をまとめて返す。
// change は、ルートを含むすべての instance location の add になる。
func NewCreation(in WriteInput) (*Write, error) {
	var failures []error
	for location, found := range decidedValues(in.Concrete, in.Value) {
		if found.hasDefault && found.value == found.def {
			continue
		}
		failures = append(failures, decidedFailure(in.Path, location, found.value))
	}
	if err := errors.Join(sortFailures(failures)...); err != nil {
		return nil, err
	}
	return newWrite(CreateCommand, in, defaultIndent, nil, nil, false)
}

// NewUpdate は、更新を返す。before は書く前の JSON ドキュメント。x-decided-by: human のサブスキーマに
// 当たるインスタンスの値が前と変われば（足す ・ 消すを含む）、その instance location ごとの decided_by_human の Failure を
// まとめて返す。change は、前後で値が違う instance location ごとに出す。
func NewUpdate(in WriteInput, before Document, beforeValue any) (*Write, error) {
	old, current := decidedValues(in.Concrete, beforeValue), decidedValues(in.Concrete, in.Value)
	var failures []error
	for location, found := range current {
		if prior, ok := old[location]; !ok || prior.value != found.value {
			failures = append(failures, decidedFailure(in.Path, location, found.value))
		}
	}
	for location, prior := range old {
		if _, ok := current[location]; !ok {
			failures = append(failures, decidedFailure(in.Path, location, prior.value))
		}
	}
	if err := errors.Join(sortFailures(failures)...); err != nil {
		return nil, err
	}
	return newWrite(UpdateCommand, in, indentOf(before.Content), before.Hash, beforeValue, true)
}

// defaultIndent は、作成と、字下げを読み取れない更新で書く字下げ。
const defaultIndent = "  "

// indentOf は、書く前のファイルの字下げ（字下げの文字と幅）を、最初に字下げした行の先頭の空白から読み取って返す。
// 字下げした行が無ければ（1行にまとめて書いてあれば）defaultIndent を返す。
func indentOf(content []byte) string {
	for _, line := range bytes.Split(content, []byte("\n")) {
		trimmed := bytes.TrimLeft(line, " \t")
		if len(trimmed) > 0 && len(trimmed) < len(line) {
			return string(line[:len(line)-len(trimmed)])
		}
	}
	return defaultIndent
}

// newWrite は、書く操作を返す。書く内容は、indent の字下げと末尾の改行にそろえ、ハッシュはそのバイト列から求める。
func newWrite(c commandName, in WriteInput, indent string, beforeHash Hash, beforeValue any, hasBefore bool) (*Write, error) {
	var formatted bytes.Buffer
	if err := json.Indent(&formatted, in.Content, "", indent); err != nil {
		return nil, Fail(InvalidJSON, in.Path.String(), err.Error())
	}
	formatted.WriteByte('\n')
	content := formatted.Bytes()
	return &Write{
		command:    c,
		document:   Document{Path: in.Path, Content: content, Hash: HashOf(content)},
		beforeHash: beforeHash,
		concrete:   in.Concrete,
		changes:    changesOf(beforeValue, hasBefore, in.Value),
	}, nil
}

func decidedFailure(path Path, location, value string) error {
	return &Failure{
		Code: DecidedByHuman, Path: path.String(), InstanceLocation: location,
		Message: "x-decided-by: human の値を変えた：" + value,
	}
}

// sortFailures は、Failure を instance location の順に並べる。同じ入力から同じ順の error になる。
func sortFailures(failures []error) []error {
	slices.SortFunc(failures, func(a, b error) int {
		var fa, fb *Failure
		if !errors.As(a, &fa) || !errors.As(b, &fb) {
			return 0
		}
		switch {
		case fa.InstanceLocation < fb.InstanceLocation:
			return -1
		case fa.InstanceLocation > fb.InstanceLocation:
			return 1
		}
		return 0
	})
	return failures
}

// decided は、x-decided-by: human のサブスキーマに当たるインスタンスの値（JSON の文字列）と、その default。
type decided struct {
	value, def string
	hasDefault bool
}

// decidedValues は、インスタンスと Concrete を一緒にたどり、x-decided-by: human の値を instance location ごとに返す。
func decidedValues(concrete Concrete, value any) map[string]decided {
	found := map[string]decided{}
	decidedByHuman(concrete.structure(), value, nil, func(location Pointer, v, def any, hasDefault bool) {
		d := decided{value: jsonText(v), hasDefault: hasDefault}
		if hasDefault {
			d.def = jsonText(def)
		}
		found[location.String()] = d
	})
	return found
}

// decidedByHuman は、インスタンスと Concrete を JSON Schema の構造（properties と items）で一緒にたどり、
// x-decided-by: human のサブスキーマに当たるインスタンスの値ごとに found を呼ぶ。配列は要素ごとにたどる。
func decidedByHuman(subschema map[string]any, value any, at []string, found func(location Pointer, value, def any, hasDefault bool)) {
	walkInstance(subschema, value, at, func(sub map[string]any, v any, location []string) {
		if sub["x-decided-by"] == "human" {
			def, hasDefault := sub["default"]
			found(PointerOf(location...), v, def, hasDefault)
		}
	})
}

// walkInstance は、ドメインサービス「サブスキーマを引く」の、インスタンスがあるときの側。インスタンスと Concrete を
// JSON Schema の構造（properties と items）で一緒にたどり、インスタンスにある値ごとに、そのサブスキーマ ・ instance location で
// visit を呼ぶ。x-decided-by: human の確認だけが使う。
func walkInstance(subschema map[string]any, value any, at []string, visit func(subschema map[string]any, value any, at []string)) {
	visit(subschema, value, at)
	if properties, ok := subschema["properties"].(map[string]any); ok {
		object, _ := value.(map[string]any) // オブジェクトでなければ、たどる値が無い
		for _, name := range sortedNames(properties) {
			child, isSchema := properties[name].(map[string]any)
			v, present := object[name]
			if isSchema && present {
				walkInstance(child, v, append(slices.Clone(at), name), visit)
			}
		}
	}
	if items, ok := subschema["items"].(map[string]any); ok {
		array, _ := value.([]any) // 配列でなければ、たどる値が無い
		for i, v := range array {
			walkInstance(items, v, append(slices.Clone(at), strconv.Itoa(i)), visit)
		}
	}
}

// Command は、書く操作のコマンド（create か update）を返す。
func (w *Write) Command() commandName { return w.command }

// Document は、書いた後の JSON ドキュメントを返す。
func (w *Write) Document() Document { return w.document }

// BeforeHash は、書く前のハッシュを返す。作成では nil。
func (w *Write) BeforeHash() Hash { return w.beforeHash }

// Concrete は、書いた JSON ドキュメントが従うスキーマを返す。
func (w *Write) Concrete() Concrete { return w.concrete }

// Changes は、instance location ごとの change を返す。
func (w *Write) Changes() []Change { return w.changes }

// MatchHash は、集約 JSON ドキュメントの不変条件（楽観ロック）として、最初に読んだときのハッシュが、書く直前のいまのハッシュと
// 同じかを確かめる。違えば hash_mismatch の Failure を返す。確かめることと書くことは、ロックの中で行う。
func MatchHash(path Path, read, current Hash) error {
	if read.String() == current.String() {
		return nil
	}
	return Fail(HashMismatch, path.String(), "読んだときのハッシュ "+read.String()+" が、書く直前のハッシュ "+current.String()+" と違う")
}
