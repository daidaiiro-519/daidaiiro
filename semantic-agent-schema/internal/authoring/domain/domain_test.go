package domain_test

import (
	"encoding/json"
	"errors"
	"slices"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

// codeOf は、err の Failure の code を返す。Failure でなければ空を返す。
func codeOf(err error) string {
	var f *domain.Failure
	if errors.As(err, &f) {
		return string(f.Code)
	}
	return ""
}

func TestNewPathAcceptsWorkingTreePaths(t *testing.T) {
	t.Parallel()
	for _, value := range []string{"a.json", "dir/.hidden/a.json", "..a/b.json"} {
		got, err := domain.NewPath(value)
		if err != nil || got.String() != value {
			t.Errorf("NewPath(%q) = %v, %v, want %q, nil", value, got, err, value)
		}
	}
}

func TestNewPathRejectsPathsOutsideWorkingTree(t *testing.T) {
	t.Parallel()
	for _, value := range []string{"/etc/a.json", "../a.json", "dir/../../a.json", "./a.json", "dir//a.json", ""} {
		_, err := domain.NewPath(value)
		if got := codeOf(err); got != "path_outside" {
			t.Errorf("NewPath(%q) code = %q, want %q", value, got, "path_outside")
		}
	}
}

func TestNewPointerRejectsInvalidForm(t *testing.T) {
	t.Parallel()
	for _, value := range []string{"a", "/a~2", "/a~"} {
		_, err := domain.NewPointer(value)
		if got := codeOf(err); got != "schema_violation" {
			t.Errorf("NewPointer(%q) code = %q, want %q", value, got, "schema_violation")
		}
	}
}

func TestPointerOfEscapes(t *testing.T) {
	t.Parallel()
	if got, want := domain.PointerOf("a/b", "c~d", "0").String(), "/a~1b/c~0d/0"; got != want {
		t.Errorf("PointerOf(a/b, c~d, 0) = %q, want %q", got, want)
	}
}

func TestNewConcreteRejectsDocumentsWithoutID(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name, content, wantCode string
	}{
		{name: "JSON として読めない", content: `{`, wantCode: "invalid_json"},
		{name: "$id が無い", content: `{"type": "object"}`, wantCode: "schema_violation"},
		{name: "オブジェクトでない", content: `[]`, wantCode: "schema_violation"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			_, err := domain.NewConcrete(nil, []byte(c.content))
			if got := codeOf(err); got != c.wantCode {
				t.Errorf("NewConcrete(%s) code = %q, want %q", c.content, got, c.wantCode)
			}
		})
	}
}

func TestConventionFindsNoViolation(t *testing.T) {
	t.Parallel()
	cases := []struct{ name, concrete string }{
		{name: "作法を満たす Concrete", concrete: `{"$id": "urn:a", "properties": {"$schema": {"const": "urn:a"},
			"x": {"x-prompt": {"rules": [{"id": "r1", "related_fields": ["/y"], "bad": [{"value": 1, "related": {"/y": 2}}], "good": []}]}},
			"y": {"x-prompt": {"rules": [{"id": "r2", "related_fields": [], "bad": [], "good": []}]}}}}`},
		{name: "const の中の x-prompt はサブスキーマでない", concrete: `{"$id": "urn:a",
			"const": {"x-prompt": {"rules": [{"id": "r1"}, {"id": "r1"}]}}}`},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			if got := conventionLocations(t, c.concrete); len(got) != 0 {
				t.Errorf("Convention(%s) = %v, want none", c.name, got)
			}
		})
	}
}

func TestConventionFindsViolations(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name, concrete string
		want           []string
	}{
		{name: "$schema の const が $id と違う", concrete: `{"$id": "urn:a", "properties": {"$schema": {"const": "urn:b"}}}`,
			want: []string{"/properties/$schema/const"}},
		{name: "rule の id が $defs と items をまたいで重なる", concrete: `{"$id": "urn:a",
			"$defs": {"d": {"x-prompt": {"rules": [{"id": "r1"}]}}},
			"items": {"x-prompt": {"rules": [{"id": "r1"}]}}}`,
			want: []string{"/items/x-prompt/rules/0/id"}},
		{name: "例の related のキーが related_fields に無い", concrete: `{"$id": "urn:a",
			"allOf": [{"x-prompt": {"rules": [{"id": "r1", "related_fields": [], "bad": [], "good": [{"value": 1, "related": {"/z": 1}}]}]}}]}`,
			want: []string{"/allOf/0/x-prompt/rules/0/related_fields"}},
		{name: "bad と good の例の related に related_fields に無いキーがあっても、同じ rule では1件", concrete: `{"$id": "urn:a",
			"x-prompt": {"rules": [{"id": "r1", "related_fields": ["/start"],
				"bad": [{"value": 1, "related": {"/start": 2, "/other": 1}}], "good": [{"value": 3, "related": {"/start": 2, "/z": 1}}]}]}}`,
			want: []string{"/x-prompt/rules/0/related_fields"}},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			if diff := cmp.Diff(c.want, conventionLocations(t, c.concrete)); diff != "" {
				t.Errorf("Convention(%s) mismatch (-want +got):\n%s", c.name, diff)
			}
		})
	}
}

