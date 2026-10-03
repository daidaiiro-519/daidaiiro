// SPDX-License-Identifier: MIT

package businesslogic

// 描画。**描画は回答の形も種類の中身も参照しない** ── スキーマの title を見出しに、
// description を説明にし、見せ方は x-view が決める。

import (
	"fmt"
	"path/filepath"
	"sort"
	"strings"

	"trialgo/dataaccess"
)

func esc(s string) string {
	s = strings.ReplaceAll(s, "&", "&amp;")
	s = strings.ReplaceAll(s, "<", "&lt;")
	s = strings.ReplaceAll(s, ">", "&gt;")
	return strings.ReplaceAll(s, `"`, "&quot;")
}

func textOf(v any) string {
	switch x := v.(type) {
	case string:
		return x
	case nil:
		return ""
	}
	return Compact(v)
}

// resolve は $ref（#/$defs/…）を解く。スキーマの中だけを指す。
func resolve(schema, root any) any {
	r, ok := strOf(schema, "$ref")
	if !ok {
		return schema
	}
	if p, ok := strings.CutPrefix(r, "#"); ok {
		if t, ok := pointer(root, p); ok {
			return t
		}
	}
	return schema
}

// mergeRefs は $ref の隣の語を参照先に重ねる。隣の語が参照先より優先する。深さで止める。
func mergeRefs(v, root any, depth int) any {
	switch x := v.(type) {
	case *Object:
		if r, ok := strOf(x, "$ref"); ok && x.Len() > 1 {
			if p, ok := strings.CutPrefix(r, "#"); ok {
				if t, ok := pointer(root, p); ok {
					if tobj, ok := t.(*Object); ok && depth < 8 {
						merged, ok := mergeRefs(tobj, root, depth+1).(*Object)
						if !ok {
							merged = NewObject()
						}
						for _, k := range x.keys {
							if k != "$ref" {
								merged.Set(k, mergeRefs(x.vals[k], root, depth))
							}
						}
						return merged
					}
				}
			}
		}
		o := NewObject()
		for _, k := range x.keys {
			o.Set(k, mergeRefs(x.vals[k], root, depth))
		}
		return o
	case []any:
		a := make([]any, len(x))
		for i, e := range x {
			a[i] = mergeRefs(e, root, depth)
		}
		return a
	}
	return v
}

func arrayOf(v any, k string) []any {
	a, _ := asArray(getOr(v, k))
	return a
}

// graphHTML は節点と辺だけの図を、つながりの並びにする。
func graphHTML(value any) string {
	nodes := arrayOf(value, "nodes")
	edges := arrayOf(value, "edges")
	if len(nodes) == 0 && len(edges) == 0 {
		return ""
	}
	label := func(id string) string {
		for _, n := range nodes {
			if s, ok := strOf(n, "id"); ok && s == id {
				if l, ok := strOf(n, "label"); ok {
					return l
				}
				break
			}
		}
		return id
	}
	var lis strings.Builder
	for _, e := range edges {
		var notes []string
		if l, ok := strOf(e, "label"); ok && l != "" {
			notes = append(notes, l)
		}
		if d, ok := getOr(e, "dashed").(bool); ok && d {
			notes = append(notes, "破線")
		}
		arrow := "→"
		if len(notes) > 0 {
			arrow = "→（" + strings.Join(notes, " ・ ") + "）→"
		}
		from, _ := strOf(e, "from")
		to, _ := strOf(e, "to")
		fmt.Fprintf(&lis, "<li>%s %s %s</li>", esc(label(from)), esc(arrow), esc(label(to)))
	}
	for _, n := range nodes {
		id, _ := strOf(n, "id")
		linked := false
		for _, e := range edges {
			f, fok := strOf(e, "from")
			t, tok := strOf(e, "to")
			if (fok && f == id) || (tok && t == id) {
				linked = true
				break
			}
		}
		if !linked {
			l, ok := strOf(n, "label")
			if !ok {
				l = id
			}
			fmt.Fprintf(&lis, "<li>%s</li>", esc(l))
		}
	}
	return `<ol class="graph">` + lis.String() + `</ol>`
}

