"""名前を添えた部品 ── 別の部品を包み、その上に名前を載せる。

節点として使われる部品は、渡された名前を描かなければならない。描かないと、
図の中でその節点が何なのかが読めなくなる（階層の中へ円グラフを置いたところ、
「内訳」という名前がどこにも出ず、階層として読めなくなった実例がある）。

ただし「すべての部品が自分で名前を描くこと」を契約にすると、単独で使う前提の
部品にまで名前を描く責務が乗る。そこで包む側を1つ足す ── 中身は既存の部品を
台帳越しに呼ぶだけなので、部品を足しても包む側は変わらない。

辺の接続先は、包んだ姿のインクから導かれる。帯も中身も同じ断片の中にあるので、
上から来る辺は帯へ、横から来る辺は中身の縁へ着く。
"""
from __future__ import annotations

import html

from .tokens import Style
from .registry import Absolute, OwnOrigin, component, render_component, render_node
from .text import text_width


@component("titled")
def titled(props: dict, style: Style) -> OwnOrigin:
    """別の部品を、名前の帯つきで描く。

    props: label（載せる名前）／of（包む部品の種別名）／
           その部品自身が要る props（そのまま渡す）

    Args:
        props: 上記の入力。
        style: resolve_style() が解決した見た目の値。

    Returns:
        ComponentResult。大きさは名前の帯を含む。

    Raises:
        KeyError: of が台帳に無い種別名のとき。
    """
    inner = render_node(props["of"], props, style)
    label = props.get("label", "")
    # 名前を描く役目は1箇所にしか置けない。中身が自分で描いたなら、包む側は
    # 描かない ── 両方が描くと同じ名前が二重に出る（box・hex で実測）。
    if not label or inner.labels_itself:
        return inner

    size = style.num("font.size-small")
    lead = size * style.num("size.label-line-h")
    pad_x = style.num("size.label-pad-x")
    # 帯の幅は、名前が収まる幅と中身の幅の大きいほう。名前がはみ出さない。
    w = max(inner.width, text_width(label, size) + pad_x * 2)
    h = inner.height + lead
    # 中身は帯のぶん下げ、横は中央へ寄せる
    dx = (w - inner.width) / 2
    # 名前は不透明な帯の上に載せる。名前は節点の一部なので動かせず、辺は
    # 曲げないので、辺の終端と名前は同じ場所を取り合う（実測：矢じりが名前の
    # 字面へ重なった）。動かせないラベルは帯で線を断つ ── 囲みの札と同じ扱い。
    # 帯は字面ぴったりに取る。節点の幅いっぱいへ広げると、名前より遥かに広い
    # 板が辺を丸ごと飲み込み、矢印が消えたように見える（実測で確認した）。
    # 字面は帯（lead）のまん中へ置く。ベースラインの位置をトークンの比で
    # 決めていたときは、字面の上端が y=-3.6 まではみ出していた ── 申告した
    # 大きさの外へインクが出ており、帯を描いて初めて見えた。ここでは lead と
    # 書体の比から毎回導くので、字面は必ず lead の内側に収まる。
    band_w = text_width(label, size) + pad_x * 2
    band_h = size * (style.num("font.cap-ratio") + style.num("font.descender-ratio"))
    band_top = (lead - band_h) / 2
    base = band_top + size * style.num("font.cap-ratio")
    svg = (
        f'<g>'
        f'<rect x="{(w - band_w) / 2:.1f}" y="{band_top:.1f}" '
        f'width="{band_w:.1f}" height="{band_h:.1f}" '
        f'fill="{style.text("color.box-fill")}"/>'
        f'<text x="{w / 2:.1f}" y="{base:.1f}" '
        f'text-anchor="middle" font-family="{style.text("font.family")}" '
        f'font-size="{size}" font-weight="{style.text("font.weight-medium")}" fill="{style.text("color.ink")}">'
        f'{html.escape(label)}</text>'
        f'<g transform="translate({dx:.1f},{lead:.1f})">{inner.svg}</g>'
        f'</g>'
    )
    # 輪郭は返さない。包んだ姿も、他の部品と同じく「描いたインクから導く」で
    # 済む ── 帯と中身を合わせた形は凹むが、中心から見た向きごとに外側を
    # 取る導出はそのまま通る。かつて中身の輪郭を引き継いだときは、帯のぶん
    # ずれた位置に辺が着き、矢じりが中身に隠れた（実測）。
    return OwnOrigin(svg=svg, width=w, height=h, labels_itself=True)
