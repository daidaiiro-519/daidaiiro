package recordstore_test

import (
	"context"
	"database/sql"
	"errors"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"testing"

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/adapters/outbound/recordstore"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
)

func validation(t *testing.T, content string) *domain.Validation {
	t.Helper()
	return validationOf(t, content, nil)
}

// validationOf は、judged が nil でなければ、AI の判定を受け取った意味的な検証を返す。
func validationOf(t *testing.T, content string, judged []domain.Judgment) *domain.Validation {
	t.Helper()
	path, err := domain.NewPath("a.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
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
		Value: value, Concrete: concrete,
		Document:   domain.Document{Path: path, Content: []byte(content), Hash: domain.HashOf([]byte(content))},
		Violations: []domain.Violation{{InstanceLocation: domain.PointerOf("x"), AbsoluteKeywordLocation: "urn:a#/minimum", Message: "m"}},
		Judged:     judged,
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

func TestLastJudgedReturnsTheLatestSemanticValidationOfPath(t *testing.T) {
	t.Parallel()
	store := open(t, t.TempDir())
	path, err := domain.NewPath("a.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	previous, err := store.LastJudged(t.Context(), path)
	if previous != nil || err != nil {
		t.Errorf("LastJudged(a.json) before append = %v, %v, want nil, nil", previous, err)
	}
	// 機械的な検証だけでは、変更点の基準は動かない
	if _, err := store.AppendValidation(t.Context(), validation(t, `{"x": 1}`)); err != nil {
		t.Fatalf("AppendValidation = %v", err)
	}
	previous, err = store.LastJudged(t.Context(), path)
	if previous != nil || err != nil {
		t.Errorf("LastJudged(a.json) after a mechanical validation = %v, %v, want nil, nil", previous, err)
	}
	if _, err := store.AppendValidation(t.Context(), validationOf(t, `{"x": 2}`, []domain.Judgment{})); err != nil {
		t.Fatalf("AppendValidation(judged) = %v", err)
	}
	previous, err = store.LastJudged(t.Context(), path)
	if err != nil || previous == nil {
		t.Errorf("LastJudged(a.json) after a semantic validation = %v, %v, want previous, nil", previous, err)
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
	if _, err := database(t, dir).ExecContext(t.Context(), "PRAGMA user_version = 99"); err != nil {
		t.Fatalf("PRAGMA user_version = 99: %v", err)
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

func creation(t *testing.T, content string) *domain.Write {
	t.Helper()
	path, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	value, err := domain.ParseJSON([]byte(content))
	if err != nil {
		t.Fatalf("ParseJSON = %v", err)
	}
	concrete, err := domain.NewConcrete(nil, []byte(`{"$id": "urn:a"}`))
	if err != nil {
		t.Fatalf("NewConcrete = %v", err)
	}
	c, err := domain.NewCreation(domain.WriteInput{Path: path, Content: []byte(content), Value: value, Concrete: concrete})
	if err != nil {
		t.Fatalf("NewCreation = %v", err)
	}
	return c
}

func TestAppendChangeRecordsChangesInsideLock(t *testing.T) {
	t.Parallel()
	dir := t.TempDir()
	store := open(t, dir)
	if _, err := store.AppendChange(t.Context(), creation(t, `{"x": 1}`)); err == nil {
		t.Errorf("AppendChange outside WithLock = nil error, want error")
	}
	var id int64
	err := store.WithLock(t.Context(), func(ctx context.Context) error {
		var err error
		id, err = store.AppendChange(ctx, creation(t, `{"x": 1}`))
		return err
	})
	if err != nil || id != 1 {
		t.Errorf("WithLock(AppendChange) = %d, %v, want 1, nil", id, err)
	}
	db := database(t, dir)
	var count int
	if err := db.QueryRowContext(t.Context(), "SELECT count(*) FROM field_changes WHERE change_id = 1").Scan(&count); err != nil || count != 2 {
		t.Errorf("SELECT count(*) FROM field_changes = %d, %v, want 2, nil", count, err)
	}
	var command string
	if err := db.QueryRowContext(t.Context(), "SELECT command FROM operations").Scan(&command); err != nil || command != "create" {
		t.Errorf("SELECT command FROM operations = %q, %v, want %q, nil", command, err, "create")
	}
	for _, statement := range []string{"UPDATE changes SET path = 'b.json'", "DELETE FROM field_changes"} {
		if _, err := db.ExecContext(t.Context(), statement); err == nil || !strings.Contains(err.Error(), "append-only") {
			t.Errorf("Exec(%s) = %v, want append-only error", statement, err)
		}
	}
}

func TestWithLockRollsBackWhenFunctionFails(t *testing.T) {
	t.Parallel()
	dir := t.TempDir()
	store := open(t, dir)
	failure := domain.Fail(domain.FileExists, "memo.json", "memo.json")
	err := store.WithLock(t.Context(), func(ctx context.Context) error {
		if _, err := store.AppendChange(ctx, creation(t, `{"x": 1}`)); err != nil {
			return err
		}
		return failure
	})
	if !errors.Is(err, failure) {
		t.Errorf("WithLock = %v, want %v", err, failure)
	}
	var count int
	if err := database(t, dir).QueryRowContext(t.Context(), "SELECT count(*) FROM changes").Scan(&count); err != nil || count != 0 {
		t.Errorf("SELECT count(*) FROM changes = %d, %v, want 0, nil", count, err)
	}
}

func TestOpenExistingDoesNotCreateRecords(t *testing.T) {
	t.Parallel()
	dir := t.TempDir()
	store, err := recordstore.OpenExisting(t.Context(), dir)
	if err != nil {
		t.Fatalf("OpenExisting(%s) = %v", dir, err)
	}
	operations, err := store.Operations(t.Context(), nil, 0)
	if err != nil || len(operations) != 0 {
		t.Errorf("Operations() = %v, %v, want none, nil", operations, err)
	}
	if err := store.Close(); err != nil {
		t.Errorf("Close() = %v", err)
	}
	if _, err := os.Stat(filepath.Join(dir, "semantic-agent-schema", "records.sqlite")); !errors.Is(err, os.ErrNotExist) {
		t.Errorf("Stat(records.sqlite) = %v, want not exist", err)
	}
}

func TestOperationsReturnsNewestFirstAndOperationReturnsDetail(t *testing.T) {
	t.Parallel()
	dir := t.TempDir()
	store := open(t, dir)
	if _, err := store.AppendValidation(t.Context(), validation(t, `{"x": 1}`)); err != nil {
		t.Fatalf("AppendValidation = %v", err)
	}
	if err := store.WithLock(t.Context(), func(ctx context.Context) error {
		_, err := store.AppendChange(ctx, creation(t, `{"x": 1}`))
		return err
	}); err != nil {
		t.Fatalf("AppendChange = %v", err)
	}
	reader, err := recordstore.OpenExisting(t.Context(), dir)
	if err != nil {
		t.Fatalf("OpenExisting = %v", err)
	}
	t.Cleanup(func() {
		if err := reader.Close(); err != nil {
			t.Errorf("Close() = %v", err)
		}
	})
	operations, err := reader.Operations(t.Context(), nil, 0)
	var got []string
	for _, o := range operations {
		got = append(got, o.Command+" "+o.Path)
	}
	if want := []string{"create memo.json", "validate a.json"}; err != nil || !slices.Equal(got, want) {
		t.Errorf("Operations() = %v, %v, want %v, nil", got, err, want)
	}
	memo, err := domain.NewPath("memo.json")
	if err != nil {
		t.Fatalf("NewPath = %v", err)
	}
	if operations, err := reader.Operations(t.Context(), []domain.Path{memo}, 1); err != nil || len(operations) != 1 {
		t.Errorf("Operations(memo.json, 1) = %v, %v, want 1 operation", operations, err)
	}
	one, err := reader.Operation(t.Context(), "validate", 1)
	if err != nil || one == nil || one.Detail.Validation != "validated" || len(one.Detail.Judgments) != 1 {
		t.Errorf("Operation(validate, 1) = %+v, %v, want validated with 1 judgment", one, err)
	}
	created, err := reader.Operation(t.Context(), "create", 1)
	if err != nil || created == nil || len(created.Detail.Changes) != 2 {
		t.Errorf("Operation(create, 1) = %+v, %v, want 2 changes", created, err)
	}
	if missing, err := reader.Operation(t.Context(), "update", 1); missing != nil || err != nil {
		t.Errorf("Operation(update, 1) = %+v, %v, want nil, nil", missing, err)
	}
}