func labelOf(schema, value any) string {
	for _, o := range arrayOf(schema, "oneOf") {
		c, ok := get(o, "const")
		if ok && equal(c, value) {
			if t, ok := strOf(o, "title"); ok {
				return t
			}
			break
		}
	}
	return textOf(value)
}

func tag(schema, value any) string {
	neg := value == false || value == "reject"
	cls := "tag"
	if neg {
		cls = "tag neg"
	}
	return fmt.Sprintf(`<span class="%s">%s</span>`, cls, esc(labelOf(schema, value)))
}

func has(v any, k string) bool {
	_, ok := get(v, k)
	return ok
}

func isScalar(schema any) bool {
	if viewOf(schema) == "hidden" || has(schema, "oneOf") {
		return true
	}
	t, _ := strOf(schema, "type")
	switch t {
	case "string", "number", "integer", "boolean":
		return true
	}
	return false
}

func titleOr(schema any, key string) string {
	if t, ok := strOf(schema, "title"); ok {
		return t
	}
	return key
}

func headOf(key string, schema any, level int) string {
	h := min(max(level, 2), 3)
	desc := ""
	if d, ok := strOf(schema, "description"); ok {
		desc = `<p class="desc">` + esc(d) + `</p>`
	}
	return fmt.Sprintf("<h%d>%s</h%d>%s", h, esc(titleOr(schema, key)), h, desc)
}

type ctx struct {
	root any
	base string
}

func viewOf(schema any) string {
	s, _ := strOf(schema, "x-view")
	return s
}

func cell(schema, value any, c *ctx) string {
	schema = resolve(schema, c.root)
	if viewOf(schema) == "tag" || has(schema, "oneOf") {
		return tag(schema, value)
	}
	return esc(textOf(value))
}

func inner(value, schema any, level int, c *ctx) string {
	schema = resolve(schema, c.root)
	switch v := value.(type) {
	case *Object:
		props, pok := asObject(getOr(schema, "properties"))
		keys := v.keys
		if pok {
			keys = props.keys
		}
		var out strings.Builder
		for _, key := range keys {
			x, ok := v.Get(key)
			if !ok {
				continue
			}
			var sub any
			if pok {
				sub = getOr(props, key)
			}
			if viewOf(resolve(sub, c.root)) == "hidden" {
				continue
			}
			fmt.Fprintf(&out, `<div class="nest">%s%s</div>`, headOf(key, resolve(sub, c.root), level), inner(x, sub, level+1, c))
		}
		return out.String()
	case []any:
		item := resolve(getOr(schema, "items"), c.root)
		props, pok := asObject(getOr(item, "properties"))
		if !pok {
			var lis strings.Builder
			for _, x := range v {
				fmt.Fprintf(&lis, "<li>%s</li>", esc(textOf(x)))
			}
			return "<ul>" + lis.String() + "</ul>"
		}
		all := true
		for _, k := range props.keys {
			if !isScalar(resolve(props.vals[k], c.root)) {
				all = false
				break
			}
		}
		if !all {
			var out strings.Builder
			for _, x := range v {
				fmt.Fprintf(&out, `<div class="item">%s</div>`, inner(x, item, level+1, c))
			}
			return out.String()
		}
		// どの行も値を保持しない列は描かない
		type col struct {
			k string
			s any
		}
		var cols []col
		for _, k := range props.keys {
			s := props.vals[k]
			used := false
			for _, x := range v {
				if has(x, k) {
					used = true
					break
				}
			}
			if !used || viewOf(resolve(s, c.root)) == "hidden" {
				continue
			}
			cols = append(cols, col{k, s})
		}
		// 列が1つなら箇条書きにする
		if len(cols) == 1 {
			var lis strings.Builder
			for _, x := range v {
				if y, ok := get(x, cols[0].k); ok {
					fmt.Fprintf(&lis, "<li>%s</li>", cell(cols[0].s, y, c))
				}
			}
			return "<ul>" + lis.String() + "</ul>"
		}
		var th, rows strings.Builder
		for _, cl := range cols {
			fmt.Fprintf(&th, "<th>%s</th>", esc(titleOr(cl.s, cl.k)))
		}
		for _, x := range v {
			rows.WriteString("<tr>")
			for _, cl := range cols {
				fmt.Fprintf(&rows, "<td>%s</td>", cell(cl.s, getOr(x, cl.k), c))
			}
			rows.WriteString("</tr>")
		}
		return `<div class="scroll"><table><thead><tr>` + th.String() + `</tr></thead><tbody>` + rows.String() + `</tbody></table></div>`
	default:
		if has(schema, "oneOf") || viewOf(schema) == "tag" {
			return tag(schema, value)
		}
		t := textOf(value)
		if strings.Contains(t, "\n") {
			return "<pre>" + esc(t) + "</pre>"
		}
		return "<p>" + esc(t) + "</p>"
	}
}

