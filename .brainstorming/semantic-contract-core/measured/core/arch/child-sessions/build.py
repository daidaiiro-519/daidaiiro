"""子のセッションの3つの型と、AI ツールごとの能力（キャッシュが効くか）の頁。実測は 2026-10-25（各1回）。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.y{color:var(--add-ink);background:var(--add);font-weight:700;padding:0 4px;border-radius:4px}.n{color:var(--bad);background:var(--bad-bg);font-weight:700;padding:0 4px;border-radius:4px}.u{color:var(--hot);background:var(--hot-bg);font-weight:700;padding:0 4px;border-radius:4px}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>子のセッションの型</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
Y = lambda t: f'<span class="y">効く</span> {t}'
N = lambda t: f'<span class="n">効かない</span> {t}'
U = lambda t: f'<span class="u">不明</span> {t}'
SIM = r'''
<h2>6　シミュレーション：同じ契約を4つのツールで動かす</h2>
<p class="meta">目的のセッション「ACDR 0199 依存の向きの検査ツール」が、約4万トークンの会話を積んだところから、3つの仕事を依頼する。呼び出し（契約）はどのツールでも同じで、ツールが返す能力で実際の動きが変わる。上のボタンでツールを切り替える</p>
<div class="sim">
<div class="tabs" role="tablist">
<button data-t="claude" class="on">Claude Code</button><button data-t="codex">Codex</button><button data-t="copilot">Copilot</button><button data-t="kiro">Kiro</button>
</div>
<div id="simbody"></div>
</div>
<style>
.sim{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:12px;display:flex;flex-direction:column;gap:10px}
.sim .tabs{display:flex;gap:6px;flex-wrap:wrap}
.sim .tabs button{font:inherit;font-size:13px;padding:4px 12px;border-radius:999px;border:1px solid var(--line);background:var(--bg);color:var(--ink);cursor:pointer}
.sim .tabs button.on{background:var(--accent);color:var(--card);border-color:var(--accent)}
.sim .step{border:1px solid var(--line);border-radius:10px;padding:8px 10px;display:grid;grid-template-columns:9em 1fr;gap:4px 10px;font-size:13.5px}
@media (max-width:640px){.sim .step{grid-template-columns:1fr}}
.sim .step b{color:var(--accent);grid-column:1/-1}
.sim .k{color:var(--sub);font-size:12.5px}
.sim pre{margin:0;font:12px/1.5 var(--mono);background:var(--code);border-radius:6px;padding:6px 8px;overflow-x:auto;white-space:pre}
.sim .tree{font:12.5px/1.6 var(--mono);background:var(--code);border-radius:8px;padding:8px 10px;white-space:pre;overflow-x:auto}
</style>
<script>
(function(){
const CALL={
 fork:'{"type":"分岐","task":"設計の答えを反証する","returns":"指摘の一覧"}',
 a:'{"type":"継続A","role":"実装役","task":"core.rs を規則どおり作る","handoff":"仕事の指示だけ"}',
 b:'{"type":"継続B","role":"設計を知る実装役","task":"設計の議論を踏まえて cargo.rs を作る"}'};
const T={
 claude:{name:'Claude Code',cap:{fork:'作れる ・ 親のキャッシュ 効く',a:'作れる ・ 再開で効く',b:'作れる ・ 親のキャッシュ 効く'},
  fork:['分岐のまま','claude -p "<反証の指示>" --resume P --fork-session --no-session-persistence','約4万のうち 96% をキャッシュから読む（実測）','残らない'],
  a:['継続Aのまま','1回目：claude -p "<指示>"（新しいセッション I）／ 2回目から：claude -p "<続き>" --resume I','1回目は指示の分だけ。2回目から自分の会話を 100% 読む（実測）','I が残る（目的が閉じるまで）'],
  b:['継続Bのまま','1回目：claude -p "<指示>" --resume P --fork-session（新しいセッション J）／ 2回目から：--resume J','1回目も親の約4万のうち 87% をキャッシュから読む（実測）','J が残る'],
  tree:'目的のセッション P（約4万）\n├─ 分岐：反証（残らない）\n├─ 継続A：実装役 I\n└─ 継続B：設計を知る実装役 J'},
 codex:{name:'Codex',cap:{fork:'作れる ・ 親のキャッシュ 効く（--ephemeral）',a:'作れる ・ 再開で効く',b:'作れる ・ 親のキャッシュ 効かない'},
  fork:['分岐のまま','codex exec fork --ephemeral P "<反証の指示>"','親を再開したときと同じ 64% をキャッシュから読む（実測）','残らない'],
  a:['継続Aのまま','1回目：codex exec "<指示>"（I）／ 2回目から：codex exec resume I "<続き>"','2回目から 64% を読む（実測）','I が残る'],
  b:['継続Bのまま（費用が上がる）','1回目：codex exec fork P "<指示>"（J）／ 2回目から：codex exec resume J','1回目は親の約4万をほぼ全部払う（29%。実測）。2回目からは効く','J が残る'],
  tree:'目的のセッション P（約4万）\n├─ 分岐：反証（残らない）\n├─ 継続A：実装役 I\n└─ 継続B：設計を知る実装役 J（作るときに約4万を1回払った）'},
 copilot:{name:'Copilot',cap:{fork:'コマンドで作れない',a:'作れる ・ 再開で効く',b:'コマンドで作れない'},
  fork:['代わりに「いまのセッションで行う」','P の会話の中で、反証の指示を1つ続けて送る（copilot -p "<反証の指示>" --resume P）','P の会話をそのまま読むので、再開と同じく 96% 効く（実測）。ただし反証のやりとりが P の会話に残る','新しいセッションは増えない'],
  a:['継続Aのまま','1回目：copilot -p "<指示>"（I）／ 2回目から：copilot -p "<続き>" --resume I','2回目から 96% を読む（実測）。Free は自動選択でモデルが変わると効かない','I が残る'],
  b:['代わりに「継続A」','親が設計の意図を指示に書いて渡す：copilot -p "<意図と指示>"（I）','1回目は指示の分だけ。意図は親が選んで書く','I が残る'],
  tree:'目的のセッション P（約4万。反証のやりとりもここに入る）\n├─ 継続A：実装役 I\n└─ 継続A：設計を知る実装役 I2（意図を指示に書いて渡した）'},
 kiro:{name:'Kiro',cap:{fork:'作れる（ACP）・ 親のキャッシュ 効かない',a:'作れる ・ 再開で効く',b:'作れる（ACP）・ 親のキャッシュ 効かない'},
  fork:['代わりに「いまのセッションで行う」','kiro-cli chat --no-interactive --resume-id P "<反証の指示>"','分岐すると親の約4万を毎回払うので、再開で行う。再開は効く（クレジットが約半分）','新しいセッションは増えない'],
  a:['継続Aのまま','1回目：kiro-cli chat --no-interactive "<指示>"（I）／ 2回目から：--resume-id I','2回目から効く（クレジットからの推測）','I が残る'],
  b:['継続Bのまま（費用が上がる）','1回目：ACP の session/fork で P から J を作る ／ 2回目から：--resume-id J','1回目は親の約4万をほぼ全部払う（推測）。2回目からは効く','J が残る'],
  tree:'目的のセッション P（約4万。反証のやりとりもここに入る）\n├─ 継続A：実装役 I\n└─ 継続B：設計を知る実装役 J（作るときに約4万を1回払った）'}
};
function esc(s){return s.replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]))}
function render(k){
 const t=T[k];let h='';
 [['fork','① 反証を依頼する'],['a','② 実装役に任せる'],['b','③ 設計を知る実装役に任せる']].forEach(([key,title])=>{
  const r=t[key];
  h+='<div class="step"><b>'+title+'</b>'
   +'<span class="k">呼び出し（契約）</span><pre>'+esc(CALL[key])+'</pre>'
   +'<span class="k">ツールが返す能力</span><span>'+esc(t.cap[key])+'</span>'
   +'<span class="k">選ばれる型</span><span>'+esc(r[0])+'</span>'
   +'<span class="k">実際のコマンド</span><pre>'+esc(r[1])+'</pre>'
   +'<span class="k">費用</span><span>'+esc(r[2])+'</span>'
   +'<span class="k">セッション</span><span>'+esc(r[3])+'</span></div>';
 });
 h+='<div><span class="k">終わったときのセッションの木（'+esc(t.name)+'）</span><div class="tree">'+esc(t.tree)+'</div></div>';
 h+='<p class="k">どのツールでも、①の指摘の一覧 ・ ②③の成果物は同じ契約の形で返る。違うのは型の選ばれ方と費用だけ</p>';
 document.getElementById('simbody').innerHTML=h;
 document.querySelectorAll('.sim .tabs button').forEach(b=>b.classList.toggle('on',b.dataset.t===k));
}
document.querySelectorAll('.sim .tabs button').forEach(b=>b.addEventListener('click',()=>render(b.dataset.t)));
render('claude');
})();
</script>
'''
body = f'''<main>
<h1>子のセッションの型</h1>
<p class="meta">利用者「forkでいいものやsessionとして別の文脈としてその目的と意図の間は継続している必要がある子セッションの区別も契約として持っていた方がいい」「ClaudeCodeとCodexとCopilotとKiroで抽象できる契約として整理できること、キャッシュを利かすことができること」「A,Bも両方あるイメージ」を受けた整理。実測は 2026-10-25、約4万トークンの資料を読ませた親から各1回</p>
<p class="ans">子は3つの型を持つ。継続する子 A は4つの AI ツールすべてで再開のキャッシュが効く。分岐でキャッシュが効くのは Claude Code と Codex。継続する子 B を作るときに親のキャッシュが効くのは Claude Code だけ。契約は型ごとに AI ツールの能力を持ち、できない ・ 効かないツールでは代わりの型にする</p>
<h2>1　3つの型</h2>
{table(['型', '親の文脈', '続くか', '向いている仕事', '寿命'], [
 ['分岐', '持って始まる', '1回で終わる。残さない', '反証、Semantic Search の整形', '1回の呼び出し'],
 ['継続する子 A（既定）', '持たずに始まる（仕事の指示だけ）', '続く。役割ごとに再開する', '指示だけで進められる長い仕事', '役割の仕事が終わるか、目的が閉じるまで'],
 ['継続する子 B', '持って始まる', '続く。役割ごとに再開する', '会話の意図を知らないと外れる長い仕事', '同上'],
])}
<h2>2　AI ツールごとの能力（キャッシュが効くか）</h2>
{table(['型', 'Claude Code 2.1.305', 'Codex 0.164.0', 'Copilot 1.0.103', 'Kiro 2.21.1'], [
 ['分岐（親の文脈を引き継ぐ）', Y('96%（実測）<br>--resume ID --fork-session --no-session-persistence'), Y('64%（実測。親の再開と同じ）<br>exec fork --ephemeral ID'), U('対話の中の /fork だけ。コマンドでは作れない'), N('クレジットが親と同じ（推測）<br>v3 の ACP の session/fork')],
 ['継続する子 A：作る', '新しいセッション（親のキャッシュは無い）', '同左', '同左', '同左'],
 ['継続する子 A ・ B：2回目からの再開', Y('100%（実測）<br>--resume ID'), Y('64%（実測）<br>exec resume ID'), Y('96%（実測）<br>--resume ID'), Y('クレジットが約半分（推測）<br>--resume-id ID')],
 ['継続する子 B：作る（親の文脈を持たせる）', Y('87%（実測）<br>--resume ID --fork-session'), N('29%（実測。共通の前置きの分だけ）<br>exec fork ID'), U('対話の中の /fork だけ'), N('分岐と同じ（推測）')],
])}
<p class="meta">Copilot Free はモデルが自動選択（auto）で、呼ぶたびにモデルが変わることがある。キャッシュはモデルをまたいで使えない（Copilot の文書「A different model can't reuse another model's cache」）。今回の再開は同じモデル（gpt-6-luna）で測れた。Kiro はトークン数を返さないので、クレジットからの推測</p>
<h2>3　ツールでできない ・ 効かないときの代わり</h2>
{table(['型', '代わり'], [
 ['分岐', 'いまのセッションの中で行う（Copilot ・ Kiro）'],
 ['継続する子 B', '親の文脈の分を1回払って作る（Codex ・ Kiro）。それが重いなら A にし、親が意図を指示に書いて渡す'],
])}
<h2>4　契約に持つもの</h2>
{table(['持つもの', '中身'], [
 ['セッションの種類', 'いまの3つ（オーケストレーター ・ 目的 ・ 任せる先）に、任せる先の型として 分岐 ・ 継続 A ・ 継続 B を足す。分岐は残さないので、セッションの集約に登録しない（判定させる AI と同じ）'],
 ['AI ツールのアダプターが返す能力', '型ごとに「コマンドで作れるか（できる ・ できない）」と「親のキャッシュが効くか ・ 再開でキャッシュが効くか（効く ・ 効かない ・ 不明）」'],
 ['選び方', '使う場面が型を求め、ツールができない ・ 効かないときは3節の代わりにする'],
])}

{SIM}
<h2>7　確かめてほしいこと</h2>
{table(['確かめてほしいこと', '私の見解'], [
 ['子の3つの型（分岐 ・ 継続 A ・ 継続 B）を契約に持つ', '持つ。既定は A'],
 ['AI ツールのアダプターが、型ごとに「作れるか」と「キャッシュが効くか」を返す', 'そうする'],
 ['Copilot の分岐 ・ B は、対話の中の /fork だけなので「コマンドでは作れない」とする', 'そうする。必要なら対話で /fork して測る'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
