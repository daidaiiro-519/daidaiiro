"""格子配置 ── 節点を、宣言が持つ座標のとおりに置く。層状・環状・放射の木に次ぐ4つ目。

層状配置は辺から段を決める。だが「対応」は辺を持たない ── 縦横の交点に何が来るかを
座標そのものが指定する。辺から決める戦略へ渡すと、座標が捨てられて1段に並ぶ。

座標は戦略の契約（大きさと辺しか受け取らない）に入っていない。契約を広げる代わりに、
呼び出し側が座標を束ねた戦略を渡す ── `partial(layout_grid, at=…)`。差し替えの継ぎ目が
既にあるので、そこへ載せるだけで済む。

layout_graph と同じ契約（LayoutResult を返す）なので、呼び出し側は戦略を差し替える
だけでよい。
"""
from __future__ import annotations

from .geometry import shift_to_origin
from .layout_contract import LayoutResult


def layout_grid(node_sizes: dict[str, tuple[float, float]],
                edges: list[tuple[str, str]],
                gap_rank: float, gap_order: float,
                direction: str = "TB",
                at: dict[str, tuple] | None = None,
                cols: list | None = None, rows: list | None = None,
                elbow: dict[tuple[str, str], str] | None = None,
                **_ignored) -> LayoutResult:
    """節点を、与えられた座標の格子へ置く。

    列の幅も行の高さも、そこに実際に居る節点の大きさから決める。決め打ちの
    升目を持たない ── 中身が図のときは大きさがまちまちになるため。

    Args:
        node_sizes: 節点idごとの (width, height)。
        edges: (from, to) の並び。対応は辺を持たないが、持っていれば中心どうしを結ぶ。
        gap_rank: 行と行の間に空ける隙間。
        gap_order: 列と列の間に空ける隙間。
        direction: 受け取るが使わない（格子に上下左右の進行方向は無い）。
        at: 節点idごとの (列の鍵, 行の鍵)。渡されない節点は格子に載らない。
        cols / rows: 列・行の並び。渡さなければ鍵を昇順に整列した順になる。
            見出しを端の列・行へ置きたいときは、その鍵を先頭に入れて渡す
            ── 整列に頼ると、見出しが中ほどへ紛れ込む。
        elbow: 鍵線にしたい辺 (from, to) → 曲がり角の置き方。
            "vertical" は角を出る側の真下（真上）へ置く ── 辺は底辺か上辺から出て、
            相手の横の辺へ着く。"horizontal" はその逆で、横の辺から出て相手の
            底辺か上辺へ着く。**辺の着き先は次の点へ向かう向きで決まる**ので、
            角を1つ挟むだけで出入りする辺が変わる。渡さなければ中心どうしの直線。

    Returns:
        LayoutResult。edge_paths は節点の中心どうしを結ぶ2点。

    Raises:
        KeyError: at に無い節点idが node_sizes にあるとき。
            elbow が、edges に無い辺を指したとき ── 綴りの誤りを黙って落とさない。
        ValueError: elbow の置き方が "vertical" でも "horizontal" でもないとき。
    """
    at = at or {}
    elbow = elbow or {}
    bad = [k for k, v in elbow.items() if v not in ("vertical", "horizontal")]
    if bad:
        raise ValueError(f"曲がり角の置き方は vertical か horizontal です: {bad}")
    absent = [k for k in elbow if k not in set(edges)]
    if absent:
        raise KeyError(f"辺に無いものが elbow にあります: {absent}")
    ids = list(node_sizes)
    if not ids:
        return LayoutResult({}, {}, 0.0, 0.0)
    missing = [i for i in ids if i not in at]
    if missing:
        raise KeyError(f"座標が無い節点があります: {missing}")

    cols = list(cols) if cols else sorted({at[i][0] for i in ids}, key=_sort_key)
    rows = list(rows) if rows else sorted({at[i][1] for i in ids}, key=_sort_key)
    # 列の幅・行の高さは、そこに居るもののうち最も大きいものに合わせる
    col_w = [max((node_sizes[i][0] for i in ids if at[i][0] == c), default=0.0) for c in cols]
    row_h = [max((node_sizes[i][1] for i in ids if at[i][1] == r), default=0.0) for r in rows]
    col_x = [sum(col_w[:c]) + gap_order * c for c in range(len(cols))]
    row_y = [sum(row_h[:r]) + gap_rank * r for r in range(len(rows))]

    positions: dict[str, tuple[float, float]] = {}
    centres: dict[str, tuple[float, float]] = {}
    for i in ids:
        c, r = cols.index(at[i][0]), rows.index(at[i][1])
        w, h = node_sizes[i]
        # 升目の中で中央へ寄せる。左上へ寄せると、大きさの違う中身が
        # ばらばらに見えて、縦横の対応が読み取りにくくなる
        x = col_x[c] + (col_w[c] - w) / 2
        y = row_y[r] + (row_h[r] - h) / 2
        positions[i] = (x, y)
        centres[i] = (x + w / 2, y + h / 2)

    positions, (min_x, min_y) = shift_to_origin(positions)
    centres = {k: (x - min_x, y - min_y) for k, (x, y) in centres.items()}
    width = max(positions[k][0] + node_sizes[k][0] for k in positions)
    height = max(positions[k][1] + node_sizes[k][1] for k in positions)
    edge_paths = {n: _path(centres[a], centres[b], elbow.get((a, b)))
                  for n, (a, b) in enumerate(edges)
                  if a in centres and b in centres}
    return LayoutResult(positions, edge_paths, width, height)


def _path(a: tuple[float, float], b: tuple[float, float],
          bend: str | None) -> list[tuple[float, float]]:
    """2つの中心を結ぶ経路。曲がり角の置き方が渡されたら、角を1つ挟む。

    角が端のどちらかと重なるなら、挟まない ── 同じ点が2つ並ぶと、そこから
    向きが決まらず、辺の着き先が定まらなくなる（実測：長さ0の経路が出て、
    終端がどのインクにも着かなかった）。
    """
    if bend is None:
        return [a, b]
    corner = (a[0], b[1]) if bend == "vertical" else (b[0], a[1])
    if corner in (a, b):
        return [a, b]
    return [a, corner, b]


def _sort_key(v):
    """数は数として、そうでないものは文字として並べる。"""
    return (0, v, "") if isinstance(v, (int, float)) else (1, 0, str(v))
