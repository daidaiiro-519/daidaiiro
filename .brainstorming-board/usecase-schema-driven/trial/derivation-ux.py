# SPDX-License-Identifier: MIT
# Copyright (c) 2026 daidaiiro
"""「候補を出し、検出信号を常時当てる」が完成したときの作業台を、動く形で組む。

    python3 derivation-ux.py    derivation-ux.html を書き出す

**これは仕様ではなく、試作である。**論点1 の答え（導出できるのは境界の候補まで。
AI は検出信号の常時適用へ回す）と、利用者の提唱（ユースケースは提供価値を含むべきで、
価値に直結しないユースケースは要らない）を、同じ画面の上で動かして確かめるために作る。

題材は利用者が挙げた基幹システムの構図（ERP ・ MES ・ SCM ・ WMS）から採る。
「在庫」が4つの箱に出ている、という指摘がそのまま検出信号の題材になる。
"""
from __future__ import annotations

import html
import json
import pathlib

# ── 題材 ──────────────────────────────────────────────────────
# 提供価値。ユースケースは、このどれかに直結する（しないものは "" を持つ）。
VALUES = [
    {"id": "v1", "name": "出荷遅延をゼロにする"},
    {"id": "v2", "name": "在庫を持たずに欠品を防ぐ"},
    {"id": "v3", "name": "不適合を後工程へ流さない"},
]

# ユースケース1件が持つもの ── 凝集の3条件（同じアクター ・ 同じ外部システム ・
# 密接に関係するデータ）と、提供価値。box は、いまその業務が載っている製品類型。
CASES = [
    {"id": "u1", "name": "受注を登録する", "actor": "営業", "ext": "CRM",
     "data": ["受注", "顧客"], "value": "v2", "box": "ERP"},
    {"id": "u2", "name": "与信を確認する", "actor": "営業", "ext": "外部与信",
     "data": ["顧客"], "value": "", "box": "ERP"},
    {"id": "u3", "name": "引当可能数を照会する", "actor": "営業", "ext": "",
     "data": ["在庫", "受注"], "value": "v2", "box": "ERP"},
    {"id": "u4", "name": "生産計画を立てる", "actor": "生産管理", "ext": "",
     "data": ["計画", "在庫"], "value": "v2", "box": "SCM"},
    {"id": "u5", "name": "作業指示を出す", "actor": "生産管理", "ext": "MES",
     "data": ["作業指示", "在庫"], "value": "v1", "box": "MES"},
    {"id": "u6", "name": "製造実績を記録する", "actor": "現場", "ext": "MES",
     "data": ["実績", "在庫"], "value": "v1", "box": "MES"},
    {"id": "u7", "name": "進捗を照会する", "actor": "生産管理", "ext": "MES",
     "data": ["実績"], "value": "v1", "box": "MES"},
    {"id": "u8", "name": "検査結果を記録する", "actor": "品質", "ext": "",
     "data": ["検査", "不適合"], "value": "v3", "box": "QMS"},
    {"id": "u9", "name": "不適合を隔離する", "actor": "品質", "ext": "",
     "data": ["不適合", "在庫"], "value": "v3", "box": "QMS"},
    {"id": "u10", "name": "是正措置を起票する", "actor": "品質", "ext": "",
     "data": ["是正", "不適合"], "value": "v3", "box": "QMS"},
    {"id": "u11", "name": "入庫を計上する", "actor": "倉庫", "ext": "WMS",
     "data": ["在庫", "棚番"], "value": "v1", "box": "WMS"},
    {"id": "u12", "name": "ピッキングを指示する", "actor": "倉庫", "ext": "WMS",
     "data": ["在庫", "棚番"], "value": "v1", "box": "WMS"},
    {"id": "u13", "name": "出荷を確定する", "actor": "倉庫", "ext": "WMS",
     "data": ["出荷", "在庫"], "value": "v1", "box": "WMS"},
    {"id": "u14", "name": "勤怠を締める", "actor": "人事", "ext": "勤怠SaaS",
     "data": ["勤怠"], "value": "", "box": "ERP"},
    {"id": "u15", "name": "月次で仕訳を出す", "actor": "経理", "ext": "会計",
     "data": ["仕訳"], "value": "", "box": "ERP"},
]

