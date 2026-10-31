-- ステップ2：書いた変更（作成 ・ 更新、ステップ4 から削除）と、変わった欄。ステップ4b で書き直した。
-- 断った操作は持たない（理由 ・ 案内 ・ 間に入った変更は、その場で呼び出し元に返す）。
-- intent は呼ぶ側が渡した意図（間に入った変更を返すときに後の書き手が読む）。
-- source はステップ3 の取り込み（outside）、cause_id はステップ4 の削除に伴う書き換えの原因。どちらも表の形はここで決める。

CREATE TABLE changes (
    id                TEXT PRIMARY KEY,
    at                TEXT NOT NULL,
    kind              TEXT NOT NULL CHECK (kind IN ('create', 'update', 'delete')),
    target_path       TEXT NOT NULL,
    before_hash       TEXT,                               -- 作成は無い
    after_hash        TEXT,                               -- 削除は無い
    intent            TEXT,
    request_id        TEXT,
    source            TEXT NOT NULL DEFAULT 'sas' CHECK (source IN ('sas', 'outside')),
    cause_id          TEXT REFERENCES changes (id),       -- 原因の削除（参照元の書き換え）
    parent_session_id TEXT REFERENCES sessions (id),      -- 呼んだ親。人は NULL
    child_session_id  TEXT REFERENCES sessions (id),      -- 書いた書く子。機械だけなら NULL
    CHECK ((kind = 'create') = (before_hash IS NULL)),
    CHECK ((kind = 'delete') = (after_hash IS NULL)),
    CHECK (kind <> 'delete' OR intent IS NOT NULL),
    CHECK (source = 'sas' OR kind <> 'delete'),
    CHECK (source = 'sas' OR kind <> 'update' OR intent IS NOT NULL),
    CHECK (cause_id IS NULL OR request_id IS NULL)
) STRICT;

-- 1つの依頼に変更は1回（やり直しで同じ変更を2回残さない）
CREATE UNIQUE INDEX changes_one_per_request ON changes (request_id) WHERE request_id IS NOT NULL;

CREATE INDEX changes_by_target ON changes (target_path, id);

CREATE INDEX changes_by_parent ON changes (parent_session_id, at) WHERE parent_session_id IS NOT NULL;

-- 変わった欄。SQL の NULL は値が無いこと、JSON の null は null の値。
-- 前の値が取れなかった欄（before_unavailable = 1、ステップ3 の取り込み）は replace で、前の値を持たない
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

CREATE TRIGGER changes_parent_is_caller BEFORE INSERT ON changes
WHEN NEW.parent_session_id IS NOT NULL AND (SELECT role FROM sessions WHERE id = NEW.parent_session_id) IS NOT 'caller'
BEGIN SELECT RAISE(ABORT, 'parent_session_id must point to a caller session'); END;

CREATE TRIGGER changes_child_is_writer BEFORE INSERT ON changes
WHEN NEW.child_session_id IS NOT NULL AND (SELECT role FROM sessions WHERE id = NEW.child_session_id) IS NOT 'writer'
BEGIN SELECT RAISE(ABORT, 'a change is made by a writer'); END;

CREATE TRIGGER changes_no_update BEFORE UPDATE ON changes
BEGIN SELECT RAISE(ABORT, 'changes are append-only'); END;

CREATE TRIGGER changes_no_delete BEFORE DELETE ON changes
BEGIN SELECT RAISE(ABORT, 'changes are append-only'); END;

CREATE TRIGGER field_changes_no_update BEFORE UPDATE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field changes are append-only'); END;

CREATE TRIGGER field_changes_no_delete BEFORE DELETE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field changes are append-only'); END;
