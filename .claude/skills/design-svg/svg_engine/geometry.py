"""座標まわりの共通の道具 ── 同じ計算を複数の場所で書かないための置き場。

同じ概念が別々に実装されていると、片方だけ直したときに気づけない。実際に
「折れ線を細かく刻む」が2箇所、「線分が箱を通るか」が2箇所、「原点を左上へ
寄せる」が5箇所にあり、刻みの細かさや余裕の取り方が場所ごとに違っていた。

**検査（verify.py）はここを使わない。** 検査が計算と同じコードを使うと、
その計算の誤りを検査が永久に見つけられなくなる（実際に、経路と検査が
「障害物＝箱」という同じ前提を共有していたため、線が文字を貫く崩れを
どちらも見逃していた）。検査は独立に書く。
"""
from __future__ import annotations

Point = tuple[float, float]

# 標本の粗さ ── 最も小さい辺を何等分するか。跳び越して見逃さないだけの細かさで、
# かつ点が増えすぎない値として 4 を採る。検査側（verify.py）も同じ粗さを使う
# ── 描く側と検査側で粗さが違うと、片方だけが見つける崩れができる。
SAMPLE_DIVISOR = 4

# 多角形として閉じるのに要る最小の頂点数。
MIN_POLYGON = 3
Rect = tuple[float, float, float, float]


def densify(pts: list[Point], step: float) -> list[Point]:
    """折れ線の途中も含めて点を拾う。

    頂点だけを見ると、両端が箱の外にある直線が箱を貫いていても分からない。
    刻みは呼び出し側が渡す ── 避けたい相手より粗いと跳び越して見逃すので、
    その相手の一番短い辺から導く。既定値を置かない。

    Args:
        pts: 折れ線の点の並び。
        step: 刻みの間隔。

    Returns:
        途中の点も含めた並び。

    Raises:
        なし。
    """
    if len(pts) < 2:
        return list(pts)
    out = [pts[0]]
    for (x1, y1), (x2, y2) in zip(pts, pts[1:]):
        d = ((x2 - x1) ** 2 + (y2 - y1) ** 2) ** 0.5
        n = max(int(d / max(step, 1e-9)), 1)
        for i in range(1, n + 1):
            t = i / n
            out.append((x1 + (x2 - x1) * t, y1 + (y2 - y1) * t))
    return out


def segment_hits_rect(p0: Point, p1: Point, rect: Rect, margin: float) -> bool:
    """線分が矩形の内側を通るか。

    刻みの細かさは矩形の短辺から決める ── 矩形より粗く刻むと跳び越す。

    Args:
        p0: 線分の始点。
        p1: 線分の終点。
        rect: (x0, y0, x1, y1)。
        margin: 矩形の内側へ取る余裕。縁に触れるのは通ったとみなさない。

    Returns:
        通るなら True。

    Raises:
        なし。
    """
    x0, y0, x1, y1 = rect
    x0 += margin
    y0 += margin
    x1 -= margin
    y1 -= margin
    if x1 <= x0 or y1 <= y0:
        return False
    step = min(x1 - x0, y1 - y0) / SAMPLE_DIVISOR
    for x, y in densify([p0, p1], max(step, 1e-9)):
        if x0 < x < x1 and y0 < y < y1:
            return True
    return False


def rects_overlap(a: Rect, b: Rect) -> bool:
    """2つの矩形が重なるか。縁で接するだけは重なりとみなさない。

    Args:
        a: (x0, y0, x1, y1)。
        b: 同上。

    Returns:
        重なるなら True。

    Raises:
        なし。
    """
    return a[0] < b[2] and a[2] > b[0] and a[1] < b[3] and a[3] > b[1]


def shift_to_origin(points: dict[str, Point]) -> tuple[dict[str, Point], Point]:
    """左上が原点に来るよう、全体を平行移動する。

    描く側は座標が0から始まる前提で画布を決めるので、負の座標を残せない。

    Args:
        points: 名前 → 座標。

    Returns:
        (移動後の座標, 移動した量)。空なら入力をそのまま返す。

    Raises:
        なし。
    """
    if not points:
        return points, (0.0, 0.0)
    off_x = min(x for x, _ in points.values())
    off_y = min(y for _, y in points.values())
    if not off_x and not off_y:
        return points, (0.0, 0.0)
    moved = {k: (x - off_x, y - off_y) for k, (x, y) in points.items()}
    return moved, (off_x, off_y)


