"""描いた結果の機械検査 ── 崩れているかどうかを、絵を見ずに判定する。

アセット追加の契約が「この検査を通ること」を求めている以上、検査は例に
置く道具ではなくエンジン本体の持ち物である。新しい部品を足した人は、
自分の部品を含む図をこの関数へ通せる。

見るのは3種類。
- check() ── 文字どうしの重なりと、画布の外へのはみ出し
- check_shapes() ── 箱どうしの重なり、辺が箱を突っ切ること、囲みの中途半端な交差
- check_attachment() ── 辺の終端が、相手のインクに着いているか

**共有してよいのは「SVGを読むこと」だけ。** 描く側と検査が同じ *判断* を
共有すると、その判断の誤りを検査が永久に見つけられない。だから
「輪郭をどう決めるか」「どこへ着けるか」（geometry.outline_of /
star_outline、compose の接続点の決め方）には一切触れない。読み取り
（path を点列にする・折れ線を刻む）は判断ではないので共有する ──
ここを二重に持つと、片方だけが新しいコマンドに対応して検査がすり抜ける。

閾値を定数で置かない。標本の粗さも内側と見なす余裕も、その図自身の
寸法（最小の箱の短辺・その辺の線幅）から毎回導く。定数で置くと、
倍率を変えただけで検査が効かなくなる。
"""
from __future__ import annotations

import re

from .geometry import densify as _densify_shared, sample_path as _sample_path_shared
from .text import text_width
from .geometry import MIN_POLYGON, SAMPLE_DIVISOR

# 曲線を刻まず、命令の区切りの点だけを拾わせるための刻み幅。実際の座標より
# 十分大きければ何でもよく、大きさそのものに意味は無い。
_VERTICES_ONLY = 1e9
from .tokens import DEFAULT_THEME, num

# 字面の高さと下ばね。書体の性質であって検査の判断ではないので、描く側と
# 同じ出どころ（トークン）から引く ── 別々に持つと、片方だけ直したときに
# 検査が古い値で測り続ける。
# 幾何の計算（刻み・重なり判定）は、あえて共有しない。描く側と同じコードで
# 測ると、その計算の誤りを検査が永久に見つけられなくなるため。
CAP = num(DEFAULT_THEME, "font.cap-ratio")
DESC = num(DEFAULT_THEME, "font.descender-ratio")
_TRANSLATE = re.compile(r"translate\(\s*(-?[\d.]+)\s*[, ]\s*(-?[\d.]+)\s*\)")
_SCALE = re.compile(r"scale\(\s*(-?[\d.]+)\s*\)")


def _text_boxes(svg: str) -> list[tuple[float, float, float, float, str]]:
    """文字の外接矩形を、祖先の変換を積んだ絶対座標で返す。

    節点は <g transform="translate(...)"> で包まれるので、変換を無視して生の
    座標を読むと、全部が原点付近に居ることになって偽の重なりを大量に出す
    （最初の実装がそうなっていた）。祖先を辿って積む。
    """
    import xml.etree.ElementTree as ET
    try:
        root = ET.fromstring(svg)
    except ET.ParseError:
        return []
    out: list[tuple[float, float, float, float, str]] = []

    def walk(el, dx: float, dy: float, sc: float):
        t = el.get("transform", "")
        m = _TRANSLATE.search(t)
        if m:
            dx += float(m.group(1)) * sc
            dy += float(m.group(2)) * sc
        m = _SCALE.search(t)
        if m:
            sc *= float(m.group(1))
        if el.tag.endswith("text"):
            content = "".join(el.itertext()).strip()
            if content and "rotate" not in t:
                try:
                    x = float(el.get("x", 0)) * sc + dx
                    y = float(el.get("y", 0)) * sc + dy
                    fs = float(el.get("font-size", DEFAULT_THEME["font.size"])) * sc
                except ValueError:
                    return
                w = text_width(content, fs)
                anchor = el.get("text-anchor", "start")
                x0 = x - w / 2 if anchor == "middle" else (x - w if anchor == "end" else x)
                out.append((x0, y - fs * CAP, x0 + w, y + fs * DESC, content))
        for ch in el:
            walk(ch, dx, dy, sc)

    walk(root, 0.0, 0.0, 1.0)
    return out


