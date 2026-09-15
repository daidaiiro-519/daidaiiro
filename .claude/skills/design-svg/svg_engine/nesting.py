"""群を含む配置 ── 群を再帰的に解き、親は群を1個の節点として扱う。

段ごとに要素を隣り合わせるだけでは足りない。群が段をまたぐと、斜めに離れた
要素の外接矩形が間の非メンバーまで飲み込む（実測で確認）。囲みが意味を成すには
「群の要素だけが、群の矩形の内側に居る」ことが要る。それを構造として保証するには、
群を先に配置して1つの大きさへ畳み、親はそれを1個として置くしかない。

辺は群の箱ではなく、実際の節点の位置へ描く。だから「できあがった配置の一部を
枠で囲む」という注記の使い方も、鎖を切らずに成り立つ。
"""
from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

from dataclasses import dataclass, field

from .layout_contract import LayoutResult, UnsupportedByStrategy  # noqa: F401
from .sugiyama import layout_graph


@dataclass
class Box:
    """配置の結果。節点1つの箱か、群の箱。"""
    x: float = 0.0
    y: float = 0.0
    width: float = 0.0
    height: float = 0.0


@dataclass
class _Container:
    """群、または最上位。子として節点と群を持つ。"""
    key: str
    label: str | None
    node_ids: list[str] = field(default_factory=list)
    children: list["_Container"] = field(default_factory=list)


def _build_tree(node_ids: list[str], groups: list[dict]) -> _Container:
    """群の包含関係から木を組む。要素の集合が包まれる側を子とする。

    Raises:
        UnsupportedByStrategy: 入れ子でない群の重なりがあるとき。この配置は
            群を入れ子の箱として畳むので、片方だけに属させる形へ縮退させると
            もう片方から要素が黙って消える。
    """
    indexed = [(f"__g{i}", g) for i, g in enumerate(groups)]
    sets = {k: set(g["members"]) for k, g in indexed}

    for i, (ka, ga) in enumerate(indexed):
        for kb, gb in indexed[i + 1:]:
            sa, sb = sets[ka], sets[kb]
            if sa & sb and not (sa <= sb or sb <= sa):
                raise UnsupportedByStrategy(
                    f"群 {ga.get('label') or ka!r} と {gb.get('label') or kb!r} が"
                    f"入れ子でなく重なっている（共通の要素: "
                    f"{sorted(sa & sb)}）。この配置は群を入れ子の箱として畳むので、"
                    f"この重なりは描けない。")

    parent: dict[str, str | None] = {}
    for k, _ in indexed:
        best: str | None = None
        for other, _ in indexed:
            if other == k or not sets[k] < sets[other]:
                continue
            if best is None or sets[other] < sets[best]:
                best = other
        parent[k] = best

    containers = {k: _Container(key=k, label=g.get("label")) for k, g in indexed}
    root = _Container(key="__root", label=None)

    for k, _ in indexed:
        up = parent[k]
        (containers[up] if up else root).children.append(containers[k])

    # 節点は、それを含む最も内側の群へ配る。どの群にも属さないものは最上位へ。
    for nid in node_ids:
        owner: str | None = None
        for k, _ in indexed:
            if nid in sets[k] and (owner is None or sets[k] < sets[owner]):
                owner = k
        (containers[owner] if owner else root).node_ids.append(nid)
    return root


def _descendant_nodes(c: _Container) -> set[str]:
    out = set(c.node_ids)
    for ch in c.children:
        out |= _descendant_nodes(ch)
    return out


