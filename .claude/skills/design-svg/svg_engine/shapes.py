"""組み込みの部品 ── どれも「構造だけ」を受け取り、色・寸法はstyleから引く。

部品を1つ増やしたいときは、このファイルを直さず、別ファイルに
`@component("新しい名前")` を書いて import すれば足りる（registry.py参照）。

**2系統あるのは、描く時点が置く前か後かで違うから。** 節点系（box等）は
置かれる前に描くので、自分がどこへ置かれるかを知らない ── だから自分の
原点(0,0)を基準に描き、申告する大きさは「自分を囲む器」になる。辺・囲み・
囲みの札は、既に置かれた節点をまたぐので、置いた後にしか描けない ── だから
最初から絶対座標を受け取り、申告する大きさは器ではなく既に決まった広がりの
記録でしかない。`ComponentResult.placement` がこの違いを持つ。
"""
from __future__ import annotations

import math
from html import escape as _e

from .tokens import Style
from .registry import Absolute, OwnOrigin, component
from .text import text_width as _text_width


# ── 節点系 ──────────────────────────────────────────────


def arrow_head(tip: tuple[float, float], angle: float, style: Style,
               color: str | None = None) -> str:
    """矢じりを1つ描く。向きは角度で受け取る。

    形はトークンから決まる（幅の比・長さの比・開き角）。ここを部品ごとに
    書くと、テーマで矢じりを大きくしても一部だけ変わらない ── 実際に、
    やり取りの図の矢じりだけがテーマに追随しなかった。

    Args:
        tip: 矢じりの先端。
        angle: 進む向き（ラジアン）。
        style: resolve_style() が解決した見た目の値。
        color: 塗る色。省略すると線の色。

    Returns:
        <polygon> の断片。

    Raises:
        なし。
    """
    x2, y2 = tip
    head_w = max(style.num("size.stroke-width") * style.num("size.arrowhead-w-ratio"),
                 style.num("size.arrowhead-min"))
    head_len = head_w * style.num("size.arrowhead-len-ratio")
    spread = math.radians(style.num("size.arrowhead-angle"))
    hx1 = x2 - head_len * math.cos(angle - spread)
    hy1 = y2 - head_len * math.sin(angle - spread)
    hx2 = x2 - head_len * math.cos(angle + spread)
    hy2 = y2 - head_len * math.sin(angle + spread)
    fill = color or style.text("color.line", style.text("color.ink-faint"))
    return (f'<polygon points="{x2:.1f},{y2:.1f} {hx1:.1f},{hy1:.1f} '
            f'{hx2:.1f},{hy2:.1f}" fill="{fill}"/>')


@component("box")
def box(props: dict, style: Style) -> OwnOrigin:
    """名前を1つ持つ、角丸の矩形。つながり・階層・包含などの節点に使う。"""
    label = str(props.get("label", ""))
    font_size = style.num("font.size")
    pad_x = style.num("size.box-pad-x")
    w = max(style.num("size.box-min-w"),
            _text_width(label, font_size, style.num("font.latin-width-ratio")) + pad_x * 2)
    h = style.num("size.box-h")
    radius = style.num("size.box-radius")
    fill = style.text("color.box-fill")
    stroke = style.text("color.box-stroke")
    sw = style.num("size.stroke-width")
    dash = style.opt_text("stroke-dasharray")
    dash_attr = f' stroke-dasharray="{dash}"' if dash else ""
    text_color = style.text("color.text", style.text("color.ink"))
    weight = style.text("font.weight", "400")
    svg = (
        f'<g class="svg-box" role="{_e(str(props.get("role", "plain")))}">'
        f'<rect x="0" y="0" width="{w:.1f}" height="{h:.1f}" rx="{radius}" '
        f'fill="{fill}" stroke="{stroke}" stroke-width="{sw}"{dash_attr}/>'
        f'<text x="{w / 2:.1f}" y="{h / 2 + font_size * style.num("font.baseline-ratio"):.1f}" '
        f'text-anchor="middle" font-family="{style.text("font.family")}" '
        f'font-size="{font_size}" font-weight="{weight}" fill="{text_color}">'
        f'{_e(label)}</text></g>'
    )
    return OwnOrigin(svg=svg, width=w, height=h, labels_itself=True)


@component("dot")
def dot(props: dict, style: Style) -> OwnOrigin:
    """始点・終点の印などに使う、塗りつぶした小さな円。"""
    r = props.get("radius", style.num("size.dot-radius"))
    fill = style.text("color.text", style.text("color.ink"))
    svg = f'<circle cx="{r:.1f}" cy="{r:.1f}" r="{r:.1f}" fill="{fill}"/>'
    return OwnOrigin(svg=svg, width=r * 2, height=r * 2)


# ── 関係系（絶対座標を受け取って描く） ──────────────────────

def _smooth_path(points: list[tuple[float, float]]) -> str:
    """点列を、角を丸めたパスへ変換する。2点なら直線のまま。

    3点以上（複数段をまたぐ辺が仮節点を経由した場合）は、各中間点を
    二次ベジェで滑らかにつなぐ。折れ線のまま出すより、辺だと分かりやすい。
    """
    if len(points) < 3:
        (x1, y1), (x2, y2) = points[0], points[-1]
        return f'M{x1:.1f},{y1:.1f} L{x2:.1f},{y2:.1f}'
    d = [f'M{points[0][0]:.1f},{points[0][1]:.1f}']
    for i in range(1, len(points) - 1):
        px, py = points[i]
        nx, ny = points[i + 1]
        mx, my = (px + nx) / 2, (py + ny) / 2
        d.append(f'Q{px:.1f},{py:.1f} {mx:.1f},{my:.1f}')
    lx, ly = points[-1]
    d.append(f'L{lx:.1f},{ly:.1f}')
    return " ".join(d)


