// Package recordstore は、アダプター。中核の業務領域と操作の記録の参照の、2つのポート Records を SQLite の操作の記録で満たす。
// 表の形は records.sql が決め、どの表も追記だけ。書く操作は BEGIN IMMEDIATE のロックの中で行う。
package recordstore

import (
	"context"
	"database/sql"
	_ "embed"
	"errors"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	_ "modernc.org/sqlite" // database/sql にドライバー名 "sqlite" で登録される

	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/authoring/domain"
	"github.com/daidaiiro-519/daidaiiro/semantic-agent-schema/internal/record/application"
)

//go:embed records.sql
var records string

// version は、表の形の版。records.sql の PRAGMA user_version と同じ。
const version = 5

// Store は、<git-common-dir>/semantic-agent-schema/records.sqlite の操作の記録。
type Store struct {
	db   *sql.DB
	file string
	// locked は、WithLock の中で、ロックを取った接続。WithLock の外では nil
	locked *sql.Conn
}

// Open は、操作の記録を開いて返す。無ければ作る。表の形の版が違えば開かず、log_unavailable の Failure を返す。
func Open(ctx context.Context, commonDir string) (*Store, error) {
	dir := filepath.Join(commonDir, "semantic-agent-schema")
	file := filepath.Join(dir, "records.sqlite")
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return nil, domain.Fail(domain.LogUnavailable, "", err.Error())
	}
	db, err := sql.Open("sqlite", "file:"+file+"?_pragma=busy_timeout(5000)&_pragma=foreign_keys(1)")
	if err != nil {
		return nil, domain.Fail(domain.LogUnavailable, "", err.Error())
	}
	s := &Store{db: db, file: file}
	if err := s.prepare(ctx); err != nil {
		return nil, errors.Join(err, s.Close())
	}
	return s, nil
}

// OpenExisting は、操作の記録を読むだけのために開く。記録を変えないので、ファイルが無くても作らず、
// 空の操作の記録として返す。表の形の版が違えば開かず、log_unavailable の Failure を返す。
func OpenExisting(ctx context.Context, commonDir string) (*Store, error) {
	file := filepath.Join(commonDir, "semantic-agent-schema", "records.sqlite")
	if _, err := os.Stat(file); errors.Is(err, os.ErrNotExist) {
		return &Store{file: file}, nil
	}
	db, err := sql.Open("sqlite", "file:"+file+"?mode=ro&_pragma=busy_timeout(5000)")
	if err != nil {
		return nil, domain.Fail(domain.LogUnavailable, "", err.Error())
	}
	s := &Store{db: db, file: file}
	var current int
	if err := db.QueryRowContext(ctx, "PRAGMA user_version").Scan(&current); err != nil {
		return nil, errors.Join(s.failure(err), s.Close())
	}
	if current != version {
		return nil, errors.Join(domain.Fail(domain.LogUnavailable, "",
			file+": user_version "+strconv.Itoa(current)+" != "+strconv.Itoa(version)), s.Close())
	}
	return s, nil
}

// Close は、操作の記録を閉じる。
func (s *Store) Close() error {
	if s.db == nil {
		return nil // 無いファイルを読むために開いたので、閉じるものが無い
	}
	if err := s.db.Close(); err != nil {
		return s.failure(err)
	}
	return nil
}

// prepare は、版が 0（新しいファイル）なら表を作り、版が違えば log_unavailable の Failure を返す。
func (s *Store) prepare(ctx context.Context) error {
	return s.withLock(ctx, func(conn *sql.Conn) error {
		var current int
		if err := conn.QueryRowContext(ctx, "PRAGMA user_version").Scan(&current); err != nil {
			return s.failure(err)
		}
		switch current {
		case version:
			return nil
		case 0:
			if _, err := conn.ExecContext(ctx, records); err != nil {
				return s.failure(err)
			}
			return nil
		default:
			return domain.Fail(domain.LogUnavailable, "",
				s.file+": user_version "+strconv.Itoa(current)+" != "+strconv.Itoa(version))
		}
	})
}

