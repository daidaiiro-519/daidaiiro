"""Semantic Agent Schema の設計（docs/design/index.html）。抽象概念の土台（../foundation）の概念をこの事業に適用した結果を、状態（決まった ・ 案 ・ 候補 ・ 未決）と一緒にタブでまとめる。
図は同じフォルダの <名前>.source.svg（class だけ）を design-svg resolve で <名前>.svg にしたもの。業務領域の名前はユースケースの一覧（../usecases/usecases.json）から引く。この台本は型に図と表を差し込むだけ。"""
import json, os, re
from html import escape as E
H = os.path.dirname(os.path.abspath(__file__))
CSS = '<style>\n/* タブで層 ・ パターン ・ 境界 ・ 単位 ・ 文脈 ・ 六角形 ・ 置き場所 ・ 対応を切り替える */\n:root{--bg:#eef2f0;--card:#fff;--ink:#111d1a;--sub:#5b6b66;--line:#d3dcd8;--accent:#0d5c55;--accent-bg:#d5e6e3;--code:#f3f6f4;--f:"Noto Sans JP","Hiragino Kaku Gothic ProN",system-ui,sans-serif}\n@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--bg:#111614;--card:#192120;--ink:#e3ebe8;--sub:#9aaba5;--line:#2f3b37;--accent:#7cc4b7;--accent-bg:#1e3330;--code:#151c1a;color-scheme:dark}}\n:root[data-theme="dark"]{--bg:#111614;--card:#192120;--ink:#e3ebe8;--sub:#9aaba5;--line:#2f3b37;--accent:#7cc4b7;--accent-bg:#1e3330;--code:#151c1a;color-scheme:dark}\n[hidden]{display:none!important}\nbody{background:var(--bg);color:var(--ink);font:15px/1.75 var(--f);padding-left:max(16px,env(safe-area-inset-left,0px));padding-right:max(16px,env(safe-area-inset-right,0px));padding-block:20px 56px}\nmain{max-width:1240px;margin:0 auto;display:flex;flex-direction:column;gap:14px}\nh1{font-size:24px;margin:0} h2{font-size:18px;margin:0;color:var(--accent)} h3{font-size:15px;margin:8px 0 0} p{margin:0}\n.tabs{display:flex;gap:6px;overflow-x:auto;padding-bottom:4px;position:sticky;top:env(safe-area-inset-top,0px);background:var(--bg);z-index:2;padding-top:6px}\n.tabs button{flex:0 0 auto;font:inherit;font-size:14px;border:1px solid var(--line);background:var(--card);color:var(--ink);border-radius:999px;padding:4px 14px;cursor:pointer}\n.tabs button[aria-selected="true"]{background:var(--accent-bg);border-color:var(--accent);color:var(--accent);font-weight:700}\n.tabs button:focus-visible{outline:2px solid var(--accent);outline-offset:2px}\nsection{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:16px 18px;min-width:0;display:flex;flex-direction:column;gap:12px}\n.ans{border-left:4px solid var(--accent);padding:8px 14px;font-weight:700;max-width:95ch}\n.read{font-size:14px;color:var(--sub);max-width:100ch}\n.fig,.tw{overflow-x:auto}\ntable{border-collapse:collapse;width:100%;font-size:13.5px} th,td{border-bottom:1px solid var(--line);padding:7px 8px;text-align:left;vertical-align:top} th{font-size:12px;color:var(--sub);font-weight:500;white-space:nowrap}\ndetails{border-top:1px solid var(--line);padding-top:8px} summary{cursor:pointer;color:var(--accent);font-size:14px} details[open] summary{margin-bottom:8px}\n.lead{font-size:16px;font-weight:700}\n.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,200px),1fr));gap:8px}\n.card{background:var(--accent-bg);border-radius:8px;padding:8px 12px;display:flex;flex-direction:column;gap:2px;min-width:0} .card b{color:var(--accent);font-size:14px} .card span{font-size:13px}\n.chips{display:flex;flex-wrap:wrap;gap:6px} .chips span{background:var(--code);border:1px solid var(--line);border-radius:999px;padding:2px 12px;font-size:13px}\nol.steps{margin:0;padding-left:1.4em;font-size:14px}\n</style>'
EXTRA = '<style>\n:root{--warn:#8a5a00;--warn-bg:#fbf1dc}\n@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--warn:#f0c56b;--warn-bg:#33291a}}\n:root[data-theme="dark"]{--warn:#f0c56b;--warn-bg:#33291a}\n.where{display:flex;flex-wrap:wrap;gap:8px;align-items:center;font-size:14px;color:var(--sub)}\n.st{display:inline-block;font-size:12px;font-weight:700;border-radius:4px;padding:0 8px;border:1px solid var(--line);white-space:nowrap}\n.st.ok{background:var(--accent-bg);color:var(--accent);border-color:var(--accent)}\n.st.wait{background:var(--warn-bg);color:var(--warn);border-color:var(--warn)}\n.st.cand{background:var(--card);color:var(--ink)}\n.st.open{background:var(--card);color:var(--sub);border-style:dashed}\n.legend{display:flex;flex-wrap:wrap;gap:6px 10px;align-items:center;font-size:13px;color:var(--sub)}\n.legend .st{margin-left:6px}\n.jump{all:unset;cursor:pointer;color:var(--accent);text-decoration:underline;text-underline-offset:3px}\n.jump:focus-visible{outline:2px solid var(--accent);outline-offset:2px}\n.cap{font-size:13px;color:var(--sub)}\nmain a{color:var(--accent)}\n.nw{white-space:nowrap}\n</style>'
SCRIPT = "<script>\n(function(){\n  var bs=[].slice.call(document.querySelectorAll('.tabs button'));\n  function show(k){bs.forEach(function(b){var on=b.id==='t-'+k;b.setAttribute('aria-selected',on?'true':'false');document.getElementById(b.getAttribute('aria-controls')).hidden=!on;});\n    try{localStorage.setItem('design-tab',k)}catch(e){}}\n  bs.forEach(function(b){b.addEventListener('click',function(){show(b.id.slice(2));window.scrollTo({top:0})})});\n  [].forEach.call(document.querySelectorAll('.jump'),function(j){j.addEventListener('click',function(){show(j.dataset.k);window.scrollTo({top:0})})});\n  var h=(location.hash||'').slice(1); var s=null; try{s=localStorage.getItem('design-tab')}catch(e){}\n  var k=document.getElementById('t-'+h)?h:(s&&document.getElementById('t-'+s)?s:null); if(k) show(k);\n})();\n</script>"
VIEWER = '<h3>実際のディレクトリ構成</h3>\n<p class="read">図の形をファイルまで下ろしたもの。左のファイルを押すと、役割 ・ 担うこと ・ コードのイメージが出る。コードは形を伝えるための抜粋で、そのまま動くものではない。いまのコードはまだこの形に移していない（ACDR で移す）。</p>\n<style>\n.lw{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1.1fr);gap:12px;align-items:start}\n@media (max-width:860px){.lw{grid-template-columns:1fr}}\n.lt{font:13px/1.55 ui-monospace,Menlo,monospace;overflow-x:auto;border:1px solid var(--line);border-radius:8px;padding:8px}\n.lt .row{display:flex;gap:8px;align-items:baseline;white-space:nowrap;padding:1px 6px;border-radius:6px}\n.lt button{all:unset;cursor:pointer;color:var(--accent)} .lt button:focus-visible{outline:2px solid var(--accent)}\n.lt .row.on{background:var(--accent-bg)} .lt .dir{font-weight:700} .lt .note{font:12px var(--f);color:var(--sub)}\n.ld{border:1px solid var(--line);border-radius:8px;padding:10px 12px;display:flex;flex-direction:column;gap:6px;min-width:0;position:sticky;top:56px;scroll-margin-top:56px}\n@media (max-width:860px){.ld{position:static}}\n.ld .p{font:12.5px ui-monospace,Menlo,monospace;color:var(--sub);word-break:break-all} .ld b{font-size:16px;color:var(--accent)}\n.ld ul{margin:0;padding-left:1.2em;font-size:14px} .ld .lbl{font-size:12px;color:var(--sub);font-weight:700}\n.ld pre{margin:0;background:var(--code);border-radius:8px;padding:10px 12px;overflow-x:auto;font:12.5px/1.6 ui-monospace,Menlo,monospace}\n</style>\n<div class="lw"><div class="lt" id="lt"></div><div class="ld" id="ld"></div></div>\n<script>\n(function(){\n  var T=__T__, tree=document.getElementById(\'lt\'), det=document.getElementById(\'ld\');\n  function esc(s){return String(s).replace(/[&<>]/g,function(c){return {\'&\':\'&amp;\',\'<\':\'&lt;\',\'>\':\'&gt;\'}[c]})}\n  function show(i){var e=T[i];\n    [].forEach.call(tree.querySelectorAll(\'.row\'),function(r){r.classList.toggle(\'on\',r.dataset.i==String(i))});\n    det.innerHTML=\'<div class="p">\'+esc(e.path)+\'</div><b>\'+esc(e.role)+\'</b><div class="lbl">担うこと</div><ul>\'+e.does.map(function(x){return \'<li>\'+esc(x)+\'</li>\'}).join(\'\')+\'</ul><div class="lbl">コードのイメージ</div><pre>\'+esc(e.code)+\'</pre>\';}\n  var notes={}; T.forEach(function(e){if(e.kind===\'dir\')notes[e.path]=e.note});\n  var seen={};\n  T.forEach(function(e,i){var parts=e.path.replace(/\\/$/,\'\').split(\'/\');\n    for(var k=1;k<=parts.length;k++){var file=(k===parts.length&&e.kind===\'file\');var p=parts.slice(0,k).join(\'/\')+(file?\'\':\'/\');\n      if(seen[p])continue;seen[p]=1;\n      var r=document.createElement(\'div\');r.className=\'row\';r.style.paddingLeft=(6+(k-1)*16)+\'px\';\n      if(!file){r.innerHTML=\'<span class="dir">\'+esc(parts[k-1])+\'/</span>\'+(notes[p]?\'<span class="note">\'+esc(notes[p])+\'</span>\':\'\');}\n      else{r.dataset.i=i;var b=document.createElement(\'button\');b.type=\'button\';b.textContent=parts[k-1];b.onclick=(function(n){return function(){show(n);if(window.matchMedia(\'(max-width:860px)\').matches)det.scrollIntoView({block:\'start\'})}})(i);r.appendChild(b);\n        var t=document.createElement(\'span\');t.className=\'note\';t.textContent=e.role;r.appendChild(t);}\n      tree.appendChild(r);}});\n  var start=0;T.forEach(function(e,i){if(/verify\\.rs$/.test(e.path))start=i});show(start);\n})();\n</script>'
UC = {x['id']: x for x in json.load(open(os.path.join(H, '..', 'usecases', 'usecases.json')))['items']}
F = '../foundation/index.html'

