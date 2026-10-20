# 表の形の図（q9-subdomains ・ q9-contextmap ・ q9-belong）を、いまの src.svg から読み取り、
# 行の差し替え ・ 追加をしてから組み直す。折り返しは make29.py と同じ規則（語を割らない ・ 行頭と行末の禁則）。
# 使い方：python3 make30.py <図の名前> <変更の JSON>
#   変更の JSON：{"caption": 見出しの aria-label（任意）,
#                 "set": [[行の番号, 列の番号, "文"], ...],
#                 "insert": [[この番号の行の後ろ, ["列1", "列2", ...]], ...]}
import json, os, re, sys, html
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from make29 import wrap  # noqa: E402

H = os.path.dirname(os.path.abspath(__file__))
name, change = sys.argv[1], json.load(open(sys.argv[2]))
src = open(os.path.join(H, name + '.src.svg')).read()

head_svg = re.match(r'<svg [^>]*>', src).group(0)
notes = re.findall(r'<text class="note" x="(\d+)" y="(\d+)">([^<]*)</text>', src)
header = [(int(x), t) for x, y, t in notes if int(y) < 30]
footer = [(int(x), t) for x, y, t in notes if int(y) >= 30]
rects = [(m.group(1), int(m.group(2)), int(m.group(3)), int(m.group(4)), int(m.group(5)))
         for m in re.finditer(r'<rect class="([^"]+)" x="(\d+)" y="(\d+)" width="(\d+)" height="(\d+)"/>', src)]
texts = [(m.group(1), int(m.group(2)), int(m.group(3)), html.unescape(m.group(4)))
         for m in re.finditer(r'<text class="(title|label)" x="(\d+)" y="(\d+)">([^<]*)</text>', src)]

def join(parts):
    # 折り返しで分かれた行をつなぐ。英数字の語と和文の境目では、元の空白を戻す
    out = ''
    for p in parts:
        if out and (ord(out[-1]) < 0x2E80 or ord(p[0]) < 0x2E80) and out[-1] not in ' 。、）」・' and p[0] not in ' （「、。':
            out += ' '
        out += p
    return out


rows = {}
for cls, x, y, w, h in rects:
    rows.setdefault(y, []).append([cls, x, w, h])
table = []
for y in sorted(rows):
    cells = []
    for cls, x, w, h in sorted(rows[y], key=lambda c: c[1]):
        lines = [(ty, t, tc) for tc, tx, ty, t in texts if x <= tx < x + w and y <= ty < y + h]
        lines.sort()
        cells.append({'cls': cls, 'x': x, 'w': w,
                      'tcls': lines[0][2] if lines else 'label',
                      'text': join([t for _, t, _ in lines])})
    table.append(cells)

for r, c, t in change.get('set', []):
    table[r][c]['text'] = t
for after, vals in sorted(change.get('insert', []), key=lambda a: -a[0]):
    tmpl = table[after]
    table.insert(after + 1, [dict(cell, text=v) for cell, v in zip(tmpl, vals)])

out, y = [], 32
for cells in table:
    wrapped = [wrap(c['text'], c['w'] + 14) if c['tcls'] == 'label' else [c['text']] for c in cells]
    n = max(len(w) for w in wrapped)
    h = max(46, 14 + 16 * n)
    for c, lines in zip(cells, wrapped):
        out.append(f'<rect class="{c["cls"]}" x="{c["x"]}" y="{y}" width="{c["w"]}" height="{h}"/>')
        start = y + (h - 16 * (len(lines) - 1)) / 2 + 4
        for i, l in enumerate(lines):
            out.append(f'<text class="{c["tcls"]}" x="{c["x"] + 10}" y="{int(start + 16 * i)}">{html.escape(l)}</text>')
    y += h + 6

foot = ''.join(f'<text class="note" x="{x}" y="{y + 16 + 18 * i}">{html.escape(t)}</text>\n' for i, (x, t) in enumerate(footer))
total = y + 10 + (18 * len(footer) + 10 if footer else 0)
width = re.search(r'viewBox="0 0 (\d+) \d+"', head_svg).group(1)
head_svg = re.sub(r'viewBox="0 0 \d+ \d+"', f'viewBox="0 0 {width} {total}"', head_svg)
if change.get('caption'):
    head_svg = re.sub(r'aria-label="[^"]*"', f'aria-label="{change["caption"]}"', head_svg)
hdr = ''.join(f'<text class="note" x="{x}" y="22">{html.escape(t)}</text>\n' for x, t in header)
open(os.path.join(H, name + '.src.svg'), 'w').write(head_svg + '\n' + hdr + '\n'.join(out) + '\n' + foot + '</svg>\n')
print(name, 'rows', len(table))
