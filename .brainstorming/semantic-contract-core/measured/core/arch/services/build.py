"""ドメインサービスの分け直しの頁（arch/services/index.html）。ddd-advisor（arch/reviews/ddd-services.*）と反証役の判定を、前提に照らして1つにしたもの。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.diff span{display:block;padding:0 10px;white-space:pre}.two{grid-template-columns:1fr!important}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>ドメインサービスの分け直し</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
SV = '<span class="ok">ドメインサービス</span>'
rows = [
 ['作成する', 'ファクトリ', 'Concrete の雛形（Abstract から）は declaration、インスタンスの雛形（Concrete から）は instance のファクトリ', '集約を1つ丸ごと作る。Evans「Create an entire aggregate as a piece, enforcing its invariants.」'],
 ['Concrete を照らす', '集約の操作 ＋ Validator', 'Concrete の中で閉じる不変条件（I4 rule の id が一意）は集約 Concrete の根。Abstract に従うか（I3 ・ I6）は、Abstract を schema として Concrete を検証する Validator', 'Concrete は Abstract のインスタンスでもある（Q1 #6）ので、従うかの検査は検証と同じ処理'],
 ['Semantic Schema Resolver', SV, 'resolver（自分の crate）', 'Concrete ・ glossary ・ 公開された言葉の版 ・ 指す先のインスタンスを読み、どの集約にも属さない'],
 ['Semantic Schema Validator', SV, 'validation。答えを5つの値にまとめる規則（I14）は値オブジェクト「検証結果」、使い直すかの判定（I16）は集約「判定の記録」へ移す', 'rule × インスタンスの組で決まり、どの集約にも属さない'],
 ['状態を導く', '値オブジェクトの操作', '「インスタンスの状態」を検証結果の並びから決定表で作る（I19）', 'Evans「moving complex logic into value objects when a concept fitting the responsibility presents itself」'],
 ['変更を見せる', 'ファクトリ ＋ 使う場面', '変更のまとまり（保存しない値）を、変更の記録と変更の前後から作るファクトリ（I31 ・ I35 ・ I36）は change。全インスタンスの検証と描画を呼ぶ順は application', 'change は検証も描画も呼ばない（4節の依存の向き）'],
 ['例外をまとめる', '値オブジェクトの操作', '検証結果の並びを原因ごとにまとめる（I23）', '1種類の値をまとめる計算'],
 ['意味で探して取り出す', '使う場面 ＋ 値オブジェクト', '探すのは application（リポジトリへの問い合わせ）。一致の決まり（I25 ・ I41）と探した範囲（I42）は値オブジェクト', 'Evans「Provide methods that select objects based on criteria meaningful to domain experts.」（Repositories）'],
 ['Semantic Schema Renderer', SV, 'render。描く前の検査（I26）は集約「ビュー」と「デザインシステム」へ移す', 'ビューが無い kind も既定の描き方で描くので、集約「ビュー」の操作にできない'],
 ['移す', '集約の操作', '集約 Concrete のコマンド。移し方は Abstract の版が持つ（I27）', '変える集約は Concrete 1つ'],
 ['取り込む', 'ファクトリ ＋ 使う場面', 'インスタンスのファクトリ（作成する の2段目と同じ）。ファイルを読む ・ 下書きの一覧は application', 'インスタンスを作る入口を1つにする'],
 ['公開された言葉の版を出す', '集約の操作', '集約 Concrete と Abstract のコマンド（出した版は変えない I1 ・ I7）', '各集約自身の不変条件'],
 ['再開か新規かを決める', SV, 'session。「閉じる」は集約「セッション」のコマンドへ移し、サービスは「再開する ID」か「新しく立てる（閉じる ID）」を返すだけにする', 'AI ツールの答え ・ 設定 ・ 時刻を合わせて決める方針で、セッション自身の状態だけでは決まらない'],
]
before = '''crates/domain/
  declaration/  services/ create.rs ・ conform.rs
  change/       services/ derive_state.rs ・ show_changes.rs ・ exceptions.rs
  resolver/     services/ resolver.rs
  validation/   services/ validator.rs
  search/       services/ search.rs
  session/      services/ resume_or_new.rs
  render/       services/ renderer.rs
  instance/     services/ import.rs
  migration/    services/ migrate.rs
  publish/      services/ publish.rs'''
after = '''crates/domain/
  declaration/  aggregates/ abstract_schema.rs（版を出す ・ 移し方）・ concrete_schema.rs（照らす ・ 移す ・ 公開する部分の版）・ glossary.rs
                factories/ concrete.rs（Abstract から Concrete の雛形）
                search_scope.rs（探した範囲 ・ 一致の決まり）
  change/       aggregates/ judgment.rs（使い直すか）・ change_record.rs
                factories/ change_set.rs（変更のまとまり）
                verdict.rs（検証結果：5つの値へまとめる ・ 原因ごとにまとめる）・ instance_state.rs（決定表）
  resolver/     services/ resolver.rs
  validation/   services/ validator.rs
  render/       aggregates/ view.rs ・ design_system.rs（描く前の検査）
                services/ renderer.rs
  instance/     aggregates/ instance.rs
                factories/ instance.rs（雛形から作る ・ 取り込む）
  session/      aggregates/ session.rs（立てる ・ 閉じる）
                services/ selector.rs（Session Selector）'''
def pre(text, cls):
    return '<pre class="v diff">' + ''.join(f'<span class="{cls}">{l}</span>' for l in text.split('\n')) + '</pre>'
body = f'''<main>
<h1>ドメインサービスの分け直し</h1>
<p class="meta">利用者の見立て「ドメインサービスではなく、エンティティや値オブジェクト、集約でやるべき処理がほぼほぼ」「再開か新規かを決めるも…ドメインサービスだと思うから４つか」を、ddd-advisor（arch/reviews/ddd-services.html）と、結論を渡さない反証役に通し、前提に照らして1つにした。arch.html と model.json はまだ直していない</p>
<p class="ans">ドメインサービスは13から4つ（Resolver ・ Validator ・ Renderer ・ 再開か新規かを決める）にする。残りの9つは、ファクトリ3 ・ 集約の操作 ・ 値オブジェクトの操作 ・ 使う場面へ移す。あなたの見立てどおりの数になった</p>
<h2>1　13の行き先</h2>
{table(['いまのドメインサービス', '分け直した後', 'どこへ', '理由'], rows)}
<h2>2　ドメインサービス4つの名前（英語と日本語の対）</h2>
{table(['英語（型の名前）', '日本語', 'ファイル'], [
 ['Semantic Schema Resolver（SemanticSchemaResolver）', 'セマンティックスキーマリゾルバー', 'resolver/src/services/resolver.rs'],
 ['Semantic Schema Validator（SemanticSchemaValidator）', 'セマンティックスキーマバリデーター', 'validation/src/services/validator.rs'],
 ['Semantic Schema Renderer（SemanticSchemaRenderer）', 'セマンティックスキーマレンダラー', 'render/src/services/renderer.rs'],
 ['Session Selector（SessionSelector）', 'セッションセレクター', 'session/src/services/selector.rs'],
])}
{table(['決めたこと', '理由'], [
 ['日本語は英語の技術用語をそのままカタカナにする', 'Q5（決着）の日本語「解決器 ・ 検証器」の「器」は、あなたが使わないと決めた語（比喩）に入っている。技術用語に戻す'],
 ['Renderer の英語は model.json のまま', 'Q5 には Renderer の名前が無い。Resolver ・ Validator と同じ形にそろえた'],
 ['再開か新規かを決めるを Session Selector にする', 'いまの名前は処理を述べた文なので、ほかの3つと同じく役割の名詞にした。開いているセッションを再開するか新しく立てるかを選び、ID か「新しく立てる」を返す。「判定」は判定の記録の語と重なるので使わない'],
])}
<h2>3　domain の中のディレクトリ</h2>
<div class="two"><div class="pane"><h3>変更前（ドメインサービス13 ・ crate 9）</h3>{pre(before, 'd')}</div><div class="pane on"><h3>変更後（ドメインサービス4 ・ crate 7）</h3>{pre(after, 'a')}</div></div>
<h3>依存の向き（変更後）</h3>
{table(['crate', '依存してよい先', '理由'], [
 ['declaration', '（無し）', 'Abstract ・ Concrete ・ glossary と、Abstract が決める値の型'],
 ['instance', 'declaration', 'インスタンスは Concrete に従う'],
 ['change', 'declaration', '判定の記録 ・ 変更の記録 ・ 検証結果 ・ 状態。対象の型は declaration'],
 ['resolver', 'instance ・ declaration', '$ref ・ glossary ・ 公開された言葉の版を解決し、x-relationship の指す先のインスタンスをたどる'],
 ['validation', 'resolver ・ change ・ instance ・ declaration', 'インスタンスを検証する。解決は resolver に任せ、判定の記録の使い直しと I33 に change の型を使う'],
 ['render', 'resolver ・ change ・ instance ・ declaration', 'インスタンスを状態と rule ごとの結果と一緒に描く。Resolver を使い（Q5 #1）、Validator は見ない'],
 ['session', '（無し）', 'セッションと Session Selector。渡された値だけで決まる'],
])}
<p class="meta">resolver を自分の crate にしたので、render は Validator を見ずに Resolver だけを使える。これまでの表には validation → instance と render → instance が無かった（Validator と Renderer はインスタンスを読むのに、型を使えなかった）ので足した。resolver を validation から分けるのは、ボード Q5 #2「1つのドメインの中のモジュールにする」を変えることになる（利用者「正しいディレクトリ構成を優先してください」）</p>
<p class="meta">migration ・ publish ・ search の crate は無くなる。移す ・ 版を出すは Concrete ・ Abstract の型と同じ crate にしか書けない（Rust Reference「An implementing type must be defined within the same crate as the original type definition.」）。探した範囲は term を持つ declaration に置く</p>
<h2>4　2つのレビューが分かれたところと、この答え</h2>
{table(['処理', 'ddd-advisor', '反証役', 'この答え'], [
 ['再開か新規かを決める', '集約「セッション」の操作（読む ・ 変えるのは1つのセッション）', 'ドメインサービス（Q1 #37 が名前で決着。AI ツールの値と設定を合わせて決める）', 'ドメインサービス。決め方は AI ツールと設定の方針で、セッション自身の状態だけでは決まらない。閉じるはセッションのコマンドへ（2人とも同じ）'],
 ['変更を見せる の突き合わせ', '変更のまとまりのファクトリ', 'ドメインサービスに残す（I31 ・ I22 ・ I35）', 'ファクトリ。作るのは保存しない値「変更のまとまり」で、突き合わせはそれを作るときの不変条件'],
 ['意味で探して取り出す', '使う場面 ＋ 値オブジェクト', '条件付きで残す（中核「意味で探す」を持つものを必要とする）', '使う場面 ＋ 値オブジェクト。中核「意味で探す」の決まりは値オブジェクトが持つ'],
 ['Concrete を照らす', '集約 Concrete の操作', 'Validator に入れる', '両方。Concrete の中で閉じる不変条件は根、Abstract に従うかは Validator'],
])}
<h2>5　直すことになる前提</h2>
{table(['前提', '変更前', '変更後'], [
 ['ボード Q1（決着）', '集約9つ ・ ドメインサービス13 ・ 2種類のポート ・ 不変条件46行', '集約9つ ・ ドメインサービス4 ・ ファクトリ3 ・ 2種類のポート ・ 不変条件46行（行は変えず、持ち主だけが移る）'],
 ['model.json', 'domain_services 13行。集約は操作を持たない', 'domain_services 4行。aggregates に commands、factories を新しく足す。value_objects に操作と、担う使う場面の欄を足す（状態を導くが担っていた U0 の行き先）'],
 ['arch.html 4節 ・ 5節', 'crate 9（migration ・ publish ・ search を含む）。validation が Resolver と Validator を持つ', 'crate 7（resolver を分け、migration ・ publish ・ search を無くす）。依存の表とディレクトリを直す'],
 ['ボード Q5 #2（決着）', '解決と検証は役割として分けるが、1つのドメインの中のモジュールにする', '解決と検証は別の crate にする。描画は解決だけを使い、検証を見ない'],
 ['model.json の 再開か新規か', '開いていたものを閉じて「新しく立てる」と返す（1つの操作）', '再開する ID か「新しく立てる（閉じる ID）」を返すだけ。閉じるのは application がセッションのコマンドで行う'],
])}
<h2>6　確かめてほしいこと</h2>
{table(['確かめてほしいこと', '私の見解'], [
 ['ドメインサービスを4つにし、残りを上の表の行き先へ移す', '移す'],
 ['再開か新規かを決めるを、セッションの集約の操作ではなくドメインサービスにする', 'ドメインサービスにする（あなたの見立てと、反証役と同じ）'],
 ['ドメインサービス4つの名前（2節の表）。日本語をカタカナの技術用語にし、Q5 の「解決器 ・ 検証器」も直す。再開か新規かを決めるは Session Selector', 'この名前にする'],
 ['Resolver を validation から分けて resolver の crate にする（Q5 #2 を変える）', '分ける。render の依存が Resolver だけで済む'],
 ['search ・ migration ・ publish の crate を無くす', '無くす。中核「意味で探す」は crate ではなく値オブジェクトと使う場面が担う'],
])}
<p class="meta">承認されたら、ACDR を起こして Q1 と model.json（changes.py に C61 以降）を直し、arch.html の4節 ・ 5節を直す</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
