-- ステップ4：削除。ステップ4b で書き直した。
-- 削除は事前の状態を変わった欄（根の欄ごとの remove）として残し、事後のハッシュを持たない（0002 の CHECK）。

-- 削除の変わった欄は、事前の根の欄ごとの remove だけ
CREATE TRIGGER field_changes_delete_removes BEFORE INSERT ON field_changes
WHEN NEW.change IS NOT 'remove'
  AND (SELECT kind FROM changes WHERE id = NEW.change_id) = 'delete'
BEGIN SELECT RAISE(ABORT, 'a delete records only removals of the previous state'); END;

-- UC7「操作の記録を読む」：変更と検証を記録した順に並べる
CREATE VIEW operations AS
    SELECT id, at, kind, target_path, before_hash, after_hash, intent, request_id, source,
           session_id
    FROM changes
    UNION ALL
    SELECT id, at, 'validate', target_path, hash, hash, NULL, NULL, 'sas',
           session_id
    FROM validations;
