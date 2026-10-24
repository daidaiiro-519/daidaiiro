"""search の完成イメージの頁（arch/search/index.html）。model.json の「意味で探して取り出す」・ I24 ・ I25 ・ I41 ・ I42 ・ U5 ・ U6 ・ U12 ・ U24 ・ U26 から、具体の場面を起こしたもの。"""
import os, json, html
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.ex{display:grid;grid-template-columns:1fr 1fr;gap:10px}@media (max-width:760px){.ex{grid-template-columns:1fr}}.ex pre{white-space:pre;overflow-x:auto;padding:8px 12px}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>search の完成イメージ</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
j = lambda o: html.escape(json.dumps(o, ensure_ascii=False, indent=2))
CS = 'urn:semantic-agent-schema:sample:record:v1'   # Concrete（文脈 sample が持つ種類 record）
TERM = 'urn:semantic-agent-schema:sample:glossary:v1#owner'
SCOPE = {"contexts": ["sample"], "index_of": "<正本の版>"}
scenes = [
 ('1　判断待ちのインスタンスを探す（人が読む U12 ・ AI が作業の前に U5）',
  'インスタンスを、種類（Concrete の $id）と状態で探す。効かなくなった判定は verified として扱わない（I24）',
  {"kind": [CS], "state": ["判断待ち"]},
  {"instances": [{"id": "r-12", "kind": CS, "state": "判断待ち",
                  "results": [{"rule": "owner-is-person", "value": "unverified", "why": "AI の答えが割れた"}]}],
   "scope": SCOPE}),
 ('2　glossary の語 owner を使っているスキーマの欄とインスタンスを探す（AI がインスタンスを変える前 U6）',
  'term は glossary の版つきの id で一致を決める。v1 の owner と v2 の owner は別の語（I25）。Concrete の欄が $ref で term を指す',
  {"term": TERM},
  {"schemas": [{"schema": CS, "at": "/properties/owner"}],
   "instances": [{"id": "r-12", "kind": CS, "at": "/owner"}, {"id": "r-15", "kind": CS, "at": "/owner"}],
   "scope": SCOPE}),
 ('3　このインスタンスを指しているインスタンスを探す（変える前に、壊れる参照を知る U6）',
  'Concrete の x-relationship をたどる。種類の名前は Concrete の $id の範囲で決まり、別の文脈の同じ名前は別物（I41）',
  {"related_to": {"id": "r-12", "kind": CS}, "direction": "incoming"},
  {"instances": [{"id": "r-20", "kind": CS, "via": "/depends_on"}], "scope": SCOPE}),
 ('4　ほかの文脈の公開された言葉で探す（U24 ・ 範囲の宣言 U26）',
  '探した範囲を必ず付ける。範囲の外は「探していない」と返し、0件を「無い」と返さない（I42）',
  {"term": "urn:semantic-agent-schema:other:glossary:v2#status", "contexts": ["sample", "other"]},
  {"schemas": [], "instances": [], "scope": SCOPE,
   "not_searched": [{"context": "other", "why": "文脈の地図に無い"}]}),
]
sec = ''
for t, rule, q, r in scenes:
    sec += f'<h3>{t}</h3><p class="meta">{rule}</p><div class="ex"><div class="pane"><b>探す条件</b><pre>{j(q)}</pre></div><div class="pane on"><b>返る結果</b><pre>{j(r)}</pre></div></div>'
body = f'''<main>
<h1>search の完成イメージ</h1>
<p class="meta">利用者「searchの対象だけなんかちゃんと認識を合わせた方がいいと思った。これの完成イメージは？？」への答え。model.json のドメインサービス「意味で探して取り出す」と不変条件 I24 ・ I25 ・ I41 ・ I42 から、具体の場面を起こした。条件と結果の JSON の形はまだ決めていない（protocol は Q4 で待ち）。下の形は案である</p>
<p class="ans">search は、Concrete Schema とそれに従うインスタンスを、スキーマで決めた意味の単位（種類 ・ term ・ x-relationship ・ 状態と判定の結果 ・ 拡張項目）で探し、見つけたものと探した範囲を返す。JSON の中を式で自由に引くものでも、全文検索でもない</p>
<h2>1　何を探すか（Abstract ・ Concrete ・ インスタンスのどれか）</h2>
{table(['層', 'search の対象か', '探すもの'], [
 ['Semantic Abstract Schema（基盤の10ファイル）', '対象にしない', 'Concrete とインスタンスの形を決める側。基盤の中にあり、探して見つけるものではない'],
 ['Semantic Concrete Schema（文脈のスキーマ）と glossary', '対象', 'term を使っている欄、x- の拡張項目を使っている欄、x-relationship の宣言'],
 ['インスタンス（Concrete に従うデータ）', '対象', '種類 ・ 状態と判定の結果 ・ term を使う欄の値 ・ x-relationship でつながるインスタンス'],
])}
<p class="meta">下の例の文脈 sample ・ 種類 record ・ 語 owner は、説明のための架空の名前である</p>
<h2>2　何で探すか</h2>
{table(['探す単位', '例', '決まり'], [
 ['種類（kind）', 'Concrete record に従うインスタンスだけ', '種類の名前の範囲は Concrete の $id で決まる（I41）'],
 ['状態と判定の結果', '判断待ちのインスタンス、violated のあるインスタンス', 'いまの入力に対する判定だけを使う（I24）'],
 ['term（glossary の語）', '語 owner を使っているスキーマの欄と、その欄を持つインスタンス', '版つきの id で一致を決める（I25）'],
 ['x-relationship', 'このインスタンスを指しているインスタンス、指しているインスタンス', '公開された言葉は、読む側が指した版で探す（I25）'],
 ['Concrete の拡張項目', 'ある x- を使っているスキーマの欄', '──'],
])}
<h2>3　場面ごとの完成イメージ</h2>
{sec}
<h2>4　search ではないもの</h2>
{table(['用途', 'どこが持つか'], [
 ['書く前に、その欄の x-prompt（読み方と書くときの rule）を受け取る', '同じ入口の「案内する」。探すのではなく、欄の位置を指して受け取る'],
 ['JSON の中を式で自由に引く（JMESPath のような式）', '持たない。返す部分を絞るときの指し方は、前提が欄の位置に使う JSON Pointer にそろえる案（未決）'],
 ['本文の文字列で探す（全文検索）', '持たない。意味の単位で探す'],
 ['関係の索引の保存', '索引は正本のファイルから作り直せるもので、遅いと測れてから手元の DB を同じポートの後ろに置く（ports.outbound）'],
])}
<h2>5　確かめてほしいこと</h2>
{table(['確かめてほしいこと', '私の見解'], [
 ['search の対象を Concrete Schema ・ glossary ・ インスタンスにし、Abstract は対象にしない', 'そうする'],
 ['search が探す単位を、2節の5つ（種類 ・ 状態と判定の結果 ・ term ・ x-relationship ・ 拡張項目）にする', 'この5つにする。model.json の定義と同じ'],
 ['x-prompt を受け取るのは search ではなく「案内する」にする', '分ける。探すのではなく、位置を指して受け取るため'],
 ['JMESPath のような式の言語を持たない', '持たない。返す部分は JSON Pointer で指す'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
