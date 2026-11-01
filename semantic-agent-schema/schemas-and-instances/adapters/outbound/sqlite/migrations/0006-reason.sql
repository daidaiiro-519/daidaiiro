-- 版32：理由を表す欄を reason の1語にそろえる（CLI の --intent を --reason にした）。変更の記録の intent を reason に名前を変える。
-- changes はほかの表の trigger が読むので作り直さず、欄の名前だけを変える（ALTER TABLE … RENAME COLUMN。行と CHECK はそのまま移る）。
-- ビュー operations は行を持たないので作り直す。

DROP VIEW operations;

ALTER TABLE changes RENAME COLUMN intent TO reason;

CREATE VIEW operations AS
    SELECT id, at, kind, target_path, before_hash, after_hash, reason, request_id, source,
           session_id, schema
    FROM changes
    UNION ALL
    SELECT id, at, 'validate', target_path, hash, hash, NULL, NULL, 'sas',
           session_id, schema
    FROM validations;