// svgOf は SVG を埋め込む。svg はファイル名（base からの経路）か、SVG そのもの。
func svgOf(p, base string) string {
	if strings.HasPrefix(trimLeft(p), "<svg") {
		return p
	}
	s, err := dataaccess.ReadToString(filepath.Join(base, p))
	if err != nil {
		return ""
	}
	return s
}

// sectionsHTML は論点の並びを、1件を1枚にして描く。
func sectionsHTML(schema, value any, c *ctx) string {
	item := resolve(getOr(schema, "items"), c.root)
	a, _ := asArray(value)
	var out strings.Builder
	for _, v := range a {
		fmt.Fprintf(&out, `<section class="block topic">%s</section>`, fields(item, v, c))
	}
	return out.String()
}

// fields は欄を順に描く。
func fields(schema, value any, c *ctx) string {
	props, ok := asObject(getOr(schema, "properties"))
	if !ok {
		return inner(value, schema, 3, c)
	}
	var out strings.Builder
	for _, k := range props.keys {
		if k == "kind" {
			continue
		}
		if v, ok := get(value, k); ok {
			out.WriteString(field(resolve(props.vals[k], c.root), v, c))
		}
	}
	return out.String()
}

// field は欄1つを、見出しを付けずに描く。
func field(schema, value any, c *ctx) string {
	switch viewOf(schema) {
	case "hidden":
		return ""
	case "heading":
		return "<h2>" + esc(textOf(value)) + "</h2>"
	case "lead":
		return `<p class="claim">` + esc(textOf(value)) + "</p>"
	case "subhead":
		return "<h3>" + esc(textOf(value)) + "</h3>"
	case "paras":
		a, _ := asArray(value)
		var out strings.Builder
		for _, p := range a {
			out.WriteString("<p>" + esc(textOf(p)) + "</p>")
		}
		return out.String()
	case "svg":
		s, _ := value.(string)
		return "<figure>" + svgOf(s, c.base) + "</figure>"
	case "code":
		return `<pre class="code">` + esc(textOf(value)) + "</pre>"
	case "units":
		return unitsHTML(schema, value, c)
	case "blocks":
		return blocksHTML(value, c.base)
	case "steps":
		return stepsHTML(schema, value, c)
	case "table":
		t := clone(value)
		o, ok := t.(*Object)
		if !ok {
			o = NewObject()
		}
		o.Set("kind", "table")
		return blocksHTML([]any{o}, c.base)
	}
	return inner(value, schema, 4, c)
}

// unitsHTML は単位の並びを描く。形は kind の値で items.oneOf から選ぶ。
func unitsHTML(schema, value any, c *ctx) string {
	var forms []any
	if items, ok := get(schema, "items"); ok {
		for _, f := range arrayOf(resolve(items, c.root), "oneOf") {
			forms = append(forms, resolve(f, c.root))
		}
	}
	a, _ := asArray(value)
	var out strings.Builder
	for _, u := range a {
		var form any
		uk, ukok := get(u, "kind")
		for _, f := range forms {
			fk, fkok := pointer(f, "/properties/kind/const")
			if optEqual(fk, fkok, uk, ukok) {
				form = f
				break
			}
		}
		label := ""
		if t, ok := strOf(form, "title"); ok {
			label = `<span class="ulabel">` + esc(t) + "</span>"
		}
		fmt.Fprintf(&out, `<div class="unit">%s%s</div>`, label, fields(form, u, c))
	}
	return out.String()
}

