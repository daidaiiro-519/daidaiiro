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
 ['作成する', 'ファクトリ', 'Concrete の雛形（Abstract から）は declaration、インスタンス（Concrete の雛形から）は instance のファクトリ。AI は semantic-create の procedure に従う', '集約を1つ丸ごと作る。Evans「Create an entire aggregate as a piece, enforcing its invariants.」'],
 ['Concrete を照らす', '集約の操作 ＋ Validator', 'Concrete の中で閉じる不変条件（I4 rule の id が一意）は集約 Concrete の根。Abstract に従うか（I3 ・ I6）は、Abstract を schema として Concrete を検証する Validator', 'Concrete は Abstract のインスタンスでもある（Q1 #6）ので、従うかの検査は検証と同じ処理'],
 ['Semantic Schema Resolver', SV, 'resolver（自分の crate）', 'Concrete ・ glossary ・ 公開された言葉の版 ・ 指す先のインスタンスを読み、どの集約にも属さない'],
 ['Semantic Schema Validator', SV, 'validation。2段で動く ── ①任せる先への依頼の一覧（呼び出し call）を値で返す ②semantic-judge で AI が返した答えを5つの値にまとめる（I14 は値オブジェクト「検証結果」）。使い直すか（I16）は集約「判定の記録」', 'rule × インスタンスの組で決まり、どの集約にも属さない。入出力をしないので、AI を呼ぶのは使う場面'],
 ['状態を導く', '値オブジェクトの操作', '「インスタンスの状態」を検証結果の並びから決定表で作る（I19）', 'Evans「moving complex logic into value objects when a concept fitting the responsibility presents itself」'],
 ['変更のまとめを作る', 'ファクトリ ＋ 使う場面', '変更のまとめ（change-summary）を、手元の変更の記録から作るファクトリは change。コミットの前に、前回のコミットからの差し引きと例外を機械的に作り、記録と変更の前後を両方向で突き合わせる（I31 ・ I35）。AI は書かない', '保存する値を1つ丸ごと作るので、不変条件はそれを作るときに守る。呼ぶ順は application'],
 ['例外をまとめる', '値オブジェクトの操作', '検証結果の並びを原因ごとにまとめる（I23）', '1種類の値をまとめる計算'],
 ['意味で探して取り出す', '使う場面 ＋ 値オブジェクト', '探すのは application（リポジトリへの問い合わせ）。探す条件（JMESPath の where ・ select）・ 出どころ ・ 一致の決まり（I25 ・ I41）・ 探した範囲（I42）は search の crate の値オブジェクト。AI は semantic-search の procedure に従う', 'Evans「Provide methods that select objects based on criteria meaningful to domain experts.」（Repositories）'],
 ['Semantic Schema Renderer', SV, 'render。描く前の検査（I26）は集約「ビュー」と「デザインシステム」。Abstract のファイル（session ・ change-summary ・ judgment）は基盤が同梱した既定のビューで描き、文脈の Concrete のビューがあればそちらで描く', 'ビューが無い kind も既定の描き方で描くので、集約「ビュー」の操作にできない'],
 ['移す', '集約の操作', '集約 Concrete のコマンド。機械で直せる欄は直し、残りは semantic-migrate の procedure で AI が直して patch で適用する。移し方は Abstract の版が持つ（I27）', '変える集約は Concrete 1つ'],
 ['取り込む', 'ファクトリ ＋ 使う場面', 'インスタンスのファクトリ（作成する と同じ入口）。$schema が1つに決まらないファイルは、semantic-import の procedure で AI が Concrete を選ぶ。ファイルを読む ・ 下書きの一覧は application', 'インスタンスを作る入口を1つにする'],
 ['公開された言葉の版を出す', '集約の操作', '集約 Concrete と Abstract のコマンド（出した版は変えない I1 ・ I7）', '各集約自身の不変条件'],
 ['再開か新規かを決める', SV, 'Session Selector（session）。「再開する ID」か「新しく立てる（閉じる ID）」を返すだけで、閉じるのは集約「セッション」のコマンド', 'AI ツールの答え ・ 設定 ・ 時刻を合わせて決める方針で、セッション自身の状態だけでは決まらない'],
]
after = '''crates/domain/
  declaration/  urn.rs ・ target.rs ・ procedure.rs（手順 ・ 入力のスキーマ ・ 出力のスキーマ）
                aggregates/ abstract_schema.rs（版を出す ・ 移し方）・ concrete_schema.rs（照らす ・ 移す ・ 公開する部分の版）・ glossary.rs
                factories/ concrete_template.rs（Abstract から Concrete の雛形）
  instance/     aggregates/ instance.rs（変える ・ 置き換えて残す）
                factories/ instance_factory.rs（雛形から作る ・ 取り込む）
  change/       verdict.rs（検証結果）・ instance_state.rs（状態の4つ）・ change_id.rs
                aggregates/ judgment.rs（使い直すか）・ change_record.rs
                factories/ change_summary.rs（変更のまとめ）
  resolver/     services/ resolver.rs
  validation/   call.rs（依頼の一覧）
                services/ validator.rs
  render/       aggregates/ view.rs ・ design_system.rs（描く前の検査）
                services/ renderer.rs
  search/       query.rs（探す条件）・ provenance.rs（出どころ）・ scope.rs（探した範囲）
  session/      session_id.rs ・ child_kind.rs（子のセッションの型）
                aggregates/ session.rs（立てる ・ 閉じる）
                services/ selector.rs（Session Selector）'''
