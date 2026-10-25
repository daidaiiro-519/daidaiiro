"""アーキテクチャの頁（arch.html）の見出しの答え ・ 1節 ・ 4節 ・ 6〜8節を、arch/ports の承認（ポートは application、domain は値の関数）に合わせて書く。5節は arch_tree.py が書く。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(H)
p = os.path.join(C, 'arch.html'); s = open(p).read()
def between(s, a, b, new):
    i = s.index(a); j = s.index(b, i); return s[:i] + new + s[j:]
def table(head, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{h}</th>' for h in head) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
s = between(s, '<p class="meta">核（ドメインモデル', '<h2>1　', '''<p class="meta">核（ドメインモデル ・ ポート）と Rust の原典から決め、ddd ・ platform ・ 反証役のレビュー（前提の束つき）を統合した。ポートの置き場所は、利用者の問いを受けて ddd ・ 反証役にもう一度通し、承認を得た（arch/ports/）。schema-driven と ctxtrace の実装は手本にしていない。全文 arch/reviews/</p>
<div class="ans">ポートとアダプター。コードを crates/ にまとめ、業務領域ごとの crate（crates/domain/）・ 使う場面とポートの application ・ 駆動される側の outbound ・ 入口の inbound に分け、許す辺を1つの表に持つ。<br>domain は値を受け取って値を返す関数だけを持ち、入出力は application がポートを通して行う。domain の crate は8つ（declaration ・ instance ・ change ・ resolver ・ validation ・ render ・ search ・ session）、ドメインサービスは4つ。入口のコマンドは、AI が行う操作のセマンティック7つ（procedure を返す）と、処理する機械的なコマンドに分かれる。依存の向きは Cargo の辺と循環の禁止で止め、Cargo が知らない「どの辺を許すか」は表1つで持ち、cargo metadata ・ cargo clippy ・ cargo build -p で確かめる。</div>
''')
s = s.replace('<td>業務の側がポート（trait）を持ち、外の技術の側が実装する。依存は外から中への1方向</td>', '<td>application がポート（trait）を持ち、outbound が実装する。依存は外から中への1方向</td>')
s = s.replace('<td>同じ入力から同じ結果（Q0 #14）</td><td>業務の側は入出力をしない。時刻と id もポートか値で受け取る</td>', '<td>同じ入力から同じ結果（Q0 #14、決着）。待ちの Q4 の下書きにも「core は入出力に触らないので変えない」</td><td>domain は値を受け取って値を返す。読む ・ 書く ・ AI への依頼は application がポートで行い、時刻と id は inbound から値で渡す</td>')
s = s.replace('<td>補完（描画 ・ 移す ・ 公開）は形と参照の検査が中心（Q1 #23）</td><td>同じ crate の形のまま、中は関数で書く。集約とリポジトリの作りを持ち込まない</td>', '<td>補完（描画 ・ 移す ・ 公開）は形と参照の検査が中心（Q1 #23）</td><td>移す ・ 公開は集約を持たず、ドメインサービスの関数だけ。描画とインスタンスは、モデルの集約（ビュー ・ デザインシステム ・ インスタンス）だけを持つ</td>')
svg = open(os.path.join(H, 'arch.svg')).read()
dep = table(['crate', '依存してよい先', '理由'], [
 ['domain/declaration', '（無し）', 'Abstract ・ Concrete ・ glossary と、Abstract が決める値の型（名前の型 ・ 対象 ・ procedure）。ファクトリ Concrete の雛形。Concrete の操作（照らす ・ 移す ・ 公開する部分の版）と Abstract の操作（版を出す ・ 移し方）は集約が持つ'],
 ['domain/instance', 'declaration', '集約 インスタンス。ファクトリ インスタンス（作る ・ 取り込む）'],
 ['domain/change', 'declaration', '判定の記録 ・ 変更の記録 ・ 検証結果 ・ 状態。ファクトリ 変更のまとめ（記録から機械的に作り、コミットする）。判定の記録はこの業務領域（Q9 #6）'],
 ['domain/resolver', 'instance ・ declaration', 'Semantic Schema Resolver。validation から分けた。描画と検証の両方が使う（Q5 #1）'],
 ['domain/validation', 'resolver ・ change ・ declaration', 'Semantic Schema Validator（2段：依頼の一覧を返す ・ 答えをまとめる）。change は検証を呼ばない'],
 ['domain/render', 'resolver ・ change ・ declaration', 'Semantic Schema Renderer。状態と rule ごとの結果を描く。基盤の既定のビューも Concrete のビューも同じ形で描く'],
 ['domain/search', 'change ・ instance ・ declaration', '探す条件（JMESPath の where ・ select）・ 出どころ ・ 探した範囲'],
 ['domain/session', '（無し）', '集約 セッション。Session Selector（再開か新規かを、渡された値だけで決める）。子のセッションの型'],
 ['application', 'domain/ のすべて', '使う場面と ports/（駆動される側13）。読む → ドメインサービス ・ 集約の操作 → 書く'],
 ['outbound/agents-contract', 'application ・ outbound/ai-cli', 'judge を実装する。依頼の一覧を Agents 契約の形へ直し、ai-cli で起動する'],
 ['outbound/ai-cli ・ git ・ fs ・ tools ・ sqlite', 'application', 'ports/ の trait を実装する。sqlite は手元の DB（セッション ・ 手元の索引 ・ 変更の記録 ・ 意図の判定）'],
 ['inbound/cli', 'application ・ outbound/*', '起動と結びつけ。コマンド（セマンティック7つ ・ 機械的）。時刻と id を値で渡す'],
])
s = between(s, '<h2>4　依存の向き</h2>', '<h2>5　', f'''<h2>4　依存の向き</h2><div class="fig" style="background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px;overflow-x:auto">{svg}</div>
<p class="meta">左の crate が、中の crate に依存してよい。書いていない組はすべて禁止。domain/ から application ・ outbound ・ inbound への辺は種類を問わず0件</p>
{dep}
<h3>計算の途中で外の情報を使うドメインサービス</h3>
{table(['ドメインサービス', 'application がすること', 'ドメインサービスがすること'], [
 ['Semantic Schema Resolver（セマンティック スキーマ リゾルバー）', '文脈のリポジトリの契約 ・ glossary と、閉じた一覧にある公開された言葉の版を先に読んで渡す（Q4 #9）', '渡された中だけで解決し、無いものは断る'],
 ['Semantic Schema Validator（セマンティック スキーマ バリデーター）', '依頼の一覧を受け取り、judge ・ tools のポートで依頼して答えを渡す。答えどうしに連鎖があれば繰り返す', '① 依頼の一覧（呼び出し call）を返す ② 答えを5つの値にまとめる（割れた答えと根拠の無い答えは unverified。I14）'],
 ['Session Selector（セッション セレクター）', 'トランザクションを取り、索引 ・ AI ツール ・ 設定を読んで渡し、返った決定を索引へ書いてコミットする', '渡された値だけで、再開する ID か「新しく立てる」を返す'],
])}
''')
s = s.replace('<td>dev-dependencies は循環を許す（change → fs → change が通った）</td><td>cargo metadata の辺を種類（normal ・ dev ・ build）ごとに architecture.json と突き合わせる。domain から adapters ・ entry への辺は種類を問わず0件</td>', '<td>dev-dependencies は循環を許す（change → fs → change が通った）</td><td>cargo metadata の辺を種類（normal ・ dev ・ build）ごとに architecture.json と突き合わせる。domain から application ・ outbound ・ inbound への辺は種類を問わず0件</td>')
if False: s = s.replace('<td>clippy の disallowed は #[allow] で外せる</td><td>domain/ の clippy を forbid にする（E0453 で止まることを実測）。主な守りは、crate ごとに許す外の crate の表</td>', '<td>clippy の disallowed は #[allow] で外せる</td><td>domain/ の clippy を forbid にする（E0453 で止まることを実測）。主な守りは、crate ごとに許す外の crate の表</td></tr><tr><td>trait を通した入出力は clippy で止まらない</td><td>domain/ に trait のポートを置かない。ポートは application/src/ports/ だけに置き、domain/ の公開の関数が引数に dyn ・ impl の trait を取らないことを検査する')
s = between(s, '<h2>6　', '<h2>7　', '<h2>6　決定的にする検査</h2><p class="meta">依存の検証の大半は Cargo がする。足すのは、Cargo が知らない「どの辺を許すか」の規則1つと、Cargo の外の2つだけ。規則はリポジトリ直下の architecture-lint で検査する</p>' + table(['何を止めるか', '誰が止めるか', 'コマンド'], [
 ['Cargo.toml に書いていない crate を使う', 'Cargo（コンパイルが止まる）', 'cargo build'],
 ['通常の依存の循環', 'Cargo（resolver が止める）', 'cargo build'],
 ['許していない辺を Cargo.toml に書く（例：domain/change → outbound/fs）。dev ・ build の辺の循環と、表に無い外の crate も同じ', 'architecture.json（component と allow の規則）。Cargo はこの向きを知らないので、これだけは規則を持つ。検査は内作の architecture-lint（ACDR 0199 ・ 0200）', 'architecture-lint check --rules architecture.json --cargo .'],
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
 ['domain の crate 9つ（migration ・ publish を含む。Resolver は validation の中）', '8つ。migration ・ publish を無くし（移す ・ 版を出すは集約の操作）、resolver を validation から分ける', '利用者「正しいディレクトリ構成を優先してください」・ arch/services の承認'],
 ['ドメインサービス13', '4つ（Resolver ・ Validator ・ Renderer ・ Session Selector）。残りは集約の操作とファクトリ3つ', '利用者「エンティティや値オブジェクト、集約でやるべき処理がほぼほぼ」・ arch/services の承認'],
 ['入口のコマンド：semantic-read ・ write ・ search と search ・ get ・ patch ・ validate、ほかは名前だけ', 'セマンティック7つ（search ・ read ・ create ・ write ・ judge ・ import ・ migrate。procedure を返す）と、機械的なコマンド。消すは無くし、不要なものは superseded で残す', 'arch/command-sim の承認'],
 ['変更の記録をコミットする', '変更の記録と意図の判定は手元の DB。目的ごとのまとめと rule の判定をコミットする', 'arch/change-summary の承認'],
 ['AI ツールごとに5 crate', 'agents-contract（腐敗防止層）1つ ＋ ai-cli 1つ（ツールはモジュール）', '3人とも'],
 ['compose の crate', 'inbound/cli の中（実行ファイルが2つになったら分ける）', 'ddd ・ platform'],
 ['検査は Cargo.toml の辺と clippy', '許す辺の表（種類と外の crate を含む）・ clippy の forbid ・ 単独ビルド。pub use の grep と trait の検査は外した', 'platform（Cargo が止めない書き方4つを実測）・ 利用者「crateである程度依存の検証はできるよね？？」'],
]))
s = between(s, '<h2>8　', '</main>', '<h2>8　決めたこと</h2>' + table(['決めたこと', '中身'], [
 ['スキーマの正本の置き場所', 'semantic-agent-schema/schemas/v1/。集約の型は crates/ の中、集約の中身（Abstract のファイル13 と既定のビュー）は crates/ の外'],
 ['Abstract のファイル（10 → 13）', 'abstract ・ change ・ judgment ・ session ・ settings ・ view ・ catalog ・ design-tokens ・ glossary ・ published に、query（探す条件 ・ 結果 ・ 出どころ）・ procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）・ change-summary（コミットするまとめ）を足す'],
 ['基盤の既定のビュー', 'session ・ change-summary ・ judgment を見るビューを、ビューの形のファイルとして schemas/v1/views/ に同梱する。文脈に Concrete のビューがあればそれを使い、無ければ既定を使う（I26）'],
 ['セマンティックのコマンド', 'AI が行う操作ごとに7つ。procedure を返し、AI は入力だけを生成し、機械的なコマンドが適用する。検査は2回（適用する前に入力、適用した後に出力）'],
 ['変更の記録の置き場所', '1件ずつの変更の記録と意図の判定は手元の DB（sqlite）。コミットごとのまとめ（前回のコミットからの差し引きと例外）と rule の判定はファイルでコミットする'],
 ['消す', '無くす。不要なものは superseded にして残す。項目を消すのは Patch の remove。確定していない変更の取り消しは Git の側'],
 ['テスト', 'システムテストは約21場面（test-suite/）。残りは単体 ・ コンポーネント統合へ（テスト計画の頁）'],
]) + '<h3>スキーマは集約なのに、なぜ crates/ の外か</h3>' + table(['分けるもの', '置き場所', '前提'], [
 ['集約 Semantic Abstract Schema の型と不変条件（出した版は変えない ・ $id の形）', 'crates/domain/declaration/src/aggregates/abstract_schema.rs', 'model.json aggregates（意味の宣言）'],
 ['集約の中身：版ごとのファイル', 'semantic-agent-schema/schemas/v1/（版を足すときは v2/ を並べる）', 'Q9「Abstract の版は公開された言葉である。版を並べて出す」・ Q6「最初の公開から出した版は変えない」'],
 ['基盤が読む方法', 'ポートを通さず、declaration の build.rs がビルドのときに取り込む', 'model.json ports.not_ports「Abstract（基盤の中）」'],
 ['crates/ の中に置かない理由', '公開された言葉の読み手は Rust の外にもいる（Concrete の作者の $schema ・ ほかの言語の Concrete ・ test-suite/ の試験）', 'boundary「実装の言語は契約に出さない」'],
]) + '<p class="meta">まだ正本（model.json ・ Abstract のファイル）には適用していない。ACDR で変更前と変更後を示し、承認を得てから適用する</p>\n')
open(p, 'w').write(s); print('ok')
