-- ステップ1：呼び出し元のセッション（SAS が立てたセッションは別の文脈の表に持つ）と、検証の操作と、rule ごとの結果。
-- 追記だけ（UPDATE ・ DELETE を trigger で拒む）。閉じたセッションには足さない。by = human は unverified だけ。

CREATE TABLE sessions (
    id         TEXT PRIMARY KEY,
    actor      TEXT NOT NULL CHECK (actor IN ('human', 'tool')),
    tool_ref   TEXT,
    started_at TEXT NOT NULL,
    ended_at   TEXT,
    CHECK ((actor = 'tool') = (tool_ref IS NOT NULL))
) STRICT;

CREATE UNIQUE INDEX sessions_tool_ref ON sessions (tool_ref) WHERE tool_ref IS NOT NULL;

CREATE TABLE operations (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES sessions (id),
    seq         INTEGER NOT NULL,
    kind        TEXT NOT NULL CHECK (kind IN ('validate')),
    target_kind TEXT NOT NULL CHECK (target_kind IN ('concrete', 'instance')),
    target_path TEXT NOT NULL,
    before_hash TEXT,
    after_hash  TEXT,
    at          TEXT NOT NULL,
    UNIQUE (session_id, seq),
    CHECK (kind <> 'validate' OR before_hash IS after_hash)
) STRICT;

CREATE TABLE rule_results (
    operation_id TEXT NOT NULL REFERENCES operations (id),
    ord          INTEGER NOT NULL,
    rule_id      TEXT NOT NULL,
    pointer      TEXT NOT NULL,
    by           TEXT NOT NULL CHECK (by IN ('machine', 'ai', 'human')),
    result       TEXT NOT NULL CHECK (result IN ('verified', 'violated', 'unverified', 'error', 'unsupported')),
    reason       TEXT NOT NULL,
    PRIMARY KEY (operation_id, ord),
    CHECK (by IS NOT 'human' OR result = 'unverified')
) STRICT;

CREATE TRIGGER sessions_no_delete BEFORE DELETE ON sessions
BEGIN SELECT RAISE(ABORT, 'sessions are append-only'); END;

CREATE TRIGGER sessions_only_close BEFORE UPDATE ON sessions
WHEN OLD.ended_at IS NOT NULL OR NEW.id IS NOT OLD.id OR NEW.actor IS NOT OLD.actor
  OR NEW.tool_ref IS NOT OLD.tool_ref OR NEW.started_at IS NOT OLD.started_at
BEGIN SELECT RAISE(ABORT, 'a session can only be closed once'); END;

CREATE TRIGGER operations_no_update BEFORE UPDATE ON operations
BEGIN SELECT RAISE(ABORT, 'operations are append-only'); END;

CREATE TRIGGER operations_no_delete BEFORE DELETE ON operations
BEGIN SELECT RAISE(ABORT, 'operations are append-only'); END;

CREATE TRIGGER operations_open_session BEFORE INSERT ON operations
WHEN (SELECT ended_at FROM sessions WHERE id = NEW.session_id) IS NOT NULL
BEGIN SELECT RAISE(ABORT, 'the session is closed'); END;

CREATE TRIGGER rule_results_no_update BEFORE UPDATE ON rule_results
BEGIN SELECT RAISE(ABORT, 'rule results are append-only'); END;

CREATE TRIGGER rule_results_no_delete BEFORE DELETE ON rule_results
BEGIN SELECT RAISE(ABORT, 'rule results are append-only'); END;