def _shapes(svg: str):
    """箱・囲み・辺を、祖先の変換を積んだ絶対座標で拾う。

    Returns:
        (節点の箱の並び, 囲みの箱の並び, 辺の点列の並び)。
    """
    import xml.etree.ElementTree as ET
    try:
        root = ET.fromstring(svg)
    except ET.ParseError:
        return [], [], []
    boxes, frames, paths = [], [], []

    def walk(el, dx, dy, sc, in_box):
        t = el.get("transform", "")
        m = _TRANSLATE.search(t)
        if m:
            dx += float(m.group(1)) * sc
            dy += float(m.group(2)) * sc
        m = _SCALE.search(t)
        if m:
            sc *= float(m.group(1))
        cls = el.get("class", "")
        here_box = in_box or "svg-box" in cls
        tag = el.tag.split("}")[-1]
        if tag == "rect":
            try:
                x = float(el.get("x", 0)) * sc + dx
                y = float(el.get("y", 0)) * sc + dy
                w = float(el.get("width", 0)) * sc
                h = float(el.get("height", 0)) * sc
            except ValueError:
                w = 0
            if w:
                if here_box:
                    boxes.append((x, y, x + w, y + h))
                elif el.get("stroke-dasharray") and el.get("fill") == "none":
                    try:
                        fsw = float(el.get("stroke-width", 1)) * sc
                    except ValueError:
                        fsw = 1.0
                    frames.append((x, y, x + w, y + h, fsw))
        if tag == "path" and el.get("fill") == "none" and el.get("d"):
            pts = _sample_path(el.get("d"))
            try:
                sw = float(el.get("stroke-width", 1)) * sc
            except ValueError:
                sw = 1.0
            paths.append(([(px * sc + dx, py * sc + dy) for px, py in pts], sw))
        for ch in el:
            walk(ch, dx, dy, sc, here_box)

    walk(root, 0.0, 0.0, 1.0, False)
    return boxes, frames, paths


# 読み取りの細かさは、対象そのものの大きさから決める。決め打ちの分割数だと、
# 大きな図ほど粗くなり、細かい崩れを跳び越す。分けの細かさは、描く側が輪郭を
# 何向きで表すかと同じ尺度に合わせる（別の尺度を持つと、描く側が細かくした
# ときに検査だけが粗いまま残る）。
_FINENESS = int(num(DEFAULT_THEME, "size.outline-facets"))


def _step_for(points) -> float:
    """その図形自身の広がりから、刻み幅を決める。"""
    if not points:
        return 1.0
    xs = [x for x, _ in points]
    ys = [y for _, y in points]
    diag = ((max(xs) - min(xs)) ** 2 + (max(ys) - min(ys)) ** 2) ** 0.5
    return max(diag, 1.0) / _FINENESS


def _sample_path(d: str):
    """path を点列にする。刻み幅はその path 自身の広がりから決める。"""
    rough = _sample_path_shared(d, _VERTICES_ONLY)   # まず頂点だけ拾って広がりを知る
    return _sample_path_shared(d, _step_for(rough))


def _densify(pts, step):
    """折れ線の頂点だけでなく、線分の途中も標本化する。

    頂点だけを見ると、両端が箱の外にある直線が箱を貫いていても検出できない
    （最初の実装がそうで、わざと壊した例を素通りさせた）。
    """
    return _densify_shared(pts, step)


def _inside(pt, box, margin):
    x, y = pt
    return (box[0] + margin < x < box[2] - margin
            and box[1] + margin < y < box[3] - margin)


def check_shapes(svg: str) -> list[str]:
    """線と箱の関係を見る。閾値は置かず、すべて図から決める。

    - 標本の粗さ ── 最小の箱の短辺の1/4。これより粗いと箱を跳び越して見逃す
    - 内側と見なす余裕 ── その辺自身の線幅。線が箱の縁に触れるのは正常
    - 端点の除外 ── 位置ではなく、始点・終点が接している箱そのものを除く
    """
    faults = []
    boxes, frames, paths = _shapes(svg)
    for i in range(len(boxes)):
        for j in range(i + 1, len(boxes)):
            a, b = boxes[i], boxes[j]
            if a[0] < b[2] and a[2] > b[0] and a[1] < b[3] and a[3] > b[1]:
                faults.append("箱どうしが重なる")
    if boxes:
        step = min(min(b[2] - b[0], b[3] - b[1]) for b in boxes) / SAMPLE_DIVISOR
        step = max(step, 0.5)
        for pts, sw in paths:
            if len(pts) < 2:
                continue
            dense = _densify(pts, step)
            # 始点・終点が触れている箱は、その辺の相手なので除く（位置で切らない）
            ends = [pts[0], pts[-1]]
            for bx in boxes:
                touching = any(bx[0] - sw <= x <= bx[2] + sw and bx[1] - sw <= y <= bx[3] + sw
                               for x, y in ends)
                if touching:
                    continue
                if any(_inside(p, bx, sw) for p in dense):
                    faults.append("辺が箱を突っ切る")
                    break
    for i in range(len(frames)):
        for j in range(i + 1, len(frames)):
            a, b = frames[i], frames[j]
            hit = a[0] < b[2] and a[2] > b[0] and a[1] < b[3] and a[3] > b[1]
            nest = (a[0] <= b[0] and a[1] <= b[1] and a[2] >= b[2] and a[3] >= b[3]) or \
                   (b[0] <= a[0] and b[1] <= a[1] and b[2] >= a[2] and b[3] >= a[3])
            if hit and not nest:
                faults.append("囲みが中途半端に交差する")
    # 囲みの線が中身へ食い込んでいないか。囲むとは「間を空けて外側を回る」ことで、
    # 縁に乗るのは囲めていない。空けるべき量はその囲み自身の線幅から決める
    # （定数を置くと、倍率を変えた瞬間に効かなくなる）。
    for fx0, fy0, fx1, fy1, fsw in frames:
        for bx0, by0, bx1, by1 in boxes:
            overlaps = fx0 < bx1 and fx1 > bx0 and fy0 < by1 and fy1 > by0
            if not overlaps:
                continue
            clear = fsw / 2
            if (bx0 < fx0 + clear or by0 < fy0 + clear
                    or bx1 > fx1 - clear or by1 > fy1 - clear):
                faults.append("囲みの線が中身に重なる")
                break
    return faults


