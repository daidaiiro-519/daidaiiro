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
proc_out = {"steps": ["問いを条件のスキーマ（query）の形に整形する", "search を呼ぶ", "答えのスキーマの形に整形し、出どころを付ける", "validate を呼ぶ"],
            "query_schema": "urn:semantic-agent-schema:core:query:v1", "commands": ["search", "get", "validate"],
            "rules": [{"id": "where-is-jmespath", "rule": "where はインスタンス1件に評価する JMESPath の式。真のものを選ぶ"}],
            "candidates": {"kind": [CS], "term": ["urn:semantic-agent-schema:sample:glossary:v1#permission"]}}
cond = {"kind": [CS], "where": "contains(subject, 'AI') && decision != null", "select": {"owner": "owner", "decision": "decision"}, "contexts": ["sample"]}
res = {"query": cond, "items": [{"id": "r-12", "kind": CS, "version": "<読んだ版>", "values": {"owner": "yamada", "decision": "許可は各ツールの設定に任せる"}}],
       "scope": {"contexts": ["sample"], "index_of": "<正本の版>"}}
ans = {"answer": {"$schema": AS, "items": [{"record": "r-12", "owner": "yamada",
        "provenance": {"id": "r-12", "kind": CS, "version": "<読んだ版>", "select": "owner"}}]},
       "results": [{"rule": "provenance-reproduces", "by": "machine", "value": "verified", "why": "その版で式を評価し直すと yamada"}]}
