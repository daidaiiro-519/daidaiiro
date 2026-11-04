// Package publishedlanguage は、公開された言葉。AI と人へ公開する入力 ・ 出力 ・ Concrete を書く型のスキーマと、
// error の instruction と next の定型を、バイナリに埋め込んで持つ。
package publishedlanguage

import "embed"

//go:embed v1/*.json
var files embed.FS

// 公開された言葉のスキーマの $id。v1/*.json の $id と同じ。
const (
	AbstractID   = "urn:semantic-agent-schema:abstract:v1"
	VocabularyID = "urn:semantic-agent-schema:vocabulary:v1"
	ArgumentsID  = "urn:semantic-agent-schema:arguments:v1"
)

// Schemas は、v1 のスキーマのファイル名。
var Schemas = []string{
	"abstract.schema.json",
	"vocabulary.schema.json",
	"dialect.schema.json",
	"arguments.schema.json",
	"event.schema.json",
}

// Read は、v1 のファイルのバイト列を返す。無い名前なら error を返す。
func Read(name string) ([]byte, error) {
	return files.ReadFile("v1/" + name)
}
