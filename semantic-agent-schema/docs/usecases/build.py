"""ユースケースの一覧（docs/usecases/index.html）を usecases.json から組む。
件数が増えても探せるように、検索 ・ 絞り込み（業務領域 ・ 種類 ・ 誰が）・ いまの条件と件数 ・ 業務領域ごとのカードか行 ・ 1件の詳細（呼ぶ ・ 呼ばれるものを押してたどれる。#U8 の形で直接開ける）を持つ。
ユースケースを足すときは usecases.json の items に1件足して、このファイルを走らせる。"""
import json, os

H = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(H, '../../..'))
doc = json.load(open(os.path.join(H, 'usecases.json')))
kinds = json.load(open(os.path.join(REPO, '.brainstorming/semantic-contract-core/measured/core/model/model.json')))['subdomains']
data = json.dumps({'items': doc['items'], 'removed': doc.get('removed', []), 'areaKind': kinds}, ensure_ascii=False).replace('</', '<\\/')

PAGE = r'''<title>ユースケースの一覧</title>
<meta name="claude:viewport-fit-sides" content="cover">
<style>
/* 上：いちばん外側の目的と件数 → 検索と絞り込み → いまの条件と件数 → 業務領域ごとのカードか行。押すと右（スマホは全面）に詳細 */
:root{--bg:#eef2f0;--card:#fff;--ink:#111d1a;--sub:#5b6b66;--line:#d3dcd8;--accent:#0d5c55;--on:#fff;--abg:#d5e6e3;--sfg:#2f5b80;--nfg:#8f5410;--sel:#fff6d6;--f:"Noto Sans JP","Hiragino Kaku Gothic ProN",system-ui,sans-serif}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--bg:#111614;--card:#192120;--ink:#e3ebe8;--sub:#9aaba5;--line:#2f3b37;--accent:#7cc4b7;--on:#0f1a18;--abg:#1e3330;--sfg:#9dc1e6;--nfg:#f0cf94;--sel:#33301a;color-scheme:dark}}
:root[data-theme="dark"]{--bg:#111614;--card:#192120;--ink:#e3ebe8;--sub:#9aaba5;--line:#2f3b37;--accent:#7cc4b7;--on:#0f1a18;--abg:#1e3330;--sfg:#9dc1e6;--nfg:#f0cf94;--sel:#33301a;color-scheme:dark}
[hidden]{display:none!important}
*{box-sizing:border-box}
body{background:var(--bg);color:var(--ink);font:15px/1.7 var(--f);padding-left:max(16px,env(safe-area-inset-left,0px));padding-right:max(16px,env(safe-area-inset-right,0px));padding-block:20px 56px;margin:0}
main{max-width:1240px;margin:0 auto;display:flex;flex-direction:column;gap:12px}
h1{font-size:24px;margin:0;text-wrap:balance} p{margin:0}
button{font:inherit;color:inherit}
button:focus-visible,input:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
.top{all:unset;cursor:pointer;background:var(--abg);border-radius:12px;padding:14px 18px;display:flex;flex-direction:column;gap:2px}
.top:focus-visible{outline:2px solid var(--accent)}
.top .k{font-size:12px;font-weight:700;color:var(--accent);letter-spacing:.04em} .top b{font-size:18px} .top .s{font-size:13px;color:var(--sub)}
.counts{font-size:13px;color:var(--sub)} .counts b{color:var(--ink);font-variant-numeric:tabular-nums}
.bar{position:sticky;top:env(safe-area-inset-top,0px);z-index:3;background:var(--bg);padding:8px 0;display:flex;flex-direction:column;gap:6px;border-bottom:1px solid var(--line)}
@media (max-width:900px){.bar{position:static}}
.search{width:100%;font:inherit;padding:8px 12px;border:1px solid var(--line);border-radius:8px;background:var(--card);color:var(--ink)}
.filters{display:flex;flex-wrap:wrap;gap:6px;align-items:center}
.filters .lbl{font-size:12px;color:var(--sub);min-width:4.5em}
.chip{font-size:13px;border:1px solid var(--line);background:var(--card);border-radius:999px;padding:1px 12px;cursor:pointer}
.chip[aria-pressed="true"]{background:var(--accent);color:var(--on);border-color:var(--accent)}
details.more summary{font-size:12.5px;color:var(--accent);cursor:pointer}
details.more[open] summary{margin-bottom:6px}
.state{display:flex;flex-wrap:wrap;gap:6px 10px;align-items:center;font-size:13.5px}
#state{display:flex;flex-wrap:wrap;gap:6px;align-items:center}
#state .n{font-weight:700;font-variant-numeric:tabular-nums}
.cond{font-size:12.5px;border:1px solid var(--accent);color:var(--accent);background:var(--card);border-radius:999px;padding:0 10px;cursor:pointer}
.view{margin-left:auto;display:flex;border:1px solid var(--line);border-radius:8px;overflow:hidden}
.view button{border:none;background:var(--card);padding:2px 12px;cursor:pointer;font-size:13px}
.view button[aria-pressed="true"]{background:var(--accent);color:var(--on)}
.layout{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,420px);gap:14px;align-items:start}
@media (max-width:900px){.layout{grid-template-columns:1fr}}
#list{display:flex;flex-direction:column;gap:16px;min-width:0}
.area{display:flex;flex-direction:column;gap:8px}
.area h2{font-size:16px;margin:0;color:var(--accent);display:flex;flex-wrap:wrap;gap:4px 8px;align-items:baseline}
.area h2 small{font-size:12px;color:var(--sub);font-weight:400}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(min(100%,250px),1fr));gap:8px}
.uc{all:unset;box-sizing:border-box;cursor:pointer;display:flex;flex-direction:column;gap:2px;border:1px solid var(--line);border-left-width:4px;border-radius:10px;padding:10px 12px;background:var(--card)}
.uc:hover{border-color:var(--accent)} .uc:focus-visible{outline:2px solid var(--accent)}
.uc[aria-current="true"]{background:var(--sel);border-color:var(--accent)}
.uc .kind{font-size:11.5px;font-weight:700;letter-spacing:.03em;color:var(--sub)} .uc .who{font-size:12.5px;color:var(--sub)} .uc b{font-size:15px;line-height:1.5}
.k-summary{border-left-color:var(--accent)} .k-summary .kind{color:var(--accent)}
.k-subfunction{border-left-color:var(--sfg)} .k-subfunction .kind{color:var(--sfg)}
.k-new{border-left-style:dashed;border-left-color:var(--nfg)} .k-new .kind{color:var(--nfg)}
.tw{overflow-x:auto}
table{border-collapse:collapse;width:100%;font-size:13.5px;background:var(--card)}
th,td{border-bottom:1px solid var(--line);padding:6px 8px;text-align:left;vertical-align:top}
th{font-size:12px;color:var(--sub);font-weight:500;white-space:nowrap}
td:nth-child(1){min-width:12em} td:nth-child(2){white-space:nowrap} td:nth-child(3){min-width:7em} td:nth-child(4){min-width:18em}
tr.row{cursor:pointer} tr.row:hover td,tr.row[aria-current="true"] td{background:var(--sel)}
.uc-row{all:unset;cursor:pointer;color:var(--accent);font-weight:700}
.uc-row:focus-visible{outline:2px solid var(--accent)}
.empty{color:var(--sub);font-size:14px;padding:12px;background:var(--card);border-radius:10px}
.detail{position:sticky;top:200px;background:var(--card);border:1px solid var(--line);border-radius:12px;padding:16px;display:flex;flex-direction:column;gap:10px;max-height:calc(100vh - 220px);overflow:auto;min-width:0}
@media (max-width:900px){.detail{position:fixed;inset:0;max-height:none;border-radius:0;border:none;z-index:10;padding:0 max(16px,env(safe-area-inset-right,0px)) calc(16px + env(safe-area-inset-bottom,0px)) max(16px,env(safe-area-inset-left,0px))}}
.dhead{display:flex;justify-content:flex-end}
@media (max-width:900px){.dhead{position:sticky;top:0;background:var(--card);padding:calc(10px + env(safe-area-inset-top,0px)) 0 8px;border-bottom:1px solid var(--line)}}
@media (min-width:901px){.dhead{display:none}}
.close{font-size:14px;border:1px solid var(--line);background:var(--card);border-radius:999px;padding:3px 14px;cursor:pointer}
.detail h3{outline:none;margin:0;font-size:19px;line-height:1.5;text-wrap:balance}
.tags{display:flex;flex-wrap:wrap;gap:6px} .tags span{font-size:12px;border-radius:999px;padding:1px 10px;background:var(--bg)}
.detail dl{margin:0;display:grid;grid-template-columns:8em minmax(0,1fr);gap:8px 10px;font-size:14px}
@media (max-width:420px){.detail dl{grid-template-columns:1fr;gap:2px}.detail dd{margin-bottom:8px}}
.detail dt{color:var(--sub);font-size:12.5px} .detail dd{margin:0}
.links{display:flex;flex-direction:column;gap:4px}
.links button{all:unset;cursor:pointer;color:var(--accent);font-size:14px} .links button:hover{text-decoration:underline} .links button:focus-visible{outline:2px solid var(--accent)}
.id{font-size:11.5px;color:var(--sub)}
.placeholder{color:var(--sub);font-size:14px}
details.removed summary{cursor:pointer;color:var(--accent);font-size:14px}
details.removed ul{margin:6px 0 0;padding-left:1.2em;font-size:13.5px;display:flex;flex-direction:column;gap:4px}
.howto{font-size:13px;color:var(--sub)}
</style>
<main>
<h1>ユースケースの一覧</h1>
<button type="button" class="top" id="top"></button>
<p class="counts" id="counts"></p>
<div class="bar">
  <input class="search" id="q" type="search" placeholder="何をする ・ 誰が ・ 保証の言葉で探す" aria-label="ユースケースを探す">
  <div class="filters" id="f-area"></div>
  <div class="filters" id="f-kind"></div>
  <details class="more" id="more"><summary>誰がで絞る</summary><div class="filters" id="f-actor"></div></details>
  <div class="state"><span id="state" aria-live="polite"></span><span class="view" role="group" aria-label="表示"><button type="button" data-view="cards">カード</button><button type="button" data-view="rows">行で比べる</button></span></div>
</div>
<div class="layout">
  <div id="list"></div>
  <aside class="detail" id="detail" aria-label="ユースケースの詳細"></aside>
</div>
<details class="removed" id="removed"></details>
<p class="howto">ユースケースを足すときは、docs/usecases/usecases.json の items に1件足して build.py を走らせる。</p>
</main>
<script>
const D=__DATA__;
const KIND={summary:'まとめの目的','user-goal':'利用者の目的',subfunction:'サブ機能'};
const items=D.items, byId=Object.fromEntries(items.map(e=>[e.id,e]));
const esc=s=>String(s??'').replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
const short=g=>String(g||'').replace(/（.*?）/g,'').trim();
const isNew=e=>e.id.startsWith('N');
const kindOf=e=>isNew(e)?'new':e.level;
const kindLabel=e=>isNew(e)?'利用者の目的（新しく足す案）':KIND[e.level];
const actorOf=e=>short(e.actor);
const CROSS='業務領域をまたぐ';
const areaOf=e=>e.area||CROSS;
const outer=items.find(e=>e.level==='summary'&&!e.area);
const areas=[...new Set(items.filter(e=>e!==outer).map(areaOf))];
const actors=[...new Set(items.map(actorOf))];
const st={q:'',area:new Set(),kind:new Set(),actor:new Set(),view:'cards',sel:null,from:null};
try{const s=JSON.parse(localStorage.getItem('uc-filters')||'null');if(s){st.area=new Set(s.area||[]);st.kind=new Set(s.kind||[]);st.actor=new Set(s.actor||[]);st.view=s.view||'cards';}}catch(_){}
function save(){try{localStorage.setItem('uc-filters',JSON.stringify({area:[...st.area],kind:[...st.kind],actor:[...st.actor],view:st.view}))}catch(_){}}
const $=id=>document.getElementById(id);
const narrow=()=>window.matchMedia('(max-width:900px)').matches;
if(st.actor.size)$('more').open=true;

$('top').innerHTML=outer?`<span class="k">いちばん外側のまとめの目的</span><span class="s">${esc(actorOf(outer))}が</span><b>${esc(outer.goal)}</b><span class="s">下の利用者の目的をくり返し果たすことで達する。押すと詳細</span>`:'';
$('top').onclick=()=>outer&&show(outer.id,true);
const cnt=k=>items.filter(e=>e.level===k).length;
$('counts').innerHTML=`全 <b>${items.length}</b> 件 ・ まとめの目的 <b>${cnt('summary')}</b> ・ 利用者の目的 <b>${cnt('user-goal')}</b> ・ サブ機能 <b>${cnt('subfunction')}</b> ・ 業務領域 <b>${areas.filter(a=>a!==CROSS).length}</b>`;

function chips(el,label,values,set,show){
  el.innerHTML=`<span class="lbl">${label}</span>`+values.map(v=>`<button type="button" class="chip" aria-pressed="${set.has(v)}" data-v="${esc(v)}">${esc(show?show(v):v)}</button>`).join('');
  el.querySelectorAll('.chip').forEach(b=>b.onclick=()=>{const v=b.dataset.v;set.has(v)?set.delete(v):set.add(v);save();render();});
}
function match(e){
  if(st.area.size&&!st.area.has(areaOf(e)))return false;
  if(st.kind.size&&!st.kind.has(e.level))return false;
  if(st.actor.size&&!st.actor.has(actorOf(e)))return false;
  if(st.q){const t=[e.goal,e.actor,e.success,e.minimum,e.area].join(' ');if(!t.includes(st.q))return false;}
  return true;
}
function stateLine(n){
  const conds=[];
  if(st.q)conds.push(['q','',`「${st.q}」`]);
  st.area.forEach(v=>conds.push(['area',v,'業務領域：'+v]));
  st.kind.forEach(v=>conds.push(['kind',v,'種類：'+KIND[v]]));
  st.actor.forEach(v=>conds.push(['actor',v,'誰が：'+v]));
  $('state').innerHTML=(conds.length?conds.map(([k,v,t])=>`<button type="button" class="cond" data-k="${k}" data-v="${esc(v)}" aria-label="${esc(t)} を外す">${esc(t)} ×</button>`).join('')+'<span>―</span>':'<span>すべて ―</span>')+`<span class="n">${n}件</span>`;
  $('state').querySelectorAll('.cond').forEach(b=>b.onclick=()=>{const k=b.dataset.k;if(k==='q'){st.q='';$('q').value='';}else st[k].delete(b.dataset.v);save();render();});
  document.querySelectorAll('[data-view]').forEach(b=>b.setAttribute('aria-pressed',b.dataset.view===st.view));
}
function card(e){
  return `<button type="button" class="uc k-${kindOf(e)}" data-id="${esc(e.id)}" aria-current="${st.sel===e.id}"><span class="kind">${esc(kindLabel(e))}</span><span class="who">${esc(actorOf(e))}が</span><b>${esc(short(e.goal))}</b></button>`;
}
function rows(es){
  return `<div class="tw"><table><thead><tr><th>何をする</th><th>種類</th><th>誰が</th><th>成功したときの保証</th></tr></thead><tbody>${es.map(e=>`<tr class="row" data-row="${esc(e.id)}" aria-current="${st.sel===e.id}"><td><button type="button" class="uc-row" data-id="${esc(e.id)}">${esc(short(e.goal))}</button></td><td>${esc(kindLabel(e))}</td><td>${esc(actorOf(e))}</td><td>${esc(e.success)}</td></tr>`).join('')}</tbody></table></div>`;
}
function render(){
  chips($('f-area'),'業務領域',areas,st.area);
  chips($('f-kind'),'種類',['summary','user-goal','subfunction'],st.kind,v=>KIND[v]);
  chips($('f-actor'),'誰が',actors,st.actor);
  let html='',n=0;
  for(const a of areas){
    const es=items.filter(e=>e!==outer&&areaOf(e)===a&&match(e));if(!es.length)continue;n+=es.length;
    const kind=D.areaKind[a]?esc(D.areaKind[a])+' ・ ':'';
    html+=`<section class="area"><h2>${esc(a)}<small>${kind}${es.length}件</small></h2>${st.view==='rows'?rows(es):`<div class="grid">${es.map(card).join('')}</div>`}</section>`;
  }
  $('list').innerHTML=html||'<p class="empty">条件に合うユースケースはありません。上の条件の × で外してください。</p>';
  stateLine(n);
  $('list').querySelectorAll('[data-id]').forEach(b=>b.onclick=ev=>{ev.stopPropagation();show(b.dataset.id,true);});
  $('list').querySelectorAll('tr.row').forEach(r=>r.onclick=()=>show(r.dataset.row,true));
}
const link=id=>byId[id]?`<button type="button" data-go="${esc(id)}">${esc(short(byId[id].goal))}</button>`:'';
const block=ids=>ids.length?`<div class="links">${ids.map(link).join('')}</div>`:'';
function placeholder(){$('detail').innerHTML='<p class="placeholder">カードを押すと、ここに詳しい内容が出ます。</p>';}
function show(id,push){
  const e=byId[id];if(!e)return;
  if(push&&st.sel!==id){try{history.pushState({id},'','#'+id)}catch(_){}}
  if(!st.sel)st.from=document.activeElement;
  st.sel=id;
  const parents=items.filter(x=>(x.calls||[]).includes(id)).map(x=>x.id);
  const subs=items.filter(x=>x.level==='subfunction'&&(x.called_by||[]).includes(id)).map(x=>x.id);
  const row=(k,v)=>v?`<dt>${k}</dt><dd>${v}</dd>`:'';
  const d=$('detail');
  d.innerHTML=`<div class="dhead"><button type="button" class="close" id="close">一覧に戻る</button></div>
   <div class="tags"><span>${esc(kindLabel(e))}</span><span>${esc(areaOf(e))}</span></div>
   <h3 tabindex="-1" id="dtitle">${esc(e.goal)}</h3>
   <dl>${row('誰が',esc(e.actor))}${row('成功したときの保証',esc(e.success))}${row('最低限の保証',esc(e.minimum))}
   ${row('含むまとめの目的',block(parents))}${row('下の目的',block(e.calls||[]))}
   ${row('使うサブ機能',block(subs))}${row('使う目的',block(e.called_by||[]))}
   ${row('中の手順',(e.steps||[]).map(s=>`<div>${esc(s)}</div>`).join(''))}
   ${row('拡張',(e.extensions||[]).map(x=>`<div>${esc(x.when)}とき → ${esc(x.then)}</div>`).join(''))}
   ${row('設計の範囲',esc(e.scope))}${row('なぜこの形か',esc(e.why))}</dl>
   <span class="id">識別子 ${esc(e.id)}</span>`;
  d.querySelectorAll('[data-go]').forEach(b=>b.onclick=()=>show(b.dataset.go,true));
  $('close').onclick=back;
  d.hidden=false;d.scrollTop=0;
  render();if(push)$('dtitle').focus({preventScroll:!narrow()});
}
function back(){history.state&&history.state.id?history.back():closeDetail();}
function closeDetail(){
  const from=st.sel;st.sel=null;placeholder();if(narrow())$('detail').hidden=true;render();
  const c=document.querySelector(`[data-id="${from}"]`)||st.from;if(c&&c.focus)c.focus();st.from=null;
}
window.addEventListener('popstate',ev=>{const id=(ev.state&&ev.state.id)||(location.hash||'').slice(1);byId[id]?show(id,false):closeDetail();});
document.addEventListener('keydown',ev=>{if(ev.key==='Escape'&&st.sel)back();});
$('q').addEventListener('input',ev=>{st.q=ev.target.value.trim();render();});
document.querySelectorAll('[data-view]').forEach(b=>b.onclick=()=>{st.view=b.dataset.view;save();render();});
$('removed').innerHTML=D.removed.length?`<summary>外したもの（${D.removed.length}件）</summary><ul>${D.removed.map(r=>`<li><b>${esc(short(r.goal))}</b>：${esc(r.why)}</li>`).join('')}</ul>`:'';
placeholder();if(narrow())$('detail').hidden=true;
render();
const h=(location.hash||'').slice(1);if(byId[h])show(h,false);
</script>
'''
open(os.path.join(H, 'index.html'), 'w').write(PAGE.replace('__DATA__', data))
