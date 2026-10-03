// SPDX-License-Identifier: MIT

// Package businesslogic は業務ロジック層である。入出力を持たず、データアクセス層だけを参照する。
//
// references は、Skill の目的を達成するために参照する情報である。種類ごとに、JSON Schema
// （<種類>.schema.json）と、それに従う JSON（<種類>.json）を置く。持つ能力は4つである ──
// 取り出す（Get）・ 検査する（Validate）・ 描画する（View）・ 既存の文書を取り込む（ImportMarkdown）。
package businesslogic

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"path/filepath"
	"sort"
	"strings"
	"unicode"
	"unicode/utf8"

	"github.com/santhosh-tekuri/jsonschema/v6"
	"golang.org/x/text/language"
	"golang.org/x/text/message"

	"trialgo/dataaccess"
)

// SchemaTail はスキーマのファイル名の末尾である。
const SchemaTail = ".schema.json"

const (
	viewTokens   = "view.tokens.json"
	viewCSS      = "view.css"
	viewTemplate = "view.template.html"
	skillTokens  = "view.skill.tokens.json"
	skillCSS     = "view.skill.css"
)

// Kind は種類1つである。Data はスキーマだけの種類では空である。
type Kind struct {
	Name   string
	Schema string
	Data   string
}

func fileNames(paths []string) []string {
	out := make([]string, 0, len(paths))
	for _, p := range paths {
		out = append(out, filepath.Base(p))
	}
	return out
}

func listNames(refs string) ([]string, error) {
	entries, err := dataaccess.List(refs)
	if err != nil {
		return nil, fmt.Errorf("%s を読めない ── %v", refs, err)
	}
	names := fileNames(entries)
	sort.Strings(names)
	return names, nil
}

// Kinds は references の種類を名前の順に並べる。見た目の正本の複製（view.*）は種類ではない。
func Kinds(refs string) ([]Kind, error) {
	names, err := listNames(refs)
	if err != nil {
		return nil, err
	}
	var out []Kind
	for _, file := range names {
		if !strings.HasSuffix(file, SchemaTail) || strings.HasPrefix(file, "view.") {
			continue
		}
		name := strings.TrimSuffix(file, SchemaTail)
		data := filepath.Join(refs, name+".json")
		k := Kind{Name: name, Schema: filepath.Join(refs, file)}
		if dataaccess.IsFile(data) {
			k.Data = data
		}
		out = append(out, k)
	}
	return out, nil
}

// Save は描画した頁を書き出す。
func Save(out, html string) error {
	if err := dataaccess.Write(out, html); err != nil {
		return fmt.Errorf("%s に書けない ── %v", out, err)
	}
	return nil
}

// ReadText は取り込む文書を読む。
func ReadText(file string) (string, error) {
	s, err := dataaccess.ReadToString(file)
	if err != nil {
		return "", fmt.Errorf("%s を読めない ── %v", file, err)
	}
	return s, nil
}

func readJSON(path string) (any, error) {
	body, err := dataaccess.ReadToString(path)
	if err != nil {
		return nil, fmt.Errorf("%s を読めない ── %v", path, err)
	}
	v, err := ParseJSON(body)
	if err != nil {
		return nil, fmt.Errorf("%s が JSON でない ── %v", path, err)
	}
	return v, nil
}

// against はスキーマで検査する。$schema の欄は、スキーマを指す印なので外してから当てる。
func against(schema, instance any, head string) []string {
	plain := clone(instance)
	if o, ok := plain.(*Object); ok {
		o.Delete("$schema")
	}
	c := jsonschema.NewCompiler()
	c.DefaultDraft(jsonschema.Draft2020)
	const url = "mem:///schema.json"
	if err := c.AddResource(url, plainJSON(schema)); err != nil {
		return []string{head + ": スキーマとして無効である"}
	}
	sch, err := c.Compile(url)
	if err != nil {
		return []string{head + ": スキーマとして無効である"}
	}
	verr := sch.Validate(plainJSON(plain))
	if verr == nil {
		return nil
	}
	ve, ok := verr.(*jsonschema.ValidationError)
	if !ok {
		return []string{fmt.Sprintf("%s:  ── %v", head, verr)}
	}
	p := message.NewPrinter(language.English)
	var found []string
	var leaves func(e *jsonschema.ValidationError)
	leaves = func(e *jsonschema.ValidationError) {
		if len(e.Causes) == 0 {
			at := strings.Join(e.InstanceLocation, "/")
			found = append(found, fmt.Sprintf("%s: %s ── %s", head, at, e.ErrorKind.LocalizedString(p)))
			return
		}
		for _, c := range e.Causes {
			leaves(c)
		}
	}
	leaves(ve)
	sort.Strings(found)
	return found
}

