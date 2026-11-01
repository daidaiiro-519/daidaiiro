-- ステップ1：誰が何で動いたか（sessions）と、検証（validations）と rule ごとの結果（rule_results）。
-- ステップ4b で書き直した（公開前なので移行の番号は足さない）。セッションは状態を持たない：開閉 ・ 終わり方 ・ 続き番号は無い。
-- どの表も追記だけ（UPDATE ・ DELETE を trigger で拒む）。

-- sas を打ったセッションは、SAS が初めて見たときに1行（sas init で登録したときも）。SAS はエージェントを起動しないので、
-- セッションどうしのつながりと起動の設定は持たない（概念のページ版28）
CREATE TABLE sessions (
    id         TEXT PRIMARY KEY,                          -- セッション ID（--session ・ ツールの環境変数 ・ sas init で SAS が決めたもの）
    tool       TEXT NOT NULL,                             -- claude-code など。分からなければ unknown
    folder     TEXT NOT NULL,                             -- 初めて見たときに打ったフォルダ（Git の中ならリポジトリの根、外なら作業フォルダ）
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
    session_id        TEXT REFERENCES sessions (id)       -- 打ったセッション。人は NULL
) STRICT;

CREATE INDEX validations_by_target ON validations (target_path, id);

-- rule ごとの結果。reads_hash は判定が読んだ所のハッシュ（入力が変わった rule だけを検証するときに比べる）
CREATE TABLE rule_results (
    validation_id TEXT NOT NULL REFERENCES validations (id),
    ord           INTEGER NOT NULL,
    rule_id       TEXT NOT NULL,
    pointer       TEXT NOT NULL,
    by            TEXT NOT NULL CHECK (by IN ('machine', 'ai')),
    result        TEXT NOT NULL CHECK (result IN ('verified', 'violated', 'unverified', 'error', 'unsupported')),
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
