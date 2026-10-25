"""既定のビュー3つ（セッション ・ 変更のまとめ ・ 判定）のデザインシステムと完成イメージ。ux-advisor の回答（reviews/ux-advisor.json）に沿う。"""
import os, json
H = os.path.dirname(os.path.abspath(__file__))
flow = open(os.path.join(H, 'flow.svg')).read()
ux = json.load(open(os.path.join(H, 'reviews', 'ux-advisor.json')))
def rows_of(head0):
    for b in ux['design_system']:
        if b['kind'] == 'table' and b['head'][0] == head0: return b['head'], b['rows']
def table(h, rows, cls=''):
    return f'<div class="tw"><table class="{cls}"><thead><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr></thead><tbody>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</tbody></table></div>'
LIGHT = dict(bg='#eef1ef', card='#f8faf9', ink='#16211e', sub='#556561', line='#ccd8d4',
  ok='#2f6528', okbg='#e3f1df', okb='#3e7b36', ng='#a3262a', ngbg='#f6e0df', ngb='#a3262a', wt='#8f5410', wtbg='#f0e2cd', wtb='#8f5410',
  bf='#6e5738', bfbg='#f8f3e8', af='#0d5c55', afbg='#d5e6e3', nextfg='#f8faf9', nextbg='#0d5c55', focus='#0d5c55', link='#0d5c55')
DARK = dict(bg='#0e1614', card='#16211e', ink='#e6eeeb', sub='#9dada8', line='#2c3b37',
  ok='#86c27a', okbg='#1c2b19', okb='#86c27a', ng='#f0a3a0', ngbg='#3a1c1d', ngb='#f0a3a0', wt='#d9a35f', wtbg='#33281a', wtb='#d9a35f',
  bf='#d9b77e', bfbg='#241d14', af='#6cc9b8', afbg='#173330', nextfg='#0e1614', nextbg='#6cc9b8', focus='#6cc9b8', link='#6cc9b8')
V = lambda d: ';'.join(f'--{k}:{v}' for k, v in d.items())
TOK = [('color.surface.page','bg'),('color.surface.card','card'),('color.text','ink'),('color.text.secondary','sub'),('color.surface.line','line'),
 ('color.status.verified','ok'),('color.status.violated','ng'),('color.status.unverified','wt'),('color.diff.before','bf'),('color.diff.after','af'),('color.next','nextbg'),('color.focus','focus')]
def swatches(cls):
    return f'<div class="tile {cls}">' + ''.join(f'<div class="sw"><span class="chip" style="background:var(--{v})"></span><code>{n}</code></div>' for n, v in TOK) + '</div>'
# ---- 部品
OK = '<span class="st ok">✓ verified</span>'; NG = '<span class="st ng">✕ violated</span>'; WT = '<span class="st wt">？ unverified</span>'
BY = lambda w: f'<span class="by">{w}</span>'
def diff(rows):
    return '<div class="tw"><table class="diff"><thead><tr><th>欄</th><th>− 変更前</th><th>＋ 変更後</th></tr></thead><tbody>' + ''.join(f'<tr><td><code>{f}</code></td><td class="b">{b}</td><td class="a">{a}</td></tr>' for f, b, a in rows) + '</tbody></table></div>'
D0199 = diff([('/title','（無い）','依存の向きの検査ツール'),('/decision','（無い）','architecture-lint を作る'),('/why','（無い）','cargo-deny を避け、言語に寄らない形で作る')])
GITD = '<pre class="gd"><span class="m">−    let deps = read_toml(path)?;</span>\n<span class="p">＋    let meta = cargo_metadata(path)?;</span>\n<span class="p">＋    let deps = meta.packages_deps();</span></pre>'
def next_band(text, cmd=None):
    c = f'<div class="cmd"><code>{cmd}</code><button class="copy" data-c="{cmd}">コピー</button></div>' if cmd else ''
    return f'<section class="next" aria-label="次にすること"><h3>次にすること</h3><p>{text}</p>{c}</section>'
def crumb(*parts):
    return '<nav class="crumb" aria-label="現在地">' + ' ＞ '.join(parts) + '</nav>'