func conventionLocations(t *testing.T, concrete string) []string {
	t.Helper()
	value, err := domain.ParseJSON([]byte(concrete))
	if err != nil {
		t.Fatalf("ParseJSON(%s) = %v", concrete, err)
	}
	var got []string
	for _, v := range domain.Convention(value) {
		got = append(got, v.InstanceLocation.String())
	}
	return got
}

func TestChangedLocationsComparesEachInstanceLocation(t *testing.T) {
	t.Parallel()
	before, err := domain.ParseJSON([]byte(`{"a": 1, "b": {"c": 1.0}, "d": [1]}`))
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	after, err := domain.ParseJSON([]byte(`{"a": 1, "b": {"c": 1}, "e": true}`))
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	var got []string
	for k := range domain.ChangedLocations(before, after) {
		got = append(got, k)
	}
	slices.Sort(got)
	want := []string{"", "/b", "/b/c", "/d", "/d/0", "/e"}
	if !slices.Equal(got, want) {
		t.Errorf("ChangedLocations = %v, want %v", got, want)
	}
}

func judgment(t *testing.T, rule, location, verdict, by string) domain.Judgment {
	t.Helper()
	j, err := domain.NewJudgment(rule, location, verdict, by, "")
	if err != nil {
		t.Fatalf("NewJudgment(%s, %s, %s, %s) = %v", rule, location, verdict, by, err)
	}
	return j
}

// evaluated は、機械的な検証が返す組（x-prompt を持つサブスキーマの keyword location と、当たった instance location）を作る。
// 組は「keyword location instance location」の文字列で書く。
func evaluated(t *testing.T, pairs ...string) []domain.Evaluation {
	t.Helper()
	var evaluations []domain.Evaluation
	for _, pair := range pairs {
		keyword, instance, _ := strings.Cut(pair, " ")
		k, err := domain.NewPointer(keyword)
		if err != nil {
			t.Fatalf("NewPointer(%s) = %v", keyword, err)
		}
		i, err := domain.NewPointer(instance)
		if err != nil {
			t.Fatalf("NewPointer(%s) = %v", instance, err)
		}
		evaluations = append(evaluations, domain.Evaluation{KeywordLocation: k, InstanceLocation: i})
	}
	return evaluations
}

func concrete(t *testing.T, content string) domain.Concrete {
	t.Helper()
	c, err := domain.NewConcrete(nil, []byte(content))
	if err != nil {
		t.Fatalf("NewConcrete(%s) = %v", content, err)
	}
	return c
}

func previous(t *testing.T, content string, hash domain.Hash, judgments ...domain.Judgment) *domain.Previous {
	t.Helper()
	p, err := domain.NewPrevious([]byte(content), hash.String(), judgments)
	if err != nil {
		t.Fatalf("NewPrevious = %v", err)
	}
	return p
}

// デシジョンテーブル：前回の検証の有無 × Concrete のハッシュ × instance location と related_fields が変更点か × judgment を出した者。
func TestInheritedDecisionTable(t *testing.T) {
	t.Parallel()
	schema := concrete(t, `{"$id": "urn:a", "properties": {
		"title": {"x-prompt": {"rules": [{"id": "r-title", "related_fields": []}]}},
		"summary": {"x-prompt": {"rules": [{"id": "r-summary", "related_fields": ["/title"]}]}},
		"items": {"x-prompt": {"rules": [{"id": "r-items", "related_fields": []}]}}}}`)
	other := concrete(t, `{"$id": "urn:a", "title": "別の版"}`)
	before := `{"title": "A", "summary": "s", "items": [1, 2]}`
	judgments := []domain.Judgment{
		judgment(t, "r-title", "/title", "verified", "ai"),
		judgment(t, "r-summary", "/summary", "verified", "ai"),
		judgment(t, "r-items", "/items", "unverified", "ai"),
		judgment(t, "urn:a#/minimum", "/items/1", "violated", "machine"),
		judgment(t, "r-gone", "/items", "verified", "ai"),
	}
	cases := []struct {
		name     string
		previous *domain.Previous
		current  string
		want     []string
	}{
		{name: "前回の検証が無ければ引き継がない", current: before},
		{name: "Concrete のハッシュが違えば引き継がない", previous: previous(t, before, other.Hash(), judgments...), current: before},
		{name: "変更点が無ければ Concrete にある rule の AI の judgment を全部引き継ぐ",
			previous: previous(t, before, schema.Hash(), judgments...), current: before,
			want: []string{"r-title", "r-summary", "r-items"}},
		{name: "instance location か related_fields が変更点の judgment は引き継がない",
			previous: previous(t, before, schema.Hash(), judgments...), current: `{"title": "B", "summary": "s", "items": [1, 2]}`,
			want: []string{"r-items"}},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			value, err := domain.ParseJSON([]byte(c.current))
			if err != nil {
				t.Fatalf("ParseJSON(%s) = %v", c.current, err)
			}
			var got []string
			for _, j := range domain.Inherited(c.previous, value, schema) {
				got = append(got, j.Rule().ID())
			}
			if !slices.Equal(got, c.want) {
				t.Errorf("Inherited(%s) = %v, want %v", c.name, got, c.want)
			}
		})
	}
}

