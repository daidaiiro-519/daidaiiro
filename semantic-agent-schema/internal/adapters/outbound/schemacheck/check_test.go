package schemacheck_test

import (
	"context"
	"errors"
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

func lookup(_ context.Context, id string) ([]byte, error) {
	if id == "urn:example:order:v1" {
		return []byte(order), nil
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
