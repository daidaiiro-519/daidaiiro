"""量を描く部品 ── 値から座標が一意に決まる図。

内訳・大小・並び順・区間・分布・基準からのずれ・2軸上の点・流れる量・位置を、
それぞれ1つの形で描く。どれも値さえ決まれば座標が決まるので、sugiyama.py の
ような配置解決は要らない（その場の算術で完結する）。

どの図がどんな言い分を運ぶかは、ここでは決めない。それは呼ぶ側が決めることで、
この部品が知っているのは「どんな値を受け取り、どんな形を描くか」だけである。

かつて別の場所にあった実測済みの計算を踏襲しつつ、この
パッケージの流儀（構造はprops、見た目はstyle、部品は台帳へ登録）へ
書き直した。色・書体・寸法はすべて style（tokens.py のトークン）から引き、
直書きしない ── 直書きすると、テーマを差し替えても寸法だけ変わらず取り残される
(実際にこのファイルでその不一致が見つかり、この形へ直した)。
"""
from __future__ import annotations

import math
from html import escape as _esc

from .labels import place_avoiding
from .tokens import Style
from .registry import Absolute, OwnOrigin, component, render_component, render_node
from .text import column_width, text_width


def _tone(style: Style, i: int) -> str:
    tones = style.tones("chart.tones")
    return tones[i % len(tones)]


def _t(x, y, s, style: Style, cls_color: str, anchor="middle", size=None, weight="400"):
    fs = size or style.num("font.size-small")
    return (f'<text x="{x:.1f}" y="{y:.1f}" text-anchor="{anchor}" '
            f'font-family="{style.text("font.family")}" font-size="{fs}" '
            f'font-weight="{weight}" fill="{style.text(cls_color)}">{_esc(str(s))}</text>')


# ── 全体と部分（円/ドーナツ） ────────────────────────────────

@component("donut")
def donut(props: dict, style: Style) -> OwnOrigin:
    """割合の輪だけを描く部品。凡例も余白も持たない。

    これは「部品」であって「図」ではない ── 自分の原点で形を描き、大きさと
    輪郭を申告するだけ。だから他の図の節点としてそのまま置ける（円の輪郭を
    申告するので、辺は円周に着く）。凡例を付けたければ、図の側が組む。

    props: slices（[{"name","value"},...]）／centre（中央に置く文字、任意）

    Args:
        props: 上記の入力。
        style: resolve_style() が解決した見た目の値。

    Returns:
        ComponentResult。輪郭は円を多角形で近似したもの。

    Raises:
        ValueError: 輪の太さが半径以上のとき。
    """
    slices = props["slices"]
    total = sum(s["value"] for s in slices) or 1
    r = style.num("chart.pie-radius")
    thickness = style.num("chart.pie-donut-thickness")
    if thickness >= r:
        raise ValueError("chart.pie-donut-thickness は chart.pie-radius より小さくすること"
                         f"(thickness={thickness}, radius={r})")
    size = (r + thickness / 2) * 2
    cx = cy = size / 2

    def arc(a0, a1):
        p = lambda a: (cx + r * math.cos(math.radians(a - 90)),
                       cy + r * math.sin(math.radians(a - 90)))
        x0, y0 = p(a0)
        x1, y1 = p(a1)
        large = 1 if a1 - a0 > 180 else 0
        return f"M{x0:.1f},{y0:.1f} A{r:.1f},{r:.1f} 0 {large},1 {x1:.1f},{y1:.1f}"

    body, a = [], 0.0
    for i, s in enumerate(slices):
        a1 = a + s["value"] / total * 360
        body.append(f'<path d="{arc(a, a1)}" fill="none" stroke="{_tone(style, i)}" '
                    f'stroke-width="{thickness}"/>')
        a = a1
    if props.get("centre"):
        fs = style.num("font.size")
        body.append(_t(cx, cy + fs * style.num("font.baseline-ratio"), props["centre"],
                       style, "color.ink", weight="600", size=fs))
    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=size, height=size)


