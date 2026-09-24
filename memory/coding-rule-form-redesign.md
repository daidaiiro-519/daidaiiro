---
name: coding-rule-form-redesign
description: CodingSkills のブレストボードは .brainstorming-board/coding-skills/ の1枚だけである。11論点のうち、いま開いているのは10と11
metadata:
  node_type: memory
  type: project
---

CodingSkills のブレストボードは `.brainstorming-board/coding-skills/` の1枚だけである。
2026-09-21 に、規則1件の形を対象にしていた `coding-rule-form/` を統合した。
以前の `.brainstorm/coding-essence/` は破棄した。

- 論点1〜8 ── 規則1件の形（保持する3つ ・ 出典に無いことを書かない ・ 腐敗の機構 ・
  原典の降下 ・ 配置の宣言 ・ 軸は4本 ・ 軸の順序 ・ 出典を持てないものの置き場所）
- 論点9 ── 決着。軸と値の対を受け取り、規則の集合を返す。コードは検査しない
- 論点10・11 ── 承認待ち（実体で持つもの ／ 検証方法をいつ実行するか）

**次にすること**

- 軸の値1つ（`go`）で規則を1件書き、meta と宣言を指して照合を動かす ── 論点10・11 の試験になる
- ヘキサゴナルの原典を取得して照合する ── 論点4 がそこに乗っている
- テストの配置が論点8 の3分割に収まらない件を、論点8 へ差し戻す