S1 = f'''<article class="screen" id="v-s" aria-labelledby="t-s">
{crumb('目的 0199', '<b>セッション S-01</b>（あなた ・ open）')}
<h2 id="t-s">セッション ─ 0199 依存の向きの検査ツール</h2>
<p class="answer"><b>コミットの前に片付けること：2件</b>（意図 I-1 ・ I-2 の承認待ち）。意図に無い変更 1件は戻し済み</p>
{next_band('意図 I-1 と I-2 の変更を確かめて、承認する', 'judge --intent I-1 I-2 --approve')}
<h3>片付けること</h3>
<div class="card"><div class="ch">{WT}<b>承認待ち</b> 意図 I-1　依存の向きを決めた記録を書く</div>
 <p class="meta2">対象 <code>0199</code> decision-record@1（新しく作った）・ 欄 3 ・ 意図の判定 {BY('AI')} pass</p>
 <details><summary>差分を開く（欄 3）</summary>{D0199}</details></div>
<div class="card"><div class="ch">{WT}<b>承認待ち</b> 意図 I-2　cargo metadata から依存を読む</div>
 <p class="meta2">対象 <code>architecture-lint/src/cargo.rs</code> ・ 意図の判定 {BY('AI')} pass</p>
 <details><summary>差分を開く（Git の差分）</summary>{GITD}</details></div>
<details class="fold"><summary>戻し済み（1件）</summary>
 <div class="card"><div class="ch">{NG}<b>戻した</b> 意図 I-1 に無い変更</div>
 <p class="meta2">対象 <code>0185</code> の <code>/decision</code> ・ 判定の理由：意図（0199 を書く）に無い記録の決定を変えている ・ 同じセッションの中で戻した（差し引き 0）</p></div></details>
<details class="fold"><summary>セッションの木（子 2 ・ 開いている 1）</summary>
 <ul class="tree"><li><b>S-01</b> あなた ・ open<ul><li><b>S-02</b> 実装 ・ 引き継ぐ ・ open ・ 10:38</li><li><b>S-03</b> レビュー ・ 引き継がない ・ closed ・ 09:55</li></ul></li></ul>
 <p class="meta2">単発の子は記録しない（反証 2 ・ 調べもの 1）</p></details>
<details class="fold"><summary>このセッションの欄</summary>{table(['欄','値'], [['tool ・ id','<code>claude-code</code> ・ <code>5790ed9c-…</code>'],['purpose_ref','0199'],['parent','（無い）'],['role ・ context','（無い。子だけが持つ）'],['state ・ last_used_at','open ・ 10:42']])}</details>
</article>'''
S2 = f'''<article class="screen" id="v-c" aria-labelledby="t-c" hidden>
{crumb('目的 0199', '<b>コミット 3f2a9c1</b>（前回 8b01d4e から）')}
<h2 id="t-c">変更のまとめ ─ 0199 依存の向きの検査ツール</h2>
<p class="answer"><b>意図 2件とも意図どおり</b>（AI pass ・ 人 承認）。例外 1件は同じ変更の中で戻した</p>
{next_band('例外 1件を確かめてから、この PR を承認する')}
<section class="exc" aria-labelledby="t-exc"><h3 id="t-exc">！ 例外（1件）</h3>
 <ul><li><code>0185</code> の <code>/decision</code> を意図に無く変えた → 同じ変更の中で戻した（差し引き 0）
 <details><summary>判定の理由を見る</summary><p>意図 I-1（0199 を書く）に無い記録の決定を変えている。判定した者 {BY('AI')}</p></details></li></ul></section>
<h3>意図ごとの変更</h3>
<div class="card"><div class="ch">{OK}<b>I-1</b> 依存の向きを決めた記録を書く</div>
 <p class="meta2">判定 {BY('AI')} pass ・ {BY('人')} 承認 ・ 対象 <code>0199</code> decision-record@1（新しく作った）・ 欄 <code>/title</code> <code>/decision</code> <code>/why</code></p>
 <details><summary>差分を開く（欄 3）</summary>{D0199}</details>
 <details><summary>判定の理由を開く</summary><p>意味「0199 に決定と理由を書いた」・ 理由「変えた欄が意図の範囲（0199）に収まる」</p></details></div>
<div class="card"><div class="ch">{OK}<b>I-2</b> cargo metadata から依存を読む</div>
 <p class="meta2">判定 {BY('AI')} pass ・ {BY('人')} 承認 ・ 対象 <code>architecture-lint/src/cargo.rs</code></p>
 <details><summary>差分を開く（Git の差分）</summary>{GITD}</details>
 <details><summary>判定の理由を開く</summary><p>意味「依存の読み取りを cargo metadata に替えた」・ 理由「変えたのは読み取りの関数だけ」</p></details></div>
<h3>突き合わせ</h3>
<p>記録に無い書き換え 0件 ・ 書き換えの無い記録 0件 ・ 状態が変わったインスタンス：<code>0199</code>（無い → {OK}）<a href="#" data-go="j" class="go">0199 の判定を見る</a></p>
<details class="fold"><summary>この変更の id と前後のコミット</summary><p><code>changes/01JB7….json</code> ・ 3f2a9c1 ← 8b01d4e</p></details>
</article>'''
S3 = f'''<article class="screen" id="v-j" aria-labelledby="t-j" hidden>
{crumb('目的 0199', '<a href="#" data-go="c" class="go">コミット 3f2a9c1</a>', '<b>0199</b>', 'いまの姿 7c3e210')}
<h2 id="t-j">判定 ─ 0199 decision-record@1</h2>
<p class="answer">{WT}<b>rule 1件の取り直しが要る</b>　✓ 4 ・ ✕ 0 ・ ？ 1</p>
{next_band('owner-format を版 4 で取り直す', 'semantic-judge 0199 owner-format')}
<h3>要対応</h3>
<div class="card" id="row-owner-format"><div class="ch">{WT}<b>owner-format</b>（版 4）</div>
 <p class="meta2">判定した値 <code>/owner</code> = maintainer daidaiiro</p>
 <p>理由：rule を版 3 → 4 に変えたので、版 3 の pass は効かない（前回 {BY('AI')} pass ・ 版 3）</p>
 <details><summary>rule の文（版 4）を開く</summary><p>責任者を role と名前で書く</p></details></div>
<details class="fold"><summary>✓ verified（4件）</summary>
{table(['rule（版）','結果','判定した者','根拠'], [['形（JSON Schema）', OK, BY('機械'), '—'],['owner-required', OK, BY('機械'), 'owner がある'],['why-explains-decision（3）', OK, BY('AI'), '理由が、決定の手段と比べた案を述べている'],['decision-is-actionable（3）', OK, BY('AI'), '作るものが名前で決まっている']])}</details>
<p class="meta2"><a href="#" data-go="c" class="go">この判定が出てくる変更のまとめへ戻る</a></p>
</article>'''
h_who, r_who = rows_of('見る人'); h_fold, r_fold = rows_of('ビュー'); h_parts, r_parts = rows_of('部品'); h_chk, r_chk = rows_of('原則'); h_ask, r_ask = rows_of('原則で決まらないこと')
GALLERY = f'''<div class="gal">
<div class="gi"><h4>答えの帯</h4><p class="answer"><b>コミットの前に片付けること：2件</b></p></div>
<div class="gi"><h4>次にすることの帯</h4>{next_band('owner-format を版 4 で取り直す', 'semantic-judge 0199 owner-format')}</div>
<div class="gi"><h4>状態の札</h4><p>{OK} {NG} {WT}</p></div>
<div class="gi"><h4>判定した者の札</h4><p>{BY('機械')} {BY('AI')} {BY('人')}</p></div>
<div class="gi"><h4>例外の一覧</h4><section class="exc"><h3>！ 例外（1件）</h3><ul><li><code>0185</code> の <code>/decision</code> を意図に無く変えた</li></ul></section></div>
<div class="gi"><h4>差分の表示</h4>{diff([('/why','（無い）','cargo-deny を避け…')])}</div>
<div class="gi"><h4>意図のカード</h4><div class="card"><div class="ch">{OK}<b>I-2</b> cargo metadata から依存を読む</div><p class="meta2">判定 {BY('AI')} pass ・ {BY('人')} 承認</p></div></div>
<div class="gi"><h4>現在地の行 ・ 畳み</h4>{crumb('目的 0199','<b>0199</b>')}<details class="fold"><summary>✓ verified（4件）</summary><p>…</p></details></div>
</div>'''
CSS = f'''
:root{{{V(LIGHT)};--mono:ui-monospace,"SF Mono",Menlo,Consolas,monospace;--font:"Noto Sans JP","Hiragino Kaku Gothic ProN",system-ui,sans-serif;--r:10px;color-scheme:light}}
@media (prefers-color-scheme:dark){{:root:not([data-theme="light"]){{{V(DARK)};color-scheme:dark}}}}
:root[data-theme="dark"]{{{V(DARK)};color-scheme:dark}}
.t-light{{{V(LIGHT)}}} .t-dark{{{V(DARK)}}}
body{{background:var(--bg);color:var(--ink);font:15px/1.7 var(--font);margin:0;padding-block:20px 48px;padding-inline:16px}}
main{{max-width:1000px;margin:0 auto;display:flex;flex-direction:column;gap:16px}}
main>*{{min-width:0}}
h1{{font-size:24px;margin:0;text-wrap:balance}} h2{{font-size:19px;margin:8px 0 0;text-wrap:balance}} h3{{font-size:15px;margin:6px 0 0}} h4{{font-size:13px;margin:0;color:var(--sub)}}
a{{color:var(--link)}} :focus-visible{{outline:2px solid var(--focus);outline-offset:2px}}
code{{font-family:var(--mono);font-size:.9em}}
.lead{{margin:0;padding:12px 14px;border-left:4px solid var(--nextbg);background:var(--card);border-radius:6px;font-weight:700}}
.tw{{overflow-x:auto}} table{{border-collapse:collapse;width:100%;font-size:13.5px;background:var(--card)}} th,td{{border:1px solid var(--line);padding:6px 8px;text-align:left;vertical-align:top}} th{{white-space:nowrap}}
.fig{{background:var(--card);border:1px solid var(--line);border-radius:var(--r);padding:10px;overflow-x:auto}}
/* 部品 */
.answer{{margin:0;padding:10px 14px;background:var(--card);border:1px solid var(--line);border-radius:var(--r);font-size:15px}}
.next{{background:var(--nextbg);color:var(--nextfg);border-radius:var(--r);padding:10px 14px;display:flex;flex-direction:column;gap:6px}}
.next h3{{margin:0;font-size:12px;letter-spacing:.06em}} .next p{{margin:0;font-weight:700}}
.cmd{{display:flex;gap:8px;align-items:center;flex-wrap:wrap}} .cmd code{{background:var(--card);color:var(--ink);border-radius:6px;padding:2px 8px;overflow-wrap:anywhere}}
.copy{{font:inherit;font-size:12px;border:1px solid var(--nextfg);background:transparent;color:var(--nextfg);border-radius:999px;padding:1px 10px;cursor:pointer}}
.st{{display:inline-block;font-size:12px;font-weight:700;border-radius:999px;padding:0 9px;border:2px solid;white-space:nowrap}}
.st.ok{{color:var(--ok);background:var(--okbg);border-color:var(--okb)}} .st.ng{{color:var(--ng);background:var(--ngbg);border-color:var(--ngb)}} .st.wt{{color:var(--wt);background:var(--wtbg);border-color:var(--wtb)}}
.by{{display:inline-block;font-size:12px;border:1px solid var(--sub);color:var(--ink);border-radius:6px;padding:0 6px}}
.exc{{border:2px solid var(--ngb);background:var(--ngbg);border-radius:var(--r);padding:10px 14px}} .exc h3{{margin:0;color:var(--ng)}} .exc ul{{margin:6px 0 0;padding-left:1.2em}}
.card{{background:var(--card);border:1px solid var(--line);border-radius:var(--r);padding:10px 14px;display:flex;flex-direction:column;gap:4px}}
.card.flash{{outline:3px solid var(--focus)}}
.ch{{display:flex;gap:8px;align-items:center;flex-wrap:wrap}}
.meta2{{margin:0;font-size:13px;color:var(--sub)}} .card p{{margin:0}}
details>summary{{cursor:pointer;color:var(--link);font-size:13.5px}} details[open]>summary{{margin-bottom:6px}}
.fold{{background:var(--card);border:1px solid var(--line);border-radius:var(--r);padding:8px 14px}}
table.diff td.b{{color:var(--bf);background:var(--bfbg)}} table.diff td.a{{color:var(--af);background:var(--afbg);font-weight:700}}
pre.gd{{margin:0;font-family:var(--mono);font-size:12.5px;white-space:pre-wrap;overflow-wrap:anywhere;background:var(--card);border:1px solid var(--line);border-radius:6px;padding:6px 8px}}
pre.gd .m{{color:var(--bf);background:var(--bfbg)}} pre.gd .p{{color:var(--af);background:var(--afbg);font-weight:700}}
.tree{{margin:0;padding-left:1.2em}} .tree ul{{padding-left:1.2em}}
.crumb{{font-size:13px;color:var(--sub)}}
/* 頁の枠 */
.tabs{{display:flex;gap:6px;flex-wrap:wrap}} .tabs button{{font:inherit;font-size:13.5px;padding:5px 14px;border-radius:999px;border:1px solid var(--line);background:var(--card);color:var(--ink);cursor:pointer}}
.tabs button[aria-selected="true"]{{background:var(--nextbg);color:var(--nextfg);border-color:var(--nextbg)}}
.screen{{border:1px solid var(--line);border-radius:14px;background:var(--bg);padding:14px;display:flex;flex-direction:column;gap:10px;box-shadow:0 1px 0 var(--line)}}
.tile{{display:grid;grid-template-columns:repeat(auto-fill,minmax(210px,1fr));gap:6px;background:var(--bg);color:var(--ink);border:1px solid var(--line);border-radius:var(--r);padding:10px}}
.sw{{display:flex;gap:8px;align-items:center;font-size:12px}} .chip{{width:28px;height:20px;border-radius:5px;border:1px solid var(--line);flex:none}}
.tiles{{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,420px),1fr));gap:10px}}
.gal{{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,300px),1fr));gap:10px}} .gi{{display:flex;flex-direction:column;gap:6px;min-width:0}} .gi p{{margin:0}}
'''
body = f'''<title>既定のビューのデザインシステム</title>
<style>{CSS}</style>
<main>
<h1>既定のビューのデザインシステムと完成イメージ</h1>
<p class="lead">3つのビューは、どれも「答えの帯 → 次にすることの帯 → 要対応 → 畳んだ残り」の順で、見る人の問いに最初の画面で答え、次の行動をコマンドで渡す。状態は記号 ・ 語 ・ 色の3つで示し、例外 ・ 判定 ・ まとめはリンクで行き来する</p>

<h2>1　見る人 ・ 問い ・ 次の行動</h2>
{table(h_who, r_who)}

<h2>2　動線</h2>
<div class="fig">{flow}</div>
<p class="meta2">入口は2つ：PR からは変更のまとめ、作業者の CLI からはセッション。静的な HTML なので、操作は「次にすることの帯」のコマンドで渡す</p>

<h2>3　完成イメージ（例：目的 0199）</h2>
<p class="meta2">リンク（「0199 の判定を見る」「コミット 3f2a9c1」など）を押すと、ほかのビューへ移る。畳んだところは押すと開く</p>
<div class="tabs" role="tablist"><button role="tab" aria-selected="true" data-v="s">① セッション（作業者）</button><button role="tab" aria-selected="false" data-v="c">② 変更のまとめ（PR を見る人）</button><button role="tab" aria-selected="false" data-v="j">③ 判定</button></div>
{S1}{S2}{S3}

<h2>4　1画面目に出すもの ・ 畳むもの</h2>
{table(h_fold, r_fold)}

<h2>5　デザインシステム</h2>
<h3>トークン（明るい地 ・ 暗い地）</h3>
<div class="tiles">{swatches('t-light')}{swatches('t-dark')}</div>
<p class="meta2">色を使うのは状態 ・ 例外 ・ 差分 ・ 次にすることの4か所だけ。差分の色（砂色 ・ 青緑）は状態の色（緑 ・ 赤）と分ける。文字は 4.5:1 以上、状態の枠は 3:1 以上を明暗とも満たす（WCAG 1.4.3 ・ 1.4.11）</p>
<h3>部品</h3>
{GALLERY}
{table(h_parts, r_parts)}

<h2>6　原則での点検</h2>
{table(h_chk, r_chk)}
<h2>7　利用者で確かめること</h2>
{table(h_ask, r_ask)}
<p class="meta2">根拠：ux-advisor の回答（reviews/ux-advisor.json。18F ・ Nielsen ・ WCAG 2.2 ・ GOV.UK ・ Design Tokens の判断基準23件）。例のデータは変更のまとめのシミュレーションと同じ</p>
</main>
<script>
const tabs=[...document.querySelectorAll('.tabs button')];
function show(v,anchor){{tabs.forEach(b=>b.setAttribute('aria-selected',b.dataset.v===v));
 document.querySelectorAll('.screen').forEach(s=>s.hidden=s.id!=='v-'+v);
 const t=anchor?document.getElementById(anchor):document.getElementById('v-'+v);
 if(t){{t.scrollIntoView({{behavior:'smooth',block:'start'}});if(anchor){{t.classList.add('flash');setTimeout(()=>t.classList.remove('flash'),1600)}}}}}}
tabs.forEach(b=>b.onclick=()=>show(b.dataset.v));
document.querySelectorAll('a.go').forEach(a=>a.onclick=e=>{{e.preventDefault();show(a.dataset.go,a.dataset.go==='j'?'row-owner-format':null)}});
document.querySelectorAll('.copy').forEach(b=>b.onclick=()=>{{const c=b.dataset.c;try{{navigator.clipboard.writeText(c).then(()=>b.textContent='コピーした',()=>{{b.textContent='選んでコピー'}})}}catch(e){{b.textContent='選んでコピー'}}}});
</script>'''
body = body.replace('<title> は', '&lt;title&gt; は').replace('札', 'バッジ').replace('記号で見分けられる', '記号で区別できる').replace('取り直しが要る', '取り直しが必要')
open(os.path.join(H, 'index.html'), 'w').write(body); print('ok')
