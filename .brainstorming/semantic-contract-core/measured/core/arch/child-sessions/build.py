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
<p class="meta">セッションの構造 ・ 子の4つの型 ・ 型ごとの手段（AI ツールの中と、外からのコマンド）・ 用途を1枚にまとめた。数字は 2026-10-25 の実測（約4万トークンの親から各1回）。「原典」は公式の文書、「推測」はクレジットなどからの見立て</p>
<p class="ans">目的のセッションが要件 ・ 設計 ・ 実装を通して受け持ち、横の仕事だけを子にする。子は「親の文脈を持つか」と「続くか」で4つの型に分かれる。契約は型を持ち、AI ツールごとに手段とキャッシュが効くかを持つ。できない ・ 効かないツールでは、代わりの型か、いまのセッションで行う</p>
<h2>1　セッションの構造</h2>
<div class="tree" style="font:12.5px/1.7 var(--mono);background:var(--code);border-radius:8px;padding:10px 12px;white-space:pre;overflow-x:auto">オーケストレーター（リポジトリに1つ。CLAUDE.md などの作業の進め方。目的 ・ 親 ・ 役割を持たない）
└─ 目的のセッション（目的1つに1つ。要件整理 → 設計 → 実装 を全部この中で）
    ├─ 分岐        親の文脈を持つ ・ 1回で終わる ・ 残さない
    ├─ 単発        親の文脈を持たない ・ 1回で終わる ・ 残さない
    ├─ 継続する子 A  親の文脈を持たない ・ 続く ・ 役割ごとに1つ
    └─ 継続する子 B  親の文脈を持つ ・ 続く ・ 役割ごとに1つ</div>
<p class="meta">工程（要件 ・ 設計 ・ 実装）でセッションを分けない。分けると目的の文脈が散らばり発散する。子にするのは横の仕事だけ</p>
<h2>2　4つの型と寿命</h2>
{table(['型', '親の文脈', '続くか', '寿命', 'セッションの集約'], [
 ['分岐', '持って始まる', '1回で終わる', '1回の呼び出し', '登録しない'],
 ['単発', '持たない（指示だけ）', '1回で終わる', '1回の呼び出し', '登録しない（判定させる AI と同じ）'],
 ['継続する子 A（既定）', '持たない（指示だけ）', '続く', '役割の仕事が終わるか、目的が閉じるまで', '登録する（目的 ・ 親 ・ 役割の組で1つ）'],
 ['継続する子 B', '持って始まる', '続く', '同上', '登録する'],
])}
<h2>3　用途と型</h2>
{table(['用途', '型', 'なぜその型か'], [
 ['要件整理 ・ 設計 ・ 実装（目的そのもの）', '目的のセッションの中で行う', '子にすると目的の文脈が散らばる'],
 ['反証（答えを出す前の穴探し）', '分岐', '会話の意図を必要とする。結果は reviews/ に残るので、セッションは残さない'],
 ['Semantic Search の整形', '分岐、またはいまのセッション', '問いの文脈を必要とする。1回で終わる'],
 ['原文の調査 ・ 下書き ・ 試作', '単発', '会話の文脈を必要としない。メインの文脈を汚さない'],
 ['判定させる AI（AI の rule）', '単発（ツールを持たせない）', '判定ごとに新しく呼ぶ（決着済み）。前の判定の会話を結果に混ぜない'],
 ['指示で進められる長い仕事（決まった設計どおりに作る）', '継続する子 A', '文脈がきれい。再開でキャッシュが効く'],
 ['会話の意図を知らないと外れる長い仕事', '継続する子 B', '意図を取りこぼさない。作るときの費用はツールによる'],
])}
<h2>4　型ごとの手段（AI ツールの中から ／ 外からのコマンド）</h2>
{table(['型', 'Claude Code 2.1.305', 'Codex 0.164.0', 'Copilot 1.0.103', 'Kiro 2.21.1'], [
 ['分岐：AI の中から', Y('fork のサブエージェント（原典）'), Y('spawn_agent fork_turns all：94%（実測）'), X('無い。サブエージェントは親の会話を引き継がない（原典）'), X('無い。sub-agents は文脈を分ける（原典）')],
 ['分岐：人が対話で', '/fork（別のセッション）', '/side（残さない）・ /fork', '/fork ・ /ask（ツールを使えない）', '/tangent ・ /rewind'],
 ['分岐：外からコマンドで', Y('--resume P --fork-session --no-session-persistence：96%（実測）'), Y('exec fork --ephemeral P：64%（実測。親の再開と同じ）'), X('無い（/fork は対話だけ）'), N('ACP の session/fork：クレジットが親と同じ（推測）')],
 ['単発', '通常のサブエージェント ／ claude -p', 'spawn_agent（履歴なし）／ codex exec --ephemeral', 'task のサブエージェント ／ copilot -p', 'sub-agents ／ kiro-cli chat --no-interactive'],
 ['継続 A：作る', '通常のサブエージェント ／ claude -p', 'spawn_agent（履歴なし）／ codex exec', 'task のサブエージェント ／ copilot -p', 'sub-agents ／ kiro-cli chat --no-interactive'],
 ['継続 A ・ B：再開', Y('--resume I：100%（実測）。サブエージェントの再開も効く（原典）'), Y('exec resume I：64%（実測）'), Y('--resume I：96%（実測）'), Y('--resume-id I：クレジットが約半分（推測）')],
 ['継続 B：作る', Y('--resume P --fork-session：87%（実測）'), Y('AI の中：spawn_agent fork_turns all（94%。その子に send_input で続ける）') + '<br>' + N('外から：exec fork P：29%（実測）'), X('外からは作れない（/fork は対話だけ）'), N('ACP の session/fork：効かない（推測）')],
])}
<p class="meta">Copilot Free はモデルが自動選択で、モデルが変わるとキャッシュは効かない（原典「A different model can't reuse another model's cache」）。Codex のサブエージェントは親と同じキャッシュキーを使う（ソースの試験）</p>
<h2>5　できない ・ 効かないときの代わり</h2>
{table(['型', 'ツール', '代わり'], [
 ['分岐', 'Copilot ・ Kiro', 'いまのセッションの中で行う（再開と同じくキャッシュが効く）。やりとりは親の会話に残る'],
 ['継続 B', 'Copilot', '継続 A にし、親が会話の意図を指示に書いて渡す'],
 ['継続 B', 'Codex（外から）・ Kiro', '作るときに親の文脈の分を1回払う。重ければ A にして意図を指示に書く'],
])}
{SIM}
<h2>7　契約に持つもの</h2>
{table(['持つもの', '中身'], [
 ['セッションの種類', 'オーケストレーター ・ 目的 ・ 任せる先。任せる先は 分岐 ・ 単発 ・ 継続 A ・ 継続 B の型を持つ。分岐と単発は集約に登録しない'],
 ['AI ツールのアダプターが返す能力', '型ごとに「外からのコマンドで作れるか」と「親のキャッシュが効くか ・ 再開で効くか（効く ・ 効かない ・ 不明）」'],
 ['選び方', '使う場面が型を求め、ツールができない ・ 効かないときは5節の代わりにする'],
 ['基盤が関わる範囲', '基盤は AI を起動しない。外からのコマンドの手段はアダプターの能力として持つだけで、AI の中の手段（サブエージェント ・ 対話のコマンド）は呼ぶ側の AI ツールの機能'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
