"""素の文字 ── 箱にもラベルにも属さない、任意の位置へ置く1つ。

**この部品が無いと、注記も、見出しの添えも、図の中の一言も置けない。**
人などの絵記号は `icon` が持つ ── 同じ枠と同じ線の決まりで描くものを、
2つの仕組みに分けない。

契約（asset-authoring-contract-for-component-svg-engines）に従い、
props は中身と構造だけを持ち、色も寸法もトークンから引く。
"""
from __future__ import annotations

from html import escape as _e

from .registry import OwnOrigin, component
from .text import text_width
from .tokens import TONES as _TONE, Style

# 太さ ── 同じく、役割の名前からトークンへ。
_WEIGHT = {
    "normal": "font.weight-normal",
    "medium": "font.weight-medium",
    "bold": "font.weight-bold",
}


def _lines(value) -> list[str]:
    """1行でも、行の並びでもよい。**呼ぶ側に形を揃えさせない。**"""
    if isinstance(value, (list, tuple)):
        return [str(v) for v in value]
    return [str(value)]


@component("text")
def text(props: dict, style: Style) -> OwnOrigin:
    """任意の位置へ置く文字。1行でも、行の並びでもよい。

    props: text（文字、または行の並び）／size（既定は font.size）／
           weight（"normal"|"medium"|"bold"）／tone（_TONE の鍵）／
           align（"start"|"middle"|"end"。**行どうしの揃え方である**）

    **インクは申告した大きさの中に収める。** 揃え方を変えても外接矩形は動かない
    ── 動かすと、置いた側が知らないところで重なりが起きる。
    """
    lines = _lines(props["text"])
    size = props.get("size", style.num("font.size"))
    color = style.text(_TONE[props.get("tone", "ink")], style.text("color.ink"))
    weight = style.text(_WEIGHT[props.get("weight", "normal")])
    align = props.get("align", "start")
    family = style.text("font.family")

    line_h = size * style.num("size.label-line-h")
    w = max(text_width(s, size) for s in lines)
    anchor_x = {"start": 0.0, "middle": w / 2, "end": w}[align]
    cap = size * style.num("font.cap-ratio")

    body = "".join(
        f'<text x="{anchor_x:.1f}" y="{cap + i * line_h:.1f}" text-anchor="{align}" '
        f'font-family="{family}" font-size="{size:.1f}" font-weight="{weight}" '
        f'fill="{color}">{_e(s)}</text>' for i, s in enumerate(lines))
    h = cap + (len(lines) - 1) * line_h + size * style.num("font.descender-ratio")
    return OwnOrigin(svg=f"<g>{body}</g>", width=w, height=h)
