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
ddl = '<pre>' + html.escape("""-- 手元の DB（SQLite）。確定しない。同じリポジトリのどの作業ディレクトリからも1つ
CREATE TABLE sessions (
  tool         TEXT NOT NULL,
  id           TEXT NOT NULL,            -- AI ツールが発行した値。基盤は解釈しない
  purpose_ref  TEXT,                     -- 目的の参照（オーケストレーターは持たない）
  parent_tool  TEXT, parent_id TEXT,     -- 親（任意）
  role         TEXT,                     -- 役割。継続の子だけ
  context      TEXT CHECK (context IN ('fresh','inherited')),  -- 親の文脈を引き継ぐか。継続の子だけ
  state        TEXT NOT NULL CHECK (state IN ('open','closed')),
  last_used_at TEXT NOT NULL,
  PRIMARY KEY (tool, id),
  CHECK ((role IS NULL) = (context IS NULL))
);
CREATE UNIQUE INDEX one_open_delegate_session ON sessions (parent_tool, parent_id, role) WHERE state = 'open' AND role IS NOT NULL;
-- 単発の子と、判定させる AI の呼び出しは登録しない

CREATE TABLE changes (                   -- 変更の記録
  id                 TEXT PRIMARY KEY,   -- 打ち合わせなしで一意な値
  purpose_ref        TEXT NOT NULL,
  intent             TEXT NOT NULL,      -- 意図（1件に1つ）
  writer_kind        TEXT NOT NULL CHECK (writer_kind IN ('human','agent')),
  replaces_intent_id TEXT,
  summary_id         TEXT,               -- 入ったまとめ（まだなら NULL）。1件は1つのまとめにだけ入る
  created_at         TEXT NOT NULL
);                                       -- セッション ID は持たない
CREATE TABLE change_items (              -- 変更の項目。前後の値は持たない（変更の仕組みから取る）
  change_id TEXT NOT NULL REFERENCES changes(id),
  target    TEXT NOT NULL,               -- kind と id、またはパス
  pointer   TEXT NOT NULL DEFAULT '',    -- 欄の JSON Pointer（ファイル全体は空）
  PRIMARY KEY (change_id, target, pointer)
);
CREATE TABLE intent_judgments (          -- 意図と変更が合うか（I33）・ 人の承認。上書きで最後の1件
  change_id  TEXT PRIMARY KEY REFERENCES changes(id),
  judge_kind TEXT NOT NULL CHECK (judge_kind IN ('ai','human')),
  result     TEXT NOT NULL CHECK (result IN ('verified','violated','unverified')),
  meaning    TEXT NOT NULL,              -- 意味
  reason     TEXT NOT NULL               -- 判定の理由
);
CREATE TABLE session_changes (           -- 手元のセッションの索引
  tool TEXT NOT NULL, id TEXT NOT NULL, change_id TEXT NOT NULL REFERENCES changes(id),
  PRIMARY KEY (tool, id, change_id)
);
CREATE TABLE relations (                 -- 関係の索引。確定したファイルから作り直せる
  from_id TEXT NOT NULL, to_id TEXT NOT NULL, via TEXT NOT NULL,
  PRIMARY KEY (from_id, to_id, via)
);
-- 消す表は無い。不要なインスタンスは superseded にして残す""") + '</pre>'
FILES = [
 ['abstract.json（本体）', 'Concrete の形 ・ 9つの x- ・ 拡張の契約', 'Concrete が $schema に書く'],
 ['change', '変更の記録の形 ・ 変更のまとまり（$defs）', '手元の DB の行の形'],
 ['change-summary', '変更のまとめ：目的 ・ 意図 ・ 対象と欄 ・ 例外 ・ 意図の判定の結果', 'コミットごとに1件。基盤が作る'],
 ['judgment', '判定の記録（rule の判定と意図の判定）', 'rule の判定はコミット、意図の判定は手元の DB'],
 ['session', 'セッションの7つの欄（context を含む）', '手元の DB の行の形'],
 ['procedure', 'セマンティックのコマンドが返す手順 ・ 入力のスキーマ ・ 出力のスキーマ', '$defs に操作ごとの入力（read-request ・ create-request ・ import-request …）'],
 ['query', '探す条件（where ・ select ・ 範囲）・ 結果 ・ $defs/provenance（出どころ）', 'semantic-search と search'],
 ['settings', '探す範囲 ・ validator ・ 設定の値', '文脈の設定'],
 ['view', 'ビュー（描く kind は Concrete か Abstract のファイル）', 'views/ に既定のビュー（session ・ change-summary ・ judgment）'],
 ['catalog ・ design-tokens', 'デザインシステム', '描く前の検査（I26）'],
 ['glossary', '文脈の言葉', ''],
 ['published', '公開された言葉の版', 'ほかの文脈が x-relationship で読む'],
]
crates = diff([('', 'semantic-agent-schema/'), ('', '  Cargo.toml ・ CODING.md ・ architecture.json（architecture-lint の規則）'),
 ('', '  schemas/v1/       Abstract の13ファイル ・ views/（既定のビュー）'), ('', '  test-suite/       システムテストの場面（入力と期待結果の JSON）'),
 ('', '  crates/'), ('', '    domain/'),
 ('', '      declaration/   集約 Semantic Abstract Schema ・ Semantic Concrete Schema ・ glossary、ファクトリ Concrete の雛形、値 procedure'),
 ('', '      instance/      集約 インスタンス、ファクトリ インスタンス（作る ・ 取り込む）'),
 ('', '      change/        集約 判定の記録 ・ 変更の記録、ファクトリ 変更のまとめ、値 検証結果 ・ 状態'),
 ('', '      resolver/      ドメインサービス Semantic Schema Resolver'),
 ('', '      validation/    ドメインサービス Semantic Schema Validator'),
 ('', '      render/        集約 ビュー ・ デザインシステム、ドメインサービス Semantic Schema Renderer'),
 ('', '      search/        値 探す条件 ・ 出どころ ・ 探した範囲'),
 ('', '      session/       集約 セッション、ドメインサービス Session Selector、値 子のセッションの型'),
 ('', '    application/     使う場面と ports/（駆動される側13）'),
 ('', '    outbound/        agents-contract ・ ai-cli ・ git ・ fs ・ tools ・ sqlite'),
 ('', '    inbound/cli/     入口（セマンティック7つ ・ 機械的なコマンド）')])
