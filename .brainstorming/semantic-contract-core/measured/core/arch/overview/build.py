"""決めたことの全体の頁（arch/overview/index.html）。ドメインモデル ・ ドメインサービス ・ 使う場面とポート ・ セッション ・ Abstract のファイル ・ DB ・ ディレクトリ ・ テストを1枚にまとめ、詳しい頁へつなぐ。"""
import os, html
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.fig{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px;overflow-x:auto}.links a{color:var(--accent)}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>決めたことの全体</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
def diff(lines):
    return '<pre>' + ''.join(f'<span class="{c}">{html.escape(t)}</span>' for c, t in lines) + '</pre>'
fig = lambda n, d=H: '<div class="fig">' + open(os.path.join(d, n)).read() + '</div>'
A = 'https://claude.ai/artifact/'
L = lambda i, t: f'<a href="{A}{i}">{t}</a>'
quad = fig('quad.svg', os.path.join(os.path.dirname(H), 'child-sessions'))
arch_svg = fig('arch.svg', os.path.join(C, 'view'))
ddl = diff([('', 'CREATE TABLE sessions ('), ('', '  …'), ('', '  role         TEXT,   -- 役割。継続の子だけが持つ'),
 ('a', "  context      TEXT CHECK (context IN ('fresh','inherited')),  -- 親の文脈を引き継ぐか。継続の子だけが持つ"),
 ('', "  state        TEXT NOT NULL CHECK (state IN ('open','closed')),"), ('', '  …'),
 ('a', '  CHECK ((role IS NULL) = (context IS NULL)),'), ('', '  …'), ('', ');'),
 ('a', '-- 単発（引き継ぐ ・ 引き継がない）と判定させる AI の呼び出しは登録しない'),
 ('', 'CREATE UNIQUE INDEX one_open_delegate_session ON sessions (parent_tool, parent_id, role) WHERE state = \'open\' AND role IS NOT NULL;  -- 変えない')])
schemas = diff([('', 'semantic-agent-schema/schemas/v1/'), ('', '  abstract.json（本体）・ change ・ judgment ・ session ・ settings ・ view ・ catalog ・ design-tokens ・ glossary ・ published'),
 ('a', '  query.schema.json（新しい11個目）：探す条件（kind ・ term ・ JMESPath の where と select ・ contexts）・ 探した結果と範囲 ・ $defs/provenance（出どころ）'),
 ('a', '  session.schema.json に context（fresh ・ inherited）を足す'),
 ('a', '  abstract.json の $defs/prompt の x-prompt に、semantic-* が返す手順を書く欄（手順 ・ 使ってよいコマンド）')])
crates = diff([('', 'semantic-agent-schema/'), ('', '  Cargo.toml ・ CODING.md ・ architecture.json（architecture-lint の規則）'),
 ('', '  schemas/v1/（公開された言葉。上の11ファイル）'), ('a', '  test-suite/（システムテストの約21場面。入力と期待結果の JSON）'),
 ('', '  crates/'), ('', '    domain/'), ('', '      declaration/   集約 Abstract ・ Concrete ・ glossary、ファクトリ Concrete の雛形'),
 ('', '      instance/      集約 インスタンス、ファクトリ インスタンス（作る ・ 取り込む）'),
 ('', '      change/        集約 判定の記録 ・ 変更の記録、ファクトリ 変更のまとまり、値 検証結果 ・ 状態'),
 ('a', '      resolver/      ドメインサービス Semantic Schema Resolver（validation から分けた）'),
 ('', '      validation/    ドメインサービス Semantic Schema Validator'),
 ('', '      render/        集約 ビュー ・ デザインシステム、ドメインサービス Semantic Schema Renderer'),
 ('', '      search/        値 探す条件 ・ 一致の決まり ・ 探した範囲 ・ 出どころ'),
 ('', '      session/       集約 セッション、ドメインサービス Session Selector'),
 ('d', '      migration/  publish/   （無くす。移す ・ 版を出すは集約の操作）'),
 ('', '    application/     使う場面と ports/（駆動される側12）'),
 ('', '    outbound/        agents-contract ・ ai-cli ・ git ・ fs ・ tools ・ sqlite'),
 ('', '    inbound/cli/     入口（semantic-* と search ・ get ・ patch ・ validate のコマンド）')])
