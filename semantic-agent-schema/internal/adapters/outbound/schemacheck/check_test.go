package schemacheck_test

import (
	"context"
	"errors"
	"slices"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/schemacheck"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
)

const order = `{
  "$schema": "urn:semantic-agent-schema:abstract:v1",
  "$id": "urn:example:order:v1",
  "x-view": "order",
  "type": "object",
  "required": ["$schema", "items"],
  "properties": {
    "$schema": { "const": "urn:example:order:v1" },
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "price": {
            "type": "number", "minimum": 0,
            "x-prompt": {
              "meaning": "1つの品の値段", "when": "注文を書くとき",
              "rules": [{"id": "price-tax", "rule": "税込みで書く", "why": "請求と同じにする", "check": "税込みか",
                         "related_fields": ["/items"], "bad": [], "good": [{"value": 110}]}]
            }
          }
        }
      }
    }
  }
}`

// plan は、title の x-prompt に rule が2件、tasks の要素の name の x-prompt に rule が1件、start に x-prompt が無い Concrete。
const plan = `{
  "$schema": "urn:semantic-agent-schema:abstract:v1",
  "$id": "urn:example:plan:v1",
  "type": "object",
  "required": ["$schema"],
  "properties": {
    "$schema": { "const": "urn:example:plan:v1" },
    "title": {
      "type": "string",
      "x-prompt": {
        "meaning": "計画の題名", "when": "計画を書くとき",
        "rules": [
          {"id": "title-subject", "rule": "主語を含める", "why": "一覧で分かる", "check": "主語を含むか", "related_fields": [], "bad": [], "good": []},
          {"id": "title-no-date", "rule": "日付を含めない", "why": "日付は start が持つ", "check": "日付を含まないか", "related_fields": [], "bad": [], "good": []}
        ]
      }
    },
    "start": { "type": "string", "format": "date" },
    "tasks": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string",
            "x-prompt": {
              "meaning": "作業の名前", "when": "計画を書くとき",
              "rules": [{"id": "name-short", "rule": "短く書く", "why": "一覧で読める", "check": "短いか", "related_fields": [], "bad": [], "good": []}]
            }
          }
        }
      }
    }
  }
}`

func lookup(_ context.Context, id string) ([]byte, error) {
	switch id {
	case "urn:example:order:v1":
		return []byte(order), nil
	case "urn:example:plan:v1":
		return []byte(plan), nil
	}
	return nil, domain.Fail(domain.SchemaNotFound, "", id)
}

// violation は、比べる違反の位置。message は比べない。
type violation struct {
	InstanceLocation, AbsoluteKeywordLocation string
}

func check(t *testing.T, schemaID, content string) ([]violation, error) {
	t.Helper()
	violations, err := schemacheck.New(lookup).Check(t.Context(), schemaID, []byte(content))
	var got []violation
	for _, v := range violations {
		got = append(got, violation{v.InstanceLocation.String(), v.AbsoluteKeywordLocation})
	}
	return got, err
}

func TestCheckReturnsNoViolation(t *testing.T) {
	t.Parallel()
	cases := []struct{ name, schemaID, content string }{
		{name: "インスタンスが Concrete を満たす", schemaID: "urn:example:order:v1",
			content: `{"$schema": "urn:example:order:v1", "items": [{"price": 1}]}`},
		{name: "Concrete が Abstract を満たす", schemaID: publishedlanguage.AbstractID, content: order},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			got, err := check(t, c.schemaID, c.content)
			if err != nil || len(got) != 0 {
				t.Errorf("Check(%s) = %v, %v, want none, nil", c.name, got, err)
			}
		})
	}
}

func TestCheckReturnsViolations(t *testing.T) {
	t.Parallel()
	concrete := func(property string) string {
		return `{"$schema": "urn:semantic-agent-schema:abstract:v1", "$id": "urn:example:a:v1", "required": ["$schema"],
		         "properties": {"$schema": {"const": "urn:example:a:v1"}, "name": ` + property + `}}`
	}
	cases := []struct {
		name, schemaID, content string
		want                    []violation
	}{
		{name: "インスタンスが Concrete に違反する", schemaID: "urn:example:order:v1",
			content: `{"$schema": "urn:example:order:v1", "items": [{"price": -1}]}`,
			want:    []violation{{"/items/0/price", "urn:example:order:v1#/properties/items/items/properties/price/minimum"}}},
		{name: "拡張項目の閉じた一覧に無い x- のキーワード", schemaID: publishedlanguage.AbstractID,
			content: concrete(`{"type": "string", "x-note": "a"}`),
			want:    []violation{{"/properties/name", "urn:semantic-agent-schema:vocabulary:v1#/propertyNames/then/enum"}}},
		{name: "x-view をルートでないサブスキーマに書く", schemaID: publishedlanguage.AbstractID,
			content: concrete(`{"type": "string", "x-view": "a"}`),
			want:    []violation{{"/properties/name/x-view", "urn:semantic-agent-schema:dialect:v1#/properties/x-view"}}},
		{name: "x-decided-by が human でない", schemaID: publishedlanguage.AbstractID,
			content: concrete(`{"x-decided-by": "ai"}`),
			want:    []violation{{"/properties/name/x-decided-by", "urn:semantic-agent-schema:vocabulary:v1#/properties/x-decided-by/const"}}},
		{name: "related_fields が絶対の JSON Pointer でない", schemaID: publishedlanguage.AbstractID,
			content: concrete(`{"x-prompt": {"meaning": "m", "when": "w", "rules": [{"id": "r", "rule": "r", "why": "w", "check": "c",
			          "related_fields": ["start"], "bad": [], "good": []}]}}`),
			want: []violation{{"/properties/name/x-prompt/rules/0/related_fields/0", "urn:semantic-agent-schema:vocabulary:v1#/$defs/pointer/pattern"}}},
		{name: "インスタンスに $schema を求めない Concrete", schemaID: publishedlanguage.AbstractID,
			content: `{"$schema": "urn:semantic-agent-schema:abstract:v1", "$id": "urn:example:a:v1", "required": [],
			          "properties": {"$schema": {"const": "urn:example:a:v1"}}}`,
			want: []violation{{"/required", "urn:semantic-agent-schema:abstract:v1#/properties/required/contains"}}},
		{name: "引数の path が working tree の外を指す", schemaID: publishedlanguage.ArgumentsID,
			content: `{"command": "validate", "paths": ["../a.json"], "schemas": ["s"]}`,
			want:    []violation{{"/paths/0", "urn:semantic-agent-schema:arguments:v1#/$defs/path/pattern"}}},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			got, err := check(t, c.schemaID, c.content)
			if err != nil {
				t.Errorf("Check(%s) error = %v, want nil", c.name, err)
			}
			if diff := cmp.Diff(c.want, got); diff != "" {
				t.Errorf("Check(%s) mismatch (-want +got):\n%s", c.name, diff)
			}
		})
	}
}

