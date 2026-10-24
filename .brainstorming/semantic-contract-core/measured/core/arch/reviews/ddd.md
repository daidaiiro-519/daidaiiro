# ddd-advisor の回答（設計相談）

回答の JSON は ddd-advisor validate に合格（検出0件）。見出しは ddd-advisor の回答テンプレート（answer.schema.json の描画）の順。

タスクとの対応：1 ポートとアダプターの適否 → 判断の道筋 1〜2 ・ 注意　／　2 案 A ・ B ・ C → 比較した案　／　3 依存の向き → 具体例（表）・ 図　／　4 未決3件 → 判断の道筋 5〜7　／　5 外せるもの → 判断の道筋 8

## 相談

Semantic Agent Schema の基盤を Rust で実装するアーキテクチャ（crate と業務領域の切り方 ・ 依存の向き ・ ディレクトリの名前）を、核のドメインモデル（model/model.json）から決める。下書き arch/draft.md の案 A ・ B ・ C と未決3件を評価する

## 前置き

| 前提 | 内容 | 出所 |
|---|---|---|
| 区切られた文脈は1つ | 基盤の内側は1つの区切られた文脈で、中に業務領域ごとの論理的な境界（名前空間 ・ モジュール）を置く。crate は、この論理的な境界をビルドで守る道具であり、文脈を分けることではない | board-answers.json Q9 #11「基盤の内側は1つの区切られた文脈にし、中に業務領域ごとの論理的な境界（名前空間 ・ モジュール）を置く」・ #12「同じデータを操作するユースケースを複数の文脈に分散させない」 |
| 業務領域11と区分 | 中核5（意味の宣言 ・ 意味の検証 ・ 変更の確かめと例外の判断 ・ 意味で探す ・ 目的ごとのセッションの管理）、補完4（スキーマの保存と公開 ・ instance の作成と変更 ・ マイグレーション ・ 描画）、一般2（AI と外のツールによる判定 ・ 保存と版の管理） | model/model.json subdomains |
| ポートでないもの | JSON Schema の検証の部品と式の評価は、ドメインサービスの中で使う部品で、ポートにしない | model/model.json ports.not_ports「JSON Schema の検証の部品と式の評価（ドメインサービスの中で使う部品）」 |
| 人の入口だけのポート | 「判定する」は人の操作の入口だけ（MCP には見せない）、「Abstract の版を出す ・ 移す」は基盤を作る側の CLI だけ | model/model.json ports.inbound の第4行 ・ 第7行。Q1 #26「判定するは人の入口だけに見せる」 |
| 腐敗防止層の置き場所 | Agents 契約は腐敗防止層として基盤の側に置く | board-answers.json Q9 #16「AI の validator は外の上流で、Agents 契約を腐敗防止層として基盤の側に置き、中核のモデルを守る」 |
| 置き場所の食い違い | Q6 #19 は正本を src/semantic-agent-schema/schemas/ とするが、利用者の新しい指示は ctxtrace と同じくリポジトリ直下に semantic-agent-schema/ を作る。前提の README の優先順で、利用者の指示を採る | premises/user-instructions.md「まずは新しくctxtraceのようにですね、このsemantic-agent-schemaというフォルダを作成して」・ premises/README.md「利用者の指示の原文 ＞ この会話の決定 ＞ ボードの決着の原文」 |
| Q3 は開いている | Q3 のいまの答え（core と adapters の2つ ＝ 案 A）は参考で、決着ではない | premises/README.md「Q3 は開いている」 |
| Rust の性質（判断基準の範囲外） | (1) ワークスペースの crate どうしの依存は明示する（ch14-03:150-151「Cargo doesn’t assume that crates in a workspace will depend on each other, so we need to be explicit about the dependency relationships.」）(2) 依存の図は循環を許さないが、dev-dependencies は例外（resolver:568-569「Usually the resolver does not allow cycles in the graph, but it does allow them for dev-dependencies.」）(3) 同じ crate の中では、子のモジュールは祖先の項目を使え、pub の項目は兄弟からも使える（ch07-03:87-88「items in child modules can use the items in their ancestor modules」）── モジュールの間の向きはコンパイラが止めない (4) orphan rule（implementations:155「a trait implementation is only allowed if either the trait or at least one of the types in the implementation is defined in the current crate」） | sources/rust/ の原文。DDD の判断基準の範囲外で、判定の根拠にはせず、構成の手段としてだけ使う |

