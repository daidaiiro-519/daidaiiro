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
a = {"answer": {"$schema": AS, "items": [{"record": "r-12", "owner": "yamada",
      "provenance": [{"id": "r-12", "kind": CS, "instanceLocation": "/owner", "version": "<読んだ版>"}]}]},
     "results": [{"rule": "provenance-exists", "by": "machine", "value": "verified"},
                 {"rule": "answer-supported", "by": "ai", "value": "unverified", "why": "親が判定を依頼する"}],
     "scope": {"contexts": ["sample"], "index_of": "<正本の版>"}}
body = f'''<main>
<h1>Semantic Search と用語</h1>
<p class="meta">利用者「自然な言語からそれに該当するようなドキュメントjsonを探し、必要な情報を取得してある形式の回答用のスキーマに当てはめて返す…それはセマンティックな検索といえますか？…必要なんじゃないでしょうか？」「それぞれの呼称はちゃんと用語としてまとめておきたい」「今そこを見直してるって話だから…どうするべきかって言う形で返して欲しい」を受け、usecase-advisor ・ ddd-advisor ・ 結論を渡さない反証役を統合した。例の名前は架空</p>
<p class="ans">自然な言葉の問いに答えのスキーマで答える Semantic Search を、使う場面 U28 として足す。作業する AI は問う者のセッションで、基盤の機械的な操作（Search ・ Semantic Read の読み取り ・ 渡した値の検証）を組み合わせる。基盤が起動するのは、答えが出どころに支えられているかを判定させる AI だけ</p>
{fig}
<h2>1　用語（セマンティックか機械的か）</h2>
{table(['英語', '日本語', '種類', '中身'], [
 ['Search', 'サーチ', '機械的', '種類 ・ term ・ x-relationship ・ 状態と判定の結果 ・ 拡張項目で探し、位置の一覧と探した範囲を返す。使う場面の中から呼ぶ部品'],
 ['Semantic Search', 'セマンティックサーチ', 'セマンティック', '使う場面 U28。自然な言葉の問いに、答えのスキーマで、出どころ付きで答える'],
 ['Semantic Read', 'セマンティックリード', 'セマンティック', 'AI が x-prompt を読む。基盤の読み取りのコマンドの部分は機械的'],
 ['Semantic Write', 'セマンティックライト', 'セマンティック', 'AI が x-prompt に従って書く。JSON Patch の適用と検証は機械的'],
 ['Semantic Schema Resolver', 'セマンティックスキーマリゾルバー', '機械的', '指す先と版を解決する'],
 ['Semantic Schema Validator', 'セマンティックスキーマバリデーター', '両方', '機械の rule は自分で検証し、AI の rule は依頼の一覧を返して答えをまとめる'],
 ['Semantic Schema Renderer', 'セマンティックスキーマレンダラー', '機械的', '人が読む形にする'],
 ['Session Selector', 'セッションセレクター', '機械的', '再開する ID か「新しく立てる」を返す'],
])}
{table(['名前の決まり', '中身'], [
 ['操作の名前の先頭の Semantic', 'AI が伴う操作（Semantic Search ・ Read ・ Write）。あなたの定義「AI が伴うか、コマンドだけで完結するか」'],
 ['Semantic Schema', 'この基盤のスキーマの名前（Semantic Abstract Schema ・ Semantic Concrete Schema）。Semantic Schema Resolver ・ Validator ・ Renderer の Semantic は、操作ではなく「どのスキーマを扱うか」を表す'],
])}
<h2>2　送る JSON と返る JSON</h2>
<div class="ex"><div class="pane"><b>問う（AI のセッションが受け取る問い）</b><pre>{j(q)}</pre></div><div class="pane on"><b>返る（検証された答え。保存しない）</b><pre>{j(a)}</pre></div></div>
<p class="meta">出どころが実在し版が合うかは機械の rule で決定論的に確かめる。答えの値が出どころに支えられているか（写した値でない要約 ・ 組み合わせを含む）は AI の rule で、結果が出るまで unverified として見える</p>
<h2>3　あるべき形と、書き換える前提</h2>
{table(['あるべき形', '変更前', '変更後'], [
 ['セマンティックな検索を使う場面として持つ', '使う場面に無い（U0〜U27）', 'U28 Semantic Search を足す。行為者は問う者（目的のセッションの AI）。業務領域は中核「意味で探す」'],
 ['機械的な検索の名前を、AI が伴わないと分かるものにする', 'Q5 ・ Q9 と model.json の「意味で探す」「意味で探して取り出す」が、AI の伴わない検索を指している', 'Search（サーチ）。業務領域「意味で探す」には Search と Semantic Search の両方が入る'],
 ['作業する AI を起動するのは外（呼ぶ側）にそろえる', 'decisions-in-session.md に「親のセッションを基盤が立ててもよい」、boundary に「AI ツールの起動は外」が並んでいる', '「AI ツールの起動は外」にそろえる。基盤が起動するのは、ツールを持たせない判定の AI だけ（I29）'],
 ['保存しない値を検証できるようにする', '駆動する側のポート「検証する」は変更の前後を受け取る形だけ', '「渡した値を検証する（保存しない）」を足す。Semantic Search の答えの検証に使う'],
 ['出どころを持つ', '無い', 'Abstract の $defs に値オブジェクト「出どころ」（id ・ kind ・ instanceLocation ・ 読んだ版）と、rule 2つ（実在と版 ＝ 機械、支えられているか ＝ AI）を足す。型は instance の crate'],
 ['Semantic Read ・ Write の頁の言い方', '「抽象のレベルでは外れない」', '「機械の rule では外れない。AI の rule は判定の結果として残る」'],
 ['答えを作る操作', 'Semantic Write で答えを書く（下書き）', '答えはファクトリで作る。Semantic Write は既にある文書の更新に限る'],
])}
<p class="meta">crate は8つのまま、依存の向きも変わらない。Semantic Search はアプリケーション層の使う場面。Q3（Agents 契約）に「作業の任せる先に Search や Semantic Read を渡す契約が要るか」の論点を足す</p>
<h2>4　確かめてほしいこと</h2>
{table(['確かめてほしいこと', '私の見解'], [
 ['U28 Semantic Search を足し、作業する AI は問う者のセッションにする', 'そうする。問いの文脈を持つのはそのセッションだけ'],
 ['用語の表（1節）と、名前の決まり（操作の Semantic と Semantic Schema の違い）', 'そうする'],
 ['3節の書き換え', 'そうする'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
