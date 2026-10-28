"""ユースケースの一覧（docs/usecases/index.html）。usecases.json から、業務領域ごとの図（areas.source.svg → design-svg resolve → areas.svg）とページを組む。"""
import html, json, os, re

H = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(H, '../../..'))
doc = json.load(open(os.path.join(H, 'usecases.json')))
items = doc['items']
E = html.escape

# カード：業務領域ごとに、誰が何をするかを言葉で並べる（記号と識別子は表にだけ出す）
KIND = {'summary': ('まとめの目的', 'k-sum'), 'user-goal': ('利用者の目的', 'k-goal'), 'subfunction': ('サブ機能', 'k-sub')}
byid = {e['id']: e for e in items}
def short(g):
    return re.sub(r'（.*?）', '', g).strip()
def card(e, extra=''):
    k, c = KIND[e['level']]
    if e['id'].startswith('N'):
        k, c = '新しく足す案', 'k-new'
    who = '' if e['level'] == 'subfunction' else f'<span class="who">{E(e["actor"] or "")}</span>'
    uses = ''
    if e['level'] != 'subfunction':
        subs = [x for x in items if x['level'] == 'subfunction' and e['id'] in x.get('called_by', [])]
        if subs:
            uses = '<div class="uses">使うサブ機能：' + '・'.join(E(short(x['goal'])) for x in subs) + '</div>'
    ext = ''.join(f'<div class="ext">{E(x["when"])}とき → {E(x["then"])}</div>' for x in e.get('extensions', []))
    return f'<div class="uc {c}"><span class="kind">{k}</span>{who}<b>{E(short(e["goal"]))}</b>{uses}{ext}{extra}</div>'
order = ['意味の宣言', '意味の検証', '変更の確かめと例外の判断', '意味で探す', '目的ごとのセッションの管理',
         'instance の作成と変更', 'マイグレーション', 'スキーマの保存と公開', '描画', 'AI と外のツールによる判定']
import json as _j
area_kind = _j.load(open(os.path.join(REPO, '.brainstorming/semantic-contract-core/measured/core/model/model.json')))['subdomains']
u0 = byid['U0']
top = f'<div class="uc k-sum top"><span class="kind">いちばん外側のまとめの目的</span><span class="who">{E(u0["actor"])}</span><b>{E(u0["goal"])}</b><div class="uses">下の利用者の目的をくり返し果たすことで達する</div></div>'
blocks = []
for a in order:
    es = [e for e in items if e['area'] == a and e['level'] != 'subfunction']
    if not es:
        continue
    cs = ''
    for e in es:
        extra = ''
        if e.get('steps'):
            extra = '<div class="uses">中の手順：' + '・'.join(E(x) for x in e['steps']) + '</div>'
        cs += card(e, extra)
    blocks.append(f'<section class="area"><h2>{E(a)}<span class="cat">{E(area_kind.get(a, ""))}</span></h2><div class="grid">{cs}</div></section>')
subs = [e for e in items if e['level'] == 'subfunction']
subcards = ''.join(
    f'<div class="uc k-sub"><span class="kind">サブ機能</span><b>{E(short(e["goal"]))}</b><div class="uses">使う目的：' + '・'.join(E(short(byid[c]['goal'])) for c in e.get('called_by', []) if c in byid) + '</div></div>' for e in subs)
def cards(es):
    return '<div class="cards">' + ''.join(
        f'<div class="card"><b>{E(short(e["goal"]))}</b><span>{E(e["why"])}</span></div>' for e in es) + '</div>'
pending = [dict(id=r['id'], goal=r['goal'], why=r['why']) for r in doc.get('removed', [])]
rows = ''.join(
    f'<tr><td>{E(e["id"])}</td><td>{E(KIND[e["level"]][0])}</td><td>{E(e["goal"])}</td><td>{E(e["actor"] or "")}</td>'
    f'<td>{E(e["area"] or "—")}</td><td>{E(e["scope"])}</td><td>{E(e["status"])}</td><td>{E(e["why"])}</td></tr>' for e in items)
