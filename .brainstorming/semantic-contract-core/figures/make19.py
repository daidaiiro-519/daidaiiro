# 第61回の完成イメージ：体験のシミュレーション（screen）。
# 同じ「記事を書く Skill」で、第59回の作り（instance ごとに承認）と第60回の作り（契約と例外だけ）を比べる。
# 色はブレストボードのデザイントークン（design-system.json）から取り、値を手で書かない。
import json, re

DS = json.load(open('../../../.claude/skills/brainstorming/assets/catalog/design-system.json'))
T, D = DS['tokens'], DS['dark']


def val(v, table):
    while isinstance(v, str) and v.startswith('{'):
        k = v.strip('{}').replace('.', '-')
        v = table.get(k, T.get(k))
    return v


USE = {'page': 'color-surface-page', 'block': 'color-surface-block', 'head': 'color-surface-head',
       'control': 'color-surface-control', 'answer': 'color-surface-answer', 'notice': 'color-surface-notice',
       'done': 'color-surface-done', 'removed': 'color-surface-removed', 'literal': 'color-surface-literal',
       'text': 'color-text-body', 'sub': 'color-text-sub', 'key': 'color-key', 'onkey': 'color-text-on-key',
       'warn': 'color-text-warn', 'textdone': 'color-text-done', 'textremoved': 'color-text-removed',
       'line': 'color-line-block', 'linectl': 'color-line-control', 'linenotice': 'color-line-notice',
       'linedone': 'color-line-done', 'source': 'color-text-source'}
light = ';'.join(f'--{k}:{val(T[v], T)}' for k, v in USE.items())
dark = ';'.join(f'--{k}:{val(D.get(v, T[v]), {**T, **D})}' for k, v in USE.items())
font = T['font-body']

