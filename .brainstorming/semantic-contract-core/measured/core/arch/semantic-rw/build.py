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
AB = 'urn:semantic-agent-schema:base:abstract:v1'
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
exs = ''.join([
 step('semantic-read：欄 owner を読む', '状態を変えない。事後状態は事前状態と同じで、出力（値 ・ x-prompt ・ 版）だけが増える',
  {"手順": "read の means に従って読む", "入力のスキーマ": "procedure#/$defs/read-request（id ・ JSON Pointer）", "出力のスキーマ": "procedure#/$defs/read-result（値 ・ x-prompt ・ 版）"},
  {"kind": CS, "id": "r-12", "pointer": "/owner"}, REC, REC,
  {"値": "team-a", "x-prompt": {"read": {"means": "この記録の決定に責任を持つ人"}, "write": [{"id": "owner-is-person", "rule": "人の名前を書く。チーム名は書かない", "by": "ai"}]},
   "results": [{"rule": "owner-is-person", "value": "violated"}], "版": 3}),
 step('semantic-write → patch：owner を直す', 'test でいまの値を確かめてから replace する。版の食い違いは patch が断る。状態は violated → unverified（AI の rule は needs で返る）',
  {"手順": "write の rule に従って書く。test でいまの値を確かめてから replace", "入力のスキーマ": "JSON Patch（test ・ add ・ replace ・ remove。対象の欄だけ）", "出力のスキーマ": "record@1 と change"},
  [{"op": "test", "path": "/owner", "value": "team-a"}, {"op": "replace", "path": "/owner", "value": "yamada"}],
  dict(REC, 状態="violated"), dict(REC, owner="yamada", 状態="unverified（owner-is-person の判定待ち）"),
  {"applied": True, "stop": True, "needs": [{"rule": "owner-is-person", "why": "AI の判定を親が依頼する（semantic-judge）"}]}),
 step('semantic-write → patch：x-prompt の rule を直す（Concrete への書き込み）', 'Concrete は Abstract のインスタンスなので、同じ semantic-write で x-prompt の rule を足す ・ 変える ・ 取り除く。rule は添字ではなく test で id を確かめてから指す。この欄の前の AI の判定は効かなくなる（U2）',
  {"手順": "rule を id で指す。test で id を確かめる", "入力のスキーマ": "JSON Patch（x-prompt の欄だけ）", "出力のスキーマ": "Abstract に合う Concrete と change"},
  [{"op": "test", "path": "/properties/owner/x-prompt/write/0/id", "value": "owner-is-person"},
   {"op": "replace", "path": "/properties/owner/x-prompt/write/0/rule", "value": "人の名前を書く。チーム名と役職名は書かない"}],
  {"$schema": AB, "$id": CS, "owner の rule": "人の名前を書く。チーム名は書かない"},
  {"$schema": AB, "$id": CS, "owner の rule": "人の名前を書く。チーム名と役職名は書かない"},
  {"applied": True, "stop": True, "note": "r-12 の owner の AI の判定は効かなくなり、unverified に戻る"}),
])
body = f'''<main>
<h1>Semantic Read と Semantic Write</h1>
<p class="meta">利用者「Semantic Read と Semantic Write というのはx-promptを読み、aiが読み書きを人間の都度の指示がなくてもループ的にできるようにしたもの」「手順とその操作における入力と出力をschemaの形式で成型するのがAIができたとしたら」。決まった内容は ACDR 0201（正本：model.json ・ ボード）。例の名前は架空</p>
<p class="ans">semantic-read と semantic-write は、AI が行う操作のセマンティックのコマンドで、procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）を返す。AI は手順に従って入力（JSON Patch）だけを生成し、適用と検査は機械的なコマンド（get ・ patch）がする。検査は入力と出力の2回で、出力が外れても保存して violated を返す</p>
{fig}
<h2>1　対象</h2>
{table(['対象', 'semantic-read', 'semantic-write'], [
 ['欄（いちばん細かい単位）', 'その欄の x-prompt ・ 値 ・ 版', 'その欄の write の rule ・ 使える op'],
 ['インスタンス', '欄ごとの means と読み進め方', '欄ごとの write の rule（複数の欄を1つの Patch で書く）'],
 ['スキーマ（Concrete ・ glossary ・ published）', '$ref を解いた形と欄ごとの means', 'x-prompt と拡張の書き方（Abstract に照らす）'],
 ['ドキュメント（Markdown ・ ソース）', '読み方の案内だけ。本文は AI が読み、基盤は中身を解釈しない', '──（AI が直し、record で変更の記録を作る）'],
 ['文脈（settings）', '──', '探す範囲 ・ validator の書き方'],
])}
<p class="meta">まだ無いものを作るときは semantic-create（雛形と書く順を返す）→ create。ほかのセマンティックのコマンドは semantic-search ・ semantic-judge ・ semantic-import ・ semantic-migrate（7つ）</p>
<h2>2　事前状態 → 入力 → 事後状態</h2>
{exs}
<h2>3　検査</h2>
{table(['検査', '外れたもの', '扱い'], [
 ['① 入力（適用する前）', 'op が1つでも当たらない', '断る。何も適用しない（RFC 6902「application of the entire patch document SHALL NOT be deemed successful」）'],
 ['', '読んだ版といまの版が違う', '断る（同じインスタンスを書くほかの任せる先と重ならないように）'],
 ['', 'id ・ $schema ・ 文書全体（path が空）を変える', '断る。種類を変えるのは作り直しで、不要になったものは superseded にして残す'],
 ['', 'move ・ copy', '断る（使える op は test ・ add ・ replace ・ remove。欄を取り除くのは remove）'],
 ['② 出力（適用した後）', '形 ・ 機械の rule に違反する値', '断らずに保存し、violated を返す（Q8）'],
 ['', '人が決める欄を AI が変えた', '断らずに保存し、rule human-entry-only の結果を返す。needs に出す（Q7 ・ I21）'],
 ['', 'AI の rule', 'unverified で返し、needs に出す。親が semantic-judge で依頼し、validate が答えをまとめる'],
])}
<h2>4　止める条件</h2>
{table(['結果', '繰り返しの中での扱い'], [
 ['機械の rule の violated', '0 になるまで AI が直す'],
 ['error（validator が答えない など）', '止めて needs で返す。書く操作は再試行しない（Q3）'],
 ['AI の rule（by: ai）', '止めて needs で返す。機械の rule が0になってから親が semantic-judge で依頼する'],
 ['人が決める欄', '止めて needs で返す。人の入口で直す'],
 ['前の回と同じ姿になった', '止めて needs で返す（同じ状態のくり返し）'],
 ['オーケストレーターのセッション（親がいない）', 'needs の代わりに、判断待ちのまま止める'],
])}
<h2>5　変更の記録</h2>
{table(['もの', '置き場所', 'いつ'], [
 ['変更の記録（意図 ・ 目的の参照 ・ 変えた欄）', '手元の DB', '止めたあとに record が意図1つにつき1件作る。semantic-write は意図を受け取らない'],
 ['意図の判定（I33 ・ 人の承認）', '手元の DB', '変更の中で判定する'],
 ['まとめ（change-summary）', 'コミットする', 'コミットの前に、基盤が前回のコミットからの差し引きと例外を機械的に作る（AI は書かない）'],
 ['rule の判定', 'コミットする', 'ほかの人の手元と CI が verified を決める材料'],
])}
<p class="meta">名前は英語と日本語の対で持つ（Q5）：Semantic Read（セマンティックリード）・ Semantic Write（セマンティックライト）。x-prompt の read ・ write のキーと紛れないよう、単独の Read ・ Write とは呼ばない</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
