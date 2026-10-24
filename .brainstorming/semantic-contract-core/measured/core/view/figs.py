"""図の正本（class で書き、design-svg resolve を通す）。どの箱も、スキーマ ・ データのファイル ・ DB の表 のどれかの札を持つ。"""
import subprocess, os
H = os.path.dirname(os.path.abspath(__file__))
DS = '/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/bin/design-svg'
KIND = {'schema': ('kind-1', 'スキーマ'), 'data': ('kind-2', 'データ'), 'db': ('kind-3', 'DB の表')}

def box(x, y, w, title, lines, kind):
    k, tag = KIND[kind]; h = 40 + len(lines) * 18 + 8
    s = (f'<rect class="box {k}" x="{x}" y="{y}" width="{w}" height="{h}"/>'
         f'<rect class="badge {k}" x="{x + w - 76}" y="{y + 9}" width="66" height="18"/>'
         f'<text class="label small {k}" x="{x + w - 43}" y="{y + 22}" font-weight="700" text-anchor="middle">{tag}</text>'
         f'<text class="title {k}" x="{x + 12}" y="{y + 23}">{title}</text>'
         f'<line class="grid" x1="{x}" y1="{y + 34}" x2="{x + w}" y2="{y + 34}"/>')
    for i, (cls, t) in enumerate(lines): s += f'<text class="{cls} small" x="{x + 12}" y="{y + 52 + i * 18}">{t}</text>'
    return s
L = lambda t: ('label', t); N = lambda t: ('note', t)
def legend(y):
    s = ''; x = 10
    for kind in ('schema', 'data', 'db'):
        k, tag = KIND[kind]
        s += f'<rect class="badge {k}" x="{x}" y="{y}" width="66" height="18"/><text class="label small {k}" x="{x + 33}" y="{y + 13}" font-weight="700" text-anchor="middle">{tag}</text>'
        x += 76
    expl = 'スキーマ：形を決めるファイル　データ：スキーマに従うファイル（確定して共有）　DB の表：手元の SQLite（確定しない）'
    return s + f'<text class="note small" x="{x + 6}" y="{y + 13}">{expl}</text>'

def er():
    P = ['<svg viewBox="0 0 1000 720" style="min-width:900px" role="img" aria-label="基盤のデータの関係（ER 図）">',
         '<text class="title" x="10" y="24">基盤のデータの関係（ER 図）</text>', legend(34),
         '<rect class="area" x="10" y="62" width="980" height="370"/><text class="note small" x="20" y="78">リポジトリのファイル（変更の仕組みで確定して共有する）</text>',
         '<rect class="area" x="10" y="450" width="980" height="214"/><text class="note small" x="20" y="466">手元の DB（SQLite。確定しない）</text>',
         box(30, 90, 240, 'Abstract（本体）', [L('PK $id'), N('x-extensions（9つの x-）'), N('x-delegates（judge）'), N('$defs（欄 ・ rule ・ prompt …）')], 'schema'),
         box(390, 90, 240, 'Concrete（文脈）', [L('PK $id'), L('FK $schema → Abstract'), N('x-delegates（任せる先）'), N('x-extensions（足した x-）'), N('properties（欄 ・ x-prompt）')], 'schema'),
         box(750, 90, 220, 'インスタンス', [L('PK id ・ kind'), L('FK $schema → Concrete'), N('欄の値')], 'data'),
         box(30, 262, 240, '変更の記録', [L('PK id（ULID）'), N('purpose_ref（外の目的）'), N('intent（意図1つ）'), N('writer_kind'), N('items（対象 ・ 欄）'), L('FK replaces → 変更の記録')], 'data'),
         box(390, 262, 270, '判定の記録', [L('PK key（対象 × rule）'), L('FK target → インスタンス か 変更の記録'), L('FK rule → Concrete か本体の rule'), N('result（5つの値）'), N('judged_values')], 'data'),
         box(750, 342, 220, '設定', [L('PK id（user ・ project ・ local）'), N('session の値')], 'data'),
         box(30, 484, 240, 'sessions', [L('PK tool ・ id'), N('purpose_ref'), L('FK parent → sessions'), N('role'), N('state ・ last_used_at')], 'db'),
         box(390, 484, 270, 'session_changes', [L('PK tool ・ id ・ change_id'), L('FK tool ・ id → sessions'), N('change_id → 変更の記録（論理）')], 'db'),
         '<path class="link" d="M270,126 H390"/><text class="label small" x="276" y="120">1</text><text class="label small" x="370" y="120">多</text>',
         '<path class="link" d="M630,126 H750"/><text class="label small" x="636" y="120">1</text><text class="label small" x="730" y="120">多</text>',
         '<path class="link" d="M860,196 V320 H660"/><text class="label small" x="866" y="212">1</text><text class="label small" x="666" y="314">多</text>',
         '<path class="link" d="M270,352 H390"/><text class="label small" x="276" y="346">1</text><text class="label small" x="350" y="346">0..1</text><text class="note small" x="282" y="368">I33</text>',
         '<path class="link" d="M200,262 V248 H740 V172 H750"/><text class="note small" x="420" y="242">items の対象（インスタンスか、それ以外はパス）</text>',
         '<path class="link" d="M270,574 H390"/><text class="label small" x="276" y="568">1</text><text class="label small" x="370" y="568">多</text>',
         '<path class="link" d="M450,484 V444 H140 V416"/><text class="note small" x="150" y="440">change_id（ファイルの id を引く）</text>',
         '<text class="note" x="10" y="700">1 と 多：関連の数。PK は主キー、FK は外部キー。sessions の parent は同じ表を指す（オーケストレーター → 目的 → 任せる先）</text>', '</svg>']
    return '\n'.join(P)