func TestCountsOnlyVerifiedIsSatisfied(t *testing.T) {
	t.Parallel()
	schema := concrete(t, `{"$id": "urn:x", "properties": {
		"a": {"x-prompt": {"rules": [{"id": "r1"}]}}, "b": {"x-prompt": {"rules": [{"id": "r2"}]}},
		"c": {"x-prompt": {"rules": [{"id": "r3"}]}}}}`)
	content := []byte(`{"a": 1, "b": 2, "c": 3}`)
	value, err := domain.ParseJSON(content)
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	path, err := domain.NewPath("x.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	v := domain.NewValidation(domain.ValidationInput{
		Document: domain.Document{Path: path, Content: content, Hash: domain.HashOf(content)}, Value: value,
		Concrete:   schema,
		Violations: []domain.Violation{{InstanceLocation: domain.PointerOf("a"), AbsoluteKeywordLocation: "urn:x#/minimum"}},
		Previous: previous(t, string(content), schema.Hash(),
			judgment(t, "r1", "/a", "verified", "ai"),
			judgment(t, "r2", "/b", "unverified", "ai"),
			judgment(t, "r3", "/c", "not-judged", "ai")),
	})
	want := domain.Counts{Verified: 1, Violated: 1, Unverified: 1, NotJudged: 1}
	if got := v.Counts(); got != want {
		t.Errorf("Counts() = %+v, want %+v", got, want)
	}
}

func TestNewJudgmentRejectsValuesOutsideClosedLists(t *testing.T) {
	t.Parallel()
	cases := []struct{ name, rule, location, verdict, by string }{
		{name: "rule の id が空", rule: "", location: "/a", verdict: "verified", by: "ai"},
		{name: "instance location が JSON Pointer でない", rule: "r", location: "a", verdict: "verified", by: "ai"},
		{name: "verdict が閉じた一覧に無い", rule: "r", location: "/a", verdict: "ok", by: "ai"},
		{name: "judgment を出した者が閉じた一覧に無い", rule: "r", location: "/a", verdict: "verified", by: "human"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			if _, err := domain.NewJudgment(c.rule, c.location, c.verdict, c.by, ""); err == nil {
				t.Errorf("NewJudgment(%q, %q, %q, %q) = nil error, want error", c.rule, c.location, c.verdict, c.by)
			}
		})
	}
}

const memo = `{"$id": "urn:example:memo:v1", "title": "メモ", "properties": {
	"$schema": {"const": "urn:example:memo:v1"},
	"title": {"type": "string", "title": "題名", "x-prompt": {"meaning": "題名", "when": "書くとき", "rules": []}},
	"items": {"type": "array", "items": {"title": "品", "properties": {"checked": {"type": "boolean", "default": false, "x-decided-by": "human"}}}},
	"status": {"enum": ["draft", "approved"], "default": "draft", "x-decided-by": "human"},
	"owner": {"type": "string", "x-decided-by": "human"}}}`

func TestNewCreatePromptMakesStepsFromProperties(t *testing.T) {
	t.Parallel()
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	p := domain.NewCreatePrompt(path, concrete(t, memo))
	var got []string
	for _, step := range p.Steps() {
		got = append(got, step.KeywordLocation.String()+" "+step.Title)
	}
	want := []string{" メモ", "/properties/items/items 品", "/properties/title 題名"}
	if !slices.Equal(got, want) {
		t.Errorf("NewCreatePrompt(memo).Steps() = %q, want %q", got, want)
	}
	if p.ResponseSchemaID() != "urn:example:memo:v1" || p.Path().String() != "memo.json" {
		t.Errorf("NewCreatePrompt(memo) = %s, %s, want urn:example:memo:v1, memo.json", p.ResponseSchemaID(), p.Path())
	}
}

// デシジョンテーブル：x-decided-by: human の instance location の値（無い ・ default ・ default と違う ・ default が無いのに有る）。
func TestNewCreationDecidedByHuman(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name          string
		value         string
		wantLocations []string
	}{
		{name: "値を書かない", value: `{"title": "t"}`},
		{name: "default のまま書く", value: `{"status": "draft"}`},
		{name: "default と違う値を書く", value: `{"status": "approved"}`, wantLocations: []string{"/status"}},
		{name: "default の無い値を書く", value: `{"owner": "x", "status": "approved"}`, wantLocations: []string{"/owner", "/status"}},
		{name: "配列の要素の値を default から変える", value: `{"items": [{"checked": false}, {}, {"checked": true}]}`, wantLocations: []string{"/items/2/checked"}},
	}
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			value, err := domain.ParseJSON([]byte(c.value))
			if err != nil {
				t.Fatalf("ParseJSON(%s) = %v", c.value, err)
			}
			_, err = domain.NewCreation(domain.WriteInput{
				Path: path, Content: []byte(c.value), Value: value, Concrete: concrete(t, memo),
			})
			var got []string
			for _, e := range unjoin(err) {
				var f *domain.Failure
				if errors.As(e, &f) && f.Code == domain.DecidedByHuman {
					got = append(got, f.InstanceLocation)
				}
			}
			if !slices.Equal(got, c.wantLocations) {
				t.Errorf("NewCreation(%s) decided_by_human = %v, want %v", c.value, got, c.wantLocations)
			}
		})
	}
}

