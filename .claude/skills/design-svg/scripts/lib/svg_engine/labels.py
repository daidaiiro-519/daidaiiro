"""ラベルの置き場所 ── 互いに重ならないよう、まとめて調整する。

1つずつ「決まった場所」に置くだけだと、近くに集まったときに重なる。
まとめて受け取り、候補をいくつか試して、既に置いたものと重ならない位置を選ぶ。

**辺のラベルも点のラベルも、この1つの仕組みを使う。**別々に書くと、片方だけが
重なりを見なくなる ── 実際に、散布図のラベルは他のラベルを一切見ておらず、
点を12個近づけると15件重なった（辺のラベルは同じ状況で0件）。置き場所の
候補をどう並べるかだけが両者の違いで、選び方は共通である。
"""
from __future__ import annotations

from .geometry import rects_overlap
from .text import text_width
from .tokens import Style

# 置き場所を試す刻みの上限。刻みは経路の長さから決まるが、極端に短い経路では
# 刻みが細かくなりすぎて回り続けるので、そこで打ち切る（見た目には効かない）。
_MAX_STEPS = 20

def _candidates(path_len: float, label_w: float) -> list[float]:
    """置き場所の候補を、経路の長さとラベルの幅から決める。

    割合を決め打ちで並べると、経路が短いとき候補どうしが重なり、長いとき
    隙間だらけになる。ラベル1つぶんずつずらした位置を、真ん中から外へ
    交互に出す（真ん中が最も読みやすいので、そこを最優先にする）。
    """
    if path_len <= 0:
        return [0.5]
    step = min(label_w / path_len, 0.5)
    if step <= 0:
        return [0.5]
    # 端の余白は、ラベルの半分が経路からはみ出さない位置。勘の数字ではなく、
    # ラベルの大きさと経路の長さから決まる。
    edge = min(label_w / 2 / path_len, 0.5)
    lo, hi = edge, 1.0 - edge
    out = [0.5]
    k = 1
    while 0.5 - step * k > lo or 0.5 + step * k < hi:
        for f in (0.5 - step * k, 0.5 + step * k):
            if lo <= f <= hi:
                out.append(f)
        k += 1
        if k > _MAX_STEPS:
            break
    return out


def _point_at_fraction(points: list[tuple[float, float]], t: float) -> tuple[float, float]:
    """折れ線の弧長に沿って、割合tの位置の点を返す。"""
    seg_lens = []
    for i in range(len(points) - 1):
        (x1, y1), (x2, y2) = points[i], points[i + 1]
        seg_lens.append(((x2 - x1) ** 2 + (y2 - y1) ** 2) ** 0.5)
    total = sum(seg_lens) or 1.0
    target = total * t
    cursor = 0.0
    for i, seg in enumerate(seg_lens):
        if cursor + seg >= target or i == len(seg_lens) - 1:
            local_t = 0.0 if seg == 0 else (target - cursor) / seg
            x1, y1 = points[i]
            x2, y2 = points[i + 1]
            return (x1 + (x2 - x1) * local_t, y1 + (y2 - y1) * local_t)
        cursor += seg
    return points[-1]


def place_avoiding(items: list[dict],
                    occupied: list[tuple[float, float, float, float]] | None = None
                    ) -> list[tuple[float, float]]:
    """候補の中から、既に置いたものと重ならない場所を1つずつ選ぶ。

    Args:
        items: [{"size": (w, h), "candidates": [(cx, cy), ...]}, ...]。
            候補は良い順に並べること（先頭が最も置きたい場所）。
        occupied: 既に他のものが占めている領域。

    Returns:
        items と同じ順の、選ばれた中心座標の並び。

    Raises:
        なし。どの候補も重なるときは最後の候補を採る（最善努力）。
    """
    placed: list[tuple[float, float, float, float]] = list(occupied or [])
    out: list[tuple[float, float]] = []
    for it in items:
        w, h = it["size"]
        cands = it["candidates"] or [(0.0, 0.0)]
        chosen = None
        for cx, cy in cands:
            box = (cx - w / 2, cy - h / 2, cx + w / 2, cy + h / 2)
            if not any(rects_overlap(box, p) for p in placed):
                chosen = (cx, cy)
                placed.append(box)
                break
        if chosen is None:
            cx, cy = cands[-1]
            chosen = (cx, cy)
            placed.append((cx - w / 2, cy - h / 2, cx + w / 2, cy + h / 2))
        out.append(chosen)
    return out


def place_edge_labels(edges: list[dict], style: Style,
                      occupied: list[tuple[float, float, float, float]] | None = None
                      ) -> dict[int, tuple[float, float]]:
    """ラベルを持つ辺それぞれについて、重ならない置き場所を1つ返す。

    Args:
        edges: [{"index": int, "points": [(x,y),...], "label": str}, ...]
               ラベルを持つ辺だけを渡すこと。
        style: font.size-small を持つ、解決済みのスタイル辞書。
        occupied: 既に他のものが占めている領域。ラベルはここを避ける
            （囲みの枠線・囲みの札など。渡さなければ他のラベルとだけ突き合わせる）。

    Returns:
        辺のindex → 置き場所(x, y)（ラベルの中心）。

    Raises:
        なし。全候補が重なっても、最後の候補をそのまま採用する（最善努力）。
    """
    fs = style.num("font.size-small")
    items = []
    for e in edges:
        w = text_width(e["label"], fs) + style.num("size.label-pad-x")
        h = fs * style.num("size.label-line-h")
        seg = sum(((b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2) ** 0.5
                  for a, b in zip(e["points"], e["points"][1:]))
        # 辺のラベルの候補は「経路上の点」。真ん中から外へ交互に。
        items.append({"size": (w, h),
                      "candidates": [_point_at_fraction(e["points"], f)
                                     for f in _candidates(seg, w)]})
    chosen = place_avoiding(items, occupied)
    return {e["index"]: c for e, c in zip(edges, chosen)}