def uc(i):
    return f'{E(UC[i]["goal"])}（{i}）' if i in UC else i

def fig(name):
    return '<div class="fig">' + open(os.path.join(H, name + '.svg')).read() + '</div>'

def tag(state):
    cls = {'決まった': 'ok', '案': 'wait', '候補': 'cand', '未決': 'open', '仮': 'cand'}[state]
    return f'<span class="st {cls}">{state}</span>'

def head(state, where):
    return f'<p class="where">{tag(state)}<span>どこで決めるか：{where}</span></p>'

def table(cols, rows):
    h = ''.join(f'<th>{c}</th>' for c in cols)
    b = ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows)
    return f'<div class="tw"><table><thead><tr>{h}</tr></thead><tbody>{b}</tbody></table></div>'

SIM = 'https://claude.ai/artifact/1cjMpzcdHo4e86TPmgkcMY'
BOARD = 'https://claude.ai/artifact/746VYbbvem7DwcRK5yYgzK'
SCHEMAS = 'https://claude.ai/artifact/MmnqRQksebjiHYS3Cikubc'
ENGINE = 'https://claude.ai/artifact/LoVrCpfdozS5PYCHuoL57x'
def a(url, text):
    return f'<a href="{url}" target="_blank" rel="noopener">{text}</a>'

