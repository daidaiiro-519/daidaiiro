---
name: semantic-contract-core-board
description: 基盤 Semantic Schema Core（Semantic Abstract Schema と Semantic Concrete Schema。いまの schema-driven を改める）を決めるボード semantic-contract-core の現在地
metadata:
  type: project
---

## 現在地（2026-10-19、第93回）

- ドメインモデルを使う場面（Q8）と業務領域（Q9）から導き直した案を Q1 に入れた：境界 ・ 拡張の契約 ・ 集約5つ（Semantic Abstract Schema ・ Semantic Concrete Schema ・ instance ・ 判定の記録 ・ 変更の記録）・ ドメインサービス12 ・ 駆動する側と駆動される側のポート ・ 不変条件43行。正本は measured/domain-model-v2.json。
- 利用者の決定：Concrete は x- を拡張できる（Core が拡張の仕方を言語に依らない契約で決める）。x- は JSON Schema で検証できない部分のためにある。判定の記録は判定した値を残す（ハッシュにしない）。外のツールは tool permissions に外出し。ほかの文脈へ見せる部分だけ版を持つ。Q9 の中核は「変更の確かめと例外の判断」。
- 変更の記録を3人の反証の統合で足した（U25 変更を確かめて取り込むかを決める、I31〜I37）。置き場所はリポジトリのファイル（id で引く。第70回）。第71回：セッションを管理する役は外、セッション ID は手元の索引（SQLite、コミットしない）だけ、記録は書いた時に ID 無しで置く。まとまりは保存せず出すときに組み立てる・理由の合致は既存の AI の判定で確かめる（利用者の決定）。Q10 の「理由は変更の仕組みに任せる」を改めた。
- 第77回：tool permissions を外した（Core は許可の境界を持たない。AI の許可は各 AI ツールの設定ファイル）。brainstorming の agents.json を廃止する ACDR が次。
- 第93回：x- を利用者と9つに決めた（x-instance-path ・ x-instance-label ・ x-delegates ・ x-extensions ・ x-prompt ・ x-relationship ・ x-derived-from ・ x-decided-by ・ x-published）。URN は urn:semantic-schema-core:<持ち主>:<種類>:v<版>。Q1 は任せる先 ・ 決定権に改めた。
- Q7 は 10-19、Q10 は 10-20 に承認（変更のデータモデル：目的で承認し、意図ごとの意味の変更で確かめる。意味の変更は I33 の判定の記録）。利用者：「少し詰めないといけない部分はあるかも」。Q10 は Q1 待ち。D6 は決着（unsupported で人が見る。人は Core から tool permissions に登録できる）。
- 実行できるモデル measured/model/ は第66回の形のままで、案と食い違う。作り直す。

## 次にすること

- Q1 の回答を受け取り、承認なら advance して Q7 を開く。
- brainstorming の update は --patch_file でファイルから受ける（ACDR 0191 承認済み）。長い図はボードのフォルダにパッチを書いて渡す。
- 実装の名前（schema-driven の CLI ・ MCP）を Semantic Schema Core に改めるのは、別の ACDR で決める。