@component("pie")
def pie(props: dict, style: Style) -> OwnOrigin:
    """輪と凡例を並べた図。輪そのものは donut 部品が描く。

    ここは「図」── 部品を組み合わせ、共有の仕組み（文字の縦揃え・隙間の
    トークン）で並べるだけ。形の計算は持たない。

    props: slices／centre（任意）／legend（bool、既定True）

    Args:
        props: 上記の入力。
        style: resolve_style() が解決した見た目の値。

    Returns:
        ComponentResult。

    Raises:
        なし。
    """
    ring = render_node("donut", props, style)
    if not props.get("legend", True):
        return ring

    slices = props["slices"]
    pad = style.num("chart.pad")
    gap = style.num("chart.gap")
    fs_small = style.num("font.size-small")
    cap = style.num("font.cap-ratio")
    row_h = style.num("chart.legend-row-h")
    # 凡例の幅は中身から決める。決め打ちにすると、項目名が短いときに値だけが
    # 遠くへ取り残される（実測：13と5が項目名から大きく離れて見えた）。
    swatch = fs_small
    name_w = column_width([s["name"] for s in slices], fs_small, gap)
    value_w = column_width([s["value"] for s in slices], fs_small, 0.0)
    legend_w = swatch + gap / 2 + name_w + value_w

    w = pad * 2 + ring.width + gap + legend_w
    h = pad * 2 + max(ring.height, row_h * len(slices))
    body = [f'<g transform="translate({pad},{(h - ring.height) / 2:.1f})">{ring.svg}</g>']
    lx = pad + ring.width + gap
    for i, s in enumerate(slices):
        y = pad + row_h * i + (row_h + fs_small * cap) / 2
        sw = fs_small
        body.append(f'<rect x="{lx}" y="{y - sw * cap - (sw - sw * cap) / 2:.1f}" '
                    f'width="{sw}" height="{sw}" rx="{style.num("size.radius-small")}" fill="{_tone(style, i)}"/>')
        body.append(_t(lx + sw + gap / 2, y, s["name"], style, "color.ink-soft", anchor="start"))
        body.append(_t(w - pad, y, s["value"], style, "color.ink", anchor="end"))
    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)


# ── 量の大小・分布・偏差（縦棒。基準線があれば正負に伸びる） ───────────