TABS = []
def tab(key, label, body):
    TABS.append((key, label, body))

# ---- 状態の一覧 ----
STATUS = [
    ('domains', '事業領域 ・ 業務領域 ・ 文脈', '決まった（事業領域） ・ 候補（業務領域） ・ 未決（文脈）', '3つの層', 'p-layers'),
    ('model', 'ドメインモデル', '候補', '13の境界', 'p-boundaries'),
    ('er', 'データ（ER 図）', '案', '3つの層', 'p-layers'),
    ('schemas', 'スキーマの構成', '案', '全体像', 'p-overview'),
    ('engine', 'エンジンの部品', '案', 'ヘキサゴナル', 'p-hexagonal'),
    ('ports', 'ポートの持ち主', '仮', 'ヘキサゴナル', 'p-hexagonal'),
    ('layout', 'ディレクトリ', '仮', 'ディレクトリ', 'p-layout'),
    ('mapping', '基盤との対応', '決まった', '全体像', 'p-overview'),
]
rows = [[f'<button type="button" class="jump" data-k="{k}">{n}</button>', s, f'<a href="{F}#{p[2:]}">{c}</a>'] for k, n, s, c, p in STATUS]
tab('status', '状態の一覧',
    '<h2>状態の一覧</h2><p class="lead">抽象概念の土台の概念を、Semantic Agent Schema に適用した結果。概念そのものは土台にだけ置き、ここでは名前とリンクで指す。</p>'
    '<div class="legend">' + tag('決まった') + 'コミット済み' + tag('案') + '承認待ちのページがある' + tag('候補') + '探索で見つけたが確かめていない' + tag('仮') + '文脈が決まると変わる' + tag('未決') + 'まだ決めていない</div>'
    + table(['中身', '状態', '関係する土台の概念'], rows)
    + '<p class="read">区切られた文脈が決まるまで、ポートの持ち主とディレクトリは仮のまま。文脈は、ガイドのボード（' + a(BOARD, 'ユースケースからドメインモデリングへ') + '）の Q3「業務領域と区切られた文脈」の手順で決める。</p>')

