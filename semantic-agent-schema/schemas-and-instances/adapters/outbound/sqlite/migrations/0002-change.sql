-- ステップ2：作成 ・ 更新の操作と、変わった欄。
-- operations の kind の CHECK を広げるため、表を作り直して行を移す（SQLite は CHECK を ALTER で変えられない）。
-- 移すあいだは外部キーの検査を止める（SqliteLog::with が止め、移したあとに foreign_key_check で確かめる）。
-- 断った操作（refusal_code がある）は対象を変えないので、事前と事後の指紋が同じ。
-- delete（ステップ4）は公開前に kind の CHECK へ足した（表を作り直すと、ほかの表の trigger を付け直すことになるため）。消した操作の事後は無い。
-- intent は呼ぶ側が渡した意図（なぜその操作をしたか）。間に入った変更を返すときに、後の書き手が読む。

CREATE TABLE operations_next (
    id           TEXT PRIMARY KEY,
    session_id   TEXT NOT NULL REFERENCES sessions (id),
    seq          INTEGER NOT NULL CHECK (seq >= 1),
    request_id   TEXT,
    kind         TEXT NOT NULL CHECK (kind IN ('validate', 'create', 'update', 'delete')),
    target_kind  TEXT NOT NULL CHECK (target_kind IN ('concrete', 'instance')),
    target_path  TEXT NOT NULL,
    concrete_ref TEXT,
    before_hash  TEXT,
    after_hash   TEXT,
    refusal_code TEXT,
    refusal      TEXT,
    intent       TEXT,
    at           TEXT NOT NULL,
    UNIQUE (session_id, seq),
    CHECK (kind <> 'validate' OR intent IS NULL),
    CHECK ((refusal_code IS NULL) = (refusal IS NULL)),
    CHECK (kind <> 'validate' OR (before_hash IS after_hash AND refusal_code IS NULL AND request_id IS NULL)),
    CHECK (kind = 'validate' OR concrete_ref IS NOT NULL),
    CHECK (kind <> 'create' OR before_hash IS NULL),
    CHECK (kind <> 'update' OR before_hash IS NOT NULL),
    CHECK (kind <> 'delete' OR before_hash IS NOT NULL),
    CHECK (kind IN ('validate', 'delete') OR refusal_code IS NOT NULL OR after_hash IS NOT NULL),
    CHECK (kind <> 'delete' OR refusal_code IS NOT NULL OR after_hash IS NULL),
    CHECK (refusal_code IS NULL OR before_hash IS after_hash)
) STRICT;

INSERT INTO operations_next (id, session_id, seq, kind, target_kind, target_path, before_hash, after_hash, at)
SELECT id, session_id, seq, kind, target_kind, target_path, before_hash, after_hash, at FROM operations ORDER BY rowid;

DROP TABLE operations;

ALTER TABLE operations_next RENAME TO operations;

-- 1つの依頼に操作は1回（やり直しで同じ操作を2回残さない）
CREATE UNIQUE INDEX operations_one_per_request ON operations (request_id) WHERE request_id IS NOT NULL;

CREATE INDEX operations_by_target ON operations (target_path, id);

CREATE TRIGGER operations_no_update BEFORE UPDATE ON operations
BEGIN SELECT RAISE(ABORT, 'operations are append-only'); END;

CREATE TRIGGER operations_no_delete BEFORE DELETE ON operations
BEGIN SELECT RAISE(ABORT, 'operations are append-only'); END;

CREATE TRIGGER operations_open_session BEFORE INSERT ON operations
WHEN (SELECT ended_at FROM sessions WHERE id = NEW.session_id) IS NOT NULL
BEGIN SELECT RAISE(ABORT, 'the session is closed'); END;

-- seq はセッションの中で、前の操作の次の番号
CREATE TRIGGER operations_seq_follows BEFORE INSERT ON operations
WHEN NEW.seq IS NOT (SELECT coalesce(max(seq), 0) + 1 FROM operations WHERE session_id = NEW.session_id)
BEGIN SELECT RAISE(ABORT, 'seq must follow the previous operation of the session'); END;

-- 判定は検証の操作にだけ付く
CREATE TRIGGER rule_results_only_validate BEFORE INSERT ON rule_results
WHEN (SELECT kind FROM operations WHERE id = NEW.operation_id) IS NOT 'validate'
BEGIN SELECT RAISE(ABORT, 'rule results belong to a validate operation'); END;

-- 変わった欄。SQL の NULL は値が無いこと、JSON の null は null の値
CREATE TABLE field_changes (
    operation_id TEXT NOT NULL REFERENCES operations (id),
    pointer      TEXT NOT NULL,
    change       TEXT NOT NULL CHECK (change IN ('add', 'remove', 'replace')),
    before       TEXT CHECK (before IS NULL OR json_valid(before)),
    after        TEXT CHECK (after IS NULL OR json_valid(after)),
    PRIMARY KEY (operation_id, pointer),
    CHECK ((change = 'add') = (before IS NULL)),
    CHECK ((change = 'remove') = (after IS NULL))
) STRICT;

-- 変わった欄は、書いた作成 ・ 更新の操作にだけ付く
CREATE TRIGGER field_changes_only_written BEFORE INSERT ON field_changes
WHEN (SELECT kind IN ('create', 'update') AND refusal_code IS NULL FROM operations WHERE id = NEW.operation_id) IS NOT 1
BEGIN SELECT RAISE(ABORT, 'field changes belong to a written create or update'); END;

CREATE TRIGGER field_changes_no_update BEFORE UPDATE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field changes are append-only'); END;

CREATE TRIGGER field_changes_no_delete BEFORE DELETE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field changes are append-only'); END;