## 結論

- 判定：**弱い**（下書き draft.md の評価）
- 結論：下書きの案 B（業務領域ごとの crate）は向きとして正しいが、辺に循環が2つあり、名前が決着 Q9 と食い違うので、判定の記録を意味の検証の crate へ置き、状態と検証結果の型を共通の値の crate へ下ろし、contexts と kernel と workflow の名前を改めてから採る
- なぜ：中核5つはドメインモデルなのでポートとアダプターが適し、業務領域の論理的な境界を crate で守るのは Q9 #11 の「1つの文脈の中の論理的な境界」に合う。ただし Validator が判定の記録を読み書きし、Renderer が状態を読むので、draft の辺のままでは意味の検証 ⇄ 変更の確かめ、描画 ⇄ 変更の確かめ の循環になり、ビルドが通らない

## 図で確認する

直したあとの crate の依存（矢印は「依存する」。ポートの trait は呼ぶ側の crate が持ち、アダプターが外から実装する）

| 節点 | 中身 |
|---|---|
| values | values（複数の業務領域が使う値オブジェクト：検証結果の5つの値 ・ インスタンスの状態 ・ 名前の型 ・ 置き場所の型 ・ 対象 ・ id） |
| declaration | declaration（意味の宣言：Semantic Abstract Schema ・ Semantic Concrete Schema ・ glossary） |
| validation | validation（意味の検証：Resolver ・ Validator ・ 判定の記録） |
| change | change（変更の確かめと例外の判断：変更の記録 ・ 状態を導く ・ 変更を見せる ・ 例外をまとめる） |
| search | search（意味で探す） |
| session | session（目的ごとのセッションの管理） |
| render | render（描画：ビュー ・ デザインシステム ・ Renderer） |
| instance | instance（instance の作成と変更 ・ 取り込む） |
| migration | migration（マイグレーション） |
| publish | publish（スキーマの保存と公開） |
| application | application（駆動する側のポート10） |
| agents | adapters/agents-contract（腐敗防止層：Agents 契約 ・ 作法の2） |
| adapters | adapters/（changes-git ・ changes-snapshot ・ files ・ sqlite ・ ai-tools） |
| cli | entry/cli（結びつけもここ） |

辺（A → B は「A が B に依存する」。点線はアダプターがポートを実装する辺）：

- declaration → values
- validation → declaration
- instance → validation
- change → validation
- change → render（描画の差分）
- render → declaration
- search → change（状態で探す）
- search → validation（判定の結果で探す）
- session → values
- migration → declaration
- publish → declaration
- application → change
- application → session
- application → search
- application → instance
- application → migration
- application → publish
- agents → validation（judge ・ 外のツールの trait を実装）
- adapters → change（changes を実装）　点線
- adapters → session（sessions_store ・ ai_session ・ settings を実装）　点線
- adapters → agents（AI ツールの起動を実装）　点線
- cli → application
- cli → adapters
- cli → agents

## 比較した案