func unjoin(err error) []error {
	if joined, ok := err.(interface{ Unwrap() []error }); ok {
		return joined.Unwrap()
	}
	if err == nil {
		return nil
	}
	return []error{err}
}

func TestNewCreationAddsEveryInstanceLocation(t *testing.T) {
	t.Parallel()
	content := []byte(`{"title": "t", "tags": ["a"]}`)
	value, err := domain.ParseJSON(content)
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	c, err := domain.NewCreation(domain.WriteInput{Path: path, Content: content, Value: value, Concrete: concrete(t, memo)})
	if err != nil {
		t.Fatalf("NewCreation = %v", err)
	}
	type change struct{ Location, Op, Before, After string }
	var got []change
	for _, ch := range c.Changes() {
		got = append(got, change{ch.InstanceLocation().String(), string(ch.Op()), ch.Before(), ch.After()})
	}
	want := []change{
		{"", "add", "", `{"tags":["a"],"title":"t"}`},
		{"/tags", "add", "", `["a"]`},
		{"/tags/0", "add", "", `"a"`},
		{"/title", "add", "", `"t"`},
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("NewCreation(%s).Changes() mismatch (-want +got):\n%s", content, diff)
	}
}

// デシジョンテーブル：x-decided-by: human の値の前後（変えない ・ 変える ・ 足す ・ 消す ・ 配列の要素で変える）。
func TestNewUpdateDecidedByHuman(t *testing.T) {
	t.Parallel()
	before := `{"title": "t", "status": "draft", "items": [{"checked": false}]}`
	cases := []struct {
		name          string
		after         string
		wantLocations []string
	}{
		{name: "x-decided-by: human の値を変えない", after: `{"title": "u", "status": "draft", "items": [{"checked": false}]}`},
		{name: "値を変える", after: `{"title": "t", "status": "approved", "items": [{"checked": false}]}`, wantLocations: []string{"/status"}},
		{name: "値を足す", after: `{"title": "t", "status": "draft", "owner": "x", "items": [{"checked": false}]}`, wantLocations: []string{"/owner"}},
		{name: "値を消す", after: `{"title": "t", "items": [{"checked": false}]}`, wantLocations: []string{"/status"}},
		{name: "配列の要素の値を変える", after: `{"title": "t", "status": "draft", "items": [{"checked": true}]}`, wantLocations: []string{"/items/0/checked"}},
	}
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	beforeValue, err := domain.ParseJSON([]byte(before))
	if err != nil {
		t.Fatalf("ParseJSON(%s) = %v", before, err)
	}
	document := domain.Document{Path: path, Content: []byte(before), Hash: domain.HashOf([]byte(before))}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			value, err := domain.ParseJSON([]byte(c.after))
			if err != nil {
				t.Fatalf("ParseJSON(%s) = %v", c.after, err)
			}
			w, err := domain.NewUpdate(domain.WriteInput{
				Path: path, Content: []byte(c.after), Value: value, Concrete: concrete(t, memo),
			}, document, beforeValue)
			var got []string
			for _, e := range unjoin(err) {
				var f *domain.Failure
				if errors.As(e, &f) && f.Code == domain.DecidedByHuman {
					got = append(got, f.InstanceLocation)
				}
			}
			if !slices.Equal(got, c.wantLocations) {
				t.Errorf("NewUpdate(%s) decided_by_human = %v, want %v", c.after, got, c.wantLocations)
			}
			if err == nil && w.BeforeHash().String() != document.Hash.String() {
				t.Errorf("NewUpdate(%s).BeforeHash() = %s, want %s", c.after, w.BeforeHash(), document.Hash)
			}
		})
	}
}

func TestMatchHashRejectsChangeAfterRead(t *testing.T) {
	t.Parallel()
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	current := domain.HashOf([]byte("a"))
	if err := domain.MatchHash(path, current, domain.HashOf([]byte("a"))); err != nil {
		t.Errorf("MatchHash(same) = %v, want nil", err)
	}
	other := domain.HashOf([]byte("b"))
	if got := codeOf(domain.MatchHash(path, current, other)); got != "hash_mismatch" {
		t.Errorf("MatchHash(other) code = %q, want %q", got, "hash_mismatch")
	}
}

