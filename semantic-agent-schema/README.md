# Semantic Agent Schema

JSON Schema を Semantic に拡張し、その型（Abstract → Concrete → インスタンス）で作成・更新・削除・検証・描画・検索する基盤。

| 置き場所 | 何か |
|---|---|
| このフォルダ | 事業領域 Semantic Agent Schema。区切られた文脈ではない |
| `contracts-and-instances/` | 区切られた文脈「契約とインスタンス」。1つのヘキサゴナル |
| `docs/foundation/` | 土台。どの事業にも使う一般の概念 |
| `docs/design/` | Semantic Agent Schema に適用した設計。仮説と、探索の記録 |
| `test-suite/` | システムテストの場面（`scenarios/`）と材料（`fixtures/`）、保証 ・ 拡張との突き合わせ（`trace.json` ・ `untested.json`） |

作り方は、ユースケースから探索してドメインモデルと構成を育てる進め方による。いまできるコマンドは `sas validate [--session <名前>] (<経路>... | --all)`（UC4 の機械の rule）・ `sas create <経路> --concrete <URN> --value <JSON>`（UC1）・ `sas update <経路> --patch <JSON Patch> [--if-match <指紋>]`（UC2）。create と update は `--intent` ・ `--session` ・ `--request-id` も受け取る。update は、--if-match を省くと SAS が最後に書いた事後の指紋といまの中身を比べ、違えば書かずに間に入った変更を返す。ステップごとの記録は `docs/design/exploration/` にある。前の実装は `archive/semantic-agent-schema-2026-10-31/`（タグ `semantic-agent-schema-archive-2026-10-31`）に残してある。