| 案 | 何が起きるか | 採否 |
|---|---|---|
| A：業務の crate 1つ（業務領域はモジュール）＋ アダプターの crate 1つ（Q3 のいまの答え） | 業務 ← アダプターの向きだけをビルドが守る。同じ crate の中では子のモジュールが祖先の項目を使え、pub の項目は兄弟から使えるので（ch07-03:87-88）、業務領域どうしの向き（例：意味の検証が変更の確かめを使う逆流）はビルドでは止まらず、レビューに頼る。利用者の求める「決定的なアーキテクチャ」を満たさない | 採らない |
| B（直して採る）：業務領域ごとの crate ＋ 値の crate ＋ application ＋ アダプター ＋ 入口 | 業務領域どうし ・ 業務とアダプターの全部の辺を Cargo.toml が持ち、表に無い辺はビルドが通らない（ch14-03:150-151）。crate は1つのワークスペースで同じ版 ・ 同じバイナリで出るので、Q9 #11 の「1つの文脈の中の論理的な境界」のままで、文脈を分けることにはならない。代わりに、業務領域の境界を直すと crate の間で型を移す手間が増える | 採る |
| C：B の各業務領域を domain と application に分ける | 駆動する側のポート10のうち、変更を記録する（変更の確かめ ＋ セッションの索引）・ 判定する（判定の記録 ＋ 変更の仕組みで確定）・ 検証する（意味の検証 ＋ 状態を導く）は業務領域をまたぐ。業務領域ごとの application に割ると、またぐポートの置き場所が決まらず、crate が倍になるだけで守る辺が増えない | 採らない |

## 判断の道筋

1. **核の業務ロジックはドメインモデルか**　中核5つは不変条件46行 ・ 集約9つ ・ 決定表（状態を導く）を持つドメインモデルである。判断基準どおり、ポートとアダプターが適する
2. **全部の業務領域に同じ技術方式を当てるか**　当てない。補完の描画（形と参照の検査だけ）・ マイグレーション ・ 公開は、同じ crate の形のまま、中はトランザクションスクリプト（関数）で書き、集約とリポジトリの作りを持ち込まない。一般の2つは業務の crate にせず、アダプターと腐敗防止層にする
3. **crate は区切られた文脈か**　違う。Q9 #11 で文脈は1つで、crate は業務領域の論理的な境界をビルドで守る手段である。したがってディレクトリ名に contexts を使わない
4. **draft の辺は循環しないか**　循環する。Validator は「判定した値がいまと同じ判定の記録は使い直す」ので判定の記録を読む（validation → 判定の記録）が、判定の記録は変更の確かめの集約で、change → validation もある。Renderer は状態を描くが状態は変更の確かめが導き、change → render もある。判定の記録を validation に、状態の型を values に置けば両方とも消える
5. **未決1：workflow の crate を置くか、業務領域ごとの application に寄せるか**　1つの application の crate を置く（名前は workflow ではなく application）。駆動する側のポート10は業務領域をまたぐので、業務領域ごとに割ると置き場所が決まらない。各モジュールは1回に1つの集約だけを変える。人の入口だけのポート2つは、mcp を足す時点で別の crate に分ける
6. **未決2：判定の記録の集約を change に置くか validation に置くか**　validation の crate に置く。Validator が書き、使い直し（判定した値が同じなら使う）と割れたら unverified の不変条件はそこで効く。change に置くと validation ⇄ change の循環になる。モデルの業務領域の付け替えは差し戻しとして核で決める
7. **未決3：AI ツールのアダプターを1つの crate にするか、ツールごとにするか**　腐敗防止層（agents-contract）を1つと、ツールごとの違いを持つ ai-tools を1つにし、ツールはモジュールで分ける。ツールが別の依存を要する時点で、そのツールだけを crate に分ける（判断基準の範囲外の判断で、DDD の根拠は腐敗防止層を1か所に置くことだけ）
8. **外せるもの**　(1) 案 C (2) compose の crate（入口が cli だけのあいだは cli の起動時に結びつける）(3) mcp の crate（Q4 が待ち）(4) AI ツールごとの5つの crate（モジュールにする）(5) settings-file の crate（files に入れる。どちらもファイルを読むだけ）(6) 関係の索引と CQRS（永続化モデルが2つになるのは、遅いと測れてから。Q1 #34）。外さないもの：changes-snapshot（Git に依らないことを同じ場面で確かめる唯一の手段）

## 設計の手順

