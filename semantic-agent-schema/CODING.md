# コードの決まり

設計の仮説は `docs/design/arch/arch-v15.json`、根拠の原文の照合は `docs/design/rust/factcheck-rust.json` にある。ここには、いま守る決まりだけを書く。

## 言語に依存しない決まり

| 決まり | 理由 |
|---|---|
| リポジトリの根は製品の置き場。区切られた文脈は、根の直下の文脈の名前のフォルダ（いまは `schemas-and-instances/`） | 製品と文脈を同じものとして扱わない。事業領域は問題の側の言葉なので、置き場所の名前にしない |
| 文脈の中はポートとアダプターで作る。外とのやりとりは application の ports を adapters が実装する（呼び出し元を決めるポートは inbound/cli、ほかは outbound）。本ではポートは業務ロジック層が定義するが、記録と描画はドメインモデルを持たないので application に置く。各モジュールの application は、自分のユースケースが使うポート（と、そのうち使うメソッド）だけを自分の ports.rs に宣言する。ほかのモジュールと同じ形のポート（DocumentStore ・ SchemaValidator）も、それぞれのモジュールに同じ名前で宣言し、1つのアダプターが複数のモジュールのポートを実装する | 外の技術を業務ロジックから切り離す。ポートは使う側が「自分に何が要るか」として宣言するもので、形が同じでも変わる理由はモジュールごと |
| モジュールの application は、ほかのモジュールの application（ports ・ use_cases）を使わない。使ってよいのは domain の crate と、同じモジュールの中だけで、例外は無い。複数のモジュールが使う判断と値は domain に置く：文書が Concrete かインスタンスかの見分け（TargetKind）・ リポジトリの文書の一覧（Corpus）・ スキーマの所に当たる欄の定義（field_schema）・ 誰が打ったか（Caller）・ 記録にある書いた変更の行（ChangeRow ・ WriteRecord など）と SAS を通らない変更の見分け（unrecorded_since）は writing-domain、当たる rule と判定が読んだ所のハッシュ（reads_hash）・ 記録にある検証と判定の行（ValidateRow ・ RuleRow ・ Judgment）は validation-domain。答えの JSON の読み方（```json の囲みを外す）やコマンドの1行の組み方のような小さな道具は、使うモジュールごとに置く | 依存の向きを crate の境界で強制し、あるモジュールの手続きを変えてもほかのモジュールが壊れないようにする |
| 誰が打ったか（人か、名乗ったセッションか）は入口が決めて、値（writing-domain の Caller）でユースケースへ渡す。決める材料（--session ・ ツールの環境変数 ・ sas init の登録）を読むのも、記録をつけるコマンドで sessions に1行足すのも入口で、その手続き（identify ・ register ・ sas init の register_session）は記録のモジュールの use case に置き、CLI からだけ呼ぶ。ユースケースは誰が打ったかを自分で決めない | 外からの入力を読むのは入口のアダプターの責務。誰が打ったかは x-decided-by の欄を変えてよいかを決めるドメインの概念で、記録のモジュールの型にしない |
| 記録のポートは、使うモジュールが自分の ports.rs に、そのモジュールのユースケースが使うメソッドだけで宣言する（書く ChangeLog ＝ append_change ・ find_request ・ writes ・ last_judgments、検証 ValidationLog ＝ append_validate ・ writes ・ last_judgments、描画 ValidationLog ＝ append_validate）。検索は記録しないので持たない。記録のモジュールに残るのは、記録を読む UC7（OperationReader）と、入口が使う SessionLog ・ IdentifyCaller。sqlite のアダプター（SqliteLog）は1つで、手続きを自分のメソッドに持ち、すべての記録のポートをそれで実装する | 表の側から作った1つの大きな trait にすると、どのモジュールも使わないメソッドにまで依存し、記録のモジュールの application への依存が残る |
| 理由を表す欄は reason の1語にする。何の理由かは、付いている物で決まる（変更の reason は変えた理由、判定の reason は判定の理由）。CLI の引数も --reason | 同じことを表す語が intent ・ why ・ reason と分かれると、読む人と AI が別のものと読む |
| x-prompt の形は、直下に meaning（値を何として読むか）・ when（いつ、何のために読むか）・ rules（書くときの決まりの並び）を並べる。rule は id ・ rule ・ why ・ check ・ bad ・ good（bad ・ good は1つ以上の例の並びで必須。例は value ・ related ・ note のオブジェクトで、Abstract の $defs/example が形を持ち、bad の例は note も必須）と、任意の related_fields（判定で一緒に読む欄）を持つ。例の related のキーは related_fields にある所だけ（組み込みの検査 example-related）。手順の中でも同じ名前で渡す（meaning ・ when ・ related_fields） | read と reads が1字違いで取り違え、write ・ interpret は動詞で欄の中身が分からなかった。例の無い rule は、判定する AI と人とで違反の境目を別に読む |
| 断った理由の型（Refused）は、各モジュールの use_cases が、そのモジュールが断るときに使う欄だけで持つ。出力の形（command-result の kind = error）は inbound/cli の output が持ち、各モジュールの Refused を From で移す | CLI の出力の形は入口の事情で、モジュールが互いの断る理由の欄を知らなくてよくする |
| 文脈の中のフォルダは、業務領域を作るモジュール（`writing/` ・ `validation/` ・ `recording/`、ステップ5 で `rendering/`、ステップ8 で `search/`）と `adapters/`。名前はユースケースと場面で実際に使う言葉から付ける。文脈の根に domain を置かない | 業務領域ごとの手続きの置き場所を、言葉から引けるようにする（名前の見直し） |
| 集約（Concrete ・ インスタンス）は writing/domain に置く。検証のモジュールは集約を読んで rule に照らす業務サービスと判定の値を validation/domain に持つ。依存は writing-application → validation-domain → writing-domain と validation-application → validation-domain → writing-domain（書くモジュールは検証のモジュールの application を使わない） | 集約は作成 ・ 更新 ・ 削除のコマンドを受けて不変条件を守る。検証は単独でも使う。書く側は書く前に形を照らし、書いたあとに入力が変わった前の判定を、検証と同じ基準（validation-domain）で決める |
| 意味（自然言語の rule に照らして中身の意味を書く ・ 判定する ・ 探すこと）は、書く ・ 検証 ・ 検索のそれぞれの中に持つ。業務領域にしない。SAS はエージェントを起動しない：意味の部分は手順を打ったセッションに返し、そのセッションが渡した値と判定を機械の書き込み ・ 検証で確かめる（概念のページ版28） | 意味は仕事の性質で、別の仕事ではない |
| 業務ロジックは、まず use_cases にトランザクションスクリプトで書く。同じ理由で変わるものだけを domain へ引き上げる。自明なものは最初から domain に置いてよいが、根拠を探索の記録に書く | ドメインモデルは育てるもの |
| domain の中は部品の種類で分ける（aggregates/・values/・services/）。購読する手続きが無いうちは events/ を作らない | 先取りしない |
| ユースケースごとのファイルやフォルダは作らない | 土台の境界6 |
| crate・モジュール・ポートは、必要と分かった時点で足す。crate を足したら `architecture.json`（architecture-lint の規則）の components と allow にも足す | 最初に構造を全部作らない |
| 外のライブラリも `architecture.json` の component にし、allow で向きを許した crate だけが使う（rusqlite は outbound の sqlite、jsonschema は outbound の jsonschema）。試験だけで使う向きは kinds を dev にして許す | 外の技術を adapter に閉じ込めたことを、依存の検査（architecture-lint）で確かめる |
| ユースケースは、先に `schemas-and-instances/system-tests/scenarios/` に場面を書き、失敗を確かめてから作る。場面と保証 ・ 拡張の突き合わせは `system-tests/trace.json`、まだ確かめないものは `untested.json` に理由と一緒に置く | 保証と拡張のどれを確かめたかを、試験で数えられるようにする |
| SAS の DB（`<git-common-dir>/sas/sas.db`、Git が無ければ `.sas/`）はコミットしない。ID と時刻は outbound の sqlite が作り、application と domain は時刻に触らない | 決定的な試験のため |
| リポジトリの文書を書くのは use_cases だけで、形と集約の書く条件を確かめてから DocumentStore の write で書く。書く手前で断ったものは記録せず、理由 ・ 案内 ・ 間に入った変更をその場で返す | 書くのは SAS だけ。型に従わないものは作らず、理由を返す（UC1 の最低保証）。断った操作の記録は SAS が読まず、コミットしない記録は監査の証拠にもならない（探索の記録 step-04b） |
| ステップごとに `docs/design/exploration/step-NN.json` を書く | 探索の過程と、仮説との突き合わせを残す |

## Rust の決まり

| 決まり | 一次情報 |
|---|---|
| 根の Cargo.toml は virtual manifest。`[workspace]` に members と `resolver = "3"` | Cargo workspaces:241・258、Book ch14-03:207〜211 |
| 外のライブラリの版は `[workspace.dependencies]` に置き、各 crate は `workspace = true` で継承する | Cargo workspaces:359・363 |
| 依存の向きを強制したい境界と、外のライブラリを閉じ込めたい所だけを crate にする。それ以外はモジュール | Book ch07-03:296・ch14-03:299 |
| crate の名前はフォルダから決める：`<モジュール>-<層>`（writing-domain など）・ `<inbound|outbound>-<技術>` ・ test-support。製品の名前は `[[bin]]`（sas）にだけ残す | 文脈は1つなので文脈の名前を前に付けない。別の文脈は別のプロジェクトになる。名前とフォルダの一致は architecture の試験が確かめる |
| mod.rs を使わない（`src/aggregates.rs` と `src/aggregates/concrete.rs` の形） | Book ch07-05:251・265・268 |
| ポートは trait で dyn で使い、dyn compatible に保つ。実装が単純になるほうを選ぶ：手続きは `&dyn` を受け取るだけで型引数を持たず、アダプターは cli の main.rs で選んで渡す。呼び先はファイル ・ SQLite ・ git コマンドなので、dyn の呼び出しの手間は効かない | Reference traits:412・429・445・459 |
| 集約の欄は private、メソッドだけ pub。ID とハッシュは newtype | API Guidelines C-STRUCT-PRIVATE:218・C-NEWTYPE:169 |
| adapters で外の trait を domain の型に実装しない（orphan rule） | Reference implementations:658・659・669 |
| 試験の部品（test-support）は dev-dependencies からだけ使う | `architecture.json` が test-support へ向かう辺を許していないので、architecture-lint が通常の依存を止める。dev-dependencies から使うときは、その辺だけを kinds dev で許す |
| ポートの代役は、使う application の `tests/` に置く（test-support に置くと dev-dependencies が循環する）。検証 ・ 書く ・ 検索の試験が共に使う代役の振る舞いは `validation/application/tests/support.rs` に置き、ほかのモジュールの試験は `#[path]` で読む。ポートはモジュールごとに宣言するので、代役をポートに当てる impl は、読む側の試験がそのモジュールのポートに書く | 探索の記録 step-01 |
| `if let` が入れ子になるときは let chains（`if let … && let …`）で書く | clippy の collapsible_if（edition 2024） |
| serde_json は `preserve_order` で使い、オブジェクトから欄を除くときは `shift_remove` を使う。書き戻した文書の欄の並びを変えないため | serde_json 1.0.154 src/map.rs:3〜4・160〜164（探索の記録 step-02） |
| SQLite の表の CHECK を変える移行は、表を作り直して行を移す。移すあいだは外部キーの検査を止め、コミットの前に `pragma_foreign_key_check` で確かめる | SQLite の ALTER TABLE の文書 8節「Making Other Kinds Of Table Schema Changes」の手順（探索の記録 step-02） |
| ほかの表の trigger が読む表（changes ・ sessions）は作り直さず、欄を足す（`ALTER TABLE … ADD COLUMN`）。欄をまたぐ決まりは trigger で拒む。作り直すのは、どの trigger からも読まれない表だけ | 作り直すと、ほかの表の trigger も消して付け直すことになる（探索の記録 step-03） |
| 移行 0001〜0004 は、公開前の書き直し（ステップ4b と、版28 で sessions から起動の欄を外したとき）で表の最終の形にした。これからは版ごとに足し、足したものは変えない | 公開前で使う人がいない（探索の記録 step-04b）。置き場所も変えたので、前の `.sas/sas.db` は読まない |
| 欄の名前を変える移行は、表を作り直さずに `ALTER TABLE … RENAME COLUMN` で変える（行と CHECK ・ trigger の参照はそのまま移る）。その欄を読むビューは、名前を変える前に消し、あとで作り直す（移行 0006） | 作り直すとほかの表の trigger も付け直すことになる。名前を変えても行が保たれることは、版5 の DB を開く試験（migrate_reason.rs）で確かめる |
| 記録は種類ごとの表に分け、種類によって空になる欄を作らない（書いた変更は changes、検証は validations）。両方を並べて読むのはビュー operations | 1つの表に種類を混ぜると、条件付きの CHECK と「断ったものを除く」絞り込みが毎回入る（探索の記録 step-04b） |
| 記録の置き場所は入口が決める：`SAS_DB_PATH` → `<git-common-dir>/sas/sas.db` → Git が無ければ `.sas/sas.db`。SAS の記録とファイルのつながりは「いまの中身が、その経路の記録のどれかの事後か」で見る | ワークツリーで記録を1つにするため。最後の事後だけと比べると、ほかのワークツリーで書くたびに外の変更に見える（探索の記録 step-04b） |
| 打ったセッションの ID は `--session` → ツールごとの環境変数 → このフォルダ（Git の中なら作業ツリーの根）で `sas init` が最後に登録したセッション → 人（null）。環境変数の名前はコードに書かず、入口の `callers.json` に置き、原文か実測で確かめたものだけを入れる | ツールが増えても表に行を足すだけにする。確かめていない名前で人と AI を取り違えない |
| Git は outbound の git だけが、git コマンドで読む（`git log` ・ `git cat-file --filters` ・ `git rev-parse --path-format=absolute --git-common-dir` ・ `git rev-parse --show-toplevel`）。ライブラリを足さず、Git に書き込むコマンドを呼ばない | 探索の記録 step-03 ・ step-04b の照合 |

## 確かめるコマンド

`cargo build`・`cargo test`（依存の向きとスキーマの検査を含む。依存の向きは architecture-lint を起動するので、PATH に入れるか `../architecture-lint` で `cargo build` しておく）・`cargo fmt --all -- --check`・`cargo clippy --workspace --all-targets -- -D warnings`
