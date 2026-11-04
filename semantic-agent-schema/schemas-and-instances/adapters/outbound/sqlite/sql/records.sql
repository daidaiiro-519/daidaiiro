-- 操作の記録の表。誰が打ったか（sessions）・ 書いた変更（changes ・ field_changes）・ 検証（validations ・ rule_results）。
-- どの表も追記だけ（UPDATE ・ DELETE を trigger で拒む）。断った操作は持たない（理由 ・ 案内 ・ 間に入った変更は、その場で呼び出し元に返す）。

-- 打ったセッションは、semantic-agent-schema が初めて見たときに1行（semantic-agent-schema init で登録したときも）。
-- エージェントを起動しないので、セッションどうしのつながりと起動の設定は持たない。セッションは状態を持たない
CREATE TABLE sessions (
    id         TEXT PRIMARY KEY,                          -- セッション ID（--session ・ ツールの環境変数 ・ semantic-agent-schema init で決めたもの）
    tool       TEXT NOT NULL,                             -- claude-code など。分からなければ unknown
    folder     TEXT NOT NULL,                             -- 初めて見たときに打ったフォルダ（作業ツリーの根）
    via        TEXT NOT NULL CHECK (via IN ('env', 'flag', 'init')),  -- ID をどこから知ったか
    first_seen TEXT NOT NULL
) STRICT;

CREATE TRIGGER sessions_no_update BEFORE UPDATE ON sessions
BEGIN SELECT RAISE(ABORT, 'sessions are append-only'); END;

CREATE TRIGGER sessions_no_delete BEFORE DELETE ON sessions
BEGIN SELECT RAISE(ABORT, 'sessions are append-only'); END;

-- 検証。対象を変えないのでハッシュは1つ。ID は changes と同じ採番（o-NNNN）から振る
CREATE TABLE validations (
    id                TEXT PRIMARY KEY,
    at                TEXT NOT NULL,
    target_path       TEXT NOT NULL,
    hash              TEXT NOT NULL,
    session_id        TEXT REFERENCES sessions (id),      -- 打ったセッション。人は NULL
    schema            TEXT                                -- 検証した文書が従う schema の $id
) STRICT;

CREATE INDEX validations_by_target ON validations (target_path, id);

-- rule ごとの結果。reads_hash は判定が読んだ所のハッシュ（入力が変わった rule だけを検証するときに比べる）
CREATE TABLE rule_results (
    validation_id TEXT NOT NULL REFERENCES validations (id),
    ord           INTEGER NOT NULL,
    rule_id       TEXT NOT NULL,
    pointer       TEXT NOT NULL,
    by            TEXT NOT NULL CHECK (by IN ('machine', 'ai')),
    result        TEXT NOT NULL CHECK (result IN ('verified', 'violated', 'unverified', 'error', 'not-judged')),
    reason        TEXT NOT NULL,
    reads_hash    TEXT NOT NULL,
    PRIMARY KEY (validation_id, ord)
) STRICT;

CREATE TRIGGER validations_no_update BEFORE UPDATE ON validations
BEGIN SELECT RAISE(ABORT, 'validations are append-only'); END;

CREATE TRIGGER validations_no_delete BEFORE DELETE ON validations
BEGIN SELECT RAISE(ABORT, 'validations are append-only'); END;

CREATE TRIGGER rule_results_no_update BEFORE UPDATE ON rule_results
BEGIN SELECT RAISE(ABORT, 'rule results are append-only'); END;

CREATE TRIGGER rule_results_no_delete BEFORE DELETE ON rule_results
BEGIN SELECT RAISE(ABORT, 'rule results are append-only'); END;

