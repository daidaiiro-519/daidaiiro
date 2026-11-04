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
		Codes    map[string]template `json:"codes"`
		Violated template            `json:"violated"`
		Prompt   template            `json:"prompt"`
		Created  template            `json:"created"`
		Updated  template            `json:"updated"`
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
	if codes.Violated.Instruction == "" || codes.Violated.Next["validate"] == "" {
		t.Errorf("error-codes.json violated = %+v, want instruction and next for validate", codes.Violated)
	}
	if codes.Created.Next["create"] == "" || codes.Updated.Next["update"] == "" {
		t.Errorf("error-codes.json created, updated = %+v, %+v, want next", codes.Created, codes.Updated)
	}
	if codes.Prompt.Next["create --semantic"] == "" || codes.Prompt.Next["update --semantic"] == "" {
		t.Errorf("error-codes.json prompt = %+v, want next for create --semantic and update --semantic", codes.Prompt)
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