func TestCheckReturnsFailures(t *testing.T) {
	t.Parallel()
	cases := []struct{ name, schemaID, content, wantCode string }{
		{name: "Concrete が schemas に無い", schemaID: "urn:example:none:v1", content: `{}`, wantCode: "schema_not_found"},
		{name: "JSON として読めない", schemaID: "urn:example:order:v1", content: `{`, wantCode: "invalid_json"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			_, err := check(t, c.schemaID, c.content)
			var f *domain.Failure
			if !errors.As(err, &f) || string(f.Code) != c.wantCode {
				t.Errorf("Check(%s) = %v, want code %s", c.name, err, c.wantCode)
			}
		})
	}
}

func TestCheckWithUsesUnwrittenConcrete(t *testing.T) {
	t.Parallel()
	updated := strings.Replace(order, `"required": ["$schema", "items"]`, `"required": ["$schema", "items", "owner"]`, 1)
	got, err := schemacheck.New(lookup).CheckWith(t.Context(), []byte(updated), []byte(`{"$schema": "urn:example:order:v1", "items": []}`))
	want := []string{"urn:example:order:v1#/required"}
	var locations []string
	for _, v := range got {
		locations = append(locations, v.AbsoluteKeywordLocation)
	}
	if err != nil || !slices.Equal(locations, want) {
		t.Errorf("CheckWith(updated order) = %v, %v, want %v, nil", locations, err, want)
	}
}

// x-prompt を持つサブスキーマが当たる位置は、JSON Schema の検証が決める。インスタンスでは Concrete の、Concrete では Abstract の
// サブスキーマの位置との組になり、x-prompt の無い start には組が無い。
func TestEvaluateReturnsWhereXPromptSubschemasApply(t *testing.T) {
	t.Parallel()
	const xPrompt = "/$defs/keywords/properties/x-prompt/properties"
	cases := []struct {
		name, schemaID, content string
		want                    []string
	}{
		{name: "インスタンスでは Concrete のサブスキーマ", schemaID: "urn:example:plan:v1",
			content: `{"$schema": "urn:example:plan:v1", "title": "t", "start": "2026-01-01", "tasks": [{"name": "a"}, {"name": "b"}]}`,
			want: []string{
				"/tasks/0/name ← /properties/tasks/items/properties/name",
				"/tasks/1/name ← /properties/tasks/items/properties/name",
				"/title ← /properties/title",
			}},
		{name: "Concrete では Abstract のサブスキーマ", schemaID: publishedlanguage.AbstractID, content: plan,
			want: []string{
				" ← ",
				"/properties/tasks/items/properties/name/x-prompt/meaning ← " + xPrompt + "/meaning",
				"/properties/tasks/items/properties/name/x-prompt/rules/0 ← " + xPrompt + "/rules/items",
				"/properties/tasks/items/properties/name/x-prompt/when ← " + xPrompt + "/when",
				"/properties/title/x-prompt/meaning ← " + xPrompt + "/meaning",
				"/properties/title/x-prompt/rules/0 ← " + xPrompt + "/rules/items",
				"/properties/title/x-prompt/rules/1 ← " + xPrompt + "/rules/items",
				"/properties/title/x-prompt/when ← " + xPrompt + "/when",
			}},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			violations, evaluations, err := schemacheck.New(lookup).Evaluate(t.Context(), c.schemaID, []byte(c.content))
			if err != nil || len(violations) != 0 {
				t.Errorf("Evaluate(%s) = %v, %v, want no violation, nil", c.name, violations, err)
			}
			var got []string
			for _, e := range evaluations {
				got = append(got, e.InstanceLocation.String()+" ← "+e.KeywordLocation.String())
			}
			slices.Sort(got)
			if diff := cmp.Diff(c.want, got); diff != "" {
				t.Errorf("Evaluate(%s) mismatch (-want +got):\n%s", c.name, diff)
			}
		})
	}
}
