"""装飾系の部品 ── 実務図には要らないが、1枚絵の完成度を上げるためだけの部品。

ここが「上限」側を担当する。グラデーション・見出しの大きな活字・区切り・
簡単なアイコンは、どれも意味を持つ構造ではなく飾りなので、既存の
box/edge/pie等とは別の層として独立させる。混ぜても破綻しない。
"""
from __future__ import annotations

import math
from html import escape as _e

from .ids import stable_id

# 波の1周期を4つに割る ── 上り・頂点・下り・谷という波の形そのもの。
_WAVE_QUARTER = 4
from .tokens import TONES, Style
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


# 絵記号の定義。**どれも 0..24 の同じ枠で描く** ── 大きさの意味が絵ごとに違うと、
# 並べたときに揃わない。線だけのものは paths、面を持つものは circles を併せ持つ。
# **対応表であって、式の中の数ではない。**
_PICTOGRAMS: dict[str, dict] = {
    "spark": {"fill": "M12,1 L14.6,9.4 L23,12 L14.6,14.6 L12,23 L9.4,14.6 L1,12 L9.4,9.4 Z"},
    "check": {"paths": ["M4,13 L10,19 L20,6"]},
    "ring": {"circles": [(12.0, 12.0, 9.0, None)]},
    "arrow-up": {"paths": ["M12,20 L12,4 M5,11 L12,4 L19,11"]},
    "doc": {"paths": ["M5,2 L15,2 L19,6 L19,22 L5,22 Z M15,2 L15,6 L19,6 "
                      "M8,11 L16,11 M8,15 L16,15 M8,19 L13,19"]},
    "screen": {"paths": ["M2,4 L22,4 L22,17 L2,17 Z M12,17 L12,21 M7,21 L17,21"]},
    "chat": {"paths": ["M2,3 L22,3 L22,15 L10,15 L5,20 L5,15 L2,15 Z "
                       "M6,7 L18,7 M6,11 L14,11"]},
    "book": {"paths": ["M2,4 C7,2 10,2 12,4 C14,2 17,2 22,4 L22,20 "
                       "C17,18 14,18 12,20 C10,18 7,18 2,20 Z M12,4 L12,20"]},
    "calendar": {"paths": ["M3,5 L21,5 L21,21 L3,21 Z M3,10 L21,10 M8,2 L8,7 "
                           "M16,2 L16,7 M7,14 L11,14 M14,14 L18,14 M7,18 L11,18"]},
    "branch": {"paths": ["M2,6 L10,6 L10,18 L22,18 M10,6 L22,6 M10,12 L22,12"]},
    "search": {"circles": [(10.0, 10.0, 7.0, None)], "paths": ["M15,15 L21,21"]},
    "gear": {"fill": "M12,3 L14,3 L14.6,6 L17,7.4 L19.6,6.4 L20.6,8.2 L18.6,10.2 "
                     "L18.6,13 L20.6,15 L19.6,16.8 L17,15.8 L14.6,17.2 L14,20 L12,20 "
                     "L10,20 L9.4,17.2 L7,15.8 L4.4,16.8 L3.4,15 L5.4,13 L5.4,10.2 "
                     "L3.4,8.2 L4.4,6.4 L7,7.4 L9.4,6 L10,3 Z"},
    # 人 ── 頭は面を持つ絵記号である。**別の部品にしない** ── 同じ枠・同じ線の
    # 決まりで描くものを2つの仕組みに分けると、片方だけが直る。
    "person": {"circles": [(10.67, 5.33, 5.33, "fill")],
               "paths": ["M0,24 L0,21.33 C0,9.33 21.33,9.33 21.33,21.33 L21.33,24"]},
}


@component("icon")
def icon(props: dict, style: Style) -> OwnOrigin:
    """絵記号。線1本ぶんの意匠で、写実ではない。

    props: name（`icon_names()` が返す名前）／size（既定 size.decor-icon）／
           tone（線の濃さの呼び名。省くと見出しの色）

    **どれも 0..24 の同じ枠で描く** ── 大きさの意味が絵ごとに違うと、
    並べたときに揃わない。**拡大しても線は太らない。**
    """
    name = props.get("name", "spark")
    if name not in _PICTOGRAMS:
        raise ValueError(f"知らない絵の名前: {name}。使えるのは {sorted(_PICTOGRAMS)} である")
    size = props.get("size", style.num("size.decor-icon"))
    tone = props.get("tone")
    color = (style.text(TONES[tone]) if tone
             else style.text("color.title", style.text("color.accent")))
    scale = size / style.num("size.decor-icon")
    width = style.num("size.stroke-width-icon") / scale
    line = (f'stroke="{color}" stroke-width="{width:.3f}" '
            f'stroke-linecap="round" stroke-linejoin="round"')

    spec = _PICTOGRAMS[name]
    body = []
    if "fill" in spec:
        body.append(f'<path d="{spec["fill"]}" fill="{color}"/>')
    for cx, cy, r, fill_tone in spec.get("circles", []):
        fill = style.text(TONES[fill_tone]) if fill_tone else "none"
        body.append(f'<circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}" {line}/>')
    for d in spec.get("paths", []):
        body.append(f'<path d="{d}" fill="none" {line}/>')
    svg = f'<g transform="scale({scale:.3f})">{"".join(body)}</g>'
    return OwnOrigin(svg=svg, width=size, height=size)


def icon_names() -> list[str]:
    """描ける絵の名前。**目録がここから引く** ── 一覧を2か所に持たない。"""
    return sorted(_PICTOGRAMS)