# ---- 1. 事業領域 ・ 業務領域 ・ 文脈 ----
AREAS = [  # 業務領域の候補（探索の手順1の結果。見本）
    ('変更の確かめと例外の判断', 4, '中核（仮）', ['S2', 'S3', 'U13', 'U16', 'U18', 'U25']),
    ('意味の検証', 3, '中核（仮）', ['S1', 'U8', 'U10']),
    ('描画', 3, '補完（仮）', ['U12']),
    ('意味の宣言', 1, 'まだ判定していない', ['U1', 'U2', 'U23', 'U24', 'S4']),
    ('インスタンスの作成と変更', 1, 'まだ判定していない', ['U5', 'U6', 'U11', 'U17']),
    ('目的ごとのセッションの管理', 1, 'まだ判定していない', ['U27', 'N3']),
    ('マイグレーション', 1, 'まだ判定していない', ['U3']),
    ('スキーマの保存と公開', 1, 'まだ判定していない', ['U4']),
    ('AI と外のツールによる判定', 1, 'まだ判定していない', ['U14']),
    ('意味で探す', 1, 'まだ判定していない', ['U26', 'U28']),
]
arows = [[E(n), f'<span class="nw">{p} ／ 4</span>', c, '<br>'.join(uc(u) for u in us)] for n, p, c, us in AREAS]
tab('domains', '事業領域 ・ 業務領域 ・ 文脈',
    '<h2>事業領域 ・ 業務領域 ・ 文脈</h2>' + head('決まった', '事業領域は 2026-10-30 の利用者の決定。業務領域と文脈はガイドの Q3')
    + table(['層', 'Semantic Agent Schema では', '状態'], [
        ['事業領域', 'Semantic Agent Schema（プロダクト全体）', tag('決まった')],
        ['業務領域', '下の10件。区分は仮で、精度4まで書いたのは1件', tag('候補')],
        ['区切られた文脈', 'まだ決めていない。シミュレーションの「（例）検証の文脈」「（例）変更の確かめの文脈」は見せるための例', tag('未決')]])
    + '<p class="read">以前の「基盤の文脈（1つ）」（プロダクト全体を1つの文脈にする）は、2026-10-30 に取り消した。Semantic Agent Schema は事業領域であって、区切られた文脈ではない。</p>'
    + '<h3>業務領域の候補</h3><p class="read">探索の手順1（ユースケースをアクター ・ 外部システム ・ データで束ねる）の結果の見本。業務領域の側がユースケースの一覧を持ち、ユースケースは業務領域の欄を持たない。</p>'
    + table(['業務領域', '精度', '区分', '束ねたユースケース'], arows)
    + '<h3>文脈の境界の手がかり</h3><p class="read">「判定」が2つの意味で使われている。rule の判定（インスタンスと rule の組ごとに1件、コミットするファイル）と、意図の判定（変更の記録ごとに1件、手元の表）。同じ意味か別の意味かを確かめると、意味の検証と変更の確かめのあいだに境界を引くかが決まる。</p>')