body = f'''<main>
<h1>決めたことの全体</h1>
<p class="meta">ACDR 0201 で、正本（ボード ・ model.json ・ アーキテクチャの頁）に適用した内容の1枚。作業ツリーに適用済みで、承認を待っている。詳しい中身は0節の各頁にある</p>
<p class="ans">ドメインサービスは4つ、domain の crate は8つ。AI が行う操作はすべてセマンティックのコマンド（7つ）で、procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）を返す。変更の記録と意図の判定は手元の DB、変更のまとめと rule の判定はコミットする。消すは持たない。Abstract のファイルは13個で、基盤の既定のビューを持つ</p>
<h2>0　詳しい頁</h2>
{table(['話', '頁'], [
 ['正本への適用（変更前と変更後）', L('TEX6vLna45nG38koC7Tfc1', 'ACDR 0201')],
 ['アーキテクチャ（依存の向き ・ ディレクトリ ・ 検査）', L('YcHWX4NkaJCXPCgZR5iQaN', 'アーキテクチャ')],
 ['ドメインサービス（4つ）・ 集約の操作 ・ ファクトリ', L('LD2wfboxHLbFP6KvybezwE', 'ドメインサービス')],
 ['ポートの置き場所', L('7hZXTxh89Ce8Cs4uJnHQAu', 'ポートの置き場所')],
 ['コマンド（セマンティック7つ ・ procedure ・ 検査2回）', L('BcTSrcczNGLWeezwUitexr', 'コマンドのシミュレーション')],
 ['Semantic Read と Write', L('7rJkrCboyJgp4MBGwZnvUq', 'Semantic Read と Write')],
 ['Semantic Search と用語', L('J4e6mDbKwUaBoDsmSHjHeV', 'Semantic Search と用語')],
 ['変更の記録とまとめ', L('GZ2Xz4eaBms5tZev9muEAC', '変更のまとめのシミュレーション')],
 ['子のセッションの型', L('Bxykci4FLw5JaERWa9gisu', '子のセッションの型')],
 ['テスト計画', L('3SeEdiiBwRmYLbrZAa699T', 'テスト計画')],
 ['依存の向きの検査ツール', L('PYFjdKyGYcatEdBuSFy9ET', 'ACDR 0200 architecture-lint')],
])}
<h2>1　ドメインモデル</h2>
{table(['', '数', '中身'], [
 ['集約', '9', 'Semantic Abstract Schema（版を出す ・ 移し方を出す）・ Semantic Concrete Schema（照らす ・ 移す）・ glossary ・ インスタンス（変える ・ 置き換えて残す）・ 判定の記録（使い直すか）・ 変更の記録 ・ ビュー ・ デザインシステム（描く前の検査）・ セッション（立てる ・ 閉じる）'],
 ['ドメインサービス', '4', 'Semantic Schema Resolver（セマンティックスキーマリゾルバー）・ Semantic Schema Validator（セマンティックスキーマバリデーター。2段：依頼の一覧を返す ・ 答えをまとめる）・ Semantic Schema Renderer（セマンティックスキーマレンダラー）・ Session Selector（セッションセレクター）'],
 ['ファクトリ', '3', 'Concrete の雛形 ・ インスタンス（作る ・ 取り込む）・ 変更のまとめ'],
 ['値オブジェクト', '32', '足したもの：探す条件 ・ 出どころ ・ 子のセッションの型 ・ procedure'],
 ['使う場面', '23', 'U28 Semantic Search を足した。U11 は「不要なインスタンスを置き換えて残す」'],
 ['不変条件', '46', 'I11（superseded の判定は効かない）・ I26（既定のビュー）・ I28 ・ I31 ・ I34 ・ I35 ・ I40（記録の置き場所）を書き換えた'],
])}
<h2>2　コマンド</h2>
{table(['種類', 'コマンド', '返すもの'], [
 ['セマンティック（AI が行う操作）', 'semantic-search ・ semantic-read ・ semantic-create ・ semantic-write ・ semantic-judge ・ semantic-import ・ semantic-migrate', 'procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）。AI は入力だけを作る'],
 ['機械的（入力を適用する）', 'search ・ get ・ create ・ patch ・ import ・ migrate ・ validate', '出力の型（query の結果 ・ 値 ・ change ・ 検証の結果）'],
 ['機械的（AI が行わない操作）', 'judge（人）・ render ・ history ・ record ・ release ・ session', 'judgment ・ HTML ・ 経緯 ・ 変更の記録 ・ 版 ・ セッション'],
])}
<p class="meta">検査は2回 ── ①適用する前に入力が入力のスキーマに合うか（外れたら断り、状態に触れない）②適用した後に出力が形 ・ 機械の rule ・ AI の rule に合うか（外れても保存し violated、AI の rule は needs で返る）。AI の判定は semantic-judge が依頼を返し、validate が答えをまとめる。基盤は AI を起動しない</p>
<h2>3　セッション</h2>
{quad}
{table(['', '中身'], [
 ['種類', 'オーケストレーター ・ 目的 ・ 任せる先。任せる先は「寿命（単発 ・ 継続）」×「親の文脈（引き継ぐ ・ 引き継がない）」の4つの型。単発は登録しない'],
 ['AI ツールのアダプター', '再開できるか ・ 有効時間 ・ 履歴の割合 ・ 型ごとの手段（fork ・ 親のセッション自身 ・ sessions import ・ session/fork）・ キャッシュが効くか'],
 ['目的を引き継ぐとき', '引き継ぐ前にコミットする（手元の DB はほかの人に見えない）'],
])}
<h2>4　DB（手元の SQLite）と ER</h2>
{fig('er.svg')}
{ddl}
<h2>5　Abstract のファイル（13）</h2>
<p class="meta">置き場所は semantic-agent-schema/schemas/v1/。$id は urn:semantic-agent-schema:base:…:v1。基盤の既定のビューは schemas/v1/views/</p>
{table(['ファイル', '中身', '使われ方'], FILES)}
<h2>5-2　変更の記録とまとめ</h2>
{table(['もの', '置き場所', '読む人'], [
 ['変更の記録 ・ 意図の判定', '手元の DB', '作業者（セッションの中で I33 を確かめる）'],
 ['変更のまとめ（change-summary）', 'コミット（コミットごとに1件。前回のコミットからの差し引きと例外。基盤が作り、AI は書かない）', 'PR を見る人 ・ 経緯をたどる人。見せ方は既定のビューか、文脈の Concrete のビュー'],
 ['rule の判定の記録', 'コミット', 'ほかの人の手元と CI（状態を導く）'],
 ['確定していない変更の取り消し', 'Git の側', '基盤は戻った姿を validate するだけ'],
])}
<h2>6　ディレクトリ構成と依存の向き</h2>
{crates}
{arch_svg}
<h2>7　テスト</h2>
{table(['レベル', '中身'], [
 ['単体（最も厚い）', 'domain の値の関数 ・ 集約の操作 ・ ファクトリ（決定表 ・ 状態遷移）'],
 ['コンポーネント統合', 'application（ポートをテストダブルに）・ outbound（本物を一時ディレクトリで）・ schemas/v1'],
 ['システム', 'バイナリをコマンドで動かす。test-suite/ の JSON'],
 ['静的', 'architecture-lint（許す辺）・ clippy の forbid ・ crate ごとの単独ビルド'],
])}
<h2>8　次にすること</h2>
{table(['順', 'すること'], [
 ['1', 'ACDR 0201 の承認を受けて、状態を accepted にしてコミットする'],
 ['2', 'semantic-agent-schema/ の Rust の雛形と CODING.md を、このアーキテクチャから作る'],
 ['3', 'model.json に状態を導く決定表の列を足し、単体テストから書く'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