1. **リポジトリ直下に semantic-agent-schema/ を作り、ワークスペースの下に schemas/ ・ domain/ ・ application/ ・ adapters/ ・ entry/ を置く**　schemas/ は Semantic Abstract Schema の10ファイル（公開された言葉。Q9 #14）で、どの crate のものでもない。domain/ の下は業務領域ごとの crate、application/ は駆動する側のポート、adapters/ は駆動される側、entry/ は入口。名前は「domain ・ application ・ adapters」を判断基準の層の名前（業務ロジック層の別名ドメイン層 ・ アプリケーション層 ・ アダプター）にそろえる
2. **domain/ に crate を10個置く：values ・ declaration ・ validation ・ change ・ search ・ session ・ render ・ instance ・ migration ・ publish**　中核5 ＋ 補完4 ＋ values。一般の2つ（AI と外のツールによる判定 ・ 保存と版の管理）は domain/ に置かない。values は、2つ以上の業務領域の crate が使う値オブジェクトだけを持つ（検証結果の5つの値 ・ インスタンスの状態の4つ ・ 名前の型（URN）・ 置き場所の型（JSON Pointer）・ 対象 ・ 変更の記録の id（ULID）・ セッション ID）。1つの業務領域だけが使う値オブジェクトは、その crate に置く
3. **集約を crate に割り当てる**　Semantic Abstract Schema ・ Semantic Concrete Schema ・ glossary → declaration。インスタンス → instance。判定の記録 → validation（モデルの業務領域は「変更の確かめと例外の判断」のままにし、置き場所だけを変える。理由は未決2）。変更の記録 → change。ビュー ・ デザインシステム → render。セッション → session
4. **ドメインサービス13を、モデルの業務領域どおりの crate に置く**　作成する ・ Concrete を照らす → declaration。Resolver ・ Validator → validation。状態を導く ・ 変更を見せる ・ 例外をまとめる → change。意味で探して取り出す → search。Renderer → render。移す → migration。取り込む → instance。公開された言葉の版を出す → publish。再開か新規かを決める → session
5. **駆動される側のポート（trait）を、それを呼ぶ crate に置く**　変更の仕組み ・ 変更の記録を読む書く → change。判定の依頼（judge）・ 外のツールを呼ぶ ・ 公開された言葉を取りに行く ・ 判定の記録を読む書く ・ インスタンスを読む → validation。インスタンスを書く → instance。契約を読む書く → declaration。描画したものを書き出す → render。sessions_store ・ ai_session ・ 設定を読む → session。ポートは業務ロジックの側が定義し、アダプターは自分の型に実装する（orphan rule で許される。implementations:155）
6. **下の「依存の向き」の表を、ワークスペースの1つのデータとして持ち、cargo metadata の依存の図と突き合わせる試験にする**　dev-dependencies は循環が許されるので（resolver:568-569）、試験は通常の依存と dev-dependencies の両方を表と照らす。表に無い辺は0件
7. **application/ を1つの crate にし、駆動する側のポート10をモジュールに分ける**　名前は draft の workflow ではなく application（判断基準の語：アプリケーション層）。各モジュールは、集約を読み込む → コマンドを1つ実行する → 保存する → 必要ならポートを呼ぶ、だけを書き、1回に変える集約は1つにする。「判定する」と「Abstract の版を出す ・ 移す」は人の入口 ・ 基盤を作る側の CLI だけに見せるので、mcp を足す時点（Q4 のあと）でこの2つを別の crate に分け、mcp → その crate の辺を表に載せない
8. **adapters/ に agents-contract（腐敗防止層）を1つ置き、AI ツールごとの違いは ai-tools の1つの crate のモジュールにする**　agents-contract は validation の judge と外のツールの trait を実装し、値の写しを渡す ・ 答えを同じ JSON Schema で検査する ・ 5つの値へ直す を1か所で持つ。そのうえで「AI ツールを起動する」trait を自分で定義し、ai-tools が Claude Code ・ Codex ・ Copilot ・ Gemini ・ Kiro ごとのモジュールで実装する。ai-tools は session の ai_session も実装する
9. **結びつけは entry/cli の起動時に書く。compose の crate は入口が2つになるまで作らない**　受け入れの試験（場面の JSON）は、application を changes-snapshot ・ files（一時ディレクトリ）・ sqlite（一時ファイル）で結びつけて回す。Git でもスナップショットでも同じ結果になることを、changes の2つのアダプターで同じ場面を通して確かめる

