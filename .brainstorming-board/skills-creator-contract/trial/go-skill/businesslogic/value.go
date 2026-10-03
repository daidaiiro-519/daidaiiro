// SPDX-License-Identifier: MIT

package businesslogic

// JSON の値。**欄の順を保つ** ── 描画はスキーマの properties の順に欄を並べるので、
// 順を失う map では、実行ごとに違う HTML が出る。
//
// 値は nil ・ bool ・ json.Number ・ string ・ []any ・ *Object のどれかである。

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"strconv"
	"strings"
)

// Object は、欄の順を保つ JSON の対象である。
type Object struct {
	keys []string
	vals map[string]any
}

// NewObject は空の対象を作る。
func NewObject() *Object { return &Object{vals: map[string]any{}} }

// Get は欄の値を返す。
func (o *Object) Get(k string) (any, bool) {
	if o == nil {
		return nil, false
	}
	v, ok := o.vals[k]
	return v, ok
}

// Set は欄を置く。**在れば位置を保って置き換え、無ければ末尾へ足す。**
func (o *Object) Set(k string, v any) {
	if _, ok := o.vals[k]; !ok {
		o.keys = append(o.keys, k)
	}
	o.vals[k] = v
}

// Delete は欄を除く。**残りの順を保つ。**
func (o *Object) Delete(k string) {
	if _, ok := o.vals[k]; !ok {
		return
	}
	delete(o.vals, k)
	for i, x := range o.keys {
		if x == k {
			o.keys = append(o.keys[:i:i], o.keys[i+1:]...)
			break
		}
	}
}

// Keys は欄の名前を順に返す。
func (o *Object) Keys() []string { return o.keys }

// Len は欄の数を返す。
func (o *Object) Len() int { return len(o.keys) }

// ParseJSON は、欄の順を保って読む。**後ろに余りがあれば誤りにする。**
func ParseJSON(text string) (any, error) {
	dec := json.NewDecoder(strings.NewReader(text))
	dec.UseNumber()
	v, err := parseValue(dec)
	if err != nil {
		return nil, err
	}
	if _, err := dec.Token(); !errors.Is(err, io.EOF) {
		return nil, errors.New("値の後ろに余りがある")
	}
	return v, nil
}

func parseValue(dec *json.Decoder) (any, error) {
	tok, err := dec.Token()
	if err != nil {
		return nil, err
	}
	switch t := tok.(type) {
	case json.Delim:
		switch t {
		case '{':
			o := NewObject()
			for dec.More() {
				kt, err := dec.Token()
				if err != nil {
					return nil, err
				}
				k, _ := kt.(string)
				v, err := parseValue(dec)
				if err != nil {
					return nil, err
				}
				o.Set(k, v)
			}
			if _, err := dec.Token(); err != nil {
				return nil, err
			}
			return o, nil
		case '[':
			a := []any{}
			for dec.More() {
				v, err := parseValue(dec)
				if err != nil {
					return nil, err
				}
				a = append(a, v)
			}
			if _, err := dec.Token(); err != nil {
				return nil, err
			}
			return a, nil
		}
		return nil, fmt.Errorf("予期しない区切り %v", t)
	default:
		return tok, nil
	}
}

// get は、対象の欄を返す。**対象でなければ無い。**
func get(v any, k string) (any, bool) {
	if o, ok := v.(*Object); ok {
		return o.Get(k)
	}
	return nil, false
}

// getOr は、欄が無ければ nil を返す。
func getOr(v any, k string) any {
	x, _ := get(v, k)
	return x
}

func strOf(v any, k string) (string, bool) {
	x, ok := get(v, k)
	if !ok {
		return "", false
	}
	s, ok := x.(string)
	return s, ok
}

func asArray(v any) ([]any, bool) {
	a, ok := v.([]any)
	return a, ok
}

func asObject(v any) (*Object, bool) {
	o, ok := v.(*Object)
	return o, ok
}

// pointer は JSON Pointer（RFC 6901）で引く。
func pointer(v any, p string) (any, bool) {
	if p == "" {
		return v, true
	}
	if !strings.HasPrefix(p, "/") {
		return nil, false
	}
	cur := v
	for _, tok := range strings.Split(p[1:], "/") {
		tok = strings.ReplaceAll(strings.ReplaceAll(tok, "~1", "/"), "~0", "~")
		switch c := cur.(type) {
		case *Object:
			x, ok := c.Get(tok)
			if !ok {
				return nil, false
			}
			cur = x
		case []any:
			if tok == "" || (len(tok) > 1 && tok[0] == '0') {
				return nil, false
			}
			i, err := strconv.Atoi(tok)
			if err != nil || i < 0 || i >= len(c) {
				return nil, false
			}
			cur = c[i]
		default:
			return nil, false
		}
	}
	return cur, true
}