@component("bars")
def bars(props: dict, style: Style) -> OwnOrigin:
    """縦棒。`baseline`を与えると、そこからの正負の差として伸びる(偏差)。
    無指定なら0から積む(量の大小・分布)。

    props: bars（[{"name","value"},...]）／baseline（任意）／
           axis_label（値の軸が何を表すか、任意）／
           item_axis_label（**項目の軸**が何を表すか、任意）

    軸を2本持てるのは、「量の大小」が読む枠に軸を2本要求するため。1本しか
    持てなかったときは項目の軸が落ちていた ── 仕様が必須と定めた欄を落とさない。
    """
    items = props["bars"]
    baseline = props.get("baseline", 0)
    values = [it["value"] - baseline for it in items]
    pad = style.num("chart.pad")
    bw = style.num("chart.bar-width")
    bar_gap = style.num("chart.bar-gap")
    # 1本ぶんの持ち場は、棒の幅と、その下に付く項目名の広いほう。棒の幅だけで
    # 決めると、名前が棒より長いときに隣どうしで重なる（文字幅の見積りを実測へ
    # 直したときに表面化した ── それまでは少なく見積もっていて見えなかった）。
    slot = max(bw + bar_gap,
               column_width([it["name"] for it in items],
                            style.num("font.size-small"), bar_gap))
    ph = style.num("chart.bar-plot-h")
    left_margin = style.num("chart.bar-left-margin")
    bottom_margin = style.num("chart.bar-bottom-margin")
    # 軸ラベルは、棒の上に出る値と同じ高さの帯を奪い合う。軸の左へ右寄せで置くと
    # 長いラベルが画布の外へ出て、軸の上へそのまま置くと最も高い棒の値と重なる
    # （どちらも実測で踏んだ）。ラベル専用の帯を確保し、作図領域をその下から始める。
    # 必要な高さは目分量ではなく計算で出す。軸ラベルの下端（ベースライン＋下ばね）と、
    # 最も高い棒の上に出る値の上端（ベースラインから字面の高さぶん上）が、
    # 離れていなければならない。係数で決めて足りなかった実例がある。
    fs_small = style.num("font.size-small")
    if props.get("axis_label"):
        label_baseline = pad + fs_small          # ラベルのベースライン
        gap = style.num("chart.gap")
        value_offset = gap / 2                    # 値は棒の上端からこれだけ上
        cap = fs_small * style.num("font.cap-ratio")  # 字面の高さ（ベースラインから上）
        breathing = gap / 2                       # 隙間
        plot_top = label_baseline + breathing + value_offset + cap
    else:
        plot_top = pad
    label_band = plot_top - pad
    gap = style.num("chart.gap")
    w = pad * 2 + left_margin + len(items) * slot
    # 負に伸びる棒は、値の札を作図領域の下へ出す。名前の行は全部の棒で同じ高さに
    # あって動かせないので、札の帯を先に確保してから名前を置く（上の軸ラベルで
    # 使っている考え方の裏返し）。確保しないと、下まで伸びた棒の値が名前へ重なる
    # ── 基準からのずれを描いたときだけ、値の札の置き場所が無くなって落ちた。
    value_band = (gap / 2 + fs_small * (style.num("font.cap-ratio") + style.num("font.descender-ratio"))
                  if min(values) < 0 else 0.0)
    item_axis_band = (fs_small * style.num("size.label-line-h")
                      if props.get("item_axis_label") else 0.0)
    h = plot_top + ph + value_band + item_axis_band + bottom_margin + pad
    x0 = pad + left_margin
    zero_y = plot_top + ph / 2 if min(values) < 0 else plot_top + ph
    top = max(abs(v) for v in values) or 1
    scale = (ph / 2 if min(values) < 0 else ph) / top

    body = [f'<line x1="{x0}" y1="{plot_top:.1f}" x2="{x0}" y2="{plot_top + ph:.1f}" '
           f'stroke="{style.text("chart.axis")}"/>',
           f'<line x1="{x0}" y1="{zero_y:.1f}" x2="{w - pad}" y2="{zero_y:.1f}" '
           f'stroke="{style.text("chart.axis")}"/>']
    if props.get("axis_label"):
        body.append(_t(x0, pad + fs_small, props["axis_label"], style, "color.ink-faint", "start"))
    for i, it in enumerate(items):
        v = it["value"] - baseline
        bx = x0 + i * slot + (slot - bw) / 2
        bh = abs(v) * scale
        by = zero_y - bh if v >= 0 else zero_y
        body.append(f'<rect x="{bx:.1f}" y="{by:.1f}" width="{bw}" height="{bh:.1f}" rx="{style.num("size.radius-small")}" '
                    f'fill="{_tone(style, i)}"/>')
        num_y = (by - gap / 2 if v >= 0
                 else by + bh + gap / 2 + fs_small * style.num("font.cap-ratio"))
        body.append(_t(bx + bw / 2, num_y, it["value"], style, "color.ink", size=fs_small))
        body.append(_t(bx + bw / 2,
                       plot_top + ph + value_band + gap + fs_small * style.num("font.cap-ratio"),
                       it["name"], style, "color.ink-faint"))
    if props.get("item_axis_label"):
        # 項目の軸の名前は、項目名の行のさらに下。名前の行と重ならないよう、
        # 行の高さぶん下げる（決め打ちを置かず、書体から導く）
        name_row = plot_top + ph + value_band + gap + fs_small * style.num("font.cap-ratio")
        body.append(_t(x0 + (w - pad - x0) / 2,
                       name_row + fs_small * style.num("size.label-line-h"),
                       props["item_axis_label"], style, "color.ink-faint"))
    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)


# ── 並び順（横並びの一覧。読ませたいのは順番で、値は添え物） ──────────────

