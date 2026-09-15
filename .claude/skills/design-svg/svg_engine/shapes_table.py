"""表(table)部品 ── **絵の中に表が要るときだけ**使う。

表は絵である必要がない。文字として出せるなら、成果物の書式（Markdown・HTML）で
書いたほうが読める ── 選択でき、折り返し、幅に追随する。ここが受け持つのは、
表が絵の一部でなければならない場合だけである（他の図と1枚に組む、セルの中に
図が入る、画像として配る）。

そのうえで、この部品は契約(asset-authoring-contract-for-component-svg-engines)を
実際になぞって追加したものでもある。

- 節点系(自分の原点(0,0)基準)として作る（契約2）
- props は headers/rows という構造だけを持ち、色・寸法は一切書かない（契約3）
- 列幅はセルの文字幅から動的に決める。固定pxにしない（契約4。spatial部品で
  固定幅が原因の不具合を踏んだのと同じ轍を踏まないため）
- テキストはhtml.escapeを通す（契約7）
- <svg>ルートタグは持たない。<g>フラグメントだけを返す（契約5）
- 呼ぶ側の語彙は知らない。受け取るのは見出しと行だけ（契約6）
"""
from __future__ import annotations

from html import escape as _e

from .tokens import Style
from .registry import Absolute, OwnOrigin, component
from .text import text_width


@component("table")
def table(props: dict, style: Style) -> OwnOrigin:
    """見出し行つきの表。

    props: headers（[str, ...]）／rows（[[str, ...], ...]。各行はheadersと同じ列数）／
           axes（[縦が何を表すか, 横が何を表すか]。任意）

    軸の名前を持てるのは、縦横が何を表すかが絵に出ないと、交点に何が来るかを
    読めないため。呼ぶ側がその名前を必要とするかどうかは、呼ぶ側が決める。
    """
    headers = props["headers"]
    rows = props["rows"]
    axes = props.get("axes") or []
    fs = style.num("font.size")
    fs_small = style.num("font.size-small")
    pad_x = style.num("chart.table-pad-x")
    row_h = style.num("chart.table-row-h")

    col_w = []
    for c, head in enumerate(headers):
        widest = text_width(str(head), fs_small)
        for r in rows:
            widest = max(widest, text_width(str(r[c]), fs))
        col_w.append(widest + pad_x * 2)

    col_x = [sum(col_w[:c]) for c in range(len(headers))]
    grid_w = sum(col_w)
    # 軸の名前は表の外側に置く。縦の名前は左へ回し、横の名前は上へ載せる。
    # 帯の厚みは書体から導く（決め打ちを置かない）。
    band = fs_small * style.num("size.label-line-h") if axes else 0.0
    left = band if len(axes) > 0 else 0.0
    top = band if len(axes) > 1 else 0.0
    w = grid_w + left
    h = row_h * (len(rows) + 1) + top

    body = [f'<g transform="translate({left:.1f},{top:.1f})">'
            f'<rect x="0" y="0" width="{grid_w:.1f}" height="{row_h:.1f}" '
            f'fill="{style.text("color.accent-bg")}"/>']
    for c, head in enumerate(headers):
        body.append(f'<text x="{col_x[c] + pad_x:.1f}" y="{row_h / 2 + fs_small * style.num("font.baseline-ratio"):.1f}" '
                    f'font-family="{style.text("font.family")}" font-size="{fs_small}" '
                    f'font-weight="{style.text("font.weight-medium")}" fill="{style.text("color.accent")}">{_e(str(head))}</text>')

    for r_idx, row in enumerate(rows):
        y = row_h * (r_idx + 1)
        body.append(f'<line x1="0" y1="{y:.1f}" x2="{w:.1f}" y2="{y:.1f}" '
                    f'stroke="{style.text("chart.grid")}"/>')
        for c, cell in enumerate(row):
            body.append(f'<text x="{col_x[c] + pad_x:.1f}" y="{y + row_h / 2 + fs * style.num("font.baseline-ratio"):.1f}" '
                        f'font-family="{style.text("font.family")}" font-size="{fs}" '
                        f'fill="{style.text("color.ink")}">{_e(str(cell))}</text>')
    grid_h = row_h * (len(rows) + 1)
    body.append(f'<rect x="0.5" y="0.5" width="{grid_w - 1:.1f}" height="{grid_h - 1:.1f}" '
               f'fill="none" stroke="{style.text("color.box-stroke")}"/></g>')
    base = fs_small * style.num("font.baseline-ratio")
    if len(axes) > 1:
        # 横が何を表すか ── 表の上、いちばん左の列に揃える
        body.append(f'<text x="{left:.1f}" y="{band / 2 + base:.1f}" '
                    f'font-family="{style.text("font.family")}" font-size="{fs_small}" '
                    f'fill="{style.text("color.ink-faint")}">{_e(str(axes[1]))}</text>')
    if len(axes) > 0:
        # 縦が何を表すか ── 表の左、90度回して縦書きにする
        cy = top + grid_h / 2
        body.append(f'<text x="{band / 2 + base - fs_small:.1f}" y="{cy:.1f}" text-anchor="middle" '
                    f'transform="rotate(-90 {band / 2 + base - fs_small:.1f} {cy:.1f})" '
                    f'font-family="{style.text("font.family")}" font-size="{fs_small}" '
                    f'fill="{style.text("color.ink-faint")}">{_e(str(axes[0]))}</text>')
    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)