def refs():
    def b(x, y, w, title, lines, kind): return box(x, y, w, title, [N(t) for t in lines], kind)
    P = ['<svg viewBox="0 0 1000 580" style="min-width:900px" role="img" aria-label="スキーマのファイルの参照のつながり">',
         '<text class="title" x="10" y="24">スキーマのファイルの参照のつながり</text>', legend(34),
         '<rect class="area" x="10" y="62" width="380" height="300"/><text class="note small" x="20" y="78">Semantic Abstract Schema（基盤）</text>',
         '<rect class="area" x="420" y="62" width="570" height="300"/><text class="note small" x="430" y="78">文脈（Skill ・ Agent のリポジトリ）</text>',
         b(30, 92, 330, '本体', ['Concrete の契約 ・ 9つの x- の定義', '$schema は自分自身'], 'schema'),
         b(30, 222, 330, 'ほかのファイル（形）', ['change ・ judgment ・ settings ・ session', 'view ・ catalog ・ design-tokens ・ glossary', 'annotations ・ published'], 'schema'),
         b(450, 92, 260, 'Concrete（ACDR など）', ['欄 ・ x- ・ 任せる先'], 'schema'),
         b(740, 92, 230, 'インスタンス', ['ACDR 0197'], 'data'),
         b(450, 222, 230, '記録と設定', ['変更の記録 ・ 判定の記録 ・ 設定'], 'data'),
         b(720, 222, 250, '描画と辞書', ['ビュー ・ デザインシステム ・ glossary'], 'data'),
         '<rect class="area" x="10" y="392" width="740" height="110"/><text class="note small" x="20" y="408">手元の DB（SQLite。確定しない）</text>',
         b(30, 418, 330, 'sessions', ['セッション（3種類）'], 'db'),
         b(400, 418, 330, 'session_changes（変更の DB）', ['セッション → 変更の記録の id'], 'db'),
         '<path class="link" d="M360,448 H400"/>',
         '<path class="flow" d="M195,222 V168"/><text class="label small" x="202" y="200">$schema</text>',
         '<path class="flow" d="M450,120 H362"/>', '<path class="flow" d="M740,120 H712"/>',
         '<path class="flow" d="M450,262 H362"/>',
         '<path class="flow" d="M845,288 V322 H362"/><text class="label small" x="560" y="316">$schema</text>',
         '<path class="link" d="M195,418 V332"/><text class="label small" x="202" y="374">形（session）から表を作る（SQLite のアダプター）</text>',
         '<path class="link" d="M480,158 V192 H300 V222"/><text class="label small" x="490" y="212">$ref（annotations ・ 変更のまとまり）</text>',
         '<path class="link" d="M650,158 V202 H820 V222"/><text class="label small" x="700" y="174">$ref（glossary の term）・ ビューの kind が指す</text>',
         '<text class="note" x="10" y="560">矢印：$schema（自分が何に従うか）。線：$ref（部品を借りる）と、ビューの kind が Concrete を指すつながり。処理は基盤のコードが持つ（Q6 #7）</text>', '</svg>']
    return '\n'.join(P)

for name, f in [('er', er), ('refs', refs)]:
    open(os.path.join(H, f'{name}.source.svg'), 'w').write(f())
    r = subprocess.run([DS, 'resolve', os.path.join(H, f'{name}.source.svg'), os.path.join(H, f'{name}.svg')], capture_output=True, text=True)
    print(name, [l for l in (r.stdout + r.stderr).split('\n') if '×' in l or '検査1' in l])