@component("ranking")
def ranking(props: dict, style: Style) -> OwnOrigin:
    """items の並び順をそのまま順位とする、横棒の一覧。"""
    items = props["items"]
    top = max(it["value"] for it in items) or 1
    pad = style.num("chart.pad")
    row_h = style.num("chart.rank-row-h")
    bar_w = style.num("chart.rank-bar-w")
    gap0 = style.num("chart.gap")
    fs0 = style.num("font.size-small")
    # 名前・順位・値の欄は、それぞれ実際に入る文字から決める
    name_w = column_width([it["name"] for it in items], fs0, gap0)
    left_margin = column_width(range(1, len(items) + 1), fs0, gap0 * 2)
    value_w = column_width([it["value"] for it in items], fs0, gap0 * 2)
    w = pad * 2 + left_margin + name_w + bar_w + value_w
    h = pad * 2 + row_h * len(items)
    gap = style.num("chart.gap")
    base = style.num("font.baseline-ratio")
    fs_small = style.num("font.size-small")
    track_h = style.num("chart.rank-track-h")
    body = []
    for i, it in enumerate(items):
        y = pad + row_h * i
        # 順位の数字は、名前の左に置く。左余白の中で右寄せにする。
        body.append(_t(pad + left_margin - gap, y + row_h / 2 + fs_small * base,
                       i + 1, style, "color.ink-faint", "end"))
        body.append(_t(pad + left_margin + name_w, y + row_h / 2 + fs_small * base, it["name"], style,
                       "color.ink-soft", "end"))
        track_x = pad + left_margin + name_w + gap
        track_y = y + (row_h - track_h) / 2
        body.append(f'<rect x="{track_x}" y="{track_y:.1f}" width="{bar_w}" height="{track_h:.1f}" '
                    f'rx="{style.num("size.radius-small")}" fill="{style.text("chart.grid")}"/>')
        fill_w = bar_w * it["value"] / top
        body.append(f'<rect x="{track_x}" y="{track_y:.1f}" width="{fill_w:.1f}" '
                    f'height="{track_h:.1f}" rx="{style.num("size.radius-small")}" fill="{_tone(style, 0)}"/>')
        body.append(_t(track_x + bar_w + value_w - gap, y + row_h / 2 + fs_small * base,
                       it["value"], style, "color.ink", "end"))
    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)


# ── 時間変化（帯） ───────────────────────────────────────────

@component("lanes")
def lanes(props: dict, style: Style) -> OwnOrigin:
    """行ごとの区間を、帯として並べる。列(時間軸)は共有する。"""
    rows = props["rows"]
    # 区間の上限は宣言が持つ。無ければ実際の最大値から決める（既定値を置かない）。
    span = props.get("span") or max((b["to"] for r in rows for b in r["bars"]), default=1)
    pad = style.num("chart.pad")
    tw = style.num("chart.lane-track-w")
    rh = style.num("chart.lane-row-h")
    fs_small = style.num("font.size-small")
    base = style.num("font.baseline-ratio")
    gap = style.num("chart.gap")
    inset = style.num("chart.lane-bar-inset")
    axis_h = style.num("chart.lane-axis-h")
    lw = column_width([r["name"] for r in rows], fs_small, gap * 2)
    w = pad * 2 + lw + tw
    h = pad * 2 + rh * len(rows) + (axis_h if props.get("axis_label") else 0)
    body = []
    for i, r in enumerate(rows):
        y = pad + rh * i
        body.append(_t(pad + lw - gap, y + rh / 2 + fs_small * base, r["name"], style,
                       "color.ink-soft", "end"))
        body.append(f'<rect x="{pad + lw}" y="{y + inset:.1f}" width="{tw}" '
                    f'height="{rh - inset * 2:.1f}" rx="{style.num("size.radius-small")}" fill="{style.text("chart.grid")}"/>')
        for j, bar in enumerate(r["bars"]):
            bx = pad + lw + tw * bar["from"] / span
            bwid = tw * (bar["to"] - bar["from"]) / span
            body.append(f'<rect x="{bx:.1f}" y="{y + inset:.1f}" width="{bwid:.1f}" '
                        f'height="{rh - inset * 2:.1f}" rx="{style.num("size.radius-small")}" fill="{_tone(style, j)}"/>')
            if bar.get("label"):
                body.append(_t(bx + bwid / 2, y + rh / 2 + fs_small * base, bar["label"],
                               style, "color.ink", size=fs_small))
    if props.get("axis_label"):
        ay = pad + rh * len(rows) + gap / 2
        body.append(f'<line x1="{pad + lw}" y1="{ay:.1f}" x2="{pad + lw + tw}" y2="{ay:.1f}" '
                    f'stroke="{style.text("chart.axis")}"/>')
        body.append(_t(pad + lw, ay + gap / 2 + fs_small, props["axis_label"], style,
                       "color.ink-faint", "start"))
    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)


# ── 相関（2軸の散布図） ────────────────────────────────────────

