package templates_test

import (
	"errors"
	"os"
	"path/filepath"
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
	return domain.Page{Out: p, Template: template, Value: map[string]any{"title": "<t>"}, Tokens: map[string]any{"c": "x"}}
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
