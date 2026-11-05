"""開発シミュレーターの頁を sim.json から作る（作業用。コミットしない）。run.py のあとに動かす。"""
import json, os

H = os.path.dirname(os.path.abspath(__file__))
data = json.load(open(os.path.join(H, 'sim.json')))
blob = json.dumps(data, ensure_ascii=False).replace('</', '<\\/')

PAGE = r'''<title>開発シミュレーター</title>
<meta name="claude:viewport-fit-sides" content="cover">
<style>
/* 1コマンドの流れを骨組みにする：上＝どこを見ているか、中＝流れの5つの箱（要約）、下＝選んだ箱の中身。境界は枠と見出しの帯で示す */
:root{--bg:#EEF3F1;--card:#FFFFFF;--band:#F5F8F7;--ink:#13201D;--sub:#5B6B66;--line:#C9D6D2;--accent:#0D5C55;--accent-bg:#D5E6E3;--bad:#9B2C2C;--bad-bg:#F8E1E1;--hot:#8A4B00;--hot-bg:#FBEBD9;--code:#F3F6F5;--add:#DDF3E4;--add-ink:#14532D;--del:#FBE3E3;--del-ink:#7F1D1D;--mark:#FFF1A8;--mark-ink:#5A4A00;
--font:"Noto Sans JP","Hiragino Kaku Gothic ProN",system-ui,sans-serif;--mono:"JetBrains Mono",ui-monospace,monospace}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--bg:#0C1311;--card:#17211E;--band:#1C2824;--ink:#E6EEEB;--sub:#9DADA8;--line:#2C3B37;--accent:#6CC9B8;--accent-bg:#173330;--bad:#F09A9A;--bad-bg:#3A1E1E;--hot:#F2B47A;--hot-bg:#3A2A1A;--code:#121B18;--add:#14341F;--add-ink:#A7E8BC;--del:#3A1A1A;--del-ink:#F5B5B5;--mark:#3D3610;--mark-ink:#F3E28A;color-scheme:dark}}
:root[data-theme="dark"]{--bg:#0C1311;--card:#17211E;--band:#1C2824;--ink:#E6EEEB;--sub:#9DADA8;--line:#2C3B37;--accent:#6CC9B8;--accent-bg:#173330;--bad:#F09A9A;--bad-bg:#3A1E1E;--hot:#F2B47A;--hot-bg:#3A2A1A;--code:#121B18;--add:#14341F;--add-ink:#A7E8BC;--del:#3A1A1A;--del-ink:#F5B5B5;--mark:#3D3610;--mark-ink:#F3E28A;color-scheme:dark}
*{box-sizing:border-box}
html,body{height:100%}
body{margin:0;background:var(--bg);color:var(--ink);font:14px/1.6 var(--font);overflow:hidden;padding-block:10px;padding-left:max(16px,env(safe-area-inset-left,0px));padding-right:max(16px,env(safe-area-inset-right,0px))}
main{height:100%;max-width:1560px;margin:0 auto;display:grid;grid-template-columns:300px minmax(0,1fr);gap:10px}
.mainc{display:grid;grid-template-rows:auto auto auto minmax(0,1fr);gap:8px;min-height:0;min-width:0}
main>*{min-width:0}
h1{font-size:15px;margin:0;white-space:nowrap}h2{font-size:14px;margin:0;min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
h3{font-size:12.5px;margin:0 0 4px;color:var(--sub)}
.meta{color:var(--sub);font-size:12px}
code,.mono{font-family:var(--mono);font-size:12px}
.box{background:var(--card);border:1px solid var(--line);border-radius:12px;min-width:0;overflow:hidden}
.band{background:var(--band);border-bottom:1px solid var(--line);padding:6px 12px;display:flex;flex-wrap:wrap;gap:6px 10px;align-items:center;min-width:0}
.pad{padding:8px 12px;display:flex;flex-wrap:wrap;gap:6px 10px;align-items:center;min-width:0}
.chip{display:inline-flex;align-items:center;gap:4px;font-size:11.5px;font-weight:700;border-radius:999px;padding:0 8px;border:1px solid var(--line);color:var(--sub);background:var(--card);white-space:nowrap}
.chip.ok{color:var(--add-ink);background:var(--add);border-color:transparent}.chip.ng{color:var(--bad);background:var(--bad-bg);border-color:transparent}
.chip.q{color:var(--hot);background:var(--hot-bg);border-color:transparent}.chip.ac{color:var(--accent);border-color:var(--accent)}
button{font:inherit;font-size:12.5px;padding:2px 10px;border-radius:999px;border:1px solid var(--line);background:var(--card);color:var(--ink);cursor:pointer;white-space:nowrap}
button:hover{border-color:var(--accent)}button:focus-visible,select:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
button[aria-pressed="true"]{background:var(--accent);color:var(--card);border-color:var(--accent)}
select{font:inherit;font-size:12.5px;padding:2px 6px;border-radius:8px;border:1px solid var(--line);background:var(--card);color:var(--ink);flex:1 1 260px;min-width:0}
.sep{width:1px;align-self:stretch;background:var(--line)}
/* 流れの5つの箱 */
.flow{display:grid;grid-template-columns:minmax(0,1fr) 22px minmax(0,1fr) 22px minmax(0,1fr) 22px minmax(0,1fr) 22px minmax(0,1fr) 14px minmax(0,.9fr);align-items:stretch}
.stage.cap{background:var(--band);border-style:dashed}.stage.cap .v{white-space:normal;font-size:12.5px;line-height:1.45}.stage.cap .hd{background:transparent}
.arw{display:flex;align-items:center;justify-content:center;color:var(--sub);font-size:16px}
.stage{all:unset;box-sizing:border-box;cursor:pointer;background:var(--card);border:1px solid var(--line);border-radius:12px;display:flex;flex-direction:column;min-width:0;overflow:hidden;position:relative}
.stage .hd{background:var(--band);border-bottom:1px solid var(--line);padding:4px 10px;font-size:12px;font-weight:700;color:var(--sub);display:flex;gap:6px;align-items:center}
.stage .hd .no{display:inline-grid;place-items:center;width:18px;height:18px;border-radius:50%;border:1px solid var(--sub);font-size:11px}
.stage .bd{padding:6px 10px;display:flex;flex-direction:column;gap:3px;min-width:0}
.stage .v{font-size:15px;font-weight:700;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.stage .s{font-size:11.5px;color:var(--sub);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.stage:hover{border-color:var(--accent)}.stage:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
.stage[aria-pressed="true"]{background:var(--card);color:var(--ink);border:2px solid var(--accent);box-shadow:0 0 0 3px var(--accent-bg)}
.stage[aria-pressed="true"] .hd{background:var(--accent);color:var(--card);border-bottom-color:var(--accent)}.stage[aria-pressed="true"] .hd .no{border-color:var(--card)}
@media (max-width:820px){.flow{display:flex;overflow-x:auto;gap:6px}.arw,.flow>span:not(.arw){display:none}.stage{flex:0 0 150px}}
/* 下：中身 */
.detail{border:2px solid var(--accent);display:grid;grid-template-rows:auto minmax(0,1fr)}
.detail>.band{background:var(--accent-bg);border-bottom-color:var(--accent)}
.detail>.band b{color:var(--accent)}
.body{min-height:0;overflow:auto;padding:10px 12px}
.cols{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:10px;height:100%;min-height:0}
.cols.code{grid-template-columns:minmax(0,4fr) minmax(0,7fr)}
.cols>.box{display:grid;grid-template-rows:auto minmax(0,1fr)}
.cols>.box>.in{overflow:auto;min-height:0;padding:8px 10px;display:flex;flex-direction:column;gap:8px}
.cols>.box>.in>*{flex:none}
@media (max-width:820px){.cols,.cols.code{grid-template-columns:minmax(0,1fr);grid-template-rows:minmax(0,1fr) minmax(0,1fr)}}
.tw{overflow-x:auto}table{border-collapse:collapse;width:100%;font-size:12.5px}
th,td{border:1px solid var(--line);padding:3px 7px;text-align:left;vertical-align:top}th{background:var(--band);font-size:11.5px}
pre{margin:0;font:11.5px/1.5 var(--mono);background:var(--code);border-radius:8px;padding:4px 0;overflow:auto}
pre .l{display:block;padding:0 8px;white-space:pre}pre .l.a{background:var(--add);color:var(--add-ink)}pre .l.d{background:var(--del);color:var(--del-ink)}pre .l.h{background:var(--mark);color:var(--mark-ink)}
pre .n{display:inline-block;width:3em;color:var(--sub);user-select:none;text-align:right;margin-right:8px}
.file{border:1px solid var(--line);border-radius:10px;overflow:hidden;min-width:0;flex:none}
.file .fh{display:flex;flex-wrap:wrap;gap:6px;align-items:center;padding:3px 8px;background:var(--band);font-size:11.5px;border-bottom:1px solid var(--line)}
.file pre{border-radius:0}
details.file>summary{list-style:none;cursor:pointer}details.file>summary::-webkit-details-marker{display:none}details.file:not([open])>summary{border-bottom:0}
.kind{border:1px solid var(--line);border-radius:10px;overflow:hidden;flex:none}
.kind>.kh{background:var(--band);border-bottom:1px solid var(--line);padding:3px 10px;font-size:12px;font-weight:700;display:flex;gap:8px;align-items:center}
.kind>.kb{padding:8px 10px;display:flex;flex-direction:column;gap:8px}
.kind.empty>.kh{border-bottom:0}.kind.empty .kh span{font-weight:400;color:var(--sub)}
.two{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:8px}
@media (max-width:820px){.two{grid-template-columns:minmax(0,1fr)}}
.two>div{display:flex;flex-direction:column;gap:6px;min-width:0}
.none{color:var(--sub);font-size:12.5px}
.lane{border:1px solid var(--line);border-radius:10px;overflow:hidden;flex:none}
.lane>.kh{background:var(--band);border-bottom:1px solid var(--line);padding:3px 10px;font-size:12px;font-weight:700}
.fns{display:flex;flex-wrap:wrap;gap:4px;padding:6px}
.fn{display:inline-flex;flex-direction:column;align-items:flex-start;border-radius:8px;padding:1px 8px;max-width:100%}
.fn .nm{font:11.5px/1.4 var(--mono);font-weight:700;overflow-wrap:anywhere}.fn .pth{font:10.5px/1.3 var(--mono);opacity:.75;overflow-wrap:anywhere}
.mx td,.mx th{text-align:center;font-size:11.5px;padding:2px 6px}.mx th:first-child,.mx td:first-child{text-align:left;white-space:nowrap}
.mx td.on{background:var(--accent-bg);color:var(--accent);font-weight:700}.mx td.now{outline:2px solid var(--hot);outline-offset:-2px;background:var(--hot-bg);color:var(--hot)}
.ex{display:flex;gap:6px;align-items:baseline;font-size:12.5px}.ex code{overflow-wrap:anywhere}

/* 目次 */
.toc{display:grid;grid-template-rows:auto minmax(0,1fr);min-height:0}
.toc .list{overflow:auto;padding:6px 8px 10px;display:flex;flex-direction:column;gap:6px}
.tg{font-size:12px;font-weight:700;color:var(--sub);padding:4px 4px 0}
.sc{all:unset;box-sizing:border-box;display:flex;gap:6px;align-items:center;cursor:pointer;border-radius:8px;padding:4px 8px;font-size:12.5px;line-height:1.4;width:100%}
.sc .nm{flex:1;min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.sc:hover{background:var(--band)}.sc:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
.sc .id{display:block;font:10.5px var(--mono);color:var(--sub)}
.sc[aria-current="true"]{background:var(--accent-bg);font-weight:700;box-shadow:inset 3px 0 0 var(--accent)}
.tag{flex:none;font-size:10.5px;font-weight:700;border-radius:999px;padding:0 6px;white-space:nowrap}.tag.pass{color:var(--add-ink);background:var(--add)}.tag.stop{color:var(--hot);background:var(--hot-bg)}
.steps{list-style:none;margin:2px 0 6px 12px;padding:0 0 0 8px;border-left:2px solid var(--line);display:flex;flex-direction:column;gap:1px}
.stp{all:unset;box-sizing:border-box;cursor:pointer;display:grid;grid-template-columns:18px minmax(0,1fr) auto;gap:6px;align-items:center;padding:2px 6px;border-radius:6px;font-size:12px;line-height:1.4;width:100%}
.stp .tx{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.stp:hover{background:var(--card)}.stp:focus-visible{outline:2px solid var(--accent)}
.stp .k{display:inline-grid;place-items:center;width:17px;height:17px;border-radius:50%;border:1px solid var(--sub);font-size:10px}
.stp[aria-current="true"]{background:var(--band);font-weight:700}
.stp[aria-current="true"] .k{background:var(--accent);border-color:var(--accent);color:var(--card)}
.star{color:var(--hot);font-weight:700}
.goal{display:flex;flex-direction:column;gap:4px;align-items:stretch}
.goal .t{font-size:16px;font-weight:700;text-wrap:balance}
.stepbar{display:flex;flex-wrap:wrap;gap:8px;align-items:center}
.stepbar .now{font-size:15px;font-weight:700;flex:1 1 320px;min-width:0}
.big{font-size:13px;padding:4px 14px}
.whybox{border-left:4px solid var(--accent);background:var(--accent-bg);border-radius:0 8px 8px 0;padding:4px 10px;font-size:13.5px;line-height:1.6}
.whybox b{color:var(--accent)}
.basis{font-size:12.5px}.basis summary{cursor:pointer;color:var(--sub)}.basis table{margin-top:4px}
.tocbtn{display:none}
@media (max-width:900px){main{grid-template-columns:minmax(0,1fr)}.toc{display:none}.toc.open{display:grid;position:fixed;inset:10px;z-index:5;box-shadow:0 8px 30px rgba(0,0,0,.3)}.tocbtn{display:inline-flex}}
</style>
<main>
<aside class="box toc" id="toc" aria-label="目次">
<div class="band" style="flex-direction:column;align-items:stretch;gap:4px"><div style="display:flex;gap:6px;align-items:center"><h1>開発シミュレーター</h1><button class="tocbtn" id="tocclose" style="margin-left:auto">閉じる</button></div><span class="meta">ステップごとに、実装済みの機能を実際の実行結果で確認できます</span><span id="stages" style="display:flex;gap:4px;flex-wrap:wrap"></span><button id="terms" style="align-self:flex-start">コマンドと用語</button></div>
<div class="list" id="list"></div>
</aside>
<div class="mainc">
<section class="box"><div class="pad goal">
<div class="row"><button class="tocbtn" id="tocopen">目次</button><span id="crumb" class="meta"></span></div>
<div class="t" id="goal"></div>
<div id="why" class="whybox"></div>
<div id="exp" class="row" style="gap:4px 8px"></div>
</div></section>
<section class="box"><div class="pad stepbar"><button id="prev" class="big">← 前の手順</button><span class="now" id="now"></span><button id="next" class="big">次の手順 →</button></div></section>
<nav class="flow" id="flow" aria-label="この手順の処理の流れ"></nav>
<section class="box detail"><div class="band" id="dh"></div><div class="body" id="db"></div></section>
</div>
</main>
<script id="data" type="application/json">__DATA__</script>
<script>
const D=JSON.parse(document.getElementById('data').textContent);
const $=id=>document.getElementById(id);
const esc=s=>String(s).replace(/[&<>"]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
const KIND={schema:'Concrete',glossary:'glossary',instance:'インスタンス',summary:'変更のまとめ',broken:'JSON でない'};
const KINDS=[['schema','JSON Schema',['schema','glossary']],['instance','インスタンス',['instance','broken']],['summary','変更のまとめ（コミットする）',['summary']],['change','変更（DB）',null],['session','セッション（DB）',null]];
const LATER={change:'記録なし（このシナリオでは変更を記録していない）',session:'未実装（ステップ9 で作成）'};
const DBT={change:['changes','change_items','intent_judgments'],session:['sessions','session_changes']};
const LAYERS=[['inbound','CLI','crates/inbound/'],['application','ユースケース','crates/application/'],['domain','ドメイン','crates/domain/'],['outbound','外部接続','crates/outbound/']];
const CMDS=['search','get','create','patch','import','migrate','validate','judge','render','history','record','release','session'];
const TARGETS=[['schema','スキーマ（Concrete）'],['glossary','glossary'],['instance','インスタンス'],['change','変更'],['summary','変更のまとめ'],['all','作業の根の下のすべて'],['judgment','判定'],['session','セッション']];
const STAGES=[['before','事前状態'],['input','コマンドと入力'],['code','実行されたコード'],['output','出力'],['after','事後状態']];
const flat=[];D.scenarios.forEach((s,i)=>s.steps.forEach((st,j)=>flat.push([i,j])));
let cur=0,sel='after',fnSel=null;
const keep=(k,v)=>{try{localStorage.setItem(k,v)}catch(e){}};const load=k=>{try{return localStorage.getItem(k)}catch(e){return null}};
{const v=+load('sim-pos2');if(v>=0&&v<flat.length)cur=v;const t=load('sim-sel');if(STAGES.some(x=>x[0]===t)||t==='can'||t==='terms')sel=t}
// 手順を文にする：何をしたか → 何が起きたか
function opText(op){const p=op.path;const v=op.value;const what=p==='/'?'根':p;
 if(op.op==='add'){if(v&&typeof v==='object'&&v.$ref)return `${what} に term の参照（${v.$ref.split('#')[1]}）を追加`;return `${what} を追加`}
 if(op.op==='replace')return `${what} を置換`;if(op.op==='remove')return `${what} を削除`;return `${what}（${op.op}）`}
function gitText(a){const c=a[1],r=a.slice(2).filter(x=>!x.startsWith('-'));
 if(c==='init')return 'Git のリポジトリを作る';if(c==='add')return 'Git：変更をコミットの対象にする';
 if(c==='commit'){const i=a.indexOf('-qm');return `Git：コミット「${i>=0?a[i+1]:''}」`}
 if(c==='checkout')return a.includes('-b')?`Git：ブランチ ${r[0]} を作って移る（ほかの人の作業）`:`Git：ブランチ ${r[0]} に戻る`;
 if(c==='merge')return `Git：ブランチ ${r[0]} を取り込む`;if(c==='switch')return a.includes('-c')?`Git：ブランチ ${a[a.indexOf('-c')+1]} を作って移る`:`Git：ブランチ ${r[0]} に戻る`;if(c==='stash')return a.includes('pop')?'Git：stash した変更を戻す':'Git：変更を stash する';if(c==='branch'&&a.includes('-m'))return `Git：ブランチ名を ${r[r.length-2]} から ${r[r.length-1]} に変える`;if(c==='worktree')return `Git：ブランチ ${a[a.indexOf('-b')+1]} のワークツリーを ${a[a.length-1]} に立てる`;return 'Git：'+a.slice(1).join(' ')}
function doText(st){const w=st.cwd?`［${st.cwd.replace('.worktrees/','ワークツリー ')}］`:'';return w+doText0(st)}
function doText0(st){const a=st.args,c=a[0],t=a[1]||'';if(c==='git')return gitText(a);
 if(c==='record'&&t==='--summary')return '変更のまとめを作る（前回のコミットからの変更と、記録を突き合わせる）';
 if(c==='validate'&&t==='.')return '作業の根の下のすべてを検証';
 if(c==='create')return a.includes('--schema')?`Concrete からインスタンスを作成（${t}）`:a.includes('glossary')?`glossary を作成（${t}）`:`Abstract から Concrete の雛形を作成（${t}）`;
 if(c==='patch'){let ops=[];try{ops=JSON.parse(a[a.indexOf('--patch')+1])}catch(e){}const im=a.includes('--if-match')?'（読んだときの ETag を確かめて）':'';return im+(ops.length>3?`${ops.length} か所を変更（${ops.slice(0,2).map(opText).join('、')} ほか）`:ops.map(opText).join('、'))}
 if(c==='get'){const i=a.indexOf('--pointer');return `${i>=0?a[i+1]:'全体'} を取得（ETag も返る）`}
 if(c==='validate')return `${t} を検証`;
 if(c==='record'){const i=a.indexOf('--intent');return `変更を記録：「${i>=0?a[i+1]:''}」`}
 if(c==='judge'){const i=a.indexOf('--result');return `人が判定：${i>=0?a[i+1]:''}（${a[1]}）`}
 if(c==='history')return `${t} の経緯を見る`;return a.join(' ')}
function gotText(st){const o=st.output;if(st.args[0]==='git')return st.exit===0?'成功':'終了コード '+st.exit;
 if(o&&st.args[1]==='--summary'&&o.ok&&o.data)return `まとめ 1 件 ・ 記録 ${o.data.records} 件 ・ 例外 ${o.data.exceptions} 件`;
 if(o&&st.args[0]==='validate'&&st.args[1]==='.'&&o.data&&o.data.files)return `${o.data.files.length} ファイルを検証 ・ 違反 ${o.findings.length} 件`;
 if(!o)return st.exit===2?'使い方の誤り':st.exit!==0?'終了コード '+st.exit:({create:'作成',patch:'保存'}[st.args[0]]||'成功');
 const f=o.findings;if(st.args[0]==='get'){if(!o.ok)return '該当なし（refused）';const v=o.data&&o.data.value;const j=JSON.stringify(v);return j.length>40?j.slice(0,40)+'…':j}
 if(f.some(x=>x.result==='refused'))return 'エラー（入力を受け付けない）';
 if(o.data&&o.data.saved===false)return `エラー（違反 ${f.length} 件）・ ファイルは変更なし`;
 if(f.some(x=>x.result==='violated'))return `violated ${f.length} 件`;
 if(f.some(x=>x.result==='error'))return `error ${f.length} 件`;
 if(st.args[0]==='validate')return '検証成功';return o.data&&o.data.saved?'保存 ・ 検証成功':'成功'}
const isKey=st=>st.phase==='when';
const COMMANDS=[
 ['create <置き場所>','スキーマを作る：--owner --kind --value ／ 用語集：--kind glossary --value ／ 記録：--schema <スキーマの名前> --value [--id]','作った中身の場所 ・ ETag ・ 決まりごとの結果 ・ 状態 ・ まだ書いていない項目'],
 ['get <置き場所>','[--pointer <場所>]','値 ・ ETag'],
 ['patch <置き場所>','--patch <JSON Patch：add ・ replace ・ remove> [--if-match <読んだときの ETag>]','保存したか ・ 新しい ETag ・ 決まりごとの結果 ・ 状態 ・ まだ書いていない項目'],
 ['validate <置き場所>','（なし）','決まりごとの結果 ・ 状態'],
 ['record <置き場所>','--intent <意図> --purpose <目的> [--pointer <変えた項目>]… [--writer human|ai-agent] [--id]','記録の id ・ 記録の中身'],
 ['record --summary','[--id]','まとめの置き場所（.semantic-agent-schema/changes/<id>.json）・ 入った記録の数 ・ 例外の数 ・ まとめの中身'],
 ['validate .','（なし）','ファイルごとの状態 ・ すべての違反（場所は「置き場所#項目」）'],
 ['judge <記録の id>','--result verified|violated|unverified --meaning <意味> --reason <理由>','判定した結果'],
 ['history <置き場所>','（なし）','その対象を変えた記録と、意図の判定'],
];
const TERMS=[
 ['ETag','記録やスキーマの中身から計算する短い値。中身が変わると別の値になる。書くときに --if-match で渡すと、読んだあとに誰も書き換えていないかを確かめられる（楽観ロック）'],
 ['スキーマ（Concrete）','記録の決まりを書いたファイル。どんな項目があり、どう書くかを決める。例：ACDR の記録のスキーマ（acdr/decision-record.schema.json）'],
 ['記録（インスタンス）','スキーマに従って書いた実際の1件。例：ACDR 0199 の記録（.acdr/0199-依存の向き/acdr.json）'],
 ['Abstract','スキーマの書き方の決まり。どのスキーマもこれに従う。基盤が持つ'],
 ['JSON Schema','JSON の形（どんな項目が必須か、何の型か）を書くための標準の書き方。スキーマはこれで書く'],
 ['$id ・ $schema','$id はスキーマの名前。$schema は「どの決まりに従うか」で、スキーマなら Abstract の名前、記録ならスキーマの名前を書く'],
 ['$ref','別の場所にある定義を指す書き方。例：用語集の言葉を項目から指す'],
 ['必須（required）','書かないとエラーになる項目'],
 ['x-prompt','項目ごとの「読み方と書き方のメモ」。read が読み方、write が書くときの決まりの並び'],
 ['決まり（rule）','書くときに守ること。1件ずつ、誰が判定するか（by）を持つ'],
 ['判定する者（by）','machine（機械が自動で確かめる）・ ai（AI に判断を頼む）・ human（人が判断する）'],
 ['判定役（x-delegates）','AI やツールに判定を任せるときの相手の定義'],
 ['独自の項目（x-extensions）','基盤に無い項目を、スキーマが自分で定義する場所'],
 ['置き場所の型（x-instance-path）','記録ファイルの置き場所の決め方。{no} のような所に項目の値が入る'],
 ['用語集（glossary）','言葉の意味をまとめたファイル。スキーマの項目から言葉を指せる'],
 ['結果の5つ','verified（合格）・ violated（違反）・ unverified（まだ判定していない）・ unsupported（判定する仕組みがまだ無い）・ error（判定の途中で失敗）'],
 ['状態','結果から決まる記録やスキーマの状態。下書き（違反あり）・ 確かめ直しを待つ ・ 判断待ち（AI や人の判定待ち）・ 検証済み'],
 ['変更のまとめ（change summary）','コミットごとに1件のファイル。前回のコミットからの変更を、目的 → 意図 → 変えた項目の前後の順にまとめる。基盤が記録から機械的に作り、AI は書かない。PR を見る人が読む'],
 ['例外','まとめの先頭に出る、確かめてほしい所。unrecorded（記録に無い書き換え）・ no-change（書き換えの無い記録）・ intent-mismatch（意図と合わない）・ human-field-changed（人が決める項目を承認なしで変えた）'],
 ['ブランチが無くなった記録（branch-gone）','記録したあとにブランチ名を変えたなどで、書いたブランチがもう無い記録。いまのブランチのまとめに入れ、元の名前を例外として出す'],
 ['取り込み（merge）','ほかの人のコミットを自分のブランチに入れること。入れたあとに validate . で、すべてを検証し直す'],
];
const last=p=>{const t=p.split('/').filter(Boolean);return t.length?t[t.length-1].replace(/~1/g,'/').replace(/~0/g,'~'):'全体'};
function shortText(st){const a=st.args,c=a[0];if(c==='git'){const m={init:'Git を作る',add:'ステージ',commit:'コミット',merge:'取り込む',worktree:'ワークツリーを立てる',switch:'ブランチを切り替える',stash:'stash',branch:'ブランチ名を変える'};return a[1]==='checkout'?(a.includes('-b')?'ブランチを作る':'ブランチに戻る'):(m[a[1]]||'Git')}
 if(c==='record'&&a[1]==='--summary')return 'まとめを作る';if(c==='validate'&&a[1]==='.')return 'すべてを検証';
 if(c==='create')return a.includes('--schema')?'インスタンスを作成':a.includes('glossary')?'glossary を作成':'雛形を作成';
 if(c==='patch'){let ops=[];try{ops=JSON.parse(a[a.indexOf('--patch')+1])}catch(e){}
  if(ops.length!==1)return ops.length+' か所を変更';const o=ops[0];
  if(o.op==='add'&&o.value&&o.value.$ref)return last(o.path)+' で term を参照';
  if(o.op==='add'&&['x-extensions','properties','$defs','x-delegates'].includes(last(o.path))&&o.value&&typeof o.value==='object')return last(o.path)+' に '+Object.keys(o.value).join('・')+' を追加';
  return last(o.path)+(o.op==='add'?' を追加':o.op==='replace'?' を置換':o.op==='remove'?' を削除':'（'+o.op+'）')}
 if(c==='get'){const i=a.indexOf('--pointer');return (i>=0?last(a[i+1]):'全体')+' を取得'}
 if(c==='validate')return '検証';if(c==='record')return '変更を記録';if(c==='judge')return '人が承認';if(c==='history')return '経緯を見る';return c}
function verdict(st){const o=st.output;if(st.exit===0)return ['pass','成功'];
 if(o&&o.findings.some(f=>f.result==='refused'))return ['stop','エラー'];if(o&&o.data&&o.data.saved===false)return ['stop','エラー'];return ['stop','違反あり']}
function targetOf(st){if(st.args[0]==='git')return 'git';if(st.args[1]==='--summary')return 'summary';if(st.args[0]==='validate'&&st.args[1]==='.')return 'all';if(st.args[0]==='record'||st.args[0]==='history')return 'change';if(st.args[0]==='judge')return 'judgment';const p=st.args[1]||'';const b=st.before.files[p]||st.after.files[p];if(st.args[0]==='create'&&st.args.includes('glossary'))return 'glossary';return b?(b.kind==='broken'?'instance':b.kind):'schema'}
function ptr(doc,p){if(p==='')return doc;let v=doc;for(const t of p.slice(1).split('/').map(x=>x.replace(/~1/g,'/').replace(/~0/g,'~'))){if(v==null||typeof v!=='object'||!(t in v))return undefined;v=v[t]}return v}
function checkExp(s,e){const st=e.of==='when'?s.steps.find(x=>x.phase==='when'):s.steps.filter(x=>x.phase==='after')[+e.of.replace(/\D/g,'')];if(!st)return [false,'記録なし',null];
 const out=[];let ok=true;if('exit' in e){ok=ok&&st.exit===e.exit;out.push('終了コード '+e.exit)}
 if(e.pointer!==undefined){const g=ptr(st.output,e.pointer);
  if('equals' in e){ok=ok&&JSON.stringify(g)===JSON.stringify(e.equals);out.push(e.pointer+' = '+JSON.stringify(e.equals))}
  if('contains' in e){ok=ok&&JSON.stringify(g??'').includes(e.contains);out.push(e.pointer+' に「'+e.contains+'」')}
  if('count' in e){ok=ok&&Array.isArray(g)&&g.length===e.count;out.push(e.pointer+' の数 '+e.count)}
  if('exists' in e){ok=ok&&(g!==undefined)===e.exists;out.push(e.pointer+(e.exists?' あり':' なし'))}}
 return [ok,out.join(' ・ '),st]}
function unchangedOk(s){const w=s.steps.find(x=>x.phase==='when');return s.unchanged.every(p=>(w.before.files[p]||{}).text===(w.after.files[p]||{}).text)}
const lines=t=>t.replace(/\n$/,'').split('\n');
function diff(a,b){const n=a.length,m=b.length;const L=Array.from({length:n+1},()=>new Int32Array(m+1));for(let i=n-1;i>=0;i--)for(let j=m-1;j>=0;j--)L[i][j]=a[i]===b[j]?L[i+1][j+1]+1:Math.max(L[i+1][j],L[i][j+1]);
 const r=[];let i=0,j=0;while(i<n&&j<m){if(a[i]===b[j]){r.push([' ',b[j]]);i++;j++}else if(L[i+1][j]>=L[i][j+1]){r.push(['-',a[i]]);i++}else{r.push(['+',b[j]]);j++}}while(i<n)r.push(['-',a[i++]]);while(j<m)r.push(['+',b[j++]]);return r}
function pretty(t){try{return JSON.stringify(JSON.parse(t),null,1)}catch(e){return t}}
function folded(rows){const keep=rows.map((r,i)=>r[0]!==' '||rows.slice(Math.max(0,i-2),i+3).some(x=>x[0]!==' '));const out=[];let gap=0;
 rows.forEach((r,i)=>{if(keep[i]){if(gap){out.push(['…',`… ${gap} 行 変更なし …`]);gap=0}out.push(r)}else gap++});if(gap)out.push(['…',`… ${gap} 行 変更なし …`]);return out}
const pre=rows=>'<pre>'+rows.map(([k,t])=>k==='…'?`<span class="l" style="color:var(--sub);font-style:italic">${esc(t)}</span>`:`<span class="l ${k==='+'?'a':k==='-'?'d':''}">${esc(k==='+'?'+ ':k==='-'?'- ':'  ')}${esc(t)}</span>`).join('')+'</pre>';
const plain=t=>pre(lines(t).map(x=>[' ',x]));
function changed(st){return Object.keys({...st.before.files,...st.after.files}).filter(p=>(st.before.files[p]||{}).text!==(st.after.files[p]||{}).text)}
function fileBox(p,f,body,badge,open){const h=`<code>${esc(p)}</code><span class="chip">${KIND[f.kind]}</span>${badge}`;return open?`<div class="file"><div class="fh">${h}</div>${body}</div>`:`<details class="file"><summary class="fh">${h}<span class="meta">クリックで表示</span></summary>${body}</details>`}
function kindBlock(l,inner,empty){return empty?`<div class="kind empty"><div class="kh">${l}<span>${esc(empty)}</span></div></div>`:`<div class="kind"><div class="kh">${l}</div><div class="kb">${inner}</div></div>`}
function rowsOf(snap){const out={};for(const [p,tables] of Object.entries(snap.dbs))for(const [t,v] of Object.entries(tables))out[t]=new Set(v.rows.map(r=>JSON.stringify(r)));return out}
function dbAdded(st){const b=rowsOf(st.before),a=rowsOf(st.after);let n=0;for(const [t,rows] of Object.entries(a))for(const r of rows)if(!(b[t]&&b[t].has(r)))n++;return n}
function rowKey(cols,r){const i=cols.indexOf('seq');return cols.includes('tool')?JSON.stringify(r):JSON.stringify(i>=0?[r[0],r[i]]:[r[0]])}
function dbTables(snap,k,prev){return Object.entries(snap.dbs).map(([p,tables])=>[p,Object.fromEntries(Object.entries(tables).filter(([t,v])=>DBT[k].includes(t)&&v.rows.length))]).filter(([p,t])=>Object.keys(t).length).map(([p,tables])=>`<div class="file"><div class="fh"><code>${esc(p.split('/').slice(-2).join('/'))}</code><span class="meta">手元の DB（1つ ・ コミットしない）</span></div>`+Object.entries(tables).map(([t,v])=>{
 const pv=prev&&prev.dbs[p]&&prev.dbs[p][t];const old={};if(pv)pv.rows.forEach(r=>old[rowKey(pv.columns,r)]=r);
 return `<div class="tw"><table><tr><th colspan="${v.columns.length+1}">${esc(t)}</th></tr><tr><th></th>${v.columns.map(c=>`<th>${esc(c)}</th>`).join('')}</tr>${v.rows.map(r=>{const o=prev?old[rowKey(v.columns,r)]:r;const added=prev&&!o;const upd=!added&&o&&JSON.stringify(o)!==JSON.stringify(r);
  const mark=added?'<span class="chip ok">追加</span>':upd?'<span class="chip q">変更</span>':'';
  return `<tr${added?' style="background:var(--add);color:var(--add-ink)"':''}><td>${mark}</td>`+r.map((c,i)=>{const ch=upd&&JSON.stringify(o[i])!==JSON.stringify(c);return ch?`<td class="mono" style="background:var(--add);color:var(--add-ink)"><s style="opacity:.6">${esc(o[i])}</s> → <b>${esc(c)}</b></td>`:`<td class="mono">${esc(c)}</td>`}).join('')+'</tr>'}).join('')}</table></div>`}).join('')+'</div>').join('')}
// ① 事前状態：その時点の中身
function vBefore(st){return '<div style="display:flex;flex-direction:column;gap:8px">'+KINDS.map(([k,l,kinds])=>{
 if(!kinds){const t=dbTables(st.before,k);return kindBlock(l,t,t?'':LATER[k])}
 const fs=Object.entries(st.before.files).filter(([p,f])=>kinds.includes(f.kind));
 return kindBlock(l,fs.map(([p,f])=>fileBox(p,f,plain(pretty(f.text)),'',true)).join(''),fs.length?'':'なし')}).join('')+'</div>'}
// ⑤ 事後状態：事前と並べて、変わった所を差分で
function vAfter(st){return '<div style="display:flex;flex-direction:column;gap:8px">'+KINDS.map(([k,l,kinds])=>{
 if(!kinds){const a=dbTables(st.after,k,st.before);return kindBlock(l,a?`<p class="meta">実行後の DB の中身。このコマンドで足した行は「追加」、値が変わった項目は「前 → 後」で示す</p>${a}`:'',a?'':LATER[k])}
 const ps=[...new Set(Object.keys(st.before.files).concat(Object.keys(st.after.files)))].filter(p=>kinds.includes((st.after.files[p]||st.before.files[p]).kind));
 if(!ps.length)return kindBlock(l,'','なし');
 return kindBlock(l,ps.map(p=>{const o=st.before.files[p],f=st.after.files[p];
  if(!o)return fileBox(p,f,pre(lines(pretty(f.text)).map(t=>['+',t])),'<span class="chip ok">新規</span>',true);
  if(!f)return fileBox(p,o,pre(lines(pretty(o.text)).map(t=>['-',t])),'<span class="chip ng">削除</span>',true);
  if(o.text===f.text)return fileBox(p,f,plain(pretty(f.text)),'<span class="chip">変更なし</span>',false);
  const dl=diff(lines(pretty(o.text)),lines(pretty(f.text)));const nm=dl.filter(r=>r[0]==='-').length,np=dl.filter(r=>r[0]==='+').length;
  return `<div class="two"><div><h3>事前</h3>${fileBox(p,o,plain(pretty(o.text)),'',false)}</div><div><h3>事後（差分：変わった行の前後だけ。削除 ${nm} 行 ・ 追加 ${np} 行）</h3>${fileBox(p,f,pre(folded(dl)),'<span class="chip q">変更</span>',true)}</div></div>`}).join(''))}).join('')+'</div>'}
function vInput(st){const pi=st.args.indexOf('--patch');
 const rows=[['コマンド',st.args[0]],['対象',st.args[1]||'']];for(let i=2;i<st.args.length;i++){if(st.args[i].startsWith('--')){const v=st.args[i+1]&&!st.args[i+1].startsWith('--')?st.args[++i]:'（値なし）';rows.push([st.args[i-(v==='（値なし）'?0:1)],v])}}
 return `<div class="cols"><div class="box"><div class="band"><b>実行したコマンド</b></div><div class="in">${plain('$ '+(st.args[0]==='git'?'':'semantic-agent-schema ')+st.args.map(a=>/[\s"{}\[\]]/.test(a)?"'"+a+"'":a).join(' '))}<div class="tw"><table><tr><th>引数</th><th>値</th></tr>${rows.map(([k,v])=>`<tr><td class="mono">${esc(k)}</td><td class="mono">${esc(k==='--patch'?'右の JSON Patch を参照':v)}</td></tr>`).join('')}</table></div></div></div>`+
  `<div class="box"><div class="band"><b>入力の JSON Patch</b></div><div class="in">${pi>=0?plain(JSON.stringify(JSON.parse(st.args[pi+1]),null,1)):'<span class="none">このコマンドに JSON Patch はありません</span>'}</div></div></div>`}
function vCode(st){if(fnSel===null||!st.code[fnSel]){fnSel=st.code.findIndex(f=>f.file.startsWith('crates/application/src/use_cases/'));if(fnSel<0)fnSel=0}
 const left=LAYERS.map(([l,en,pfx])=>{const fs=st.code.map((f,i)=>[f,i]).filter(([f])=>f.file.startsWith(pfx));return `<div class="lane"><div class="kh">${l}（${en}）　${fs.length}</div><div class="fns">`+(fs.map(([f,i])=>`<button class="fn" aria-pressed="${i===fnSel}" data-f="${i}"><span class="nm">${esc(f.name)}${f.count>1?' ×'+f.count:''}</span><span class="pth">${esc(f.file.replace(pfx,''))}</span></button>`).join('')||'<span class="none">なし</span>')+'</div></div>'}).join('');
 const f=st.code[fnSel];let right='<span class="none">なし</span>',title='ソース';
 if(f){const src=D.sources[f.file]||[];const hit=new Set(f.lines);const a=Math.max(1,f.start-2),z=Math.min(src.length,f.end+1);title=`<code>${esc(f.name)}</code><span class="meta">${esc(f.file)}:${f.start} ・ 色付きの行が実行された行</span>`;
  right='<pre>';for(let n=a;n<=z;n++)right+=`<span class="l ${hit.has(n)?'h':''}"><span class="n">${n}</span>${esc(src[n-1]??'')}</span>`;right+='</pre>'}
 return `<div class="cols code"><div class="box"><div class="band"><b>実行された関数（層ごと。呼び出し順ではありません）</b></div><div class="in">${left}</div></div><div class="box"><div class="band">${title}</div><div class="in">${right}</div></div></div>`}
function vOutput(s,st){const o=st.output;
 const left=(o?`<div style="display:flex;gap:6px;flex-wrap:wrap"><span class="chip ${st.exit===0?'ok':st.exit===1?'q':'ng'}">終了コード ${st.exit}（${st.exit===0?'正常':st.exit===1?'検出あり':'使い方の誤り'}）</span><span class="chip ${o.ok?'ok':'q'}">ok：${o.ok}</span></div>`:'')
  +(o&&o.findings.length?'<div class="tw"><table><tr><th>result</th><th>rule</th><th>pointer</th><th>reason</th><th>by</th></tr>'+o.findings.map(f=>`<tr><td><span class="chip ${f.result==='refused'?'ng':f.result==='verified'?'ok':'q'}">${esc(f.result)}</span></td><td class="mono">${esc(f.rule||'')}</td><td class="mono">${esc(f.pointer)}</td><td>${esc(f.reason)}</td><td class="mono">${esc(f.by)}</td></tr>`).join('')+'</table></div>':'<span class="none">findings なし</span>')
  +(o&&o.data&&o.data.state?`<div style="display:flex;gap:6px;flex-wrap:wrap;align-items:center;margin-top:6px"><b style="font-size:13px">状態</b><span class="chip ${o.data.state.name==='verified'?'ok':o.data.state.name==='draft'?'ng':'q'}">${esc(o.data.state.label)}</span>`+['verified','violated','unverified','unsupported','error'].map(v=>{const n=(o.data.results||[]).filter(r=>r.result===v).length;return n?`<span class="chip">${v} ${n}</span>`:''}).join('')+'</div>':'')
  +(o&&o.data&&Array.isArray(o.data.results)&&o.data.results.length?'<details style="margin-top:4px"><summary class="meta" style="cursor:pointer">rule ごとの結果（'+o.data.results.length+' 件）を開く</summary><div class="tw"><table><tr><th>rule</th><th>結果</th><th>判定する者</th><th>場所</th><th>根拠</th></tr>'+o.data.results.map(r=>`<tr><td class="mono">${esc(r.rule)}</td><td><span class="chip ${r.result==='verified'?'ok':r.result==='violated'||r.result==='error'?'ng':'q'}">${esc(r.result)}</span></td><td class="mono">${esc(r.by)}</td><td class="mono">${esc(r.pointer)}</td><td>${esc(r.reason)}</td></tr>`).join('')+'</table></div></details>':'')
  +(o&&o.data&&Array.isArray(o.data.unfilled)&&o.data.unfilled.length?'<h3 style="margin-top:6px">まだ書いていない項目（Abstract の案内）</h3><div class="tw"><table><tr><th>項目</th><th>名前（title）</th><th>何を表すか（description）</th><th>書くときの rule</th></tr>'+o.data.unfilled.map(u=>`<tr><td class="mono">${esc(u.name)}</td><td>${esc(u.title??'')}</td><td>${esc(u.description??'')}</td><td class="mono">${esc((u.rules||[]).join(' ・ '))}</td></tr>`).join('')+'</table></div>':'')
  +'<h3 style="margin-top:6px">期待値と実際の出力</h3>'+s.then.map(e=>{const [ok,t]=checkExp(s,e);return `<div class="ex"><span class="chip ${ok?'ok':'ng'}">${ok?'一致':'不一致'}</span><span class="meta">${esc(e.of==='when'?'★の手順':'確認'+(+e.of.replace(/\D/g,'')+1))}</span><code>${esc(t)}</code></div>`}).join('')+(s.unchanged.length?`<div class="ex"><span class="chip ${unchangedOk(s)?'ok':'ng'}">${unchangedOk(s)?'一致':'不一致'}</span><span class="meta">★の手順</span><code>変更されない：${s.unchanged.map(esc).join(' ・ ')}</code></div>`:'');
 return `<div class="cols"><div class="box"><div class="band"><b>結果</b></div><div class="in">${left}</div></div><div class="box"><div class="band"><b>標準出力${o?'（--json）':''}</b></div><div class="in">${o?plain(JSON.stringify(o,null,1)):plain((st.stdout+st.stderr)||'（無し）')}</div></div></div>`}
function vCan(st){const used={};const nowK=targetOf(st)+'|'+st.args[0];D.scenarios.forEach(x=>x.steps.forEach(y=>used[targetOf(y)+'|'+y.args[0]]=1));
 const mx='<div class="tw"><table class="mx"><tr><th>対象</th>'+CMDS.map(c=>`<th class="mono">${c}</th>`).join('')+'</tr>'+TARGETS.map(([t,l])=>'<tr><td>'+l+'</td>'+CMDS.map(c=>{const k=t+'|'+c;return `<td class="${used[k]?'on':''} ${k===nowK?'now':''}">${used[k]?'●':''}</td>`}).join('')+'</tr>').join('')+'</table></div>';
 const sg='<div class="tw"><table><tr><th>ステップ</th><th>内容</th><th>状態</th></tr>'+D.stages.map(g=>{const n=D.scenarios.filter(x=>x.stage===g.no).length;return `<tr><td>${esc(g.no)}</td><td>${esc(g.title)}</td><td><span class="chip ${g.status==='done'?'ok':g.status==='doing'?'q':''}">${g.status==='done'?'完了':g.status==='doing'?'作業中':'未着手'} ・ シナリオ ${n}</span></td></tr>`}).join('')+'</table></div>';
 return `<div class="cols"><div class="box"><div class="band"><b>対象 × コマンド（● 実装済み ・ 枠：表示中のコマンド）</b></div><div class="in">${mx}</div></div><div class="box"><div class="band"><b>ステップ</b></div><div class="in">${sg}</div></div></div>`}

function render(){const [si,sj]=flat[cur];const s=D.scenarios[si],st=s.steps[sj];keep('sim-pos2',cur);keep('sim-sel',sel);
 // 目次
 $('stages').innerHTML=D.stages.map(g=>{const t=g.status==='done'?'完了':g.status==='doing'?'作業中':'未着手';return `<span class="chip ${g.status==='done'?'ac':g.status==='doing'?'q':''}" title="${esc(g.title)}">${esc(g.no)} ${t}</span>`}).join('');
 $('list').innerHTML=D.stages.map(g=>{const ss=D.scenarios.map((x,i)=>[x,i]).filter(([x])=>x.stage===g.no);
  return `<div class="tg">${esc(g.no)}　${esc(g.title)}${g.status==='doing'?'（作業中）':ss.length?'':'（未着手）'}</div>`+ss.map(([x,i])=>{const on=i===si;
   const allOk=x.then.every(e=>checkExp(x,e)[0])&&unchangedOk(x);
   const key=x.steps.find(isKey);const [vc,vt]=verdict(key);
   return `<div><button class="sc" data-s="${i}" aria-current="${on}" title="${esc(x.title)}"><span class="nm">${allOk?'':'✗ '}${esc(x.name)}</span><span class="tag ${vc}">${vt}</span></button>`+(on?'<ol class="steps">'+x.steps.map((y,k)=>{const [c2,t2]=verdict(y);return `<li><button class="stp" data-k="${k}" aria-current="${k===sj}" title="${esc(doText(y))} → ${esc(gotText(y))}"><span class="k">${k+1}</span><span class="tx">${isKey(y)?'<span class="star">★ </span>':''}${esc(shortText(y))}</span><span class="tag ${c2}">${t2}</span></button></li>`}).join('')+'</ol>':'')+'</div>'}).join('')}).join('');
 $('list').querySelectorAll('.sc').forEach(b=>b.onclick=()=>{const i=+b.dataset.s;cur=flat.findIndex(([a])=>a===i);fnSel=null;$('toc').classList.remove('open');render()});
 $('list').querySelectorAll('.stp').forEach(b=>b.onclick=()=>{cur=flat.findIndex(([a,c])=>a===si&&c===+b.dataset.k);fnSel=null;$('toc').classList.remove('open');render()});
 const cs=$('list').querySelector('.stp[aria-current="true"]');if(cs)cs.scrollIntoView({block:'nearest'});
 // シナリオの目的と期待値
 $('crumb').textContent=`${s.stage}　${D.stages.find(g=>g.no===s.stage).title}　›　${s.id}`;
 $('goal').innerHTML=esc(s.name)+'<div style="font-size:13px;font-weight:400;color:var(--sub);margin-top:2px">確認すること：'+esc(s.title)+'</div>';
 const key=s.steps.find(isKey);const seen=new Set();
 ((key.output&&key.output.findings)||[]).forEach(f=>{if(f.rule&&D.rules[f.rule])seen.add(f.rule)});
 (s.why.match(/rule ([a-z0-9-]+)/g)||[]).forEach(m=>{const r=m.slice(5);if(D.rules[r])seen.add(r)});
 const basis='<details class="basis"><summary>根拠を開く（使う場面 ・ 不変条件 '+s.traces.length+' 件'+(seen.size?' ・ Abstract の rule '+seen.size+' 件':'')+'）</summary><div class="tw"><table><tr><th>根拠</th><th>中身</th></tr>'
   +s.traces.map(t=>{const id=t.split(':')[0];const side=t.includes(':')?'（'+t.split(':')[1]+'）':'';return `<tr><td class="mono">${esc(t)}</td><td>${esc((D.names[id]||'')+side)}</td></tr>`}).join('')
   +[...seen].map(r=>`<tr><td class="mono">rule ${esc(r)}</td><td>${esc(D.rules[r].rule)}<br><span class="meta">理由：${esc(D.rules[r].why)}</span></td></tr>`).join('')+'</table></div></details>';
 $('why').innerHTML='<b>なぜ必要か</b>　'+esc(s.why)+(s.repo?'<br><b>準備</b>　Git のリポジトリの中で始める（実際の利用では、すでにリポジトリの中にいる）':'')+basis;
 $('exp').innerHTML=s.then.map(e=>{const [ok,t]=checkExp(s,e);return `<span class="chip ${ok?'ok':'ng'}" title="${esc(t)}">${ok?'✓':'✗'} ${esc(t)}</span>`}).join('')+(s.unchanged.length?`<span class="chip ${unchangedOk(s)?'ok':'ng'}">${unchangedOk(s)?'✓':'✗'} ★の手順で ${s.unchanged.map(esc).join('・')} が変更されない</span>`:'');
 // 手順
 $('now').innerHTML=`手順 ${sj+1} / ${s.steps.length}：${isKey(st)?'<span class="star">★ 確認ポイント　</span>':''}${esc(doText(st))}　<span class="chip ${st.exit===0?'ok':st.exit===1?'q':'ng'}">${esc(gotText(st))}</span>`;
 const lastStep=sj===s.steps.length-1,firstStep=sj===0;
 $('next').textContent=cur===flat.length-1?'最後の手順':lastStep?'次のシナリオへ →':'次の手順 →';$('next').disabled=cur===flat.length-1;
 $('prev').textContent=firstStep&&cur>0?'← 前のシナリオへ':'← 前の手順';$('prev').disabled=cur===0;
 // 流れの5つの箱
 const nb=Object.keys(st.before.files).length,ch=changed(st).length,o=st.output;
 const lay=LAYERS.map(([l,en,p])=>l+' '+st.code.filter(f=>f.file.startsWith(p)).length).join(' ・ ');
 const sum={before:[`ファイル ${nb}`,nb?Object.keys(st.before.files).join(' ・ '):'ファイルなし'],input:[st.args[0],st.args.slice(1).join(' ')],code:[`${st.code.length} 関数`,lay],
  output:[`終了コード ${st.exit}`,o?((o.data&&o.data.state?o.data.state.label+' ・ ':'')+(o.findings.length?o.findings.length+' 件 '+o.findings[0].result:'findings 0 件')):(st.stderr||'').trim()],after:(()=>{const d=dbAdded(st);const parts=[];if(ch)parts.push(`ファイル ${ch}`);if(d)parts.push(`DB ${d} 行`);return [parts.length?'変更 '+parts.join(' ・ '):'変更なし',ch?changed(st).join(' ・ '):d?'手元の DB の行が増えた ・ 変わった':'事前状態と同じ']})()};
 const cmds=[...new Set(D.scenarios.filter(x=>x.stage===s.stage).flatMap(x=>x.steps.filter(y=>y.args[0]!=='git').map(y=>y.args[1]==='--summary'?'record\u00a0--summary':y.args[0]==='validate'&&y.args[1]==='.'?'validate\u00a0.':y.args[0])))];
 $('flow').innerHTML=STAGES.map(([k,l],i)=>(i?'<span class="arw" aria-hidden="true">→</span>':'')+`<button class="stage" aria-pressed="${sel===k}" data-g="${k}"><span class="hd"><span class="no">${i+1}</span>${l}</span><span class="bd"><span class="v ${k==='input'||k==='code'?'mono':''}">${esc(sum[k][0])}</span><span class="s">${esc(sum[k][1])}</span></span></button>`).join('')
  +`<span></span><button class="stage cap" aria-pressed="${sel==='can'}" data-g="can"><span class="hd">使えるコマンド</span><span class="bd"><span class="v mono" style="font-size:13px">${cmds.map(c=>`<span style="white-space:nowrap">${esc(c)}</span>`).join(' ・ ')}</span><span class="s">クリックで対象 × コマンドの一覧</span></span></button>`;
 $('flow').querySelectorAll('.stage').forEach(b=>b.onclick=()=>{sel=b.dataset.g;render()});
 const lbl=sel==='terms'?'コマンドと用語':sel==='can'?s.stage+' で使えるコマンド':STAGES.findIndex(x=>x[0]===sel)+1+'　'+STAGES.find(x=>x[0]===sel)[1];
 const note={before:'コマンド実行前の作業ディレクトリ',input:'実行したコマンドと入力',terms:'いまのコマンドの形と、画面に出てくる言葉の意味',code:'カバレッジ計測による、このコマンドで実行された関数と行',output:'実行ファイルの出力と、シナリオの期待値との照合',after:'コマンド実行後の作業ディレクトリ。変更されたファイルは事前状態との差分を表示',can:'実装済みのコマンドとステップ'}[sel];
 $('dh').innerHTML=`<b>${lbl}</b><span class="meta">${note}</span>`;
 $('db').innerHTML=sel==='terms'?'<h3>コマンドの一覧（共通：--json で {ok, findings, data} を返す。終了コードは 0 正常 ・ 1 検出あり ・ 2 使い方の誤り）</h3><div class="tw"><table><tr><th>コマンド</th><th>引数</th><th>返すもの</th></tr>'+COMMANDS.map(([a,b,c])=>`<tr><td class="mono"><b>${esc(a)}</b></td><td class="mono">${esc(b)}</td><td>${esc(c)}</td></tr>`).join('')+'</table></div><h3 style="margin-top:10px">用語</h3><div class="tw"><table><tr><th>用語</th><th>意味</th></tr>'+TERMS.map(([a,b])=>`<tr><td><b>${esc(a)}</b></td><td>${esc(b)}</td></tr>`).join('')+'</table></div>':sel==='before'?vBefore(st):sel==='input'?vInput(st):sel==='code'?vCode(st):sel==='output'?vOutput(s,st):sel==='after'?vAfter(st):vCan(st);
 $('db').style.overflow=(sel==='code'||sel==='input'||sel==='output'||sel==='can')?'hidden':'auto';
 $('db').querySelectorAll('[data-f]').forEach(b=>b.onclick=()=>{fnSel=+b.dataset.f;render()});
}
$('prev').onclick=()=>{if(cur>0){cur--;fnSel=null;render()}};
$('next').onclick=()=>{if(cur<flat.length-1){cur++;fnSel=null;render()}};
$('terms').onclick=()=>{sel=sel==='terms'?'after':'terms';$('toc').classList.remove('open');render()};
$('tocopen').onclick=()=>$('toc').classList.add('open');$('tocclose').onclick=()=>$('toc').classList.remove('open');
document.addEventListener('keydown',e=>{if(e.key==='ArrowRight')$('next').click();if(e.key==='ArrowLeft')$('prev').click()});
render();
</script>
'''

open(os.path.join(H, 'index.html'), 'w').write(PAGE.replace('__DATA__', blob))
print('ok', os.path.getsize(os.path.join(H, 'index.html')))
