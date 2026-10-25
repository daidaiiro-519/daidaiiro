"""Semantic Search と用語の頁（arch/semantic-search/index.html）。usecase-advisor ・ ddd-advisor ・ 反証役を統合（arch/reviews/usecase-semantic-search.* ・ ddd-semantic-search.*）。"""
import os, json, html
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.fig{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px;overflow-x:auto}.ex{display:grid;grid-template-columns:1fr 1fr;gap:10px}@media (max-width:760px){.ex{grid-template-columns:1fr}}.ex pre{white-space:pre;overflow-x:auto;padding:8px 12px}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>Semantic Search と用語</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
j = lambda o: html.escape(json.dumps(o, ensure_ascii=False, indent=2))
CS = 'urn:semantic-agent-schema:sample:record:v1'
AS = 'urn:semantic-agent-schema:sample:answer-owner:v1'
fig = '<div class="fig">' + open(os.path.join(H, 'flow.svg')).read() + '</div>'
q = {"question": "AI に何を許すかを決めた記録と、その責任者は？", "answer_schema": AS, "contexts": ["sample"]}
cond = {"$schema": "urn:semantic-agent-schema:core:query:v1", "kind": [CS], "term": ["urn:semantic-agent-schema:sample:glossary:v1#permission"], "contexts": ["sample"]}
a = {"query": cond,
     "answer": {"$schema": AS, "items": [{"record": "r-12", "owner": "yamada",
      "provenance": [{"x-relationship": {"id": "r-12", "kind": CS}, "instanceLocation": "/owner", "version": "<読んだ版>"}]}]},
     "results": [{"rule": "provenance-in-scope", "by": "machine", "value": "verified"},
                 {"rule": "copied-value-matches", "by": "machine", "value": "verified"},
                 {"rule": "answer-supported", "by": "ai", "value": "unverified"}],
     "needs": [{"rule": "answer-supported", "why": "答えを使う前に、問う者が判定を依頼する"}],
     "scope": {"contexts": ["sample"], "index_of": "<正本の版>"}}
