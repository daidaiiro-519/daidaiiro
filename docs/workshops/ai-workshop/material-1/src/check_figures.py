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
    # 別の枠に入った箱は、同じ高さでも別の段である。いちばん内側の囲む枠ごとに分ける
    all_r = rects(svg, fill=True)
    def parent(r):
        outs = [o for o in all_r if o is not r and o[0] <= r[0] and o[2] >= r[2] and o[1] <= r[1] and o[3] >= r[3]
                and (o[2] - o[0]) * (o[3] - o[1]) > (r[2] - r[0]) * (r[3] - r[1])]
        m = min(outs, key=lambda o: (o[2] - o[0]) * (o[3] - o[1]), default=None)
        return None if m is None else tuple(round(v) for v in m[:4])
    rows = {}
    for r in all_r:
        rx0, ry0, rx1, ry1, fill = r
        rows.setdefault((round(ry0), round(ry1), fill, parent(r)), []).append((rx0, rx1))
    for (ry0, ry1, fill, par), xs in rows.items():
        if len(xs) < 3:
            continue
        xs.sort()
        ws = {round(b - a) for a, b in xs}
        if len(ws) > 1:
            bad.append(f'同じ段の箱の幅が揃っていない y{ry0} 幅 {sorted(ws)}')
        band = sum(len(v) for (y0, y1, _, pp), v in rows.items() if (y0, y1, pp) == (ry0, ry1, par))
        gaps = {round(xs[i + 1][0] - xs[i][1]) for i in range(len(xs) - 1)}
        if band == len(xs) and len(gaps) > 1:
            bad.append(f'同じ段の箱の間隔が揃っていない y{ry0} 間隔 {sorted(gaps)}')
    # 横向きの矢印は、両端が何か（箱 ・ 文字 ・ 図形）の縁に届いている ── 届いていないと、何から何へ渡すのかが読めない
    targets = [r[:4] for r in rects(svg)] + [b[:4] for b in bs]
    targets += [(cx - r, cy - r, cx + r, cy + r) for cx, cy, r in circles(svg)]
    # 閉じた多角形（階段 ・ 文書の形）も、届く相手に含める。M ・ L ・ H ・ V の絶対座標から外接矩形を取る
    for m in re.finditer(r'd="(M[^"]*?Z)"', svg):
        xs, ys, cx, cy = [], [], 0.0, 0.0
        for cmd, args in re.findall(r'([MLHV])\s*([\d.\s-]+)', m[1]):
            nums = [float(v) for v in args.split()]
            if cmd in 'ML':
                for i in range(0, len(nums) - 1, 2):
                    cx, cy = nums[i], nums[i + 1]; xs.append(cx); ys.append(cy)
            elif cmd == 'H':
                cx = nums[-1]; xs.append(cx); ys.append(cy)
            else:
                cy = nums[-1]; xs.append(cx); ys.append(cy)
        if xs:
            targets.append((min(xs), min(ys), max(xs), max(ys)))
    # 届いたとみなす隙間。ロボットの図形は腕の外に余白を持つので、20 まで許す
    reach = 20
    # 矢印は visuals.arrow が描く形だけを見る ── 横線のあとに矢じり（l8 7 -8 7）が続く。表の罫線やロボットの腕は矢印ではない
    for m in re.finditer(r'd="M([\d.-]+) ([\d.-]+) H([\d.-]+) M[\d.-]+ [\d.-]+ l-?8 7 -?8 7"', svg):
        x1, y, x2 = float(m[1]), float(m[2]), float(m[3])
        right = x2 > x1
        def near(x, edge):
            # 端が、縁から外側へ reach 以内にある対象を探す。矢じり側は相手の手前、根元側は元の後ろ
            for t in targets:
                if not (t[1] - 2 <= y <= t[3] + 2):
                    continue
                e = t[0] if edge == 'left' else t[2]
                if abs(e - x) <= reach:
                    return True
            return False
        head_ok = near(x2, 'left' if right else 'right')
        tail_ok = near(x1, 'right' if right else 'left')
        if not (head_ok and tail_ok):
            where = '矢じり' if not head_ok else '根元'
            bad.append(f'矢印の{where}が何にも届いていない x{x1:.0f}→{x2:.0f} y{y:.0f}')
    for b in bad: print(f'  × {name}: {b}')
    return len(bad)

if __name__ == '__main__':
    # 教材で実際に使っている図は、組み立ての対応表（FIG）が持つ。そこから全部を見る
    import build_lessons as B
    n = sum(check(f'{k}({fn.__name__})', fn()) for k, fn in B.FIG.items())
    print(f'図の検査　{"通った" if not n else f"通っていない（{n} 件）"}　／　図 {len(B.FIG)} 枚')
    sys.exit(1 if n else 0)
