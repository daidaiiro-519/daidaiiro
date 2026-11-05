package publishedlanguage_test

import (
	"encoding/json"
	"slices"
	"testing"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
)

func read(t *testing.T, name string, into any) {
	t.Helper()
	content, err := publishedlanguage.Read(name)
	if err != nil {
		t.Fatalf("Read(%s) = %v", name, err)
	}
	if err := json.Unmarshal(content, into); err != nil {
		t.Fatalf("json.Unmarshal(%s) = %v", name, err)
	}
}

func TestErrorCodesMatchEventSchema(t *testing.T) {
	t.Parallel()
	type template struct {
		Instruction string            `json:"instruction"`
		Next        map[string]string `json:"next"`
	}
	var codes struct {
		ConcreteRange string              `json:"concrete_range"`
		Codes         map[string]template `json:"codes"`
		ByMachine     template            `json:"violated_by_machine"`
		ByAI          template            `json:"violated_by_ai"`
		NotJudged     template            `json:"not_judged"`
		NoTargets     struct {
			Instruction string `json:"instruction"`
		} `json:"no_judgment_targets"`
		Prompt struct {
			Instruction map[string]string `json:"instruction"`
			Next        map[string]string `json:"next"`
		} `json:"prompt"`
		Created template `json:"created"`
		Updated template `json:"updated"`
	}
	read(t, "error-codes.json", &codes)
	var event struct {
		Defs struct {
			Rejected struct {
				Properties struct {
					Code struct {
						Enum []string `json:"enum"`
					} `json:"code"`
				} `json:"properties"`
			} `json:"rejected"`
		} `json:"$defs"`
	}
	read(t, "event.schema.json", &event)
	var keys []string
	for code, c := range codes.Codes {
		keys = append(keys, code)
		if c.Instruction == "" || len(c.Next) == 0 {
			t.Errorf("error-codes.json %s = %+v, want instruction and next", code, c)
		}
	}
	for name, v := range map[string]template{"violated_by_machine": codes.ByMachine, "violated_by_ai": codes.ByAI, "not_judged": codes.NotJudged} {
		if v.Instruction == "" || v.Next["*"] == "" {
			t.Errorf("error-codes.json %s = %+v, want instruction and next", name, v)
		}
	}
	if codes.ConcreteRange == "" {
		t.Errorf("error-codes.json concrete_range = empty, want --range form")
	}
	if codes.NoTargets.Instruction == "" {
		t.Errorf("error-codes.json no_judgment_targets = empty, want instruction")
	}
	if codes.Created.Next["create"] == "" || codes.Updated.Next["update"] == "" {
		t.Errorf("error-codes.json created, updated = %+v, %+v, want next", codes.Created, codes.Updated)
	}
	for _, key := range []string{"semantic create", "semantic update", "semantic validate"} {
		if codes.Prompt.Next[key] == "" {
			t.Errorf("error-codes.json prompt.next[%q] = empty, want next", key)
		}
	}
	if codes.Prompt.Instruction["semantic validate"] == "" {
		t.Errorf("error-codes.json prompt.instruction[%q] = empty, want instruction", "semantic validate")
	}
	slices.Sort(keys)
	enum := slices.Sorted(slices.Values(event.Defs.Rejected.Properties.Code.Enum))
	if !slices.Equal(keys, enum) {
		t.Errorf("error-codes.json codes = %v, want event.schema.json rejected code %v", keys, enum)
	}
}

func TestSchemasHaveID(t *testing.T) {
	t.Parallel()
	for _, name := range publishedlanguage.Schemas {
		var schema struct {
			ID string `json:"$id"`
		}
		read(t, name, &schema)
		if schema.ID == "" {
			t.Errorf("Read(%s) $id = %q, want non-empty", name, schema.ID)
		}
	}
}

// Abstract の x-prompt の骨組み（$defs/keywords の properties.x-prompt）は、Concrete に書く x-prompt の値の位置に当たる。
// 骨組みの位置（meaning ・ when ・ rules.items）は、vocabulary の x-prompt の形の中にあり、rule の id は Abstract の中で重ならない。
func TestAbstractRulesSitInXPromptShape(t *testing.T) {
	t.Parallel()
	type xPrompt struct {
		Rules []struct {
			ID string `json:"id"`
		} `json:"rules"`
	}
	type skeleton struct {
		XPrompt    *xPrompt            `json:"x-prompt"`
		Properties map[string]skeleton `json:"properties"`
		Items      *skeleton           `json:"items"`
	}
	var abstract struct {
		XPrompt xPrompt `json:"x-prompt"`
		Defs    struct {
			Keywords struct {
				Properties struct {
					XPrompt skeleton `json:"x-prompt"`
				} `json:"properties"`
			} `json:"keywords"`
		} `json:"$defs"`
	}
	read(t, "abstract.schema.json", &abstract)
	var vocabulary struct {
		Defs struct {
			XPrompt struct {
				Properties map[string]struct {
					Items json.RawMessage `json:"items"`
				} `json:"properties"`
			} `json:"x-prompt"`
		} `json:"$defs"`
	}
	read(t, "vocabulary.schema.json", &vocabulary)
	shape := vocabulary.Defs.XPrompt.Properties
	var ids, places []string
	collect := func(place string, s skeleton) {
		if s.XPrompt == nil {
			t.Errorf("abstract.schema.json の %s に x-prompt が無い", place)
			return
		}
		places = append(places, place)
		for _, r := range s.XPrompt.Rules {
			ids = append(ids, r.ID)
		}
	}
	for _, r := range abstract.XPrompt.Rules {
		ids = append(ids, r.ID)
	}
	for name, s := range abstract.Defs.Keywords.Properties.XPrompt.Properties {
		if _, ok := shape[name]; !ok {
			t.Errorf("abstract.schema.json の x-prompt の骨組み %s が、vocabulary の x-prompt の properties に無い", name)
		}
		if s.Items == nil {
			collect(name, s)
			continue
		}
		if shape[name].Items == nil {
			t.Errorf("abstract.schema.json の x-prompt の骨組み %s.items が、vocabulary の x-prompt の形に無い", name)
		}
		collect(name+".items", *s.Items)
	}
	if want := []string{"meaning", "rules.items", "when"}; !slices.Equal(slices.Sorted(slices.Values(places)), want) {
		t.Errorf("abstract.schema.json の x-prompt の骨組み = %v, want %v", slices.Sorted(slices.Values(places)), want)
	}
	if len(ids) == 0 || len(slices.Compact(slices.Sorted(slices.Values(ids)))) != len(ids) {
		t.Errorf("abstract.schema.json の rule の id = %v, want 重ならない1件以上", ids)
	}
}
