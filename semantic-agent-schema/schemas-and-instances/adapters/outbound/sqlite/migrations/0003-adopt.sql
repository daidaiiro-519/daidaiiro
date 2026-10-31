-- ステップ3：SAS を通らない変更の取り込み（source = outside）・ 前の値が取れなかった欄 ・ 判定が読んだ所の指紋。
-- operations は欄を足すだけで作り直さない（ほかの表の trigger が operations を読むので、作り直すと付け直しが要る）。
-- 欄をまたぐ決まりは CHECK に書けないので trigger で拒む。
-- field_changes は CHECK を変えるので作り直す（ほかの表から指されていない）。

ALTER TABLE operations ADD COLUMN source TEXT NOT NULL DEFAULT 'sas' CHECK (source IN ('sas', 'outside'));

-- 取り込みは作成か更新だけ
CREATE TRIGGER operations_outside_is_change BEFORE INSERT ON operations
WHEN NEW.source = 'outside' AND NEW.kind NOT IN ('create', 'update')
BEGIN SELECT RAISE(ABORT, 'only create and update can adopt a change made outside SAS'); END;

-- 更新の取り込みには意図が要る（あとで読む人が、なぜ外で直したかを追えるように）
CREATE TRIGGER operations_adopt_has_intent BEFORE INSERT ON operations
WHEN NEW.source = 'outside' AND NEW.kind = 'update' AND NEW.refusal_code IS NULL AND NEW.intent IS NULL
BEGIN SELECT RAISE(ABORT, 'adopting an update needs an intent'); END;

-- 更新の取り込みの事前は、その対象を最後に書いた操作の事後（記録の指紋のつながりを切らない）
CREATE TRIGGER operations_adopt_follows BEFORE INSERT ON operations
WHEN NEW.source = 'outside' AND NEW.kind = 'update' AND NEW.refusal_code IS NULL
  AND NEW.before_hash IS NOT (
    SELECT after_hash FROM operations
    WHERE target_path = NEW.target_path AND kind IN ('create', 'update') AND refusal_code IS NULL
    ORDER BY rowid DESC LIMIT 1)
BEGIN SELECT RAISE(ABORT, 'an adopted update must start from the last recorded state'); END;

-- 作成の取り込みは、まだ書いた記録の無い対象だけ
CREATE TRIGGER operations_adopt_create_first BEFORE INSERT ON operations
WHEN NEW.source = 'outside' AND NEW.kind = 'create' AND NEW.refusal_code IS NULL
  AND EXISTS (
    SELECT 1 FROM operations
    WHERE target_path = NEW.target_path AND kind IN ('create', 'update') AND refusal_code IS NULL)
BEGIN SELECT RAISE(ABORT, 'the target is already recorded; adopt it as an update'); END;

-- 変わった欄。前の値が取れなかった欄（before_unavailable = 1）は replace で、前の値を持たない
CREATE TABLE field_changes_next (
    operation_id       TEXT NOT NULL REFERENCES operations (id),
    pointer            TEXT NOT NULL,
    change             TEXT NOT NULL CHECK (change IN ('add', 'remove', 'replace')),
    before             TEXT CHECK (before IS NULL OR json_valid(before)),
    after              TEXT CHECK (after IS NULL OR json_valid(after)),
    before_unavailable INTEGER NOT NULL DEFAULT 0 CHECK (before_unavailable IN (0, 1)),
    PRIMARY KEY (operation_id, pointer),
    CHECK (before_unavailable = 0 OR (change = 'replace' AND before IS NULL)),
    CHECK (before_unavailable = 1 OR (change = 'add') = (before IS NULL)),
    CHECK ((change = 'remove') = (after IS NULL))
) STRICT;

INSERT INTO field_changes_next (operation_id, pointer, change, before, after)
SELECT operation_id, pointer, change, before, after FROM field_changes ORDER BY rowid;

DROP TABLE field_changes;

ALTER TABLE field_changes_next RENAME TO field_changes;

CREATE TRIGGER field_changes_only_written BEFORE INSERT ON field_changes
WHEN (SELECT kind IN ('create', 'update') AND refusal_code IS NULL FROM operations WHERE id = NEW.operation_id) IS NOT 1
BEGIN SELECT RAISE(ABORT, 'field changes belong to a written create or update'); END;

-- 前の値が取れないのは、取り込んだ更新だけ
CREATE TRIGGER field_changes_unavailable_only_adopted BEFORE INSERT ON field_changes
WHEN NEW.before_unavailable = 1
  AND (SELECT source = 'outside' AND kind = 'update' FROM operations WHERE id = NEW.operation_id) IS NOT 1
BEGIN SELECT RAISE(ABORT, 'only an adopted update can lack the previous value'); END;

CREATE TRIGGER field_changes_no_update BEFORE UPDATE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field changes are append-only'); END;

CREATE TRIGGER field_changes_no_delete BEFORE DELETE ON field_changes
BEGIN SELECT RAISE(ABORT, 'field changes are append-only'); END;

-- 判定が読んだ所の指紋（入力が変わった rule だけを検証するときに比べる）。ステップ2 までの行は NULL で、入力が変わったとみなす
ALTER TABLE rule_results ADD COLUMN reads_hash TEXT;
