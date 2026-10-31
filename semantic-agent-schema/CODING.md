# コードの決まり

設計の仮説は `docs/design/arch/arch-v15.json`、根拠の原文の照合は `docs/design/rust/factcheck-rust.json` にある。ここには、いま守る決まりだけを書く。

## 言語に依存しない決まり

| 決まり | 理由 |
|---|---|
| リポジトリの根は事業領域。区切られた文脈は、根の直下の文脈の名前のフォルダ（いまは `schemas-and-instances/`） | 事業領域と文脈を同じものとして扱わない |
| 文脈全体を1つのヘキサゴナルにする。外とのやりとりは application の ports を adapters/outbound が実装する | 外の技術を業務ロジックから切り離す |
| 業務ロジックは、まず use_cases にトランザクションスクリプトで書く。同じ理由で変わるものだけを domain へ引き上げる。自明なものは最初から domain に置いてよいが、根拠を探索の記録に書く | ドメインモデルは育てるもの |
| domain の中は部品の種類で分ける（aggregates/・values/・services/）。購読する手続きが無いうちは events/ を作らない | 先取りしない |
| ユースケースごとのファイルやフォルダは作らない | 土台の境界6 |
| crate・モジュール・ポートは、必要と分かった時点で足す。足したら `architecture.json` にも足す | 最初に構造を全部作らない |
| 外のライブラリは、`architecture.json` の allow の libraries に書いた crate だけが使う（rusqlite は outbound の sqlite、jsonschema は outbound の jsonschema） | 外の技術を adapter に閉じ込めたことを、依存の検査で確かめる |
| ユースケースは、先に `test-suite/scenarios/` に場面を書き、失敗を確かめてから作る。場面と保証 ・ 拡張の突き合わせは `test-suite/trace.json`、まだ確かめないものは `untested.json` に理由と一緒に置く | 保証と拡張のどれを確かめたかを、試験で数えられるようにする |
| SAS の DB（`.sas/`）はコミットしない。ID と時刻は outbound の sqlite が作り、application と domain は時刻に触らない | 決定的な試験のため |
| リポジトリの文書を書くのは use_cases だけで、形と集約の書く条件を確かめてから DocumentStore の write で書く。書く手前で断ったものも、操作として記録する | 書くのは SAS だけ。型に従わないものは作らず、理由と記録を残す（UC1 の最低保証） |
| ステップごとに `docs/design/exploration/step-NN.json` を書く | 探索の過程と、仮説との突き合わせを残す |

## Rust の決まり

| 決まり | 一次情報 |
|---|---|
| 根の Cargo.toml は virtual manifest。`[workspace]` に members と `resolver = "3"` | Cargo workspaces:241・258、Book ch14-03:207〜211 |
| 外のライブラリの版は `[workspace.dependencies]` に置き、各 crate は `workspace = true` で継承する | Cargo workspaces:359・363 |
| 依存の向きを強制したい境界と、外のライブラリを閉じ込めたい所だけを crate にする。それ以外はモジュール | Book ch07-03:296・ch14-03:299 |
| crate の名前は `schemas-and-instances-<業務領域>-<層>`。製品の名前は `[[bin]]`（sas）にだけ残す | 文脈の名前で事業領域と分ける |
| mod.rs を使わない（`src/aggregates.rs` と `src/aggregates/concrete.rs` の形） | Book ch07-05:251・265・268 |
| ポートは trait で dyn で使い、dyn compatible に保つ | Reference traits:412・429・445・459 |
| 集約の欄は private、メソッドだけ pub。ID と指紋は newtype | API Guidelines C-STRUCT-PRIVATE:218・C-NEWTYPE:169 |
| adapters で外の trait を domain の型に実装しない（orphan rule） | Reference implementations:658・659・669 |
| 試験の部品（test-support）は dev-dependencies からだけ使う | `architecture.json` の dev_only と、その検査 |
| ポートの代役は、使う application の `tests/` に置く（test-support に置くと dev-dependencies が循環する） | 探索の記録 step-01 |
| `if let` が入れ子になるときは let chains（`if let … && let …`）で書く | clippy の collapsible_if（edition 2024） |
| serde_json は `preserve_order` で使い、オブジェクトから欄を除くときは `shift_remove` を使う。書き戻した文書の欄の並びを変えないため | serde_json 1.0.154 src/map.rs:3〜4・160〜164（探索の記録 step-02） |
| SQLite の表の CHECK を変える移行は、表を作り直して行を移す。移すあいだは外部キーの検査を止め、コミットの前に `pragma_foreign_key_check` で確かめる | SQLite の ALTER TABLE の文書 8節「Making Other Kinds Of Table Schema Changes」の手順（探索の記録 step-02） |
| ほかの表の trigger が読む表（operations）は作り直さず、欄を足す（`ALTER TABLE … ADD COLUMN`）。欄をまたぐ決まりは trigger で拒む。作り直すのは、どの trigger からも読まれない表だけ | 作り直すと、ほかの表の trigger も消して付け直すことになる（探索の記録 step-03） |
| Git は outbound の git だけが、git コマンドで読む（`git log` ・ `git cat-file --filters`）。ライブラリを足さず、Git に書き込むコマンドを呼ばない | 探索の記録 step-03 の照合 |

## 確かめるコマンド

`cargo build`・`cargo test`（依存の向きとスキーマの検査を含む）・`cargo fmt --all -- --check`・`cargo clippy --workspace --all-targets -- -D warnings`
