# Semantic Agent Schema

JSON Schema を Semantic に拡張し、その型（Abstract → Concrete → インスタンス）で作成・更新・削除・検証・描画・検索する基盤。

本質（指針がぶれたときに立ち返る所）は `docs/essence.md` にある。

| 置き場所 | 何か |
|---|---|
| このフォルダ | 製品 Semantic Agent Schema のリポジトリ。区切られた文脈ではない（事業領域は「AI のエージェントが扱う構造化データを、スキーマに従って書き、検証すること」） |
| `schemas-and-instances/` | 区切られた文脈「スキーマとインスタンス」。中の作り方はポートとアダプター |
| `schemas-and-instances/published-language/v1/` | 公開された言葉（使う側が従い、読む通信規約）の9つ：Concrete を書く型（abstract ・ vocabulary ・ dialect）・ CLI の出力（command-result）・ 打ったセッションへの手順（procedure）と、そのセッションが --answer で渡す答えの形（judgment-answer ・ search-answer）・ 受け取りの型（design-system ・ template-language） |
| `schemas-and-instances/writing/` ・ `validation/` ・ `search/` ・ `recording/` ・ `rendering/` | writing と validation は業務領域「書いて検証する」（中核）を作る2つのモジュール（domain と application）。search は業務領域「検索」、recording は業務領域「記録」、rendering は業務領域「描画」（どれも補完）のモジュール（application だけ）。集約は writing/domain |
| `schemas-and-instances/adapters/` | 入口（inbound/cli、semantic-agent-schema）と外の技術（outbound/filesystem ・ git ・ jsonschema ・ sqlite ・ templates ・ javascript） |
| `schemas-and-instances/system-tests/` | システムテストの場面（`scenarios/`）と材料（`fixtures/`）、保証 ・ 拡張との突き合わせ（`trace.json` ・ `untested.json`） |
| `docs/abstract-modeling/` | 抽象モデリング概念。技術に依らない「何を ・ なぜ」：本質 ・ 概念モデル ・ 入力の契約 ・ ユースケース ・ ドメインモデル |
| `docs/concrete-modeling/` | 具体モデリング概念。どう作るか：構成 ・ ポートとアダプター ・ コマンドと引数 ・ 公開された言葉 ・ 記録 ・ 処理の流れ |
| `docs/design/` | Semantic Agent Schema に適用した設計。仮説と、探索の記録 |

作り方は、ユースケースから探索してドメインモデルと構成を育てる進め方による。ステップごとの記録は `docs/design/exploration/` にある。

## コマンド

入力を作るのは AI で、Semantic Agent Schema は入力の形と値を確かめて決まった手順で処理する。足りない入力は補わずに断る。

| コマンド | すること |
|---|---|
| `validate <経路>... --schemas <置き場所>...` | UC4 確かめる。形の検査と組み込みの検査をし、検証を記録する。`--stale` は前の判定が読んだ所が変わった rule だけ、`--semantic` は意味の判定の手順を返し、`--answer <ファイル \| ->` で判定（JSON だけ）を渡す。デザインシステムの経路を渡すと、受け取りの型とテンプレートの書き方の契約に照らす |
| `create <経路> --schemas <置き場所>... --concrete <$id> --value <JSON>` | UC1 作る。`--semantic` は書く手順を返し、`--adopt` は記録に無い既存のファイルを取り込む |
| `update <経路> --schemas <置き場所>... --patch <JSON Patch>` | UC2 直す。`--semantic --task` は書く手順を返し、`--adopt --reason`（`--base <コミット>`）は外で直したファイルを取り込む。Concrete を直すときは `--range <経路>...` を必ず渡す |
| `delete <経路> --reason <理由>` | UC3 消す。Concrete を消すときは `--range <経路>...` を必ず渡す |
| `render <経路>... --schemas <置き場所>... --design-system <経路> --out <書き出す先>` | UC5 描く。使う側が渡すデザインシステムのテンプレートに値を置く。記録しない |
| `search <経路>... --schemas <置き場所>...` | UC6 探す。`--where <JSON Pointer>=<JSON>` で構造で絞り、`--semantic --question` で意味で探す手順を返し、`--answer` で答え（JSON だけ）の出どころを確かめる。記録しない |
| `log [<経路>...]` ・ `log --operation <ID>` | UC7 記録を読む。読むだけで何も変えない |
| `init` | 新しいセッションの ID を作って登録し、返す。あとで打つコマンドの既定にはならない |

| 受け取り方 | 決まり |
|---|---|
| 経路 | どこで打っても Git の根からの相対。読むのは渡された経路だけで、フォルダを歩かない |
| `--schemas` | ファイルか、直下の JSON のファイルを読むフォルダ。Concrete と `$ref` の先は、この中から `$id` で引く。同じ `$id` が2つ以上なら全部の経路を添えて断る |
| `--range` | Concrete を直す ・ 消すときの範囲。渡された範囲と記録で従うインスタンスを確かめ、見た文書を `checked` で返す |
| 誰が打ったか | AI は `--session <ID>`（Claude Code などツールの環境変数があれば省ける。Gemini CLI ・ Cursor は毎回付ける。ID は `init` で作れる）、人は `--human`。ツールの環境変数があるところでは `--human` を受け取らない。どれも無ければ断る |
| 返すコマンド | 手順の `command` と案内の `hint` は、受け取った `--schemas` ・ `--range` と、誰が打ったかを引き継ぐ。`<…>` の所だけを埋めて打つ |
| 記録 | `<git-common-dir>/semantic-agent-schema/records.sqlite`（`SEMANTIC_AGENT_SCHEMA_DATABASE_FILE` で指せる）。追記だけで、断った操作は記録しない。Git のリポジトリの外では断る |

Semantic Agent Schema はエージェントを起動しない。意味の部分は、x-prompt から決まった形の手順（`published-language/v1/procedure.schema.json`）を返し、打ったセッションが判定 ・ 生成した値を機械の検証で確かめる。