// stepsHTML は手順を番号付きで描く。各段の1つ目の欄を太字にし、残りを添える。
func stepsHTML(schema, value any, c *ctx) string {
	item := resolve(getOr(schema, "items"), c.root)
	var keys []string
	if p, ok := asObject(getOr(item, "properties")); ok {
		keys = p.keys
	}
	a, _ := asArray(value)
	var lis strings.Builder
	for _, v := range a {
		lis.WriteString("<li>")
		first := true
		for _, k := range keys {
			x, ok := get(v, k)
			if !ok {
				continue
			}
			if first {
				lis.WriteString(`<p class="lead">` + esc(textOf(x)) + "</p>")
				first = false
			} else {
				lis.WriteString(`<p class="sub">` + esc(textOf(x)) + "</p>")
			}
		}
		lis.WriteString("</li>")
	}
	return `<ol class="steps">` + lis.String() + "</ol>"
}

// blocksHTML は本文の塊を描く ── 段落 ・ 一覧 ・ 表 ・ コード ・ 図。
func blocksHTML(blocks any, base string) string {
	a, _ := asArray(blocks)
	var out strings.Builder
	for _, b := range a {
		kind, _ := strOf(b, "kind")
		switch kind {
		case "list":
			t := "ul"
			if o, ok := getOr(b, "ordered").(bool); ok && o {
				t = "ol"
			}
			out.WriteString("<" + t + ">")
			for _, x := range arrayOf(b, "items") {
				out.WriteString("<li>" + esc(textOf(x)) + "</li>")
			}
			out.WriteString("</" + t + ">")
		case "table":
			row := func(r any, t string) string {
				ra, _ := asArray(r)
				var s strings.Builder
				for _, x := range ra {
					fmt.Fprintf(&s, "<%s>%s</%s>", t, esc(textOf(x)), t)
				}
				return s.String()
			}
			th := ""
			if h, ok := get(b, "head"); ok {
				th = row(h, "th")
			}
			var trs strings.Builder
			for _, r := range arrayOf(b, "rows") {
				trs.WriteString("<tr>" + row(r, "td") + "</tr>")
			}
			out.WriteString(`<div class="scroll"><table><thead><tr>` + th + `</tr></thead><tbody>` + trs.String() + `</tbody></table></div>`)
		case "figure":
			svg := ""
			if p, ok := strOf(b, "svg"); ok {
				svg = svgOf(p, base)
			}
			capt := ""
			if cp, ok := strOf(b, "caption"); ok {
				capt = "<figcaption>" + esc(cp) + "</figcaption>"
			}
			out.WriteString("<figure>" + svg + capt + "</figure>")
		case "code":
			t, _ := strOf(b, "text")
			out.WriteString(`<pre class="code">` + esc(t) + "</pre>")
		default:
			t, _ := strOf(b, "text")
			out.WriteString("<p>" + esc(t) + "</p>")
		}
	}
	return out.String()
}

// outline は節の入れ子を、見出しと本文の字下げで組む。
func outline(items any, level int, base string) string {
	h := min(max(level+1, 3), 6)
	a, _ := asArray(items)
	var out strings.Builder
	for _, n := range a {
		title, _ := strOf(n, "title")
		body := ""
		if b, ok := get(n, "blocks"); ok {
			body = blocksHTML(b, base)
		}
		kids := ""
		if k, ok := get(n, "sections"); ok {
			kids = outline(k, level+1, base)
		}
		fmt.Fprintf(&out, `<div class="nest"><h%d>%s</h%d>%s%s</div>`, h, esc(title), h, body, kids)
	}
	return out.String()
}