// withLock は、BEGIN IMMEDIATE で書き込みのロックを取り、f が error を返さなければ COMMIT する。
func (s *Store) withLock(ctx context.Context, f func(conn *sql.Conn) error) (err error) {
	conn, err := s.db.Conn(ctx)
	if err != nil {
		return s.failure(err)
	}
	defer func() {
		if closeErr := conn.Close(); closeErr != nil {
			err = errors.Join(err, s.failure(closeErr))
		}
	}()
	if _, err := conn.ExecContext(ctx, "BEGIN IMMEDIATE"); err != nil {
		return s.failure(err)
	}
	if err := f(conn); err != nil {
		return errors.Join(err, s.rollback(ctx, conn))
	}
	if _, err := conn.ExecContext(ctx, "COMMIT"); err != nil {
		return errors.Join(s.failure(err), s.rollback(ctx, conn))
	}
	return nil
}

func (s *Store) rollback(ctx context.Context, conn *sql.Conn) error {
	if _, err := conn.ExecContext(ctx, "ROLLBACK"); err != nil {
		return s.failure(err)
	}
	return nil
}

// failure は、ロックが取れないときは log_locked、ほかは log_unavailable の Failure を返す。
func (s *Store) failure(err error) error {
	text := err.Error()
	if strings.Contains(text, "SQLITE_BUSY") || strings.Contains(text, "database is locked") {
		return domain.Fail(domain.LogLocked, "", s.file+": "+text)
	}
	return domain.Fail(domain.LogUnavailable, "", s.file+": "+text)
}

// WithLock は、BEGIN IMMEDIATE で書き込みのロックを取り、その中で f を行う。f が error を返さなければ COMMIT し、
// 返せば ROLLBACK する。書く操作（いまのハッシュの確認 ・ ファイルを書く ・ 追記）は、f の中で行う。
func (s *Store) WithLock(ctx context.Context, f func(ctx context.Context) error) error {
	return s.withLock(ctx, func(conn *sql.Conn) error {
		s.locked = conn
		defer func() { s.locked = nil }()
		return f(ctx)
	})
}

// AppendChange は、WithLock の中で、書く操作（作成か更新）の change を追記し、操作の ID を返す。
func (s *Store) AppendChange(ctx context.Context, c *domain.Write) (int64, error) {
	conn := s.locked
	if conn == nil {
		return 0, domain.Fail(domain.LogUnavailable, "", s.file+": AppendChange outside WithLock")
	}
	document := c.Document()
	result, err := conn.ExecContext(ctx,
		`INSERT INTO changes (path, schema_id, command, before_hash, after_hash, at)
		 VALUES (?, ?, ?, ?, ?, ?)`,
		document.Path.String(), c.Concrete().ID(), string(c.Command()),
		nullableHash(c.BeforeHash()), document.Hash.String(),
		now())
	if err != nil {
		return 0, s.failure(err)
	}
	id, err := result.LastInsertId()
	if err != nil {
		return 0, s.failure(err)
	}
	for _, change := range c.Changes() {
		_, err := conn.ExecContext(ctx,
			`INSERT INTO field_changes (change_id, instance_location, op, before, after) VALUES (?, ?, ?, ?, ?)`,
			id, change.InstanceLocation().String(), string(change.Op()), nullable(change.Before()), nullable(change.After()))
		if err != nil {
			return 0, s.failure(err)
		}
	}
	return id, nil
}

// nullableHash は、作成の書く前のハッシュ（nil）を NULL にする。
func nullableHash(h domain.Hash) sql.NullString {
	if h == nil {
		return sql.NullString{}
	}
	return sql.NullString{String: h.String(), Valid: true}
}

// nullable は、無い値（空）を NULL にする。
func nullable(text string) sql.NullString {
	return sql.NullString{String: text, Valid: text != ""}
}

// LastJudged は、path の前回の意味的な検証（AI の判定を受け取った検証）を返す。無ければ nil を返す。
func (s *Store) LastJudged(ctx context.Context, path domain.Path) (*domain.Previous, error) {
	var (
		id           int64
		concreteHash string
		content      string
	)
	err := s.db.QueryRowContext(ctx,
		"SELECT id, concrete_hash, content FROM validations WHERE path = ? AND event = 'judged' ORDER BY id DESC LIMIT 1",
		path.String()).Scan(&id, &concreteHash, &content)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, s.failure(err)
	}
	judgments, err := s.judgments(ctx, id)
	if err != nil {
		return nil, err
	}
	return domain.NewPrevious([]byte(content), concreteHash, judgments)
}

