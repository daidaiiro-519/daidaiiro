"""自由曲線 ── ベジェも円弧も含む、任意のパスデータをそのまま渡せる部品。

「登録済みの部品の組み合わせでしか表現できない」という制約への回答が、
完成された形の部品を1つ足すことではなく、任意の形そのものを渡せる
この部品を足すこと。ペンツールのGUIは無いが、ペンツールが作るのと
同じ入力(パスデータ)を受け取れる。

**受けるのはSVGのパスの文法そのものである** ── 相対座標の小文字も、
H/V の水平垂直も、S/T の省略形も、A の円弧も、書いたまま渡せる。
描く側は座標を書き換えず、外接矩形ぶんだけ平行移動して置く。

契約(asset-authoring-contract-for-component-svg-engines)に従い、
props はパスの構造(d)と閉じるかどうかだけを持ち、色はstyleから引く。
"""
from __future__ import annotations

import math
import re

from .boolean import boolean_op
from .tokens import Style
from .registry import Absolute, OwnOrigin, component

_NUM = re.compile(r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?")
_CMD = re.compile(r"([MmLlHhVvCcSsQqTtAaZz])([^MmLlHhVvCcSsQqTtAaZz]*)")

# コマンドが1回に取る引数の個数。SVGのパスの文法が決めている ── 選んだ値ではない。
_ARITY = {"M": 2, "L": 2, "H": 1, "V": 1, "C": 6, "S": 4,
          "Q": 4, "T": 2, "A": 7, "Z": 0}

# 円弧を外接矩形へ含めるときの刻み（度）。細かくするほど外接矩形は小さくなる。
_ARC_STEP_DEG = 5.0


def _numbers(chunk: str) -> list[float]:
    return [float(x) for x in _NUM.findall(chunk)]


def _arc_points(x0: float, y0: float, rx: float, ry: float, rot: float,
                large: float, sweep: float, x1: float, y1: float) -> list[tuple[float, float]]:
    """円弧の通る点を刻んで返す。**外接矩形を出すためだけに使う。**

    端点だけでは足りない ── 弧は端点の外側へ膨らむ。描く側は d を書き換えず、
    この点の並びは大きさの申告にしか使わない。
    """
    if rx == 0 or ry == 0 or (x0, y0) == (x1, y1):
        return [(x1, y1)]
    phi = math.radians(rot)
    cos_p, sin_p = math.cos(phi), math.sin(phi)
    dx2, dy2 = (x0 - x1) / 2, (y0 - y1) / 2
    x1p, y1p = cos_p * dx2 + sin_p * dy2, -sin_p * dx2 + cos_p * dy2
    rx, ry = abs(rx), abs(ry)
    lam = (x1p / rx) ** 2 + (y1p / ry) ** 2
    if lam > 1:
        rx, ry = rx * math.sqrt(lam), ry * math.sqrt(lam)
    num = rx ** 2 * ry ** 2 - rx ** 2 * y1p ** 2 - ry ** 2 * x1p ** 2
    den = rx ** 2 * y1p ** 2 + ry ** 2 * x1p ** 2
    coef = math.sqrt(max(num, 0) / den) if den else 0.0
    if large == sweep:
        coef = -coef
    cxp, cyp = coef * rx * y1p / ry, -coef * ry * x1p / rx
    cx = cos_p * cxp - sin_p * cyp + (x0 + x1) / 2
    cy = sin_p * cxp + cos_p * cyp + (y0 + y1) / 2
    start = math.atan2((y1p - cyp) / ry, (x1p - cxp) / rx)
    end = math.atan2((-y1p - cyp) / ry, (-x1p - cxp) / rx)
    sweep_angle = end - start
    if sweep and sweep_angle < 0:
        sweep_angle += 2 * math.pi
    if not sweep and sweep_angle > 0:
        sweep_angle -= 2 * math.pi
    steps = max(1, int(abs(sweep_angle) / math.radians(_ARC_STEP_DEG)))
    out = []
    for i in range(steps + 1):
        t = start + sweep_angle * i / steps
        out.append((cx + rx * math.cos(t) * cos_p - ry * math.sin(t) * sin_p,
                    cy + rx * math.cos(t) * sin_p + ry * math.sin(t) * cos_p))
    return out


def path_points(d: str) -> list[tuple[float, float]]:
    """パスが通る点を、絶対座標で返す。

    **曲線は制御点も含める** ── 見積りは実物以上でなければならない。少なく
    見積もると、申告した大きさの外へインクが出て、置いた側が重ねてしまう。
    """
    unknown = set(re.findall(r"[A-DF-Za-df-z]", d)) - set("MmLlHhVvCcSsQqTtAaZz")
    if unknown:
        raise ValueError(f"パスの文法に無いコマンドがある: {sorted(unknown)}")
    out: list[tuple[float, float]] = []
    x = y = 0.0
    start_x = start_y = 0.0
    pos = 0
    for m in _CMD.finditer(d):
        if m.start() != pos and d[pos:m.start()].strip():
            raise ValueError(f"パスとして読めない断片がある: {d[pos:m.start()].strip()!r}")
        pos = m.end()
        cmd = m.group(1)
        upper = cmd.upper()
        rel = cmd.islower()
        nums = _numbers(m.group(2))
        arity = _ARITY[upper]
        if upper == "Z":
            x, y = start_x, start_y
            out.append((x, y))
            continue
        if not nums or len(nums) % arity:
            raise ValueError(f"{cmd} の引数の数が {arity} の倍数になっていない: {nums}")
        for i in range(0, len(nums), arity):
            a = nums[i:i + arity]
            if upper == "H":
                x = x + a[0] if rel else a[0]
            elif upper == "V":
                y = y + a[0] if rel else a[0]
            elif upper == "A":
                nx, ny = (x + a[5], y + a[6]) if rel else (a[5], a[6])
                out += _arc_points(x, y, a[0], a[1], a[2], a[3], a[4], nx, ny)
                x, y = nx, ny
                continue
            else:
                pairs = [(a[j], a[j + 1]) for j in range(0, arity, 2)]
                for px, py in pairs:
                    out.append((x + px, y + py) if rel else (px, py))
                x, y = out[-1]
                if upper == "M":
                    start_x, start_y = x, y
            out.append((x, y))
    if pos != len(d) and d[pos:].strip():
        raise ValueError(f"パスとして読めない断片がある: {d[pos:].strip()!r}")
    return out


def path_bounds(d: str) -> tuple[float, float, float, float]:
    """パスの外接矩形 ── (左, 上, 幅, 高さ)。"""
    pts = path_points(d)
    if not pts:
        return (0.0, 0.0, 0.0, 0.0)
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    return (min(xs), min(ys), max(xs) - min(xs), max(ys) - min(ys))


@component("path")
def path(props: dict, style: Style) -> OwnOrigin:
    """任意のパスデータをそのまま描く。

    props: d（SVGのパスデータ。M/L/H/V/C/S/Q/T/A/Z を、絶対でも相対でも）／
           filled（bool、既定True。Falseなら塗らずに線だけにする）

    **座標を書き換えない。** 節点系の契約(自分の原点(0,0)基準で描く)へは、
    外接矩形ぶんの平行移動で適合させる ── 書き換えると、円弧や省略形の
    意味まで解釈し直すことになり、そこで形が壊れる。
    """
    filled = props.get("filled", True)
    fill = style.text("color.shape-fill", style.text("color.accent")) if filled else "none"
    stroke = style.text("color.shape-stroke", style.text("color.accent"))
    sw = style.num("size.stroke-width")
    d = props["d"]
    x0, y0, w, h = path_bounds(d)
    svg = (f'<g transform="translate({-x0:.2f},{-y0:.2f})">'
           f'<path d="{d}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/></g>')
    return OwnOrigin(svg=svg, width=w, height=h)


@component("boolean")
def boolean(props: dict, style: Style) -> OwnOrigin:
    """多角形どうしの和・積・差(Illustratorの型抜きに相当)。

    props: shapes（多角形(点の並び)を2つ以上。boolean.circle_polygon/
           rect_polygon等で作れる）／op（"union"|"intersect"|"subtract"）

    差(subtract)で、後の形が先の形の内側へ完全に収まる場合は穴になる。
    結果の輪郭は1つの<path>の中の別々の輪郭として置き、偶奇規則で塗る。
    輪郭ごとに<path>を分けると内側が穴にならず塗り重なるので分けない。
    """
    polygons = boolean_op(props["shapes"], props["op"])
    fill = style.text("color.shape-fill", style.text("color.accent"))

    # 節点系の契約(自分の原点(0,0)基準で描く)へ適合させるため、入力の座標系が
    # どこにあっても、結果の外接矩形の左上を(0,0)へ揃え直してから描く。
    # これを怠ると、呼び出し側のviewBoxが(0,0)起点である前提と食い違い、
    # 描画結果が画布の外へ出て何も見えなくなる(実測で踏んだ不具合)。
    all_pts = [p for poly in polygons for p in poly]
    x0 = min((p[0] for p in all_pts), default=0.0)
    y0 = min((p[1] for p in all_pts), default=0.0)

    # 輪郭ごとに別の path にすると、内周が穴にならず塗り重なる。偶奇規則で
    # 穴を作るには、外周と内周を1つの path の中へ入れる必要がある。
    subpaths = []
    for poly in polygons:
        subpaths.append("M" + " L".join(f"{x - x0:.1f},{y - y0:.1f}" for x, y in poly) + " Z")
    parts = [f'<path d="{" ".join(subpaths)}" fill="{fill}" fill-rule="evenodd"/>'] if subpaths else []
    xs = [p[0] - x0 for p in all_pts]
    ys = [p[1] - y0 for p in all_pts]
    w = max(xs) if xs else 0.0
    h = max(ys) if ys else 0.0
    return OwnOrigin(svg=f'<g>{"".join(parts)}</g>', width=w, height=h)