-- 書いた変更（作成 ・ 更新 ・ 削除）。reason は呼ぶ側が渡した理由（間に入った変更を返すときに後の書き手が読む）。
-- source = outside は Semantic Agent Schema を通らない変更の取り込みで、作成か更新だけ。更新の取り込みには理由が要る。
-- 「取り込みの事前は、その経路の記録のどこかにある事後」と「作成の取り込みは記録の無い対象だけ」は、書く前に application が確かめる。
-- worktree は書いたワークツリー（作業ツリーの根の絶対の経路）で、外の変更は同じワークツリーで記録した最後の事後と比べる
CREATE TABLE changes (
    id                TEXT PRIMARY KEY,
    at                TEXT NOT NULL,
    kind              TEXT NOT NULL CHECK (kind IN ('create', 'update', 'delete')),
    target_path       TEXT NOT NULL,
    before_hash       TEXT,                               -- 作成は無い
    after_hash        TEXT,                               -- 削除は無い
    reason            TEXT,
    source            TEXT NOT NULL DEFAULT 'semantic-agent-schema' CHECK (source IN ('semantic-agent-schema', 'outside')),
    session_id        TEXT REFERENCES sessions (id),      -- 打ったセッション。人は NULL
    schema            TEXT,                               -- 書いた文書が従う schema の $id（削除は消す前の文書のもの）
    worktree          TEXT,
    CHECK ((kind = 'create') = (before_hash IS NULL)),
    CHECK ((kind = 'delete') = (after_hash IS NULL)),
    CHECK (kind <> 'delete' OR reason IS NOT NULL),
    CHECK (source = 'semantic-agent-schema' OR kind <> 'delete'),
    CHECK (source = 'semantic-agent-schema' OR kind <> 'update' OR reason IS NOT NULL)
) STRICT;

CREATE INDEX changes_by_target ON changes (target_path, id);

CREATE INDEX changes_by_session ON changes (session_id, at) WHERE session_id IS NOT NULL;

-- 変わった欄。SQL の NULL は値が無いこと、JSON の null は null の値。
-- 前の値が取れなかった欄（before_unavailable = 1。取り込んだ更新）は replace で、前の値を持たない。
-- 削除は事前の状態を根の欄ごとの remove として残す
CREATE TABLE field_changes (
    change_id          TEXT NOT NULL REFERENCES changes (id),
    pointer            TEXT NOT NULL,
    change             TEXT NOT NULL CHECK (change IN ('add', 'remove', 'replace')),
    before             TEXT CHECK (before IS NULL OR json_valid(before)),
    after              TEXT CHECK (after IS NULL OR json_valid(after)),
    before_unavailable INTEGER NOT NULL DEFAULT 0 CHECK (before_unavailable IN (0, 1)),
    PRIMARY KEY (change_id, pointer),
    CHECK (before_unavailable = 0 OR (change = 'replace' AND before IS NULL)),
    CHECK (before_unavailable = 1 OR (change = 'add') = (before IS NULL)),
    CHECK ((change = 'remove') = (after IS NULL))
) STRICT;

CREATE TRIGGER changes_no_update BEFORE UPDATE ON changes
BEGIN SELECT RAISE(ABORT, 'changes are append-only'); END;

CREATE TRIGGER changes_no_delete BEFORE DELETE ON changes
BEGIN SELECT RAISE(ABORT, 'changes are append-only'); END;

CREATE TRIGGER field_changes_no_update BEFORE UPDATE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field changes are append-only'); END;

CREATE TRIGGER field_changes_no_delete BEFORE DELETE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field changes are append-only'); END;

-- 前の値が取れないのは、取り込んだ更新だけ
CREATE TRIGGER field_changes_unavailable_only_adopted BEFORE INSERT ON field_changes
WHEN NEW.before_unavailable = 1
  AND (SELECT source = 'outside' AND kind = 'update' FROM changes WHERE id = NEW.change_id) IS NOT 1
BEGIN SELECT RAISE(ABORT, 'only an adopted update can lack the previous value'); END;

-- 削除の変わった欄は、事前の根の欄ごとの remove だけ
CREATE TRIGGER field_changes_delete_removes BEFORE INSERT ON field_changes
WHEN NEW.change IS NOT 'remove'
  AND (SELECT kind FROM changes WHERE id = NEW.change_id) = 'delete'
BEGIN SELECT RAISE(ABORT, 'a delete records only removals of the previous state'); END;

-- UC7「操作の記録を読む」：変更と検証を記録した順に並べる
CREATE VIEW operations AS
    SELECT id, at, kind, target_path, before_hash, after_hash, reason, source,
           session_id, schema
    FROM changes
    UNION ALL
    SELECT id, at, 'validate', target_path, hash, hash, NULL, 'semantic-agent-schema',
           session_id, schema
    FROM validations;