## 具体例

依存の向きの表（左の crate が中の crate に依存してよい。書いていない組はすべて禁止。application は domain/ の全部に、adapters/ は自分が実装するポートの crate と values に、entry/cli は application と adapters/ に依存してよい）

| 依存する側 | 依存してよい先 | 使う場面 ・ ドメインサービス |
|---|---|---|
| declaration | values | 作成する ・ Concrete を照らす が名前の型と置き場所の型を使う |
| validation | declaration ・ values | Resolver が $schema ・ $ref ・ glossary ・ Concrete の拡張項目 ・ 公開する部分の版を解決する |
| instance | validation ・ declaration ・ values | 作成と変更のあとに検証する。取り込むは $schema を1つに決めるのに Concrete を読む |
| change | validation ・ render ・ declaration ・ values | 変更を見せる が全インスタンスを検証し、描画の差分を出し、決定権の欄の書き換えを例外にする。状態を導く が判定の記録を読む。I33 は Validator に、対象と前後の値を値として渡して判定させる |
| render | declaration ・ values | ビューの式が指す欄が Concrete に在るかを検査する。状態と rule ごとの結果は values の型の値として受け取る |
| search | change ・ validation ・ declaration ・ values | term ・ x-relationship ・ glossary ・ 判定の結果 ・ 状態で探す |
| session | values だけ | 再開か新規かは、セッションの枠 ・ AI ツールの答え ・ 設定の値だけで決まる |
| migration | declaration ・ values | 移す が Abstract の新しい版へ Concrete を合わせる |
| publish | declaration ・ values | 公開された言葉の版を出す が Concrete の公開する部分を読む |

禁止の要点：validation → change（判定の記録を validation に置いて消す）・ render → change（状態の型を values に置いて消す）・ session → ほかの業務領域 ・ declaration → validation ・ domain/ のどれか → adapters/ ・ mcp → 人の入口だけのポート

## 根拠