def _overlaps(a, b) -> bool:
    return a[0] < b[2] and a[2] > b[0] and a[1] < b[3] and a[3] > b[1]


def check(svg: str) -> list[str]:
    """崩れを列挙する。空なら崩れ無し。"""
    faults = []
    m = re.search(r'viewBox="0 0 ([\d.]+) ([\d.]+)"', svg)
    if not m:
        return ["viewBoxが無い"]
    vw, vh = float(m.group(1)), float(m.group(2))
    boxes = _text_boxes(svg)
    for i in range(len(boxes)):
        bx = boxes[i]
        if bx[0] < -1 or bx[1] < -1 or bx[2] > vw + 1 or bx[3] > vh + 1:
            faults.append(f"画布の外: {bx[4]!r}")
        for j in range(i + 1, len(boxes)):
            if _overlaps(bx, boxes[j]):
                faults.append(f"文字が重なる: {bx[4]!r} × {boxes[j][4]!r}")
    return faults


def _point_to_segment(p, a, b) -> float:
    """点から線分までの距離。線分の外側なら端点までの距離になる。"""
    px, py = p
    ax, ay = a
    bx, by = b
    ex, ey = bx - ax, by - ay
    length2 = ex * ex + ey * ey
    if length2 <= 0:
        return ((px - ax) ** 2 + (py - ay) ** 2) ** 0.5
    t = max(0.0, min(1.0, ((px - ax) * ex + (py - ay) * ey) / length2))
    qx, qy = ax + ex * t, ay + ey * t
    return ((px - qx) ** 2 + (py - qy) ** 2) ** 0.5


