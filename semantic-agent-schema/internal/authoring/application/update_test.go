package application_test

import (
	"context"
	"errors"
	"os"
	"os/exec"
	"path/filepath"
	"testing"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/files"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/patching"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/recordstore"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/schemacheck"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/application"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/publishedlanguage"
)

// changeBeforeLock は、本物の Records を包み、WithLock に入る直前に change を行う型。WithLock に入る直前に別の書き手がファイルを書き換えたことを表す Records。
type changeBeforeLock struct {
	application.Records
	change func()
}

func (r changeBeforeLock) WithLock(ctx context.Context, f func(ctx context.Context) error) error {
	r.change()
	return r.Records.WithLock(ctx, f)
}

func TestUpdateRunComparesHashAgainInsideLock(t *testing.T) {
	t.Parallel()
	ctx := t.Context()
	top := t.TempDir()
	schema, err := os.ReadFile("../../../cmd/semantic-agent-schema/testdata/fixtures/memo.schema.json")
	if err != nil {
		t.Fatalf("ReadFile(memo.schema.json) = %v", err)
	}
	original := []byte("{\n  \"$schema\": \"urn:example:memo:v1\",\n  \"title\": \"定例会議\"\n}\n")
	other := []byte("{\n  \"$schema\": \"urn:example:memo:v1\",\n  \"title\": \"別の人が書いた\"\n}\n")
	for name, content := range map[string][]byte{"schemas/memo.json": schema, "memo.json": original} {
		if err := os.MkdirAll(filepath.Dir(filepath.Join(top, name)), 0o755); err != nil {
			t.Fatalf("MkdirAll(%s) = %v", name, err)
		}
		if err := os.WriteFile(filepath.Join(top, name), content, 0o644); err != nil {
			t.Fatalf("WriteFile(%s) = %v", name, err)
		}
	}
	if out, err := exec.Command("git", "init", "-q", top).CombinedOutput(); err != nil {
		t.Fatalf("git init = %v: %s", err, out)
	}
	records, err := recordstore.Open(ctx, filepath.Join(top, ".git"))
	if err != nil {
		t.Fatalf("Open = %v", err)
	}
	t.Cleanup(func() {
		if err := records.Close(); err != nil {
			t.Errorf("Close = %v", err)
		}
	})
	abstract, err := publishedlanguage.Read("abstract.schema.json")
	if err != nil {
		t.Fatalf("Read(abstract.schema.json) = %v", err)
	}
	dir, err := domain.NewPath("schemas")
	if err != nil {
		t.Fatalf("NewPath(schemas) = %v", err)
	}
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath(memo.json) = %v", err)
	}
	concretes := files.NewSchemas(top, []domain.Path{dir}, map[string][]byte{publishedlanguage.AbstractID: abstract})
	update := application.Update{
		Documents: files.NewDocuments(top), Schemas: concretes, Validator: schemacheck.New(concretes.Content),
		Patcher: patching.New(), AbstractID: publishedlanguage.AbstractID, PatchID: publishedlanguage.PatchID,
		Records: changeBeforeLock{Records: records, change: func() {
			if err := os.WriteFile(filepath.Join(top, "memo.json"), other, 0o644); err != nil {
				t.Errorf("WriteFile(memo.json) = %v", err)
			}
		}},
	}
	_, _, err = update.Run(ctx, application.UpdateInput{
		Path:  path,
		Patch: []byte(`[{"op": "replace", "path": "/title", "value": "週次会議"}]`),
	})
	var f *domain.Failure
	if !errors.As(err, &f) || f.Code != domain.HashMismatch {
		t.Fatalf("Run(file changed after read) = %v, want hash_mismatch", err)
	}
	got, err := os.ReadFile(filepath.Join(top, "memo.json"))
	if err != nil || string(got) != string(other) {
		t.Errorf("ReadFile(memo.json) = %q, %v, want the other writer's content %q", got, err, other)
	}
}