@component("scatter")
def scatter(props: dict, style: Style) -> OwnOrigin:
    """2つの軸上の座標として点を置く。点が端に寄っても、ラベルが枠の外へ
    はみ出さないよう、点の位置に応じて寄せる向きを変える(実測で見つかった不具合)。
    """
    pts = props["points"]
    w = style.num("chart.scatter-w")
    h = style.num("chart.scatter-h")
    x0 = style.num("chart.scatter-margin-left")
    y0 = style.num("chart.scatter-margin-top")
    x1 = w - style.num("chart.scatter-margin-left")
    y1 = h - style.num("chart.scatter-margin-bottom")
    r = style.num("chart.scatter-point-r")
    edge_threshold = style.num("chart.scatter-edge-threshold")
    fs_small = style.num("font.size-small")
    cap = fs_small * style.num("font.cap-ratio")
    gap = style.num("chart.gap")
    xs = [p["x"] for p in pts] or [1]
    ys = [p["y"] for p in pts] or [1]
    mx, my = max(xs) or 1, max(ys) or 1
    body = [f'<line x1="{x0}" y1="{y0}" x2="{x0}" y2="{y1}" stroke="{style.text("chart.axis")}"/>',
           f'<line x1="{x0}" y1="{y1}" x2="{x1}" y2="{y1}" stroke="{style.text("chart.axis")}"/>']
    # 点の位置を先に全部決めてから、ラベルをまとめて置く。1点ずつ置くと
    # 他のラベルを見られず、点が近いと必ず重なる（実測：12点で15件）。
    placed_pts = []
    for p in pts:
        px = x0 + (x1 - x0) * (p["x"] / mx)
        py = y1 - (y1 - y0) * (p["y"] / my)
        placed_pts.append((px, py))
        body.append(f'<circle cx="{px:.1f}" cy="{py:.1f}" r="{r}" fill="{_tone(style, 0)}"/>')

    off = r + gap / 2          # 点から離す量。点の大きさと隙間から決まる
    items = []
    for (px, py), p in zip(placed_pts, pts):
        lw = text_width(p["name"], fs_small) + gap
        lh = cap + gap / 2
        # 候補は点の周り8方向。上を最優先にし、外側から順に試す。
        cands = [(px, py - off - cap / 2), (px, py + off + cap / 2),
                 (px - off - lw / 2, py), (px + off + lw / 2, py),
                 (px - off - lw / 2, py - off), (px + off + lw / 2, py - off),
                 (px - off - lw / 2, py + off), (px + off + lw / 2, py + off)]
        # 画布の外へ出る候補は使わない
        inside = [(cx, cy) for cx, cy in cands
                  if cx - lw / 2 >= 0 and cx + lw / 2 <= w and cy - cap >= 0 and cy <= h]
        items.append({"size": (lw, lh), "candidates": inside or cands})
    # 点そのものも避ける相手に入れる ── ラベルが点に乗ると読めない
    occupied = [(px - r, py - r, px + r, py + r) for px, py in placed_pts]
    for (cx, cy), p in zip(place_avoiding(items, occupied), pts):
        body.append(_t(cx, cy, p["name"], style, "color.ink-soft"))
    if props.get("x_label"):
        body.append(_t((x0 + x1) / 2, h - gap, props["x_label"], style, "color.ink-faint"))
    if props.get("y_label"):
        fs = style.num("font.size-small")
        label_x = style.num("chart.pad")
        body.append(f'<text x="{label_x}" y="{(y0 + y1) / 2:.1f}" font-family="{style.text("font.family")}" '
                    f'font-size="{fs}" fill="{style.text("color.ink-faint")}" text-anchor="middle" '
                    f'transform="rotate(-90 {label_x} {(y0 + y1) / 2:.1f})">'
                    f'{_esc(props["y_label"])}</text>')
    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)


# ── 流量（帯グラフ／サンキー風） ────────────────────────────────

