package templates_test

import (
	"errors"
	"os"
	"path/filepath"
	"slices"
	"testing"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/templates"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

func page(t *testing.T, out, template string) domain.Page {
	t.Helper()
	p, err := domain.NewPath(out)
	if err != nil {
		t.Fatalf("NewPath(%s) = %v", out, err)
	}
	source, err := domain.NewPath("design/page.html")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	return domain.Page{Out: p, Template: domain.Template{Path: source, Text: template}, Value: map[string]any{"title": "<t>"}, Tokens: map[string]any{"c": "x"}}
}

func TestRenderWritesAllPages(t *testing.T) {
	t.Parallel()
	top := t.TempDir()
	err := templates.New(top).Render(t.Context(), []domain.Page{
		page(t, "site/a.html", `<p class="{{.Tokens.c}}">{{.Value.title}}</p>`),
		page(t, "site/b.html", `<p>{{.Value.title}}</p>`),
	})
	if err != nil {
		t.Fatalf("Render = %v", err)
	}
	content, err := os.ReadFile(filepath.Join(top, "site", "a.html"))
	if want := `<p class="x">&lt;t&gt;</p>`; err != nil || string(content) != want {
		t.Errorf("ReadFile(site/a.html) = %q, %v, want %q", content, err, want)
	}
}

func TestRenderWritesNothingWhenAPageFails(t *testing.T) {
	t.Parallel()
	top := t.TempDir()
	err := templates.New(top).Render(t.Context(), []domain.Page{
		page(t, "site/a.html", `<p>{{.Value.title}}</p>`),
		page(t, "site/b.html", `<p>{{.Value.missing}}</p>`),
	})
	var f *domain.Failure
	if !errors.As(err, &f) || f.Code != domain.TemplateInvalid {
		t.Errorf("Render = %v, want template_invalid", err)
	}
	if _, err := os.Stat(filepath.Join(top, "site", "a.html")); !errors.Is(err, os.ErrNotExist) {
		t.Errorf("Stat(site/a.html) = %v, want not exist", err)
	}
}

func TestRenderNamesTheTemplateFileInFailures(t *testing.T) {
	t.Parallel()
	err := templates.New(t.TempDir()).Render(t.Context(), []domain.Page{page(t, "site/a.html", "<p>\n{{.Value.missing}}</p>")})
	var f *domain.Failure
	if !errors.As(err, &f) || f.Path != "design/page.html" || f.Line != 2 {
		t.Errorf("Render = %+v, want template_invalid at design/page.html line 2", f)
	}
}

func TestReferencesCollectsTemplateAndScriptReferences(t *testing.T) {
	t.Parallel()
	source, err := domain.NewPath("design/page.html")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	text := `<h1 class="{{.Tokens.heading}}">{{.Value.title}}</h1>
{{range .Value.items}}<li>{{.name}}</li>{{end}}
{{with .Value.owner}}{{.mail}}{{end}}
{{range $i, $e := .Value.items}}{{$e.price}}{{end}}
<script>
const doc = {{.Value}};
for (const d of doc.decisions) { console.log(d.text, doc["title"], d[0]); }
const x = compute(doc).y;
</script>
<script type="application/json">{{.Value}}</script>`
	got, err := templates.New(t.TempDir()).References(domain.Template{Path: source, Text: text})
	if err != nil {
		t.Fatalf("References = %v", err)
	}
	var paths []string
	for _, r := range got {
		paths = append(paths, r.String())
	}
	for _, want := range []string{
		".Tokens.heading", ".Value.title", ".Value.items[].name", ".Value.owner.mail", ".Value.items[].price",
		".Value.decisions", ".Value.decisions[].text", ".Value.title",
	} {
		if !slices.Contains(paths, want) {
			t.Errorf("References() = %v, want to contain %s", paths, want)
		}
	}
	if slices.Contains(paths, ".Value.y") {
		t.Errorf("References() = %v, want no reference through a function call", paths)
	}
}

func TestReferencesRejectsScriptSyntaxErrors(t *testing.T) {
	t.Parallel()
	source, err := domain.NewPath("design/page.html")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	_, err = templates.New(t.TempDir()).References(domain.Template{Path: source, Text: "<p></p>\n<script>\nconst doc = {{.Value}};\nfor (const d of doc.items {\n</script>"})
	var f *domain.Failure
	if !errors.As(err, &f) || f.Code != domain.TemplateInvalid {
		t.Errorf("References = %v, want template_invalid", err)
	}
}
