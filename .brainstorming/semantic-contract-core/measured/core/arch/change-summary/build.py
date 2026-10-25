"""変更の記録を手元の DB に置き、目的ごとのまとめをコミットする形を、目的のセッションの段ごとに確かめる頁。"""
import os, json
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>変更のまとめのシミュレーション</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
def card(k, n, t, f, s=''): return {"k": k, "n": n, "t": t, "f": f, "s": s}
SES = lambda s: card('セッション', 'S-01', 'session', {"目的": "0199 依存の向きの検査ツール"}, s)
CH = lambda n, f, s='手元': card('変更の記録', n, 'change', f, s)
JI = lambda n, f: card('判定（意図）', n, 'judgment', f)
JR = lambda n, f, s='未確定': card('判定（rule）', n, 'judgment', f, s)
FI = lambda n, f, s='未確定': card('ファイル', n, f.pop('形式'), f, s)
SUM = lambda f, s='未確定': card('まとめ', 'changes/01JB7….json', 'change-summary', f, s)
# 段：題 ・ 使う場面 ・ 誰が何をした ・ 事前（DB, リポジトリ）・ 事後（DB, リポジトリ）
C1 = CH('C-1', {"意図": "I-1 依存の向きを決めた記録を書く", "対象": "0199", "欄": "title ・ decision ・ why"})
C2 = CH('C-2', {"意図": "I-1", "対象": "0185", "欄": "decision"})
C3 = CH('C-3', {"意図": "I-1", "対象": "0185", "欄": "decision（C-2 を戻した）"})
C4 = CH('C-4', {"意図": "I-2 cargo metadata から依存を読む", "対象": "architecture-lint/src/cargo.rs", "欄": "（ファイル全体）"})
F0199 = lambda s='未確定': FI('0199.json', {"形式": "decision-record@1", "decision": "architecture-lint を作る"}, s)
FCARGO = lambda s='未確定': FI('cargo.rs', {"形式": "Rust", "中身": "直した版"}, s)
F0185 = FI('0185.json', {"形式": "decision-record@1", "decision": "層の並びで検査する"}, 'コミット済み')
JR1 = lambda s='未確定': JR('J-2', {"対象": "0199 × why-explains-decision", "結果": "pass", "誰": "AI"}, s)
SUMF = {"目的": "0199", "意図": "I-1 → 0199（title ・ decision ・ why） ／ I-2 → cargo.rs",
        "例外": "0185 /decision を意図に無く変えた → 戻した（差し引き0）。記録に無い書き換え ・ 書き換えの無い記録：0件", "意図の判定": "I-1 pass（AI）・ 承認（人） ／ I-2 pass（AI）・ 承認（人）", "数": "変更の記録 4件 → まとめ 2意図"}
J1 = JI('J-1', {"対象": "C-1 × 意図と変更が合うか", "結果": "pass", "誰": "AI"})
J3 = JI('J-3', {"対象": "C-2 × 意図と変更が合うか", "結果": "violated（意図に無い）", "誰": "AI"})
J4 = JI('J-4', {"対象": "I-1 ・ I-2", "結果": "承認", "誰": "人"})
def done(c): return dict(c, s='まとめに入った')
DB = [SES('open'), C1, J1, C2, J3, C3, C4, J4]
S = [
 dict(t="目的のセッションを立てる", uc="U27", who="人が目的 0199 を渡す", post=[[SES('open')], [F0185]]),
 dict(t="0199 を作って書く", uc="U5", who="AI：semantic-create → create、semantic-write → patch。基盤が変更の記録を作る", post=[[SES('open'), C1], [F0185, F0199()]]),
 dict(t="意図と変更が合うかを判定する", uc="U16 ・ I33", who="AI：semantic-judge。意図の判定は手元の DB に、rule の判定はリポジトリに置く", post=[[SES('open'), C1, J1], [F0185, F0199(), JR1()]]),
 dict(t="意図に無い変更が混ざる → 作業者が気づいて戻す", uc="U16 ・ U7", who="AI が 0185 も書き換えた。I33 の判定が violated で返り、作業者がセッションの中で戻す（0185.json は元のまま。差し引き0）", post=[[SES('open'), C1, J1, C2, J3, C3], [F0185, F0199(), JR1()]]),
 dict(t="ソースを直す", uc="U13", who="AI がファイルを直し、record で変更の記録を作る", post=[[SES('open'), C1, J1, C2, J3, C3, C4], [F0185, F0199(), JR1(), FCARGO()]]),
 dict(t="人が意図を承認する", uc="U16", who="人：judge（意図 I-1 ・ I-2）", post=[[SES('open'), C1, J1, C2, J3, C3, C4, J4], [F0185, F0199(), JR1(), FCARGO()]]),
 dict(t="まとめを作る（コミットの前）", uc="U25", who="基盤が DB の記録から機械的に作る（AI は書かない）。作業者の手元で、validate が記録と変更の前後を両方向で突き合わせ（I31）、例外をまとめに入れる", post=[DB, [F0185, F0199(), JR1(), FCARGO(), SUM(dict(SUMF))]]),
 dict(t="コミットする", uc="U25", who="人が Git でコミットする。変更の記録と意図の判定は手元に残り、「まとめに入った」の印が付く", post=[[SES('open')] + [done(c) for c in DB[1:]], [F0185, F0199('コミット'), JR1('コミット'), FCARGO('コミット'), SUM(dict(SUMF), 'コミット')]]),
]
prev = [[], []]
for st in S:
    st['pre'] = prev; prev = st['post']
