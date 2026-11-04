-- 操作の記録の表の形。版は PRAGMA user_version で持ち、recordstore の version と同じにする。
-- どの表も追記だけ。UPDATE と DELETE は trigger で拒否する。

CREATE TABLE changes (
  id          INTEGER PRIMARY KEY,
  actor       TEXT NOT NULL CHECK (actor IN ('ai', 'human')),
  path        TEXT NOT NULL,
  schema_id   TEXT NOT NULL,
  command     TEXT NOT NULL CHECK (command IN ('create', 'update')),
  before_hash TEXT,
  after_hash  TEXT NOT NULL,
  at          TEXT NOT NULL
);

CREATE TABLE field_changes (
  change_id         INTEGER NOT NULL REFERENCES changes (id),
  instance_location TEXT NOT NULL,
  op                TEXT NOT NULL CHECK (op IN ('add', 'remove', 'replace')),
  before            TEXT,
  after             TEXT
);

CREATE INDEX field_changes_change ON field_changes (change_id);

CREATE TABLE validations (
  id            INTEGER PRIMARY KEY,
  actor         TEXT NOT NULL CHECK (actor IN ('ai', 'human')),
  path          TEXT NOT NULL,
  schema_id     TEXT NOT NULL,
  concrete_hash TEXT NOT NULL,
  content       TEXT NOT NULL,
  content_hash  TEXT NOT NULL,
  at            TEXT NOT NULL
);

CREATE INDEX validations_path ON validations (path, id);

CREATE TABLE judgments (
  validation_id     INTEGER NOT NULL REFERENCES validations (id),
  rule_id           TEXT NOT NULL,
  instance_location TEXT NOT NULL,
  "by"              TEXT NOT NULL CHECK ("by" IN ('machine', 'ai')),
  verdict           TEXT NOT NULL CHECK (verdict IN ('verified', 'violated', 'unverified', 'not-judged')),
  reason            TEXT NOT NULL
);

CREATE INDEX judgments_validation ON judgments (validation_id);

CREATE TRIGGER changes_no_update BEFORE UPDATE ON changes
BEGIN SELECT RAISE(ABORT, 'changes is append-only'); END;

CREATE TRIGGER changes_no_delete BEFORE DELETE ON changes
BEGIN SELECT RAISE(ABORT, 'changes is append-only'); END;

CREATE TRIGGER field_changes_no_update BEFORE UPDATE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field_changes is append-only'); END;

CREATE TRIGGER field_changes_no_delete BEFORE DELETE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field_changes is append-only'); END;

CREATE TRIGGER validations_no_update BEFORE UPDATE ON validations
BEGIN SELECT RAISE(ABORT, 'validations is append-only'); END;

CREATE TRIGGER validations_no_delete BEFORE DELETE ON validations
BEGIN SELECT RAISE(ABORT, 'validations is append-only'); END;

CREATE TRIGGER judgments_no_update BEFORE UPDATE ON judgments
BEGIN SELECT RAISE(ABORT, 'judgments is append-only'); END;

CREATE TRIGGER judgments_no_delete BEFORE DELETE ON judgments
BEGIN SELECT RAISE(ABORT, 'judgments is append-only'); END;

-- 操作の記録の参照のビュー。changes と validations を UNION ALL し、at の順に並べる。
CREATE VIEW operations AS
SELECT id, command, actor, path, at FROM changes
UNION ALL
SELECT id, 'validate' AS command, actor, path, at FROM validations
ORDER BY at;

PRAGMA user_version = 2;
