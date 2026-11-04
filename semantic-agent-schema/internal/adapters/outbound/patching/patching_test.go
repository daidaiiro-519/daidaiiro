package patching_test

import (
	"encoding/json"
	"testing"

	"github.com/google/go-cmp/cmp"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/patching"
)

func TestApplyReplacesValue(t *testing.T) {
	t.Parallel()
	got, err := patching.New().Apply([]byte(`{"a": 1, "b": [1]}`), []byte(`[{"op": "replace", "path": "/a", "value": 2}, {"op": "add", "path": "/b/-", "value": 3}]`))
	if err != nil {
		t.Fatalf("Apply = %v", err)
	}
	var value any
	if err := json.Unmarshal(got, &value); err != nil {
		t.Fatalf("json.Unmarshal(%s) = %v", got, err)
	}
	want := map[string]any{"a": 2.0, "b": []any{1.0, 3.0}}
	if diff := cmp.Diff(want, value); diff != "" {
		t.Errorf("Apply mismatch (-want +got):\n%s", diff)
	}
}

func TestApplyRejectsPatchThatCannotApply(t *testing.T) {
	t.Parallel()
	for _, patch := range []string{
		`[{"op": "remove", "path": "/nothing"}]`,
		`[{"op": "test", "path": "/a", "value": 2}]`,
	} {
		if _, err := patching.New().Apply([]byte(`{"a": 1}`), []byte(patch)); err == nil {
			t.Errorf("Apply(%s) = nil error, want error", patch)
		}
	}
}