func block(key string, schema, value any, c *ctx) string {
	schema = resolve(schema, c.root)
	head := headOf(key, schema, 2)
	section := func(body string) string { return `<section class="block">` + head + body + "</section>" }
	switch viewOf(schema) {
	case "hidden":
		return ""
	case "outline":
		return section(outline(value, 2, c.base))
	case "blocks":
		return section(blocksHTML(value, c.base))
	case "sections":
		return sectionsHTML(schema, value, c)
	case "group":
		return section(fields(schema, value, c))
	case "card":
		title := esc(titleOr(schema, key))
		if s, ok := value.(string); ok {
			return `<section class="card"><div class="cardhead"><h2>` + title + `</h2></div><p class="lead">` + esc(s) + "</p></section>"
		}
		var tags strings.Builder
		var lines []string
		if props, ok := asObject(getOr(schema, "properties")); ok {
			for _, k := range props.keys {
				v, ok := get(value, k)
				if !ok {
					continue
				}
				p := props.vals[k]
				if viewOf(p) == "tag" {
					tags.WriteString(tag(p, v))
				} else {
					lines = append(lines, esc(textOf(v)))
				}
			}
		}
		var body strings.Builder
		for i, l := range lines {
			cls := "sub"
			if i == 0 {
				cls = "lead"
			}
			fmt.Fprintf(&body, `<p class="%s">%s</p>`, cls, l)
		}
		return `<section class="card"><div class="cardhead"><h2>` + title + "</h2>" + tags.String() + "</div>" + body.String() + "</section>"
	case "figure":
		svg := ""
		if p, ok := strOf(value, "svg"); ok {
			svg = svgOf(p, c.base)
		}
		if svg == "" {
			svg = graphHTML(value)
		}
		capt, _ := strOf(value, "caption")
		return section("<figure><figcaption>" + esc(capt) + "</figcaption>" + svg + "</figure>")
	case "steps":
		return section(stepsHTML(schema, value, c))
	}
	return section(inner(value, schema, 3, c))
}

// pageBody は1件を描画する。x-view が heading の欄を h1、トップの tag の欄を見出しの上の札にする。
func pageBody(schema, value any, c *ctx) string {
	schema = resolve(schema, c.root)
	props, ok := asObject(getOr(schema, "properties"))
	if !ok {
		return inner(value, schema, 2, c)
	}
	rootTitle, _ := strOf(c.root, "title")
	eyebrow := []string{esc(rootTitle)}
	h1 := ""
	var body strings.Builder
	for _, k := range props.keys {
		p := resolve(props.vals[k], c.root)
		v, ok := get(value, k)
		if !ok {
			continue
		}
		switch viewOf(p) {
		case "tag":
			eyebrow = append(eyebrow, tag(p, v))
		case "meta":
			if t := textOf(v); t != "" {
				eyebrow = append(eyebrow, esc(t))
			}
		case "heading":
			h1 = esc(textOf(v))
		default:
			body.WriteString(block(k, p, v, c))
		}
	}
	lead := h1
	if lead == "" {
		if t, ok := strOf(value, "title"); ok {
			lead = esc(t)
		}
	}
	return `<header><p class="eyebrow">` + strings.Join(eyebrow, " ・ ") + "</p><h1>" + lead + "</h1></header>" + body.String()
}

func missing(name string) error {
	return fmt.Errorf("references/%s が無い ── 見た目の正本の複製である。Skill を生んだ道具が置く（その道具の check が正本との差を報告する）", name)
}

// paletteVars は明と暗の色の変数を組む。共通のパレットの後ろに Skill 固有のトークンを足し、鍵の順に並べる。
func paletteVars(refs string) (string, string, error) {
	tokens, err := readJSON(filepath.Join(refs, viewTokens))
	if err != nil {
		return "", "", missing(viewTokens)
	}
	name, ok := strOf(tokens, "palette")
	if !ok {
		return "", "", fmt.Errorf("%s: palette が無い", viewTokens)
	}
	chosen, ok := get(getOr(tokens, "palettes"), name)
	if !ok {
		return "", "", fmt.Errorf("%s: palettes に %s が無い", viewTokens, name)
	}
	sources := []any{chosen}
	if skill, err := readJSON(filepath.Join(refs, skillTokens)); err == nil {
		sources = append(sources, skill)
	}
	var out [2]string
	for i, scheme := range []string{"light", "dark"} {
		type kv struct{ k, v string }
		var vars []kv
		for _, src := range sources {
			o, ok := asObject(getOr(src, scheme))
			if !ok {
				continue
			}
			for _, k := range o.keys {
				v, ok := o.vals[k].(string)
				if !ok {
					continue
				}
				kept := vars[:0]
				for _, x := range vars {
					if x.k != k {
						kept = append(kept, x)
					}
				}
				vars = append(kept, kv{k, v})
			}
		}
		sort.Slice(vars, func(a, b int) bool {
			if vars[a].k != vars[b].k {
				return vars[a].k < vars[b].k
			}
			return vars[a].v < vars[b].v
		})
		var s strings.Builder
		for _, x := range vars {
			s.WriteString("--" + x.k + ":" + x.v + ";")
		}
		out[i] = s.String()
	}
	return out[0], out[1], nil
}

