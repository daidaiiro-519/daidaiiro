-- ACDR 0224：外の変更をワークツリーごとに検出する。記録はワークツリーで1つのまま、書いた変更に、書いたワークツリー（作業ツリーの根の
-- 絶対の経路。git rev-parse --show-toplevel）を足す。外の変更は、同じワークツリーで記録した最後の事後と比べる。
-- changes はほかの表の trigger が読むので作り直さず、欄を足す（CODING.md）。足す前の行は NULL（どのワークツリーのものでもない）。

ALTER TABLE changes ADD COLUMN worktree TEXT;
