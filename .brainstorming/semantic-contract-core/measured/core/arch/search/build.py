"""search の完成イメージの頁（arch/search/index.html）。model.json の「意味で探して取り出す」・ I24 ・ I25 ・ I41 ・ I42 ・ U5 ・ U6 ・ U12 ・ U24 ・ U26 から、具体の場面を起こしたもの。"""
import os, json, html
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.ex{display:grid;grid-template-columns:1fr 1fr;gap:10px}@media (max-width:760px){.ex{grid-template-columns:1fr}}.ex pre{white-space:pre;overflow-x:auto;padding:8px 12px}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>search の完成イメージ</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
j = lambda o: html.escape(json.dumps(o, ensure_ascii=False, indent=2))
SC = 'urn:semantic-agent-schema:acdr:decision-record:v1'
scenes = [
 ('1　判断待ちの ACDR を探す（人が読む U12 ・ AI が作業の前に U5）',
  '状態と判定の結果で探す。効かなくなった判定は verified として扱わない（I24）',
  {"kind": [SC], "state": ["判断待ち"]},
  {"items": [{"id": "0197-ai-permissions", "kind": SC, "state": "判断待ち",
              "results": [{"rule": "I33", "value": "unverified", "why": "AI の答えが割れた"}]}],
   "scope": {"contexts": ["daidaiiro"], "index_of": "HEAD 60ce3cf6"}}),
 ('2　語「承認」を使っている欄を探す（AI がインスタンスを変える前 U6）',
  'term は glossary の版つきの id で一致を決める。v1 の「承認」と v2 の「承認」は別の語（I25）',
  {"term": "urn:semantic-agent-schema:acdr:glossary:v1#approval"},
  {"items": [{"id": "0197-ai-permissions", "kind": SC, "at": "/decision", "term": "urn:semantic-agent-schema:acdr:glossary:v1#approval"},
             {"id": "0199-dependency-rules", "kind": SC, "at": "/why", "term": "urn:semantic-agent-schema:acdr:glossary:v1#approval"}],
   "scope": {"contexts": ["daidaiiro"], "index_of": "HEAD 60ce3cf6"}}),
 ('3　この ACDR を指しているものを探す（変える前に、壊れる参照を知る U6）',
  'x-relationship をたどる。種類の名前は Concrete の $id の範囲で決まり、別の文脈の同じ名前は別物（I41）',
  {"related_to": {"id": "0199-dependency-rules", "kind": SC}, "direction": "incoming"},
  {"items": [{"id": "0200-dependency-rules-name", "kind": SC, "via": "/supersedes"}],
   "scope": {"contexts": ["daidaiiro"], "index_of": "HEAD 60ce3cf6"}}),
 ('4　ほかの文脈の公開された言葉を読む（U24 ・ 範囲の宣言 U26）',
  '探した範囲を必ず付ける。範囲の外は「探していない」と返し、0件を「無い」と返さない（I42）',
  {"term": "urn:semantic-agent-schema:brainstorming:glossary:v2#settled", "contexts": ["daidaiiro", "other-repo"]},
  {"items": [],
   "scope": {"contexts": ["daidaiiro"], "index_of": "HEAD 60ce3cf6"},
   "not_searched": [{"context": "other-repo", "why": "文脈の地図に無い"}]}),
]
sec = ''
for t, rule, q, r in scenes:
    sec += f'<h3>{t}</h3><p class="meta">{rule}</p><div class="ex"><div class="pane"><b>探す条件</b><pre>{j(q)}</pre></div><div class="pane on"><b>返る結果</b><pre>{j(r)}</pre></div></div>'
body = f'''<main>
<h1>search の完成イメージ</h1>
<p class="meta">利用者「searchの対象だけなんかちゃんと認識を合わせた方がいいと思った。これの完成イメージは？？」への答え。model.json のドメインサービス「意味で探して取り出す」と不変条件 I24 ・ I25 ・ I41 ・ I42 から、具体の場面を起こした。条件と結果の JSON の形はまだ決めていない（protocol は Q4 で待ち）。下の形は案である</p>
<p class="ans">search は、スキーマで決めた意味の単位（種類 ・ term ・ x-relationship ・ 状態と判定の結果）でインスタンスを探し、見つけたものと探した範囲を返す。JSON の中を式で自由に引くものでも、全文検索でもない</p>
<h2>1　何で探すか</h2>
{table(['探す単位', '例', '決まり'], [
 ['種類（kind）', 'ACDR の記録だけ', '種類の名前の範囲は Concrete の $id で決まる（I41）'],
 ['状態と判定の結果', '判断待ちの ACDR、violated のあるインスタンス', 'いまの入力に対する判定だけを使う（I24）'],
 ['term（glossary の語）', '語「承認」を使っている欄', '版つきの id で一致を決める（I25）'],
 ['x-relationship', 'この ACDR を指しているもの、指しているもの', '公開された言葉は、読む側が指した版で探す（I25）'],
 ['Concrete の拡張項目', 'ある x- を使っている欄', '──'],
])}
<h2>2　場面ごとの完成イメージ</h2>
{sec}
<h2>3　search ではないもの</h2>
{table(['用途', 'どこが持つか'], [
 ['書く前に、その欄の x-prompt（読み方と書くときの rule）を受け取る', '同じ入口の「案内する」。探すのではなく、欄の位置を指して受け取る'],
 ['JSON の中を式で自由に引く（JMESPath のような式）', '持たない。返す部分を絞るときの指し方は、前提が欄の位置に使う JSON Pointer にそろえる案（未決）'],
 ['本文の文字列で探す（全文検索）', '持たない。意味の単位で探す'],
 ['関係の索引の保存', '索引は正本のファイルから作り直せるもので、遅いと測れてから手元の DB を同じポートの後ろに置く（ports.outbound）'],
])}
<h2>4　確かめてほしいこと</h2>
{table(['確かめてほしいこと', '私の見解'], [
 ['search が探す単位を、1節の5つ（種類 ・ 状態と判定の結果 ・ term ・ x-relationship ・ 拡張項目）にする', 'この5つにする。model.json の定義と同じ'],
 ['x-prompt を受け取るのは search ではなく「案内する」にする', '分ける。探すのではなく、位置を指して受け取るため'],
 ['JMESPath のような式の言語を持たない', '持たない。返す部分は JSON Pointer で指す'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