body = f'''<main>
<h1>決めたことの全体</h1>
<p class="meta">アーキテクチャ ・ ドメインサービス ・ Semantic 系 ・ セッションの話で決めたことを1枚にまとめた。詳しい中身は各頁にある。まだ正本（ボード ・ model.json ・ Abstract のファイル）には適用していない。承認されたら、この頁を変更前と変更後として ACDR にまとめて適用する</p>
<p class="ans">ドメインサービスは4つ、domain の crate は8つ。基盤のコマンドは「手順を返すセマンティック系」と「処理する機械的な系」に分かれる。子のセッションは「単発か継続か」×「親の文脈を引き継ぐか」で4つの型になり、4つの AI ツールすべてで引き継げる。Abstract のファイルは11個になり、DB の表はセッションに1列足す</p>
<h2>0　詳しい頁</h2>
{table(['話', '頁', '状態'], [
 ['アーキテクチャ（依存の向き ・ ディレクトリ ・ 検査）', L('YcHWX4NkaJCXPCgZR5iQaN', 'アーキテクチャ'), '承認済みの部分を反映済み。ドメインサービスの分け直しはまだ'],
 ['ポートの置き場所', L('7hZXTxh89Ce8Cs4uJnHQAu', 'ポートの置き場所'), '承認済み'],
 ['ドメインサービスの分け直し（13 → 4）', L('LD2wfboxHLbFP6KvybezwE', 'ドメインサービスの分け直し'), '承認済み'],
 ['Semantic Read と Write', L('7rJkrCboyJgp4MBGwZnvUq', 'Semantic Read と Write'), '承認済み'],
 ['Semantic Search と用語', L('J4e6mDbKwUaBoDsmSHjHeV', 'Semantic Search と用語'), '承認済み'],
 ['子のセッションの型', L('Bxykci4FLw5JaERWa9gisu', '子のセッションの型'), '承認済み'],
 ['テスト計画', L('3SeEdiiBwRmYLbrZAa699T', 'テスト計画'), '承認済み'],
 ['依存の向きの検査ツール', L('PYFjdKyGYcatEdBuSFy9ET', 'ACDR 0200 architecture-lint'), '作った（ACDR 0199 ・ 0200 承認済み）'],
])}
<h2>1　ドメインモデル（変更前 → 変更後）</h2>
{table(['', '変更前', '変更後'], [
 ['集約', '9つ', '9つ（変えない）。操作（コマンド）を持たせる：Concrete（照らす ・ 移す ・ 公開する部分の版）・ Abstract（版を出す ・ 移し方）・ 判定の記録（使い直すか）・ ビュー ・ デザインシステム（描く前の検査）・ セッション（立てる ・ 閉じる）'],
 ['ドメインサービス', '13', '4つ：Semantic Schema Resolver ・ Semantic Schema Validator（2段：依頼の一覧を返す ・ 答えをまとめる）・ Semantic Schema Renderer ・ Session Selector'],
 ['ファクトリ', '無い', '3つ：Concrete の雛形 ・ インスタンス（作る ・ 取り込む）・ 変更のまとまり'],
 ['値オブジェクト', '28', '足す：探す条件（JMESPath の where ・ select）・ 出どころ（id ・ kind ・ 版 ・ where ・ select）・ 子のセッションの型（単発 ・ 継続 × 引き継ぐ ・ 引き継がない）。検証結果 ・ インスタンスの状態に操作を持たせる。値オブジェクトにも担う使う場面の欄を足す'],
 ['使う場面', 'U0〜U27', 'U28 Semantic Search を足す'],
 ['不変条件', '46行', '持ち主が集約 ・ 値オブジェクトへ移る。行は変えない。出どころの rule（範囲の中 ・ 写した値の一致 ・ 版）を足す'],
])}
<h2>2　使う場面とコマンド</h2>
{table(['種類', 'コマンド', '中身'], [
 ['セマンティック（手順を返す）', 'semantic-read ・ semantic-write ・ semantic-search', 'AI がする作業の手順（x-prompt ・ 書き方の rule ・ 使ってよいコマンド ・ 条件と答えのスキーマ）を返す。処理はしない'],
 ['機械的（処理する）', 'search ・ get ・ patch ・ validate', '探す（JMESPath）・ 欄を読む ・ JSON Patch を適用する ・ 渡した値を保存せずに検証する'],
 ['そのほか（変えない）', '作成 ・ 削除 ・ 取り込む ・ 判定する（人だけ）・ 描画する ・ 版を出す ・ 移す ・ 経緯 ・ セッション ・ 変更を記録する', '──'],
])}
<p class="meta">基盤は AI を起動しない。AI の rule の判定は needs として返し、呼ぶ側が依頼する。繰り返すのは AI で、基盤は1回ずつ答える</p>
<h2>3　セッション</h2>
{quad}
{table(['', '変更前', '変更後'], [
 ['種類', 'オーケストレーター ・ 目的 ・ 任せる先', '任せる先に「寿命（単発 ・ 継続）」と「親の文脈（引き継ぐ ・ 引き継がない）」を持たせる。単発は登録しない'],
 ['AI ツールのアダプター', '再開できるか ・ 有効時間 ・ 履歴の割合', '加えて、型ごとの手段（fork ・ 親のセッション自身 ・ sessions import ・ session/fork）と、キャッシュが効くか'],
])}
<h2>4　DB（手元の SQLite）と ER</h2>
{fig('er.svg')}
{ddl}
<h2>5　Abstract のファイル（10 → 11）</h2>
{schemas}
<h2>6　ディレクトリ構成と依存の向き</h2>
{crates}
{fig('crates.svg')}
<p class="meta">domain の外側（inbound → outbound → application → domain）はアーキテクチャの頁の図のまま。ACDR の適用で、アーキテクチャの頁の図もこの形に直す</p>
<h2>7　テスト</h2>
{table(['レベル', '中身'], [
 ['単体（最も厚い）', 'domain の値の関数 ・ 集約の操作 ・ ファクトリ（決定表 ・ 状態遷移）'],
 ['コンポーネント統合', 'application（ポートをテストダブルに）・ outbound（本物を一時ディレクトリで）・ schemas/v1'],
 ['システム（約21場面）', 'バイナリを protocol のコマンドで動かす。test-suite/ の JSON'],
 ['静的', 'architecture-lint（許す辺）・ clippy の forbid ・ crate ごとの単独ビルド'],
])}
<h2>8　次にすること</h2>
{table(['順', 'すること'], [
 ['1', 'この頁を1件の ACDR にまとめる（ボードの Q1 ・ Q3 ・ Q5 ・ Q8 ・ Q9 ・ Q11、model.json、Abstract のファイル、DB の DDL、アーキテクチャの頁）'],
 ['2', '承認を受けて正本に適用し、核の試験（参照実装の場面）を通す'],
 ['3', 'semantic-agent-schema/ の Rust の雛形と CODING.md を、このアーキテクチャから作る'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