@component("edge")
def edge(props: dict, style: Style) -> Absolute:
    """複数の点を通って結ばれる線。矢じり・ラベル・破線を持てる。

    props: points（絶対座標の(x,y)の並び。2点なら直線、3点以上なら
           途中の仮節点を通る滑らかな経路になる）／label（任意）／
           label_at（(x,y)、任意。他の辺との重なりを避けて呼び出し側が
           決めた置き場所。省略時はこの部品自身が経路の真ん中を使う）／
           arrow（"none"|"head"、既定"head"）／dashed（bool、既定False）
    """
    points: list[tuple[float, float]] = props["points"]
    x1, y1 = points[0]
    x2, y2 = points[-1]
    color = style.text("color.line", style.text("color.ink-faint"))
    sw = style.num("size.stroke-width")
    dash = ' stroke-dasharray="4 3"' if props.get("dashed") else ""
    path_d = _smooth_path(points)
    marker = ""
    if props.get("arrow", "head") != "none":
        # 矢じりの向きは、実際に終点へ入る最後の線分の傾きから決める
        # （3点以上のときは、直前の中点から終点への直線がその線分にあたる）。
        if len(points) >= 3:
            px = (points[-2][0] + points[-1][0]) / 2
            py = (points[-2][1] + points[-1][1]) / 2
        else:
            px, py = points[-2]
        ang = math.atan2(y2 - py, x2 - px)
        marker = arrow_head((x2, y2), ang, style, color)
    label_svg = ""
    if props.get("label"):
        if props.get("label_at"):
            mx, my = props["label_at"]
        else:
            mi = len(points) // 2
            mx, my = points[mi] if len(points) % 2 else (
                (points[mi - 1][0] + points[mi][0]) / 2, (points[mi - 1][1] + points[mi][1]) / 2)
        fs = style.num("font.size-small")
        text_w = _text_width(props["label"], fs,
                             style.num("font.latin-width-ratio")) + style.num("size.label-pad-x")
        label_svg = (
            f'<rect x="{mx - text_w / 2:.1f}" y="{my - fs:.1f}" width="{text_w:.1f}" '
            f'height="{style.num("size.label-band-h"):.1f}" fill="{style.text("color.box-fill")}"/>'
            f'<text x="{mx:.1f}" y="{my + fs * style.num("font.baseline-ratio"):.1f}" text-anchor="middle" '
            f'font-family="{style.text("font.family")}" font-size="{fs}" '
            f'fill="{style.text("color.ink-faint")}">{_e(props["label"])}</text>'
        )
    svg = f'<path d="{path_d}" fill="none" stroke="{color}" stroke-width="{sw}"{dash}/>{marker}{label_svg}'
    xs = [p[0] for p in points]
    ys = [p[1] for p in points]
    return Absolute(svg=svg, width=max(xs) - min(xs), height=max(ys) - min(ys))


@component("frame_label")
def frame_label(props: dict, style: Style) -> Absolute:
    """囲みの札だけを描く。枠線とは別の層に置くための部品。

    札は不透明な帯を持つので、線の上に載れば線を断って読める。避けられる
    場所があるなら避けるが、無いときは載る ── どちらにせよ線は曲げない。

    props: x（札の左端の絶対座標）／y（枠線の上辺の絶対座標）／label

    Args:
        props: 上記の入力。
        style: resolve_style() が解決した見た目の値。

    Returns:
        ComponentResult。

    Raises:
        なし。
    """
    fs = style.num("font.size-small")
    pad_x = style.num("size.label-pad-x")
    text_w = _text_width(props["label"], fs, style.num("font.latin-width-ratio")) + pad_x
    rise = fs * style.num("size.frame-label-rise")
    card_h = fs * style.num("size.frame-label-h")
    base = fs * style.num("size.frame-label-baseline")
    x, y = props["x"], props["y"]
    svg = (f'<rect x="{x:.1f}" y="{y - rise:.1f}" width="{text_w:.1f}" '
           f'height="{card_h:.1f}" fill="{style.text("color.box-fill")}"/>'
           f'<text x="{x + text_w / 2:.1f}" y="{y - rise + base:.1f}" '
           f'text-anchor="middle" font-family="{style.text("font.family")}" font-size="{fs}" '
           f'fill="{style.text("color.accent")}">{_e(props["label"])}</text>')
    return Absolute(svg=svg, width=text_w, height=card_h)


@component("frame")
def frame(props: dict, style: Style) -> Absolute:
    """区画を示す破線の囲み。包含や『ここは領域の内側』のような注記に使う。

    札は描かない ── 札は線より後に描く必要があり、置き場所も線を避けて決まる
    ので、frame_label という別の部品が受け持つ。同じものを2か所で描かない。

    props: x, y, width, height（絶対座標）

    Args:
        props: 上記の入力。
        style: resolve_style() が解決した見た目の値。

    Returns:
        ComponentResult。

    Raises:
        なし。
    """
    x, y, w, h = props["x"], props["y"], props["width"], props["height"]
    svg = (f'<rect x="{x:.1f}" y="{y:.1f}" width="{w:.1f}" height="{h:.1f}" rx="{style.num("size.radius-large")}" '
           f'fill="none" stroke="{style.text("color.accent")}" stroke-width="{style.num("size.stroke-width-thin")}" '
           f'stroke-dasharray="5 4" opacity="{style.num("opacity.soft")}"/>')
    return Absolute(svg=svg, width=w, height=h)


