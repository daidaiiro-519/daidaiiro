"""処理の流れの図の元（figs/*.source.svg）を、ステップの並びから作る。座標は行の高さから毎回計算する。"""
from html import escape
from pathlib import Path

OUT = Path(__file__).resolve().parent / 'figs'
LINE = 15      # 1行の高さ
PAD = 12       # 箱の上下の余白
GAP = 22       # ステップの間


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
    # ステップどうしの矢印：前の箱の下の中央から、次の箱の上の中央へ（AI の箱は相手の列の中央を使う）
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
    'flow-semantic', '処理の流れ：procedure で更新し、変更点を判定する',
    'update --semantic で procedure を受け取り、AI が JSON Patch を作って update --patch で書く。'
    'validate --semantic は前回の検証と比べた変更点の rule だけで procedure を返し、AI が answer を validate --answer で渡すと、検証してから操作の記録に追記する',
    [('authoring（application ・ domain）', 10, 200), ('ポート → アダプター', 216, 194)],
    [
        ('ai', ['AI：更新の procedure を依頼する', 'update a.json --semantic', '--schemas s/']),
        (1, ['1 Documents.Read → files', 'Schemas → files']),
        (0, ['2 domain：サブスキーマを引き', 'procedure を作る。next に', '--schemas を入れる']),
        ('ai', ['AI：procedure に従い JSON Patch を作る', 'update a.json --patch P', '--schemas s/']),
        (1, ['3 Patcher → patching', 'Validator → schemacheck']),
        (0, ['4 domain：集約 JSON ドキュメント', 'x-decided-by ・ 読んだハッシュ', 'change を出す']),
        (1, ['5 Records.WithLock', '→ recordstore：ロックの中で', '追記して Documents.Write']),
        ('ai', ['AI：判定の procedure を依頼する', 'validate a.json --semantic', '--schemas s/']),
        (1, ['6 Records', '.LastValidation', '→ recordstore：前回の検証']),
        (0, ['7 domain：変更点を求める', 'JSON Pointer ごとに比べ', 'その rule で procedure を作る']),
        ('ai', ['AI：rule ごとに判定する', 'validate a.json --answer J', '--schemas s/']),
        (1, ['8 Validator → schemacheck', 'judgment-answer.schema.json']),
        (0, ['9 domain：集約検証', 'answer の組が procedure と同じ']),
        (1, ['10 Records → recordstore', 'validations ・ judgments']),
        ('ai', ['AI：validation のログイベントを読む']),
    ],
)

build(
    'flow-error', '処理の流れ：error のときの AI の次のアクション',
    '更新で、書く直前にファイルのハッシュが最初に読んだときから変わっていたときの流れ。Semantic Agent Schema は書かずに hash_mismatch の error のログイベントを返し、'
    'instruction と next を持たせる。AI は next の同じコマンドを打ち直し、Semantic Agent Schema は読み直したファイルに JSON Patch を当てて書く',
    [('Semantic Agent Schema', 10, 200), ('出力（ログイベント）', 216, 194)],
    [
        ('ai', ['AI：update a.json --patch P', '--schemas s/']),
        (0, ['1 読んでハッシュ aaa を控え', 'JSON Patch を当てて検証する']),
        (0, ['2 ロックの中でハッシュを比べる', 'いまは bbb（ほかが書いた）']),
        (1, ['3 error のログイベント', 'code：hash_mismatch', 'instruction：同じコマンドを', '打ち直す']),
        (1, ['next：update a.json', '--patch P --schemas s/']),
        ('ai', ['AI：next を実行する', 'update a.json --patch P --schemas s/']),
        (0, ['4 bbb を読んで JSON Patch を当て', 'ロックの中でも bbb なので書く']),
        (1, ['5 change のログイベント', '書けた']),
    ],
)
