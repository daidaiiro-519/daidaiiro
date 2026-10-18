---
name: semantic-contract-core-board
description: 基盤 Semantic Schema Core（Semantic Abstract Schema と Semantic Concrete Schema。いまの schema-driven を改める）を決めるボード semantic-contract-core の現在地
metadata:
  type: project
---

## 現在地（2026-10-18、第71回の反証まで）

- ドメインモデルを使う場面（Q8）と業務領域（Q9）から導き直した案を Q1 に入れた：境界 ・ 拡張の契約 ・ 集約5つ（Semantic Abstract Schema ・ Semantic Concrete Schema ・ instance ・ 判定の記録 ・ 変更の記録）・ ドメインサービス12 ・ 駆動する側と駆動される側のポート ・ 不変条件39行。正本は measured/domain-model-v2.json。
- 利用者の決定：Concrete は x- を拡張できる（Core が拡張の仕方を言語に依らない契約で決める）。x- は JSON Schema で検証できない部分のためにある。判定の記録は判定した値を残す（ハッシュにしない）。外のツールは tool permissions に外出し。ほかの文脈へ見せる部分だけ版を持つ。Q9 の中核は「変更の確かめと例外の判断」。
- 変更の記録を3人の反証の統合で足した（U25 変更を確かめて取り込むかを決める、I31〜I37）。置き場所はリポジトリのファイル（id で引く。第70回）。第71回：セッションを管理する役は外、セッション ID は手元の索引（SQLite、コミットしない）だけ、記録は書いた時に ID 無しで置く。まとまりは保存せず出すときに組み立てる・理由の合致は既存の AI の判定で確かめる（利用者の決定）。Q10 の「理由は変更の仕組みに任せる」を改めた。
- 回答待ち：Q1（いま見る論点）。Q10 は Q1 待ち。D6 は決着（unsupported で人が見る。人は Core から tool permissions に登録できる）。
- 実行できるモデル measured/model/ は第66回の形のままで、案と食い違う。作り直す。

## 次にすること

- Q1 の回答を受け取り、承認なら advance して Q7 を開く。
- brainstorming の update は --patch_file でファイルから受ける（ACDR 0191 承認済み）。長い図はボードのフォルダにパッチを書いて渡す。
- 実装の名前（schema-driven の CLI ・ MCP）を Semantic Schema Core に改めるのは、別の ACDR で決める。
