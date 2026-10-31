-- ステップ1：誰が何で動いたか（sessions）と、検証（validations）と rule ごとの結果（rule_results）。
-- ステップ4b で書き直した（公開前なので移行の番号は足さない）。セッションは状態を持たない：開閉 ・ 終わり方 ・ 続き番号は無い。
-- どの表も追記だけ（UPDATE ・ DELETE を trigger で拒む）。by = human は unverified だけ。

-- 親は SAS が初めて見たときに1行（role = caller）。子（writer ・ checker）は起動したときに1行で、ステップ6 から書く
CREATE TABLE sessions (
    id         TEXT PRIMARY KEY,                          -- ツールのセッション ID（親は --session か環境変数）
    role       TEXT NOT NULL CHECK (role IN ('caller', 'writer', 'checker')),
    parent_id  TEXT REFERENCES sessions (id),             -- 子を起動させた親。親の行と、人が起動させた子は NULL
    tool       TEXT NOT NULL,                             -- claude-code など。分からなければ unknown
    model      TEXT,                                      -- 子を起動したモデル。親は分からなければ NULL
    launch     TEXT CHECK (launch IS NULL OR json_valid(launch)),  -- 子の起動の設定。--continue で同じものを渡す
    first_seen TEXT NOT NULL,
    CHECK (role <> 'caller' OR (parent_id IS NULL AND launch IS NULL)),
    CHECK (role = 'caller' OR launch IS NOT NULL)
) STRICT;

CREATE INDEX sessions_by_parent ON sessions (parent_id, role) WHERE parent_id IS NOT NULL;

CREATE TRIGGER sessions_no_update BEFORE UPDATE ON sessions
BEGIN SELECT RAISE(ABORT, 'sessions are append-only'); END;

CREATE TRIGGER sessions_no_delete BEFORE DELETE ON sessions
BEGIN SELECT RAISE(ABORT, 'sessions are append-only'); END;

CREATE TRIGGER sessions_parent_is_caller BEFORE INSERT ON sessions
WHEN NEW.parent_id IS NOT NULL AND (SELECT role FROM sessions WHERE id = NEW.parent_id) IS NOT 'caller'
BEGIN SELECT RAISE(ABORT, 'the parent must be a caller session'); END;

-- 検証。対象を変えないので指紋は1つ。ID は changes と同じ採番（o-NNNN）から振る
CREATE TABLE validations (
    id                TEXT PRIMARY KEY,
    at                TEXT NOT NULL,
    target_path       TEXT NOT NULL,
    hash              TEXT NOT NULL,
    parent_session_id TEXT REFERENCES sessions (id),      -- 呼んだ親。人は NULL
    child_session_id  TEXT REFERENCES sessions (id)       -- 判定した確かめる子。機械と人の rule だけなら NULL
) STRICT;

CREATE INDEX validations_by_target ON validations (target_path, id);

-- rule ごとの結果。reads_hash は判定が読んだ所の指紋（入力が変わった rule だけを検証するときに比べる）
CREATE TABLE rule_results (
    validation_id TEXT NOT NULL REFERENCES validations (id),
    ord           INTEGER NOT NULL,
    rule_id       TEXT NOT NULL,
    pointer       TEXT NOT NULL,
    by            TEXT NOT NULL CHECK (by IN ('machine', 'ai', 'human')),
    result        TEXT NOT NULL CHECK (result IN ('verified', 'violated', 'unverified', 'error', 'unsupported')),
    reason        TEXT NOT NULL,
    reads_hash    TEXT NOT NULL,
    PRIMARY KEY (validation_id, ord),
    CHECK (by IS NOT 'human' OR result = 'unverified')
) STRICT;

CREATE TRIGGER validations_parent_is_caller BEFORE INSERT ON validations
WHEN NEW.parent_session_id IS NOT NULL AND (SELECT role FROM sessions WHERE id = NEW.parent_session_id) IS NOT 'caller'
BEGIN SELECT RAISE(ABORT, 'parent_session_id must point to a caller session'); END;

CREATE TRIGGER validations_child_is_checker BEFORE INSERT ON validations
WHEN NEW.child_session_id IS NOT NULL AND (SELECT role FROM sessions WHERE id = NEW.child_session_id) IS NOT 'checker'
BEGIN SELECT RAISE(ABORT, 'a validation is made by a checker'); END;

CREATE TRIGGER validations_no_update BEFORE UPDATE ON validations
BEGIN SELECT RAISE(ABORT, 'validations are append-only'); END;

CREATE TRIGGER validations_no_delete BEFORE DELETE ON validations
BEGIN SELECT RAISE(ABORT, 'validations are append-only'); END;

CREATE TRIGGER rule_results_no_update BEFORE UPDATE ON rule_results
BEGIN SELECT RAISE(ABORT, 'rule results are append-only'); END;

CREATE TRIGGER rule_results_no_delete BEFORE DELETE ON rule_results
BEGIN SELECT RAISE(ABORT, 'rule results are append-only'); END;
