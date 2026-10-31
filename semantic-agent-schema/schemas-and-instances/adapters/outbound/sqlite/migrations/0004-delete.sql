-- ステップ4：削除と、削除に伴う参照元の書き換え（changes.cause_id）。ステップ4b で書き直した。
-- 削除は事前の状態を変わった欄（根の欄ごとの remove）として残し、事後の指紋を持たない（0002 の CHECK）。
-- 参照元の書き換えは、削除を原因とする同じ呼び出しの続きの update。原因を持つ書き換えは依頼の ID を持たない（0002 の CHECK）。

-- 原因を持つのは SAS が書いた update だけで、原因は削除
CREATE TRIGGER changes_cause_is_delete BEFORE INSERT ON changes
WHEN NEW.cause_id IS NOT NULL
  AND (NEW.kind IS NOT 'update' OR NEW.source IS NOT 'sas'
       OR (SELECT kind FROM changes WHERE id = NEW.cause_id) IS NOT 'delete')
BEGIN SELECT RAISE(ABORT, 'a cause must be a delete, and only an update made by SAS has one'); END;

-- 削除の変わった欄は、事前の根の欄ごとの remove だけ
CREATE TRIGGER field_changes_delete_removes BEFORE INSERT ON field_changes
WHEN NEW.change IS NOT 'remove'
  AND (SELECT kind FROM changes WHERE id = NEW.change_id) = 'delete'
BEGIN SELECT RAISE(ABORT, 'a delete records only removals of the previous state'); END;

-- UC7「操作の記録を読む」：変更と検証を記録した順に並べる
CREATE VIEW operations AS
    SELECT id, at, kind, target_path, before_hash, after_hash, intent, request_id, source, cause_id,
           parent_session_id, child_session_id
    FROM changes
    UNION ALL
    SELECT id, at, 'validate', target_path, hash, hash, NULL, NULL, 'sas', NULL,
           parent_session_id, child_session_id
    FROM validations;
