-- ステップ3：SAS を通らない変更の取り込み（changes.source = outside）。ステップ4b で書き直した。
-- 「取り込みは作成か更新だけ」「更新の取り込みには意図が要る」は changes の CHECK（0002）。
-- 「取り込みの事前は、その経路の記録のどこかにある事後」と「作成の取り込みは記録の無い対象だけ」は、
-- 書く前に application が確かめる（DB はワークツリーで1つなので、最後の事後だけをつながりにしない）。

-- 前の値が取れないのは、取り込んだ更新だけ
CREATE TRIGGER field_changes_unavailable_only_adopted BEFORE INSERT ON field_changes
WHEN NEW.before_unavailable = 1
  AND (SELECT source = 'outside' AND kind = 'update' FROM changes WHERE id = NEW.change_id) IS NOT 1
BEGIN SELECT RAISE(ABORT, 'only an adopted update can lack the previous value'); END;