def layout_nested(node_sizes: dict[str, tuple[float, float]],
                   edges: list[tuple[str, str]],
                   groups: list[dict],
                   gap_rank: float, gap_order: float,
                   direction: str, frame_pad: float, label_h: float,
                   layout: "Callable[..., LayoutResult] | None" = None,
                   ) -> tuple[dict[str, Box], dict[str, Box],
                              dict[int, list[tuple[float, float]]], float, float]:
    """群を再帰的に解き、節点と群それぞれの絶対座標を返す。

    Args:
        node_sizes: 実節点idごとの (width, height)。
        edges: (from, to) の並び。
        groups: [{"label": str, "members": [id, ...]}, ...]。入れ子でもよい。
        gap_rank / gap_order: 段の間隔・段内の間隔。
        direction: "TB" または "LR"。
        frame_pad: 群の枠が中身の外側へ取る余白。
        label_h: 群のラベルが枠の上に要る高さ。
        layout: 各段で座標を解く戦略。省略時は層状配置。
            受け取って使うのは、呼び出し側が戦略を選んだのに群があるという
            だけで黙って層状に描かれる、ということが起きないようにするため
            （以前はこの引数が無く、群を渡した経路では選ばれた戦略が例外も
            警告も無しに捨てられていた）。

    Returns:
        (節点idごとのBox, 群のキーごとのBox, 辺の番号ごとの経路, 全体の幅, 全体の高さ)。

    Raises:
        なし。
    """
    root = _build_tree(list(node_sizes), groups)
    node_boxes: dict[str, Box] = {}
    group_boxes: dict[str, Box] = {}

    def emplace(c: _Container, ox: float, oy: float) -> None:
        """解いた相対位置を、絶対座標へ展開する。"""
        w, h, placed = solved[c.key]
        local_of, local_paths = paths_local.get(c.key, ({}, {}))
        for gidx, lidx in local_of.items():
            pts = local_paths.get(lidx)
            if pts:
                edge_paths[gidx] = [(ox + x, oy + y) for x, y in pts]
        for nid in c.node_ids:
            x, y = placed[nid]
            node_boxes[nid] = Box(ox + x, oy + y, *node_sizes[nid])
        for ch in c.children:
            x, y = placed[ch.key]
            cw, ch_h = child_size[ch.key]
            group_boxes[ch.key] = Box(ox + x, oy + y, cw, ch_h)
            # 子の中身は、枠の余白とラベルのぶん内側へ入る
            emplace(ch, ox + x + frame_pad,
                    oy + y + frame_pad + (label_h if ch.label else 0))

    solved: dict[str, tuple[float, float, dict]] = {}
    child_size: dict[str, tuple[float, float]] = {}
    paths_local: dict[str, tuple[dict[int, int], dict[int, list]]] = {}
    edge_paths: dict[int, list[tuple[float, float]]] = {}

    def measure(c: _Container) -> tuple[float, float]:
        sizes: dict[str, tuple[float, float]] = {}
        for nid in c.node_ids:
            sizes[nid] = node_sizes[nid]
        for ch in c.children:
            cw, chh = measure(ch)
            pad_w = frame_pad * 2
            pad_h = frame_pad * 2 + (label_h if ch.label else 0)
            child_size[ch.key] = (cw + pad_w, chh + pad_h)
            sizes[ch.key] = child_size[ch.key]

        owner: dict[str, str] = {nid: nid for nid in c.node_ids}
        for ch in c.children:
            for nid in _descendant_nodes(ch):
                owner[nid] = ch.key
        pairs: list = []
        local_of: dict[int, int] = {}   # 元の辺の番号 → この段での辺の番号
        for idx, (a, b) in enumerate(edges):
            ra, rb = owner.get(a), owner.get(b)
            if ra is None or rb is None or ra == rb:
                continue
            local_of[idx] = len(pairs)
            pairs.append((ra, rb))

        if not sizes:
            solved[c.key] = (0.0, 0.0, {})
            return (0.0, 0.0)
        res = (layout or layout_graph)(sizes, pairs, gap_rank, gap_order, direction)
        solved[c.key] = (res.width, res.height, dict(res.positions))
        # 経路は、その段の layout_graph が仮節点を通して解いたものを使う。
        # 始点と終点だけの直線に置き換えると、多段をまたぐ辺が間の箱を突き抜ける。
        paths_local[c.key] = (local_of, dict(res.edge_paths))
        return (res.width, res.height)

    total_w, total_h = measure(root)
    emplace(root, 0.0, 0.0)
    return node_boxes, group_boxes, edge_paths, total_w, total_h
