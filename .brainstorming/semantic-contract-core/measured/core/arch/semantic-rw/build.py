"""Semantic Read ・ Semantic Write の頁（arch/semantic-rw/index.html）。ddd-advisor（arch/reviews/ddd-semantic-rw.*）と反証役の判定を、前提に照らして1つにしたもの。"""
import os, json, html
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.fig{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px;overflow-x:auto}.ex{display:grid;grid-template-columns:1fr 1fr;gap:10px}@media (max-width:760px){.ex{grid-template-columns:1fr}}.ex pre{white-space:pre;overflow-x:auto;padding:8px 12px}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>Semantic Read と Write</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
j = lambda o: html.escape(json.dumps(o, ensure_ascii=False, indent=2))
def ex(title, note, a, b, la='送る', lb='返る'):
    return f'<h3>{title}</h3><p class="meta">{note}</p><div class="ex"><div class="pane"><b>{la}</b><pre>{j(a)}</pre></div><div class="pane on"><b>{lb}</b><pre>{j(b)}</pre></div></div>'
CS = 'urn:semantic-agent-schema:sample:record:v1'
fig = '<div class="fig">' + open(os.path.join(H, 'loop.svg')).read() + '</div>'
exs = ''.join([
 ex('Semantic Read：欄 owner を読む', 'まだインスタンスが無いとき（作る前）は、種類とスキーマの中の位置だけで呼べる。返る x-prompt には write の rule も入る',
    {"kind": CS, "id": "r-12", "keywordLocation": "/properties/owner"},
    {"x-prompt": {"read": {"means": "この記録の決定に責任を持つ人", "when": "記録を作るとき"},
                  "write": [{"id": "owner-is-person", "rule": "人の名前を書く。チーム名は書かない", "by": "ai"}]},
     "instanceLocation": "/owner", "value": "team-a",
     "results": [{"rule": "owner-is-person", "value": "violated", "why": "チーム名が入っている"}],
     "version": "<読んだ版>"}),
 ex('Semantic Write：owner を直す', 'test で読んだときの値と版を確かめてから replace する。使える op は test ・ add ・ replace ・ remove の4つ',
    {"kind": CS, "id": "r-12", "version": "<読んだ版>",
     "patch": [{"op": "test", "path": "/owner", "value": "team-a"}, {"op": "replace", "path": "/owner", "value": "yamada"}]},
    {"applied": True, "state": "判断待ち",
     "results": [{"rule": "owner-is-person", "value": "unverified", "why": "AI の rule は判定を依頼する"}],
     "stop": True,
     "needs": [{"rule": "owner-is-person", "why": "AI の判定を親が依頼する"}]}),
 ex('Semantic Write：x-prompt 自体を直す（Concrete への Write）', 'Concrete は Abstract のインスタンスなので、同じ Semantic Write で x-prompt の rule を足す ・ 変える ・ 消す。rule は添字ではなく test で id を確かめてから指す',
    {"kind": "urn:semantic-agent-schema:core:abstract:v1", "id": CS, "version": "<読んだ版>",
     "patch": [{"op": "test", "path": "/properties/owner/x-prompt/write/0/id", "value": "owner-is-person"},
               {"op": "replace", "path": "/properties/owner/x-prompt/write/0/rule", "value": "人の名前を書く。チーム名と役職名は書かない"}]},
    {"applied": True, "results": [], "stop": True,
     "note": "x-prompt を変えた変更は、インスタンスの繰り返しの止める条件に数えない。この欄の前の AI の判定は効かなくなる（U2）"}),
])
body = f'''<main>
<h1>Semantic Read と Semantic Write</h1>
<p class="meta">利用者「Semantic Read と Semantic Write というのはx-promptを読み、aiが読み書きを人間の都度の指示がなくてもループ的にできるようにしたものだと思ってください」「x-prompt自体への更新や削除、新規作成にも使用できるなどが必要です。そのときはJsonPatchなども使用できると思います」を、ddd-advisor（arch/reviews/ddd-semantic-rw.html）と、結論を渡さない反証役に通し、前提に照らして1つにした。例の名前は架空。入出力の形は protocol（Q4、待ち）が決まるまで案</p>
<p class="ans">Concrete のスキーマが、インスタンスの各欄の読み方と書き方（x-prompt）を決めている。AI はその指示に目的と意図を加えてインスタンスを書き、中身は目的ごとに変わる。それでも、指示から外れていないかは基盤が返す。機械の rule では決定論的に外れず、AI の rule は判定の結果（判定が出るまでは unverified）として残る。Semantic Read はその指示と値を返し、Semantic Write は JSON Patch を適用して結果と「止めてよいか」を返す</p>
{fig}
<h2>1　送る JSON と返る JSON</h2>
{exs}
<h2>2　Write で断るもの ・ 断らないもの</h2>
{table(['Patch の中身', '扱い', '前提'], [
 ['op が1つでも当たらない', '断る。何も適用しない', 'RFC 6902「application of the entire patch document SHALL NOT be deemed successful」'],
 ['読んだ版といまの版が違う', '断る（ほかの書き手と重ならないように）', '同じ目的で役割の違う任せる先が同じインスタンスを書ける（I46）'],
 ['id ・ $schema ・ 文書全体（path が空）を変える', '断る', 'id は使い直さない（I11）。種類を変えるのは作り直し'],
 ['move ・ copy', '断る（使える op は test ・ add ・ replace ・ remove）', 'remove と add で書ける。人が決めた欄から move すると、人の値を消す'],
 ['rule に違反する値', '断らずに保存し、violated を返す。インスタンスは下書きになる', 'Q8「基盤は保存を断らず、検証して「取り込めない理由」を出す」'],
 ['人が決める欄を AI が変える', '断らずに保存し、rule human-entry-only の結果を返す。needs に出す', 'Q7「AI が書き写すのはよいが、値は人の入口から来たものに限る」・ I21'],
])}
<h2>3　止める条件</h2>
{table(['結果', '繰り返しの中での扱い'], [
 ['機械の rule の violated', '0 になるまで AI が直す'],
 ['error（validator が答えない など）', '止めて needs で返す。書く操作は再試行しない（Q3）'],
 ['AI の rule（by: ai）', '止めて needs で返す。判定を依頼するのは親（Validator の2段）。機械の rule が0になってから依頼する'],
 ['人が決める欄', '止めて needs で返す。人の入口で直す'],
 ['前の回と同じ姿になった', '止めて needs で返す（同じ状態のくり返し）'],
 ['オーケストレーターのセッション（親がいない）', 'needs の代わりに、判断待ちのまま止める'],
])}
<h2>4　駆動する側のポート（変更前 → 変更後）</h2>
{table(['変更前', '変更後'], [
 ['探して取り出す ・ 案内する', 'search と Semantic Read の2つに分ける'],
 ['インスタンスを書く（作成 ・ 更新 ・ 削除 ・ 取り込む）', '更新は Semantic Write（対象にインスタンスと Concrete）。作成（ファクトリ）・ 削除 ・ 取り込むは別のコマンドに残す'],
 ['意味を宣言する（x-prompt の定義を含む）', 'x-prompt の作成 ・ 変更 ・ 削除は Semantic Write へ。Concrete を作る ・ 照らす ・ 版を出すは残す'],
 ['インスタンスを書くが意図を受け取る', 'Write は意図を受け取らない。止めたあと「変更を記録する」が意図1つにつき1件を作る（I32 で記録は書き換えない）'],
])}
<p class="meta">名前は英語と日本語の対で持つ（Q5）：Semantic Read（セマンティックリード）・ Semantic Write（セマンティックライト）。x-prompt の read ・ write のキーと紛れないよう、単独の Read ・ Write とは呼ばない。位置の欄は JSON Schema の出力と同じ keywordLocation（スキーマの中）と instanceLocation（インスタンスの中）にする</p>
<h2>5　search で欄の値を探すか（決めてほしいこと）</h2>
{table(['案', '中身', '前提との関係'], [
 ['A　値では探さない（いまの5つの単位のまま）', '値の条件は Concrete の rule にして、search は状態と判定の結果で探す', 'ボードで外した D2「宣言的な式の言語 …もともとの範囲に無い追加だった」と、Q9「意味で探すは、語 ・ 参照 ・ rule の結果で探すので中核」に合う'],
 ['B　JMESPath を選ぶ条件にだけ使う', 'where の式をインスタンス1件ずつに評価し、真になったものを返す。返す形は基盤が決め、形を変える用途（projection ・ multiselect）には使わない', '値で探したいという要望に応える。D2 を戻すことになる'],
])}
<p class="meta">JSON Schema で値を探す案はやめた（当たった位置が出力から取れない ・ x- が黙って無視される ・ 保守しにくい）</p>
<h2>6　確かめてほしいこと</h2>
{table(['確かめてほしいこと', '私の見解'], [
 ['Semantic Read ・ Semantic Write を使う場面の名前にし、ポートを上の表のとおり分け直す', 'そうする'],
 ['繰り返すのは AI で、基盤は1回ずつ答え、「止めてよいか」と needs を返す', 'そうする。ボード Q8 で「直すループを回す」は基盤の外'],
 ['Write の断るもの ・ 断らないもの（2節）', 'そうする'],
 ['search で欄の値を探すか（5節の A か B）', 'あなたが決めること。範囲を広げるかどうかの判断なので'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
