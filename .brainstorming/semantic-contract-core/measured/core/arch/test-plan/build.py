"""テスト計画の頁（arch/test-plan/index.html）を組む。qa-advisor（arch/reviews/qa-test-plan.*）と反証役の指摘を、前提に照らして1つにしたもの。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.fig{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px;overflow-x:auto}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>基盤のテスト計画</title>')
fig = lambda n: '<div class="fig">' + open(os.path.join(H, n)).read() + '</div>'
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
body = f'''<main>
<h1>基盤のテスト計画</h1>
<p class="meta">利用者の問い「テスト計画としてはどのようなテストをする予定ですかね？？テスト規約はどうなるんだろうか。テストピラミッド的に何をどうするのがいいでしょうか。」への答え。qa-advisor の計画（arch/reviews/qa-test-plan.html）を、結論を渡さない反証役に通し、前提に照らして1つにした。arch.html はまだ直していない</p>
<p class="ans">単体テストを最も厚くし、domain の値の関数を決定表と状態遷移で確かめる。バイナリを通すシステムテストは、核で作った70場面のうち、バイナリでしか確かめられない約21場面だけにする。残りの場面は、それを守る側のテストへ移す</p>
{fig('pyramid.svg')}
<h2>1　レベルごとに何を確かめるか</h2>
{table(['レベル', '対象', 'テストダブル', '核の場面から移すもの'], [
 ['単体', 'domain の集約 ・ ドメインサービス ・ 値オブジェクト。再開か新規かを決める（決定表）・ 状態を導く（決定表）・ セッションの開いている → 閉じた（状態遷移 I45）・ 答えを5つの値へまとめる（I14）', '使わない（値を渡すだけ）', 'S-SES-2〜9（AI ツールの答え ・ 有効時間 ・ 設定の組み合わせ ＝ 決定表の列）'],
 ['コンポーネント統合：application', '使う場面ごとに、ポートが失敗したときの経路と、I33（意図と変更が合っているか）の判定の流れ', '保存のポートは Fake、外から答えを受け取るポート（judge ・ ai_sessions ・ tools）は Test Stub', 'given.judge を使う2場面（I33）'],
 ['コンポーネント統合：outbound', '本物の git ・ SQLite ・ ファイルを一時ディレクトリで動かす。git と snapshot に同じテストを当て、出力が一致することも確かめる（核の K5）。AI ツールは記録した出力で確かめ、本物は CI で呼ばない', '使わない（本物）', 'sql の14場面と S-SES-10（同時の登録）は crates/outbound/sqlite/tests/ へ'],
 ['コンポーネント統合：schemas', 'schemas/v1 の10ファイルが、正しいインスタンスを通し、誤ったインスタンスを断るか', '使わない', 'op が schema の24場面（核でも基盤を呼ばず JSON Schema の実装で検査していた）'],
 ['システム', 'バイナリを起動し、変更の記録 ・ 変更を見せる ・ 承認 ・ 設計の検査を protocol のコマンドだけで動かす。前提の状態づくりも結果の読み取りも protocol を通し、手元の DB を直接書かない', '使わない（模擬の AI を外から渡す経路を必要としない）', '残る約21場面（changes 14 ・ design 6 ・ S-SES-1）。形は protocol（Q4）が決まってから決める'],
 ['静的', 'arch.html 6節の検査と、test-suite/ の形を scenario.schema.json で検査する', '──', '──'],
])}
<h2>2　テストの規約（CODING.md のテストの節に置く）</h2>
{table(['規約', '中身', '根拠'], [
 ['単体テストの置き場所', '確かめるコードと同じファイルの #[cfg(test)] mod tests', 'Rust Book ch11-03「The convention is to create a module named tests in each file to contain the test functions and to annotate the module with cfg(test).」'],
 ['統合テストの置き場所', 'crate ごとに tests/ のファイル1つにし、中をモジュールに分ける', 'Cargo「you may want to consider creating a single integration test, and split the tests into multiple modules」（cargo-targets:285）'],
 ['テストダブル', 'tests/ と dev-dependencies の中だけに置く。本番のバイナリに切り替えの分岐を入れない。名前を grep する検査は持たない', 'Cargo「Dev-dependencies are not used when compiling a package for building, but are used for compiling tests, examples, and benchmarks.」・ qa：Test Logic in Production'],
 ['決定性', '時刻 ・ id ・ 乱数は値で渡す。domain での生成は clippy の forbid で止める（arch.html 6節と同じ仕組み）。テストごとに新しい前提の状態を作る', 'Q0 #14「同じ入力から同じ結果を返し」'],
 ['基盤が作る値の期待', 'ULID やコミットの参照を決まった文字列で期待しない。「記録と索引で同じ値」「ULID の形」のように値の関係で期待する', 'I37「書いた時に1回だけ、ほかの書き手と打ち合わせずに一意に決まる値」'],
 ['1テストに1つ', '1つのテストは1つの振る舞いを確かめる。テストの中に分岐を書かない。比べるときは何を確かめたかの文を付ける', 'qa：Conditional Test Logic ・ Assertion Roulette'],
 ['名前', 'テスト関数の名前は確かめる振る舞いを書く。使う場面と不変条件への結びつきは test-suite/ の traces と、単体テストのモジュールの説明に1か所だけ書く', '反証役：結びつけを2か所に持たない'],
 ['テストデータの正本', 'test-suite/*.json を正本にし、手で直す。核の build.py ・ ref/ ・ coverage.json は移さない', 'qa：正本を2つにしない'],
])}
<h2>3　完了基準と CI</h2>
{table(['完了基準', '中身'], [
 ['テストが通る', '静的テスト ・ cargo test --workspace ・ crate ごとの単独のビルドとテスト（arch.html 6節の -p の段と1つにまとめる）'],
 ['同じ入力から同じ結果', '検証の操作（I13 の範囲）は2回実行して出力が同じ。git と snapshot で出力が一致する'],
 ['確かめるテストが抜けていない', '不変条件46と使う場面の事後 ・ 失敗時の全行に、それを確かめるテストがある。いま核では90行のうち65行が「確かめない」（参照実装の範囲外）なので、実装の段でそのテストを書く'],
])}
<p class="meta">CI は毎回の push で、静的 → 単体とコンポーネント統合 → システムの順に全部を回す。本物の AI ツールは呼ばない</p>
<h2>4　外したもの</h2>
{table(['外したもの', '理由'], [
 ['テストダブルの名前 ・ 環境変数を grep する検査', 'dev-dependencies と tests/ の crate が本番のバイナリに入らないので、Cargo の仕組みで足りる'],
 ['場面の件数を固定する検査', '確かめるテストの無い65行に場面を足すたびに CI が落ちる'],
 ['全部の場面を2回実行して1バイトも同じ', 'ULID とコミットの参照で必ず違う。I13 が同じ結果を求めるのは検証の操作だけ'],
 ['使う場面ごとに呼び出しの順を確かめる Mock Object のテスト', '実装を直すたびに落ちる（qa：Overspecified Software）'],
 ['inbound/cli の単体テスト', '起動と結びつけだけ。ただし I37（id を1回だけ決める）を確かめるのは、sqlite のテストと ULID の外の crate だと表に書く'],
 ['行のカバレッジの数値目標', '不変条件と使う場面からテストへの対応表（完了基準の3つ目）で足りる'],
])}
<h2>5　前提の扱いを直したこと</h2>
{table(['変更前', '変更後', '指摘'], [
 ['「試験は、入力と期待結果の JSON の組で持ち…」「core は入出力に触らない」「ほかの言語の Concrete も、バイナリを呼んで…」を決着した前提として引いていた', 'どれも Q4（待ち）の下書き。決着しているのは Q0 #14「同じ入力から同じ結果を返し、別の言語の実装が使える適合の試験を持つ」。ポートを application に置く決定は、この Q0 #14 とあなたの承認で成り立つ', '反証役：board-answers.json の Q3 は open、Q4 は waiting'],
])}
<h2>6　決めてほしいこと</h2>
{table(['決めてほしいこと', '私の見解'], [
 ['このピラミッドと、核の70場面の移し替え（システムテストは約21場面）', '採る。あなたの「基盤側で作りこむ内容ではない」にも沿い、テストのためだけの入口を必要としない'],
 ['使う場面と不変条件からテストへの対応表を、実装のリポジトリに持つか（model.json を semantic-agent-schema/ に置くか）', '持つ。実装と一緒に変わるので、確かめるテストが抜けたことを CI で見つけられる場所に置く'],
])}
<p class="meta">先に決めることが1つある：状態を導く決定表の列（violated と error が同時にあるときの優先など）がモデルに無い。単体テストの前に model.json の states に表を足す</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