# ---- 2. ドメインモデル ----
tab('model', 'ドメインモデル',
    '<h2>ドメインモデル</h2>' + head('候補', 'ガイドの Q4「見つかるものから部品へ」。詳しい頁は ' + a(SIM, 'シミュレーション') + ' の場面5')
    + '<p class="read">ddd-advisor が書いた見本のインスタンスの要点。状態はどれも候補で、文脈が決まるまで確定しない。</p>'
    + '<h3>集約「変更の記録」（変更の確かめと例外の判断）</h3><p class="read">1回の変更について、意図 ・ 目的の参照 ・ 変えた欄を持ち、まとめに入るまで未確定のまま手元に残る記録。</p>'
    + table(['守る条件', '出どころ'], [
        ['I1 意図 ・ 目的の参照 ・ 変えた欄の3つがそろわなければ、記録を作らず理由を返す', '原文（' + uc('S2') + ' の保証）'],
        ['I2 書いた記録は消さない。書いたセッションが error で終わっても残す', '原文（' + uc('U27') + ' ・ ' + uc('N3') + ' の最低限の保証）'],
        ['I3 基盤は、書いた記録の意図を書き換えない', '推測'],
        ['I4 まとめに入った記録は、ほかのまとめに入らない', '推測']])
    + table(['受け付ける指示', '断るとき', '起きたことの通知'], [
        ['変更を記録する', 'I1 を破るとき', '変更を記録した'],
        ['まとめに入れる（名前は推測）', 'I4 を破るとき', 'まとめに入った']])
    + '<p class="read">置かない指示：記録を消す（I2 に反する）、意図を書き換える（I3 に反する）。状態は「未確定」から「まとめに入った」へ進み、戻らない。目的のセッション ・ 変更のまとめ ・ 意図の判定は外に置き、ID で参照する。</p>'
    + '<h3>値オブジェクト「変えた欄」</h3><p class="read">対象（変えたインスタンスの ID）と欄（変えた欄のパス）の2つを持ち、両方が同じなら同じものとみなす。ほかの変更を取り込むときの衝突の判定（' + uc('U18') + '）と、対象ごとに並べる表示（' + uc('U13') + '）に使う。</p>'
    + '<h3>決まっていないこと</h3>' + table(['問い', '決まると変わるもの'], [
        ['「判定」は、rule の判定と意図の判定で別の意味か', '文脈の境界 ・ 判定の記録を値オブジェクトにするか業務イベントにするか'],
        [uc('U13') + ' の主アクター「監査する人」は、監査記録を一貫して残すことを求めるか', '変更の確かめと例外の判断の実装の方法（求めるならイベント履歴式ドメインモデル）'],
        ['I3 は基盤が断る条件か、書き換えを見つけるだけでよいか', '集約「変更の記録」の守る条件']]))

