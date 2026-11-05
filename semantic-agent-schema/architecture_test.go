package semanticagentschema_test

import (
	"encoding/json"
	"errors"
	"io"
	"os/exec"
	"slices"
	"strings"
	"testing"

	"github.com/nishanths/exhaustive/passes/exhaustive"
	"golang.org/x/tools/go/analysis"
	"golang.org/x/tools/go/analysis/checker"
	"golang.org/x/tools/go/packages"
)

const module = "github.com/daidaiiro-519/daidaiiro/semantic-agent-schema"

// rule は、具体モデリング概念の「依存の決まり」の1行。
type rule struct {
	internal []string // 使ってよい internal の package（module からの相対）。"*" は全部
	external []string // 使ってよい外のライブラリ
}

var applications = []string{
	"internal/authoring/application", "internal/rendering/application",
	"internal/search/application", "internal/record/application",
}

func rules() map[string]rule {
	r := map[string]rule{
		"":                                      {}, // この試験だけを持つ根の package
		"internal/authoring/domain":             {},
		"internal/publishedlanguage":            {},
		"internal/adapters/outbound/files":      {internal: []string{"internal/authoring/domain"}},
		"internal/adapters/outbound/repository": {internal: []string{"internal/authoring/domain"}},
		"internal/adapters/outbound/templates": {
			internal: []string{"internal/authoring/domain"},
			external: []string{"github.com/tdewolff/parse/v2", "github.com/tdewolff/parse/v2/js"},
		},
		"internal/adapters/outbound/patching": {external: []string{"github.com/evanphx/json-patch/v5"}},
		"internal/adapters/outbound/schemacheck": {
			internal: []string{"internal/authoring/domain", "internal/publishedlanguage"},
			external: []string{"github.com/santhosh-tekuri/jsonschema/v6"},
		},
		"internal/adapters/outbound/recordstore": {
			internal: []string{"internal/authoring/domain", "internal/authoring/application", "internal/record/application"},
			external: []string{"modernc.org/sqlite"},
		},
		"internal/adapters/inbound/cli": {
			internal: append(slices.Clone(applications), "internal/authoring/domain", "internal/publishedlanguage"),
			external: []string{"github.com/spf13/pflag"},
		},
		"cmd/semantic-agent-schema": {internal: []string{"*"}},
	}
	for _, a := range applications {
		r[a] = rule{internal: []string{"internal/authoring/domain"}}
	}
	return r
}

type listed struct {
	ImportPath string
	Imports    []string
	Standard   bool
	Module     *struct{ Path string }
}

// TestDependencies は、go list -deps -json ./... の ImportPath ・ Imports ・ Standard を読み、表に無い依存があれば落とす。
func TestDependencies(t *testing.T) {
	t.Parallel()
	out, err := exec.Command("go", "list", "-deps", "-json", "./...").Output()
	if err != nil {
		t.Fatalf("go list -deps -json ./... = %v", err)
	}
	standard := map[string]bool{}
	var ours []listed
	decoder := json.NewDecoder(strings.NewReader(string(out)))
	for {
		var p listed
		if err := decoder.Decode(&p); errors.Is(err, io.EOF) {
			break
		} else if err != nil {
			t.Fatalf("Decode(go list) = %v", err)
		}
		standard[p.ImportPath] = p.Standard
		if p.ImportPath == module || strings.HasPrefix(p.ImportPath, module+"/") {
			ours = append(ours, p)
		}
	}
	table := rules()
	for _, p := range ours {
		name := strings.TrimPrefix(strings.TrimPrefix(p.ImportPath, module), "/")
		r, ok := table[name]
		if !ok {
			t.Errorf("rules()[%q] = none, want a row of the dependency table", name)
			continue
		}
		for _, imported := range p.Imports {
			switch {
			case standard[imported]:
			case strings.HasPrefix(imported, module+"/"):
				target := strings.TrimPrefix(imported, module+"/")
				if !slices.Contains(r.internal, "*") && !slices.Contains(r.internal, target) {
					t.Errorf("Imports(%s) = %s, want one of %v", name, target, r.internal)
				}
			default:
				if !slices.Contains(r.external, imported) {
					t.Errorf("Imports(%s) = %s, want one of %v", name, imported, r.external)
				}
			}
		}
	}
}

// TestExhaustive は、閉じた一覧（区分）の switch がすべての値を持つかを確かめる。-d（default で網羅とみなす）は付けない。
func TestExhaustive(t *testing.T) {
	t.Parallel()
	pkgs, err := packages.Load(&packages.Config{Mode: packages.LoadAllSyntax}, "./...")
	if err != nil {
		t.Fatalf("packages.Load(./...) = %v", err)
	}
	if packages.PrintErrors(pkgs) > 0 {
		t.Fatal("packages.Load(./...) has errors")
	}
	graph, err := checker.Analyze([]*analysis.Analyzer{exhaustive.Analyzer}, pkgs, nil)
	if err != nil {
		t.Fatalf("checker.Analyze(exhaustive) = %v", err)
	}
	for action := range graph.All() {
		if !action.IsRoot {
			continue
		}
		if action.Err != nil {
			t.Errorf("Analyze(%s) = %v, want nil", action, action.Err)
		}
		for _, d := range action.Diagnostics {
			t.Errorf("exhaustive %s = %s, want no diagnostic", action.Package.Fset.Position(d.Pos), d.Message)
		}
	}
}
