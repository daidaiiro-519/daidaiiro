-- ステップ4：削除と、削除に伴う参照元の書き換え（cause_id）。
-- 削除の操作は事前の状態を変わった欄（根の欄ごとの remove）として残し、事後の指紋を持たない。
-- 参照元の書き換えは、削除を原因（cause_id）とする同じセッションの続きの update。
-- 消したあとは記録のつながりが切れるので、取り込みの決まりは、その経路の最後の削除より後の操作だけを見る。

ALTER TABLE operations ADD COLUMN cause_id TEXT REFERENCES operations (id);

-- 原因を持つのは、書いた update だけ。原因は同じセッションの、先に記録した、消した削除
CREATE TRIGGER operations_cause_is_delete BEFORE INSERT ON operations
WHEN NEW.cause_id IS NOT NULL
  AND (NEW.kind IS NOT 'update' OR NEW.refusal_code IS NOT NULL OR NEW.source IS NOT 'sas'
       OR (SELECT kind = 'delete' AND refusal_code IS NULL AND session_id = NEW.session_id
           FROM operations WHERE id = NEW.cause_id) IS NOT 1)
BEGIN SELECT RAISE(ABORT, 'a cause must be a delete written earlier in the same session, and only an update has one'); END;

-- 原因を持つ書き換えには、依頼の ID を付けない（1つの依頼に操作は1回で、それは削除の側）
CREATE TRIGGER operations_cause_no_request BEFORE INSERT ON operations
WHEN NEW.cause_id IS NOT NULL AND NEW.request_id IS NOT NULL
BEGIN SELECT RAISE(ABORT, 'an update caused by a delete carries no request id'); END;

DROP TRIGGER operations_adopt_follows;

-- 更新の取り込みの事前は、その対象を最後に書いた操作の事後（最後の削除より後のものだけ）
CREATE TRIGGER operations_adopt_follows BEFORE INSERT ON operations
WHEN NEW.source = 'outside' AND NEW.kind = 'update' AND NEW.refusal_code IS NULL
  AND NEW.before_hash IS NOT (
    SELECT after_hash FROM operations
    WHERE target_path = NEW.target_path AND kind IN ('create', 'update') AND refusal_code IS NULL
      AND rowid > coalesce((SELECT max(rowid) FROM operations
                            WHERE target_path = NEW.target_path AND kind = 'delete' AND refusal_code IS NULL), 0)
    ORDER BY rowid DESC LIMIT 1)
BEGIN SELECT RAISE(ABORT, 'an adopted update must start from the last recorded state'); END;

DROP TRIGGER operations_adopt_create_first;

-- 作成の取り込みは、まだ書いた記録の無い対象だけ（消したあとは、また記録の無い対象になる）
CREATE TRIGGER operations_adopt_create_first BEFORE INSERT ON operations
WHEN NEW.source = 'outside' AND NEW.kind = 'create' AND NEW.refusal_code IS NULL
  AND EXISTS (
    SELECT 1 FROM operations
    WHERE target_path = NEW.target_path AND kind IN ('create', 'update') AND refusal_code IS NULL
      AND rowid > coalesce((SELECT max(rowid) FROM operations
                            WHERE target_path = NEW.target_path AND kind = 'delete' AND refusal_code IS NULL), 0))
BEGIN SELECT RAISE(ABORT, 'the target is already recorded; adopt it as an update'); END;

DROP TRIGGER field_changes_only_written;

-- 変わった欄は、書いた作成 ・ 更新と、消した削除にだけ付く
CREATE TRIGGER field_changes_only_written BEFORE INSERT ON field_changes
WHEN (SELECT kind IN ('create', 'update', 'delete') AND refusal_code IS NULL FROM operations WHERE id = NEW.operation_id) IS NOT 1
BEGIN SELECT RAISE(ABORT, 'field changes belong to a written create, update or delete'); END;

-- 削除の変わった欄は、事前の根の欄ごとの remove だけ
CREATE TRIGGER field_changes_delete_removes BEFORE INSERT ON field_changes
WHEN NEW.change IS NOT 'remove'
  AND (SELECT kind FROM operations WHERE id = NEW.operation_id) = 'delete'
BEGIN SELECT RAISE(ABORT, 'a delete records only removals of the previous state'); END;
