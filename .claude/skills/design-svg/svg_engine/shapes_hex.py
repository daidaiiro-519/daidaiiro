"""試しに足す部品 ── 六角形の節点。

コアを触らずに足せるかを測るために作った。台帳へ登録し、テーマの
parts.node をこの名前へ向ければ、節点が六角形になる。
"""
from __future__ import annotations

from html import escape as _e

from .tokens import Style
from .registry import Absolute, OwnOrigin, component
from .text import text_width as _text_width


@component("hex")
def hexagon(props: dict, style: Style) -> OwnOrigin:
    """名前を1つ持つ六角形。box と同じ契約（構造だけ受け取り、見た目はstyleから）。"""
    label = str(props.get("label", ""))
    fs = style.num("font.size")
    pad_x = style.num("size.box-pad-x")
    h = style.num("size.box-h")
    w = max(style.num("size.box-min-w"),
            _text_width(label, fs, style.num("font.latin-width-ratio")) + pad_x * 2 + h)
    cut = h / 2   # 斜めに切り落とす幅。左右で h/2 ずつ要るので幅にも足す
    pts = f"{cut},0 {w - cut},0 {w},{h / 2} {w - cut},{h} {cut},{h} 0,{h / 2}"
    svg = (f'<g class="svg-box">'
           f'<polygon points="{pts}" fill="{style.text("color.box-fill")}" '
           f'stroke="{style.text("color.box-stroke")}" stroke-width="{style.num("size.stroke-width")}"/>'
           f'<text x="{w / 2:.1f}" y="{h / 2 + fs * style.num("font.baseline-ratio"):.1f}" '
           f'text-anchor="middle" font-family="{style.text("font.family")}" font-size="{fs}" '
           f'fill="{style.text("color.text", style.text("color.ink"))}">{_e(label)}</text></g>')
    return OwnOrigin(svg=svg, width=w, height=h, labels_itself=True)
