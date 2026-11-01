# Semantic Agent Schema

JSON Schema を Semantic に拡張し、その型（Abstract → Concrete → インスタンス）で作成・更新・削除・検証・描画・検索する基盤。

| 置き場所 | 何か |
|---|---|
| このフォルダ | 製品 Semantic Agent Schema のリポジトリ。区切られた文脈ではない（事業領域は「AI のエージェントが扱う構造化データを、スキーマに従って書き、検証すること」） |
| `schemas-and-instances/` | 区切られた文脈「スキーマとインスタンス」。中の作り方はポートとアダプター |
| `schemas-and-instances/published-language/v1/` | 公開された言葉（使う側が従い、読む通信規約）。Abstract の本体（abstract.schema.json）・ 語彙 ・ 方言と、CLI の出力（operation-result）。操作の依頼も作るステップでここに置く |
| `schemas-and-instances/writing/` ・ `validation/` ・ `recording/` ・ `rendering/` | writing と validation は業務領域「書いて検証する」（中核）を作る2つのモジュール（domain と application）。recording は業務領域「記録」、rendering は業務領域「描画」（どちらも補完）のモジュール（application だけ）。集約は writing/domain |
| `schemas-and-instances/adapters/` | 入口（inbound/cli、sas）と外の技術（outbound/filesystem ・ git ・ jsonschema ・ sqlite ・ templates ・ javascript） |
| `schemas-and-instances/system-tests/` | システムテストの場面（`scenarios/`）と材料（`fixtures/`）、保証 ・ 拡張との突き合わせ（`trace.json` ・ `untested.json`） |
| `docs/foundation/` | 土台。どの事業にも使う共通の概念 |
| `docs/design/` | Semantic Agent Schema に適用した設計。仮説と、探索の記録 |

作り方は、ユースケースから探索してドメインモデルと構成を育てる進め方による。いまできるコマンドは `sas validate [--session <名前>] [--stale] (<経路>... | --all)`（UC4 の機械の rule）・ `sas create <経路> --concrete <URN> --value <JSON>`（UC1）・ `sas update <経路> --patch <JSON Patch> [--if-match <指紋>]`（UC2）。create と update は `--intent` ・ `--session` ・ `--request-id` も受け取る。`sas delete <経路> --intent <意図>`（UC3）は、参照元の記載も一緒に消す。update と delete は、--if-match を省くと、いまの中身が SAS の記録のどれかの事後かを確かめ、違えば書かずに間に入った変更を返す。呼んだ親のセッション ID は `--session` か、AI のツールの環境変数（`adapters/inbound/cli/callers.json`）で決まり、どちらも無ければ人とみなす。記録は `<git-common-dir>/sas/sas.db`（ワークツリーで1つ。コミットしない）に置き、断った操作は記録しない。SAS を通らない変更は、`sas create <経路> --adopt`（記録に無い既存のファイル）と `sas update <経路> --adopt --intent <意図>`（SAS が書いたあとに外で直されたファイル）で、ファイルを書かずに記録へ取り込む。`sas validate --stale` は、前の判定が読んだ所が変わった rule だけを判定し直す。`sas render <経路>... --design-system <経路> --out <書き出す先>`（UC5）は、使う側が渡すデザインシステムのテンプレート（Mustache。Concrete の根の x-view で選ぶ。無ければ page）に、インスタンスの値（value ・ json）とデザイントークン（tokens ・ css）を入れて描く。テンプレートが表現するロジックと UI ・ UX はテンプレートの JS が持ち、SAS は持たない。テンプレートの書き方は published-language/v1/template-language.json が契約で、`sas validate <デザインシステム>` で型 ・ トークン ・ テンプレート ・ JS を契約に照らせる（テンプレートが置く名前と、JS が値の入口 sas-data から読む欄が、Concrete にあるか）。外れたデザインシステムでは描かない。同じ入力からは同じページになり、描いたページは記録もコミットもしない。ステップごとの記録は `docs/design/exploration/` にある。前の実装は `archive/semantic-agent-schema-2026-10-31/`（タグ `semantic-agent-schema-archive-2026-10-31`）に残してある。
