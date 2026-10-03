// SPDX-License-Identifier: MIT

package businesslogic

import "strings"

func trimLeft(s string) string  { return strings.TrimLeftFunc(s, isSpace) }
func trimRight(s string) string { return strings.TrimRightFunc(s, isSpace) }
func trim(s string) string      { return strings.TrimFunc(s, isSpace) }

func plainText(t string) string {
	return strings.ReplaceAll(strings.ReplaceAll(t, "**", ""), "`", "")
}

// numbered は「数字. 」で始まる行なら、その後ろを返す。
func numbered(t string) (string, bool) {
	n, rest, ok := strings.Cut(t, ". ")
	if !ok || n == "" {
		return "", false
	}
	for _, c := range n {
		if c < '0' || c > '9' {
			return "", false
		}
	}
	return rest, true
}

func cells(line string) []any {
	parts := strings.Split(strings.Trim(trim(line), "|"), "|")
	out := make([]any, len(parts))
	for i, c := range parts {
		out[i] = plainText(trim(c))
	}
	return out
}

func obj(kv ...any) *Object {
	o := NewObject()
	for i := 0; i+1 < len(kv); i += 2 {
		o.Set(kv[i].(string), kv[i+1])
	}
	return o
}

// BlocksOf は本文を塊に分ける ── 段落 ・ 箇条書き ・ 表 ・ コード。区切りの線（---）は捨てる。
func BlocksOf(lines []string) []any {
	out := []any{}
	var para []string
	flush := func() {
		if len(para) > 0 {
			out = append(out, obj("kind", "para", "text", strings.Join(para, "")))
			para = nil
		}
	}
	i := 0
	for i < len(lines) {
		line := trimRight(lines[i])
		t := trimLeft(line)
		_, isNum := numbered(t)
		switch {
		case strings.HasPrefix(t, "```"):
			flush()
			var code []string
			i++
			for i < len(lines) && !strings.HasPrefix(trimLeft(lines[i]), "```") {
				code = append(code, lines[i])
				i++
			}
			out = append(out, obj("kind", "code", "text", strings.Join(code, "\n")))
		case strings.HasPrefix(t, "|"):
			flush()
			head := cells(t)
			rows := []any{}
			i++
			for i < len(lines) && strings.HasPrefix(trimLeft(lines[i]), "|") {
				r := cells(lines[i])
				sep := true
				for _, c := range r {
					if strings.Trim(c.(string), "-: ") != "" {
						sep = false
						break
					}
				}
				if !sep {
					rows = append(rows, r)
				}
				i++
			}
			out = append(out, obj("kind", "table", "head", head, "rows", rows))
			continue
		case strings.HasPrefix(t, "- ") || strings.HasPrefix(t, "* ") || isNum:
			flush()
			ordered := !strings.HasPrefix(t, "-") && !strings.HasPrefix(t, "*")
			items := []any{}
			for i < len(lines) {
				l := trimLeft(lines[i])
				var item string
				if x, ok := strings.CutPrefix(l, "- "); ok {
					item = x
				} else if x, ok := strings.CutPrefix(l, "* "); ok {
					item = x
				} else if x, ok := numbered(l); ok {
					item = x
				} else {
					break
				}
				items = append(items, plainText(item))
				i++
			}
			out = append(out, obj("kind", "list", "ordered", ordered, "items", items))
			continue
		case t == "" || strings.Trim(t, "-") == "":
			flush()
		default:
			para = append(para, plainText(trim(strings.TrimLeft(t, ">"))))
		}
		i++
	}
	flush()
	return out
}

// lines は Rust の str::lines と同じに行へ分ける（末尾の改行は行を作らず、行末の \r は外す）。
func lines(s string) []string {
	if s == "" {
		return nil
	}
	parts := strings.Split(s, "\n")
	if parts[len(parts)-1] == "" {
		parts = parts[:len(parts)-1]
	}
	for i, p := range parts {
		parts[i] = strings.TrimSuffix(p, "\r")
	}
	return parts
}

type mdNode struct {
	title    string
	level    int
	body     []string
	children []*mdNode
}

func (n *mdNode) toJSON() *Object {
	m := NewObject()
	m.Set("title", n.title)
	if b := BlocksOf(n.body); len(b) > 0 {
		m.Set("blocks", b)
	}
	if len(n.children) > 0 {
		kids := make([]any, len(n.children))
		for i, c := range n.children {
			kids[i] = c.toJSON()
		}
		m.Set("sections", kids)
	}
	return m
}

// ImportMarkdown は Markdown の文書を document の1件へ取り込む。見出しを節の入れ子に分ける。
func ImportMarkdown(id, location, fetched, text string) any {
	stack := []*mdNode{{}}
	fence := false
	// frontmatter は取り込まない
	body := text
	if rest, ok := strings.CutPrefix(text, "---\n"); ok {
		if _, after, ok := strings.Cut(rest, "\n---\n"); ok {
			body = after
		}
	}
	popInto := func() {
		done := stack[len(stack)-1]
		stack = stack[:len(stack)-1]
		parent := stack[len(stack)-1]
		parent.children = append(parent.children, done)
	}
	for _, line := range lines(body) {
		if strings.HasPrefix(trimLeft(line), "```") {
			fence = !fence
		}
		hashes := len(line) - len(strings.TrimLeft(line, "#"))
		heading := !fence && hashes >= 1 && hashes <= 6 && strings.HasPrefix(line[hashes:], " ")
		if !heading {
			top := stack[len(stack)-1]
			top.body = append(top.body, line)
			continue
		}
		for len(stack) > 1 && stack[len(stack)-1].level >= hashes {
			popInto()
		}
		stack = append(stack, &mdNode{title: trim(line[hashes:]), level: hashes})
	}
	for len(stack) > 1 {
		popInto()
	}
	root := stack[0]
	title := id
	if len(root.children) > 0 {
		title = root.children[0].title
	}
	doc := obj("id", id, "title", title,
		"source", obj("location", location, "sha256", SHA256Hex([]byte(text)), "fetched", fetched))
	sections := make([]any, len(root.children))
	for i, c := range root.children {
		sections[i] = c.toJSON()
	}
	doc.Set("sections", sections)
	if lead := BlocksOf(root.body); len(lead) > 0 {
		doc.Set("blocks", lead)
	}
	return doc
}
