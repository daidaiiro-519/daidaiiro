package domain

import (
	"bytes"
	"encoding/json"
	"errors"
	"io"
)

// Document は、ポート Documents が読んだ JSON ドキュメントのバイト列と、そのハッシュ。
type Document struct {
	Path    Path
	Content []byte
	Hash    Hash
}

// clone は、Content を複製した Document を返す。
func (d Document) clone() Document {
	d.Content = bytes.Clone(d.Content)
	return d
}

// Violation は、機械的な検証の違反1つ。JSON Schema の実装が出した違反の葉か、作法の検査の違反。
type Violation struct {
	InstanceLocation        Pointer
	AbsoluteKeywordLocation string
	Message                 string
}

// ParseJSON は、JSON ドキュメントのバイト列を、数を json.Number のまま読んだ値を返す。
// JSON として読めないときと、値の後に余分な文字があるときは invalid_json の Failure を返す。
func ParseJSON(content []byte) (any, error) {
	decoder := json.NewDecoder(bytes.NewReader(content))
	decoder.UseNumber()
	var value any
	if err := decoder.Decode(&value); err != nil {
		return nil, Fail(InvalidJSON, "", err.Error())
	}
	if _, err := decoder.Token(); !errors.Is(err, io.EOF) {
		return nil, Fail(InvalidJSON, "", "値の後に余分な文字がある")
	}
	return value, nil
}

// SchemaIDOf は、インスタンスの $schema（従う Concrete の $id）を返す。無ければ false を返す。
func SchemaIDOf(value any) (string, bool) {
	object, ok := value.(map[string]any)
	if !ok {
		return "", false
	}
	id, ok := object["$schema"].(string)
	return id, ok
}
