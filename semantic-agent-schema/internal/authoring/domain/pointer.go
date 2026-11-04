package domain

import (
	"regexp"
	"strings"
)

// Pointer は、値オブジェクト JSON Pointer（RFC 6901）。instance location を表す。
type Pointer interface {
	String() string
	pointer()
}

type jsonPointer struct{ value string }

func (p jsonPointer) String() string { return p.value }
func (jsonPointer) pointer()         {}

var pointerForm = regexp.MustCompile(`^(/([^~/]|~[01])*)*$`)

// NewPointer は、JSON Pointer の文字列から Pointer を返す。形が違えば schema_violation の Failure を返す。
func NewPointer(value string) (Pointer, error) {
	if !pointerForm.MatchString(value) {
		return nil, Fail(SchemaViolation, "", value+" は JSON Pointer の形でない")
	}
	return jsonPointer{value}, nil
}

// PointerOf は、参照トークンの並びを Pointer にする。~ と / はエスケープする。
func PointerOf(tokens ...string) Pointer {
	var b strings.Builder
	for _, token := range tokens {
		b.WriteByte('/')
		b.WriteString(strings.ReplaceAll(strings.ReplaceAll(token, "~", "~0"), "/", "~1"))
	}
	return jsonPointer{b.String()}
}
