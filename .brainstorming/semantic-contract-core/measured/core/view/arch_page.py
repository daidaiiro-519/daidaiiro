"""アーキテクチャの頁（arch.html）の見出しの答え ・ 1節 ・ 4節 ・ 6〜8節を、arch/ports の承認（ポートは application、domain は値の関数）に合わせて書く。5節は arch_tree.py が書く。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(H)
p = os.path.join(C, 'arch.html'); s = open(p).read()
def between(s, a, b, new):
    i = s.index(a); j = s.index(b, i); return s[:i] + new + s[j:]
def table(head, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{h}</th>' for h in head) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
s = between(s, '<p class="meta">核（ドメインモデル', '<h2>1　', '''<p class="meta">核（ドメインモデル ・ ポート）と Rust の原典から決め、ddd ・ platform ・ 反証役のレビュー（前提の束つき）を統合した。ポートの置き場所は、利用者の問いを受けて ddd ・ 反証役にもう一度通し、承認を得た（arch/ports/）。schema-driven と ctxtrace の実装は手本にしていない。全文 arch/reviews/</p>
<div class="ans">ポートとアダプター。コードを crates/ にまとめ、業務領域ごとの crate（crates/domain/）・ 使う場面とポートの application ・ 駆動される側の outbound ・ 入口の inbound に分け、許す辺を1つの表に持つ。<br>domain は値を受け取って値を返す関数だけを持ち、入出力は application がポートを通して行う。依存の向きは Cargo の辺と循環の禁止で止め、Cargo が知らない「どの辺を許すか」は表1つで持ち、cargo metadata ・ cargo clippy ・ cargo build -p で確かめる。</div>
''')
s = s.replace('<td>業務の側がポート（trait）を持ち、外の技術の側が実装する。依存は外から中への1方向</td>', '<td>application がポート（trait）を持ち、outbound が実装する。依存は外から中への1方向</td>')
s = s.replace('<td>同じ入力から同じ結果（Q0 #14）</td><td>業務の側は入出力をしない。時刻と id もポートか値で受け取る</td>', '<td>同じ入力から同じ結果（Q0 #14、決着）。待ちの Q4 の下書きにも「core は入出力に触らないので変えない」</td><td>domain は値を受け取って値を返す。読む ・ 書く ・ AI への依頼は application がポートで行い、時刻と id は inbound から値で渡す</td>')
s = s.replace('<td>補完（描画 ・ 移す ・ 公開）は形と参照の検査が中心（Q1 #23）</td><td>同じ crate の形のまま、中は関数で書く。集約とリポジトリの作りを持ち込まない</td>', '<td>補完（描画 ・ 移す ・ 公開）は形と参照の検査が中心（Q1 #23）</td><td>移す ・ 公開は集約を持たず、ドメインサービスの関数だけ。描画とインスタンスは、モデルの集約（ビュー ・ デザインシステム ・ インスタンス）だけを持つ</td>')
svg = open(os.path.join(H, 'arch.svg')).read()
dep = table(['crate', '依存してよい先', '理由'], [
 ['domain/declaration', '（無し）', 'Abstract ・ Concrete ・ glossary と、Abstract が決める値の型（名前の型 ・ 対象）。作成する ・ Concrete を照らす'],
 ['domain/change', 'declaration', '判定の記録 ・ 変更の記録 ・ 検証結果 ・ 状態。状態を導く ・ 変更を見せる（並べる順と例外）・ 例外をまとめる。判定の記録はこの業務領域（Q9 #6）'],
 ['domain/validation', 'change ・ declaration', 'Resolver ・ Validator。判定の記録の使い直しと I33（変更の記録）に change の型を使う。change は検証を呼ばない'],
 ['domain/render', 'validation ・ change ・ declaration', 'Renderer。Resolver を使い（Q5 #1）、状態と rule ごとの結果を描く'],
 ['domain/search', 'change ・ declaration', '意味で探して取り出す。判定の結果と状態でも探す'],
 ['domain/instance', 'declaration', 'インスタンス ・ 取り込む。作成のあとの検証は application が順に呼ぶ'],
 ['domain/migration ・ domain/publish', 'declaration', '移す ・ 公開された言葉の版を出す'],
 ['domain/session', '（無し）', 'セッション ・ 再開か新規かを決める（渡された値だけで決まる）'],
 ['application', 'domain/ のすべて', '使う場面10と ports/（駆動される側12）。読む → ドメインサービス → 書く'],
 ['outbound/agents-contract', 'application ・ outbound/ai-cli', 'judge を実装する。依頼の一覧を Agents 契約の形へ直し、ai-cli で起動する'],
 ['outbound/ai-cli ・ git ・ fs ・ tools ・ sqlite', 'application', 'ports/ の trait を実装する'],
 ['inbound/cli', 'application ・ outbound/*', '起動と結びつけ。時刻と id を値で渡す'],
])
s = between(s, '<h2>4　依存の向き</h2>', '<h2>5　', f'''<h2>4　依存の向き</h2><div class="fig" style="background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px;overflow-x:auto">{svg}</div>
<p class="meta">左の crate が、中の crate に依存してよい。書いていない組はすべて禁止。domain/ から application ・ outbound ・ inbound への辺は種類を問わず0件</p>
{dep}
<h3>計算の途中で外の情報を使う3つのドメインサービス</h3>
{table(['ドメインサービス', 'application がすること', 'ドメインサービスがすること'], [
 ['Semantic Schema Resolver', '文脈のリポジトリの契約 ・ glossary と、閉じた一覧にある公開された言葉の版を先に読んで渡す（Q4 #9）', '渡された中だけで解決し、無いものは断る'],
 ['Semantic Schema Validator', '依頼の一覧を受け取り、judge ・ tools のポートで依頼して答えを渡す。答えどうしに連鎖があれば繰り返す', '① 依頼の一覧（呼び出し call）を返す ② 答えを5つの値にまとめる（割れた答えと根拠の無い答えは unverified。I14）'],
 ['再開か新規かを決める', 'トランザクションを取り、索引 ・ AI ツール ・ 設定を読んで渡し、返った決定を索引へ書いてコミットする', '渡された値だけで、再開する ID か「新しく立てる」を返す'],
])}
''')
s = s.replace('<td>dev-dependencies は循環を許す（change → fs → change が通った）</td><td>cargo metadata の辺を種類（normal ・ dev ・ build）ごとに architecture.json と突き合わせる。domain から adapters ・ entry への辺は種類を問わず0件</td>', '<td>dev-dependencies は循環を許す（change → fs → change が通った）</td><td>cargo metadata の辺を種類（normal ・ dev ・ build）ごとに architecture.json と突き合わせる。domain から application ・ outbound ・ inbound への辺は種類を問わず0件</td>')
if False: s = s.replace('<td>clippy の disallowed は #[allow] で外せる</td><td>domain/ の clippy を forbid にする（E0453 で止まることを実測）。主な守りは、crate ごとに許す外の crate の表</td>', '<td>clippy の disallowed は #[allow] で外せる</td><td>domain/ の clippy を forbid にする（E0453 で止まることを実測）。主な守りは、crate ごとに許す外の crate の表</td></tr><tr><td>trait を通した入出力は clippy で止まらない</td><td>domain/ に trait のポートを置かない。ポートは application/src/ports/ だけに置き、domain/ の公開の関数が引数に dyn ・ impl の trait を取らないことを検査する')
s = between(s, '<h2>6　', '<h2>7　', '<h2>6　決定的にする検査</h2><p class="meta">依存の検証の大半は Cargo がする。足すのは、Cargo が知らない「どの辺を許すか」の規則1つと、Cargo の外の2つだけ。規則はリポジトリ直下の dependency-rules で検査する</p>' + table(['何を止めるか', '誰が止めるか', 'コマンド'], [
 ['Cargo.toml に書いていない crate を使う', 'Cargo（コンパイルが止まる）', 'cargo build'],
 ['通常の依存の循環', 'Cargo（resolver が止める）', 'cargo build'],
 ['許していない辺を Cargo.toml に書く（例：domain/change → outbound/fs）。dev ・ build の辺の循環と、表に無い外の crate も同じ', 'architecture.json（component と allow の規則）。Cargo はこの向きを知らないので、これだけは規則を持つ。検査は内作の dependency-rules（ACDR 0199）', 'dependency-rules check --rules architecture.json --cargo .'],
 ['domain が std の入出力 ・ 時刻 ・ 乱数を使う', 'clippy の disallowed（forbid にして #[allow] で外せなくする）', 'cargo clippy --workspace'],
 ['workspace でまとめてビルドしたときだけ feature が足される（jsonschema の取りに行く機能など）', 'crate を1つずつ単独にビルドする', 'cargo build -p &lt;crate&gt; ・ cargo tree -e features'],
]) + '<p class="meta">外したもの：外の crate の pub use の grep（コードレビューで足りる）・ domain の関数が trait を引数に取らないかの検査（outbound は application にしか依存できないので、domain の trait を実装できない。上の表の検査で足りる）</p>\n')
s = between(s, '<h2>7　レビューで直したこと</h2>', '<h2>8　', '<h2>7　レビューで直したこと</h2>' + table(['下書き', '直した後', '指摘'], [
 ['contexts/', 'domain/', 'ddd ・ 反証：Q9 #11（文脈は1つ）と食い違う'],
 ['kernel → values（2つ以上の領域が使う値）', '無くす。値オブジェクトは、その語を持つ業務領域の crate へ', '反証（2回目）：Evans p.15 Modules「Give the modules names that become part of the ubiquitous language」'],
 ['workflow', 'application', 'ddd：層の語にそろえる'],
 ['ポートは domain の各 crate の ports.rs', 'application の ports/。domain は値の関数', '利用者「私はportsはよくapplication層に置いてるんだよね」・ ddd ・ 反証（2回目）：Q0 #14「同じ入力から同じ結果」と Q4 の下書き「core は入出力に触らない」'],
 ['判定の記録を validation へ移す', 'change のまま。validation → change の向きにする', '反証（2回目）：Q9 #6 と食い違う'],
 ['render は declaration ・ values だけ', 'validation ・ change ・ declaration', '反証（2回目）：Q5 #1（Resolver は描画も使う）'],
 ['instance → validation', '外す（application が順に呼ぶ）', '反証（2回目）：処理の順序は domain の辺ではない'],
 ['答えを5つの値へ直すのは agents-contract', 'validation（Validator の②）', '反証（2回目）：I14 はドメインの不変条件。Q9 #16'],
 ['adapters/ ・ entry/', 'outbound/ ・ inbound/', '反証（2回目）：Q5 #9「英語はコードのフォルダ名 inbound ・ outbound と同じ」'],
 ['実装の無いポート3つ', 'fs/published.rs ・ fs/rendered.rs ・ tools', '反証（2回目）'],
 ['AI ツールごとに5 crate', 'agents-contract（腐敗防止層）1つ ＋ ai-cli 1つ（ツールはモジュール）', '3人とも'],
 ['compose の crate', 'inbound/cli の中（実行ファイルが2つになったら分ける）', 'ddd ・ platform'],
 ['検査は Cargo.toml の辺と clippy', '許す辺の表（種類と外の crate を含む）・ clippy の forbid ・ 単独ビルド。pub use の grep と trait の検査は外した', 'platform（Cargo が止めない書き方4つを実測）・ 利用者「crateである程度依存の検証はできるよね？？」'],
]))
s = between(s, '<h2>8　決めてほしいこと</h2>', '</main>', '<h2>8　決めてほしいこと</h2>' + table(['決めてほしいこと', '私の見解'], [
 ['スキーマの正本の置き場所を、Q6 #19 の src/semantic-agent-schema/schemas/ から semantic-agent-schema/schemas/v1/ へ', '移す。下の表のとおり、集約の型は crates/ の中、集約の中身（10ファイル）は crates/ の外に置く'],
 ['Q3 の答え（core と adapters の2つ）を、このページの構成に置き換える', '置き換える'],
 ['behaviors/ の扱いを、下の表の変更後にする（qa-advisor の判定）', '変える。製品の機能ではなく基盤自身のテストデータなので、名前と持ち込む範囲を絞る'],
]) + '<h3>behaviors/ は何か ・ 何を持ち込むか（qa-advisor ・ 全文 arch/reviews/qa-behaviors.html）</h3>' + table(['', '変更前（いまの5節）', '変更後', '根拠'], [
 ['何か', '受け入れの場面', '基盤自身のシステムテストのテストケースとテストデータ。製品の機能ではない。利用者が行う受け入れテスト（UAT）でもない', 'qa-advisor：受け入れテストは「想定ユーザーが実施することが理想的」で、この場面は機械が操作の結果を比べる'],
 ['なぜ持ち込むか', '──', '前提の「試験は、入力と期待結果の JSON の組で持ち、どのインターフェースとどの言語の実装も同じ試験を通す」の実体だから。言語に依らないことはバイナリと protocol で満たすので、バイナリを外から起動して回す', 'Q4 の下書き（待ち。premises/board-answers.json:162 ・ 164）と Q0 #14（決着）「別の言語の実装が使える適合の試験を持つ」'],
 ['置き場所と名前', 'behaviors/ に70場面', 'test-suite/ にインターフェースに依らない56場面（schema 24 ・ check_design 6 ・ 基盤の操作 26）。SQLite のアダプターだけを試験する sql の14場面は crates/outbound/sqlite/tests/ へ', 'Cargo「Integration tests go in the tests directory.」・ 判断基準の語「テストスイート」'],
 ['回す側', 'crates/inbound/cli/tests/acceptance.rs', 'crates/inbound/cli/tests/test_suite.rs。CARGO_BIN_EXE でバイナリを起動する。模擬の判定と模擬の AI はバイナリの外から渡し、バイナリに切り替えの分岐を入れない', 'qa-advisor：Test Logic in Production'],
 ['持ち込まないもの', 'coverage.json も移す', 'coverage.json ・ build.py ・ ref/（参照実装）はボードに残す。核の段の記録で、実装の段では使わない。場面の traces（使う場面と不変条件への結びつき）は残す', 'qa-advisor：traceability ・ 正本を2つにしない'],
]) + '<h3>スキーマは集約なのに、なぜ crates/ の外か</h3>' + table(['分けるもの', '置き場所', '前提'], [
 ['集約 Semantic Abstract Schema の型と不変条件（出した版は変えない ・ $id の形）', 'crates/domain/declaration/src/aggregates/abstract_schema.rs', 'model.json aggregates（意味の宣言）'],
 ['集約の中身：版ごとの10ファイル', 'semantic-agent-schema/schemas/v1/（版を足すときは v2/ を並べる）', 'Q9「Abstract の版（10ファイルの形）は公開された言葉である。版を並べて出す」・ Q6「最初の公開から出した版は変えない」'],
 ['基盤が読む方法', 'ポートを通さず、declaration の build.rs がビルドのときに取り込む', 'model.json ports.not_ports「Abstract（基盤の中）」'],
 ['crates/ の中に置かない理由', '公開された言葉の読み手は Rust の外にもいる（Concrete の作者の $schema ・ ほかの言語の Concrete ・ behaviors/ の試験）。crate の並びを直すたびに公開された言葉の場所が動くことになる', 'boundary「検証のエンジンと protocol … 実装の言語は契約に出さない」'],
 ['条件', 'crate を crates.io へ出さない（cargo package は crate の外のファイルを含めない）。配るのはバイナリ', 'Q4 の下書き（待ち）「ほかの言語の Concrete も、バイナリを呼んで基盤を使う」'],
]) + '<p class="meta">判定の記録の業務領域を移す問いは取り下げた（Q9 #6 のまま）。Validator の定義の文は承認どおり2段に直した（model.json C60）</p>\n')
open(p, 'w').write(s); print('ok')
