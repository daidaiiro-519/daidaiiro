// SPDX-License-Identifier: MIT

// Package service はサービス層である。道具の一覧（能力の正本）であり、業務ロジック層だけを参照する。
//
// 戻り値は3つの欄を持つ ── ok（正常に終わったか）・ findings（検出したもの。誤りではない）・
// data（機械が読む本体）。
package service

import (
	"fmt"
	"path/filepath"

	bl "trialgo/businesslogic"
	"trialgo/dataaccess"
)

// Arg は引数1つである。位置引数と旗を区別しない。
type Arg struct {
	Name     string
	Summary  string
	Required bool
	Many     bool
	Default  *string
}

// Need は省略できない引数を宣言する。
func Need(name, summary string) Arg { return Arg{Name: name, Summary: summary, Required: true} }

// Opt は省略できる引数を宣言する。
func Opt(name, summary string, def *string) Arg {
	return Arg{Name: name, Summary: summary, Default: def}
}

// Given は渡された引数である。プレゼンテーション層が組み、道具が読む。
type Given struct {
	values map[string][]string
}

// Push は1つの値を足す。
func (g *Given) Push(name, value string) {
	if g.values == nil {
		g.values = map[string][]string{}
	}
	g.values[name] = append(g.values[name], value)
}

// One は名前に対する最初の値を返す。無ければ既定を返す。
func (g *Given) One(name, def string) string {
	if v := g.values[name]; len(v) > 0 {
		return v[0]
	}
	return def
}

// Has は名前に対する値が在るかを返す。
func (g *Given) Has(name string) bool {
	_, ok := g.values[name]
	return ok
}

// All は名前に対する値を全部返す。
func (g *Given) All(name string) []string { return g.values[name] }

// SkillRoot はこの Skill の置き場所である。skill_root が渡されればそれを優先し、
// 渡されなければ実行ファイルの1つ上（bin/ の親）に SKILL.md が在るかで求める。
func (g *Given) SkillRoot() (string, error) {
	if g.Has("skill_root") {
		return g.One("skill_root", "."), nil
	}
	exe, err := dataaccess.Executable()
	if err != nil {
		return "", fmt.Errorf("実行ファイルの位置を取れない ── %v", err)
	}
	root := filepath.Dir(filepath.Dir(exe))
	if dataaccess.IsFile(filepath.Join(root, "SKILL.md")) {
		return root, nil
	}
	return "", fmt.Errorf("Skill の置き場所が見つからない ── %s に SKILL.md が無い。--skill_root で渡す", root)
}

// Outcome は道具の戻り値である。印字はしない。
type Outcome struct {
	OK       bool
	Findings []string
	Data     any
}

// Found は検出を伴う正常な結果を組む。
func Found(findings []string, data any) Outcome {
	if findings == nil {
		findings = []string{}
	}
	return Outcome{OK: true, Findings: findings, Data: data}
}

// Misuse は誤用を組む。
func Misuse(reason error) Outcome {
	return Outcome{OK: false, Findings: []string{reason.Error()}, Data: bl.NewObject()}
}

// ToJSON は機械が読む形へ組む。
func (o Outcome) ToJSON() any {
	f := make([]any, len(o.Findings))
	for i, x := range o.Findings {
		f[i] = x
	}
	return Obj("ok", o.OK, "findings", f, "data", o.Data)
}

// ExitCode は終了コードを返す ── 0 正常 ／ 1 検出あり ／ 2 誤用。
func (o Outcome) ExitCode() int {
	if !o.OK {
		return 2
	}
	if len(o.Findings) == 0 {
		return 0
	}
	return 1
}

// Tool は道具1つである。Run は結果を返し、Human は人向けの文を組む。
type Tool struct {
	Name    string
	Summary string
	Args    []Arg
	Run     func(*Given) Outcome
	Human   func(Outcome) string
}

// Obj は欄の順を保つ対象を組む。
func Obj(kv ...any) *bl.Object {
	o := bl.NewObject()
	for i := 0; i+1 < len(kv); i += 2 {
		o.Set(kv[i].(string), kv[i+1])
	}
	return o
}

// Catalog は道具の一覧を、機械が読む形で返す。動詞なしで --json を付けたときに CLI が返す。
func Catalog(all []Tool, given *Given) Outcome {
	tools := []any{}
	for _, t := range all {
		args := []any{}
		for _, a := range t.Args {
			args = append(args, Obj("name", a.Name, "required", a.Required, "many", a.Many))
		}
		tools = append(tools, Obj("name", t.Name, "summary", t.Summary, "args", args))
	}
	root, err := given.SkillRoot()
	if err != nil {
		root = ""
	}
	return Found(nil, Obj("tools", tools, "skill_root", root))
}

// Tools は道具の一覧である。この Skill 固有の道具は、ここへ足す。
func Tools() []Tool { return RefsTools() }

// Pretty は JSON を2字の字下げで文字列にする。
func Pretty(v any) string { return bl.Pretty(v) }
