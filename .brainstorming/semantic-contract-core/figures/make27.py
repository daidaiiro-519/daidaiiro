# 第95回の案（Q10）：変更を人が取り込むかを決める場面のシミュレーション（screen）。
# 同じ変更を、Git の差分で見る場合と、Semantic Agent Schema の意味の差分で見る場合で左右に並べる。
import runpy
L = runpy.run_path('make19.py', run_name='lib')
light, dark, font = L['light'], L['dark'], L['font']

HTML = r'''<!doctype html><html lang="ja"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<style>
:root{%LIGHT%}
@media (prefers-color-scheme:dark){:root{%DARK%}}
*{box-sizing:border-box}
body{margin:0;padding:12px 16px;background:var(--page);color:var(--text);font:14px/1.7 %FONT%}
button{font:inherit;cursor:pointer;border:1px solid var(--linectl);background:var(--control);color:var(--text);border-radius:6px;padding:4px 14px}
button.key{background:var(--key);color:var(--onkey);border-color:var(--key)}
button:disabled{opacity:.4;cursor:default}
.lead{background:var(--block);border:1px solid var(--line);border-radius:8px;padding:10px 12px;margin-bottom:10px}
.nav{display:flex;align-items:center;gap:8px;flex-wrap:wrap;margin-bottom:10px}
.dots{display:flex;gap:4px}
.dots span{width:22px;height:6px;border-radius:3px;background:var(--head);border:1px solid var(--line)}
.dots span.on{background:var(--key);border-color:var(--key)}
.story{background:var(--answer);border:1px solid var(--key);border-radius:8px;padding:10px 12px;margin-bottom:10px}
.story .n{font-size:12px;color:var(--sub)}
.story .t{font-size:16px;font-weight:700}
.grid{display:grid;grid-template-columns:1fr 1fr;gap:10px}
@media (max-width:640px){.grid{grid-template-columns:1fr}}
.col{background:var(--block);border:1px solid var(--line);border-radius:8px;padding:10px;min-width:0}
.col h4{margin:0 0 2px;font-size:14px}
.col .sub{font-size:12px;color:var(--sub);margin-bottom:6px}
.ops{display:flex;gap:14px;align-items:baseline;margin:2px 0 8px;font-size:13px}
.ops b{font-size:20px;color:var(--key)}
.f{border:1px solid var(--line);background:var(--literal);border-radius:6px;padding:4px 8px;margin:4px 0;font-size:12px;overflow-x:auto}
.mono{font-family:"JetBrains Mono",monospace;font-size:11.5px;white-space:pre;line-height:1.5}
.add{color:var(--textdone)}.del{color:var(--textremoved)}
.ex{border:1px solid var(--textremoved);background:var(--removed);color:var(--textremoved);border-radius:6px;padding:4px 8px;margin:4px 0;font-size:13px}
.sec{font-weight:700;font-size:13px;margin:8px 0 2px}
.row{border:1px solid var(--line);background:var(--literal);border-radius:6px;margin:4px 0;font-size:13px}
.row>.h{display:flex;gap:6px;align-items:baseline;padding:4px 8px}
.row>.h .c{margin-left:auto;font-size:12px;color:var(--sub);white-space:nowrap}
.row.open{outline:2px solid var(--key)}
.row .body{border-top:1px solid var(--line);padding:4px 8px}
.kv{display:grid;grid-template-columns:auto 1fr 1fr;gap:2px 8px;font-size:12px}
.kv .hd{color:var(--sub)}
.under{padding-left:14px;color:var(--sub);font-size:12px}
.tag{border:1px solid var(--line);background:var(--head);border-radius:10px;padding:0 6px;font-size:11px;white-space:nowrap}
.tag.hu{background:var(--notice);color:var(--warn);border-color:var(--linenotice)}
.st{display:flex;gap:6px;align-items:center;font-size:13px;margin:3px 0}
.b-ok{white-space:nowrap;background:var(--done);color:var(--textdone);border:1px solid var(--linedone);border-radius:10px;padding:0 6px;font-size:12px}
.b-wait{white-space:nowrap;background:var(--notice);color:var(--warn);border:1px solid var(--linenotice);border-radius:10px;padding:0 6px;font-size:12px}
.b-ng{background:var(--removed);color:var(--textremoved);border-radius:10px;padding:0 6px;font-size:12px}
.pair{display:grid;grid-template-columns:1fr 1fr;gap:6px}
.card{border:1px solid var(--line);background:var(--page);border-radius:6px;padding:6px 8px;font-size:12.5px;min-width:0}
.card .cap{font-size:11px;color:var(--sub)}
.card h5{margin:2px 0;font-size:14px}
.chg{background:var(--notice);border-radius:3px;padding:0 2px}
.none{color:var(--sub);font-size:13px}
.why{background:var(--block);border:1px solid var(--line);border-radius:8px;padding:8px 12px;margin-top:10px}
.sum{display:grid;grid-template-columns:1fr 1fr;gap:10px}
.sum div{background:var(--block);border:1px solid var(--line);border-radius:8px;padding:8px 12px}
</style></head><body>
<div class="lead">記事を書く Skill を例に、<b>AI が出した大きな変更を、人が取り込むかを決める場面</b>を見る。同じ変更を、左は Git の差分で、右は Semantic Agent Schema が出す意味の差分で見る。「次へ」で1場面ずつ進む。</div>
<div class="nav"><button id="prev">前へ</button><button id="next" class="key">次へ</button><div class="dots" id="dots"></div><button id="again">はじめから</button></div>
<div class="story"><div class="n" id="no"></div><div class="t" id="title"></div><div id="detail"></div></div>
<div class="grid">
 <div class="col"><h4>Git の差分で見る</h4><div class="sub">PR のファイルの差分</div><div class="ops">人の判断 <b id="o1">0</b> 回 ／ 読む行 <b id="r1">0</b></div><div id="left"></div></div>
 <div class="col"><h4>Semantic Agent Schema で見る</h4><div class="sub">変更の前後から導いた意味の差分（保存しない）</div><div class="ops">人の判断 <b id="o2">0</b> 回 ／ 読む行 <b id="r2">0</b></div><div id="right"></div></div>
</div>
<div class="why" id="why"></div>
<script>
const $=s=>document.querySelector(s);
const GIT=()=>'<div class="f mono">articles/記事01.json  <span class="add">+3</span> <span class="del">-3</span>\narticles/記事02.json  <span class="add">+5</span> <span class="del">-4</span>\narticles/記事03.json  <span class="add">+4</span> <span class="del">-4</span>\n…（記事04〜20 も同じように並ぶ）\n.changes/9b1e7c40.json  <span class="add">+9</span>\n…（変更の記録 12件）\nviews/article.json  <span class="add">+2</span> <span class="del">-1</span></div>';
const DIFF=()=>'<div class="f mono"><span class="del">-  "body": "来月から価格を5%上げる。…",</span>\n<span class="add">+  "body": "来月から価格を8%上げる。…",</span>\n<span class="del">-  "summary": "価格を5%上げる",</span>\n<span class="add">+  "summary": "価格を8%上げる",</span>\n<span class="del">-  "word_count": 412,</span>\n<span class="add">+  "word_count": 413,</span></div>';
const EX='<div class="sec">例外 1件</div><div class="ex">記録に無い：記事07「新製品の発表」/タイトル（基盤を通らない書き換え）</div>';
const PURPOSE='<div class="sec">目的</div><div class="f">ACDR-0201 価格改定を記事に反映する</div>';
const ROWS=[
 ['価格改定を反映する（5% → 8%）','記事2 ほか 9件',18],
 ['本文の改定に要約を合わせる','記事2 ほか 9件',10],
 ['改定の告知を足す','記事3',2],
 ['…（ほか 9件）','',50]];
const rows=(open)=>'<div class="sec">理由ごとの変更（12件）</div>'+ROWS.map((r,i)=>'<div class="row'+(open&&i===0?' open':'')+'"><div class="h"><span>'+r[0]+'</span><span class="c">'+r[1]+(r[2]?' ・ '+r[2]+'か所':'')+'</span></div>'+(open&&i===0?'<div class="body"><div class="kv"><span class="hd">欄</span><span class="hd">前</span><span class="hd">後</span><span>記事2「価格改定のお知らせ」/本文</span><span>…5%上げる。</span><span>…8%上げる。</span></div><div class="under">↳ 一緒に変わった欄：/文字数 412 → 413（本文から導く）</div><div class="kv" style="margin-top:4px"><span>記事2 /公開日 <span class="tag hu">人が決める欄</span></span><span>10月1日</span><span>10月1日（変わっていない）</span></div></div>':'')+'</div>').join('');
const BEH='<div class="sec">検証結果の差分</div><div class="st">記事2「価格改定のお知らせ」<span class="b-ok">検証済み</span> → <span class="b-ok">検証済み</span></div><div class="st">記事5「採用情報」<span class="b-ok">検証済み</span> → <span class="b-wait">判断待ち</span><span style="font-size:12px;color:var(--sub)">「要約は本文と矛盾しない」を AI が判定できなかった</span></div><div class="none">ほか 18件は変わらず検証済み（件数だけ出す）</div>';
const RENDER='<div class="sec">描画の差分（前後とも、いまのビューで描く）</div><div class="pair"><div class="card"><div class="cap">変更前</div><h5>価格改定のお知らせ</h5>来月から価格を<span class="chg">5%</span>上げる。対象は全商品。</div><div class="card"><div class="cap">変更後</div><h5>価格改定のお知らせ</h5>来月から価格を<span class="chg">8%</span>上げる。対象は全商品。</div></div>';
const STEPS=[
 {t:'AI が変更を出す',d:'AI の Agent が、目的「ACDR-0201 価格改定」で記事を20件直した。子のセッションも使い、変更の記録は12件。ほかに、誰かが記事07のタイトルを手で書き換えた。',
  L:()=>GIT(),R:()=>'<div class="none">変更の記録 12件が、記事と一緒にコミットされている。まだ何も見せていない。</div>',o:[0,0],r:[0,0],
  why:'変更の量はどちらも同じ。違うのは、人に何を見せるか。'},
 {t:'PR を開く',d:'人が、この変更を取り込むかを決めるために PR を開く。',
  L:()=>GIT(),R:()=>EX+PURPOSE+rows(false),o:[0,0],r:[45,16],
  why:'左はファイルの一覧から始まる。右は、例外 → 目的 → 理由の順に並ぶ。理由は12行で、変わった欄は理由の下にたたまれている。'},
 {t:'理由の1行を開く（内容の差分）',d:'「価格改定を反映する」が本当に 5% → 8% かを確かめたい。',
  L:()=>GIT()+DIFF(),R:()=>EX+PURPOSE+rows(true),o:[0,0],r:[160,22],
  why:'左は、記事ごとのファイルを開いて JSON の行を読む。右は、理由の行を開くと、対象の名前つきで欄の前後が出る。文字数のように導出で一緒に変わった欄は、原因の欄の下に入る。人が決める欄（公開日）は印つきで、変わっていないことも見える。'},
 {t:'振る舞いを確かめる（検証結果の差分）',d:'この変更で、決まりを満たさなくなった記事は無いか。',
  L:()=>GIT()+'<div class="none">差分には出ない。CI のログを別に開いて探す。</div>',R:()=>BEH,o:[0,0],r:[160,25],
  why:'右は、状態が変わった記事だけを出す。記事5が判断待ちになったので、人が見るべきものが1件あると分かる。変わらなかった18件は件数だけ。'},
 {t:'見た目を確かめる（描画の差分）',d:'読者に見える記事の姿は、どう変わるか。',
  L:()=>GIT()+'<div class="none">差分には出ない。手元で描画して見比べる。</div>',R:()=>RENDER,o:[0,0],r:[160,27],
  why:'右は、状態か人が決める欄が変わった記事だけ、変更の前後を同じビューで描いて並べる。ビューが同じなので、違いは内容の違いだけ。'},
 {t:'例外と判断待ちに答える',d:'記事07の手の書き換えと、記事5の判断待ちに答える。',
  L:()=>GIT()+'<div class="none">手の書き換えにも、判断待ちにも気づく手がかりが無い。全ファイルを1つずつ確かめる。</div>',
  R:()=>'<div class="ex">記録に無い：記事07 /タイトル → 書いた人が変更の記録を足したので、例外が消えた</div><div class="st">記事5「採用情報」<span class="b-wait">判断待ち</span> → 人が「合っている」と判定 → <span class="b-ok">検証済み</span></div><div class="none">人の判定は判定の記録に残る。同じ値なら、次は聞き直さない。</div>',
  o:[80,2],r:[160,28],
  why:'左は、20ファイル ・ 80か所を1つずつ「これでよいか」と見ることになる（約80回）。右で人が判断したのは、例外1件と判断待ち1件の2回。機械の例外は、記録を直せば消える。'},
 {t:'取り込む',d:'人が PR を取り込む。',
  L:()=>'<div class="none">PR を取り込む（1回）。何を承認したかは、ファイルの差分しか残らない。</div>',
  R:()=>'<div class="none">PR を取り込む（1回）。承認は Git が PR に残す。Core は承認の記録を持たない。同じ変更の前後からは同じ意味の差分が出るので、何を承認したかを後から作り直せる。</div>',
  o:[81,3],r:[160,28],
  why:'左：判断 約81回、読む行 160行以上。右：判断 3回、読む行 約28行。仮定の数で、利用者で測ってはいない。'},
];
let step=-1;
function render(){$('#dots').innerHTML=STEPS.map((_,i)=>'<span class="'+(i<=step?'on':'')+'"></span>').join('');
 $('#prev').disabled=step<0;$('#next').disabled=step>=STEPS.length-1;
 if(step<0){$('#no').textContent='はじめに';$('#title').textContent='価格改定の変更を取り込むか';$('#detail').textContent='記事の Skill に、AI が大きな変更を出した。人はそれを取り込むかを決める。';
  $('#left').innerHTML=$('#right').innerHTML='<div class="none">まだ何も起きていない</div>';['o1','o2','r1','r2'].forEach(i=>$('#'+i).textContent=0);$('#why').textContent='左右の「人の判断」と「読む行」の数を比べる。';return;}
 const s=STEPS[step];$('#no').textContent='場面 '+(step+1)+' / '+STEPS.length;$('#title').textContent=s.t;$('#detail').textContent=s.d;
 $('#left').innerHTML=s.L();$('#right').innerHTML=s.R();$('#o1').textContent=s.o[0];$('#o2').textContent=s.o[1];$('#r1').textContent=s.r[0];$('#r2').textContent=s.r[1];$('#why').textContent=s.why;}
$('#next').onclick=()=>{step++;render();};$('#prev').onclick=()=>{step--;render();};$('#again').onclick=()=>{step=-1;render();};
render();
</script></body></html>'''
out = HTML.replace('%LIGHT%', light).replace('%DARK%', dark).replace('%FONT%', font)
open('q10-review-sim.html', 'w').write(out)
print(len(out))
