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
  "additionalProperties": false,
  "x-prompt": {"meaning": "注文", "when": "注文を書くとき", "rules": []},
  "properties": {
    "$schema": { "const": "urn:example:order:v1" },
    "items": {
      "type": "array",
      "x-prompt": {"meaning": "注文の品の一覧", "when": "注文を書くとき", "rules": []},
      "items": {
        "type": "object",
        "additionalProperties": false,
        "x-prompt": {"meaning": "注文の1つの品", "when": "注文を書くとき", "rules": []},
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

// plan は、title の x-prompt に rule が2件、tasks の要素の name の x-prompt に rule が1件の Concrete。
const plan = `{
  "$schema": "urn:semantic-agent-schema:abstract:v1",
  "$id": "urn:example:plan:v1",
  "type": "object",
  "required": ["$schema"],
  "additionalProperties": false,
  "x-prompt": {"meaning": "計画", "when": "計画を書くとき", "rules": []},
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
    "start": {
      "type": "string", "format": "date",
      "x-prompt": {"meaning": "計画の始まりの日", "when": "計画を書くとき", "rules": []}
    },
    "tasks": {
      "type": "array",
      "x-prompt": {"meaning": "作業の一覧", "when": "計画を書くとき", "rules": []},
      "items": {
        "type": "object",
        "additionalProperties": false,
        "x-prompt": {"meaning": "1つの作業", "when": "計画を書くとき", "rules": []},
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
	const prompt = `"x-prompt": {"meaning": "m", "when": "w", "rules": []}`
	concrete := func(property string) string {
		return `{"$schema": "urn:semantic-agent-schema:abstract:v1", "$id": "urn:example:a:v1", "required": ["$schema"],
		         "type": "object", "additionalProperties": false, ` + prompt + `,
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
			content: concrete(`{"type": "string", "x-note": "a", ` + prompt + `}`),
			want:    []violation{{"/properties/name", "urn:semantic-agent-schema:vocabulary:v1#/propertyNames/then/enum"}}},
		{name: "x-view をルートでないサブスキーマに書く", schemaID: publishedlanguage.AbstractID,
			content: concrete(`{"type": "string", "x-view": "a", ` + prompt + `}`),
			want:    []violation{{"/properties/name/x-view", "urn:semantic-agent-schema:dialect:v1#/properties/x-view"}}},
		{name: "related_fields が絶対の JSON Pointer でない", schemaID: publishedlanguage.AbstractID,
			content: concrete(`{"x-prompt": {"meaning": "m", "when": "w", "rules": [{"id": "r", "rule": "r", "why": "w", "check": "c",
			          "related_fields": ["start"], "bad": [], "good": []}]}}`),
			want: []violation{{"/properties/name/x-prompt/rules/0/related_fields/0", "urn:semantic-agent-schema:vocabulary:v1#/$defs/pointer/pattern"}}},
		{name: "インスタンスに $schema を求めない Concrete", schemaID: publishedlanguage.AbstractID,
			content: `{"$schema": "urn:semantic-agent-schema:abstract:v1", "$id": "urn:example:a:v1", "required": [],
			          "type": "object", "additionalProperties": false, ` + prompt + `,
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

// サブスキーマが当たる位置は、JSON Schema の検証が決める。インスタンスでは Concrete の、Concrete では Abstract の
// サブスキーマの位置との組になる。Concrete では、$ref ・ allOf で当たるサブスキーマの組も返るので、x-prompt の組が含まれることを確かめる。
func TestEvaluateReturnsWhereXPromptSubschemasApply(t *testing.T) {
	t.Parallel()
	const xPrompt = "/$defs/keywords/properties/x-prompt/properties"
	cases := []struct {
		name, schemaID, content string
		want                    []string
		subset                  bool // want が、返った組の一部であればよい
	}{
		{name: "インスタンスでは Concrete のサブスキーマ", schemaID: "urn:example:plan:v1",
			content: `{"$schema": "urn:example:plan:v1", "title": "t", "start": "2026-01-01", "tasks": [{"name": "a"}, {"name": "b"}]}`,
			want: []string{
				" ← ",
				"/$schema ← /properties/$schema",
				"/start ← /properties/start",
				"/tasks ← /properties/tasks",
				"/tasks/0 ← /properties/tasks/items",
				"/tasks/0/name ← /properties/tasks/items/properties/name",
				"/tasks/1 ← /properties/tasks/items",
				"/tasks/1/name ← /properties/tasks/items/properties/name",
				"/title ← /properties/title",
			}},
		{name: "Concrete では Abstract のサブスキーマ", schemaID: publishedlanguage.AbstractID, content: plan, subset: true,
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
			if c.subset {
				got = slices.DeleteFunc(got, func(pair string) bool { return !slices.Contains(c.want, pair) })
			}
			if diff := cmp.Diff(c.want, got); diff != "" {
				t.Errorf("Evaluate(%s) mismatch (-want +got):\n%s", c.name, diff)
			}
		})
	}
}

// xp は、x-prompt を書く JSON の断片。
const xp = `"x-prompt": {"meaning": "m", "when": "w", "rules": []}`

// base は、ルートに x-prompt と unevaluatedProperties を持つ Concrete。tail は、ルートへ足すキーワード（先頭にカンマ）。
func base(properties, tail string) string {
	return `{"$schema": "urn:semantic-agent-schema:abstract:v1", "$id": "urn:example:a:v1", "required": ["$schema"],
	         "type": "object", "unevaluatedProperties": false, ` + xp + `,
	         "properties": {"$schema": {"const": "urn:example:a:v1"}` + properties + `}` + tail + `}`
}

func locations(t *testing.T, content string) []string {
	t.Helper()
	got, err := check(t, publishedlanguage.AbstractID, content)
	if err != nil {
		t.Fatalf("Check(%s) error = %v, want nil", content, err)
	}
	var at []string
	for _, v := range got {
		at = append(at, v.InstanceLocation)
	}
	slices.Sort(at)
	return slices.Compact(at)
}

func TestAbstractRequiresXPromptWhereValuesAreWritten(t *testing.T) {
	t.Parallel()
	cases := []struct{ name, concrete, want string }{
		{"ルート", strings.Replace(base("", ""), xp+",", "", 1), ""},
		{"properties の値", base(`, "name": {"type": "string"}`, ""), "/properties/name"},
		{"patternProperties の値", base("", `, "patternProperties": {"^a": {"type": "string"}}`), "/patternProperties/^a"},
		{"additionalProperties のサブスキーマ", strings.Replace(base("", `, "additionalProperties": {"type": "string"}`), `"unevaluatedProperties": false,`, "", 1), "/additionalProperties"},
		{"unevaluatedProperties のサブスキーマ", strings.Replace(base("", ""), `"unevaluatedProperties": false`, `"unevaluatedProperties": {"type": "string"}`, 1), "/unevaluatedProperties"},
		{"items のサブスキーマ", base(`, "list": {"type": "array", "items": {"type": "string"}, `+xp+`}`, ""), "/properties/list/items"},
		{"prefixItems の要素", base(`, "pair": {"type": "array", "prefixItems": [{"type": "string"}], "items": false, `+xp+`}`, ""), "/properties/pair/prefixItems/0"},
		{"unevaluatedItems のサブスキーマ", base(`, "list": {"type": "array", "unevaluatedItems": {"type": "string"}, `+xp+`}`, ""), "/properties/list/unevaluatedItems"},
		{"oneOf の枝", base(`, "due": {"oneOf": [{"type": "string"}], `+xp+`}`, ""), "/properties/due/oneOf/0"},
		{"anyOf の枝", base(`, "due": {"anyOf": [{"type": "string"}], `+xp+`}`, ""), "/properties/due/anyOf/0"},
		{"then", base("", `, "if": {"required": ["a"]}, "then": {"required": ["b"]}`), "/then"},
		{"else", base("", `, "if": {"required": ["a"]}, "else": {"required": ["b"]}`), "/else"},
		{"dependentSchemas の枝", base("", `, "dependentSchemas": {"a": {"required": ["b"]}}`), "/dependentSchemas/a"},
		{"const の枝", base(`, "due": {"oneOf": [{"const": 1}], `+xp+`}`, ""), "/properties/due/oneOf/0"},
		{"値が true", base(`, "name": true`, ""), "/properties/name"},
		{"type が object で additionalProperties も unevaluatedProperties も無い", base(`, "box": {"type": "object", `+xp+`}`, ""), "/properties/box"},
		{"type が array で items も prefixItems も unevaluatedItems も無い", base(`, "list": {"type": "array", `+xp+`}`, ""), "/properties/list"},
		{"$defs", base("", `, "$defs": {"a": {"type": "string", `+xp+`}}`), "/$defs"},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			if got := locations(t, c.concrete); !slices.Equal(got, []string{c.want}) {
				t.Errorf("Check(%s) violation locations = %q, want [%q]", c.name, got, c.want)
			}
		})
	}
}

func TestAbstractAllowsNoXPromptWhereValuesAreNotWritten(t *testing.T) {
	t.Parallel()
	cases := []struct{ name, concrete string }{
		{"const の値", base(`, "kind": {"const": "a"}`, "")},
		{"要素1つの enum の値", base(`, "kind": {"enum": ["a"]}`, "")},
		{"if", base("", `, "if": {"required": ["a"]}, "then": {"required": ["b"], `+xp+`}`)},
		{"not", base("", `, "not": {"required": ["a"]}`)},
		{"contains", base(`, "list": {"type": "array", "contains": {"type": "string"}, "items": false, `+xp+`}`, "")},
		{"propertyNames", base("", `, "propertyNames": {"pattern": "^a"}`)},
		{"allOf の枝", base("", `, "allOf": [{"required": ["a"]}]`)},
		{"then の中で properties を定義し unevaluatedProperties を false にする",
			base(`, "style": {"enum": ["onsite", "online"], `+xp+`}`,
				`, "if": {"properties": {"style": {"const": "onsite"}}}, "then": {"required": ["place"], `+xp+`,
				  "properties": {"place": {"type": "string", `+xp+`}}}`)},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			if got := locations(t, c.concrete); len(got) != 0 {
				t.Errorf("Check(%s) violation locations = %q, want none", c.name, got)
			}
		})
	}
}

// event は、催しの Concrete。style で then と else に分かれ、due は日付か開催の何日前かの oneOf。
const event = `{
  "$schema": "urn:semantic-agent-schema:abstract:v1",
  "$id": "urn:example:event:v1",
  "type": "object",
  "required": ["$schema", "title", "style"],
  "additionalProperties": false,
  "x-prompt": {"meaning": "社内の催し1件の計画", "when": "催しを企画するとき", "rules": []},
  "properties": {
    "$schema": {"const": "urn:example:event:v1"},
    "title": {"type": "string", "x-prompt": {"meaning": "一覧で見分ける催しの名前", "when": "催しを企画するとき", "rules": []}},
    "style": {"enum": ["onsite", "online"], "x-prompt": {"meaning": "集まり方", "when": "会場を決めるとき", "rules": []}},
    "place": {"type": "string", "x-prompt": {"meaning": "会場の名前", "when": "会場を決めるとき", "rules": []}},
    "due": {
      "x-prompt": {"meaning": "申し込みの締め切り", "when": "申し込みを受け付けるとき", "rules": []},
      "oneOf": [
        {"type": "string", "format": "date", "x-prompt": {"meaning": "締め切りの日付", "when": "開催日が決まっているとき", "rules": []}},
        {"type": "integer", "minimum": 1, "x-prompt": {"meaning": "開催の何日前に締め切るか", "when": "開催日がまだ決まっていないとき", "rules": []}}
      ]
    }
  },
  "if": {"properties": {"style": {"const": "onsite"}}},
  "then": {"required": ["place"], "x-prompt": {"meaning": "会場に集まる催し", "when": "style が onsite のとき", "rules": []}},
  "else": {"x-prompt": {"meaning": "オンラインの催し", "when": "style が online のとき", "rules": []}}
}`

// anyOfCode は、code が anyOf の3枝（maxLength ・ pattern ・ minLength）を持つ Concrete。
const anyOfCode = `{
  "$schema": "urn:semantic-agent-schema:abstract:v1", "$id": "urn:example:code:v1", "type": "object", "required": ["$schema"],
  "additionalProperties": false, ` + xp + `,
  "properties": {
    "$schema": {"const": "urn:example:code:v1"},
    "code": {` + xp + `, "anyOf": [
      {"maxLength": 3, ` + xp + `}, {"pattern": "^[0-9-]+$", ` + xp + `}, {"minLength": 1, ` + xp + `}]}
  }
}`

// oneOfCode は、code が oneOf の2枝を持つ。2枝は type が同じで、pattern だけが違う。
const oneOfCode = `{
  "$schema": "urn:semantic-agent-schema:abstract:v1", "$id": "urn:example:code:v1", "type": "object", "required": ["$schema"],
  "additionalProperties": false, ` + xp + `,
  "properties": {
    "$schema": {"const": "urn:example:code:v1"},
    "code": {` + xp + `, "oneOf": [
      {"type": "string", "pattern": "^[a-z]+$", ` + xp + `}, {"type": "string", "pattern": "^[0-9]+$", ` + xp + `}]}
  }
}`

// 組は、JSON Schema の注釈と同じに集める。検証に落ちたサブスキーマの組は返さず、通った枝の組は、anyOf が打ち切った枝も含めてすべて返す。
func TestEvaluateCollectsAnnotationsLikeJSONSchema(t *testing.T) {
	t.Parallel()
	cases := []struct {
		name, schemaID, concrete, content string
		want                              []string
	}{
		{name: "onsite の催しは then と日付の枝の組を返し、else と整数の枝は返さない", schemaID: "urn:example:event:v1", concrete: event,
			content: `{"$schema": "urn:example:event:v1", "title": "新人歓迎会", "style": "onsite", "place": "本社 3F", "due": "2026-04-01"}`,
			want: []string{
				" ← ", " ← /if", " ← /then",
				"/$schema ← /properties/$schema",
				"/due ← /properties/due", "/due ← /properties/due/oneOf/0",
				"/place ← /properties/place",
				"/style ← /if/properties/style", "/style ← /properties/style",
				"/title ← /properties/title",
			}},
		{name: "online の催しは else と整数の枝の組を返し、then と日付の枝は返さない", schemaID: "urn:example:event:v1", concrete: event,
			content: `{"$schema": "urn:example:event:v1", "title": "読書会", "style": "online", "due": 7}`,
			want: []string{
				" ← ", " ← /else",
				"/$schema ← /properties/$schema",
				"/due ← /properties/due", "/due ← /properties/due/oneOf/1",
				"/style ← /properties/style",
				"/title ← /properties/title",
			}},
		{name: "anyOf は、通った枝すべての組を返し、通らない枝は返さない", schemaID: "urn:example:code:v1", concrete: anyOfCode,
			content: `{"$schema": "urn:example:code:v1", "code": "2026-04-01"}`,
			want: []string{
				" ← ",
				"/$schema ← /properties/$schema",
				"/code ← /properties/code", "/code ← /properties/code/anyOf/1", "/code ← /properties/code/anyOf/2",
			}},
		{name: "oneOf の枝が同じ type で pattern だけが違うとき、合わない枝の組は返さない", schemaID: "urn:example:code:v1", concrete: oneOfCode,
			content: `{"$schema": "urn:example:code:v1", "code": "abc"}`,
			want: []string{
				" ← ",
				"/$schema ← /properties/$schema",
				"/code ← /properties/code", "/code ← /properties/code/oneOf/0",
			}},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			t.Parallel()
			lookup := func(_ context.Context, id string) ([]byte, error) {
				if id == c.schemaID {
					return []byte(c.concrete), nil
				}
				return nil, domain.Fail(domain.SchemaNotFound, "", id)
			}
			violations, evaluations, err := schemacheck.New(lookup).Evaluate(t.Context(), c.schemaID, []byte(c.content))
			if err != nil || len(violations) != 0 {
				t.Fatalf("Evaluate(%s) = %v, %v, want no violation, nil", c.name, violations, err)
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
