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
proc = {"for": "semantic-search", "question": "AI に何を許すかを決めた記録と、その責任者は？", "answer_schema": AS}
proc_out = {"手順": ["問いを入力のスキーマ（query の request）の形に整形する", "search を呼ぶ", "結果を答えのスキーマの形に整形し、出どころを付ける", "validate を呼ぶ"],
            "入力のスキーマ": "urn:semantic-agent-schema:base:query:v1#/$defs/request", "出力のスキーマ": AS,
            "使ってよいコマンド": ["search", "get", "validate"],
            "rules": [{"id": "where-is-jmespath", "rule": "where はインスタンス1件に評価する JMESPath の式。JMESPath の真偽の決まり（false ・ null ・ 空の文字列 ・ 空の配列 ・ 空のオブジェクト以外が真）で真のものを選ぶ"}],
            "候補": {"kind": [CS], "term": ["urn:semantic-agent-schema:sample:glossary:v1#permission"]}}
cond = {"kind": [CS], "where": "contains(subject, 'AI') && decision != `null`", "select": {"owner": "owner", "decision": "decision"}, "contexts": ["sample"]}
res = {"$schema": "urn:semantic-agent-schema:base:query:v1#/$defs/result", "query": cond, "items": [{"id": "r-12", "kind": CS, "version": "<読んだ版>", "values": {"owner": "yamada", "decision": "許可は各ツールの設定に任せる"}}],
       "scope": {"contexts": ["sample"], "index_of": "<正本の版>"}, "not_searched": []}
TERM = "urn:semantic-agent-schema:sample:glossary:v1#permission"
cond2 = {"kind": [CS], "term": [TERM], "select": {"subject": "subject"}, "contexts": ["sample", "other"]}
res2 = {"$schema": "urn:semantic-agent-schema:base:query:v1#/$defs/result", "query": cond2,
        "items": [{"id": "r-12", "kind": CS, "version": "<読んだ版>", "values": {"subject": "AI に何を許すか"}}],
        "scope": {"contexts": ["sample"], "index_of": "<正本の版>"},
        "not_searched": [{"context": "other", "why": "文脈の地図に無い"}]}
ans = {"answer": {"$schema": AS, "items": [{"record": "r-12", "owner": "yamada",
        "provenance": {"id": "r-12", "kind": CS, "version": "<読んだ版>", "where": cond["where"], "select": "owner"}}]},
       "results": [{"rule": "provenance-reproduces", "by": "machine", "pointer": "/items/0/provenance", "result": "verified", "reason": "その版で where が真になり、select の owner を評価し直すと yamada"}]}