@component("flow")
def flow(props: dict, style: Style) -> OwnOrigin:
    """左右2列の間を、太さが値を表すリボンで結ぶ。"""
    links = props["links"]
    total = sum(l["value"] for l in links) or 1
    w = style.num("chart.flow-w")
    h = style.num("chart.flow-h")
    pad = style.num("chart.pad")
    margin = style.num("chart.flow-margin")
    bw = style.num("chart.flow-bar-w")
    node_gap = style.num("chart.flow-node-gap")
    gap = style.num("chart.gap")
    base = style.num("font.baseline-ratio")
    fs_small = style.num("font.size-small")
    lefts: dict[str, float] = {}
    rights: dict[str, float] = {}
    for l in links:
        lefts.setdefault(l["from"], 0)
        lefts[l["from"]] += l["value"]
        rights.setdefault(l["to"], 0)
        rights[l["to"]] += l["value"]
    # 左右の余白は、実際に入る名前から決める。トークンの値は下限として使う
    # ── 固定の余白のままだと、長い名前が画布の外へ出る（実測：右の名前が
    # はみ出して「画布の外」と鳴った）。
    span = w - (pad + margin) * 2          # 図そのものが占める横幅は変えない
    margin_l = max(margin, max((text_width(str(k), fs_small) for k in lefts), default=0.0) + gap)
    margin_r = max(margin, max((text_width(str(k), fs_small) for k in rights), default=0.0) + gap)
    w = pad + margin_l + span + margin_r + pad
    xl, xr = pad + margin_l, pad + margin_l + span
    # 節点どうしの隙間は、値に比例する高さとは別に要る。引かずに配ると、
    # 隙間の分だけ最後の節点が下へ押し出され、その名前が画布の外へ出る
    # （実測：画布の高さ210に対し、最後の名前の下端が213.8だった）。
    # 節点どうしの隙間は、値に比例する高さとは別に要る。引かずに配ると、
    # 隙間の分だけ最後の節点が下へ押し出され、その名前が画布の外へ出る
    # （実測：画布の高さ210に対し、最後の名前の下端が213.8だった）。
    # 左右で配分を変えるとリボンの太さが両端で食い違うので、節点の多いほうに
    # 合わせて1つの配分にする ── 少ないほうは下に余りが出るだけで済む。
    span_h = h - pad * 2 - node_gap * max(len(lefts) - 1, len(rights) - 1, 0)
    pos_l, pos_r, cy = {}, {}, float(pad)
    for k, v in lefts.items():
        pos_l[k] = [cy, cy + span_h * v / total]
        cy = pos_l[k][1] + node_gap
    cy = float(pad)
    for k, v in rights.items():
        pos_r[k] = [cy, cy + span_h * v / total]
        cy = pos_r[k][1] + node_gap

    body = []
    for i, l in enumerate(links):
        hgt = span_h * l["value"] / total
        a, b = pos_l[l["from"]][0], pos_r[l["to"]][0]
        pos_l[l["from"]][0] += hgt
        pos_r[l["to"]][0] += hgt
        m = (xl + xr) / 2
        body.append(f'<path d="M{xl + bw},{a:.1f} C{m},{a:.1f} {m},{b:.1f} {xr},{b:.1f} '
                    f'L{xr},{b + hgt:.1f} C{m},{b + hgt:.1f} {m},{a + hgt:.1f} '
                    f'{xl + bw},{a + hgt:.1f} Z" fill="{_tone(style, i)}" '
                    f'opacity="{style.num("chart.ribbon-opacity")}"/>')
    for k, (y0, y1) in pos_l.items():
        body.append(f'<rect x="{xl}" y="{y0:.1f}" width="{bw}" height="{y1 - y0:.1f}" '
                    f'fill="{style.text("color.ink-faint")}"/>')
        body.append(_t(xl - gap, (y0 + y1) / 2 + fs_small * base, k, style, "color.ink-soft", "end"))
    for k, (y0, y1) in pos_r.items():
        body.append(f'<rect x="{xr - bw}" y="{y0:.1f}" width="{bw}" height="{y1 - y0:.1f}" '
                    f'fill="{style.text("color.ink-faint")}"/>')
        body.append(_t(xr + gap, (y0 + y1) / 2 + fs_small * base, k, style, "color.ink-soft", "start"))
    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)


# ── 空間（位置が意味を運ぶ、並べた区画） ────────────────────────────