HTML = r'''<!doctype html><html lang="ja"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<style>
:root{%LIGHT%}
@media (prefers-color-scheme:dark){:root{%DARK%}}
*{box-sizing:border-box}
body{margin:0;padding:12px 16px;background:var(--page);color:var(--text);font:13px/1.6 %FONT%}
h3{margin:0 0 6px;font-size:13px}
button{font:inherit;cursor:pointer;border:1px solid var(--linectl);background:var(--control);color:var(--text);border-radius:6px;padding:3px 10px}
button.key{background:var(--key);color:var(--onkey);border-color:var(--key)}
button:disabled{opacity:.45;cursor:default}
.modes{display:flex;gap:6px;flex-wrap:wrap;margin-bottom:8px}
.modes button[aria-pressed=true]{background:var(--key);color:var(--onkey);border-color:var(--key)}
.scenes{display:flex;gap:6px;flex-wrap:wrap;margin-bottom:10px}
.scenes button{background:var(--block)}
.grid{display:grid;grid-template-columns:1fr 1fr;gap:10px}
@media (max-width:640px){.grid{grid-template-columns:1fr}}
.box{background:var(--block);border:1px solid var(--line);border-radius:8px;padding:10px;min-width:0}
.count{display:flex;gap:12px;flex-wrap:wrap;margin-bottom:6px}
.count b{font-size:20px;color:var(--key)}
.card{border:1px solid var(--linenotice);background:var(--notice);border-radius:6px;padding:8px;margin:6px 0}
.card .t{font-weight:700}
.card .row{display:flex;gap:6px;align-items:center;flex-wrap:wrap;margin-top:4px}
.empty{color:var(--sub)}
.art{display:flex;justify-content:space-between;gap:8px;align-items:center;border:1px solid var(--line);background:var(--literal);border-radius:6px;padding:4px 8px;margin:4px 0;cursor:pointer}
.art[aria-selected=true]{outline:2px solid var(--key)}
.badge{font-size:12px;border-radius:10px;padding:0 8px;border:1px solid var(--line);background:var(--head);white-space:nowrap}
.b-ok{background:var(--done);color:var(--textdone);border-color:var(--linedone)}
.b-ng{background:var(--removed);color:var(--textremoved)}
.b-wait{background:var(--notice);color:var(--warn);border-color:var(--linenotice)}
table{border-collapse:collapse;width:100%;font-size:12px}
th,td{border:1px solid var(--line);padding:3px 6px;text-align:left;vertical-align:top}
th{background:var(--head)}
td code,pre{font-family:"JetBrains Mono",monospace;font-size:11px}
pre{margin:0;white-space:pre-wrap;word-break:break-all;background:var(--literal);border:1px solid var(--line);border-radius:6px;padding:6px;max-height:220px;overflow:auto}
.tabs{display:flex;gap:4px;margin-bottom:6px;flex-wrap:wrap}
.tabs button[aria-pressed=true]{background:var(--answer);border-color:var(--key)}
.log{max-height:170px;overflow:auto;font-size:12px}
.log div{border-bottom:1px solid var(--line);padding:2px 0}
.who{display:inline-block;min-width:3.2em;font-weight:700}
.who-人,.who-作者{color:var(--warn)}.who-AI{color:var(--source)}.who-基盤{color:var(--key)}
.note{color:var(--sub);font-size:12px}
.wide{margin-top:10px}
</style></head><body>
<div class="modes" role="group" aria-label="作り">
 <button data-mode="r60" aria-pressed="true">第60回の作り（契約と例外だけ）</button>
 <button data-mode="r59" aria-pressed="false">第59回の作り（instance ごとに承認）</button>
</div>
<div class="scenes">
 <button data-s="write">① AI が記事を5件書く</button>
 <button data-s="fix">② AI が下書きを直す</button>
 <button data-s="edit">③ AI が判断済みの記事の本文を直す</button>
 <button data-s="ext">④ ファイルを直接書き換える</button>
 <button data-s="swap">⑤ validator を ai@2 に替える</button>
 <button data-s="weaken">⑥ AI が rule を弱める</button>
 <button data-s="addhuman">⑦ 作者が人の rule を足す</button>
 <button data-s="reset">はじめから</button>
</div>
<div class="grid">
 <section class="box"><h3>人に届くもの（受信箱）</h3>
  <div class="count"><span>人の操作 <b id="ops">0</b> 回</span><span>いま届いている <b id="got">0</b> 件</span></div>
  <div id="inbox"></div></section>
 <section class="box"><h3>記事（押すと状態の計算が出る）</h3><div id="arts"></div>
  <p class="note" id="artnote"></p></section>
</div>
<section class="box wide"><h3 id="calch">状態の計算</h3><div id="calc" class="note">記事を押すと、この記事の状態を基盤がどう導いたかが出る。</div></section>
<section class="box wide"><h3>中で保存している記録</h3>
 <div class="tabs" id="rtabs"></div><div id="rec"></div></section>
<section class="box wide"><h3>起きたこと</h3><div class="log" id="log"></div></section>
<script>
const $=s=>document.querySelector(s);
const clone=o=>JSON.parse(JSON.stringify(o));
const H=s=>{let h=5381;for(const c of s)h=((h<<5)+h+c.codePointAt(0))>>>0;return h.toString(16).padStart(8,'0').slice(0,6)};
const RULES0={title:{name:'タイトルは20字以内',by:'機械',reads:['タイトル']},summary:{name:'要約は本文と矛盾しない',by:'AI',reads:['要約','本文']}};
const SEED=[
 ['記事1','新製品の発表','新製品を出す','新製品を10月に出す。'],
 ['記事2','価格改定のお知らせ','価格を上げる','来月から価格を5%上げる。'],
 ['記事3','社内イベントの報告と来期の予定と感想のまとめ','社内イベントを開いた','先週、社内イベントを開いた。'],
 ['記事4','サポート窓口の変更','窓口が変わる','窓口の場所は曖昧なまま。'],
 ['記事5','採用情報','募集を続ける（本文と矛盾）','募集は締め切った。']];
let S;
function reset(mode){S={mode,c:{v:1,rules:clone(RULES0)},pv:null,asg:{summary:'ai@1'},pa:null,arts:[],recs:[],judg:[],ops:0,got:0,log:[],sel:null,rtab:'contract',old:{state:{},cards:[]}};log('基盤','契約 v1（rule 2つ）と validator の割り当て（要約 → ai@1）が公開されている');render();}
function log(who,t){S.log.unshift([who,t]);}
const art=id=>S.arts.find(a=>a.id===id);
const assignOf=rid=>S.c.rules[rid].by==='AI'?S.asg[rid]:(S.c.rules[rid].by==='機械'?'engine@1':'人');
function runV(a,rid,rules,asg){const r=(rules||S.c.rules)[rid],f=a.f,v=r.by==='AI'?(asg||S.asg)[rid]:null;
 if(r.by==='機械')return f['タイトル'].length<=20?['verified','タイトル '+f['タイトル'].length+'字']:['violated','タイトル '+f['タイトル'].length+'字'];
 if(r.by==='人')return['unverified','人が判定する rule'];
 if(!v)return['unsupported','validator が無い'];
 if(v==='ai@1'){if(f['本文'].includes('曖昧'))return['unverified','本文から判定できない'];if(f['要約'].includes('矛盾'))return['violated','要約「'+f['要約']+'」'];return['verified','本文「'+f['本文']+'」'];}
 if(f['要約'].includes('矛盾'))return['violated','要約「'+f['要約']+'」'];
 if(f['本文'].includes('曖昧'))return['verified','本文「'+f['本文']+'」から読み取れる'];
 if(f['要約'].length>f['本文'].length)return['violated','要約が本文より長い'];
 return['verified','本文「'+f['本文']+'」'];}
function inputOf(a,rid,withV=true){const r=S.c.rules[rid];return H(JSON.stringify([rid,r,r.reads.map(k=>a.f[k]),withV?assignOf(rid):'']));}
function current(a,rid){const inp=inputOf(a,rid);const rs=S.recs.filter(x=>x.inst===a.id&&x.rule===rid&&x.input===inp);return rs.length?rs[rs.length-1]:null;}
function judgOf(a,rid){const js=S.judg.filter(j=>j.inst===a.id&&j.rule===rid&&!j.revoked);const j=js[js.length-1];if(!j)return null;return Object.assign({ok:j.shown===inputOf(a,rid,false)},j);}
function eff(a,rid){const j=judgOf(a,rid);if(j&&j.ok)return j.verdict==='合っている'?'verified（人）':'violated（人）';const c=current(a,rid);return c?c.result:'確かめ中';}
function state(a){const e=Object.keys(S.c.rules).map(r=>eff(a,r));
 if(e.some(x=>x.startsWith('violated')))return'下書き';if(e.includes('確かめ中'))return'確かめ中';
 if(e.some(x=>x==='unverified'||x==='unsupported'))return'判断待ち';return'検証済み';}
function verifyAll(){for(const a of S.arts)for(const rid of Object.keys(S.c.rules)){if(current(a,rid))continue;const[res,ev]=runV(a,rid);
 S.recs.push({inst:a.id,rule:rid,input:inputOf(a,rid),result:res,by:assignOf(rid),evidence:ev});log('基盤','検証の記録に1行足した：'+a.id+' ×「'+S.c.rules[rid].name+'」→ '+res);}}
function settle(){render();setTimeout(()=>{if(S.mode==='r60')verifyAll();render();},700);}
function trial(apply){const keep=JSON.stringify(S);const before={};for(const a of S.arts)before[a.id]=state(a);
 apply();verifyAll();const changed=S.arts.filter(a=>state(a)!==before[a.id]).map(a=>a.id);
 const need=S.arts.filter(a=>state(a)==='判断待ち').length;S=JSON.parse(keep);return{changed,need};}
// ── 第59回の作り（instance ごとに承認） ──
function oldCard(id,kind){if(!S.old.cards.some(c=>c.id===id&&c.kind===kind)){S.old.cards.push({id,kind});S.got++;}}
function oldNeeds(a){return Object.keys(S.c.rules).some(r=>runV(a,r)[0]==='violated');}
// ── 場面 ──
function scene(s){
 if(s==='reset')return reset(S.mode);
 if(s==='write'){if(S.arts.length){log('AI','記事は作り済み');return render();}
  for(const[id,t,su,b]of SEED){S.arts.push({id,v:S.c.v,f:{'タイトル':t,'要約':su,'本文':b}});}
  log('AI','記事を5件作った');
  if(S.mode==='r59'){for(const a of S.arts){if(oldNeeds(a))log('基盤',a.id+'は違反があるので AI へ返す');else{S.old.state[a.id]='承認待ち';oldCard(a.id,'承認');}}log('基盤','違反の無い記事ごとに、人へ「承認してください」を出した');}
  return settle();}
 if(!S.arts.length){log('基盤','先に ① で記事を作る');return render();}
 if(s==='fix'){const a3=art('記事3'),a5=art('記事5');a3.f['タイトル']='社内イベントの報告';a5.f['要約']='募集は締め切った';log('AI','記事3のタイトルと記事5の要約を直した');
  if(S.mode==='r59'){for(const a of[a3,a5]){S.old.state[a.id]='承認待ち';oldCard(a.id,'承認');}}return settle();}
 if(s==='edit'){const a=S.arts.find(x=>Object.keys(S.c.rules).some(r=>{const j=judgOf(x,r);return j&&j.ok;}))||art('記事4');
  if(S.mode==='r59'&&S.old.state[a.id]==='承認済み'){oldCard(a.id,'変更の提案');log('AI',a.id+'は承認済みなので、本文の変更を「変更の提案」として出した');return render();}
  a.f['本文']=a.f['本文'].replace('曖昧なまま','3階に移る');if(!a.f['本文'].includes('3階'))a.f['本文']+='（追記）';log('AI',a.id+'の本文を直した');
  if(S.mode==='r59'){S.old.state[a.id]='承認待ち';oldCard(a.id,'承認');}return settle();}
 if(s==='ext'){const a=art('記事2');a.f['要約']='価格を5%上げる';log('人',a.id+'のファイルを、基盤を通らずに直接書き換えた');
  if(S.mode==='r59'&&S.old.state[a.id]==='承認済み'){S.old.state[a.id]='陳腐化';oldCard(a.id,'陳腐化した。承認し直してください');}return settle();}
 if(s==='swap'){if(S.mode==='r59'){S.asg.summary='ai@2';log('基盤','validator を ai@2 に替えた（人の承認は通らない）');
   for(const a of S.arts)if(S.old.state[a.id]==='承認済み'){S.old.state[a.id]='確かめ直し';oldCard(a.id,'確かめ直し。承認し直してください');}return render();}
  const t=trial(()=>{S.asg.summary='ai@2';});S.pa={to:'ai@2',trial:t,samples:S.arts.filter(a=>state(a)!=='下書き').slice(0,2).map(a=>({id:a.id,ai:runV(a,'summary',null,{summary:'ai@2'})[0],v:null}))};S.got++;
  log('作者','validator を ai@2 に替える承認を依頼した（試した結果と抜き取り2件付き）');return render();}
 if((s==='weaken'||s==='addhuman'||s==='swap')&&S.mode==='r60'&&(S.pv||S.pa)){log('基盤','承認待ちの依頼があるので、先にそれを承認するか差し戻す');return render();}
 if(s==='weaken'){if(S.mode==='r59'){delete S.c.rules.summary;S.c.v++;log('AI','rule「要約は本文と矛盾しない」を消した。人には何も届かない（契約の変更に関門が無い）');return render();}
  const rules=clone(S.c.rules);delete rules.summary;const t=trial(()=>{S.c.rules=rules;});S.pv={rules,diff:['−「要約は本文と矛盾しない」（AI）を消す'],trial:t,by:'AI'};S.got++;log('AI','rule を消した新しい版の承認を依頼した');return render();}
 if(s==='addhuman'){const rules=clone(S.c.rules);rules.publish={name:'社外に出してよい内容か',by:'人',reads:['本文']};
  if(S.mode==='r59'){S.c.rules=rules;S.c.v++;log('作者','rule を足した');for(const a of S.arts)if(S.old.state[a.id]==='承認済み'){S.old.state[a.id]='陳腐化';oldCard(a.id,'陳腐化した。承認し直してください');}return settle();}
  const t=trial(()=>{S.c.rules=rules;});S.pv={rules,diff:['＋「社外に出してよい内容か」（人が判定 ・ 本文を読む）を足す'],trial:t,by:'作者'};S.got++;log('作者','rule を足した新しい版の承認を依頼した');return render();}
}
// ── 人の操作 ──
function op(){S.ops++;}
function judge(id,rid,verdict){const a=art(id);op();S.judg.push({inst:id,rule:rid,shown:inputOf(a,rid,false),verdict,by:'人'});log('人',id+' ×「'+S.c.rules[rid].name+'」を「'+verdict+'」と判断した');render();}
function judgeAll(rid){const ids=S.arts.filter(a=>eff(a,rid)==='unverified'||eff(a,rid)==='unsupported').map(a=>a.id);op();for(const id of ids){const a=art(id);S.judg.push({inst:id,rule:rid,shown:inputOf(a,rid,false),verdict:'合っている',by:'人'});}log('人','画面に出した'+ids.length+'件をまとめて「合っている」と判断した（判断は1件ずつ保存）');render();}
function approveV(ok){op();if(ok){S.c={v:S.c.v+1,rules:S.pv.rules};log('人','契約 v'+S.c.v+' を承認した。基盤が新しい版で検証する');S.pv=null;return settle();}log('人','差し戻した。契約は v'+S.c.v+' のまま');S.pv=null;render();}
function sample(i,v){op();S.pa.samples[i].v=v;log('人','抜き取り '+S.pa.samples[i].id+' を「'+v+'」と確かめた');render();}
function approveA(ok){op();if(ok){S.asg.summary=S.pa.to;log('人','validator の割り当て（→ '+S.pa.to+'）を承認した。基盤が AI の rule だけ検証し直す');S.pa=null;return settle();}log('人','差し戻した');S.pa=null;render();}
function oldApprove(i){const c=S.old.cards[i];op();S.old.cards.splice(i,1);const a=art(c.id);
 if(c.kind==='変更の提案'){a.f['本文']=a.f['本文'].replace('曖昧なまま','3階に移る');}
 const und=Object.keys(S.c.rules).filter(r=>runV(a,r)[0]==='unverified').length;S.ops+=und;
 S.old.state[c.id]='承認済み';log('人',c.id+'を承認した'+(und?'（未検証の rule '+und+'件も1件ずつ判断した）':''));render();}
// ── 描く ──
const badge=st=>'<span class="badge '+({検証済み:'b-ok',承認済み:'b-ok',下書き:'b-ng',陳腐化:'b-ng',確かめ中:'b-wait',判断待ち:'b-wait',承認待ち:'b-wait',確かめ直し:'b-wait'}[st]||'')+'">'+st+'</span>';
function render(){
 document.querySelectorAll('.modes button').forEach(b=>b.setAttribute('aria-pressed',b.dataset.mode===S.mode));
 $('#ops').textContent=S.ops;
 const ib=[];
 if(S.mode==='r60'){
  if(S.pv)ib.push('<div class="card"><div class="t">契約の新しい版の承認（依頼：'+S.pv.by+'）</div>'+S.pv.diff.map(d=>'<div>'+d+'</div>').join('')+'<div class="note">試した結果：状態が変わる記事 '+S.pv.trial.changed.length+'件（'+(S.pv.trial.changed.join('、')||'なし')+'）／ 承認すると人の判断が要る記事 '+S.pv.trial.need+'件</div><div class="row"><button class="key" onclick="approveV(true)">承認する</button><button onclick="approveV(false)">差し戻す</button></div></div>');
  if(S.pa){const done=S.pa.samples.every(x=>x.v);ib.push('<div class="card"><div class="t">validator の割り当ての承認（要約 → '+S.pa.to+'）</div><div class="note">試した結果：状態が変わる記事 '+S.pa.trial.changed.length+'件（'+(S.pa.trial.changed.join('、')||'なし')+'）</div><div>抜き取り（AI の判定を人が確かめる）</div>'+S.pa.samples.map((x,i)=>'<div class="row">'+x.id+'：AI は '+x.ai+' '+(x.v?'→ 人は「'+x.v+'」':'<button onclick="sample('+i+',\'合っている\')">合っている</button><button onclick="sample('+i+',\'合っていない\')">合っていない</button>')+'</div>').join('')+'<div class="row"><button class="key" '+(done?'':'disabled')+' onclick="approveA(true)">承認する</button><button onclick="approveA(false)">差し戻す</button>'+(done?'':'<span class="note">抜き取りを確かめるまで承認できない</span>')+'</div></div>');}
  for(const rid of Object.keys(S.c.rules)){const list=S.arts.filter(a=>{const e=eff(a,rid);return e==='unverified'||e==='unsupported';});if(!list.length)continue;
   ib.push('<div class="card"><div class="t">判断が要る：「'+S.c.rules[rid].name+'」'+list.length+'件</div>'+list.map(a=>{const c=current(a,rid);return'<div class="row">'+a.id+'：'+(c?c.evidence:'')+' <button onclick="judge(\''+a.id+'\',\''+rid+'\',\'合っている\')">合っている</button><button onclick="judge(\''+a.id+'\',\''+rid+'\',\'合っていない\')">合っていない</button></div>';}).join('')+(list.length>1?'<div class="row"><button onclick="judgeAll(\''+rid+'\')">出ている'+list.length+'件をまとめて「合っている」</button></div>':'')+'</div>');}
 }else{
  S.old.cards.forEach((c,i)=>ib.push('<div class="card"><div class="t">'+c.id+'：'+(c.kind==='承認'?'承認してください':c.kind)+'</div><div class="row"><button class="key" onclick="oldApprove('+i+')">承認する</button></div></div>'));
 }
 $('#got').textContent=ib.length;
 $('#inbox').innerHTML=ib.join('')||'<p class="empty">届いているものは無い</p>';
 $('#arts').innerHTML=S.arts.map(a=>'<div class="art" aria-selected="'+(S.sel===a.id)+'" onclick="S.sel=\''+a.id+'\';render()"><span>'+a.id+'　'+a.f['タイトル']+'</span>'+badge(S.mode==='r60'?state(a):(S.old.state[a.id]||(oldNeeds(a)?'下書き':'承認待ち')))+'</div>').join('')||'<p class="empty">まだ無い。① を押す</p>';
 $('#artnote').textContent=S.mode==='r60'?'下書き（違反）は人ではなく AI へ返る。検証済みには何も届かない。':'違反の無い記事は、すべて人の承認を待つ。';
 renderCalc();renderRec();
 $('#log').innerHTML=S.log.slice(0,40).map(([w,t])=>'<div><span class="who who-'+w+'">'+w+'</span>'+t+'</div>').join('');}
function renderCalc(){const a=S.sel&&art(S.sel);if(!a){$('#calc').innerHTML='記事を押すと、この記事の状態を基盤がどう導いたかが出る。';return;}
 $('#calch').textContent='状態の計算：'+a.id;
 if(S.mode==='r59'){$('#calc').innerHTML='<p>第59回の作りでは、状態（'+(S.old.state[a.id]||'承認待ち')+'）を記事ごとに保存し、承認 ・ 確かめ直し ・ 陳腐化を出来事のたびに書き換える。</p>';return;}
 const rows=Object.keys(S.c.rules).map(rid=>{const r=S.c.rules[rid],c=current(a,rid),j=judgOf(a,rid);
  return'<tr><td>'+r.name+'<br><span class="note">'+r.by+' ・ 読む欄 '+r.reads.join('・')+'</span></td><td><code>'+inputOf(a,rid)+'</code></td><td>'+(c?c.result+'（'+c.by+'）':'一致する記録が無い')+'</td><td>'+(j?(j.ok?'効く：'+j.verdict:'効かない（入力が変わった）'):'無い')+'</td><td>'+badge(eff(a,rid).startsWith('verified')?'検証済み':eff(a,rid)==='確かめ中'?'確かめ中':eff(a,rid).startsWith('violated')?'下書き':'判断待ち').replace(/>[^<]+</,'>'+eff(a,rid)+'<')+'</td></tr>';}).join('');
 $('#calc').innerHTML='<table><tr><th>rule</th><th>① いまの入力のハッシュ</th><th>② 一致する検証の記録</th><th>③ 人の判断</th><th>効いている結果</th></tr>'+rows+'</table><p>④ 決定表：violated があれば下書き → 確かめ中があれば確かめ中 → 判断が要るものがあれば判断待ち → それ以外は検証済み。<b>この記事は '+state(a)+'</b>。状態は保存せず、読むたびにこの手順で導く。</p>';}
function renderRec(){const tabs=S.mode==='r60'?[['contract','契約と割り当て'],['recs','検証の記録'],['judg','人の判断'],['arts','記事']]:[['contract','契約'],['old','記事ごとの承認の状態'],['arts','記事']];
 if(!tabs.some(t=>t[0]===S.rtab))S.rtab='contract';
 $('#rtabs').innerHTML=tabs.map(([k,n])=>'<button aria-pressed="'+(S.rtab===k)+'" onclick="S.rtab=\''+k+'\';render()">'+n+'</button>').join('');
 let out;
 if(S.rtab==='contract')out=JSON.stringify({version:S.c.v,rules:S.c.rules,assignment:S.asg,pending:S.pv?'契約の新しい版（承認待ち）':S.pa?'割り当て（承認待ち）':null},null,1);
 if(S.rtab==='recs')out=S.recs.length?'<table><tr><th>記事</th><th>rule</th><th>input</th><th>result</th><th>by</th><th>根拠</th></tr>'+S.recs.slice(-14).reverse().map(r=>'<tr><td>'+r.inst+'</td><td>'+S.c.rules[r.rule]?.name+'</td><td><code>'+r.input+'</code></td><td>'+r.result+'</td><td>'+r.by+'</td><td>'+r.evidence+'</td></tr>').join('')+'</table><p class="note">足すだけで書き換えない。新しい14行を表示（全'+S.recs.length+'行）。</p>':'<p class="empty">まだ無い</p>';
 if(S.rtab==='judg')out=S.judg.length?'<table><tr><th>記事</th><th>rule</th><th>示した入力</th><th>判断</th><th>いま</th></tr>'+S.judg.slice().reverse().map(j=>{const a=art(j.inst);return'<tr><td>'+j.inst+'</td><td>'+(S.c.rules[j.rule]?.name||j.rule)+'</td><td><code>'+j.shown+'</code></td><td>'+j.verdict+'</td><td>'+(a&&S.c.rules[j.rule]&&j.shown===inputOf(a,j.rule,false)?'効く':'効かない')+'</td></tr>';}).join('')+'</table>':'<p class="empty">まだ無い</p>';
 if(S.rtab==='arts')out=JSON.stringify(S.arts.map(a=>({id:a.id,$schema:'articles@v'+S.c.v,...a.f})),null,1);
 if(S.rtab==='old')out=JSON.stringify(S.old.state,null,1);
 $('#rec').innerHTML=out.startsWith('<')?out:'<pre>'+out.replace(/[<&]/g,c=>c==='<'?'&lt;':'&amp;')+'</pre>';}
document.querySelectorAll('.modes button').forEach(b=>b.onclick=()=>reset(b.dataset.mode));
document.querySelectorAll('.scenes button').forEach(b=>b.onclick=()=>scene(b.dataset.s));
reset('r60');
</script></body></html>'''

out = HTML.replace('%LIGHT%', light).replace('%DARK%', dark).replace('%FONT%', font)
open('q1-simulation.html', 'w').write(out)
print(len(out))
