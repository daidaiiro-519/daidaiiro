"""子のセッションの3つの型と、AI ツールごとの能力（キャッシュが効くか）の頁。実測は 2026-10-25（各1回）。"""
import os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '.y{color:var(--add-ink);background:var(--add);font-weight:700;padding:0 4px;border-radius:4px}.n{color:var(--bad);background:var(--bad-bg);font-weight:700;padding:0 4px;border-radius:4px}.u{color:var(--hot);background:var(--hot-bg);font-weight:700;padding:0 4px;border-radius:4px}\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>子のセッションの型</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
Y = lambda t: f'<span class="y">効く</span> {t}'
N = lambda t: f'<span class="n">効かない</span> {t}'
U = lambda t: f'<span class="u">不明</span> {t}'
body = f'''<main>
<h1>子のセッションの型</h1>
<p class="meta">利用者「forkでいいものやsessionとして別の文脈としてその目的と意図の間は継続している必要がある子セッションの区別も契約として持っていた方がいい」「ClaudeCodeとCodexとCopilotとKiroで抽象できる契約として整理できること、キャッシュを利かすことができること」「A,Bも両方あるイメージ」を受けた整理。実測は 2026-10-25、約4万トークンの資料を読ませた親から各1回</p>
<p class="ans">子は3つの型を持つ。継続する子 A は4つの AI ツールすべてで再開のキャッシュが効く。分岐でキャッシュが効くのは Claude Code と Codex。継続する子 B を作るときに親のキャッシュが効くのは Claude Code だけ。契約は型ごとに AI ツールの能力を持ち、できない ・ 効かないツールでは代わりの型にする</p>
<h2>1　3つの型</h2>
{table(['型', '親の文脈', '続くか', '向いている仕事', '寿命'], [
 ['分岐', '持って始まる', '1回で終わる。残さない', '反証、Semantic Search の整形', '1回の呼び出し'],
 ['継続する子 A（既定）', '持たずに始まる（仕事の指示だけ）', '続く。役割ごとに再開する', '指示だけで進められる長い仕事', '役割の仕事が終わるか、目的が閉じるまで'],
 ['継続する子 B', '持って始まる', '続く。役割ごとに再開する', '会話の意図を知らないと外れる長い仕事', '同上'],
])}
<h2>2　AI ツールごとの能力（キャッシュが効くか）</h2>
{table(['型', 'Claude Code 2.1.305', 'Codex 0.164.0', 'Copilot 1.0.103', 'Kiro 2.21.1'], [
 ['分岐（親の文脈を引き継ぐ）', Y('96%（実測）<br>--resume ID --fork-session --no-session-persistence'), Y('64%（実測。親の再開と同じ）<br>exec fork --ephemeral ID'), U('対話の中の /fork だけ。コマンドでは作れない'), N('クレジットが親と同じ（推測）<br>v3 の ACP の session/fork')],
 ['継続する子 A：作る', '新しいセッション（親のキャッシュは無い）', '同左', '同左', '同左'],
 ['継続する子 A ・ B：2回目からの再開', Y('100%（実測）<br>--resume ID'), Y('64%（実測）<br>exec resume ID'), Y('96%（実測）<br>--resume ID'), Y('クレジットが約半分（推測）<br>--resume-id ID')],
 ['継続する子 B：作る（親の文脈を持たせる）', Y('87%（実測）<br>--resume ID --fork-session'), N('29%（実測。共通の前置きの分だけ）<br>exec fork ID'), U('対話の中の /fork だけ'), N('分岐と同じ（推測）')],
])}
<p class="meta">Copilot Free はモデルが自動選択（auto）で、呼ぶたびにモデルが変わることがある。キャッシュはモデルをまたいで使えない（Copilot の文書「A different model can't reuse another model's cache」）。今回の再開は同じモデル（gpt-6-luna）で測れた。Kiro はトークン数を返さないので、クレジットからの推測</p>
<h2>3　ツールでできない ・ 効かないときの代わり</h2>
{table(['型', '代わり'], [
 ['分岐', 'いまのセッションの中で行う（Copilot ・ Kiro）'],
 ['継続する子 B', '親の文脈の分を1回払って作る（Codex ・ Kiro）。それが重いなら A にし、親が意図を指示に書いて渡す'],
])}
<h2>4　契約に持つもの</h2>
{table(['持つもの', '中身'], [
 ['セッションの種類', 'いまの3つ（オーケストレーター ・ 目的 ・ 任せる先）に、任せる先の型として 分岐 ・ 継続 A ・ 継続 B を足す。分岐は残さないので、セッションの集約に登録しない（判定させる AI と同じ）'],
 ['AI ツールのアダプターが返す能力', '型ごとに「コマンドで作れるか（できる ・ できない）」と「親のキャッシュが効くか ・ 再開でキャッシュが効くか（効く ・ 効かない ・ 不明）」'],
 ['選び方', '使う場面が型を求め、ツールができない ・ 効かないときは3節の代わりにする'],
])}
<h2>5　確かめてほしいこと</h2>
{table(['確かめてほしいこと', '私の見解'], [
 ['子の3つの型（分岐 ・ 継続 A ・ 継続 B）を契約に持つ', '持つ。既定は A'],
 ['AI ツールのアダプターが、型ごとに「作れるか」と「キャッシュが効くか」を返す', 'そうする'],
 ['Copilot の分岐 ・ B は、対話の中の /fork だけなので「コマンドでは作れない」とする', 'そうする。必要なら対話で /fork して測る'],
])}
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