func styleFor(refs, selector string) (string, error) {
	light, dark, err := paletteVars(refs)
	if err != nil {
		return "", err
	}
	css, err := dataaccess.ReadToString(filepath.Join(refs, viewCSS))
	if err != nil {
		return "", missing(viewCSS)
	}
	own, _ := dataaccess.ReadToString(filepath.Join(refs, skillCSS))
	return selector + "{" + light + "}@media (prefers-color-scheme:dark){" + selector + "{" + dark + "}}" + css + own, nil
}

// ScopedStyle は .rv の囲みの中だけに適用される規則と色を返す。他の型の頁に埋め込むときに使う。
func ScopedStyle(refs string) (string, error) { return styleFor(refs, ".rv") }

// RenderBody は1件の本文を組む（.rv の囲みを含む）。
func RenderBody(schema, value any, base string) string {
	merged := mergeRefs(schema, schema, 0)
	return `<div class="rv">` + pageBody(merged, value, &ctx{root: merged, base: base}) + "</div>"
}

// View は種類の JSON（か、その id の1件か、渡された JSON）を HTML にする。
func View(refs, kind string, id, file *string) (string, error) {
	root0, err := readJSON(filepath.Join(refs, kind+SchemaTail))
	if err != nil {
		return "", err
	}
	root := mergeRefs(root0, root0, 0)
	var value any
	var base string
	if file != nil {
		value, err = readJSON(*file)
		if err != nil {
			return "", err
		}
		base = filepath.Dir(*file)
	} else {
		value, err = Get(refs, kind, nil)
		if err != nil {
			return "", err
		}
		base = refs
	}
	c := &ctx{root: root, base: base}
	top := resolve(root, root)
	var body string
	if id != nil {
		var one any
		if file != nil {
			found := false
			if a, ok := itemsField(value); ok {
				for _, x := range a {
					if s, ok := strOf(x, "id"); ok && s == *id {
						one, found = x, true
						break
					}
				}
			}
			if !found {
				return "", fmt.Errorf("id %s の項目が無い", *id)
			}
		} else {
			one, err = Get(refs, kind, id)
			if err != nil {
				return "", err
			}
		}
		itemsSchema, ok := pointer(top, "/properties/items/items")
		if !ok {
			itemsSchema = getOr(top, "items")
		}
		body = pageBody(itemsSchema, one, c)
	} else {
		plain := clone(value)
		po, isObj := plain.(*Object)
		if isObj {
			po.Delete("$schema")
		}
		onlyItems := isObj && po.Len() == 1
		list, lok := asArray(getOr(plain, "items"))
		itemSchema, sok := pointer(top, "/properties/items/items")
		if onlyItems && lok && sok {
			var b strings.Builder
			for _, it := range list {
				b.WriteString(`<section class="entry">` + pageBody(itemSchema, it, c) + "</section>")
			}
			body = b.String()
		} else {
			body = pageBody(root, plain, c)
		}
	}
	template, err := dataaccess.ReadToString(filepath.Join(refs, viewTemplate))
	if err != nil {
		return "", missing(viewTemplate)
	}
	t, ok := strOf(root, "title")
	if !ok {
		t = kind
	}
	style, err := styleFor(refs, ":root")
	if err != nil {
		return "", err
	}
	out := strings.ReplaceAll(template, "{{title}}", esc(t))
	out = strings.ReplaceAll(out, "{{style}}", style)
	return strings.ReplaceAll(out, "{{body}}", body), nil
}
