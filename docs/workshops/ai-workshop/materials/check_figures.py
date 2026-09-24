"""図の文字が、画布の外へ出ていないか・重なっていないかを機械で見る。

幅は日本語1文字＝字高、英数＝0.55字高で見積もる。厳密ではないので、
見積もりを超えたものだけを目視の候補として出す。
"""
import re, sys, unicodedata
import lesson_visuals as V

def width(s, size):
    w = 0.0
    for ch in s:
        w += size * (0.55 if unicodedata.east_asian_width(ch) in 'NaH' else 1.0)
    return w

def boxes(svg):
    out = []
    for m in re.finditer(r'<text x="([\d.-]+)" y="([\d.-]+)" font-size="([\d.]+)"[^>]*text-anchor="(\w+)">([^<]*)</text>', svg):
        x, y, size, anchor, s = float(m[1]), float(m[2]), float(m[3]), m[4], m[5]
        w = width(s, size)
        x0 = x if anchor == 'start' else (x - w / 2 if anchor == 'middle' else x - w)
        out.append((x0, y - size, x0 + w, y + size * 0.3, s))
    return out

def rects(svg, fill=False):
    out = []
    for m in re.finditer(r'<rect x="([\d.-]+)" y="([\d.-]+)" width="([\d.]+)" height="([\d.]+)" rx="[\d.]+" fill="([^"]+)"', svg):
        box = (float(m[1]), float(m[2]), float(m[1]) + float(m[3]), float(m[2]) + float(m[4]))
        out.append(box + (m[5],) if fill else box)
    return out

def circles(svg):
    return [(float(m[1]), float(m[2]), float(m[3]))
            for m in re.finditer(r'<circle cx="([\d.-]+)" cy="([\d.-]+)" r="([\d.]+)"', svg)]

def check(name, svg):
    vb = re.search(r'viewBox="0 0 ([\d.]+) ([\d.]+)"', svg)
    W, H = float(vb[1]), float(vb[2])
    bs = boxes(svg)
    bad = []
    for x0, y0, x1, y1, s in bs:
        if x0 < -1 or x1 > W + 1 or y1 > H + 1:
            bad.append(f'画布の外 [{s}] x{x0:.0f}..{x1:.0f} y{y1:.0f}（画布 {W:.0f}×{H:.0f}）')
    for i, a in enumerate(bs):
        for b in bs[i+1:]:
            if a[0] < b[2] and b[0] < a[2] and a[1] < b[3] and b[1] < a[3]:
                bad.append(f'文字が重なる [{a[4]}] と [{b[4]}]')
    for x0, y0, x1, y1, s in bs:
        cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
        for rx0, ry0, rx1, ry1 in rects(svg):
            inside = rx0 <= cx <= rx1 and ry0 <= cy <= ry1
            if inside and (x0 < rx0 - 1 or x1 > rx1 + 1):
                bad.append(f'箱からはみ出す [{s}] 文字 x{x0:.0f}..{x1:.0f} 箱 x{rx0:.0f}..{rx1:.0f}')
            elif not inside and x0 < rx1 - 1 and rx0 < x1 - 1 and y0 < ry1 - 1 and ry0 < y1 - 1:
                bad.append(f'箱に掛かる [{s}] 文字 x{x0:.0f}..{x1:.0f} 箱 x{rx0:.0f}..{rx1:.0f}')
        for ccx, ccy, r in circles(svg):
            near = max(abs(cx - ccx) - (x1 - x0) / 2, 0) ** 2 + max(abs(cy - ccy) - (y1 - y0) / 2, 0) ** 2
            if near < r * r and not (abs(cx - ccx) ** 2 + abs(cy - ccy) ** 2 < (r * 0.8) ** 2):
                bad.append(f'円の縁に掛かる [{s}]')
    # 同じ段に並ぶ箱は、幅・高さ・間隔を揃える ── 揃っていないと、揺らぎとして見える
    # 役割（塗り）が同じ箱どうしで見る ── 役割が違う箱は、幅が違ってよい
    rows = {}
    for rx0, ry0, rx1, ry1, fill in rects(svg, fill=True):
        rows.setdefault((round(ry0), round(ry1), fill), []).append((rx0, rx1))
    for (ry0, ry1, fill), xs in rows.items():
        if len(xs) < 3:
            continue
        xs.sort()
        ws = {round(b - a) for a, b in xs}
        if len(ws) > 1:
            bad.append(f'同じ段の箱の幅が揃っていない y{ry0} 幅 {sorted(ws)}')
        band = sum(len(v) for (y0, y1, _), v in rows.items() if (y0, y1) == (ry0, ry1))
        gaps = {round(xs[i + 1][0] - xs[i][1]) for i in range(len(xs) - 1)}
        if band == len(xs) and len(gaps) > 1:
            bad.append(f'同じ段の箱の間隔が揃っていない y{ry0} 間隔 {sorted(gaps)}')
    for b in bad: print(f'  × {name}: {b}')
    return len(bad)

if __name__ == '__main__':
    names = [n for n in dir(V) if n.startswith(('l1_', 'l2_', 'l3_', 'l4_', 'l5_', 'l6_'))]
    n = sum(check(x, getattr(V, x)()) for x in names)
    print(f'図の検査　{"通った" if not n else f"通っていない（{n} 件）"}　／　図 {len(names)} 枚')
    sys.exit(1 if n else 0)
