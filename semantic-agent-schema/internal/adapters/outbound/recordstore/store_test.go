package recordstore_test

import (
	"database/sql"
	"errors"
	"path/filepath"
	"strings"
	"testing"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/recordstore"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

func validation(t *testing.T, content string) *domain.Validation {
	t.Helper()
	path, err := domain.NewPath("a.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	actor, err := domain.NewActor(false, false)
	if err != nil {
		t.Fatalf("NewActor = %v", err)
	}
	value, err := domain.ParseJSON([]byte(content))
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	concrete, err := domain.NewConcrete(nil, []byte(`{"$id": "urn:a"}`))
	if err != nil {
		t.Fatalf("NewConcrete = %v", err)
	}
	return domain.NewValidation(domain.ValidationInput{
		Actor: actor, Value: value, Concrete: concrete,
		Document:   domain.Document{Path: path, Content: []byte(content), Hash: domain.HashOf([]byte(content))},
		Violations: []domain.Violation{{InstanceLocation: domain.PointerOf("x"), AbsoluteKeywordLocation: "urn:a#/minimum", Message: "m"}},
	})
}

func open(t *testing.T, dir string) *recordstore.Store {
	t.Helper()
	store, err := recordstore.Open(t.Context(), dir)
	if err != nil {
		t.Fatalf("Open(%s) = %v", dir, err)
	}
	t.Cleanup(func() {
		if err := store.Close(); err != nil {
			t.Errorf("Close() = %v", err)
		}
	})
	return store
}

func TestAppendValidationGivesEachOperationAnID(t *testing.T) {
	t.Parallel()
	store := open(t, t.TempDir())
	var ids []int64
	for _, content := range []string{`{"x": 1}`, `{"x": 2}`} {
		id, err := store.AppendValidation(t.Context(), validation(t, content))
		if err != nil {
			t.Errorf("AppendValidation(%s) = %v", content, err)
		}
		ids = append(ids, id)
	}
	if ids[0] != 1 || ids[1] != 2 {
		t.Errorf("AppendValidation ids = %v, want [1 2]", ids)
	}
}

func TestLastValidationReturnsTheLatestValidationOfPath(t *testing.T) {
	t.Parallel()
	store := open(t, t.TempDir())
	path, err := domain.NewPath("a.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	previous, err := store.LastValidation(t.Context(), path)
	if previous != nil || err != nil {
		t.Errorf("LastValidation(a.json) before append = %v, %v, want nil, nil", previous, err)
	}
	for _, content := range []string{`{"x": 1}`, `{"x": 2}`} {
		if _, err := store.AppendValidation(t.Context(), validation(t, content)); err != nil {
			t.Fatalf("AppendValidation(%s) = %v", content, err)
		}
	}
	previous, err = store.LastValidation(t.Context(), path)
	if err != nil || previous == nil {
		t.Errorf("LastValidation(a.json) = %v, %v, want previous, nil", previous, err)
	}
}

func TestTablesAreAppendOnly(t *testing.T) {
	t.Parallel()
	dir := t.TempDir()
	store := open(t, dir)
	if _, err := store.AppendValidation(t.Context(), validation(t, `{"x": 1}`)); err != nil {
		t.Fatalf("AppendValidation = %v", err)
	}
	db := database(t, dir)
	for _, statement := range []string{
		"UPDATE validations SET path = 'b.json'",
		"DELETE FROM validations",
		"UPDATE judgments SET verdict = 'verified'",
		"DELETE FROM judgments",
	} {
		if _, err := db.ExecContext(t.Context(), statement); err == nil || !strings.Contains(err.Error(), "append-only") {
			t.Errorf("Exec(%s) = %v, want append-only error", statement, err)
		}
	}
	var command string
	if err := db.QueryRowContext(t.Context(), "SELECT command FROM operations WHERE id = 1").Scan(&command); err != nil || command != "validate" {
		t.Errorf("SELECT command FROM operations = %q, %v, want %q, nil", command, err, "validate")
	}
}

func TestOpenRejectsAnotherTableVersion(t *testing.T) {
	t.Parallel()
	dir := t.TempDir()
	store := open(t, dir)
	if _, err := store.AppendValidation(t.Context(), validation(t, `{"x": 1}`)); err != nil {
		t.Fatalf("AppendValidation = %v", err)
	}
	if _, err := database(t, dir).ExecContext(t.Context(), "PRAGMA user_version = 2"); err != nil {
		t.Fatalf("PRAGMA user_version = 2: %v", err)
	}
	_, err := recordstore.Open(t.Context(), dir)
	var f *domain.Failure
	if !errors.As(err, &f) || f.Code != domain.LogUnavailable {
		t.Errorf("Open(%s) = %v, want log_unavailable", dir, err)
	}
}

func database(t *testing.T, dir string) *sql.DB {
	t.Helper()
	db, err := sql.Open("sqlite", filepath.Join(dir, "semantic-agent-schema", "records.sqlite"))
	if err != nil {
		t.Fatalf("sql.Open = %v", err)
	}
	t.Cleanup(func() {
		if err := db.Close(); err != nil {
			t.Errorf("Close() = %v", err)
		}
	})
	return db
}
