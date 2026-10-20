# 図「使う場面」（q8-usecases.src.svg）を measured/domain-model-v2.json の use_cases から組む。
# 列の位置と行の高さは前の図（make16.py ・ make22.py が作ったもの）に合わせる。
# 実行：python3 make29.py（このフォルダで）。出た src.svg を design-svg resolve に通す。
import json, os, html

H = os.path.dirname(os.path.abspath(__file__))
dm = json.load(open(os.path.join(H, '..', 'measured', 'domain-model-v2.json')))

# 列：(箱の x, 箱の幅)。文字は箱の x+10 から書く
COLS = [(84, 140), (232, 240), (480, 300), (788, 300)]
HEAD = [(20, '#'), (88, '主体'), (236, '目的'), (484, '成功したとき'), (792, '最低限')]
LINE, TOP, GAP = 17, 25, 6
NO_HEAD = set('、。）」・ー,.)')  # 行頭に置かない文字


def width(ch):
    return 7.6 if ord(ch) < 0x2E80 else 12.0


def wrap(text, w):
    # 英数字の語（- _ $ を含む）は1つの塊として扱い、途中で折り返さない
    import re
    toks = re.findall(r'[A-Za-z0-9_$@#.\-]+|.', text)
    lim, lines, cur, cw = w - 34, [], "", 0.0
    for tk in toks:
        tw = sum(width(c) for c in tk)
        if cw + tw > lim and cur.strip():
            if tk[0] in NO_HEAD and len(tk) == 1:
                # 行頭に来る記号は、前の1文字と一緒に次の行へ送る
                last = cur[-1]
                lines.append(cur[:-1])
                cur, cw = last + tk, width(last) + tw
                continue
            lines.append(cur)
            cur, cw = '', 0.0
            if tk == ' ':
                continue
        cur += tk
        cw += tw
    if cur:
        lines.append(cur)
    lines = [l.strip() for l in lines if l.strip()]
    # 行頭の禁則（・ 、 など）と行末の禁則（（ 「）を、前後の行と1文字ずつやり取りして直す
    for k in range(1, len(lines)):
        while lines[k] and lines[k][0] in NO_HEAD and lines[k - 1]:
            lines[k] = lines[k - 1][-1] + lines[k]
            lines[k - 1] = lines[k - 1][:-1].rstrip()
        lines[k] = lines[k].strip()
    for k in range(len(lines) - 1):
        while lines[k] and lines[k][-1] in '（「':
            lines[k + 1] = lines[k][-1] + lines[k + 1]
            lines[k] = lines[k][:-1].rstrip()
    return [l for l in lines if l]


out, y = [], 32
for u in dm['use_cases']:
    cells = [wrap(c, w) for c, (_, w) in zip(u[1:5], COLS)]
    n = max(len(c) for c in cells)
    h = 24 + LINE * n
    out.append(f'<rect class="box focus" x="16" y="{y}" width="60" height="{h}"/>')
    out.append(f'<text class="title" x="26" y="{y + TOP}">{u[0]}</text>')
    for lines, (x, w) in zip(cells, COLS):
        out.append(f'<rect class="box" x="{x}" y="{y}" width="{w}" height="{h}"/>')
        for i, l in enumerate(lines):
            out.append(f'<text class="label" x="{x + 10}" y="{y + TOP + LINE * i}">{html.escape(l)}</text>')
    y += h + GAP

head = ''.join(f'<text class="note" x="{x}" y="22">{t}</text>\n' for x, t in HEAD)
svg = (f'<svg viewBox="0 0 1104 {y + 10}" style="min-width:900px" role="img" '
       f'aria-label="使う場面：主体 ・ 目的 ・ 成功したときの保証 ・ 最低限の保証">\n'
       + head + '\n'.join(out) + '\n</svg>\n')
open(os.path.join(H, 'q8-usecases.src.svg'), 'w').write(svg)
print('rows', len(dm['use_cases']), 'height', y + 10)