# ---------------------------------------------------------------------------
# 描いたインクから、辺の着き先を導く
#
# 輪郭を部品ごとに手で書くと、書いた形と描いた形がずれる。実際に、輪郭を
# 申告していたのは11種のうち3種だけで、残りは外接矩形で代用され、インクが
# 矩形の一部にしか無い部品（波・三日月）では辺が何も無いところへ着いていた。
#
# ここでは輪郭という中間物を置かない。部品が描いたインクの点をそのまま持ち、
# 「狙った側にいちばん近いインク」を着き先にする。導出が1本なら、部品を
# 足しても着き先の決め方が増えない。
# ---------------------------------------------------------------------------

_NUM = None  # 遅延importのための場所取り


def _numbers(text: str) -> list[float]:
    import re
    global _NUM
    if _NUM is None:
        _NUM = re.compile(r"-?\d*\.?\d+(?:[eE][-+]?\d+)?")
    return [float(v) for v in _NUM.findall(text)]


def _arc_points(p0: Point, rx: float, ry: float, rot: float,
                large: int, sweep: int, p1: Point, step: float) -> list[Point]:
    """円弧を点列にする。SVGの終点指定を中心指定へ直してから刻む。

    刻む数は弧の長さと刻み幅から決める。決め打ちにすると、大きな弧では
    点と点の間が開き、その間の向きが輪郭から抜け落ちる（輪の部品で実測：
    半径52の輪の輪郭が、向きによって60〜64.5とばらついた）。
    """
    import math
    if rx == 0 or ry == 0:
        return [p1]
    phi = math.radians(rot)
    cos_p, sin_p = math.cos(phi), math.sin(phi)
    dx2, dy2 = (p0[0] - p1[0]) / 2, (p0[1] - p1[1]) / 2
    x1 = cos_p * dx2 + sin_p * dy2
    y1 = -sin_p * dx2 + cos_p * dy2
    rx, ry = abs(rx), abs(ry)
    lam = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry)
    if lam > 1:
        rx *= lam ** 0.5
        ry *= lam ** 0.5
    num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1
    den = rx * rx * y1 * y1 + ry * ry * x1 * x1
    coef = (max(num, 0.0) / den) ** 0.5 if den else 0.0
    if large == sweep:
        coef = -coef
    cx1 = coef * rx * y1 / ry
    cy1 = -coef * ry * x1 / rx
    cx = cos_p * cx1 - sin_p * cy1 + (p0[0] + p1[0]) / 2
    cy = sin_p * cx1 + cos_p * cy1 + (p0[1] + p1[1]) / 2
    a0 = math.atan2((y1 - cy1) / ry, (x1 - cx1) / rx)
    a1 = math.atan2((-y1 - cy1) / ry, (-x1 - cx1) / rx)
    sweep_angle = a1 - a0
    if sweep and sweep_angle < 0:
        sweep_angle += 2 * math.pi
    elif not sweep and sweep_angle > 0:
        sweep_angle -= 2 * math.pi
    steps = max(2, int(abs(sweep_angle) * max(rx, ry) / max(step, 1e-9)) + 1)
    out = []
    for i in range(1, steps + 1):
        a = a0 + sweep_angle * i / steps
        px = cos_p * rx * math.cos(a) - sin_p * ry * math.sin(a) + cx
        py = sin_p * rx * math.cos(a) + cos_p * ry * math.sin(a) + cy
        out.append((px, py))
    return out


