"""多角形のブーリアン演算 ── 和・積・差。Greiner-Hormannのアルゴリズム。

SVGにはブーリアン演算が無いので、Illustratorの「型抜き」に相当する結果を
座標そのものを解いて作る。曲線は扱わず、多角形(点の並び)どうしの演算に限る
── 円・自由曲線を扱いたい場合は、先に十分な数の点へ分解(サンプリング)する。

退化した場合(交点が無く、一方が他方を完全に包含する／完全に離れている)は、
単純な内包判定で処理する。片方がもう片方の内側に完全に収まる差(A-Bで
Bの方が小さい)は、外周と内周の2つの輪郭を返す。描く側がこの2つを1つの
パスの中の別々の輪郭として置き、偶奇規則で塗ると、内側が穴になる。
"""
from __future__ import annotations

Point = tuple[float, float]

# 円を多角形で近似するときの既定の分割数。呼ぶ側が細かさを持っているなら渡す。
_CIRCLE_FACETS = 48


class _V:
    __slots__ = ("x", "y", "next", "prev", "intersect", "entry", "neighbor", "alpha", "visited")

    def __init__(self, x: float, y: float, intersect: bool = False, alpha: float = 0.0):
        self.x, self.y = x, y
        # 頂点は常に環の中にいる。最初は自分ひとりの環で、_build が繋ぎ直す
        # ── None を許すと、繋ぐ前に触れる経路が型の上で残ってしまう。
        self.next: "_V" = self
        self.prev: "_V" = self
        self.intersect = intersect
        self.entry = True
        self.neighbor: "_V | None" = None
        self.alpha = alpha
        self.visited = False


def _point_in_polygon(pt: Point, poly: list[Point]) -> bool:
    x, y = pt
    inside = False
    n = len(poly)
    for i in range(n):
        x1, y1 = poly[i]
        x2, y2 = poly[(i + 1) % n]
        if (y1 > y) != (y2 > y):
            x_at_y = (x2 - x1) * (y - y1) / (y2 - y1) + x1
            if x < x_at_y:
                inside = not inside
    return inside


def _seg_intersect(p1: Point, p2: Point, q1: Point, q2: Point):
    """線分p1-p2とq1-q2が、両方とも端点を除く内部で交わるなら
    (pに沿った割合, qに沿った割合, x, y) を返す。交わらないならNone。
    """
    x1, y1 = p1
    x2, y2 = p2
    x3, y3 = q1
    x4, y4 = q2
    d = (x2 - x1) * (y4 - y3) - (y2 - y1) * (x4 - x3)
    if abs(d) < 1e-12:
        return None
    a = ((x3 - x1) * (y4 - y3) - (y3 - y1) * (x4 - x3)) / d
    b = ((x3 - x1) * (y2 - y1) - (y3 - y1) * (x2 - x1)) / d
    if 1e-9 < a < 1 - 1e-9 and 1e-9 < b < 1 - 1e-9:
        return (a, b, x1 + a * (x2 - x1), y1 + a * (y2 - y1))
    return None


def _build(points: list[Point]) -> list[_V]:
    verts = [_V(x, y) for x, y in points]
    n = len(verts)
    for i in range(n):
        verts[i].next = verts[(i + 1) % n]
        verts[i].prev = verts[(i - 1) % n]
    return verts


def _insert(vertex: _V, s1: _V, s2: _V) -> None:
    """s1→s2の辺の上に、既に挿入済みの交点も含めてalpha順に挿む。"""
    cur = s1
    while cur.next is not s2 and cur.next.alpha < vertex.alpha:
        cur = cur.next
    vertex.next = cur.next
    vertex.prev = cur
    cur.next.prev = vertex
    cur.next = vertex


def _mark_entries(head: _V, other_poly: list[Point], start_inside: bool) -> None:
    entry = not start_inside
    v = head
    while True:
        if v.intersect and not v.visited:
            v.entry = entry
            entry = not entry
        v = v.next
        if v is head:
            break


def _trace(start_candidates: list[_V], want_entry_first: bool) -> list[list[Point]]:
    polygons: list[list[Point]] = []
    for start in start_candidates:
        if start.visited:
            continue
        poly: list[Point] = []
        current = start
        while True:
            current.visited = True
            poly.append((current.x, current.y))
            forward = current.entry
            current = current.next if forward else current.prev
            while not current.intersect:
                poly.append((current.x, current.y))
                current = current.next if forward else current.prev
            current.visited = True
            # 交点の頂点は必ず相方を持つ（_clip が対で結ぶ）
            assert current.neighbor is not None
            current = current.neighbor
            if current is start or current is None:
                break
        if len(poly) >= 3:
            polygons.append(poly)
    return polygons


