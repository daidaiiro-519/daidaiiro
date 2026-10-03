// SPDX-License-Identifier: MIT

package service

// references の道具を、道具の一覧に載せる。どの Skill も同じ4つを持つ ── 取り出す ・ 検査する ・
// 描画する ・ 取り込む。実体は業務ロジック層が持つ。

import (
	"fmt"
	"path/filepath"
	"strings"

	bl "trialgo/businesslogic"
)

func refsDir(g *Given) (string, error) {
	root, err := g.SkillRoot()
	if err != nil {
		return "", err
	}
	return filepath.Join(root, "references"), nil
}

func opt(g *Given, name string) *string {
	s := g.One(name, "")
	if s == "" {
		return nil
	}
	return &s
}

func runGet(g *Given) Outcome {
	dir, err := refsDir(g)
	if err != nil {
		return Misuse(err)
	}
	got, err := bl.Get(dir, g.One("kind", ""), opt(g, "id"))
	if err != nil {
		return Misuse(err)
	}
	// 除く欄を渡すと、入れ子のすべてから除いて返す（例：omit=source）
	var keys []string
	if o := opt(g, "omit"); o != nil {
		for _, k := range strings.Split(*o, ",") {
			if k = strings.TrimSpace(k); k != "" {
				keys = append(keys, k)
			}
		}
	}
	return Found(nil, bl.Omit(got, keys))
}

func runValidate(g *Given) Outcome {
	dir, err := refsDir(g)
	if err != nil {
		return Misuse(err)
	}
	var found []string
	if file := opt(g, "file"); file != nil {
		found, err = bl.ValidateFile(dir, g.One("kind", ""), *file)
	} else {
		found, err = bl.Validate(dir)
	}
	if err != nil {
		return Misuse(err)
	}
	ks, err := bl.Kinds(dir)
	if err != nil {
		return Misuse(err)
	}
	kinds := []any{}
	for _, k := range ks {
		kinds = append(kinds, k.Name)
	}
	// 何を検査したかを返す ── 合格したのか、検査が実行されなかったのかを区別させる
	checked := "references"
	if f := opt(g, "file"); f != nil {
		checked = *f
	}
	return Found(found, Obj("kinds", kinds, "checked", checked))
}

func humanValidate(o Outcome) string {
	if !o.OK || len(o.Findings) > 0 {
		return human(o)
	}
	checked := "references"
	if d, ok := o.Data.(*bl.Object); ok {
		if v, ok := d.Get("checked"); ok {
			if s, ok := v.(string); ok {
				checked = s
			}
		}
	}
	return fmt.Sprintf("合格 ── %s に検出は無い", checked)
}

func runView(g *Given) Outcome {
	dir, err := refsDir(g)
	if err != nil {
		return Misuse(err)
	}
	html, err := bl.View(dir, g.One("kind", ""), opt(g, "id"), opt(g, "file"))
	if err != nil {
		return Misuse(err)
	}
	if out := opt(g, "out"); out != nil {
		if err := bl.Save(*out, html); err != nil {
			return Misuse(err)
		}
		return Found(nil, Obj("out", *out, "bytes", len(html)))
	}
	return Found(nil, Obj("html", html))
}

func runImport(g *Given) Outcome {
	dir, err := refsDir(g)
	if err != nil {
		return Misuse(err)
	}
	file := g.One("file", "")
	text, err := bl.ReadText(file)
	if err != nil {
		return Misuse(err)
	}
	doc := bl.ImportMarkdown(g.One("id", ""), g.One("source", file), g.One("fetched", ""), text)
	path, err := bl.PutDocument(dir, doc)
	if err != nil {
		return Misuse(err)
	}
	found, err := bl.Validate(dir)
	if err != nil {
		return Misuse(err)
	}
	return Found(found, Obj("path", path))
}

func human(o Outcome) string {
	if !o.OK || len(o.Findings) > 0 {
		lines := make([]string, len(o.Findings))
		for i, x := range o.Findings {
			lines[i] = "  ×  " + x
		}
		return strings.Join(lines, "\n")
	}
	if d, ok := o.Data.(*bl.Object); ok {
		if v, ok := d.Get("html"); ok {
			if s, ok := v.(string); ok {
				return s
			}
		}
	}
	return Pretty(o.Data)
}

// RefsTools は references の4つの道具である。
func RefsTools() []Tool {
	root := Opt("skill_root", "この Skill の置き場所（既定は、実行ファイルの1つ上）", nil)
	return []Tool{
		{Name: "get", Summary: "references の種類の JSON を取り出す。id を渡すと、その1件だけを返す",
			Args: []Arg{Need("kind", "種類の名前"), Opt("id", "項目の id", nil), Opt("omit", "除く欄の名前（, で区切る。例：source）", nil), root},
			Run:  runGet, Human: human},
		{Name: "validate", Summary: "references の JSON を、指しているスキーマで検査する。file を渡すと、その JSON を種類のスキーマで検査する",
			Args: []Arg{Opt("kind", "種類の名前（file と一緒に渡す）", nil), Opt("file", "検査する JSON", nil), root},
			Run:  runValidate, Human: humanValidate},
		{Name: "view", Summary: "references の種類の JSON を、スキーマの title と x-view に従って HTML に描画する",
			Args: []Arg{Need("kind", "種類の名前"), Opt("id", "項目の id", nil), Opt("file", "描画する JSON（回答など）", nil), Opt("out", "HTML の置き場所", nil), root},
			Run:  runView, Human: human},
		{Name: "import", Summary: "Markdown の文書を、見出しを節の入れ子に分けて document へ取り込む",
			Args: []Arg{Need("file", "取り込む Markdown"), Need("id", "document の id"), Opt("source", "元の場所（既定は file）", nil), Opt("fetched", "取り込んだ日", nil), root},
			Run:  runImport, Human: human},
	}
}