func TestNewUpdatePromptMakesStepsFromInstanceLocations(t *testing.T) {
	t.Parallel()
	content := []byte(`{"title": "t", "items": [{"checked": false}, {}]}`)
	value, err := domain.ParseJSON(content)
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	document := domain.Document{Path: path, Content: content, Hash: domain.HashOf(content)}
	p := domain.NewUpdatePrompt(domain.UpdateInput{Document: document, Value: value, Concrete: concrete(t, memo), PatchID: "urn:patch"})
	var got []string
	for _, step := range p.Steps() {
		got = append(got, step.InstanceLocation.String()+" "+step.KeywordLocation.String())
	}
	want := []string{" ", "/items/0 /properties/items/items", "/items/1 /properties/items/items", "/title /properties/title"}
	var subschemas []string
	for _, sub := range p.Subschemas() {
		subschemas = append(subschemas, sub.KeywordLocation.String()+" "+sub.Title)
	}
	if wantSubschemas := []string{" メモ", "/properties/items/items 品", "/properties/title 題名"}; !slices.Equal(subschemas, wantSubschemas) {
		t.Errorf("NewUpdatePrompt(memo).Subschemas() = %q, want %q", subschemas, wantSubschemas)
	}
	if !slices.Equal(got, want) {
		t.Errorf("NewUpdatePrompt(memo).Steps() = %q, want %q", got, want)
	}
	if p.ResponseSchemaID() != "urn:patch" {
		t.Errorf("NewUpdatePrompt(memo).ResponseSchemaID() = %s, want urn:patch", p.ResponseSchemaID())
	}
}

