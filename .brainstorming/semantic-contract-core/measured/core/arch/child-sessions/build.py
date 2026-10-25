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
X = lambda t: f'<span class="n">できない</span> {t}'
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
 fork:'{"lifetime":"単発","context":"引き継ぐ","task":"設計の答えを反証する","returns":"指摘の一覧"}',
 a:'{"lifetime":"継続","context":"引き継がない","role":"実装役","task":"core.rs を規則どおり作る","handoff":"仕事の指示だけ"}',
 b:'{"lifetime":"継続","context":"引き継ぐ","role":"設計を知る実装役","task":"設計の議論を踏まえて cargo.rs を作る"}'};
const T={
 claude:{name:'Claude Code',cap:{fork:'作れる ・ 親のキャッシュ 効く',a:'作れる ・ 再開で効く',b:'作れる ・ 親のキャッシュ 効く'},
  fork:['単発 ・ 引き継ぐ のまま','claude -p "<反証の指示>" --resume P --fork-session --no-session-persistence','約4万のうち 96% をキャッシュから読む（実測）','残らない'],
  a:['継続 ・ 引き継がない のまま','1回目：claude -p "<指示>"（新しいセッション I）／ 2回目から：claude -p "<続き>" --resume I','1回目は指示の分だけ。2回目から自分の会話を 100% 読む（実測）','I が残る（目的が閉じるまで）'],
  b:['継続 ・ 引き継ぐ のまま','1回目：claude -p "<指示>" --resume P --fork-session（新しいセッション J）／ 2回目から：--resume J','1回目も親の約4万のうち 87% をキャッシュから読む（実測）','J が残る'],
  tree:'目的のセッション P（約4万）\n├─ 単発 ・ 引き継ぐ：反証（残らない）\n├─ 継続 ・ 引き継がない：実装役 I\n└─ 継続 ・ 引き継ぐ：設計を知る実装役 J'},
 codex:{name:'Codex',cap:{fork:'作れる ・ 親のキャッシュ 効く（--ephemeral ・ spawn_agent）',a:'作れる ・ 再開で効く',b:'作れる ・ 親のキャッシュ 効かない'},
  fork:['単発 ・ 引き継ぐ のまま','codex exec fork --ephemeral P "<反証の指示>"','親を再開したときと同じ 64% をキャッシュから読む（実測）','残らない'],
  a:['継続 ・ 引き継がない のまま','1回目：codex exec "<指示>"（I）／ 2回目から：codex exec resume I "<続き>"','2回目から 64% を読む（実測）','I が残る'],
  b:['継続 ・ 引き継ぐ のまま（費用が上がる）','1回目：codex exec fork P "<指示>"（J）／ 2回目から：codex exec resume J','1回目は親の約4万をほぼ全部払う（29%。実測）。2回目からは効く','J が残る'],
  tree:'目的のセッション P（約4万）\n├─ 単発 ・ 引き継ぐ：反証（残らない）\n├─ 継続 ・ 引き継がない：実装役 I\n└─ 継続 ・ 引き継ぐ：設計を知る実装役 J（作るときに約4万を1回払った）'},
 copilot:{name:'Copilot',cap:{fork:'コマンドで作れない',a:'作れる ・ 再開で効く',b:'コマンドで作れない'},
  fork:['代わりに「いまのセッションで行う」','P の会話の中で、反証の指示を1つ続けて送る（copilot -p "<反証の指示>" --resume P）','P の会話をそのまま読むので、再開と同じく 96% 効く（実測）。ただし反証のやりとりが P の会話に残る','新しいセッションは増えない'],
  a:['継続 ・ 引き継がない のまま','1回目：copilot -p "<指示>"（I）／ 2回目から：copilot -p "<続き>" --resume I','2回目から 96% を読む（実測）。Free は自動選択でモデルが変わると効かない','I が残る'],
  b:['代わりに「継続 ・ 引き継がない」','親が設計の意図を指示に書いて渡す：copilot -p "<意図と指示>"（I）','1回目は指示の分だけ。意図は親が選んで書く','I が残る'],
  tree:'目的のセッション P（約4万。反証のやりとりもここに入る）\n├─ 継続 ・ 引き継がない：実装役 I\n└─ 継続 ・ 引き継がない：設計を知る実装役 I2（意図を指示に書いて渡した）'},
 kiro:{name:'Kiro',cap:{fork:'作れる（ACP）・ 親のキャッシュ 効かない',a:'作れる ・ 再開で効く',b:'作れる（ACP）・ 親のキャッシュ 効かない'},
  fork:['代わりに「いまのセッションで行う」','kiro-cli chat --no-interactive --resume-id P "<反証の指示>"','session/fork で作ると親の約4万を毎回払うので、再開で行う。再開は効く（クレジットが約半分）','新しいセッションは増えない'],
  a:['継続 ・ 引き継がない のまま','1回目：kiro-cli chat --no-interactive "<指示>"（I）／ 2回目から：--resume-id I','2回目から効く（クレジットからの推測）','I が残る'],
  b:['継続 ・ 引き継ぐ のまま（費用が上がる）','1回目：ACP の session/fork で P から J を作る ／ 2回目から：--resume-id J','1回目は親の約4万をほぼ全部払う（推測）。2回目からは効く','J が残る'],
  tree:'目的のセッション P（約4万。反証のやりとりもここに入る）\n├─ 継続 ・ 引き継がない：実装役 I\n└─ 継続 ・ 引き継ぐ：設計を知る実装役 J（作るときに約4万を1回払った）'}
};
function esc(s){return s.replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]))}
function render(k){
 const t=T[k];let h='';
 [['fork','① 反証を依頼する（単発 ・ 引き継ぐ）'],['a','② 実装役に任せる（継続 ・ 引き継がない）'],['b','③ 設計を知る実装役に任せる（継続 ・ 引き継ぐ）']].forEach(([key,title])=>{
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
<p class="meta">利用者と整理した観点：起点は目的、その中で作業が出てくる。作業ごとに「寿命（単発か継続か）」と「親の文脈（引き継ぐか引き継がないか）」を決める。数字は 2026-10-25 の実測（約4万トークンの親から各1回）</p>
<p class="ans">作業は「単発か継続か」と「親の文脈を引き継ぐか」の2つの軸で4つの型になる。単発はサブエージェントで済ませる（引き継ぐなら fork）。継続は別のセッションを役割ごとに再開する。既定は引き継がない側で、会話の意図を必要とするときだけ引き継ぐ</p>
<h2>1　起点は目的、その中で作業</h2>
<div style="font:12.5px/1.7 var(--mono);background:var(--code);border-radius:8px;padding:10px 12px;white-space:pre;overflow-x:auto">目的のセッション（目的1つに1つ。要件整理 → 設計 → 実装 はこの中で進める）
 │
 ├─ 作業：反証            → 単発 ・ 引き継ぐ
 ├─ 作業：原文の調査      → 単発 ・ 引き継がない
 ├─ 作業：AI の rule の判定 → 単発 ・ 引き継がない（会話を見せない）
 ├─ 作業：決まった設計どおりの実装 → 継続 ・ 引き継がない
 └─ 作業：議論の意図を踏まえた実装 → 継続 ・ 引き継ぐ</div>
<h2>2　2つの軸と4つの型</h2>
{table(['', '親の文脈を引き継ぐ', '親の文脈を引き継がない'], [
 ['<b>単発</b>（1回で終わる。残さない）', '反証、Semantic Search の整形<br><span class="meta">手段：fork のサブエージェント</span>', '原文の調査、AI の rule の判定<br><span class="meta">手段：通常のサブエージェント</span>'],
 ['<b>継続</b>（目的を達成するまで続ける）', '議論の意図を踏まえた長い作業<br><span class="meta">手段：親から分けたセッションを、役割ごとに再開する</span>', '指示で進められる長い作業（既定）<br><span class="meta">手段：新しいセッションを、役割ごとに再開する</span>'],
])}
<h2>3　どちらに決めるか</h2>
{table(['軸', '決め方'], [
 ['単発か継続か', '1回の答えで終わるなら単発。何度もやりとりして積み上げるなら継続'],
 ['単発で引き継ぐか', '良し悪しが、この会話の意図で決まるなら引き継ぐ（反証は「利用者の言葉から外れていないか」を見る）'],
 ['単発で引き継がないか', '会話の外の事実だけで決まるとき（原文の調査）。会話を見せてはいけないとき（AI の rule の判定は、値と x-prompt だけで判定すると決着済み）'],
 ['継続で引き継ぐか', '意図を指示に書き切れず、会話そのものを持たせないと外れるときだけ。続けるあいだ親の会話（横道も含む）を背負うので、文脈が二重になり発散しやすい'],
 ['継続で引き継がないか', '意図を指示に書き切れるとき。既定'],
])}
<h2>4　型ごとの手段（AI ツールの中から ／ 外からのコマンド）</h2>
{table(['型', 'Claude Code 2.1.305', 'Codex 0.164.0', 'Copilot 1.0.103', 'Kiro 2.21.1'], [
 ['単発 ・ 引き継ぐ：AI の中から', Y('fork のサブエージェント（原典）'), Y('spawn_agent fork_turns all：94%（実測）'), X('無い。サブエージェントは親の会話を引き継がない（原典）'), X('無い。sub-agents は文脈を分ける（原典）')],
 ['単発 ・ 引き継ぐ：外からコマンドで', Y('--resume P --fork-session --no-session-persistence：96%（実測）'), Y('exec fork --ephemeral P：64%（実測。親の再開と同じ）'), X('無い（/fork は対話だけ）'), N('ACP の session/fork：クレジットが親と同じ（推測）')],
 ['単発 ・ 引き継がない', '通常のサブエージェント ／ claude -p', 'spawn_agent（履歴なし）／ codex exec --ephemeral', 'task のサブエージェント ／ copilot -p', 'sub-agents ／ kiro-cli chat --no-interactive'],
 ['継続 ・ 引き継がない：作る', '通常のサブエージェント ／ claude -p', 'spawn_agent（履歴なし）／ codex exec', 'task のサブエージェント ／ copilot -p', 'sub-agents ／ kiro-cli chat --no-interactive'],
 ['継続 ・ 引き継ぐ：作る', Y('--resume P --fork-session：87%（実測）'), Y('AI の中：spawn_agent fork_turns all（94%。その子に send_input で続ける）') + '<br>' + N('外から：exec fork P：29%（実測）'), X('外からは作れない（/fork は対話だけ）'), N('ACP の session/fork：効かない（推測）')],
 ['継続（両方）：2回目からの再開', Y('--resume I：100%（実測）。サブエージェントの再開も効く（原典）'), Y('exec resume I：64%（実測）'), Y('--resume I：96%（実測）'), Y('--resume-id I：クレジットが約半分（推測）')],
 ['人が対話で使う手段（参考）', '/fork', '/side ・ /fork', '/fork ・ /ask', '/tangent ・ /rewind'],
])}
<p class="meta">Copilot Free はモデルが自動選択で、モデルが変わるとキャッシュは効かない（原典「A different model can't reuse another model's cache」）。Codex のサブエージェントは親と同じキャッシュキーを使う（ソースの試験）</p>
<h2>5　できない ・ 効かないときの代わり</h2>
{table(['型', 'ツール', '代わり'], [
 ['単発 ・ 引き継ぐ', 'Copilot ・ Kiro', 'いまのセッションの中で行う（再開と同じくキャッシュが効く）。やりとりは親の会話に残る'],
 ['継続 ・ 引き継ぐ', 'Copilot', '継続 ・ 引き継がない にし、親が会話の意図を指示に書いて渡す'],
 ['継続 ・ 引き継ぐ', 'Codex（外から）・ Kiro', '作るときに親の文脈の分を1回払う。重ければ 引き継がない にして意図を指示に書く'],
])}
{SIM}
<h2>7　契約に持つもの</h2>
{table(['持つもの', '中身'], [
 ['作業の型', '寿命（単発 ・ 継続）と親の文脈（引き継ぐ ・ 引き継がない）。単発はセッションの集約に登録しない。継続は目的 ・ 親 ・ 役割の組で1つ'],
 ['AI ツールのアダプターが返す能力', '型ごとに「外からのコマンドで作れるか」と「親のキャッシュが効くか ・ 再開で効くか（効く ・ 効かない ・ 不明）」'],
 ['選び方', '使う場面が型を求め、ツールができない ・ 効かないときは5節の代わりにする'],
 ['基盤が関わる範囲', '基盤は AI を起動しない。外からのコマンドの手段はアダプターの能力として持つだけで、AI の中の手段（サブエージェント ・ 対話のコマンド）は呼ぶ側の AI ツールの機能'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