# ---- 3. データ（ER 図） ----
tab('er', 'データ（ER 図）',
    '<h2>データ（ER 図）</h2>' + head('案', a(SCHEMAS, '基盤が用意するスキーマ') + ' の承認')
    + '<p class="read">Abstract の本体（型）から利用者が Concrete（スキーマ）を作り、Concrete からインスタンスを作る。この図は、承認待ちの「基盤が用意するスキーマ」の案を入れた形。SQLite の表は手元の内部の形で、JSON Schema を持たない。</p>'
    + fig('er') + '<p class="cap">矢印の数字は対応の数（N:1 なら、矢印の元が複数で先が1つ）。</p>'
    + '<h3>call から呼ばれるものと、できるもの</h3><p class="read">call は ER 図の要素ではなく、基盤が任せる先を起動するコマンド。できるものは、どれもスキーマに従うインスタンス。決めきれず返す点だけは、インスタンスにならず呼んだ主体へ返る。</p>'
    + fig('call'))

# ---- 4. スキーマの構成 ----
NOW = sorted(os.listdir(os.path.join(H, '..', '..', 'schemas', 'v1')))
PLAN = {'semantic-abstract-schema.json': 'Concrete を書く型 ・ 任せる先の入出力（input-judging ・ input-working を足す）',
        'vocab-meta.schema.json': 'Concrete を書く型（検査のため）', 'dialect.schema.json': 'Concrete を書く型（検査のため）',
        'glossary.schema.json': 'コミットするファイル', 'judgment.schema.json': 'コミットするファイル（意図の判定の枝を外す）',
        'change-summary.schema.json': 'コミットするファイル（range など4つをここへ）',
        'change.schema.json': 'CLI の出力（record ・ history が出す変更の記録の形と定め直す）',
        'call.schema.json': '消す（渡す形は本体の input-judging ・ input-working へ）'}
tab('schemas', 'スキーマの構成',
    '<h2>スキーマの構成</h2>' + head('案', a(SCHEMAS, '基盤が用意するスキーマ') + ' の承認。そのあと ACDR で schemas/v1 とコードへ')
    + '<p class="read">公開する契約は「任せる先の入出力」「コミットするファイル」「CLI の出力」「Concrete を書く型」の4種類だけにする。schemas/v1 は8つから7つにする。</p>'
    + table(['いまの schemas/v1（実物）', '案での扱い'], [[f'<code>{n}</code>', PLAN.get(n, '—')] for n in NOW])
    + fig('compare') + '<p class="cap">点線の箱はスキーマを持たないもの。手元の SQLite の表は、どちらでも表の定義と Rust の型が形の正本。</p>'
    + '<p class="read">あわせて：target を本体の $defs へ移す。history の出力（fields ・ intent_judgment）はどのスキーマにも従っていない不具合なので、change の形にそろえる。ACDR 0201 の「手元の DB の行の形」を改める。</p>')

# ---- 5. エンジンの部品 ----
tab('engine', 'エンジンの部品',
    '<h2>エンジンの部品</h2>' + head('案', a(ENGINE, 'Q3 の3つの答え') + ' の承認。ボード semantic-contract-core の Q3')
    + table(['問い', '案'], [
        ['rule の判定の記録', '集約にせず、書いたら変えない値の記録にする。インスタンスと rule の組で丸ごと置き換える。ポートは「置く」と「引く」の2つ。作るときに、判定した者が rule の by と合うかを確かめる'],
        ['インスタンスと変更の記録を書く順番', 'いまの形のまま。2つは別のコマンドで、片方だけ書けた状態は、コミットの前に変更のまとめが例外として出す'],
        ['call の置き場所', '「AI と外のツールによる判定」の domain に置く。答えを5つの値にまとめる処理は意味の検証に残す']])
    + fig('deps'))