// Validate は references を検査する。スキーマに合わない JSON、スキーマを指さない JSON、Markdown を並べる。
func Validate(refs string) ([]string, error) {
	files, err := listNames(refs)
	if err != nil {
		return nil, err
	}
	found := []string{}
	for _, file := range files {
		if strings.HasSuffix(file, ".md") {
			found = append(found, file+": Markdown が在る ── references は JSON Schema と JSON で持つ（Markdown は SKILL.md だけ）")
		}
		if !strings.HasSuffix(file, ".json") || strings.HasSuffix(file, SchemaTail) {
			continue
		}
		data, err := readJSON(filepath.Join(refs, file))
		if err != nil {
			found = append(found, err.Error())
			continue
		}
		own := strings.TrimSuffix(file, ".json") + SchemaTail
		want, ok := strOf(data, "$schema")
		if !ok {
			found = append(found, fmt.Sprintf("%s: $schema が無い ── %s を指す", file, own))
			want = own
		}
		schema, err := readJSON(filepath.Join(refs, want))
		if err != nil {
			found = append(found, fmt.Sprintf("%s: スキーマ %s が無い", file, want))
			continue
		}
		found = append(found, against(schema, data, file)...)
	}
	return found, nil
}

// ValidateFile は1つの JSON を種類のスキーマで検査する。形に合っても、指す先が無ければ通さない。
func ValidateFile(refs, kind, file string) ([]string, error) {
	schema, err := readJSON(filepath.Join(refs, kind+SchemaTail))
	if err != nil {
		return nil, err
	}
	data, err := readJSON(file)
	if err != nil {
		return nil, err
	}
	head := file
	found := against(schema, data, head)
	if found == nil {
		found = []string{}
	}
	merged := mergeRefs(schema, schema, 0)
	var cited []string
	cite(refs, merged, merged, data, "", head, &cited)
	sort.Strings(cited)
	return append(found, cited...), nil
}

// itemsOf は指す先の種類の項目を返す。項目が0件なら無い。
func itemsOf(refs, kind string) ([]any, bool) {
	data, err := readJSON(filepath.Join(refs, kind+".json"))
	if err != nil {
		return nil, false
	}
	items, ok := itemsField(data)
	if !ok || len(items) == 0 {
		return nil, false
	}
	return items, true
}

// itemsField は items の欄を、無ければ値そのものを、並びとして返す。
func itemsField(data any) ([]any, bool) {
	if x, ok := get(data, "items"); ok {
		return asArray(x)
	}
	return asArray(data)
}

func isSpace(r rune) bool { return unicode.Is(unicode.White_Space, r) }

func dropSpace(s string) string {
	return strings.Map(func(r rune) rune {
		if isSpace(r) {
			return -1
		}
		return r
	}, s)
}

// plainOf は項目の中の文字列をすべて、空白を外してつなぐ。
func plainOf(v any, out *strings.Builder) {
	switch x := v.(type) {
	case string:
		out.WriteString(dropSpace(x))
	case []any:
		for _, e := range x {
			plainOf(e, out)
		}
	case *Object:
		for _, k := range x.keys {
			plainOf(x.vals[k], out)
		}
	}
}

// fragments は引用を照らす断片に分ける。省略（…）と並び（ ・ ）で切り、「欄の題：」を外す。
func fragments(quote string) []string {
	var out []string
	for _, a := range strings.Split(quote, "…") {
		for _, x := range strings.Split(a, " ・ ") {
			if _, v, ok := strings.Cut(x, "："); ok {
				x = v
			}
			x = strings.TrimRight(dropSpace(x), "。")
			if utf8.RuneCountInString(x) >= 2 {
				out = append(out, x)
			}
		}
	}
	return out
}

