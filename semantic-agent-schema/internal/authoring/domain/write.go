package domain

import (
	"bytes"
	"encoding/json"
)

// Write は、集約「JSON ドキュメント」を書く操作（作成か更新）。書いた後の JSON ドキュメント ・ 書く前のハッシュ ・
// 従う Concrete ・ instance location ごとの change を持つ。
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

// NewCreation は、作成を返す。change は、ルートを含むすべての instance location の add になる。
func NewCreation(in WriteInput) (*Write, error) {
	return newWrite(CreateCommand, in, defaultIndent, nil, nil, false)
}

// NewUpdate は、更新を返す。before は書く前の JSON ドキュメント。change は、前後で値が違う instance location ごとに出す。
func NewUpdate(in WriteInput, before Document, beforeValue any) (*Write, error) {
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