// 直す対象が AI の判定の違反なら、step の keyword location は、その rule を置いたサブスキーマ（properties ・ items で
// たどれない $defs の中でも）にする。
func TestNewUpdatePromptPointsViolationsAtRuleSubschema(t *testing.T) {
	t.Parallel()
	c := concrete(t, `{"$id": "urn:example:meta:v1", "$defs": {"keywords": {"properties": {"x-prompt": {"properties": {
		"meaning": {"x-prompt": {"meaning": "m", "rules": [{"id": "meaning-domain"}]}}}}}}}}`)
	content := []byte(`{"properties": {"title": {"x-prompt": {"meaning": "題名"}}}}`)
	value, err := domain.ParseJSON(content)
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	path, err := domain.NewPath("schemas/memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	document := domain.Document{Path: path, Content: content, Hash: domain.HashOf(content)}
	violated := judgment(t, "meaning-domain", "/properties/title/x-prompt/meaning", "violated", "ai")
	p := domain.NewUpdatePrompt(domain.UpdateInput{Document: document, Value: value, Concrete: c, Violations: []domain.Judgment{violated}})
	var got []string
	for _, step := range p.Steps() {
		got = append(got, step.InstanceLocation.String()+" "+step.KeywordLocation.String())
	}
	if want := []string{"/properties/title/x-prompt/meaning /$defs/keywords/properties/x-prompt/properties/meaning"}; !slices.Equal(got, want) {
		t.Errorf("NewUpdatePrompt(違反あり).Steps() = %q, want %q", got, want)
	}
	if subs := p.Subschemas(); len(subs) != 1 || subs[0].XPrompt == nil {
		t.Errorf("NewUpdatePrompt(違反あり).Subschemas() = %v, want rule を置いたサブスキーマの x-prompt 1つ", subs)
	}
}

func TestPromptKeepsXPromptInWrittenOrder(t *testing.T) {
	t.Parallel()
	xPrompt := `{"meaning": "題名", "rules": [{"id": "title-short", "rule": "短く書く", "check": "短いか", "bad": [], "good": []}]}`
	c := concrete(t, `{"$id": "urn:example:order-of-keys:v1", "properties": {"title": {"x-prompt": `+xPrompt+`}}}`)
	content := []byte(`{"title": "t"}`)
	value, err := domain.ParseJSON(content)
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	document := domain.Document{Path: path, Content: content, Hash: domain.HashOf(content)}
	prompts := map[string]domain.Prompt{
		"作成": domain.NewCreatePrompt(path, c),
		"更新": domain.NewUpdatePrompt(domain.UpdateInput{Document: document, Value: value, Concrete: c}),
		"判定": domain.NewJudgmentPrompt(document, c, evaluated(t, "/properties/title /title"), nil, "urn:judgment"),
	}
	for name, p := range prompts {
		steps := p.Subschemas()
		if steps == nil {
			steps = p.Steps()
		}
		var got []string
		for _, step := range steps {
			if step.XPrompt != nil {
				got = append(got, string(step.XPrompt))
			}
		}
		if want := []string{xPrompt}; !slices.Equal(got, want) {
			t.Errorf("%s の prompt の x-prompt = %q, want %q", name, got, want)
		}
	}
}

func TestValidationCountsPairsToJudgeAsNotJudged(t *testing.T) {
	t.Parallel()
	content := []byte(`{"title": "t", "start": "a", "end": "b"}`)
	value, err := domain.ParseJSON(content)
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	path, err := domain.NewPath("plan.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	document := domain.Document{Path: path, Content: content, Hash: domain.HashOf(content)}
	c := concrete(t, plan)
	evaluations := evaluated(t, "/properties/title /title", "/properties/end /end")
	got := domain.NewValidation(domain.ValidationInput{Document: document, Value: value, Concrete: c, Evaluations: evaluations}).Counts()
	if want := (domain.Counts{NotJudged: 2}); got != want {
		t.Errorf("NewValidation(前回の検証なし).Counts() = %+v, want %+v", got, want)
	}
	judged, err := domain.NewJudgment("title-short", "/title", "verified", "ai", "短い")
	if err != nil {
		t.Fatalf("NewJudgment = %v", err)
	}
	got = domain.NewValidation(domain.ValidationInput{
		Document: document, Value: value, Concrete: c, Judged: []domain.Judgment{judged}, Evaluations: evaluations,
	}).Counts()
	if want := (domain.Counts{Verified: 1}); got != want {
		t.Errorf("NewValidation(判定を受け取った).Counts() = %+v, want %+v", got, want)
	}
}

const plan = `{"$id": "urn:example:plan:v1", "properties": {
	"title": {"x-prompt": {"rules": [{"id": "title-short", "related_fields": []}]}},
	"start": {"type": "string"},
	"end": {"x-prompt": {"rules": [{"id": "end-after-start", "related_fields": ["/start"]}]}},
	"items": {"items": {"x-prompt": {"rules": [{"id": "item-name"}]}}}}}`

// デシジョンテーブル：変更点（すべて ・ 無し ・ instance location ・ related_fields ・ 配列の要素）× rule。
// 組は、JSON Schema の実装が集める順に頼らず、instance location の順に並ぶ。x-prompt の無い start には rule が当たらない。
func TestNewJudgmentPromptSelectsRulesByChanges(t *testing.T) {
	t.Parallel()
	evaluations := evaluated(t, "/properties/items/items /items/1", "/properties/title /title",
		"/properties/items/items /items/0", "/properties/end /end", "/properties/title /title")
	path, err := domain.NewPath("plan.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	cases := []struct {
		name    string
		changed map[string]bool
		want    []string
	}{
		{name: "すべてが対象なら、すべての組", changed: nil,
			want: []string{"/end end-after-start", "/items/0 item-name", "/items/1 item-name", "/title title-short"}},
		{name: "変更点が無ければ、組は無い", changed: map[string]bool{}},
		{name: "instance location が変更点の rule", changed: map[string]bool{"/title": true, "": true}, want: []string{"/title title-short"}},
		{name: "related_fields が変更点の rule", changed: map[string]bool{"/start": true, "": true}, want: []string{"/end end-after-start"}},
		{name: "配列の要素が変更点の rule", changed: map[string]bool{"/items/1": true, "/items": true, "": true}, want: []string{"/items/1 item-name"}},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			var got []string
			document := domain.Document{Path: path, Content: []byte("{}"), Hash: domain.HashOf([]byte("{}"))}
			for _, step := range domain.NewJudgmentPrompt(document, concrete(t, plan), evaluations, c.changed, "urn:r").Steps() {
				got = append(got, step.InstanceLocation.String()+" "+step.Rule)
			}
			if !slices.Equal(got, c.want) {
				t.Errorf("NewJudgmentPrompt(%s) = %q, want %q", c.name, got, c.want)
			}
		})
	}
}

func TestMatchResponseRejectsPairsOtherThanPrompt(t *testing.T) {
	t.Parallel()
	path, err := domain.NewPath("plan.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	document := domain.Document{Path: path, Content: []byte("{}"), Hash: domain.HashOf([]byte("{}"))}
	evaluations := evaluated(t, "/properties/title /title", "/properties/end /end")
	prompt := domain.NewJudgmentPrompt(document, concrete(t, plan), evaluations, nil, "urn:r")
	end := judgment(t, "end-after-start", "/end", "verified", "ai")
	title := judgment(t, "title-short", "/title", "verified", "ai")
	other := judgment(t, "title-short", "/end", "verified", "ai")
	cases := []struct {
		name      string
		judgments []domain.Judgment
		want      []string
	}{
		{name: "prompt と同じ組", judgments: []domain.Judgment{title, end}},
		{name: "足りない組", judgments: []domain.Judgment{end}, want: []string{"/title"}},
		{name: "余分な組", judgments: []domain.Judgment{end, title, other}, want: []string{"/end"}},
		{name: "重なる組", judgments: []domain.Judgment{end, title, title}, want: []string{"/title"}},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			var got []string
			for _, e := range unjoin(domain.MatchResponse(path, prompt, c.judgments)) {
				var f *domain.Failure
				if errors.As(e, &f) && f.Code == domain.ResponseMismatch {
					got = append(got, f.InstanceLocation)
				}
			}
			if !slices.Equal(got, c.want) {
				t.Errorf("MatchResponse(%s) = %v, want %v", c.name, got, c.want)
			}
		})
	}
}

