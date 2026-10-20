# ER 図を、表の形で組む。集約は破線の境界で囲み、中にルート（濃い箱）と値オブジェクト（薄い箱）を置く。
# 箱は見出しの行と属性の行に分け、左の列に印（ID ・ 参照 ・ 複数）を書く。
# 使い方：python3 make31.py <図の定義の JSON> <出す src.svg>
#   定義：{"width", "height", "aria", "legend",
#          "groups": [{"title", "x", "y", "w", "h", "kind": "aggregate|area|external"}],
#          "tables": [{"id", "title", "kind": "root|vo|index|external|output", "x", "y", "w",
#                      "rows": [["印", "属性"], ...]}],
#          "edges": [{"d": "M..", "label", "lx", "ly"}]}
import json, sys, html

spec = json.load(open(sys.argv[1]))
HEAD, ROW = 28, 20
MARKW = spec.get('markw', 34)
CLS = {'root': 'box focus', 'entity': 'box focus', 'vo': 'box', 'index': 'box', 'output': 'box', 'external': 'boundary'}
GCLS = {'aggregate': 'boundary', 'area': 'area', 'external': 'boundary'}
out = []
for g in spec.get('groups', []):
    out.append(f'<rect class="{GCLS[g["kind"]]}" x="{g["x"]}" y="{g["y"]}" width="{g["w"]}" height="{g["h"]}"/>')
    out.append(f'<text class="note" x="{g["x"] + 10}" y="{g["y"] + 16}">{html.escape(g["title"])}</text>')
    if g.get('file'):
        out.append(f'<text class="note" x="{g["x"] + 10}" y="{g["y"] + g["h"] - 6}">{html.escape(g["file"])}</text>')
for t in spec['tables']:
    x, y, w = t['x'], t['y'], t['w']
    h = HEAD + ROW * len(t['rows']) + 6
    out.append(f'<rect class="{CLS[t["kind"]]}" x="{x}" y="{y}" width="{w}" height="{h}"/>')
    out.append(f'<text class="title" x="{x + 10}" y="{y + 19}">{html.escape(t["title"])}</text>')
    if t['rows']:
        out.append(f'<path class="link" d="M{x},{y + HEAD} H{x + w}"/>')
    for i, (mark, name) in enumerate(t['rows']):
        ry = y + HEAD + ROW * i + 15
        if mark:
            out.append(f'<text class="note" x="{x + 8}" y="{ry}">{html.escape(mark)}</text>')
        out.append(f'<text class="label" x="{x + 8 + MARKW}" y="{ry}">{html.escape(name)}</text>')
for e in spec.get('edges', []):
    out.append(f'<path class="flow" d="{e["d"]}"/>')
    if e.get('label'):
        out.append(f'<text class="note" x="{e["lx"]}" y="{e["ly"]}">{html.escape(e["label"])}</text>')
legend = f'<text class="note" x="16" y="20">{html.escape(spec["legend"])}</text>\n' if spec.get('legend') else ''
svg = (f'<svg viewBox="0 0 {spec["width"]} {spec["height"]}" style="min-width:{spec.get("minw", 860)}px" role="img" '
       f'aria-label="{html.escape(spec["aria"])}">\n' + legend + '\n'.join(out) + '\n</svg>\n')
open(sys.argv[2], 'w').write(svg)
print('ok', sys.argv[2])