def _node_ink(root, fineness: int):
    """節点ごとに、その節点が実際に置いたインクの点を集める。

    節点の印（class="wf-node"）が付いた群の中だけを見る。位置の付け方から
    推測すると、囲みや辺と混ざる。

    返すのは点ではなく**線分の並び**。点までの距離で測ると、標本と標本の
    あいだに落ちた終端が、標本の粗さのぶんだけ「離れている」と出る（実測：
    着いているのに1.5〜1.7の隔たりと報告した）。線分までの距離なら、
    標本の粗さが結果に出ない。

    線分には、その線が中心線から外へ及ぶ量を添える。線は幅を持つので、
    中心線までの隔たりで測ると太い線ほど「着いていない」と誤って言う
    （輪の部品で実測：線幅24の輪で、着いているのに隔たり11.6と出た）。
    これはSVGの読み方であって描き方の判断ではないので、描く側と同じ事実を
    使ってよい。
    """
    groups: list[list[tuple[tuple[float, float], tuple[float, float], float]]] = []

    def drawable(el, dx, dy, sc, into):
        t = el.get("transform", "")
        m = _TRANSLATE.search(t)
        if m:
            dx += float(m.group(1)) * sc
            dy += float(m.group(2)) * sc
        m = _SCALE.search(t)
        if m:
            sc *= float(m.group(1))
        tag = el.tag.split("}")[-1]
        local: list[tuple[float, float]] = []
        try:
            if tag == "rect":
                x, y = float(el.get("x", 0)), float(el.get("y", 0))
                w, h = float(el.get("width", 0)), float(el.get("height", 0))
                if w and h:
                    local = [(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)]
            elif tag in ("polygon", "polyline"):
                v = [float(n) for n in re.findall(r"-?[\d.]+", el.get("points", ""))]
                local = list(zip(v[0::2], v[1::2]))
            elif tag == "path" and el.get("d"):
                local = _sample_path(el.get("d"))
            elif tag == "circle":
                cx, cy, r = (float(el.get("cx", 0)), float(el.get("cy", 0)),
                             float(el.get("r", 0)))
                import math
                n = max(MIN_POLYGON, int(2 * math.pi * fineness))
                local = [(cx + r * math.cos(2 * math.pi * k / n),
                          cy + r * math.sin(2 * math.pi * k / n)) for k in range(n)]
            elif tag.endswith("text"):
                content = "".join(el.itertext()).strip()
                if content:
                    x, y = float(el.get("x", 0)), float(el.get("y", 0))
                    fs = float(el.get("font-size", DEFAULT_THEME["font.size"]))
                    w = text_width(content, fs)
                    anchor = el.get("text-anchor", "start")
                    x0 = x - w / 2 if anchor == "middle" else (x - w if anchor == "end" else x)
                    top, bot = y - fs * CAP, y + fs * DESC
                    local = [(x0, top), (x0 + w, top), (x0 + w, bot), (x0, bot), (x0, top)]
        except ValueError:
            local = []
        if local:
            halo = 0.0
            if el.get("stroke") and el.get("stroke") != "none":
                try:
                    halo = float(el.get("stroke-width", 1)) / 2 * sc
                except ValueError:
                    halo = 0.0
            moved = [(px * sc + dx, py * sc + dy) for px, py in local]
            if len(moved) == 1:
                into.append((moved[0], moved[0], halo))
            for a, b in zip(moved, moved[1:]):
                into.append((a, b, halo))
        for ch in el:
            drawable(ch, dx, dy, sc, into)

    def find(el, dx, dy, sc):
        t = el.get("transform", "")
        m = _TRANSLATE.search(t)
        if m:
            dx += float(m.group(1)) * sc
            dy += float(m.group(2)) * sc
        m = _SCALE.search(t)
        if m:
            sc *= float(m.group(1))
        if "wf-node" in (el.get("class") or ""):
            acc: list[tuple[tuple[float, float], tuple[float, float], float]] = []
            for ch in el:
                drawable(ch, dx, dy, sc, acc)
            groups.append(acc)
            return
        for ch in el:
            find(ch, dx, dy, sc)

    find(root, 0.0, 0.0, 1.0)
    return groups


def check_attachment(svg: str) -> list[str]:
    """辺の終端が、どこかの節点のインクに着いているかを見る。

    辺は必ず何かと何かを結ぶ。終端が節点のインクから離れていれば、その辺は
    空白を指している ── 外接矩形の縁に着いた辺は、形が矩形でない部品（輪・
    波・三日月）や、インクが一部にしか無い図（円グラフ・棒グラフ）で、必ず
    ここに引っかかる。

    離れてよい量は、その辺自身の線幅から決める。定数を置くと、倍率を変えた
    瞬間に効かなくなる。

    Args:
        svg: 完成したSVG文字列。

    Returns:
        崩れの説明の並び。空なら崩れ無し。

    Raises:
        なし。
    """
    import xml.etree.ElementTree as ET
    try:
        root = ET.fromstring(svg)
    except ET.ParseError:
        return ["SVGとして読めない"]
    inks = _node_ink(root, _FINENESS)
    if not inks:
        return []
    all_ink = [p for g in inks for p in g]

    faults = []
    seen = set()

    def walk(el, dx, dy, sc, inside_node):
        t = el.get("transform", "")
        m = _TRANSLATE.search(t)
        if m:
            dx += float(m.group(1)) * sc
            dy += float(m.group(2)) * sc
        m = _SCALE.search(t)
        if m:
            sc *= float(m.group(1))
        inside_node = inside_node or "wf-node" in (el.get("class") or "")
        tag = el.tag.split("}")[-1]
        if (not inside_node and tag == "path" and el.get("fill") == "none"
                and el.get("d")):
            pts = [(x * sc + dx, y * sc + dy) for x, y in _sample_path(el.get("d"))]
            if len(pts) >= 2:
                try:
                    tol = float(el.get("stroke-width", 1)) * sc
                except ValueError:
                    tol = 1.0
                for end in (pts[0], pts[-1]):
                    gap = min(_point_to_segment(end, a, b) - halo
                              for a, b, halo in all_ink)
                    if gap > tol:
                        key = (round(end[0], 1), round(end[1], 1))
                        if key not in seen:
                            seen.add(key)
                            faults.append(
                                f"辺の終端が何にも着いていない: "
                                f"({end[0]:.1f},{end[1]:.1f}) から最も近いインクまで {gap:.1f}"
                                f"（許される隔たり {tol:.1f}）")
        for ch in el:
            walk(ch, dx, dy, sc, inside_node)

    walk(root, 0.0, 0.0, 1.0, False)
    return faults
