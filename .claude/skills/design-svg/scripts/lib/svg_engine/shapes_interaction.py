"""やり取り専用の部品 ── 参加者ごとの縦のライフラインと、段ごとに行き来する
横向きのメッセージ。Mermaidのsequence diagramに相当する。

「順序」（`box`をrank/orderで並べるだけ）や「つながり」の汎用グラフでは、
『誰と誰の間か』という軸を表現できない。やり取りだけがこの軸を持つので、
専用の部品として起こす。

分かれ(cases)を持つ囲みは、ケースの見出しぶんの高さを行として確保してから
段を並べる。見出しをメッセージの行へ後から重ね書きすると、実際に描いて
文字と線が衝突する不具合が出た（このファイルの最初の実装で発生）。

寸法はすべて style（tokens.py のトークン）から引く。直書きしていたときは、
テーマを差し替えても寸法だけ取り残される不一致があった。
"""
from __future__ import annotations

import math

from html import escape as _e

from .tokens import Style
from .registry import Absolute, OwnOrigin, component
from .shapes import arrow_head
from .text import column_width, text_width


@component("exchange")
def exchange(props: dict, style: Style) -> OwnOrigin:
    """参加者の縦のライフラインと、段ごとのメッセージ。

    props:
        participants: [str, ...] ── 左から並べる順。
        steps: [{"from": str, "to": str, "label": str(任意),
                 "kind": "call"(既定)|"return"}, ...] ── 上から順に描く段。
        groups: [{"label": str, "span": [開始段, 終了段]}, ...]（任意）
                または cases を持つ分かれ:
                [{"label": str, "cases": [{"name": str, "span": [s, e]}, ...]}]
    """
    who = props["participants"]
    steps = props["steps"]
    groups = props.get("groups", [])

    pad = style.num("chart.pad")
    colw = style.num("chart.exchange-col-w")
    head_h = style.num("chart.exchange-head-h")
    row_h = style.num("chart.exchange-row-h")
    case_head_h = style.num("chart.exchange-case-head-h")
    # 参加者の箱の幅は、入る名前から決める（決め打ちだと長い名前がはみ出す）
    box_h = style.num("chart.exchange-box-h")
    arrow_len = style.num("chart.exchange-arrow-len")
    fs = style.num("font.size")
    fs_small = style.num("font.size-small")
    box_w = column_width(who, fs, style.num("chart.gap") * 2)
    w = pad * 2 + colw * len(who)

    # ── 段ごとの縦位置を、分かれの見出し行ぶんも織り込んで先に確定させる ──
    case_header_at: dict[int, list[str]] = {}
    for g in groups:
        for c in g.get("cases", []):
            case_header_at.setdefault(c["span"][0], []).append(c["name"])

    row_top: dict[int, float] = {}
    header_top: dict[int, list[tuple[float, str]]] = {}
    cursor = pad + head_h
    for idx in range(len(steps)):
        if idx in case_header_at:
            slots = []
            for name in case_header_at[idx]:
                slots.append((cursor, name))
                cursor += case_head_h
            header_top[idx] = slots
        row_top[idx] = cursor
        cursor += row_h
    h = cursor + pad / 2

    def x_of(i: int) -> float:
        return pad + colw * i + colw / 2

    def mid_of(idx: int) -> float:
        return row_top[idx] + row_h / 2

    body = []

    # ── 囲み(groups) ── ライフラインより手前、メッセージより奥に描く ──
    tab_h = style.num("chart.exchange-tab-h")
    base = style.num("font.baseline-ratio")
    cap = style.num("font.cap-ratio")
    gap = style.num("chart.gap")
    for g in groups:
        cases = g.get("cases")
        spans = [c["span"] for c in cases] if cases else [g["span"]]
        s = min(sp[0] for sp in spans)
        e = max(sp[1] for sp in spans)
        top = (header_top[s][0][0] - tab_h / 2) if s in header_top else (row_top[s] - tab_h)
        bottom = row_top[e] + row_h - gap
        body.append(f'<rect x="{pad}" y="{top:.1f}" width="{w - pad * 2:.1f}" '
                    f'height="{bottom - top:.1f}" rx="{style.num("size.radius-small")}" fill="none" '
                    f'stroke="{style.text("color.accent")}"/>')
        tab_w = min(text_width(g["label"], fs_small - 1) + fs_small, w - pad * 2)
        body.append(f'<rect x="{pad}" y="{top:.1f}" width="{tab_w:.1f}" height="{tab_h:.1f}" '
                    f'fill="{style.text("color.accent-bg")}"/>')
        body.append(f'<text x="{pad + gap / 2}" y="{top + tab_h / 2 + fs_small * base:.1f}" '
                    f'font-family="{style.text("font.family")}" '
                    f'font-size="{fs_small - 1}" fill="{style.text("color.accent")}">'
                    f'{_e(g["label"])}</text>')
        if cases:
            for i, c in enumerate(cases):
                head_y = next(y for y, n in header_top[c["span"][0]] if n == c["name"])
                if i > 0:
                    body.append(f'<line x1="{pad}" y1="{head_y:.1f}" x2="{w - pad}" y2="{head_y:.1f}" '
                                f'stroke="{style.text("color.accent")}" stroke-dasharray="5 4"/>')
                body.append(f'<text x="{pad + fs_small}" y="{head_y + case_head_h / 2 + fs_small * base:.1f}" '
                            f'font-weight="{style.text("font.weight-medium")}" font-family="{style.text("font.family")}" '
                            f'font-size="{fs}" fill="{style.text("color.ink")}">{_e(c["name"])}</text>')

    # ── 参加者の箱とライフライン ── メッセージの下敷きになるので、後に描く ──
    for i, name in enumerate(who):
        cx = x_of(i)
        body.append(f'<line x1="{cx:.1f}" y1="{pad + box_h:.1f}" x2="{cx:.1f}" y2="{h - pad / 2:.1f}" '
                    f'stroke="{style.text("color.box-stroke")}" stroke-dasharray="3 4"/>')
    for i, name in enumerate(who):
        cx = x_of(i)
        body.append(f'<rect x="{cx - box_w / 2:.1f}" y="{pad}" width="{box_w}" height="{box_h}" rx="{style.num("size.radius")}" '
                    f'fill="{style.text("color.box-fill")}" stroke="{style.text("color.box-stroke")}"/>')
        body.append(f'<text x="{cx:.1f}" y="{pad + box_h / 2 + fs_small * base:.1f}" text-anchor="middle" '
                    f'font-family="{style.text("font.family")}" font-size="{fs}" '
                    f'fill="{style.text("color.ink")}">{_e(name)}</text>')

    # ── メッセージ ── 一番手前。
    for r, s in enumerate(steps):
        y = mid_of(r)
        a, b = who.index(s["from"]), who.index(s["to"])
        xa, xb = x_of(a), x_of(b)
        dashed = s.get("kind") == "return"
        dash_attr = ' stroke-dasharray="4 3"' if dashed else ""
        body.append(f'<line x1="{xa:.1f}" y1="{y:.1f}" x2="{xb:.1f}" y2="{y:.1f}" '
                    f'stroke="{style.text("color.ink-faint")}" stroke-width="{style.num("size.stroke-width")}"'
                    f'{dash_attr}/>')
        # 矢じりの底辺は、先端より「来た方」へ置く。進む方へ置くと三角が逆を向く
        # （実測で全部の矢印が逆向きになっていた）。
        back = -1 if xb > xa else 1
        # 矢じりは辺の部品と同じ作り方をする。ここで自前の三角形を描くと、
        # テーマで矢じりを大きくしてもこの図だけ変わらない（実測で確認）。
        body.append(arrow_head((xb, y), math.pi if back > 0 else 0.0, style,
                               style.text("color.ink-faint")))
        if s.get("label"):
            mx = (xa + xb) / 2
            body.append(f'<text x="{mx:.1f}" y="{y - gap / 2:.1f}" text-anchor="middle" '
                        f'font-family="{style.text("font.family")}" font-size="{fs_small}" '
                        f'fill="{style.text("color.ink-soft")}">{_e(s["label"])}</text>')

    return OwnOrigin(svg=f'<g>{"".join(body)}</g>', width=w, height=h)