func TestMatchesComparesValuesAtInstanceLocations(t *testing.T) {
	t.Parallel()
	value, err := domain.ParseJSON([]byte(`{"title": "t", "n": 1, "tags": ["a", "b"], "o": {"k": null}}`))
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	condition := func(location, v string) domain.Condition {
		t.Helper()
		p, err := domain.NewPointer(location)
		if err != nil {
			t.Fatalf("NewPointer(%s) = %v", location, err)
		}
		parsed, err := domain.ParseJSON([]byte(v))
		if err != nil {
			t.Fatalf("ParseJSON(%s) = %v", v, err)
		}
		return domain.Condition{InstanceLocation: p, Value: parsed}
	}
	cases := []struct {
		name       string
		conditions []domain.Condition
		want       bool
	}{
		{name: "文字列が同じ", conditions: []domain.Condition{condition("/title", `"t"`)}, want: true},
		{name: "配列の要素が同じ", conditions: []domain.Condition{condition("/tags/1", `"b"`)}, want: true},
		{name: "配列が同じ", conditions: []domain.Condition{condition("/tags", `["a","b"]`)}, want: true},
		{name: "null が同じ", conditions: []domain.Condition{condition("/o/k", `null`)}, want: true},
		{name: "すべての条件を満たす", conditions: []domain.Condition{condition("/title", `"t"`), condition("/n", `1`)}, want: true},
		{name: "1つでも満たさなければ合わない", conditions: []domain.Condition{condition("/title", `"t"`), condition("/n", `2`)}},
		{name: "数は書いたままの文字列で比べる", conditions: []domain.Condition{condition("/n", `1.0`)}},
		{name: "無い instance location は合わない", conditions: []domain.Condition{condition("/missing", `"t"`)}},
		{name: "配列の範囲の外は合わない", conditions: []domain.Condition{condition("/tags/2", `"c"`)}},
		{name: "先頭に 0 の付いた添字は合わない", conditions: []domain.Condition{condition("/tags/01", `"b"`)}},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			if got := domain.Matches(value, c.conditions); got != c.want {
				t.Errorf("Matches(%s) = %v, want %v", c.name, got, c.want)
			}
		})
	}
}

func TestSelectTemplateUsesXView(t *testing.T) {
	t.Parallel()
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	templates := map[string]domain.Template{"memo": {Path: path, Text: "<p>memo</p>"}}
	cases := []struct {
		name, concrete, want, wantCode string
	}{
		{name: "x-view の名前のテンプレートを選ぶ", concrete: `{"$id": "urn:a", "x-view": "memo"}`, want: "<p>memo</p>"},
		{name: "x-view の名前のテンプレートが無い", concrete: `{"$id": "urn:a", "x-view": "plan"}`, wantCode: "template_missing"},
		{name: "x-view が無い", concrete: `{"$id": "urn:a"}`, wantCode: "template_missing"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			got, err := domain.SelectTemplate(path, concrete(t, c.concrete), templates)
			if got.Text != c.want || codeOf(err) != c.wantCode {
				t.Errorf("SelectTemplate(%s) = %q, %q, want %q, %q", c.name, got.Text, codeOf(err), c.want, c.wantCode)
			}
		})
	}
}

func TestCheckReferencesAgainstConcreteAndTokens(t *testing.T) {
	t.Parallel()
	path, err := domain.NewPath("design/page.html")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	template := domain.Template{Path: path}
	schema := concrete(t, `{"$id": "urn:a", "properties": {"title": {"type": "string"},
		"items": {"type": "array", "items": {"properties": {"name": {"type": "string"}}}}}}`)
	tokens := map[string]any{"heading": "h", "colors": map[string]any{"main": "#000"}}
	name := func(n string) domain.Segment { return domain.Segment{Name: n} }
	element := domain.Segment{Element: true}
	cases := []struct {
		name      string
		reference domain.Reference
		wantCode  string
	}{
		{name: "Concrete の properties にある欄", reference: domain.Reference{Root: "Value", Segments: []domain.Segment{name("title")}}},
		{name: "配列の要素の properties にある欄", reference: domain.Reference{Root: "Value", Segments: []domain.Segment{name("items"), element, name("name")}}},
		{name: "properties を持たない値の組み込みの読み方", reference: domain.Reference{Root: "Value", Segments: []domain.Segment{name("items"), name("length")}}},
		{name: "Concrete に無い欄", reference: domain.Reference{Root: "Value", Segments: []domain.Segment{name("titel")}}, wantCode: "template_reference_missing"},
		{name: "配列の要素に無い欄", reference: domain.Reference{Root: "Value", Segments: []domain.Segment{name("items"), element, name("nam")}}, wantCode: "template_reference_missing"},
		{name: "デザイントークンにある欄", reference: domain.Reference{Root: "Tokens", Segments: []domain.Segment{name("colors"), name("main")}}},
		{name: "デザイントークンに無い欄", reference: domain.Reference{Root: "Tokens", Segments: []domain.Segment{name("color")}}, wantCode: "template_reference_missing"},
		{name: ".Value と .Tokens でない名前", reference: domain.Reference{Root: "Other"}, wantCode: "template_reference_missing"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			err := domain.CheckReferences(template, schema, tokens, []domain.Reference{c.reference})
			if got := codeOf(err); got != c.wantCode {
				t.Errorf("CheckReferences(%s) code = %q, want %q", c.reference, got, c.wantCode)
			}
		})
	}
}