# ---- 6. ポートの持ち主 ----
tab('ports', 'ポートの持ち主',
    '<h2>ポートの持ち主</h2>' + head('仮', '区切られた文脈が決まったあと、ACDR でコードへ移すときに確定')
    + '<p class="read">土台の「ポートは、その会話を受け持つ業務領域の application が持つ」（<a href="' + F + '#hexagonal">ヘキサゴナル</a>）を適用したもの。文脈の切り方で持ち主が変わる。</p>'
    + fig('hexagon')
    + table(['ポート（会話の目的）', '持ち主の業務領域', '使う業務領域', 'アダプター'], [
        ['文書を保管する', '保存と版の管理', 'ほぼすべて', 'fs'],
        ['確定した姿と比べる', '保存と版の管理', '変更の確かめ', 'git'],
        ['変更の記録と承認を残し、たどる', '変更の確かめ', '変更の確かめ', 'sqlite'],
        ['承認された欄を問う', '意味の検証', '意味の検証', 'sqlite'],
        ['rule の判定を残し、使い直す', '意味の検証', '意味の検証', 'fs'],
        ['任せる先に依頼する', 'AI と外のツールによる判定', '意味の検証（のちにセッション）', 'tools ・ coding-agents']]))

# ---- 7. ディレクトリ ----
tab('layout', 'ディレクトリ',
    '<h2>ディレクトリ</h2>' + head('仮', '区切られた文脈が決まったあと、ACDR でコードへ移す')
    + '<p class="read">土台の<a href="' + F + '#layout">ディレクトリ</a>の決まり（業務領域の下に domain と application、外側は inbound と outbound）を適用したもの。文脈が決まると根の置き方が変わる。いまのコードはまだこの形に移していない。</p>'
    + fig('directories')
    + '<details><summary>置き方の決まり（この事業での適用）</summary><ul class="steps"><li>core/&lt;業務領域&gt;/ の下に domain（概念）と application（use_cases ・ ports）の2つの crate。1つにすると業務領域どうしの依存が循環する</li><li>ポートは、その会話を受け持つ業務領域の application に置く。共有の置き場は作らない</li><li>外側は inbound（入口）と outbound（外の技術）。HTTP の API を足すときは inbound に足す</li><li>Cargo の勧め（平らに並べる）から外れることと理由は CODING.md に書き、フォルダと crate 名の一致は検査で守る</li><li>schemas/v1/call.schema.json は、スキーマの案では消す</li></ul></details>'
    + '{{TREE}}')

# ---- 8. 基盤との対応 ----
tab('mapping', '基盤との対応',
    '<h2>基盤との対応</h2>' + head('決まった', 'コミット済み。変えるときは ACDR')
    + '<p class="read">いまの基盤にある語と、土台の概念の対応。</p>' + fig('mapping'))

nav = ''.join(f'<button type="button" role="tab" id="t-{k}" aria-controls="p-{k}" aria-selected="{"true" if i == 0 else "false"}">{l}</button>' for i, (k, l, _) in enumerate(TABS))
panels = ''.join(f'<section role="tabpanel" id="p-{k}" aria-labelledby="t-{k}"{"" if i == 0 else " hidden"}>{b}</section>' for i, (k, l, b) in enumerate(TABS))
tree = json.dumps(json.load(open(os.path.join(H, 'layout.json')))['entries'], ensure_ascii=False).replace('</', '<\\/')
out = ('<title>Semantic Agent Schema の設計</title>\n<meta name="claude:viewport-fit-sides" content="cover">\n' + CSS + EXTRA
       + '\n<main>\n<h1>Semantic Agent Schema の設計</h1>\n<p class="lead">抽象概念の土台（<a href="' + F + '">../foundation</a>）を、この事業に適用した具体。ディレクトリ ・ ドメインモデル ・ データ ・ スキーマを1か所に置く。</p>\n'
       + '<nav class="tabs" role="tablist" aria-label="内容">' + nav + '</nav>\n' + panels + '\n</main>\n' + SCRIPT)
out = out.replace('{{TREE}}', VIEWER.replace('__T__', tree))
open(os.path.join(H, 'index.html'), 'w').write(out)