| 判断基準 | 引用 |
|---|---|
| ports-and-adapters | 条件：業務ロジックがドメインモデル ・ 対応：ポートとアダプターを選択する ・ 理由：業務ロジックを技術的な関心事から切り離せる。レイヤードアーキテクチャでも実装はできるが、集約と値オブジェクトを基盤から独立させるには工夫をしなければならない |
| ports-and-adapters | ポート：業務ロジック層が定義する抽象的なインターフェースで、インフラストラクチャ層が実装する。業務ロジック層は、基盤のコンポーネントを直接には参照せず、ポートを介して連係する。 |
| design-heuristics | 実装方法：ドメインモデル ・ 向く業務領域：業務ルールが複雑な業務領域、つまり中核 ・ 選ぶときの手がかり：業務ルールが複雑である |
| layered-architecture | 技術方式（レイヤードアーキテクチャ、ポートとアダプター、CQRS）は、システム全体の構造ではなく、1つの区切られた文脈の全体構造とも限らない。 |
| bounded-context | 物理的な境界は1つだが、内部には業務領域ごとの論理的な境界（名前空間 ・ モジュール ・ パッケージ）が存在する |
| bounded-context | 区分：区切られた文脈 ・ 発見か設計か：設計する |
| bounded-context | 対応：業務ロジックをよく理解していないときや、変更が多いときは、広い範囲で区切る ・ 理由：誤って分割した文脈の修正は高くつく。論理的な境界の再定義は、物理的な境界の変更よりはるかに容易である |
| layered-architecture | レイヤーは論理的な境界で、すべての層が同じライフサイクルで、1つの単位として実装 ・ 進化 ・ デプロイされる。 |
| shared-kernel | モデルの共有（Shared Kernel）：複数の区切られた文脈が、同じモデルかその一部を実装する連係方法。共有部分は関係するすべての文脈の要求に基づいて設計し、それを使用するすべての文脈と整合させる。 |
| aggregate | 対応：集約の状態は集約内部の業務ロジックだけが変更できるようにし、外部には参照だけを許可する ・ 理由：関連する業務ロジックを、すべて集約自身の1か所に実装できる |
| aggregate | 対応：1つのトランザクションで変更する集約は1つにし、複数の集約の状態は集約ごとに別々にコミットする ・ 理由：一貫性を維持する境界の定義が、正しいことを保証する |
| aggregate | 対応：集約 ・ フィールド ・ メソッド ・ 業務イベントの名前を、区切られた文脈の同じ言葉で命名する ・ 理由：ソースコードは、開発者どうしや業務エキスパートとの会話で使う同じ言葉に基づいて書く。複雑な業務ロジックでは特に重要である |
| domain-service | 問い：複数の集約を参照する計算のロジックか ・ 「はい」なら：業務サービスで記述すると、わかりやすくなる |
| anticorruption-layer | 場合：下流の文脈が中核の業務領域を含む ・ 理由：中核の業務領域のモデル作りには細心の注意を必要とする。供給側のモデルの影響を受けると、中核の課題を表すモデルが歪む可能性がある |
| ports-and-adapters | ポートとアダプターの結びつけは、依存性注入か、アプリケーションの起動時の設定で実施する。 |
| ports-and-adapters | 条件：ドメインモデルで、永続化モデルが複数 ・ 対応：CQRS を選択する ・ 理由：技術方式を判定する経験則では、永続化モデルが複数なら CQRS へ進む |
| design-heuristics | できるだけ単純な道具を選択し、CQRS・ドメインモデル・イベント履歴式ドメインモデルなどの複雑な手段は、本当に必要なときに最後の手段として使用する |

## 注意

| アンチパターン | リスク | 代替案 |
|---|---|---|
| ディレクトリ名 contexts/（draft 4節） | Q9 #11 で文脈は1つと決着している。crate ごとに「文脈」と名乗ると、crate ごとに語の意味を変えてよい、という読み方を招き、同じ語（例：対象 ・ 判定）が crate ごとに別の型になる | domain/ にし、crate の名前は業務領域の語にする。文脈を分け直すのは Q9 #13 の契機（チームが増える ・ 別のライフサイクル ・ 同じ語が2つの意味）が起きてから |
| 共通の crate の名前 kernel（draft 4節） | 判断基準の「モデルの共有（Shared Kernel）」は区切られた文脈どうしの連係で、1つの文脈の中には当たらない。名前から、文脈が複数あるように読まれる。さらに、何でも入れる置き場所になりやすい | values にし、2つ以上の業務領域の crate が使う値オブジェクトだけを置く。1つの crate だけが使う型が values にあれば移す、という検査を試験に足す（（推測）型ごとの使用先の数を数える試験） |
| 判定の記録を change に置いたまま validation ← change の辺を引く（draft の未決2 と辺） | Validator は判定の記録を使い直すので validation → change が要り、change → validation と循環する。Cargo は循環をビルドで止める | 判定の記録の集約を validation の crate に置く。人の判定（U16）は change から validation の集約のコマンドを呼ぶ。I33 は change が対象と前後の値を値として Validator に渡す |
| Renderer が状態を変更の確かめから読む | change → render（描画の差分）と render → change（状態）で循環する | インスタンスの状態の4つの値を values に置き、Renderer は状態を値として受け取る。状態を導く決定表は change に残す |
| 一般の業務領域を domain/ に入れる、または腐敗防止層を AI ツールごとに書く | Agents 契約（値の写しを渡す ・ 答えを JSON Schema で検査する ・ 5つの値へ直す）が5つのアダプターに散り、ツールごとに答えの扱いがずれる | adapters/agents-contract を1つ置き、ツールごとの違い（旗 ・ セッションの続け方 ・ JSON の取り出し）だけを ai-tools のモジュールに置く |
| 補完の業務領域にも集約とリポジトリの作りを持ち込む | 描画は「形と参照の検査だけ」を持つ（Q1 #23）。中核と同じ作りにすると、変える理由の無いコードが増える | crate の形は同じにし、中は関数（トランザクションスクリプト）で書く。ポート（保存の口）は中核と同じものを使う |