// cite は参照の注記（x-refers ・ x-quotes）に従って、指す先を照らす。
func cite(refs string, schema, root, value any, at, head string, out *[]string) {
	node := resolve(schema, root)
	switch v := value.(type) {
	case *Object:
		props, _ := get(node, "properties")
		for _, k := range v.keys {
			x := v.vals[k]
			ps0, ok := get(props, k)
			if !ok {
				continue
			}
			ps := resolve(ps0, root)
			path := at + "/" + k
			kind, kok := strOf(ps, "x-refers")
			id, iok := x.(string)
			if kok && iok {
				if items, ok := itemsOf(refs, kind); ok {
					hit := false
					for _, it := range items {
						if s, ok := strOf(it, "id"); ok && s == id {
							hit = true
							break
						}
					}
					if !hit {
						*out = append(*out, fmt.Sprintf("%s: %s ── %s に id %s が無い", head, path, kind, id))
					}
				}
			}
			sib, sok := strOf(ps, "x-quotes")
			quote, qok := x.(string)
			if sok && qok {
				var skind string
				skok := false
				if sp, ok := get(props, sib); ok {
					skind, skok = strOf(resolve(sp, root), "x-refers")
				}
				sid, sidok := strOf(v, sib)
				if skok && sidok {
					var item any
					found := false
					if items, ok := itemsOf(refs, skind); ok {
						for _, it := range items {
							if s, ok := strOf(it, "id"); ok && s == sid {
								item, found = it, true
								break
							}
						}
					}
					if found {
						var text strings.Builder
						plainOf(item, &text)
						t := text.String()
						for _, f := range fragments(quote) {
							if !strings.Contains(t, f) {
								*out = append(*out, fmt.Sprintf("%s: %s ── %s の %s の記述に無い ──「%s」", head, path, skind, sid, f))
							}
						}
					}
				}
			}
			cite(refs, ps, root, x, path, head, out)
		}
	case []any:
		if items, ok := get(node, "items"); ok {
			for i, x := range v {
				cite(refs, items, root, x, fmt.Sprintf("%s/%d", at, i), head, out)
			}
		}
	}
}

// Omit は指定した欄を、入れ子のすべてから除く。
func Omit(v any, keys []string) any {
	switch x := v.(type) {
	case *Object:
		o := NewObject()
		for _, k := range x.keys {
			skip := false
			for _, d := range keys {
				if d == k {
					skip = true
					break
				}
			}
			if !skip {
				o.Set(k, Omit(x.vals[k], keys))
			}
		}
		return o
	case []any:
		a := make([]any, len(x))
		for i, e := range x {
			a[i] = Omit(e, keys)
		}
		return a
	}
	return v
}

// Get は種類の JSON を取り出す。id を渡すと、その1件だけを返す。
func Get(refs, kind string, id *string) (any, error) {
	data, err := readJSON(filepath.Join(refs, kind+".json"))
	if err != nil {
		return nil, err
	}
	if o, ok := data.(*Object); ok {
		o.Delete("$schema")
	}
	if id == nil {
		return data, nil
	}
	items, ok := itemsField(data)
	if !ok {
		return nil, fmt.Errorf("%s は項目の並びではない ── id で取り出せない", kind)
	}
	for _, x := range items {
		if s, ok := strOf(x, "id"); ok && s == *id {
			return x, nil
		}
	}
	return nil, fmt.Errorf("%s に id %s の項目が無い", kind, *id)
}

// SHA256Hex は SHA-256 を16進で返す。
func SHA256Hex(data []byte) string {
	h := sha256.Sum256(data)
	return hex.EncodeToString(h[:])
}

// PutDocument は document の並びへ1件を置く。同じ id が在れば置き換える。
func PutDocument(refs string, doc any) (string, error) {
	path := filepath.Join(refs, "document.json")
	var all any
	if dataaccess.IsFile(path) {
		v, err := readJSON(path)
		if err != nil {
			return "", err
		}
		all = v
	} else {
		o := NewObject()
		o.Set("$schema", "document.schema.json")
		o.Set("items", []any{})
		all = o
	}
	id, idok := get(doc, "id")
	itemsV, ok := get(all, "items")
	items, aok := itemsV.([]any)
	if !ok || !aok {
		return "", fmt.Errorf("document.json に items が無い")
	}
	kept := []any{}
	for _, x := range items {
		xid, xok := get(x, "id")
		if idok && xok && equal(xid, id) || (!idok && xok && xid == nil) {
			continue
		}
		kept = append(kept, x)
	}
	kept = append(kept, doc)
	all.(*Object).Set("items", kept)
	if err := dataaccess.Write(path, Pretty(all)+"\n"); err != nil {
		return "", fmt.Errorf("%s に書けない ── %v", path, err)
	}
	return path, nil
}