// LastValidation は、path の直前の検証（機械的な検証か意味的な検証）を返す。無ければ nil を返す。
func (s *Store) LastValidation(ctx context.Context, path domain.Path) (*domain.Previous, error) {
	var (
		id           int64
		concreteHash string
		content      string
	)
	err := s.db.QueryRowContext(ctx,
		"SELECT id, concrete_hash, content FROM validations WHERE path = ? ORDER BY id DESC LIMIT 1",
		path.String()).Scan(&id, &concreteHash, &content)
	if errors.Is(err, sql.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, s.failure(err)
	}
	judgments, err := s.judgments(ctx, id)
	if err != nil {
		return nil, err
	}
	return domain.NewPrevious([]byte(content), concreteHash, judgments)
}

func (s *Store) judgments(ctx context.Context, validationID int64) (judgments []domain.Judgment, err error) {
	rows, err := s.db.QueryContext(ctx,
		`SELECT rule_id, instance_location, "by", verdict, reason FROM judgments WHERE validation_id = ? ORDER BY rowid`,
		validationID)
	if err != nil {
		return nil, s.failure(err)
	}
	defer func() {
		if closeErr := rows.Close(); closeErr != nil {
			err = errors.Join(err, s.failure(closeErr))
		}
	}()
	for rows.Next() {
		var rule, location, by, verdict, reason string
		if err := rows.Scan(&rule, &location, &by, &verdict, &reason); err != nil {
			return nil, s.failure(err)
		}
		j, err := domain.NewJudgment(rule, location, verdict, by, reason)
		if err != nil {
			return nil, err
		}
		judgments = append(judgments, j)
	}
	if err := rows.Err(); err != nil {
		return nil, s.failure(err)
	}
	return judgments, nil
}

// AppendValidation は、ロックの中で、検証した JSON ドキュメントと judgment を追記し、操作の ID を返す。
func (s *Store) AppendValidation(ctx context.Context, v *domain.Validation) (int64, error) {
	var id int64
	err := s.withLock(ctx, func(conn *sql.Conn) error {
		document := v.Document()
		result, err := conn.ExecContext(ctx,
			`INSERT INTO validations (path, schema_id, concrete_hash, content, content_hash, event, at)
			 VALUES (?, ?, ?, ?, ?, ?, ?)`,
			document.Path.String(), v.Concrete().ID(), v.Concrete().Hash().String(),
			string(document.Content), document.Hash.String(), eventOf(v), now())
		if err != nil {
			return s.failure(err)
		}
		if id, err = result.LastInsertId(); err != nil {
			return s.failure(err)
		}
		for _, j := range v.Judgments() {
			_, err := conn.ExecContext(ctx,
				`INSERT INTO judgments (validation_id, rule_id, instance_location, "by", verdict, reason)
				 VALUES (?, ?, ?, ?, ?, ?)`,
				id, j.Rule().ID(), j.InstanceLocation().String(), string(j.By()), string(j.Verdict()), j.Reason())
			if err != nil {
				return s.failure(err)
			}
		}
		return nil
	})
	return id, err
}

// eventOf は、検証のログイベントの名前を返す。AI の判定を受け取った意味的な検証は judged、ほかは validated。
func eventOf(v *domain.Validation) string {
	if v.Judged() {
		return "judged"
	}
	return "validated"
}

// Operations は、ビュー operations を新しい順に読み、path（空ならすべて）の操作を limit 件（0 なら全部）まで返す。
func (s *Store) Operations(ctx context.Context, paths []domain.Path, limit int) (operations []application.Operation, err error) {
	if s.db == nil {
		return nil, nil // 操作の記録のファイルが無いので、操作も無い
	}
	query := "SELECT id, command, path, at FROM operations"
	var values []any
	if len(paths) > 0 {
		query += " WHERE path IN (?" + strings.Repeat(", ?", len(paths)-1) + ")"
		for _, p := range paths {
			values = append(values, p.String())
		}
	}
	query += " ORDER BY at DESC, id DESC"
	if limit > 0 {
		query += " LIMIT ?"
		values = append(values, limit)
	}
	rows, err := s.db.QueryContext(ctx, query, values...)
	if err != nil {
		return nil, s.failure(err)
	}
	defer func() {
		if closeErr := rows.Close(); closeErr != nil {
			err = errors.Join(err, s.failure(closeErr))
		}
	}()
	for rows.Next() {
		var o application.Operation
		if err := rows.Scan(&o.ID, &o.Command, &o.Path, &o.At); err != nil {
			return nil, s.failure(err)
		}
		operations = append(operations, o)
	}
	if err := rows.Err(); err != nil {
		return nil, s.failure(err)
	}
	return operations, nil
}