body = f'''<main>
<h1>Semantic Search と用語</h1>
<p class="meta">利用者「処理自体は多分全部機械的…整形する部分がaiが行う部分で、それを整形する部分の契約としてjsonスキーマがある」「サーチする対象はこの項目だけではなく、どっちかというとインスタンスもあるし、Schemaもあるし、なんならJsonではないような普通のドキュメントやソースだって対象」「返す方はschemaで定義した型」。決まった内容は ACDR 0201（正本：model.json ・ ボード）。例の名前は架空</p>
<p class="ans">Semantic Search は、自然な言葉の問いに答えのスキーマで答える使う場面（U28）。semantic-search は procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）を返し、AI は問いを条件（query#/$defs/request）に、結果（query#/$defs/result）を答えに整形するだけ。探す ・ 確かめるは機械的なコマンドで、出どころに式と版を残すので、答えは何度でも同じ結果で確かめられる。状態は変えない</p>
{fig}
<h2>1　コマンドと用語</h2>
{table(['コマンド', '種類', '返すもの'], [
 ['semantic-search', 'セマンティック（procedure を返す）', '手順 ・ 入力のスキーマ（query#/$defs/request）・ 出力のスキーマ（答えのスキーマ）・ 書き方の rule ・ kind と term の候補'],
 ['semantic-read ・ semantic-write ・ semantic-create ・ semantic-judge（残すかは未決）・ semantic-import ・ semantic-migrate', 'セマンティック（procedure を返す）', 'AI が行うほかの操作の手順と入出力のスキーマ（Semantic Read と Write の頁）'],
 ['search', '機械的', '検査①のあと、インスタンス ・ スキーマごとに JMESPath の where を評価し、query#/$defs/result（id ・ 版 ・ 式 ・ select の値 ・ 探した範囲 scope ・ 範囲の外 not_searched）を返す。範囲の外は「探していない」と返し、0件を「無い」と返さない（I42）'],
 ['get', '機械的', 'data.path ・ pointer ・ value ・ etag（実装済み）。x-prompt と、スキーマの $ref を解いた形は、semantic-read の procedure が返すもの（未実装）'],
 ['validate', '機械的', '検査②。&lt;パス&gt; の文書を検証し、data.results の各行（rule ・ by ・ pointer ・ result ・ reason）を返す（実装済み）。保存していない答えを渡す入力と、出どころの式をその版で評価し直す検査（provenance-reproduces）は、まだ実装に無い'],
])}
<p class="meta">AI が行う操作はすべてセマンティックのコマンドで、procedure を返す。AI は手順に従って入力を生成し、機械的なコマンドを呼ぶ</p>
<h2>2　探す対象</h2>
{table(['対象', '探し方', '返す型'], [
 ['インスタンス', 'JMESPath の where ・ select をインスタンス1件ずつに評価する', 'query#/$defs/result'],
 ['スキーマ（Concrete ・ glossary ・ published）', '同じ（スキーマも JSON）。例：ある rule を使っている Concrete', 'query#/$defs/result'],
 ['ドキュメント（Markdown ・ ソース）', '本文は基盤が探さない。変更の記録とまとめ（JSON）から、意図 ・ 目的で変えたファイルのパスを引く。本文は AI が読む', 'query#/$defs/result（パス）'],
 ['集まり（文脈 ・ 探す範囲）', 'contexts と、文脈の settings の探す範囲。文脈の地図に無い文脈は探さない', '結果の scope（探した範囲）と not_searched（探していない文脈と理由）'],
])}
<h2>3　送る JSON と返る JSON</h2>
<p class="meta">状態を変えない操作なので、事後状態は事前状態と同じ。増えるのは出力だけ</p>
<div class="ex"><div class="pane"><b>① semantic-search に問う</b><pre>{j(proc)}</pre></div><div class="pane on"><b>procedure が返る</b><pre>{j(proc_out)}</pre></div></div>
<div class="ex"><div class="pane"><b>② AI が生成した入力 → ③ search（検査①）</b><pre>{j(cond)}</pre></div><div class="pane on"><b>出力：query#/$defs/result</b><pre>{j(res)}</pre></div></div>
<div class="ex"><div class="pane"><b>②′ term で探し、文脈の地図に無い文脈を渡した入力</b><pre>{j(cond2)}</pre></div><div class="pane on"><b>出力：範囲の外は not_searched で返る</b><pre>{j(res2)}</pre></div></div>
<p class="meta">request は kind ・ term ・ where ・ select ・ contexts を持つ（形は Abstract のファイル query の予定。query のファイルはボードの schemas にも実装の schemas/v1 にもまだ無く、この頁の例が形の唯一の出どころ。search ・ semantic-* は未実装）。term は glossary の版付きの id（…:glossary:v1#permission）で一致を決め、別の版の term は別の語として探す（I25）。渡した文脈のうち文脈の地図に無いもの（other）は探さず、not_searched に理由と一緒に返す。items が0件でも、scope と not_searched で「どこを探して、どこを探していないか」が分かる（I42）</p>
<div class="ex"><div class="pane"><b>④ AI が整形した答え → ⑤ validate（検査②。答えを渡す入力と provenance-reproduces は未実装）</b><pre>{j(ans["answer"])}</pre></div><div class="pane on"><b>validate の結果（保存しない）</b><pre>{j(ans["results"])}</pre></div></div>
<p class="meta">select は「出す名前 → JMESPath の式」の対応で、values のキーはその名前（呼ぶ側が決める）。出どころの select はその名前を指す。JMESPath は選ぶ（where）と取り出す（select）にだけ使い、それ以外の返す形は基盤が決める。読んだ版がいまの正本と違えば、validate は violated を返し、AI が読み直す（未決：インスタンスに版の欄は無く、読んだ時点の中身を指せるのは ETag（保存しない）だけ。version ・ index_of に ETag とコミットのどちらを入れるかは決まっていない）。写した値でない要約 ・ 組み合わせを答えに入れたときだけ、AI の rule（根拠の判定）が必要になる。この判定は問う者の AI ではなく、基盤がコーディングエージェントごとのアダプター（coding-agents）で、判定ごとに新しいセッションの AI に判定させる</p>
<h2>4　呼ぶ側</h2>
{table(['呼ぶ側', '種類', '使うコマンド'], [
 ['コーディングエージェント（Claude Code ・ Codex ・ Copilot ・ Kiro）と、それを動かす harness', 'AI が伴う', 'semantic-* で procedure を受け取り、機械的なコマンドを呼ぶ'],
 ['CI', 'AI が伴わない', '機械的なコマンド（search ・ validate など）を直接呼ぶ'],
])}
<h2>5　AI をどこで動かすか</h2>
{table(['形', 'セッション', 'キャッシュ', '使う場面'], [
 ['A　いまのセッションで行う（基本）', '増えない', 'いまの会話のまま効く', '問う者が AI のとき'],
 ['B　文脈を引き継ぎ、残さない fork', '残らない', 'Claude Code の fork：効く（原典「its first request reads the parent\'s cache」）。Codex の fork --ephemeral：効く（実測 64%、再開と同じ）', 'いまの会話から切り離して1回だけ整形させたいとき'],
])}
<p class="meta">fork は呼ぶ側のコーディングエージェントの機能で、基盤は関わらない（基盤は作業させる AI を起動しない。判定させる AI は基盤が coding-agents で起動する）。子のセッションの4つの型と、ツールごとの手段は「子のセッションの型」の頁が持つ</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