body = f'''<main>
<h1>Semantic Search と用語</h1>
<p class="meta">利用者と私で固めた見解（AI は整形を受け持ち、処理は機械的。入出力は JSON Schema の契約で決定論的。セッションを乱立させない）を、見解と理由と見直しの意図を渡した反証役に通し、直した。例の名前は架空</p>
<p class="ans">Semantic Search は、自然な言葉の問いに、答えのスキーマで出どころ付きで答える使う場面（U28）。AI がするのは3つ（問いを条件の JSON に整形する ・ 候補から選ぶ ・ 答えの JSON に整形する）で、探す ・ 読む ・ 確かめるは基盤の機械的なコマンド。決定論的と言えるのは条件の JSON から先で、使った条件も返す</p>
{fig}
<h2>1　用語</h2>
{table(['名前', '種類', '中身'], [
 ['Semantic Search（セマンティックサーチ）', '使う場面 ・ AI が伴う', 'U28。問い → 条件 → Search → Get → 選ぶ → 答え → Validate'],
 ['Semantic Read（セマンティックリード）', '使う場面 ・ AI が伴う', 'AI が x-prompt を読んで、欄の読み方と書き方を知る。中で Get を使う'],
 ['Semantic Write（セマンティックライト）', '使う場面 ・ AI が伴う', 'AI が x-prompt に従って書く。中で Patch を使う'],
 ['Search（サーチ）', 'コマンド ・ 機械的', '条件の JSON で探し、位置の一覧 ・ 探した範囲 ・ 使った条件を返す'],
 ['Get（ゲット）', 'コマンド ・ 機械的', '欄の x-prompt ・ 値 ・ 版を返す'],
 ['Patch（パッチ）', 'コマンド ・ 機械的', 'JSON Patch（RFC 6902。test ・ add ・ replace ・ remove）を適用し、検証の結果と needs を返す'],
 ['Validate（バリデート）', 'コマンド ・ 機械的', '渡した値を保存せずに検証し、結果と needs を返す'],
 ['Semantic Schema Resolver ・ Validator ・ Renderer', 'ドメインサービス ・ 機械的', 'Validator は AI の rule を自分で判定せず needs として返す。判定を依頼するのは呼ぶ側'],
 ['Session Selector（セッションセレクター）', 'ドメインサービス ・ 機械的', '再開する ID か「新しく立てる」を返す'],
])}
{table(['名前の決まり', '中身'], [
 ['先頭の Semantic（操作の名前）', 'AI が伴う使う場面。基盤のコマンドの名前には付けない'],
 ['Semantic Schema', 'この基盤のスキーマの名前の一部（Semantic Abstract ・ Concrete Schema）。AI の有無ではない'],
 ['Get ・ Patch', 'HTTP の GET と RFC 5789 の PATCH と同じ語。x-prompt の read ・ write のキーと紛れない'],
])}
<h2>2　送る JSON と返る JSON</h2>
<div class="ex"><div class="pane"><b>問う ／ AI が作った条件（Abstract「探す条件」の形）</b><pre>{j(q)}</pre><pre>{j(cond)}</pre></div><div class="pane on"><b>返る（保存しない）</b><pre>{j(a)}</pre></div></div>
<p class="meta">出どころの id と kind は x-relationship で書き、足すのは instanceLocation と読んだ版だけ。会話から思い出した値は出どころが ② の範囲に無いので violated になる</p>
<h2>3　AI をどこで動かすか</h2>
{table(['呼ぶ側', 'どこで', '決まり'], [
 ['AI のセッション（いちばん多い）', 'いまのセッション（新しく立てない）', '会話の値が答えに混ざっても、出どころが範囲の中 ・ 値の一致 ・ 版の機械の rule で violated になる'],
 ['AI のセッションで、文脈を分けたいとき', '目的のセッションの任せる先「探す役」を、Session Selector で再開する', '目的 ・ 親 ・ 役割で1つまで。同じ枠への問いは前の問いが返るまで次を出さない。オーケストレーターからは立てない（いまのセッションで行う）'],
 ['AI でない呼ぶ側（CI ・ harness）', 'Search を直接呼ぶ', '自然な言葉の問いを持たないので Semantic Search を使わない（あなたに確かめたいこと）'],
])}
<h2>4　あるべき形と、書き換える前提</h2>
{table(['前提', '変更前', '変更後'], [
 ['使う場面', 'U0〜U27', 'U28 Semantic Search を足す。成功の保証：各値の出どころは同じ呼び出しで Search が返した範囲の中にあり、版と写した値が正本と合う。使った条件を返す。支えられているかの判定は needs で返る'],
 ['手順とコマンドの一覧', '無い', 'Abstract のファイル「探す条件」（Search の入力と結果の形）を足し、その x-prompt に手順と各欄の書き方を書く。kind と term の候補は Resolver が enum として出す。Abstract は10ファイル → 11ファイル'],
 ['出どころ', '無い', 'Abstract の $defs に足す（x-relationship ＋ instanceLocation ＋ 読んだ版）。rule は、範囲の中にある ・ 写した値が一致する ・ 版が合う（機械）と、支えられている（AI）'],
 ['駆動する側のポート', '探して取り出す ・ 案内する ／ インスタンスを書く ／ 検証する', 'コマンドは Search ・ Get ・ Patch ・ Validate（渡した値、保存しない）。使う場面は Semantic Read ・ Write ・ Search'],
 ['判定させる AI を誰が呼ぶか', '頁と model.json に「基盤が起動する」が残っていた', 'Validator は needs を返すだけ。答えを使う前に、問う者がまとめて依頼する（decisions-in-session.md「呼ぶのは親」）'],
 ['値の意味で探すこと（Semantic Read ・ Write の頁の5節）', 'A（値では探さない）か B（JMESPath を条件に使う）が未決', 'A。自由記述の値の意味で選ぶのは AI。前もって rule の判定の結果として残しておけば、Search が「状態と判定の結果」で機械的に探せる'],
 ['Q8 ・ Q9 の「意味で探す」', '語 ・ 参照 ・ rule の結果の意味で探す', 'Search はそのまま。値の意味で選ぶのは Semantic Search の AI の受け持ちと書き足す'],
])}
<h2>5　確かめてほしいこと</h2>
{table(['確かめてほしいこと', '私の見解'], [
 ['AI がするのは3つ（条件への整形 ・ 選ぶ ・ 答えへの整形）で、決定論的なのは条件の JSON から先', 'そうする。あなたの「整形」に「選ぶ」を足した。値の中身で選ぶ部分は機械の5つの単位では表せないため'],
 ['用語（使う場面は Semantic ○○、コマンドは Search ・ Get ・ Patch ・ Validate）', 'そうする'],
 ['AI でない呼ぶ側（CI ・ harness）は Semantic Search を使わず Search を直接呼ぶ', '反証役の推測。あなたの使い方と合うか確かめたい'],
 ['値の意味で探すのは A（JMESPath は使わない）', 'A。意味は rule の判定の結果として残し、機械で探す'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
