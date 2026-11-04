package domain_test

import (
	"errors"
	"slices"
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
			want: []string{"/allOf/0/x-prompt/rules/0/good/0/related"}},
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
	j, err := domain.RecordedJudgment(rule, location, verdict, by, "")
	if err != nil {
		t.Fatalf("RecordedJudgment(%s, %s, %s, %s) = %v", rule, location, verdict, by, err)
	}
	return j
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

func TestRecordedJudgmentRejectsValuesOutsideClosedLists(t *testing.T) {
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
			if _, err := domain.RecordedJudgment(c.rule, c.location, c.verdict, c.by, ""); err == nil {
				t.Errorf("RecordedJudgment(%q, %q, %q, %q) = nil error, want error", c.rule, c.location, c.verdict, c.by)
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
	p := domain.NewUpdatePrompt(document, value, concrete(t, memo), "urn:patch")
	var got []string
	for _, step := range p.Steps() {
		got = append(got, step.InstanceLocation.String()+" "+step.Title)
	}
	want := []string{" メモ", "/items/0 品", "/items/1 品", "/title 題名"}
	if !slices.Equal(got, want) {
		t.Errorf("NewUpdatePrompt(memo).Steps() = %q, want %q", got, want)
	}
	if p.ResponseSchemaID() != "urn:patch" {
		t.Errorf("NewUpdatePrompt(memo).ResponseSchemaID() = %s, want urn:patch", p.ResponseSchemaID())
	}
}