// デシジョンテーブル：別名の先（在る ・ 無い ・ グループ ・ 輪 ・ 型が違う）× 型の決まり方（自分 ・ グループ ・ 別名の先 ・ 無い）。
func TestResolveTokens(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name      string
		tokens    string
		wantCode  string
		wantValue string // 解いた値の木の、別名のトークンの値（css を足したもの。JSON）
	}{
		{name: "別名を解いた値を木に置き、型は別名の先から受け継ぐ",
			tokens:    `{"c": {"$type": "color", "base": {"$value": {"colorSpace": "srgb", "components": [1, 0, 0]}}}, "a": {"$value": "{c.base}"}}`,
			wantValue: `{"colorSpace":"srgb","components":[1,0,0],"css":"color(srgb 1 0 0)"}`},
		{name: "別名の別名をたどる",
			tokens:    `{"n": {"$type": "number", "$value": 2}, "m": {"$value": "{n}"}, "a": {"$value": "{m}"}}`,
			wantValue: `{"css":"2","value":2}`},
		{name: "別名の先が無い", tokens: `{"a": {"$value": "{nothing}"}}`, wantCode: "token_alias_invalid"},
		{name: "別名の先がグループ", tokens: `{"g": {"x": {"$type": "number", "$value": 1}}, "a": {"$value": "{g}"}}`, wantCode: "token_alias_invalid"},
		{name: "別名が輪", tokens: `{"a": {"$value": "{b}"}, "b": {"$value": "{a}"}}`, wantCode: "token_alias_invalid"},
		{name: "別名の先と型が違う", tokens: `{"n": {"$type": "number", "$value": 1}, "a": {"$type": "duration", "$value": "{n}"}}`, wantCode: "token_alias_invalid"},
		{name: "型が決まらない", tokens: `{"a": {"$value": 1}}`, wantCode: "schema_violation"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			value, err := domain.ParseJSON([]byte(c.tokens))
			if err != nil {
				t.Fatalf("ParseJSON(%s) = %v", c.tokens, err)
			}
			resolved, _, err := domain.ResolveTokens(value)
			if got := codeOf(err); got != c.wantCode {
				t.Errorf("ResolveTokens(%s) = %v, want code %q", c.name, err, c.wantCode)
			}
			if c.wantValue == "" {
				return
			}
			tree, _ := resolved.(map[string]any) // 木はいつもオブジェクト
			got, err := json.Marshal(tree["a"])
			if err != nil || string(got) != c.wantValue {
				t.Errorf("ResolveTokens(%s) a = %s, %v, want %s", c.name, got, err, c.wantValue)
			}
		})
	}
}

func TestCSSOfConvertsTypesToCSS(t *testing.T) {
	t.Parallel()
	cases := []struct{ tokenType, value, want string }{
		{"dimension", `{"value": 10, "unit": "px"}`, "10px"},
		{"duration", `{"value": 1.5, "unit": "s"}`, "1.5s"},
		{"color", `{"colorSpace": "srgb", "components": [1, 0, 0], "hex": "#ff0000"}`, "#ff0000"},
		{"color", `{"colorSpace": "srgb", "components": [1, 0, 0], "alpha": 0.5, "hex": "#ff0000"}`, "color(srgb 1 0 0 / 0.5)"},
		{"color", `{"colorSpace": "lab", "components": [60.17, 93.54, -60.5]}`, "lab(60.17 93.54 -60.5)"},
		{"color", `{"colorSpace": "hwb", "components": [330, 0, 0], "alpha": 1}`, "hwb(330 0 0)"},
		{"fontFamily", `["Helvetica Neue", "Arial", "sans-serif"]`, `"Helvetica Neue", Arial, sans-serif`},
		{"fontFamily", `"Comic Sans MS"`, `"Comic Sans MS"`},
		{"fontWeight", `"extra-black"`, "950"},
		{"fontWeight", `350`, "350"},
		{"cubicBezier", `[0, 0, 0.5, 1]`, "cubic-bezier(0, 0, 0.5, 1)"},
		{"number", `2.3`, "2.3"},
	}
	for _, c := range cases {
		value, err := domain.ParseJSON([]byte(c.value))
		if err != nil {
			t.Fatalf("ParseJSON(%s) = %v", c.value, err)
		}
		if got := domain.CSSOf(c.tokenType, value); got != c.want {
			t.Errorf("CSSOf(%s, %s) = %q, want %q", c.tokenType, c.value, got, c.want)
		}
	}
}
