"""Semantic Read ・ Semantic Write の頁（arch/semantic-rw/index.html）。ddd-advisor（arch/reviews/ddd-semantic-rw.*）と反証役の判定を、前提に照らして1つにしたもの。"""
import os, json, html
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.fig{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px;overflow-x:auto}.ex{display:grid;grid-template-columns:1fr 1fr;gap:10px}@media (max-width:760px){.ex{grid-template-columns:1fr}}.ex pre{white-space:pre;overflow-x:auto;padding:8px 12px}.st3{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1.15fr) minmax(0,1fr);gap:10px}@media (max-width:860px){.st3{grid-template-columns:1fr}}.st3 .pane{min-width:0}.st3 pre{white-space:pre-wrap;overflow-wrap:anywhere;padding:8px 10px;font-size:12px}.st3 .sa{background:var(--hot-bg);color:var(--hot);font-weight:700;white-space:pre-wrap}.st3 .sd{opacity:.5;text-decoration:line-through;white-space:pre-wrap}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>Semantic Read と Write</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
j = lambda o: html.escape(json.dumps(o, ensure_ascii=False, indent=2))
def ex(title, note, a, b, la='送る', lb='返る'):
    return f'<h3>{title}</h3><p class="meta">{note}</p><div class="ex"><div class="pane"><b>{la}</b><pre>{j(a)}</pre></div><div class="pane on"><b>{lb}</b><pre>{j(b)}</pre></div></div>'
CS = 'urn:semantic-agent-schema:sample:record:v1'
AB = 'urn:semantic-agent-schema:base:semantic-abstract-schema:v1'
fig = '<div class="fig">' + open(os.path.join(H, 'loop.svg')).read() + '</div>'
def lines(o): return json.dumps(o, ensure_ascii=False, indent=2).split('\n')
def state(pre, post):
    a, b = lines(pre), lines(post); sa, sb = set(x.rstrip(',') for x in a), set(x.rstrip(',') for x in b)
    l = ''.join((html.escape(x) if x.rstrip(',') in sb else f'<span class="sd">{html.escape(x)}</span>') + '\n' for x in a)
    r = ''.join((html.escape(x) if x.rstrip(',') in sa else f'<span class="sa">{html.escape(x)}</span>') + '\n' for x in b)
    return l, r
def step(title, note, proc, inp, pre, post, out=None):
    l, r = state(pre, post)
    o = f'<b>出力</b><pre>{j(out)}</pre>' if out is not None else ''
    return (f'<h3>{title}</h3><p class="meta">{note}</p><div class="st3">'
            f'<div class="pane"><b>事前状態</b><pre>{l}</pre></div>'
            f'<div class="pane on"><b>procedure</b><pre>{j(proc)}</pre><b>AI が生成した入力</b><pre>{j(inp)}</pre></div>'
            f'<div class="pane"><b>事後状態</b><pre>{r}</pre>{o}</div></div>')
