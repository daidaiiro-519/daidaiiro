# 表の図を、列と行の定義から組む（見出しの列は濃い箱、ほかは薄い箱）。折り返しは make29.py と同じ規則。
# 使い方：python3 make32.py <定義の JSON> <出す src.svg>
#   定義：{"aria", "cols": [["見出し", 幅], ...], "rows": [["列1", "列2", ...], ...]}
import json, sys, html, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make29 import wrap

spec = json.load(open(sys.argv[1]))
xs, x = [], 16
for _, w in spec['cols']:
    xs.append(x); x += w + 8
width = x + 8
out = [f'<text class="note" x="{xx + 4}" y="22">{html.escape(h)}</text>' for xx, (h, _) in zip(xs, spec['cols'])]
y = 32
for row in spec['rows']:
    cells = [wrap(c, w + 14) if i else [c] for i, (c, (_, w)) in enumerate(zip(row, spec['cols']))]
    n = max(len(c) for c in cells)
    h = max(40, 14 + 17 * n)
    for i, (lines, xx, (_, w)) in enumerate(zip(cells, xs, spec['cols'])):
        out.append(f'<rect class="box{" focus" if i == 0 else ""}" x="{xx}" y="{y}" width="{w}" height="{h}"/>')
        for k, l in enumerate(lines):
            out.append(f'<text class="{"title" if i == 0 else "label"}" x="{xx + 10}" y="{y + 25 + 17 * k}">{html.escape(l)}</text>')
    y += h + 6
svg = (f'<svg viewBox="0 0 {width} {y + 10}" style="min-width:{min(width, 900)}px" role="img" aria-label="{html.escape(spec["aria"])}">\n'
       + '\n'.join(out) + '\n</svg>\n')
open(sys.argv[2], 'w').write(svg)
print('ok', sys.argv[2])
