# Semantic Agent Schema

JSON Schema を Semantic に拡張し、その型（Abstract → Concrete → インスタンス）で作成・更新・削除・検証・描画・検索する基盤。

| 置き場所 | 何か |
|---|---|
| このフォルダ | 事業領域 Semantic Agent Schema。区切られた文脈ではない |
| `schemas-and-instances/` | 区切られた文脈「スキーマとインスタンス」。1つのヘキサゴナル |
| `docs/foundation/` | 土台。どの事業にも使う一般の概念 |
| `docs/design/` | Semantic Agent Schema に適用した設計。仮説と、探索の記録 |
| `test-suite/` | システムテストの場面（`scenarios/`）と材料（`fixtures/`）、保証 ・ 拡張との突き合わせ（`trace.json` ・ `untested.json`） |

作り方は、ユースケースから探索してドメインモデルと構成を育てる進め方による。いまできるコマンドは `sas validate [--session <名前>] [--stale] (<経路>... | --all)`（UC4 の機械の rule）・ `sas create <経路> --concrete <URN> --value <JSON>`（UC1）・ `sas update <経路> --patch <JSON Patch> [--if-match <指紋>]`（UC2）。create と update は `--intent` ・ `--session` ・ `--request-id` も受け取る。`sas delete <経路> --intent <意図>`（UC3）は、参照元の記載も一緒に消す。update と delete は、--if-match を省くと、いまの中身が SAS の記録のどれかの事後かを確かめ、違えば書かずに間に入った変更を返す。呼んだ親のセッション ID は `--session` か、AI のツールの環境変数（`callers.json`）で決まり、どちらも無ければ人とみなす。記録は `<git-common-dir>/sas/sas.db`（ワークツリーで1つ。コミットしない）に置き、断った操作は記録しない。SAS を通らない変更は、`sas create <経路> --adopt`（記録に無い既存のファイル）と `sas update <経路> --adopt --intent <意図>`（SAS が書いたあとに外で直されたファイル）で、ファイルを書かずに記録へ取り込む。`sas validate --stale` は、前の判定が読んだ所が変わった rule だけを判定し直す。ステップごとの記録は `docs/design/exploration/` にある。前の実装は `archive/semantic-agent-schema-2026-10-31/`（タグ `semantic-agent-schema-archive-2026-10-31`）に残してある。