@component("spatial")
def spatial(props: dict, style: Style) -> OwnOrigin:
    """items を置く。読ませたいのは位置そのもの。

    props: items（[{"name", "at": [x, y] または "depth", "role"}, ...]）／
           cols（at を使わないときの列数）／
           ground（この空間が何の上にあるか、任意）／
           axis_label（縦が何を表すか、任意）

    **at があれば座標で置く。** 無ければ宣言の並び順のまま積む（従来の形）。
    座標と地を受け取れるのは、「空間」が読む枠に地と軸を要求するため ── 深さと
    列数しか取れなかったときは、座標も地も落ちて単なる並びになっていた。

    箱の幅は、列ごとに最長のラベルへ合わせる(固定幅にすると、実測で長い
    ラベルがはみ出す不具合が出た)。
    """
    items = props["items"]
    xs = sorted({it["at"][0] for it in items if it.get("at") is not None})
    ys = sorted({it["at"][1] for it in items if it.get("at") is not None})
    by_at = bool(xs)
    cols = len(xs) if by_at else props.get("cols", 1)
    ch = style.num("chart.spatial-row-h")
    gap = style.num("chart.spatial-gap")
    pad_x = style.num("chart.spatial-pad-x")
    fs = style.num("font.size")
    fs_small = style.num("font.size-small")
    rows = len(ys) if by_at else math.ceil(len(items) / cols)
    # 名前の右には、深さの数字が右寄せで入る。その欄も中身から決める
    # （決め打ちの余白だと、長い名前が数字とぶつかる ── 実測で踏んだ）。
    depth_w = column_width([it["depth"] for it in items if it.get("depth") is not None],
                           fs_small, gap * 2)
    def cell(i, it):
        """その要素が何列目・何行目に来るか。"""
        if by_at:
            return xs.index(it["at"][0]), ys.index(it["at"][1])
        return i % cols, i // cols

    col_w = [0.0] * cols
    for i, it in enumerate(items):
        c, _ = cell(i, it)
        col_w[c] = max(col_w[c], text_width(it["name"], fs) + pad_x * 2 + depth_w)
    col_x = [sum(col_w[:c]) + gap * c for c in range(cols)]
    grid_w = sum(col_w) + gap * (cols - 1)
    grid_h = rows * ch + (rows - 1) * gap
    # 地と軸の名前は、置いたものの外側に帯を取る。厚みは書体から導く。
    band = fs_small * style.num("size.label-line-h")
    left = band if props.get("axis_label") else 0.0
    topb = band if props.get("ground") else 0.0
    w = grid_w + left
    h = grid_h + topb
    body = []
    if props.get("ground"):
        # 地 ── 置いたものが何の上にあるか。背に敷き、名前を左上へ置く
        body.append(f'<rect x="{left:.1f}" y="{topb:.1f}" width="{grid_w:.1f}" '
                    f'height="{grid_h:.1f}" rx="{style.num("size.radius-small")}" fill="{style.text("chart.grid")}" opacity="{style.num("opacity.faint")}"/>')
        body.append(_t(left, band / 2 + fs_small * style.num("font.baseline-ratio"),
                       props["ground"], style, "color.ink-faint", "start", size=fs_small))
    if props.get("axis_label"):
        cy = topb + grid_h / 2
        ax = band / 2 + fs_small * style.num("font.baseline-ratio") - fs_small
        body.append(f'<text x="{ax:.1f}" y="{cy:.1f}" text-anchor="middle" '
                    f'transform="rotate(-90 {ax:.1f} {cy:.1f})" '
                    f'font-family="{style.text("font.family")}" font-size="{fs_small}" '
                    f'fill="{style.text("color.ink-faint")}">{_esc(str(props["axis_label"]))}</text>')
    for i, it in enumerate(items):
        c, r = cell(i, it)
        x = left + col_x[c]
        y = topb + r * (ch + gap)
        cw = col_w[c]
        focus = it.get("role") == "focus"
        fill = style.text("color.accent-bg") if focus else style.text("color.box-fill")
        stroke = style.text("color.accent") if focus else style.text("color.box-stroke")
        body.append(f'<rect x="{x}" y="{y}" width="{cw}" height="{ch}" rx="{style.num("size.radius")}" '
                    f'fill="{fill}" stroke="{stroke}"/>')
        body.append(_t(x + pad_x, y + ch / 2 + fs * style.num("font.baseline-ratio"), it["name"], style,
                       "color.accent" if focus else "color.ink", "start", size=fs))
        if it.get("depth") is not None:
            body.append(_t(x + cw - gap, y + ch / 2 + fs_small * style.num("font.baseline-ratio"),
                           it["depth"], style, "color.ink-faint", "end", size=fs_small))
    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)