REC = {"$schema": CS, "id": "r-12", "subject": "AI の許可", "owner": "team-a"}
PATH = 'records/r-12.json'
RULE = {"id": "owner-is-person", "rule": "人の名前を書く。チーム名は書かない", "why": "チーム名では、決定に責任を持つ人に問い合わせられない", "test": "owner が人の名前で、チーム名でないかを見る", "by": "ai", "agent": "reviewer"}
WAIT = {"name": "awaiting-judgment", "label": "判断待ち"}
exs = ''.join([
 step('semantic-read → get：欄 owner を読む', '状態を変えない。事後状態は事前状態と同じで、出力（value ・ etag）だけが増える。x-prompt は procedure が手順として渡す',
  {"手順": "欄の title ・ description と x-prompt の read に従って読む。書くときのために etag を控える", "title": "責任者", "description": "この記録の決定に責任を持つ人", "x-prompt": {"write": [RULE]}, "入力のスキーマ": "procedure#/$defs/read-request（パス ・ JSON Pointer）", "出力のスキーマ": "get の出力（data.path ・ pointer ・ value ・ etag）"},
  {"path": PATH, "pointer": "/owner"}, REC, REC,
  {"ok": True, "findings": [], "data": {"path": PATH, "pointer": "/owner", "value": "team-a", "etag": "3f2a9c0d81b7e4a65c12"}}),
 step('semantic-write → patch：owner を直す', 'get で控えた etag を --if-match で渡す。読んだあとに中身が変わっていれば（ETag が違えば）patch が断る。状態は violated → 判断待ち（AI の rule owner-is-person は unverified。判定は基盤が coding-agents で起動する AI がする）',
  {"手順": "write の rule に従って書く。get で控えた etag を --if-match で渡す", "入力のスキーマ": "JSON Patch（add ・ replace ・ remove。対象の欄だけ）", "出力のスキーマ": "record と patch の出力（ok ・ findings ・ data.saved ・ results ・ state ・ etag ・ unfilled）"},
  {"--patch": [{"op": "replace", "path": "/owner", "value": "yamada"}], "--if-match": "3f2a9c0d81b7e4a65c12"},
  dict(REC, 状態="violated"), dict(REC, owner="yamada", 状態="判断待ち（owner-is-person の判定待ち）"),
  {"ok": True, "findings": [], "data": {"saved": True, "path": PATH, "results": [{"rule": "owner-is-person", "by": "ai", "pointer": "/owner", "result": "unverified", "reason": "AI の判定を依頼する（任せる先 reviewer）"}], "state": WAIT, "etag": "（書いたあとの中身から計算した新しい値）", "unfilled": []}}),
 step('semantic-write → patch：x-prompt の rule を直す（Concrete への書き込み）', 'Concrete は Abstract のインスタンスなので、同じ semantic-write で x-prompt の rule を足す ・ 変える ・ 取り除く。rule は添字で指し、get で控えた etag を --if-match で渡す（読んだあとに rule の並びが変わっていれば patch が断る）。この欄の前の AI の判定は効かなくなり、r-12 の owner は unverified に戻る（U2）',
  {"手順": "get で rule の並びと etag を読み、rule を添字で指す", "入力のスキーマ": "JSON Patch（x-prompt の欄だけ。add ・ replace ・ remove）", "出力のスキーマ": "Abstract に合う Concrete と patch の出力"},
  {"--patch": [{"op": "replace", "path": "/properties/owner/x-prompt/write/0/rule", "value": "人の名前を書く。チーム名と役職名は書かない"}], "--if-match": "9d04be71a3c25f8e0b6d"},
  {"$schema": AB, "$id": CS, "owner の rule": "人の名前を書く。チーム名は書かない"},
  {"$schema": AB, "$id": CS, "owner の rule": "人の名前を書く。チーム名と役職名は書かない"},
  {"ok": True, "findings": [], "data": {"saved": True, "state": WAIT, "etag": "（新しい値）", "results": ["…（rule ごとの結果）"]}}),
])
body = f'''<main>
<h1>Semantic Read と Semantic Write</h1>
<p class="meta">利用者「Semantic Read と Semantic Write というのはx-promptを読み、aiが読み書きを人間の都度の指示がなくてもループ的にできるようにしたもの」「手順とその操作における入力と出力をschemaの形式で成型するのがAIができたとしたら」。決まった内容は ACDR 0201（正本：model.json ・ ボード。ただし model.json の validate のコマンドの行は未更新で、Q8「外れても保存し violated で返す」と「semantic-judge の答えを validate がまとめる」が古いまま残っている。aggregates のインスタンスの commands の「変える（JSON Patch：test ・ add ・ replace ・ remove）」も古く、今は test を使わず add ・ replace ・ remove だけを受け付ける。この頁の「新しく持ち込んだ機械の違反は断る」「判定させる AI は基盤が coding-agents で起動する」が今の決定）。例の名前は架空</p>
<p class="ans">semantic-read と semantic-write は、AI が行う操作のセマンティックのコマンドで、procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）を返す。AI は手順に従って入力（JSON Patch）だけを生成し、適用と検査は機械的なコマンド（get ・ patch）がする。検査は入力と出力の2回で、出力に新しく持ち込んだ機械の rule の違反があれば書き込みを断り（data.saved:false）、前からある違反と AI ・ 人の rule は断らずに状態として返す</p>
<p class="meta">実装との対応：get ・ patch は実装にあり、出力は ok ・ findings ・ data の形（例の出力はその形）。semantic-read ・ semantic-write（procedure を返す）と needs はまだ実装に無い。楽観ロックは ETag（get の data.etag を patch --if-match で渡す）</p>
{fig}
<h2>1　対象</h2>
{table(['対象', 'semantic-read', 'semantic-write'], [
 ['欄（いちばん細かい単位）', 'その欄の x-prompt ・ 値 ・ ETag', 'その欄の write の rule ・ 使える op'],
 ['インスタンス', '欄ごとの title ・ description と読み進め方（x-prompt の read：shown ・ interpret ・ when）', '欄ごとの write の rule（複数の欄を1つの Patch で書く）'],
 ['スキーマ（Concrete ・ glossary ・ published）', '$ref を解いた形と欄ごとの title ・ description', 'x-prompt と拡張の書き方（Abstract に照らす）'],
 ['ドキュメント（Markdown ・ ソース）', '読み方の案内だけ。本文は AI が読み、基盤は中身を解釈しない', '──（AI が直し、record で変更の記録を作る）'],
 ['文脈（settings）', '──', '探す範囲 ・ validator の書き方'],
])}
<p class="meta">まだ無いものを作るときは semantic-create（雛形と書く順を返す）→ create。ほかのセマンティックのコマンドは semantic-search ・ semantic-judge ・ semantic-import ・ semantic-migrate（7つ。AI の rule の判定は基盤が coding-agents で起動する AI がするので、semantic-judge を残すかは未決）</p>
<h2>2　事前状態 → 入力 → 事後状態</h2>
{exs}
<h2>3　検査</h2>
{table(['検査', '外れたもの', '扱い'], [
 ['① 入力（適用する前）', 'op が1つでも当たらない', '断る。何も適用しない（RFC 6902「application of the entire patch document SHALL NOT be deemed successful」）'],
 ['', '読んだときの ETag（get の data.etag を patch --if-match で渡す）といまの ETag が違う', '断る。findings/0/result が refused（同じインスタンスを書くほかの任せる先と重ならないように）'],
 ['', 'id ・ $schema ・ 文書全体（path が空）を変える', '断る。種類を変えるのは作り直しで、不要になったものは superseded にして残す'],
 ['', 'test ・ move ・ copy', '断る（使える op は add ・ replace ・ remove だけ。欄を取り除くのは remove）'],
 ['② 出力（適用した後）', '新しく持ち込んだ、形 ・ 機械の rule の違反', '書き込みを断る（data.saved:false、exit 1。ファイルは変わらない）'],
 ['', '前からある機械の rule の違反', '断らずに保存し、状態として返す'],
 ['', '人が決める欄を AI が変えた', '断らずに保存し、rule human-entry-only を unverified で返す。状態は判断待ちで、人が judge で承認すると verified（Q7 ・ I21）'],
 ['', 'AI の rule', '断らずに unverified で返す。基盤が coding-agents のアダプターで判定ごとに新しいセッションの AI を起動して判定させ、答えを Validator の2段目に渡す（ステップ4）'],
])}
<h2>4　止める条件</h2>
{table(['結果', '繰り返しの中での扱い'], [
 ['機械の rule の violated', '0 になるまで AI が直す'],
 ['error（validator が答えない など）', '止めて needs で返す。書く操作は再試行しない（Q3）'],
 ['AI の rule（by: ai）', '止めて needs で返す。判定は、機械の rule が0になってから基盤が coding-agents で起動する AI がする（ステップ4）'],
 ['人が決める欄', '止めて needs で返す。人の入口で直す'],
 ['前の回と同じ姿になった', '止めて needs で返す（同じ状態のくり返し）'],
 ['オーケストレーターのセッション（親がいない）', 'needs の代わりに、判断待ちのまま止める'],
])}
<h2>5　変更の記録</h2>
{table(['もの', '置き場所', 'いつ'], [
 ['変更の記録（意図 ・ 目的の参照 ・ 変えた欄）', '手元の DB', '止めたあとに record が意図1つにつき1件作る。semantic-write は意図を受け取らない'],
 ['意図の判定（I33 ・ 人の承認）', '手元の DB', '変更の中で判定する。判定ごとに1行足し、最新が効く（覆された判定も残る。実装はステップ4）'],
 ['まとめ（change-summary）', 'コミットする（.semantic-agent-schema/changes/<ULID>.json）', 'コミットの前に record --summary で、基盤が前回のコミットからの差し引きと例外を機械的に作る（AI は書かない）'],
 ['rule の判定', 'コミットする', 'ほかの人の手元と CI が verified を決める材料'],
])}
<p class="meta">名前は英語と日本語の対で持つ（Q5）：Semantic Read（セマンティックリード）・ Semantic Write（セマンティックライト）。x-prompt の read ・ write のキーと紛れないよう、単独の Read ・ Write とは呼ばない</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
