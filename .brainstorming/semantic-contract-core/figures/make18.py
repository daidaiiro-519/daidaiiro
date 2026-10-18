# 第59回の図：Q1 を、使う場面（Q8）から導いた不変条件 ・ 能力 ・ 状態表で組み直す。
# 材料は measured/domain-model.json（場面から導き、ddd-advisor ・ qa-advisor ・ ux-advisor の反証を当てたもの）。
import json, runpy
e = runpy.run_path('make13_table.py')['e']

def width(s):
    return sum(7.6 if ord(ch) < 128 else 12.8 for ch in s)

def wrap(text, w):
    """欄の幅に収まるように、句読点か空白の後ろで折り返す。"""
    lines, cur = [], ''
    for ch in text:
        if width(cur + ch) > w - 20 and cur:
            cut = max(cur.rfind('、'), cur.rfind('。'), cur.rfind(' '), cur.rfind('・'))
            if cut >= len(cur) * 0.5:
                lines.append(cur[:cut + 1].rstrip()); cur = cur[cut + 1:].lstrip() + ch
            elif ch.isascii() and ch.isalnum() and cur[-1].isascii() and cur[-1].isalnum():
                # 英数の語の途中では切らず、語の前で切る
                k = len(cur)
                while k > 0 and cur[k - 1].isascii() and cur[k - 1].isalnum():
                    k -= 1
                if k > 0:
                    lines.append(cur[:k].rstrip()); cur = cur[k:] + ch
                else:
                    lines.append(cur); cur = ch
            else:
                lines.append(cur); cur = ch
        else:
            cur += ch
    if cur:
        lines.append(cur)
    return lines

def grouped(name, label, heads, widths, rows, group_col=1, line_h=17, pad=12):
    """1列目を固めて並べる表。行の高さは、欄の中で折り返した行の数で決める。"""
    X = [16]
    for w in widths[:-1]:
        X.append(X[-1] + w + 8)
    W = X[-1] + widths[-1] + 16
    cells = [[wrap(str(v), widths[c]) for c, v in enumerate(r)] for r in rows]
    hs = [max(len(l) for l in cl) * line_h + pad * 2 for cl in cells]
    ys = [32]
    for h in hs[:-1]:
        ys.append(ys[-1] + h + 6)
    H = ys[-1] + hs[-1] + 16
    o = [f'<svg viewBox="0 0 {W} {H}" style="min-width:{min(W, 900)}px" role="img" aria-label="{e(label)}">']
    for x, h in zip(X, heads):
        o.append(f'<text class="note" x="{x + 4}" y="22">{e(h)}</text>')
    i = 0
    while i < len(rows):
        j = i
        while j + 1 < len(rows) and rows[j + 1][group_col] == rows[i][group_col]:
            j += 1
        y = ys[i]; h = ys[j] + hs[j] - y
        lines = wrap(rows[i][group_col], widths[group_col])
        o.append(f'<rect class="box focus" x="{X[group_col]}" y="{y}" width="{widths[group_col]}" height="{h}"/>')
        for k, l in enumerate(lines):
            o.append(f'<text class="title" x="{X[group_col] + 10}" y="{y + pad + 13 + line_h * k}">{e(l)}</text>')
        i = j + 1
    for r, (y, h, cl) in enumerate(zip(ys, hs, cells)):
        for c, lines in enumerate(cl):
            if c == group_col:
                continue
            o.append(f'<rect class="box" x="{X[c]}" y="{y}" width="{widths[c]}" height="{h}"/>')
            for k, l in enumerate(lines):
                o.append(f'<text class="label" x="{X[c] + 10}" y="{y + pad + 13 + line_h * k}">{e(l)}</text>')
    o.append('</svg>')
    open(name + '.src.svg', 'w').write('\n'.join(o))

def matrix(name, label, rows_h, cols_h, cell, w0=110, cw=150, line_h=16, pad=10):
    """状態 × 出来事の表。欄には遷移先と、それを決める不変条件を書く。"""
    data = [[wrap(cell(r, c), cw) for c in cols_h] for r in rows_h]
    hs = [max(len(l) for l in row) * line_h + pad * 2 for row in data]
    hh = max(len(wrap(c, cw)) for c in cols_h) * line_h + pad * 2
    W = 16 + w0 + 8 + len(cols_h) * (cw + 8) + 8
    ys = [16 + hh + 6]
    for h in hs[:-1]:
        ys.append(ys[-1] + h + 6)
    H = ys[-1] + hs[-1] + 16
    o = [f'<svg viewBox="0 0 {W} {H}" style="min-width:900px" role="img" aria-label="{e(label)}">']
    for k, c in enumerate(cols_h):
        x = 16 + w0 + 8 + k * (cw + 8)
        o.append(f'<rect class="box focus" x="{x}" y="16" width="{cw}" height="{hh}"/>')
        for n, l in enumerate(wrap(c, cw)):
            o.append(f'<text class="title" x="{x + 8}" y="{16 + pad + 12 + line_h * n}">{e(l)}</text>')
    for r, (y, h, row) in enumerate(zip(ys, hs, data)):
        o.append(f'<rect class="box focus" x="16" y="{y}" width="{w0}" height="{h}"/><text class="title" x="26" y="{y + pad + 12}">{e(rows_h[r])}</text>')
        for k, lines in enumerate(row):
            x = 16 + w0 + 8 + k * (cw + 8)
            cls = 'box warn' if lines and lines[0].startswith('無効') else 'box'
            o.append(f'<rect class="{cls}" x="{x}" y="{y}" width="{cw}" height="{h}"/>')
            for n, l in enumerate(lines):
                o.append(f'<text class="label" x="{x + 8}" y="{y + pad + 12 + line_h * n}">{e(l)}</text>')
    o.append('</svg>')
    open(name + '.src.svg', 'w').write('\n'.join(o))

if __name__ == '__main__':
    d = json.load(open('../measured/domain-model.json'))
    order = d['concept_order']
    inv = sorted(d['invariants'], key=lambda i: (order.index(i['concept']), i['order']))
    grouped('q1-invariants', '不変条件：使う場面から導いた行を、概念ごとに固めた表',
            ['#', '概念', '不変条件', '検査の時点', '場面'], [56, 150, 520, 110, 90],
            [[i['id'], i['concept'], i['text'], i['when'], ' ・ '.join(i['from_u'])] for i in inv])
    kinds = d['kind_order']
    caps = sorted(d['capabilities'], key=lambda c: (kinds.index(c['kind']), c['order']))
    grouped('q1-capabilities', 'エンジンの能力：種類ごとに固めた表',
            ['#', '種類', '能力', '何をするか', '場面'], [40, 130, 170, 470, 110],
            [[str(n + 1), c['kind'], c['name'], c['does'], ' ・ '.join(c['from_u'])] for n, c in enumerate(caps)])
    st = d['state_table']
    by = {(c['state'], c['event']): c for c in st['cells']}
    def cell(s, ev):
        c = by[(s, ev)]
        return c['to'] + ('（' + ' ・ '.join(c['by']) + '）' if c['by'] else '')
    half = (len(st['events']) + 1) // 2
    matrix('q1-states-a', '承認の状態 × 出来事（前半）：遷移先と、それを決める不変条件', st['states'], st['events'][:half], cell)
    matrix('q1-states-b', '承認の状態 × 出来事（後半）：遷移先と、それを決める不変条件', st['states'], st['events'][half:], cell)
