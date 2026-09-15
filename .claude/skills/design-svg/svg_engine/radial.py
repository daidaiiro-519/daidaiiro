"""環状配置 ── 節点を輪の上へ等間隔に置く。層状配置とは別の戦略。

並びが閉じていることを見せたい図で使う。層状配置で縦一列に並べて
最後から最初へ長い辺を戻すと、閉じていることが図から読めない。輪に
置けば、閉じていることが位置そのもので示される。

ただし輪に置くだけでは足りない。輪の上の並び順を辺から決めないと、
宣言の順しだいで隣り合うべき節点が輪の反対側へ行き、絵はもつれた星に
なる（幾何的な崩れは出ないので、検査は通ってしまう）。だから並び順は
辺をたどって決める。

たどりきれない形（1つの節点から3方向以上へ分かれる木など）は、この
やり方では描けない。黙って歪んだ絵を返さず、戦略の側から申告する。

layout_graph と同じ契約（LayoutResult を返す）なので、呼び出し側は
戦略を差し替えるだけでよい。
"""
from __future__ import annotations

import math

from .geometry import segment_hits_rect, shift_to_origin
from .layout_contract import LayoutResult, UnsupportedByStrategy


def _ring_order(ids: list[str], edges: list[tuple[str, str]]) -> list[str]:
    """辺をたどって輪の上の並び順を決める。

    隣り合う節点が輪の上でも隣り合うようにする。分岐や行き止まりに
    当たったら、まだ置いていないものの中から辺の少ないものを次に採る
    （辺の多い節点を後回しにすると、その節点の辺が余計に弦になる）。

    Args:
        ids: 節点idの並び。
        edges: (from, to) の並び。向きは輪の上では区別しない。

    Returns:
        輪に置く順に並べた節点idの並び。
    """
    adj: dict[str, set[str]] = {i: set() for i in ids}
    for a, b in edges:
        if a in adj and b in adj and a != b:
            adj[a].add(b)
            adj[b].add(a)
    remaining = set(ids)
    # 行き止まり（辺が1本だけ）があればそこから始める。輪なら始点はどこでもよい。
    start = min(ids, key=lambda i: (len(adj[i]) != 1, len(adj[i]), ids.index(i)))
    order = [start]
    remaining.discard(start)
    while remaining:
        cur = order[-1]
        nxt = [k for k in adj[cur] if k in remaining]
        if nxt:
            pick = min(nxt, key=lambda k: (len(adj[k]), ids.index(k)))
        else:
            pick = min(remaining, key=lambda k: (len(adj[k]), ids.index(k)))
        order.append(pick)
        remaining.discard(pick)
    return order


def layout_radial(node_sizes: dict[str, tuple[float, float]],
                   edges: list[tuple[str, str]],
                   gap_rank: float, gap_order: float,
                   direction: str = "TB",
                   **_ignored) -> LayoutResult:
    """節点を輪の上へ等間隔に置く。

    輪の半径は、節点の大きさと個数から決める ── 節点が大きいほど、数が
    多いほど輪は大きくなる。決め打ちの半径を持たない。

    Args:
        node_sizes: 節点idごとの (width, height)。
        edges: (from, to) の並び。輪の上の並び順はここからたどって決める
            （宣言の並びは使わない）。
        gap_rank: 節点どうしの間に空ける最小の隙間。
        gap_order: 同上（環状では区別しないので大きい方を使う）。
        direction: 受け取るが使わない（環状に上下左右は無い）。

    Returns:
        LayoutResult。edge_paths は節点の中心どうしを結ぶ2点。実際の
        接続点は、呼び出し側が部品の輪郭から決める。

    Raises:
        UnsupportedByStrategy: 輪の内側を横切る辺が、他の節点の箱を
            突っ切ってしまうとき。輪に並べきれない形（1つの節点から
            3方向以上へ分かれる木など）がこれに当たる。
    """
    ids = list(node_sizes)
    n = len(ids)
    if n == 0:
        return LayoutResult({}, {}, 0.0, 0.0)
    if n == 1:
        w, h = node_sizes[ids[0]]
        return LayoutResult({ids[0]: (0.0, 0.0)}, {}, w, h)

    gap = max(gap_rank, gap_order)
    # 輪の周長は、各節点が占める幅と隙間の合計を下回れない。
    # そこから半径を出す（決め打ちの半径を置かない）。
    span = sum(max(w, h) for w, h in node_sizes.values()) + gap * n
    radius = span / (2 * math.pi)
    # 隣り合う節点が重ならない半径も別途要る（少数のときはこちらが効く）。
    biggest = max(max(w, h) for w, h in node_sizes.values())
    chord = biggest + gap
    radius = max(radius, chord / (2 * math.sin(math.pi / n)))

    positions: dict[str, tuple[float, float]] = {}
    centres: dict[str, tuple[float, float]] = {}
    ring = _ring_order(ids, edges)
    for i, nid in enumerate(ring):
        ang = -math.pi / 2 + 2 * math.pi * i / n   # 真上から時計回り
        cx = radius * math.cos(ang)
        cy = radius * math.sin(ang)
        w, h = node_sizes[nid]
        centres[nid] = (cx, cy)
        positions[nid] = (cx - w / 2, cy - h / 2)

    # 原点を左上へ寄せる
    positions, (min_x, min_y) = shift_to_origin(positions)
    centres = {k: (x - min_x, y - min_y) for k, (x, y) in centres.items()}

    width = max(positions[k][0] + node_sizes[k][0] for k in ids)
    height = max(positions[k][1] + node_sizes[k][1] for k in ids)

    # 輪の内側を横切る辺が、当事者でない節点の箱を突っ切っていないか。
    # 突っ切るなら、この置き方では描けない ── 黙って歪んだ絵を返さず申告する。
    # 閾値は置かない。箱そのものと交わるかを見る。
    boxes = {k: (positions[k][0], positions[k][1],
                 positions[k][0] + node_sizes[k][0],
                 positions[k][1] + node_sizes[k][1]) for k in ids}
    for a, b in edges:
        if a not in centres or b not in centres or a == b:
            continue
        for k in ids:
            if k in (a, b):
                continue
            if segment_hits_rect(centres[a], centres[b], boxes[k], 0.0):
                raise UnsupportedByStrategy(
                    f"環状配置では描けません: 辺 {a}→{b} が節点 {k} を突っ切ります。"
                    "輪の上に並べきれない形（1つの節点から3方向以上へ分かれる等）です。"
                    "layout_tree（放射状の木）か layout_graph（層状）を使ってください。")

    edge_paths = {i: [centres[a], centres[b]]
                  for i, (a, b) in enumerate(edges)
                  if a in centres and b in centres}
    return LayoutResult(positions, edge_paths, width, height)
