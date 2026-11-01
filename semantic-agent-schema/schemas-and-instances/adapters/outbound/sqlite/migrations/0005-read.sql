-- ステップ9：記録を読む（UC7）。操作の1件を、書いたとき ・ 検証したときと同じ形で返すために、従う schema の $id を足す。
-- changes ・ sessions はほかの表の trigger が読むので作り直さず、欄を足す（CODING.md）。足す前の行は NULL。
-- schema は、変更なら書いた文書が従う schema（削除は消す前の文書のもの）、検証なら検証した文書が従う schema。

ALTER TABLE changes ADD COLUMN schema TEXT;

ALTER TABLE validations ADD COLUMN schema TEXT;

-- 変更と検証を並べて読むビューに schema を足す（ビューは行を持たないので作り直す）
DROP VIEW operations;

CREATE VIEW operations AS
    SELECT id, at, kind, target_path, before_hash, after_hash, intent, request_id, source,
           session_id, schema
    FROM changes
    UNION ALL
    SELECT id, at, 'validate', target_path, hash, hash, NULL, NULL, 'sas',
           session_id, schema
    FROM validations;