def pre(text, cls):
    return '<pre class="v diff">' + ''.join(f'<span class="{cls}">{l}</span>' for l in text.split('\n')) + '</pre>'
body = f'''<main>
<h1>ドメインサービスの分け直し</h1>
<p class="meta">利用者の見立て「ドメインサービスではなく、エンティティや値オブジェクト、集約でやるべき処理がほぼほぼ」「再開か新規かを決めるも…ドメインサービスだと思うから４つか」を、ddd-advisor（arch/reviews/ddd-services.html）と反証役に通して決めた。ACDR 0201 で model.json ・ ボード Q1 ・ Q5 ・ arch.html に入れた</p>
<p class="ans">ドメインサービスは4つ（Resolver ・ Validator ・ Renderer ・ Session Selector）。ほかの処理は、ファクトリ3 ・ 集約の操作 ・ 値オブジェクトの操作 ・ 使う場面が持つ。AI が行う操作は、使う場面がセマンティックのコマンドとして procedure を返す</p>
<h2>1　処理ごとの持ち主</h2>
{table(['処理', '種類', 'どこが持つか', '理由'], rows)}
<h2>2　ドメインサービス4つの名前（英語と日本語の対）</h2>
{table(['英語（型の名前）', '日本語', 'ファイル'], [
 ['Semantic Schema Resolver（SemanticSchemaResolver）', 'セマンティックスキーマリゾルバー', 'resolver/src/services/resolver.rs'],
 ['Semantic Schema Validator（SemanticSchemaValidator）', 'セマンティックスキーマバリデーター', 'validation/src/services/validator.rs'],
 ['Semantic Schema Renderer（SemanticSchemaRenderer）', 'セマンティックスキーマレンダラー', 'render/src/services/renderer.rs'],
 ['Session Selector（SessionSelector）', 'セッションセレクター', 'session/src/services/selector.rs'],
])}
{table(['決めたこと', '理由'], [
 ['日本語は英語の技術用語をそのままカタカナにする', 'Q5 の旧い訳語（比喩の語）を使わず、技術用語に戻す'],
 ['再開か新規かを決めるを Session Selector と呼ぶ', '開いているセッションを再開するか新しく立てるかを選ぶ役割の名詞。「判定」は判定の記録の語と重なるので使わない'],
])}
<h2>3　domain の中のディレクトリ</h2>
{pre(after, '')}
<h3>依存の向き</h3>
{table(['crate', '依存してよい先', '理由'], [
 ['declaration', '（無し）', 'Abstract ・ Concrete ・ glossary と、Abstract が決める値の型（procedure を含む）'],
 ['instance', 'declaration', 'インスタンスは Concrete に従う'],
 ['change', 'declaration', '判定の記録 ・ 変更の記録 ・ 変更のまとめ ・ 検証結果 ・ 状態。対象の型は declaration'],
 ['resolver', 'instance ・ declaration', '$ref ・ glossary ・ 公開された言葉の版を解決し、x-relationship の指す先のインスタンスをたどる'],
 ['validation', 'resolver ・ change ・ instance ・ declaration', 'インスタンスを検証する。解決は resolver に任せ、判定の記録の使い直しと I33 に change の型を使う'],
 ['render', 'resolver ・ change ・ instance ・ declaration', 'インスタンスを状態と rule ごとの結果と一緒に描く。Resolver を使い（Q5 #1）、Validator は見ない'],
 ['search', 'change ・ instance ・ declaration', '探す条件 ・ 出どころ ・ 一致の決まり（I25 ・ I41）・ 探した範囲（I42）。判定の結果と状態（I24）は change の型'],
 ['session', '（無し）', 'セッションと Session Selector。渡された値だけで決まる'],
])}
<p class="meta">migration ・ publish の crate は持たない。移す ・ 版を出すは Concrete ・ Abstract の型と同じ crate にしか書けない（Rust Reference「An implementing type must be defined within the same crate as the original type definition.」）。search は中核なので crate を持つ</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