// equal は2つの値が同じかを返す。
func equal(a, b any) bool {
	switch x := a.(type) {
	case nil:
		return b == nil
	case bool:
		y, ok := b.(bool)
		return ok && x == y
	case string:
		y, ok := b.(string)
		return ok && x == y
	case json.Number:
		y, ok := b.(json.Number)
		return ok && x == y
	case []any:
		y, ok := b.([]any)
		if !ok || len(x) != len(y) {
			return false
		}
		for i := range x {
			if !equal(x[i], y[i]) {
				return false
			}
		}
		return true
	case *Object:
		y, ok := b.(*Object)
		if !ok || x.Len() != y.Len() {
			return false
		}
		for _, k := range x.keys {
			yv, ok := y.Get(k)
			if !ok || !equal(x.vals[k], yv) {
				return false
			}
		}
		return true
	}
	return false
}

// optEqual は、在るかどうかも含めて比べる（片方だけ在れば違う）。
func optEqual(a any, aok bool, b any, bok bool) bool {
	if aok != bok {
		return false
	}
	return !aok || equal(a, b)
}

// clone は値を深く写す。
func clone(v any) any {
	switch x := v.(type) {
	case *Object:
		o := NewObject()
		for _, k := range x.keys {
			o.Set(k, clone(x.vals[k]))
		}
		return o
	case []any:
		a := make([]any, len(x))
		for i, e := range x {
			a[i] = clone(e)
		}
		return a
	}
	return v
}

// Compact は、空白を入れずに JSON の文字列にする。
func Compact(v any) string {
	var b bytes.Buffer
	write(&b, v, "", 0)
	return b.String()
}

// Pretty は、2字の字下げで JSON の文字列にする。
func Pretty(v any) string {
	var b bytes.Buffer
	write(&b, v, "  ", 0)
	return b.String()
}

func write(b *bytes.Buffer, v any, indent string, depth int) {
	nl := func(d int) {
		if indent != "" {
			b.WriteByte('\n')
			b.WriteString(strings.Repeat(indent, d))
		}
	}
	switch x := v.(type) {
	case nil:
		b.WriteString("null")
	case bool:
		b.WriteString(strconv.FormatBool(x))
	case json.Number:
		b.WriteString(string(x))
	case string:
		writeString(b, x)
	case []string:
		a := make([]any, len(x))
		for i, s := range x {
			a[i] = s
		}
		write(b, a, indent, depth)
	case []any:
		if len(x) == 0 {
			b.WriteString("[]")
			return
		}
		b.WriteByte('[')
		for i, e := range x {
			if i > 0 {
				b.WriteByte(',')
			}
			nl(depth + 1)
			write(b, e, indent, depth+1)
		}
		nl(depth)
		b.WriteByte(']')
	case *Object:
		if x.Len() == 0 {
			b.WriteString("{}")
			return
		}
		b.WriteByte('{')
		for i, k := range x.keys {
			if i > 0 {
				b.WriteByte(',')
			}
			nl(depth + 1)
			writeString(b, k)
			b.WriteByte(':')
			if indent != "" {
				b.WriteByte(' ')
			}
			write(b, x.vals[k], indent, depth+1)
		}
		nl(depth)
		b.WriteByte('}')
	case int:
		b.WriteString(strconv.Itoa(x))
	default:
		writeString(b, fmt.Sprint(x))
	}
}

func writeString(b *bytes.Buffer, s string) {
	b.WriteByte('"')
	for _, r := range s {
		switch r {
		case '"':
			b.WriteString(`\"`)
		case '\\':
			b.WriteString(`\\`)
		case '\n':
			b.WriteString(`\n`)
		case '\r':
			b.WriteString(`\r`)
		case '\t':
			b.WriteString(`\t`)
		case '\b':
			b.WriteString(`\b`)
		case '\f':
			b.WriteString(`\f`)
		default:
			if r < 0x20 {
				fmt.Fprintf(b, `\u%04x`, r)
			} else {
				b.WriteRune(r)
			}
		}
	}
	b.WriteByte('"')
}

// plainJSON は、順を保つ値を、検査のライブラリが読む形（map と json.Number）へ写す。
func plainJSON(v any) any {
	switch x := v.(type) {
	case *Object:
		m := make(map[string]any, x.Len())
		for _, k := range x.keys {
			m[k] = plainJSON(x.vals[k])
		}
		return m
	case []any:
		a := make([]any, len(x))
		for i, e := range x {
			a[i] = plainJSON(e)
		}
		return a
	}
	return v
}