READERS = [
 ['PR を見る人', 'U25 変更を確かめる', 'まとめ1件', '目的 → 意図 → 変えた所 → 例外 → 意図の判定の順に読める。1件ずつの記録（C-2 ・ C-3 の行き来）は読まなくてよい', '足りる'],
 ['半年後に経緯をたどる人', 'U13 経緯を対象ごとに確かめる', 'まとめを対象（cargo.rs）で探す', 'cargo.rs → まとめ → 意図 I-2 にたどれる。欄の単位までは分かる。1件ずつの書き換えの順は Git の履歴で見る。1件ずつの記録そのものは見えない', '足りる'],
 ['ほかの人の手元 ・ CI', 'U8 ・ U0 全部を検証する', '判定（rule）のファイル ・ まとめ', '判定（rule）がコミットされているので、0199 は verified のまま。コミットの差分とまとめを片方向で突き合わせ、まとめに無い書き換えは「記録に無い」で出る。逆向き（書き換えの無い記録）は、作業者の手元でまとめを作るときにだけ見る', '足りる（逆向きは手元だけ）'],
 ['ほかの変更を取り込む人', 'U18', 'まとめ ・ 判定（rule）', '同じ欄を両方で変えたら、Git の衝突として出る。まとめの id は打ち合わせなしで一意な値なので、2人が同じ目的でコミットしても名前が衝突しない', '足りる'],
 ['同じ目的を引き継ぐ人', 'U27 ・ U16', 'コミット済みのまとめ', 'コミットの前の意図 ・ 変更の記録 ・ 意図の承認は、引き継ぐ人には見えない。U16 の承認は作業した本人がする', '引き継ぐ前にコミットして、まとめを渡す'],
 ['作業者（セッションの中）', 'U16 ・ U7', '手元の DB の変更の記録', '意図に無い変更（C-2）が、その場で violated として出て戻せた', '足りる'],
 ['手元の DB を失った作業者', 'I40', 'コミット済みのもの', 'コミットの前なら、意図と変更の記録を失う（ファイルの変更は Git に残る）。コミットの後なら、必要なものはまとめに残っている', 'コミットの前の分は失う'],
]
PLACE = [
 ['1件ずつの変更の記録（意図 ・ 変えた欄）', 'コミットする', '手元の DB', '作業者と基盤が I33 を確かめるため'],
 ['目的ごとのまとめ', '保存しない（見るたびに作る）', 'コミットする', 'PR を見る人と、経緯をたどる人が読む'],
 ['判定（rule）：インスタンス × rule', 'コミットする', 'コミットする（変えない）', 'ほかの人の手元と CI が verified を決める材料'],
 ['判定（意図）：I33 ・ 人の承認', 'コミットする', '手元の DB。結果はまとめに入る', '意図の判定は変更の記録が対象で、インスタンスの状態の導出には使わない（I33 ・ I17）'],
 ['セッション ・ 手元の索引', '手元の DB', '手元の DB（変えない）', ''],
]
data = json.dumps({"S": S}, ensure_ascii=False)
body = f'''<main>
<h1>変更のまとめのシミュレーション</h1>
<p class="ans">1件ずつの変更の記録は、作業者がセッションの中で確かめるために手元の DB に置く。コミットするのは、目的ごとのまとめ（ACDR のように確かめやすい形）と、ほかの人が verified を決めるのに使う rule の判定。まとめは基盤が記録から機械的に作り、AI は書かない</p>

<h2>1　何をどこに置くか</h2>
{table(['もの','いま（model.json）','この形','なぜ'], PLACE)}

<h2>2　目的のセッションの流れ：「0199 依存の向きの検査ツール」</h2>
<div class="sim">
 <div class="ctl"><button id="prev">← 前</button><span id="pos"></span><button id="next">次 →</button></div>
 <p class="who" id="who"></p>
 <div class="stp">
  <div class="sp"><h3>事前状態</h3><div class="lanes"><div><h4>手元の DB</h4><div id="b0"></div></div><div><h4>リポジトリ</h4><div id="b1"></div></div></div></div>
  <div class="arr">→</div>
  <div class="sp"><h3>事後状態</h3><div class="lanes"><div><h4>手元の DB</h4><div id="a0"></div></div><div><h4>リポジトリ</h4><div id="a1"></div></div></div></div>
 </div>
</div>

<h2>3　読む人ごとに、まとめで足りるか</h2>
{table(['読む人','使う場面','読むもの','分かること','結果'], READERS)}

<h2>4　この形にすると書き換える前提</h2>
{table(['前提','いまの文','書き換えた文'], [
 ['保存の決まり ・ I40','変更の記録 ・ 判定の記録の正本は、確定するファイル','rule の判定とまとめは、確定するファイル。変更の記録と意図の判定は、手元の DB'],
 ['境界（変更の記録の置き場所）','文脈のリポジトリ','手元の DB。まとめは文脈のリポジトリ'],
 ['I35','まとまりは保存しない。出すたびに記録から作る','コミットの前に、前回のコミットからの差し引きと例外でまとめを作って保存する。1件の変更の記録は、1つのまとめにだけ入る'],
 ['I31','取り込みの時点で、記録と変更の前後を両方向で突き合わせる','まとめを作るとき（作業者の手元）に両方向で突き合わせ、CI はコミットの差分とまとめを片方向で突き合わせる'],
 ['I28','判定の記録と変更の記録と履歴から、誰がいつ何を変えたかが分かる','rule の判定とまとめと履歴から分かる'],
 ['I34','確定する変更の記録と判定の記録にセッション ID を入れない','まとめにもセッション ID を入れない'],
 ['U25 の受け入れ','条件に紐づくすべての変更の記録が出る','条件に紐づくまとめが出る'],
 ['U16','人の判定の記録が意図の未確定の変更に入り、基盤がポートで確定する','人の意図の承認は手元の DB に入り、結果がまとめに入る'],
 ['「変更の記録を基盤が持つ理由」の流れ4 ・ 5','4：インスタンスと一緒にコミット　5：PR を見る人へまとめる（その場で作る）','4：インスタンスと一緒にまとめをコミット　5：PR を見る人は、コミットされたまとめを読む'],
 ['Abstract のファイル','（まとめの形が無い）','change-summary を足す（procedure と合わせて2つ増える）'],
])}

<h2>5　決めたこと</h2>
{table(['決めたこと','根拠'], [
 ['変更の記録と意図の判定は、手元の DB に置く','意図の判定の対象は変更の記録で、インスタンスの状態の導出には使わない（I33 ・ I17）。作業者の手元にあれば足りる'],
 ['まとめはコミットごとに1件。中身は前回のコミットからの差し引きと例外。id は打ち合わせなしで一意な値','コミットの差分とまとめが1対1になり、CI が「まとめに無い書き換え」を機械的に出せる'],
 ['まとめは基盤が機械的に作る。AI が書く欄は持たない','意図の文が説明になっている。AI の欄を足すと、その欄の rule と判定を必要とする'],
 ['rule の判定はコミットに残す','状態は変更に入れた判定の記録だけから導く（I17）。残さないと、ほかの人の手元と CI では unverified になる'],
 ['同じ目的を引き継ぐときは、引き継ぐ前にコミットする','手元の DB はほかの人に見えない'],
])}
<p class="meta">0185 ・ C-1〜C-4 ・ J-1〜J-4 は例。まとめのファイル名 changes/01JB7….json は、打ち合わせなしで一意な id の例</p>
</main>
<style>
main>*,.sim>*{{min-width:0;max-width:100%}}
.sim{{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:12px;display:flex;flex-direction:column;gap:10px}}
.ctl{{display:flex;gap:8px;align-items:center;flex-wrap:wrap}}
.ctl button{{font:inherit;font-size:13px;padding:4px 14px;border-radius:999px;border:1px solid var(--line);background:var(--bg);color:var(--ink);cursor:pointer}}
#pos{{font-weight:700}}
p.who{{margin:0;padding:6px 10px;border-radius:8px;background:var(--hot-bg);font-weight:700}}
.stp{{display:grid;grid-template-columns:minmax(0,1fr) auto minmax(0,1fr);gap:10px;align-items:start}}
@media (max-width:860px){{.stp{{grid-template-columns:1fr}} .arr{{transform:rotate(90deg);justify-self:center}}}}
.arr{{align-self:center;font-size:24px;color:var(--accent)}}
.sp{{display:flex;flex-direction:column;gap:6px;min-width:0}} .sp h3{{margin:0;font-size:13px}}
.lanes{{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1fr);gap:8px}}
.lanes>div{{display:flex;flex-direction:column;gap:6px;min-width:0;background:var(--bg);border-radius:10px;padding:6px}}
.lanes h4{{margin:0;font-size:12px;color:var(--sub)}}
@media (max-width:420px){{.lanes{{grid-template-columns:1fr}}}}
.cd{{border:1.5px solid var(--line);border-radius:10px;background:var(--card);padding:6px 8px;display:flex;flex-direction:column;gap:3px;min-width:0}}
.cd.new{{border-color:var(--hot);background:var(--hot-bg)}}
.cd.gone{{opacity:.5;border-style:dashed}}
.cd.k-まとめ{{border-color:var(--accent);border-width:2px}}
.chd{{display:flex;flex-wrap:wrap;gap:6px;align-items:center;font-size:12.5px}}
.ck2{{font-size:11px;font-weight:700;color:var(--sub);border:1px solid var(--line);border-radius:6px;padding:0 5px}}
.ct{{font-family:var(--mono);font-size:11px;color:var(--sub)}}
.bd{{font-size:11px;font-weight:700;border-radius:999px;padding:0 8px;border:1.5px solid var(--line)}}
.bd.コミット,.bd.コミット済み,.bd.まとめに入った{{border-color:var(--accent);color:var(--accent)}}
.bd.was{{opacity:.55}}
.fr{{display:grid;grid-template-columns:auto minmax(0,1fr);gap:8px;font-size:12px;border-radius:4px;padding:0 4px}}
.fk{{color:var(--sub)}} .fv{{overflow-wrap:anywhere}}
.fr.chg{{background:var(--hot-bg);font-weight:700}}
.fr s{{display:block;font-weight:400;opacity:.55}}
.nt2{{font-size:11px;font-weight:700;color:var(--hot)}}
.none{{font-size:12px;color:var(--sub);margin:0}}
</style>
<script>
const D={data};
function esc(s){{return String(s).replace(/[&<>]/g,c=>({{'&':'&amp;','<':'&lt;','>':'&gt;'}}[c]))}}
function cards(list, other, side){{
 if(!list.length)return '<p class="none">（無い）</p>';
 return list.map(c=>{{const o=other.find(x=>x.n===c.n&&x.k===c.k);
  const isNew=side==='post'&&!o, gone=side==='pre'&&!o;
  let st=c.s?'<span class="bd '+c.s+'">'+esc(c.s)+'</span>':'';
  if(side==='post'&&o&&o.s!==c.s&&c.s)st='<span class="bd '+o.s+' was">'+esc(o.s)+'</span>→'+st;
  const rows=Object.entries(c.f).map(([k,v])=>{{const ov=o?o.f[k]:undefined;const chg=side==='post'&&o&&ov!==v;
   return '<div class="fr'+(chg?' chg':'')+'"><span class="fk">'+esc(k)+'</span><span class="fv">'+esc(v)+(chg&&ov!==undefined?'<s>'+esc(ov)+'</s>':'')+'</span></div>'}}).join('');
  return '<div class="cd k-'+c.k+(isNew?' new':'')+(gone?' gone':'')+'"><div class="chd"><span class="ck2">'+esc(c.k)+'</span><b>'+esc(c.n)+'</b><span class="ct">'+esc(c.t)+'</span>'+st+'</div>'+rows+(isNew?'<div class="nt2">新しく増えた</div>':'')+'</div>'}}).join('')}}
let i=0;
function draw(){{const s=D.S[i];document.getElementById('pos').textContent=(i+1)+' / '+D.S.length+'　'+s.t+'（'+s.uc+'）';
 document.getElementById('who').textContent=s.who;
 for(const k of [0,1]){{document.getElementById('b'+k).innerHTML=cards(s.pre[k],s.post[k],'pre');document.getElementById('a'+k).innerHTML=cards(s.post[k],s.pre[k],'post')}}}}
document.getElementById('prev').onclick=()=>{{i=Math.max(0,i-1);draw()}};
document.getElementById('next').onclick=()=>{{i=Math.min(D.S.length-1,i+1);draw()}};
draw();
</script>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
