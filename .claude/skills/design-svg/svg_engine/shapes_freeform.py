"""自由曲線 ── ベジェを含む任意のパスデータをそのまま渡せる部品。

「登録済みの部品の組み合わせでしか表現できない」という制約への回答が、
完成された形の部品を1つ足すことではなく、任意の形そのものを渡せる
この部品を足すこと。ペンツールのGUIは無いが、ペンツールが作るのと
同じ入力(パスデータ)を受け取れる。

契約(asset-authoring-contract-for-component-svg-engines)に従い、
props はパスの構造(d)と閉じるかどうかだけを持ち、色はstyleから引く。
"""
from __future__ import annotations

import re

from .boolean import boolean_op
from .tokens import Style
from .registry import Absolute, OwnOrigin, component

_NUM = re.compile(r"-?\d+(?:\.\d+)?")
_TOKEN = re.compile(r"([MLCQZ])([^MLCQZ]*)")

# 絶対座標の引数の個数(x,yの組の数)。Zは引数を持たない。
_ARITY = {"M": 1, "L": 1, "C": 3, "Q": 2, "Z": 0}


def _parse(d: str) -> list[tuple[str, list[float]]]:
    """M/L/C/Q/Z(絶対座標のみ)の並びとして解析する。

    これ以外のコマンド(相対座標の小文字、円弧のA、水平/垂直のH/V等)は、
    この部品が受け取る入力として想定しておらず、誤って解釈すると座標を
    壊すので、検出したら黙って通さず例外にする。
    """
    bad_letters = set(re.findall(r"[A-Za-z]", d)) - set("MLCQZ")
    if bad_letters:
        raise ValueError(f"pathはM/L/C/Q/Z(絶対座標)のみ受け付ける。未対応のコマンド: {sorted(bad_letters)}")

    out = []
    pos = 0
    for m in _TOKEN.finditer(d):
        if m.start() != pos:
            bad = d[pos:m.start()].strip()
            if bad:
                raise ValueError(f"pathはM/L/C/Q/Z(絶対座標)のみ受け付ける。未対応: {bad!r}")
        cmd = m.group(1)
        nums = [float(n) for n in _NUM.findall(m.group(2))]
        arity = _ARITY[cmd]
        if arity and len(nums) % (arity * 2) != 0:
            raise ValueError(f"{cmd}の引数の数が{arity}組の倍数になっていない: {nums}")
        out.append((cmd, nums))
        pos = m.end()
    if pos != len(d):
        tail = d[pos:].strip()
        if tail:
            raise ValueError(f"pathはM/L/C/Q/Z(絶対座標)のみ受け付ける。未対応: {tail!r}")
    return out


def normalize_path(d: str) -> tuple[str, float, float]:
    """パスの座標を、外接矩形の左上が(0,0)に来るよう平行移動する。

    節点系の契約(自分の原点(0,0)基準で描く)を守るため。座標をそのまま
    渡すとboolean部品で実際に踏んだのと同じ不具合(viewBoxの外へ出て
    見えなくなる)が起きる。

    Returns:
        (平行移動済みのd文字列, 幅, 高さ)。
    """
    tokens = _parse(d)
    all_xy = [nums[i:i + 2] for cmd, nums in tokens for i in range(0, len(nums), 2)]
    if not all_xy:
        return (d, 0.0, 0.0)
    x0 = min(p[0] for p in all_xy)
    y0 = min(p[1] for p in all_xy)
    x1 = max(p[0] for p in all_xy)
    y1 = max(p[1] for p in all_xy)

    parts = []
    for cmd, nums in tokens:
        shifted = []
        for i in range(0, len(nums), 2):
            shifted.append(f"{nums[i] - x0:.2f},{nums[i + 1] - y0:.2f}")
        parts.append(cmd + " ".join(shifted))
    return (" ".join(parts), x1 - x0, y1 - y0)


@component("path")
def path(props: dict, style: Style) -> OwnOrigin:
    """任意のパスデータをそのまま描く。

    props: d（SVGのpath data文字列。M/L/C/Q/Z等、そのまま渡す）／
           filled（bool、既定True。Falseなら塗らずに線だけにする）

    dの座標がどこにあっても、節点系の契約(自分の原点(0,0)基準で描く)を
    守れるよう、外接矩形の左上が(0,0)に来る形へ平行移動してから描く。
    対応するコマンドはM/L/C/Q/Z(絶対座標)のみ。それ以外(相対座標の
    小文字・円弧のA等)を含むdを渡すと、座標を誤って壊さないよう例外にする。
    """
    filled = props.get("filled", True)
    fill = style.text("color.shape-fill", style.text("color.accent")) if filled else "none"
    stroke = style.text("color.shape-stroke", style.text("color.accent"))
    sw = style.num("size.stroke-width")
    d, w, h = normalize_path(props["d"])
    svg = f'<path d="{d}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>'
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

    # 節点系の契約(自分の原点(0,0)基準で描く)を守るため、入力の座標系が
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
