"""装飾系の部品 ── 実務図には要らないが、1枚絵の完成度を上げるためだけの部品。

ここが「上限」側を担う。グラデーション・見出しの大きな活字・区切り・
簡単なアイコンは、どれも意味を持つ構造ではなく飾りなので、既存の
box/edge/pie等とは別の層として独立させる。混ぜても崩れない。
"""
from __future__ import annotations

import math
from html import escape as _e

from .ids import stable_id

# 波の1周期を4つに割る ── 上り・頂点・下り・谷という波の形そのもの。
_WAVE_QUARTER = 4
from .tokens import Style
from .registry import Absolute, OwnOrigin, component


@component("gradient_rect")
def gradient_rect(props: dict, style: Style) -> OwnOrigin:
    """グラデーションで塗った矩形。背景や強調帯に使う。

    props: width, height／stops（[(割合0-1, 色), ...]。既定はテーマの
    accentから薄い方へ）／direction（"h"|"v"|"radial"、既定"v"）／radius（角丸、既定0）
    """
    w, h = props["width"], props["height"]
    stops = props.get("stops") or [(0.0, style.text("color.accent")), (1.0, style.text("color.accent-bg"))]
    direction = props.get("direction", "v")
    radius = props.get("radius", 0)
    gid = stable_id("grad", stops, direction, radius, w, h)
    stop_svg = "".join(f'<stop offset="{o * 100:.0f}%" stop-color="{c}"/>' for o, c in stops)
    if direction == "radial":
        defs = f'<radialGradient id="{gid}" cx="50%" cy="50%" r="75%">{stop_svg}</radialGradient>'
    else:
        x2, y2 = ("100%", "0%") if direction == "h" else ("0%", "100%")
        defs = f'<linearGradient id="{gid}" x1="0%" y1="0%" x2="{x2}" y2="{y2}">{stop_svg}</linearGradient>'
    svg = (f'<defs>{defs}</defs>'
          f'<rect x="0" y="0" width="{w:.1f}" height="{h:.1f}" rx="{radius}" fill="url(#{gid})"/>')
    return OwnOrigin(svg=svg, width=w, height=h)


@component("title")
def title(props: dict, style: Style) -> OwnOrigin:
    """大きな見出しの活字。本文の書体(font.family)とは別の、表題用の書体を使う。

    props: text／subtitle（任意）／align（"start"|"middle"、既定"start"）
    """
    text = props["text"]
    size = style.num("font.size-display")
    family = style.text("font.family-display", style.text("font.family"))
    color = style.text("color.title", style.text("color.ink"))
    align = props.get("align", "start")
    w = props.get("width", style.num("size.decor-title-w"))
    anchor_x = w / 2 if align == "middle" else 0
    body = [f'<text x="{anchor_x}" y="{size:.0f}" text-anchor="{align}" '
           f'font-family="{family}" font-size="{size:.0f}" font-weight="{style.text("font.weight-bold")}" '
           f'letter-spacing="0.01em" fill="{color}">{_e(text)}</text>']
    h = size + size * style.num("font.baseline-ratio")
    if props.get("subtitle"):
        sub_size = style.num("font.size")
        lead = style.num("chart.gap")
        body.append(f'<text x="{anchor_x}" y="{size + sub_size + lead:.0f}" text-anchor="{align}" '
                    f'font-family="{style.text("font.family")}" font-size="{sub_size}" '
                    f'fill="{style.text("color.ink-soft")}">{_e(props["subtitle"])}</text>')
        h += sub_size + lead + sub_size * style.num("font.baseline-ratio")

    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)


@component("divider")
def divider(props: dict, style: Style) -> OwnOrigin:
    """区切り。飾りの波線／既定は直線。

    props: width／kind（"line"|"wave"、既定"line"）
    """
    w = props["width"]
    color = style.text("color.title", style.text("color.accent"))
    if props.get("kind") == "wave":
        amp = style.num("size.divider-amp")
        period = style.num("size.divider-period")
        pts = []
        x = 0.0
        while x <= w:
            pts.append((x, amp * math.sin(x / period * math.pi)))
            x += period / _WAVE_QUARTER
        d = "M" + " L".join(f"{x:.1f},{y + amp:.1f}" for x, y in pts)
        svg = f'<path d="{d}" fill="none" stroke="{color}" stroke-width="{style.num("size.rule-width")}"/>'
        h = amp * 2 + 2
    else:
        svg = f'<line x1="0" y1="1" x2="{w:.1f}" y2="1" stroke="{color}" stroke-width="{style.num("size.rule-width")}"/>'
        h = 2
    return OwnOrigin(svg=svg, width=w, height=h)


_ICON_PATHS = {
    # 0..24 の枠内、線幅2相当のシンプルな幾何形状。飾りの最小セット。
    "spark": "M12,1 L14.6,9.4 L23,12 L14.6,14.6 L12,23 L9.4,14.6 L1,12 L9.4,9.4 Z",
    "check": "M4,13 L10,19 L20,6",
    "ring": None,  # 円は path でなく circle で描く
    "arrow-up": "M12,20 L12,4 M5,11 L12,4 L19,11",
}


@component("icon")
def icon(props: dict, style: Style) -> OwnOrigin:
    """小さな飾りの記号。凝った画像ではなく、線1本ぶんの意匠。

    props: name（"spark"|"check"|"ring"|"arrow-up"）／size（既定24）
    """
    name = props.get("name", "spark")
    size = props.get("size", style.num("size.decor-icon"))
    color = style.text("color.title", style.text("color.accent"))
    scale = size / style.num("size.decor-icon")
    if name == "ring":
        body = f'<circle cx="12" cy="12" r="9" fill="none" stroke="{color}" stroke-width="{style.num("size.stroke-width-icon")}"/>'
    elif name == "spark":
        body = f'<path d="{_ICON_PATHS["spark"]}" fill="{color}"/>'
    else:
        body = (f'<path d="{_ICON_PATHS[name]}" fill="none" stroke="{color}" '
               f'stroke-width="{style.num("size.stroke-width-icon")}" stroke-linecap="round" stroke-linejoin="round"/>')
    svg = f'<g transform="scale({scale:.3f})">{body}</g>'
    return OwnOrigin(svg=svg, width=size, height=size)
