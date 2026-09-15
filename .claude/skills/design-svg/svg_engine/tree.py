"""放射状の木 ── 根を中心に置き、深さを輪で表す。層状・環状に次ぐ3つ目の戦略。

層状配置は、根から多方向へ枝分かれする形を1本の帯へ引き伸ばす（枝10本で
縦横比26:1の帯になり、1枚の絵として使えない）。環状配置は正方形に収まるが、
全ての節点を1つの輪に載せるので深さを表せず、輪を横切る辺が節点を突っ切る。

この戦略は、深さごとに別の輪を割り当て、親の持つ角度の扇を子で分け合う。
だから辺は常に輪の外向きに伸び、他の節点を横切らない。

layout_graph と同じ契約（LayoutResult を返す）なので、呼び出し側は戦略を
差し替えるだけでよい。
"""
from __future__ import annotations

import math

from .geometry import shift_to_origin
from .layout_contract import LayoutResult


def _pick_root(ids: list[str], edges: list[tuple[str, str]]) -> str:
    """入ってくる辺が無い節点を根とする。無ければ出る辺が最も多いものを選ぶ。"""
    incoming = {i: 0 for i in ids}
    outgoing = {i: 0 for i in ids}
    for a, b in edges:
        if b in incoming:
            incoming[b] += 1
        if a in outgoing:
            outgoing[a] += 1
    roots = [i for i in ids if incoming[i] == 0]
    if roots:
        return max(roots, key=lambda i: outgoing[i])
    return max(ids, key=lambda i: outgoing[i])


def _spanning_tree(root: str, ids: list[str],
                   edges: list[tuple[str, str]]) -> tuple[dict[str, list[str]], dict[str, int]]:
    """幅優先で木を張る。輪になっていても、後から届いた辺は木に加えない。

    Returns:
        (親→子の並び, 節点→深さ)。根から届かない節点は最後の深さに置く。
    """
    adj: dict[str, list[str]] = {i: [] for i in ids}
    for a, b in edges:
        if a in adj and b in adj:
            adj[a].append(b)
    children: dict[str, list[str]] = {i: [] for i in ids}
    depth = {root: 0}
    queue = [root]
    while queue:
        cur = queue.pop(0)
        for nxt in adj[cur]:
            if nxt not in depth:
                depth[nxt] = depth[cur] + 1
                children[cur].append(nxt)
                queue.append(nxt)
    # 根から届かないものは、いちばん外の輪へまとめて置く
    stray = [i for i in ids if i not in depth]
    if stray:
        outer = max(depth.values()) + 1
        for i in stray:
            depth[i] = outer
            children[root].append(i)
    return children, depth


def _leaves(node: str, children: dict[str, list[str]]) -> int:
    """その節点がぶら下げる葉の数。扇の広さを分けるのに使う。"""
    if not children[node]:
        return 1
    return sum(_leaves(c, children) for c in children[node])


def layout_tree(node_sizes: dict[str, tuple[float, float]],
                edges: list[tuple[str, str]],
                gap_rank: float, gap_order: float,
                direction: str = "TB",
                **_ignored) -> LayoutResult:
    """根を中心に、深さごとの輪へ節点を置く。

    輪の半径も扇の広さも、その深さに実際に居る節点の大きさと数から決める。
    決め打ちの半径・決め打ちの角度を持たない。

    Args:
        node_sizes: 節点idごとの (width, height)。
        edges: (from, to) の並び。輪になっていてもよい（木に入らない辺は
            そのまま直線で描かれる）。
        gap_rank: 輪と輪の間に空ける最小の隙間。
        gap_order: 同じ輪の上で隣り合う節点の間に空ける最小の隙間。
        direction: 受け取るが使わない（放射状に上下左右は無い）。

    Returns:
        LayoutResult。edge_paths は節点の中心どうしを結ぶ2点。実際の
        接続点は、呼び出し側が部品の輪郭から決める。

    Raises:
        なし。
    """
    ids = list(node_sizes)
    if not ids:
        return LayoutResult({}, {}, 0.0, 0.0)
    if len(ids) == 1:
        w, h = node_sizes[ids[0]]
        return LayoutResult({ids[0]: (0.0, 0.0)}, {}, w, h)

    root = _pick_root(ids, edges)
    children, depth = _spanning_tree(root, ids, edges)
    max_depth = max(depth.values())

    # 深さごとの輪の半径。1つ前の輪から、両側の節点の張り出しと隙間だけ離す。
    radius: dict[int, float] = {0: 0.0}
    for d in range(1, max_depth + 1):
        here = [k for k in ids if depth[k] == d]
        prev = [k for k in ids if depth[k] == d - 1]
        out_prev = max((max(node_sizes[k]) for k in prev), default=0.0) / 2
        out_here = max((max(node_sizes[k]) for k in here), default=0.0) / 2
        step = out_prev + out_here + gap_rank
        # その輪に並ぶ節点が触れ合わないだけの周長も要る
        need = sum(max(node_sizes[k]) + gap_order for k in here)
        radius[d] = max(radius[d - 1] + step, need / (2 * math.pi))

    centres: dict[str, tuple[float, float]] = {root: (0.0, 0.0)}

    def place(node: str, a0: float, a1: float) -> None:
        kids = children[node]
        if not kids:
            return
        total = sum(_leaves(c, children) for c in kids)
        cur = a0
        for c in kids:
            share = (a1 - a0) * _leaves(c, children) / total
            mid = cur + share / 2
            r = radius[depth[c]]
            centres[c] = (r * math.cos(mid), r * math.sin(mid))
            place(c, cur, cur + share)
            cur += share

    place(root, -math.pi / 2, -math.pi / 2 + 2 * math.pi)

    positions = {k: (cx - node_sizes[k][0] / 2, cy - node_sizes[k][1] / 2)
                 for k, (cx, cy) in centres.items()}
    positions, (min_x, min_y) = shift_to_origin(positions)
    centres = {k: (x - min_x, y - min_y) for k, (x, y) in centres.items()}

    width = max(positions[k][0] + node_sizes[k][0] for k in positions)
    height = max(positions[k][1] + node_sizes[k][1] for k in positions)

    edge_paths = {i: [centres[a], centres[b]]
                  for i, (a, b) in enumerate(edges)
                  if a in centres and b in centres}
    return LayoutResult(positions, edge_paths, width, height)
