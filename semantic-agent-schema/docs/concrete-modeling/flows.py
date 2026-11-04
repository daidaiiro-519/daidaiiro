"""処理の流れの図の元（figs/*.source.svg）を、段の並びから組む。座標は行の高さから毎回計算する。"""
from html import escape
from pathlib import Path

OUT = Path(__file__).resolve().parent / 'figs'
LINE = 15      # 1行の高さ
PAD = 12       # 箱の上下の余白
GAP = 22       # 段の間


def build(name, title, aria, cols, rows):
    """cols: [(見出し, x, w)]。rows: (列の番号 | 'ai', [行...])。"""
    out = []
    y = 34
    out.append(f'<text class="title" x="10" y="22">{escape(title)}</text>')
    for head, x, w in cols:
        out.append(f'<text class="note small" x="{x + w / 2:g}" y="{y + 10}" text-anchor="middle">{escape(head)}</text>')
    y += 22
    boxes = []
    for col, lines in rows:
        h = PAD + LINE * len(lines)
        if col == 'ai':
            x, w, cls = 10, 400, 'box kind-1'
        else:
            _, x, w = cols[col]
            cls = 'box'
        boxes.append((col, x, y, w, h))
        out.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{w}" height="{h}"/>')
        for i, t in enumerate(lines):
            c = 'label small' if i == 0 else 'note small'
            out.append(f'<text class="{c}" x="{x + 10}" y="{y + PAD / 2 + LINE * (i + 1) - 3:g}">{escape(t)}</text>')
        y += h + GAP
    # 段どうしの矢印：前の箱の下の中央から、次の箱の上の中央へ（AI の箱は相手の列の中央を使う）
    for a, b in zip(boxes, boxes[1:]):
        ca, xa, ya, wa, ha = a
        cb, xb, yb, wb, hb = b
        ax = xa + wa / 2 if ca != 'ai' else xb + wb / 2
        bx = xb + wb / 2 if cb != 'ai' else ax
        top, bot = ya + ha, yb - 2
        mid = top + GAP / 2
        if ax == bx:
            d = f'M{ax:g},{top} V{bot}'
        else:
            d = f'M{ax:g},{top} V{mid:g} H{bx:g} V{bot}'
        out.append(f'<path class="flow" d="{d}" fill="none"/>')
    height = y - GAP + 10
    svg = f'<svg viewBox="0 0 420 {height}" role="img" aria-label="{escape(aria)}">\n' + '\n'.join(out) + '\n</svg>\n'
    open(f'{OUT}/{name}.source.svg', 'w').write(svg)


build(
    'flow-validate', '処理の流れ：validate 1件',
    'validate 1件の流れ。AI が対象と置き場所とセッションを付けて打つ。入口が引数を確かめ、validation の application を呼ぶ。'
    'DocumentStore で filesystem から文書を読み、SchemaValidator で jsonschema が置き場所の中から $id で引いた Concrete に照らす。'
    'validation の domain が当たる rule と組み込みの検査を当てる。ValidationLog で sqlite に記録し、入口が結果を1行で返す',
    [('入口（cli）', 10, 104), ('モジュール', 120, 150), ('ポート → アダプター', 276, 134)],
    [
        ('ai', ['AI：対象 ・ 置き場所 ・ セッションを決めて打つ', 'validate a.json --schemas s/ --session S']),
        (0, ['1 引数を確かめる', '足りなければ断る']),
        (1, ['2 validation：validate', '経路 ・ 置き場所', 'Caller']),
        (2, ['3 DocumentStore', 'read → filesystem', '渡された経路だけ']),
        (2, ['4 SchemaValidator', '→ jsonschema', '置き場所から $id で引く']),
        (1, ['5 validation/domain', '当たる rule', '組み込みの検査3つ']),
        (2, ['6 ValidationLog', 'append_validate', '→ sqlite']),
        (0, ['7 結果を1行で返す', 'command-result']),
        ('ai', ['AI：結果（validation）を読む']),
    ],
)

build(
    'flow-semantic', '処理の流れ：意味で書いて、変わった所を判定する',
    'update --semantic で手順をもらい、AI が JSON Patch を作って update --patch で書く。書いた変更は項目ごとに記録される。'
    'validate --semantic は記録から変わった所だけの手順を返し、AI が判定を validate --answer で渡すと、照らしてから記録する',
    [('モジュール（application ・ domain）', 10, 214), ('ポート → アダプター', 230, 180)],
    [
        ('ai', ['AI：意味で書く手順を依頼する', 'update a.json --semantic', '--schemas s/ --session S']),
        (1, ['1 DocumentStore.read', '→ filesystem', 'SchemaValidator', '→ jsonschema']),
        (0, ['2 writing：x-prompt から手順を作る', 'command に置き場所 ・ session を', '引き継ぐ。記録しない']),
        ('ai', ['AI：steps を行い、JSON Patch を作って打つ', 'update a.json --patch P --if-match H', '--schemas s/ --session S']),
        (0, ['3 writing：Patch を当てて照らす', '形 ・ 集約の書く条件 ・ --if-match']),
        (1, ['4 DocumentStore.write', '→ filesystem', 'ChangeLog.append_change', '→ sqlite：pointer ごとの前後']),
        ('ai', ['AI：変わった所を確かめる手順を依頼する', 'validate a.json --semantic', '--schemas s/ --session S']),
        (1, ['5 ValidationLog → sqlite', 'writes ・ last_judgments']),
        (0, ['6 validation：変わった所だけ集める', 'field_changes の pointer と', 'reads_hash で当たる rule を選ぶ']),
        ('ai', ['AI：rule ごとに判定して打つ', 'validate a.json --answer J', '--schemas s/ --session S']),
        (0, ['7 validation：答えを照らす', 'judgment-answer の形 ・ hash', 'step ・ rule ・ pointer']),
        (1, ['8 ValidationLog', 'append_validate → sqlite', 'rule_results（by = ai）']),
        ('ai', ['AI：結果（validation）を読む']),
    ],
)
