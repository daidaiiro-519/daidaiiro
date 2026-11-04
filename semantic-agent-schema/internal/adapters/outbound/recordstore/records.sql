-- 操作の記録の表の形。版は PRAGMA user_version で持ち、recordstore の version と同じにする。
-- どの表も追記だけ。UPDATE と DELETE は trigger で拒否する。

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

CREATE TRIGGER validations_no_update BEFORE UPDATE ON validations
BEGIN SELECT RAISE(ABORT, 'validations is append-only'); END;

CREATE TRIGGER validations_no_delete BEFORE DELETE ON validations
BEGIN SELECT RAISE(ABORT, 'validations is append-only'); END;

CREATE TRIGGER judgments_no_update BEFORE UPDATE ON judgments
BEGIN SELECT RAISE(ABORT, 'judgments is append-only'); END;

CREATE TRIGGER judgments_no_delete BEFORE DELETE ON judgments
BEGIN SELECT RAISE(ABORT, 'judgments is append-only'); END;

-- 操作の記録の参照のビュー。いまは validations だけを持つ。
CREATE VIEW operations AS
SELECT id, 'validate' AS command, actor, path, at FROM validations
ORDER BY at;

PRAGMA user_version = 1;
