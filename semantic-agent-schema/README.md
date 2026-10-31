# Semantic Agent Schema

JSON Schema を Semantic に拡張し、その型（Abstract → Concrete → インスタンス）で作成・更新・削除・検証・描画・検索する基盤。

| 置き場所 | 何か |
|---|---|
| このフォルダ | 事業領域 Semantic Agent Schema。区切られた文脈ではない |
| `contracts-and-instances/` | 区切られた文脈「契約とインスタンス」。1つのヘキサゴナル |
| `docs/foundation/` | 土台。どの事業にも使う一般の概念 |
| `docs/design/` | Semantic Agent Schema に適用した設計。仮説と、探索の記録 |
| `test-suite/` | システムテストの場面の形 |

作り方は、ユースケースから探索してドメインモデルと構成を育てる進め方による。ステップごとの記録は `docs/design/exploration/` にある。前の実装は `archive/semantic-agent-schema-2026-10-31/`（タグ `semantic-agent-schema-archive-2026-10-31`）に残してある。