## 差し戻し

| 疑われる上流 | 疑う理由 |
|---|---|
| 核のモデル model.json の 判定の記録 の業務領域（いまは「変更の確かめと例外の判断」） | 書くのは主に Validator（意味の検証）で、使い直しの不変条件もその時点で効く。crate の置き場所だけを変えるのは設計の判断として成り立つが、業務領域の付け替えをするかは核の合意の論点として決める |
| ボード Q6 #19 の正本の置き場所 src/semantic-agent-schema/schemas/ | 利用者の新しい指示（ctxtrace と同じくリポジトリ直下に semantic-agent-schema/）と食い違う。semantic-agent-schema/schemas/ へ直す |
| ボード Q3 #3（crate は core と adapters の2つ） | 業務領域どうしの向きをビルドで守れない。Q3 の答えを、この回答の crate の割り当てと依存の向きの表で置き換える |

## 次にすること

- 依存の向きの表を、ワークスペースの1つのデータとして書く
- 判定の記録の業務領域を付け替えるかを、核の論点として利用者に出す
- semantic-agent-schema/ に schemas ・ domain ・ application ・ adapters ・ entry の空の crate を作り、表と cargo metadata の突き合わせの試験を先に通す
- Q3 #3 と Q6 #19 の直しを、ACDR の差分に入れる

## 関連する基準

| 判断基準 | 関係 |
|---|---|
| open-host-service | schemas/ の10ファイルと protocol は、外の Skill ・ Agent への公開された言葉（Q9 #14）。crate の内部の型と分けて版を保つ |
| layered-architecture | 補完の業務領域の中の書き方（関数で書く）と、レイヤーは同じライフサイクルの論理的な境界であること |

## AI が作業するときの注意

| 違反しやすい点 | 検査する方法 |
|---|---|
| 実装する AI が、便利だからと domain/ の crate の Cargo.toml に辺を足す（例：validation に change を足す） | 依存の向きの表と cargo metadata の突き合わせの試験（通常と dev-dependencies の両方）を CI で回し、表に無い辺を0件にする |
| src/schema-driven/ や ctxtrace の名前（core ・ engine など）を crate 名に持ち込む | crate 名がモデルの業務領域の語 ・ 判断基準の層の語（domain ・ application ・ adapters）のどれかに当たるかを、名前の一覧で照らす |
| values に、1つの業務領域だけが使う型を置き始める | values の各型を使う domain/ の crate の数を数え、1以下の型を出す |
| application のモジュールが2つの集約を1回で書き換える | （推測）application の各モジュールで、保存のポートを呼ぶ集約が1種類かをレビューの項目にする |

## 前提の外として分けた指摘（判断基準にも sources/rust/ にも原文が無い）

- clippy の disallowed（draft 5節）で業務の crate の入出力を止めること：std の入出力は Cargo の辺では止まらないので要るが、clippy の原文は sources/rust/ に無い
- schemas/ の10ファイルを crate に取り込む手段（include_str! など）：原文が sources/rust/ に無い
- （推測）変更を記録する（変更の記録のファイル ＋ 手元のセッションの索引）は2つの置き場所に書くので、途中で止まると索引の無い変更の記録が残る。U2 ・ U5 の「作業した本人はそれを書いたセッションで引ける」を守る書く順と、残ったときの扱いは platform と qa の観点で決める