body = f'''<main>
<h1>Semantic Search と用語</h1>
<p class="meta">利用者の言葉「処理自体は多分全部機械的…整形する部分がaiが行う部分で、それを整形する部分の契約としてjsonスキーマがある」「コマンドとしてはセマンティック系もあるでしょ？ただそれが返すのはそのセマンティックをするための手順を返す」「jmethを使わないというのはそれさっきいったユースケースをクリアできない」「ハーネスをaiではない方にカテゴライズするのは意味不明」と、Codex ・ Claude Code の fork とキャッシュの実測を合わせて整理した。例の名前は架空。入出力の形は protocol（Q4）が決まるまで案</p>
<p class="ans">Semantic Search は、自然な言葉の問いに答えのスキーマで答える使う場面（U28）。semantic-search のコマンドが手順を返し、AI は問いを JMESPath の条件に、結果を答えに整形するだけ。探す ・ 確かめるは機械的なコマンドで、出どころに式と版を残すので、答えは何度でも同じ結果で確かめられる</p>
{fig}
<h2>1　コマンドと用語</h2>
{table(['コマンド', '種類', '返すもの'], [
 ['semantic-search', 'セマンティック（手順を返す）', '条件のスキーマ ・ 使ってよいコマンド ・ 答えのスキーマ ・ 書き方の rule ・ kind と term の候補'],
 ['semantic-read', 'セマンティック（手順を返す）', 'その欄の x-prompt（read と write の rule）と読み進め方'],
 ['semantic-write', 'セマンティック（手順を返す）', 'その欄の書き方の rule ・ 使える Patch の op ・ 書いたあとに呼ぶ validate'],
 ['search', '機械的', 'インスタンスごとに JMESPath の where を評価し、id ・ 版 ・ 式 ・ select の値 ・ 探した範囲を返す'],
 ['get', '機械的', '欄の x-prompt ・ 値 ・ 版'],
 ['patch', '機械的', 'JSON Patch（test ・ add ・ replace ・ remove）を適用し、検証の結果と needs'],
 ['validate', '機械的', '渡した値を保存せずに検証する。出どころの式をその版で評価し直して一致を確かめる'],
])}
<p class="meta">セマンティック系のコマンドは処理をせず、手順を返す。手順は契約から決まるので、返すもの自体は決定論的。AI はその手順に従って機械的なコマンドを呼ぶ</p>
<h2>2　送る JSON と返る JSON</h2>
<div class="ex"><div class="pane"><b>① semantic-search に問う</b><pre>{j(proc)}</pre></div><div class="pane on"><b>手順が返る</b><pre>{j(proc_out)}</pre></div></div>
<div class="ex"><div class="pane"><b>② AI が作った条件 → ③ search</b><pre>{j(cond)}</pre></div><div class="pane on"><b>search の結果</b><pre>{j(res)}</pre></div></div>
<div class="ex"><div class="pane"><b>④ AI が整形した答え → ⑤ validate</b><pre>{j(ans["answer"])}</pre></div><div class="pane on"><b>validate の結果（保存しない）</b><pre>{j(ans["results"])}</pre></div></div>
<p class="meta">JMESPath は選ぶ（where）と取り出す（select）にだけ使い、返す形は基盤が決める。JMESPath に位置が付かない弱点は、出どころに id ・ 版 ・ 式を残して評価し直すことで補う。写した値でない要約 ・ 組み合わせを答えに入れたときだけ、AI の rule（根拠の判定）が必要になる</p>
<h2>3　呼ぶ側</h2>
{table(['呼ぶ側', '種類', '使うコマンド'], [
 ['AI ツール（Claude Code ・ Codex ・ Copilot ・ Kiro）と、それを動かす harness', 'AI が伴う', 'semantic-* で手順を受け取り、機械的なコマンドを呼ぶ'],
 ['CI', 'AI が伴わない', '機械的なコマンド（search ・ validate など）を直接呼ぶ'],
])}
<h2>4　AI をどこで動かすか</h2>
{table(['形', 'セッション', 'キャッシュ', '使う場面'], [
 ['A　いまのセッションで行う（基本）', '増えない', 'いまの会話のまま効く', '問う者が AI のとき'],
 ['B　文脈を引き継ぎ、残さない fork', '残らない', 'Claude Code の fork：効く（原典「its first request reads the parent\'s cache」）。Codex の fork --ephemeral：効く（実測 64%、再開と同じ）', 'いまの会話から切り離して1回だけ整形させたいとき'],
 ['採らない：保存する fork ・ 毎回新しいセッション', '残る ・ 増える', 'Codex の保存する fork は 29%（実測）。新しいセッションは親のキャッシュを読まない', '──'],
])}
<p class="meta">Copilot と Kiro は、残さない fork をコマンドだけで作れるか ・ キャッシュが効くかが原文に無い（不明）。基盤は fork を前提にせず、AI ツールのアダプターが「残さない fork ができるか」「そのときキャッシュが効くか（効く ・ 効かない ・ 不明）」を返し、できなければ A にする</p>
<h2>5　書き換える前提</h2>
{table(['前提', '変更前', '変更後'], [
 ['使う場面', 'U0〜U27', 'U28 Semantic Search を足す'],
 ['値で探すこと（D2 で外した「宣言的な式の言語」）', '持たない', 'search の条件に JMESPath の where ・ select を持つ。返す形は基盤が決め、形を変える用途（projection ・ multiselect）には使わず、答えのスキーマへの整形は AI が行う'],
 ['駆動する側のポート', '探して取り出す ・ 案内する ／ インスタンスを書く ／ 検証する', 'セマンティック系：semantic-search ・ semantic-read ・ semantic-write（手順を返す）。機械的：search ・ get ・ patch ・ validate'],
 ['出どころ', '無い', 'Abstract の $defs に id ・ kind ・ 版 ・ 式（select）を持つ値オブジェクトを足し、validate が評価し直す機械の rule を持つ'],
 ['手順', '無い', 'Abstract のファイル「探す条件」とその x-prompt。semantic-search が返す'],
 ['AI ツールのアダプター', 'セッションを問う（再開できるか ・ 有効時間 ・ 履歴の割合）', '加えて「残さない fork ができるか」と「そのときキャッシュが効くか」を返す'],
])}
<h2>6　確かめてほしいこと</h2>
{table(['確かめてほしいこと', '私の見解'], [
 ['semantic-* のコマンドは手順を返し、機械的なコマンドが処理する', 'そうする（あなたのイメージ）'],
 ['search の条件に JMESPath（where ・ select）を使い、出どころに式と版を残して validate が評価し直す', 'そうする'],
 ['harness は AI の側、AI でない呼ぶ側は CI', 'そうする'],
 ['AI は基本いまのセッションで動かし、切り離したいときは残さない fork（できる AI ツールだけ）', 'そうする'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