def _curve_steps(control: list[Point], step: float) -> int:
    """制御点をたどった長さから、その曲線を何分割するかを決める。"""
    length = sum(((b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2) ** 0.5
                 for a, b in zip(control, control[1:]))
    return max(2, int(length / max(step, 1e-9)) + 1)


def sample_path(d: str, step: float) -> list[Point]:
    """path の d を点列にする。M/L/H/V/C/S/Q/T/A/Z と、その相対形に対応する。

    曲線は制御点ではなく曲線上を刻む。制御点を輪郭に混ぜると、実際には
    インクが無いところまで輪郭が張り出す。

    Args:
        d: path の d 属性。
        step: 刻み幅。曲線は制御点までの長さからこの幅に見合う数へ分ける
            ── 本数を決め打ちにすると、大きな曲線ほど粗くなる。

    Returns:
        点の並び。

    Raises:
        なし（読めないコマンドは読み飛ばす）。
    """
    import re
    out: list[Point] = []
    cur = (0.0, 0.0)
    start = (0.0, 0.0)
    prev_c: Point | None = None
    prev_q: Point | None = None
    for m in re.finditer(r"([MmLlHhVvCcSsQqTtAaZz])([^MmLlHhVvCcSsQqTtAaZz]*)", d):
        cmd, nums = m.group(1), _numbers(m.group(2))
        rel = cmd.islower()
        up = cmd.upper()
        i = 0
        first = True
        while True:
            need = {"M": 2, "L": 2, "H": 1, "V": 1, "C": 6,
                    "S": 4, "Q": 4, "T": 2, "A": 7, "Z": 0}[up]
            if up == "Z":
                if out:
                    out.append(start)
                cur = start
                break
            if i + need > len(nums):
                break
            v = nums[i:i + need]
            i += need
            ox, oy = (cur if rel else (0.0, 0.0))
            if up == "M":
                cur = (v[0] + ox, v[1] + oy)
                if first:
                    start = cur
                out.append(cur)
                up = "L"          # 2つ目以降の座標対は暗黙の L
            elif up == "L":
                cur = (v[0] + ox, v[1] + oy)
                out.append(cur)
            elif up == "H":
                cur = (v[0] + ox, cur[1])
                out.append(cur)
            elif up == "V":
                cur = (cur[0], v[0] + oy)
                out.append(cur)
            elif up in ("C", "S"):
                if up == "C":
                    c1 = (v[0] + ox, v[1] + oy)
                    c2 = (v[2] + ox, v[3] + oy)
                    end = (v[4] + ox, v[5] + oy)
                else:
                    c1 = (2 * cur[0] - prev_c[0], 2 * cur[1] - prev_c[1]) if prev_c else cur
                    c2 = (v[0] + ox, v[1] + oy)
                    end = (v[2] + ox, v[3] + oy)
                n = _curve_steps([cur, c1, c2, end], step)
                for k in range(1, n + 1):
                    t = k / n
                    u = 1 - t
                    out.append((u**3 * cur[0] + 3*u*u*t * c1[0] + 3*u*t*t * c2[0] + t**3 * end[0],
                                u**3 * cur[1] + 3*u*u*t * c1[1] + 3*u*t*t * c2[1] + t**3 * end[1]))
                prev_c, prev_q, cur = c2, None, end
            elif up in ("Q", "T"):
                if up == "Q":
                    c1 = (v[0] + ox, v[1] + oy)
                    end = (v[2] + ox, v[3] + oy)
                else:
                    c1 = (2 * cur[0] - prev_q[0], 2 * cur[1] - prev_q[1]) if prev_q else cur
                    end = (v[0] + ox, v[1] + oy)
                n = _curve_steps([cur, c1, end], step)
                for k in range(1, n + 1):
                    t = k / n
                    u = 1 - t
                    out.append((u*u * cur[0] + 2*u*t * c1[0] + t*t * end[0],
                                u*u * cur[1] + 2*u*t * c1[1] + t*t * end[1]))
                prev_q, prev_c, cur = c1, None, end
            elif up == "A":
                end = (v[5] + ox, v[6] + oy)
                out.extend(_arc_points(cur, v[0], v[1], v[2], int(v[3]), int(v[4]), end, step))
                prev_c = prev_q = None
                cur = end
            if up not in ("C", "S"):
                prev_c = None
            if up not in ("Q", "T"):
                prev_q = None
            first = False
    return out


def sample_ink(fragment: str, step: float) -> list[tuple[Point, float]]:
    """部品が描いたSVG断片から、インクの通る点を拾う。

    文字は含めない ── 名前は形の内側に置かれるもので、輪郭を広げる役目を
    持たない（含めると、箱より広い名前が輪郭を押し広げてしまう）。

    Args:
        fragment: 部品が返したSVG断片（ルートタグを含まない）。
        step: 刻む間隔。角だけを拾うと、辺の途中が輪郭から抜け落ちて、
            形が実物より膨らむ。

    Returns:
        (点, その点でインクが中心線からどれだけ外へ及ぶか) の並び。線は幅を
        持つので、中心線だけを輪郭にすると線の太さの半分だけ内側になる
        （輪の部品で実測：外周が線幅の半分ぶん小さくなる）。読めなければ空。

    Raises:
        なし。
    """
    import math
    import re
    import xml.etree.ElementTree as ET
    try:
        root = ET.fromstring(f"<g>{fragment}</g>")
    except ET.ParseError:
        return []
    pts: list[tuple[Point, float]] = []
    tr = re.compile(r"translate\(\s*(-?[\d.]+)\s*[, ]\s*(-?[\d.]+)\s*\)")
    sc = re.compile(r"scale\(\s*(-?[\d.]+)\s*\)")

    def num(el, name, default=0.0):
        try:
            return float(el.get(name, default))
        except (TypeError, ValueError):
            return default

    def walk(el, dx, dy, s):
        t = el.get("transform", "")
        m = tr.search(t)
        if m:
            dx += float(m.group(1)) * s
            dy += float(m.group(2)) * s
        m = sc.search(t)
        if m:
            s *= float(m.group(1))
        tag = el.tag.split("}")[-1]
        local: list[Point] = []
        if tag == "rect":
            x, y = num(el, "x"), num(el, "y")
            w, h = num(el, "width"), num(el, "height")
            if w and h:
                local = [(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)]
        elif tag in ("polygon", "polyline"):
            v = _numbers(el.get("points", ""))
            local = list(zip(v[0::2], v[1::2]))
            if tag == "polygon" and local:
                local.append(local[0])
        elif tag == "path":
            local = sample_path(el.get("d", ""), step)
        elif tag == "line":
            local = [(num(el, "x1"), num(el, "y1")), (num(el, "x2"), num(el, "y2"))]
        elif tag in ("circle", "ellipse"):
            cx, cy = num(el, "cx"), num(el, "cy")
            rx = num(el, "r") or num(el, "rx")
            ry = num(el, "r") or num(el, "ry")
            # 刻み幅に見合う数へ分ける。最低でも三角形にはする。
            n = max(MIN_POLYGON, int(2 * math.pi * max(rx, ry) / max(step, 1e-9)) + 1)
            local = [(cx + rx * math.cos(2 * math.pi * k / n),
                      cy + ry * math.sin(2 * math.pi * k / n)) for k in range(n + 1)]
        halo = 0.0
        if el.get("stroke") and el.get("stroke") != "none":
            halo = num(el, "stroke-width", 1.0) / 2 * s
        for px, py in (densify(local, step) if len(local) > 1 else local):
            pts.append(((px * s + dx, py * s + dy), halo))
        for ch in el:
            walk(ch, dx, dy, s)

    walk(root, 0.0, 0.0, 1.0)
    return pts


def ink_surface(fragment: str, width: float, height: float,
                fineness: int) -> list[Point]:
    """部品が描いたインクの「表面」の点を返す。

    線は幅を持つので、中心線を中心から見て外へ線幅の半分だけ押し出す。
    押し出さないと、太い線の部品では表面が線の内側になる。

    輪郭（閉じた多角形）を作らないのは、作れない形があるため。波や
    チェック記号のようにインクが角度方向に途切れる形では、輪郭の頂点を
    結んだ弦がインクの無いところをまたぐ ── そこへ辺が着くと、見た目には
    空白を指す（実測：波で13.7、記号で7.1の隔たり）。点のままなら、必ず
    実際にインクのある場所を選べる。

    Args:
        fragment: 部品が返したSVG断片。
        width: 部品が申告した幅。
        height: 部品が申告した高さ。
        fineness: 刻みの細かさ。部品の短辺をこの数で割った幅で刻む。

    Returns:
        原点(0,0)基準の点の並び。インクが拾えなければ外接矩形の縁を返す。

    Raises:
        なし。
    """
    step = max(min(width, height), 1.0) / max(fineness, 1)
    cx, cy = width / 2, height / 2
    out: list[Point] = []
    for (px, py), halo in sample_ink(fragment, step):
        if halo:
            dx, dy = px - cx, py - cy
            r = (dx * dx + dy * dy) ** 0.5
            if r > 0:
                px, py = cx + dx / r * (r + halo), cy + dy / r * (r + halo)
        out.append((px, py))
    if out:
        return out
    box = [(0.0, 0.0), (width, 0.0), (width, height), (0.0, height), (0.0, 0.0)]
    return densify(box, step)


def nearest(points: list[Point], aim: Point) -> Point:
    """狙った場所にいちばん近い点を返す。

    Args:
        points: 選ぶ対象の点。
        aim: 狙った場所。

    Returns:
        いちばん近い点。points が空なら aim をそのまま返す。

    Raises:
        なし。
    """
    if not points:
        return aim
    return min(points, key=lambda p: (p[0] - aim[0]) ** 2 + (p[1] - aim[1]) ** 2)