def _do_clip(subject: list[Point], clip: list[Point],
             invert_subject: bool, invert_clip: bool) -> list[list[Point]]:
    subj_v = _build(subject)
    clip_v = _build(clip)
    ns, nc = len(subj_v), len(clip_v)

    found_any = False
    for i in range(ns):
        s1, s2 = subj_v[i], subj_v[(i + 1) % ns]
        for j in range(nc):
            c1, c2 = clip_v[j], clip_v[(j + 1) % nc]
            hit = _seg_intersect((s1.x, s1.y), (s2.x, s2.y), (c1.x, c1.y), (c2.x, c2.y))
            if hit:
                found_any = True
                a, b, ix, iy = hit
                iv_s = _V(ix, iy, intersect=True, alpha=a)
                iv_c = _V(ix, iy, intersect=True, alpha=b)
                iv_s.neighbor = iv_c
                iv_c.neighbor = iv_s
                _insert(iv_s, s1, s2)
                _insert(iv_c, c1, c2)

    if not found_any:
        return _degenerate(subject, clip, invert_subject)

    subj_inside_start = _point_in_polygon((subj_v[0].x, subj_v[0].y), clip)
    clip_inside_start = _point_in_polygon((clip_v[0].x, clip_v[0].y), subject)
    if invert_subject:
        subj_inside_start = not subj_inside_start
    if invert_clip:
        clip_inside_start = not clip_inside_start
    _mark_entries(subj_v[0], clip, subj_inside_start)
    _mark_entries(clip_v[0], subject, clip_inside_start)

    subj_intersections = [v for v in _walk(subj_v[0]) if v.intersect]
    return _trace(subj_intersections, True)


def _walk(head: _V):
    v = head
    while True:
        yield v
        v = v.next
        if v is head:
            break


def _degenerate(subject: list[Point], clip: list[Point], invert: bool) -> list[list[Point]]:
    subj_in_clip = _point_in_polygon(subject[0], clip)
    clip_in_subj = _point_in_polygon(clip[0], subject)
    if invert:  # 和(union)
        if subj_in_clip:
            return [clip]
        if clip_in_subj:
            return [subject]
        return [subject, clip]  # 離れている
    # 積(intersect)。差(subtract)は呼び出し側がclipを反転させて渡す。
    if subj_in_clip:
        return [subject]
    if clip_in_subj:
        return [clip]
    return []


def _degenerate_subtract(subject: list[Point], clip: list[Point]) -> list[list[Point]]:
    """交点が無いときの差。包まれる側があるなら穴として返す。

    輪郭を1つしか返せないと『穴あき』が表現できないが、外周と内周の2つを
    返し、塗り分けを偶奇規則にすれば穴になる。
    """
    if _point_in_polygon(clip[0], subject):
        # clip が subject の内側 → 外周＋内周（向きを逆にして穴にする）
        return [subject, list(reversed(clip))]
    if _point_in_polygon(subject[0], clip):
        return []          # subject が丸ごと削られる
    return [subject]       # 離れている


def _has_crossing(a: list[Point], b: list[Point]) -> bool:
    """2つの多角形の辺が交わるか。"""
    for i in range(len(a)):
        p1, p2 = a[i], a[(i + 1) % len(a)]
        for j in range(len(b)):
            q1, q2 = b[j], b[(j + 1) % len(b)]
            if _seg_intersect(p1, p2, q1, q2):
                return True
    return False


def boolean_op(shapes: list[list[Point]], op: str) -> list[list[Point]]:
    """3つ以上の形も、先頭から順に演算を畳み込んで処理する。

    Args:
        shapes: 多角形(点の並び)を2つ以上。
        op: "union" | "intersect" | "subtract"。

    Returns:
        結果の多角形の並び(複数になることがある。例: 離れた形の和)。

    Raises:
        ValueError: opが未知の値、またはshapesが2つ未満のとき。
    """
    if len(shapes) < 2:
        raise ValueError("boolean_opにはshapesを2つ以上渡すこと")
    acc = [shapes[0]]
    for nxt in shapes[1:]:
        merged: list[list[Point]] = []
        for a in acc:
            if op == "union":
                merged.extend(_do_clip(a, nxt, invert_subject=True, invert_clip=True))
            elif op == "intersect":
                merged.extend(_do_clip(a, nxt, invert_subject=False, invert_clip=False))
            elif op == "subtract":
                if not _has_crossing(a, nxt):
                    merged.extend(_degenerate_subtract(a, nxt))
                else:
                    merged.extend(_do_clip(a, nxt, invert_subject=False, invert_clip=True))
            else:
                raise ValueError(f"知らない演算です: {op}")
        acc = merged
        if not acc:
            break
    return acc


def circle_polygon(cx: float, cy: float, r: float, n: int = _CIRCLE_FACETS) -> list[Point]:
    import math
    return [(cx + r * math.cos(2 * math.pi * i / n), cy + r * math.sin(2 * math.pi * i / n))
            for i in range(n)]


def rect_polygon(x: float, y: float, w: float, h: float) -> list[Point]:
    return [(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