# 外から受け取る2問。価値ごとに1回だけ聞く ── ユースケースごとに聞くと15回になる。
ASKS = [
    {"id": "diff", "q": "この価値を、他社と同じやり方にしてしまってよいか",
     "no": "違いを生む", "yes": "同じでよい"},
    {"id": "pkg", "q": "実績あるパッケージ製品で賄えるか",
     "no": "賄えない", "yes": "賄える"},
]


def build() -> str:
    data = json.dumps({"values": VALUES, "cases": CASES, "asks": ASKS},
                      ensure_ascii=False)
    return _PAGE.replace("/*DATA*/", data)


_PAGE = r"""<!doctype html><html lang="ja"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>導出の作業台</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Shippori+Mincho:wght@600&family=Zen+Kaku+Gothic+New:wght@400;500;700&family=JetBrains+Mono:wght@400;700&display=swap">
<style>
:root{--paper:#eef1ef;--card:#f8faf9;--ink:#111d1a;--muted:#5b6b66;--rule:#ccd8d4;
--rule-soft:#dfe7e4;--sunk:#e7ecea;--key:#0d5c55;--key-soft:#d5e6e3;--warn:#8f5410;
--warn-soft:#f0e2cd;--dim:#7d8a86;
--shadow:0 1px 0 rgba(17,29,26,.06),0 8px 22px -18px rgba(17,29,26,.5)}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--paper:#0e1614;--card:#16211e;
--ink:#e6eeeb;--muted:#93a29d;--rule:#2a3936;--rule-soft:#22302d;--sunk:#111a18;--key:#6cc9b8;
--key-soft:#173330;--warn:#d9a35f;--warn-soft:#33281a;--dim:#7b8985;
--shadow:0 1px 0 rgba(0,0,0,.4),0 10px 26px -18px #000}}
:root[data-theme="dark"]{--paper:#0e1614;--card:#16211e;--ink:#e6eeeb;--muted:#93a29d;
--rule:#2a3936;--rule-soft:#22302d;--sunk:#111a18;--key:#6cc9b8;--key-soft:#173330;
--warn:#d9a35f;--warn-soft:#33281a;--dim:#7b8985}
*{box-sizing:border-box}
body{margin:0;background:var(--paper);color:var(--ink);font-size:15px;line-height:1.85;
font-family:"Zen Kaku Gothic New","Hiragino Kaku Gothic ProN",system-ui,sans-serif;
-webkit-font-smoothing:antialiased}
.wrap{max-width:54rem;margin:0 auto;padding:1.8rem 1.15rem 5rem;display:flex;flex-direction:column;gap:1.4rem}
h1{font-family:"Shippori Mincho",serif;font-weight:600;font-size:1.7rem;margin:0;line-height:1.4}
.eyebrow{font-family:"JetBrains Mono",monospace;font-size:.68rem;letter-spacing:.16em;
text-transform:uppercase;color:var(--muted)}
p{margin:0}
header{border-bottom:1px solid var(--rule);padding-bottom:1.1rem;display:flex;
flex-direction:column;gap:.55rem}
.lede{color:var(--muted);font-size:.95rem}
.lede b{color:var(--ink)}
/* ── 手順 ── */
.step{background:var(--card);border:1px solid var(--rule-soft);border-radius:.6rem;
box-shadow:var(--shadow);padding:1.15rem 1.2rem;display:flex;flex-direction:column;gap:.75rem}
.sh{display:flex;gap:.65rem;align-items:baseline;flex-wrap:wrap}
.sn{font-family:"JetBrains Mono",monospace;font-size:.72rem;font-weight:700;color:var(--paper);
background:var(--key);border-radius:.25rem;padding:.12rem .5rem;flex:none}
.sh h2{font-family:"Shippori Mincho",serif;font-weight:600;font-size:1.1rem;margin:0}
.sh .do{font-size:.8rem;color:var(--warn);font-weight:700}
.say{font-size:.88rem;color:var(--muted)}
.say b{color:var(--ink)}
.out{background:var(--sunk);border-radius:.45rem;padding:.8rem .85rem}
.outh{font-size:.74rem;letter-spacing:.06em;color:var(--muted);font-weight:700;margin-bottom:.45rem}
/* ── ユースケース ── */
.ucs{display:grid;grid-template-columns:repeat(auto-fill,minmax(15.5rem,1fr));gap:.4rem}
.uc{border:1px solid var(--rule-soft);border-radius:.4rem;padding:.4rem .55rem;
background:var(--paper);cursor:pointer;transition:border-color .12s,opacity .12s}
.uc:hover{border-color:var(--key)}
.uc.sel{border-color:var(--key);background:var(--key-soft)}
.uc.off{opacity:.38}
.uc .n{font-size:.86rem;font-weight:500}
.uc .m{font-size:.7rem;color:var(--muted);font-family:"JetBrains Mono",monospace}
.tag{display:inline-block;font-size:.64rem;font-weight:700;border-radius:.2rem;
padding:.02em .4em;border:1px solid currentColor;white-space:nowrap}
.tag.v{color:var(--key)}.tag.none{color:var(--warn)}
/* ── 候補 ── */
.cand{border:1px solid var(--rule-soft);border-left:3px solid var(--key);border-radius:.4rem;
padding:.6rem .75rem;background:var(--paper)}
.cand+.cand{margin-top:.5rem}
.cand.loose{border-left-color:var(--warn)}
.cand.hit{border-color:var(--key);background:var(--key-soft)}
.cand h3{margin:0;font-size:.95rem;font-weight:700}
.cand .why{font-size:.76rem;color:var(--muted)}
.cand .mem{font-size:.8rem}
/* ── 信号 ── */
.sig{border:1px solid var(--rule-soft);border-radius:.4rem;padding:.5rem .7rem;background:var(--paper)}
.sig+.sig{margin-top:.4rem}
.sig.on{border-color:var(--warn);background:var(--warn-soft)}
.sig .t{font-size:.85rem;font-weight:700;display:flex;gap:.45rem;align-items:center}
.sig .d{font-size:.78rem;color:var(--muted)}
.dotm{width:.5rem;height:.5rem;border-radius:50%;background:var(--dim);flex:none}
.sig.on .dotm{background:var(--warn)}
/* ── 問いと結果 ── */
.ask{display:flex;gap:.6rem;align-items:center;flex-wrap:wrap;font-size:.84rem;
padding:.3rem 0;border-top:1px dashed var(--rule-soft)}
.ask .q{flex:1;min-width:16rem}
.bts{display:flex;gap:.3rem}
button{font:inherit;font-size:.78rem;padding:.2rem .65rem;border-radius:.3rem;cursor:pointer;
border:1px solid var(--rule);background:var(--paper);color:var(--ink)}
button[aria-pressed="true"]{background:var(--key);border-color:var(--key);color:var(--paper);font-weight:700}
.big{font-size:.86rem;padding:.35rem 1rem;font-weight:700}
table{border-collapse:collapse;width:100%;font-size:.84rem}
th,td{border:1px solid var(--rule);padding:.4rem .6rem;text-align:left;vertical-align:top}
th{background:var(--sunk);font-size:.74rem;color:var(--muted)}
.verdict{font-weight:700}
.verdict.core{color:var(--key)}.verdict.wait{color:var(--warn)}
.bar{display:flex;gap:.6rem;align-items:center;flex-wrap:wrap}
footer{border-top:1px solid var(--rule);padding-top:1rem;color:var(--muted);font-size:.85rem;
display:flex;flex-direction:column;gap:.4rem}
footer b{color:var(--ink)}
code{font-family:"JetBrains Mono",monospace;font-size:.85em;background:var(--sunk);
padding:.05rem .3rem;border-radius:.2rem}
</style></head><body><div class="wrap">
<header>
 <p class="eyebrow">試作 ／ usecase-schema-driven</p>
 <h1>導出の作業台</h1>
 <p class="lede"><b>上から順に読む画面である。</b>ユースケースを入れると業務領域の候補が出て、
 誤りの信号が常時点く ── そこまでが機械の仕事で、<b>最後の2問だけ人が答える</b>。
 題材は基幹システムの構図から採った。</p>
 <div class="bar">
  <button id="demo" class="big">見本を入れて、最後まで見る</button>
  <span class="say" id="demonote">押すと、手順4 の問いに答えが入る。</span>
 </div>
</header>

<section class="step">
 <div class="sh"><span class="sn">手順1</span><h2>ユースケースを並べる</h2>
  <span class="do">← あなたが書く</span></div>
 <p class="say">1件が持つのは、<b>誰が</b>（アクター）・<b>どの外部システムと</b>・<b>何を触るか</b>（データ）
 ・そして<b>どの提供価値に直結するか</b>である。
 <br><b>価値に直結しない件は、使わない</b> ── 下のボタンで切り替えると、何が変わるかが見える。</p>
 <div class="bar">
  <button id="filt" aria-pressed="true">価値に直結する件だけを使う</button>
  <span class="say" id="filtnote"></span>
 </div>
 <div class="out"><div class="outh" id="uccount"></div><div class="ucs" id="ucs"></div></div>
 <p class="say">1件押すと、それが手順2 のどの候補に入ったかが光る。</p>
</section>

<section class="step">
 <div class="sh"><span class="sn">手順2</span><h2>業務領域の候補が出る</h2>
  <span class="do">← 機械が出す</span></div>
 <p class="say">同じ価値を共有し、同じデータを触るユースケースが1つの集まりになる。これが候補である。
 <br><b>「確定」ではない。</b>原典が「囲みが正しいと確定させる図ではない」と言っているので、
 ここは候補で止める。</p>
 <div class="out"><div class="outh" id="candcount"></div><div id="cands"></div></div>
</section>

<section class="step">
 <div class="sh"><span class="sn">手順3</span><h2>誤りの信号が、常時点く</h2>
  <span class="do">← 機械が当て続ける</span></div>
 <p class="say">候補が正しいかは機械には言えない。<b>言えるのは「おかしい」だけである。</b>
 原典が挙げる4つの信号を、書いたものから毎回判定する
 ── <b>ここが AI の持ち場で、導出ではない。</b></p>
 <div class="out"><div class="outh" id="sigcount"></div><div id="sigs"></div></div>
</section>

<section class="step">
 <div class="sh"><span class="sn">手順4</span><h2>2問だけ、人が答える</h2>
  <span class="do">← あなたが答える</span></div>
 <p class="say">この2問は<b>ユースケースをいくら並べても出てこない</b> ── 他社と市場を見ないと答えが無いからである。
 <br><b>ただし価値ごとに聞くので、3回で済む。</b>ユースケースごとに聞けば15回になる。</p>
 <div class="out"><div class="outh">価値ごとに2問</div><div id="asks"></div></div>
</section>

<section class="step">
 <div class="sh"><span class="sn">手順5</span><h2>実装方法が決まる</h2>
  <span class="do">← 分岐で決まる</span></div>
 <p class="say">手順4 の答えでカテゴリーが決まり、カテゴリーから実装方法が決まる。
 <br><b>ドメインモデルは、4つのうちの1つでしかない</b> ── 補完や一般に当たった候補で
 ドメインモデルを作ると、過剰設計になる。</p>
 <div class="out"><div class="outh">候補ごとの行き先</div><div id="result"></div></div>
</section>

<footer id="foot"></footer>
</div>
<script>
const DATA = /*DATA*/;
const S = {filter:true, sel:null, ans:{}};

const val = id => DATA.values.find(v=>v.id===id);
const used = () => DATA.cases.filter(c => !S.filter || c.value);

function candidates(){
  const out = [];
  for(const v of DATA.values){
    const mem = used().filter(c=>c.value===v.id);
    if(mem.length) out.push({id:v.id, name:v.name, kind:"value", mem});
  }
  const rest = used().filter(c=>!c.value);
  const byActor = {};
  for(const c of rest){ (byActor[c.actor] ||= []).push(c); }
  for(const [a,mem] of Object.entries(byActor))
    out.push({id:"a-"+a, name:a+"（価値に直結しない）", kind:"loose", mem});
  return out;
}

function signals(cands){
  const sig = [];
  const where = {};
  for(const c of cands) for(const m of c.mem) for(const d of m.data)
    (where[d] ||= new Set()).add(c.name);
  const multi = Object.entries(where).filter(([,s])=>s.size>1)
    .sort((a,b)=>b[1].size-a[1].size);
  sig.push({on:multi.length>0, t:"同じ語が、複数の候補にまたがる",
    d: multi.length
      ? multi.map(([d,s])=>`「${d}」が ${s.size}件（${[...s].join(" ／ ")}）`).join("　")
        + " ── 同じものなら分散、別のものなら別の言葉である。どちらかの判定が要る"
      : "またがる語は無い"});
  const owner = {};
  for(const d of Object.keys(where)){
    let best = null, n = -1;
    for(const c of cands){
      const k = c.mem.filter(m=>m.data.includes(d)).length;
      if(k > n){ n = k; best = c.name; }
    }
    owner[d] = best;
  }
  const dep = new Set();
  for(const c of cands) for(const m of c.mem) for(const d of m.data)
    if(owner[d] && owner[d] !== c.name) dep.add(c.name + "→" + owner[d]);
  const pairs = [...new Set([...dep].filter(e=>{
    const [a,b] = e.split("→"); return dep.has(b + "→" + a);
  }).map(e=>e.split("→").sort().join(" ⇄ ")))];
  sig.push({on:pairs.length>0, t:"依存が、双方向になっている",
    d: pairs.length
      ? pairs.join("　") + " ── 互いのデータを触り合っている。片側へ寄せるか、境界を引き直すか"
      : "依存は一方向に収まっている"});
  const split = cands.filter(c=>new Set(c.mem.map(m=>m.box)).size>1);
  sig.push({on:split.length>0, t:"候補が、いまのシステムの箱をまたぐ",
    d: split.length
      ? split.map(c=>`${c.name} → ${[...new Set(c.mem.map(m=>m.box))].join(" ・ ")}`).join("　")
        + " ── 箱は境界ではなく、いまの姿を記述する語彙である"
      : "候補は、いまの箱に収まっている"});
  const waiting = cands.filter(c=>c.kind==="value" && verdict(c.id).k==="wait").length;
  sig.push({on:waiting>0, t:"カテゴリーが、まだ決まっていない",
    d: waiting ? `${waiting}件の候補が、手順4 の答えを待っている ── 実装方法はまだ選べない`
               : "全ての候補にカテゴリーが付いた"});
  return sig;
}

function verdict(vid){
  const a = S.ans[vid] || {};
  if(a.diff===undefined || a.pkg===undefined)
    return {k:"wait", label:"入力待ち", impl:"── 手順4 に答えると決まる"};
  if(a.diff===false) return {k:"core", label:"中核",
    impl:"ドメインモデル ── 自分たちで作る"};
  if(a.pkg===true) return {k:"gen", label:"一般",
    impl:"既製品 ・ 外部サービス ── 自分たちでは作らない"};
  return {k:"sup", label:"補完",
    impl:"トランザクションスクリプト ／ アクティブレコード ── 手の込んだ設計にしない"};
}

function render(){
  const cands = candidates();
  const sig = signals(cands);
  const selCase = DATA.cases.find(c=>c.id===S.sel);

  document.getElementById("ucs").innerHTML = DATA.cases.map(c=>{
    const off = S.filter && !c.value;
    const v = c.value ? `<span class="tag v">${val(c.value).name}</span>`
                      : `<span class="tag none">価値に直結しない</span>`;
    return `<div class="uc${off?" off":""}${S.sel===c.id?" sel":""}" data-u="${c.id}">
      <div class="n">${c.name}　${v}</div>
      <div class="m">${c.actor}${c.ext?" ／ "+c.ext:""} ／ ${c.data.join(" ")} ／ いまは ${c.box}</div></div>`;
  }).join("");
  document.getElementById("uccount").textContent =
    `${DATA.cases.length} 件のうち、${used().length} 件を使う`;

  document.getElementById("cands").innerHTML = cands.map(c=>{
    const hit = selCase && c.mem.some(m=>m.id===selCase.id);
    const data = [...new Set(c.mem.flatMap(m=>m.data))];
    return `<div class="cand${c.kind==="loose"?" loose":""}${hit?" hit":""}">
      <h3>${c.name}</h3>
      <div class="mem">${c.mem.map(m=>m.name).join(" ／ ")}</div>
      <div class="why">共有するデータ： ${data.join(" ・ ")}</div></div>`;
  }).join("");
  document.getElementById("candcount").textContent =
    `候補 ${cands.length} 件` + (selCase ? `　── いま光っているのは「${selCase.name}」の行き先` : "");

  document.getElementById("sigs").innerHTML = sig.map(s=>
    `<div class="sig${s.on?" on":""}"><div class="t"><span class="dotm"></span>${s.t}</div>
     <div class="d">${s.d}</div></div>`).join("");
  document.getElementById("sigcount").textContent =
    `4つのうち ${sig.filter(s=>s.on).length} つが点いている`;

  document.getElementById("asks").innerHTML = DATA.values.map(v=>{
    const a = S.ans[v.id]||{};
    const rows = DATA.asks.map(q=>{
      const cur = a[q.id];
      return `<div class="ask"><span class="q">${q.q}</span><span class="bts">
        <button data-v="${v.id}" data-q="${q.id}" data-a="0" aria-pressed="${cur===false}">${q.no}</button>
        <button data-v="${v.id}" data-q="${q.id}" data-a="1" aria-pressed="${cur===true}">${q.yes}</button>
        </span></div>`;
    }).join("");
    return `<div class="cand"><h3>${v.name}</h3>${rows}</div>`;
  }).join("");

  document.getElementById("result").innerHTML =
    `<table><tr><th>候補</th><th>カテゴリー</th><th>実装方法</th></tr>` +
    cands.map(c=>{
      const vd = c.kind==="value" ? verdict(c.id)
        : {k:"gen", label:"候補にしない", impl:"価値に直結しないので、この一覧から外す"};
      return `<tr><td>${c.name}</td>
        <td class="verdict ${vd.k}">${vd.label}</td><td>${vd.impl}</td></tr>`;
    }).join("") + `</table>`;

  const done = DATA.values.filter(v=>{
    const a=S.ans[v.id]||{}; return a.diff!==undefined && a.pkg!==undefined;}).length;
  document.getElementById("filtnote").textContent = S.filter
    ? "価値の無い3件を外している"
    : "全件を使っている ── 価値に直結しない件が、候補に混じる";
  document.getElementById("foot").innerHTML =
    `<div>人が答えたのは <b>${done} / ${DATA.values.length}</b> 件の価値ぶん、つまり <b>${done*2} 問</b>。
     ユースケースごとに聞けば ${DATA.cases.length*2} 問になる。</div>
     <div><b>ここで言えるのは「この一覧に対しては、候補が立った」までである。</b>
     最適であることは証明していない ── 唯一の正解が無いので、証明する手立てが無い。</div>
     <div>集約の設計は、この画面では出さない。原典が
     「適切なトランザクション境界を最初から見つけることは、ほとんど不可能である」と言っているためで、
     ここから先は反復になる。</div>`;
}

document.addEventListener("click", e=>{
  const u = e.target.closest("[data-u]");
  if(u){ S.sel = S.sel===u.dataset.u ? null : u.dataset.u; render(); return; }
  const b = e.target.closest("button[data-q]");
  if(b){ (S.ans[b.dataset.v] ||= {})[b.dataset.q] = b.dataset.a==="1"; render(); return; }
  if(e.target.id==="filt"){ S.filter=!S.filter;
    e.target.setAttribute("aria-pressed", S.filter);
    e.target.textContent = S.filter ? "価値に直結する件だけを使う" : "全件を使う";
    render(); return; }
  if(e.target.id==="demo"){
    /* 見本 ── 出荷遅延は違いを生む（中核）、欠品防止も中核、不適合は賄える（一般） */
    S.ans = {v1:{diff:false, pkg:false}, v2:{diff:false, pkg:false}, v3:{diff:true, pkg:true}};
    document.getElementById("demonote").textContent =
      "入れた。手順3 の4つ目が消え、手順5 が埋まっている。";
    render(); }
});
render();
</script>
"""


def main() -> int:
    out = pathlib.Path(__file__).resolve().parent / "derivation-ux.html"
    out.write_text(build(), encoding="utf-8")
    print("書き出し:", out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