page = f'''<title>ユースケースの一覧</title>
<meta name="claude:viewport-fit-sides" content="cover">
<style>
/* 結論 → 業務領域ごとの図 → 要判断 → 新規の案 → 全件の表（折りたたみ） */
:root{{--bg:#eef2f0;--card:#fff;--ink:#111d1a;--sub:#5b6b66;--line:#d3dcd8;--accent:#0d5c55;--abg:#d5e6e3;--q:#8f5410;--qb:#f0e2cd;--sbg:#dce7f1;--sfg:#2f5b80;--f:"Noto Sans JP","Hiragino Kaku Gothic ProN",system-ui,sans-serif}}
@media (prefers-color-scheme:dark){{:root:not([data-theme="light"]){{--bg:#111614;--card:#192120;--ink:#e3ebe8;--sub:#9aaba5;--line:#2f3b37;--accent:#7cc4b7;--abg:#1e3330;--q:#f0cf94;--qb:#3a2d17;--sbg:#1d2b3b;--sfg:#9dc1e6;color-scheme:dark}}}}
:root[data-theme="dark"]{{--bg:#111614;--card:#192120;--ink:#e3ebe8;--sub:#9aaba5;--line:#2f3b37;--accent:#7cc4b7;--abg:#1e3330;--q:#f0cf94;--qb:#3a2d17;--sbg:#1d2b3b;--sfg:#9dc1e6;color-scheme:dark}}
[hidden]{{display:none!important}}
body{{background:var(--bg);color:var(--ink);font:15px/1.75 var(--f);padding-left:max(16px,env(safe-area-inset-left,0px));padding-right:max(16px,env(safe-area-inset-right,0px));padding-block:24px 56px}}
main{{max-width:1160px;margin:0 auto;display:flex;flex-direction:column;gap:16px}}
h1{{font-size:24px;margin:0}} h2{{font-size:17px;margin:0;color:var(--accent)}} p{{margin:0}}
.lead{{font-weight:700;font-size:16px}}
section{{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:14px 16px;min-width:0;display:flex;flex-direction:column;gap:10px}}
.fig,.tw{{overflow-x:auto}}
.cards{{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,300px),1fr));gap:8px}}
.card{{background:var(--qb);border-radius:8px;padding:10px 12px;display:flex;flex-direction:column;gap:4px;min-width:0}} .card b{{color:var(--q);font-size:14px}} .card span{{font-size:13px}}
.new .card{{background:var(--abg)}} .new .card b{{color:var(--accent)}}
.legend{{display:flex;flex-wrap:wrap;gap:8px}} .legend span{{border-radius:999px;padding:2px 12px;font-size:13px}}
.area h2{{display:flex;gap:10px;align-items:baseline}} .cat{{font-size:12px;color:var(--sub);font-weight:400}}
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(min(100%,260px),1fr));gap:10px}}
.uc{{border-radius:10px;padding:10px 12px;display:flex;flex-direction:column;gap:4px;min-width:0;border:1px solid var(--line)}}
.uc b{{font-size:15px;line-height:1.5}} .uc .kind{{font-size:11.5px;font-weight:700}} .uc .who{{font-size:12.5px;color:var(--sub)}}
.uc .uses,.uc .ext{{font-size:12.5px;color:var(--sub)}} .uc .ext{{border-top:1px dashed var(--line);padding-top:4px}}
.k-sum{{background:var(--abg);color:var(--ink)}} .k-sum .kind,.legend .k-sum{{color:var(--accent)}} .legend .k-sum{{background:var(--abg)}}
.k-goal{{background:var(--card)}} .k-goal .kind{{color:var(--ink)}} .legend .k-goal{{background:var(--card);border:1px solid var(--line)}}
.k-sub{{background:var(--sbg)}} .k-sub .kind,.legend .k-sub{{color:var(--sfg)}} .legend .k-sub{{background:var(--sbg)}}
.k-new{{background:var(--qb)}} .k-new .kind,.legend .k-new{{color:var(--q)}} .legend .k-new{{background:var(--qb)}}
.top b{{font-size:17px}}
details summary{{cursor:pointer;color:var(--accent);font-size:14px}}
table{{border-collapse:collapse;width:100%;font-size:13px}} th,td{{border-bottom:1px solid var(--line);padding:6px 8px;text-align:left;vertical-align:top}} th{{font-size:12px;color:var(--sub);font-weight:500;white-space:nowrap}}
</style>
<main>
<h1>ユースケースの一覧</h1>
<p class="lead">業務領域ごとに、誰が何をするかを並べた案。いちばん外側のまとめの目的の下に、利用者の目的と、いくつもの目的から使うサブ機能がある。</p>
{top}
<div class="legend"><span class="k-sum">まとめの目的</span><span class="k-goal">利用者の目的</span><span class="k-sub">サブ機能</span><span class="k-new">新しく足す案</span></div>
{''.join(blocks)}
<section class="area"><h2>サブ機能<span class="cat">いくつもの目的から使う手順</span></h2><div class="grid">{subcards}</div></section>
<section><h2>外したもの（{len(pending)}件）</h2>{cards(pending)}</section>
<section><details><summary>識別子つきの全件の表（{len(items)}件）</summary><div class="tw"><table><thead><tr><th>id</th><th>レベル</th><th>目的</th><th>主体</th><th>業務領域</th><th>設計の範囲</th><th>扱い</th><th>なぜ</th></tr></thead><tbody>{rows}</tbody></table></div></details></section>
</main>
'''
open(os.path.join(H, 'index.html'), 'w').write(page)
