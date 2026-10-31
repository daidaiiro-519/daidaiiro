# コードの決まり

設計の仮説は `docs/design/arch/arch-v15.json`、根拠の原文の照合は `docs/design/rust/factcheck-rust.json` にある。ここには、いま守る決まりだけを書く。

## 言語に依存しない決まり

| 決まり | 理由 |
|---|---|
| リポジトリの根は事業領域。区切られた文脈は、根の直下の文脈の名前のフォルダ（いまは `contracts-and-instances/`） | 事業領域と文脈を同じものとして扱わない |
| 文脈全体を1つのヘキサゴナルにする。外とのやりとりは application の ports を adapters/outbound が実装する | 外の技術を業務ロジックから切り離す |
| 業務ロジックは、まず use_cases にトランザクションスクリプトで書く。同じ理由で変わるものだけを domain へ引き上げる。自明なものは最初から domain に置いてよいが、根拠を探索の記録に書く | ドメインモデルは育てるもの |
| domain の中は部品の種類で分ける（aggregates/・values/・services/）。購読する手続きが無いうちは events/ を作らない | 先取りしない |
| ユースケースごとのファイルやフォルダは作らない | 土台の境界6 |
| crate・モジュール・ポートは、必要と分かった時点で足す。足したら `architecture.json` にも足す | 最初に構造を全部作らない |
| ステップごとに `docs/design/exploration/step-NN.json` を書く | 探索の過程と、仮説との突き合わせを残す |

## Rust の決まり

| 決まり | 一次情報 |
|---|---|
| 根の Cargo.toml は virtual manifest。`[workspace]` に members と `resolver = "3"` | Cargo workspaces:241・258、Book ch14-03:207〜211 |
| 外のライブラリの版は `[workspace.dependencies]` に置き、各 crate は `workspace = true` で継承する | Cargo workspaces:359・363 |
| 依存の向きを強制したい境界と、外のライブラリを閉じ込めたい所だけを crate にする。それ以外はモジュール | Book ch07-03:296・ch14-03:299 |
| crate の名前は `contracts-and-instances-<業務領域>-<層>`。製品の名前は `[[bin]]`（sas）にだけ残す | 文脈の名前で事業領域と分ける |
| mod.rs を使わない（`src/aggregates.rs` と `src/aggregates/concrete.rs` の形） | Book ch07-05:251・265・268 |
| ポートは trait で dyn で使い、dyn compatible に保つ | Reference traits:412・429・445・459 |
| 集約の欄は private、メソッドだけ pub。ID と指紋は newtype | API Guidelines C-STRUCT-PRIVATE:218・C-NEWTYPE:169 |
| adapters で外の trait を domain の型に実装しない（orphan rule） | Reference implementations:658・659・669 |
| 試験の部品（test-support）は dev-dependencies からだけ使う | `architecture.json` の dev_only と、その検査 |

## 確かめるコマンド

`cargo build`・`cargo test`（依存の向きとスキーマの検査を含む）・`cargo fmt --all -- --check`・`cargo clippy --workspace --all-targets -- -D warnings`