// Operation は、command と ID の操作を、書く操作なら field_changes、検証なら judgments と一緒に返す。無ければ nil。
func (s *Store) Operation(ctx context.Context, command string, id int64) (*application.Operation, error) {
	if s.db == nil {
		return nil, nil // 操作の記録のファイルが無いので、操作も無い
	}
	o := application.Operation{ID: id, Command: command, Detail: &application.Detail{}}
	var err error
	if command == "validate" {
		err = s.db.QueryRowContext(ctx,
			"SELECT path, at, schema_id, content_hash, event FROM validations WHERE id = ?", id).
			Scan(&o.Path, &o.At, &o.Detail.SchemaID, &o.Detail.ContentHash, &o.Detail.Validation)
	} else {
		var before sql.NullString
		err = s.db.QueryRowContext(ctx,
			"SELECT path, at, schema_id, before_hash, after_hash FROM changes WHERE id = ? AND command = ?", id, command).
			Scan(&o.Path, &o.At, &o.Detail.SchemaID, &before, &o.Detail.ContentHash)
		o.Detail.BeforeHash = before.String
	}
	if errors.Is(err, sql.ErrNoRows) {
		return nil, nil
	}
	if err != nil {
		return nil, s.failure(err)
	}
	if command == "validate" {
		err = s.judgmentsOf(ctx, &o)
	} else {
		err = s.changesOf(ctx, &o)
	}
	if err != nil {
		return nil, err
	}
	return &o, nil
}

func (s *Store) changesOf(ctx context.Context, o *application.Operation) (err error) {
	rows, err := s.db.QueryContext(ctx,
		"SELECT instance_location, op, before, after FROM field_changes WHERE change_id = ? ORDER BY rowid", o.ID)
	if err != nil {
		return s.failure(err)
	}
	defer func() {
		if closeErr := rows.Close(); closeErr != nil {
			err = errors.Join(err, s.failure(closeErr))
		}
	}()
	for rows.Next() {
		var c application.Change
		var before, after sql.NullString
		if err := rows.Scan(&c.InstanceLocation, &c.Op, &before, &after); err != nil {
			return s.failure(err)
		}
		c.Before, c.After = before.String, after.String
		o.Detail.Changes = append(o.Detail.Changes, c)
	}
	if err := rows.Err(); err != nil {
		return s.failure(err)
	}
	return nil
}

func (s *Store) judgmentsOf(ctx context.Context, o *application.Operation) (err error) {
	rows, err := s.db.QueryContext(ctx,
		`SELECT rule_id, instance_location, verdict, "by", reason FROM judgments WHERE validation_id = ? ORDER BY rowid`, o.ID)
	if err != nil {
		return s.failure(err)
	}
	defer func() {
		if closeErr := rows.Close(); closeErr != nil {
			err = errors.Join(err, s.failure(closeErr))
		}
	}()
	for rows.Next() {
		var j application.Judgment
		if err := rows.Scan(&j.Rule, &j.InstanceLocation, &j.Verdict, &j.By, &j.Reason); err != nil {
			return s.failure(err)
		}
		o.Detail.Judgments = append(o.Detail.Judgments, j)
	}
	if err := rows.Err(); err != nil {
		return s.failure(err)
	}
	return nil
}

// now は、操作の時刻を、桁をそろえた UTC の RFC 3339 で返す。文字列の順が時刻の順になるので、ビュー operations を at で並べられる。
func now() string {
	return time.Now().UTC().Format("2006-01-02T15:04:05.000000000Z07:00")
}
