# SPDX-License-Identifier: MIT
# Copyright (c) 2026 daidaiiro
"""coding-essence のブレストボードが持つ図を、構造化データの宣言から組む。

固有の語彙（原典・判定・除外先・規則文書）から、エンジンが知っている
節点・辺・囲みへ直す変換は、呼ぶ側であるこの module に置く
── design-svg のガードレール「利用側の語彙をエンジンへ持ち込まない」。

図の幅は 814 で統一する。幅は中身の大きさから決まるので、合わせる手は
「段内の間隔」というトークン1つを目標の幅から解くことだけにする
── 勘で置いた数ではなく、814 という要求から導かれる量である。

    python3 figures.py          # figures/*.svg を書き出す
"""
from __future__ import annotations

import pathlib
import sys
from functools import partial

_SKILL = pathlib.Path(__file__).resolve().parents[2] / ".claude" / "skills" / "design-svg"
if str(_SKILL) not in sys.path:
    sys.path.insert(0, str(_SKILL))

from svg_engine import DEFAULT_THEME, render_figure  # noqa: E402
from svg_engine.compose import figure_fragment  # noqa: E402
from svg_engine.grid import layout_grid  # noqa: E402

FIG_WIDTH = 814.0
OUT = pathlib.Path(__file__).resolve().parent / "figures"

# 役割を2つ足す。エンジンには触れず、テーマへ行を足すだけで増える。
#   reroute ── 点線だが、除外ではない。破棄せずに別の決めごとへ送る先
#   added   ── 今回足した項目。他と枠線が違うことで、新しいと分かる
THEME = dict(DEFAULT_THEME, **{
    "role.reroute.color.box-fill": "none",
    "role.reroute.color.box-stroke": "color.accent",
    "role.reroute.color.text": "color.accent",
    "role.reroute.font.weight": "font.weight-medium",
    "role.reroute.stroke-dasharray": "4 3",

    "role.added.color.box-fill": "color.accent-bg",
    "role.added.color.box-stroke": "color.accent",
    "role.added.color.text": "color.accent",
    "role.added.size.stroke-width": "size.stroke-width-focus",
    "role.added.font.weight": "font.weight-medium",
})


def fit(draw, target: float = FIG_WIDTH) -> str:
    """段内の間隔を動かして、図の幅を目標へ合わせる。

    幅は間隔について単調に増えるので、二分法で一意に決まる。丸めた幅では
    境目をまたげないので、丸める前の値（figure_fragment が返す幅）で挟む。

    Args:
        draw: テーマと raw を受け取って図を返す関数。
        target: 合わせたい幅。

    Returns:
        `<svg>...</svg>` 文字列。

    Raises:
        AssertionError: 解いた間隔でも目標の幅にならなかったとき。
    """
    lo, hi = 0.0, 400.0
    for _ in range(60):
        mid = (lo + hi) / 2
        if draw(dict(THEME, **{"size.gap-order": mid}), raw=True).width < target:
            lo = mid
        else:
            hi = mid
    svg = draw(dict(THEME, **{"size.gap-order": (lo + hi) / 2}))
    assert f'viewBox="0 0 {target:.0f} ' in svg, svg[:80]
    return svg


# ── 1枚目：2段のふるい ────────────────────────────────────────────
# 本線は左から右へ（判定1 → 判定2 → 載る）、除外の経路は真下へ。
# どこに置くかを決めているのは辺ではなく座標なので、格子に解かせる。
_SIEVE_AT = {
    "origin": ("a", 1),
    "judge1": ("a", 2), "judge2": ("b", 2), "goal": ("c", 2),
    "drop1": ("a", 3), "drop2": ("b", 3),
}


def sieve(theme: dict, raw: bool = False):
    """原典が2段の判定を通って「Go の規則」へ載るまで。

    除外先は2つあり、意味が違う。判定1の除外先は載らない（灰色・点線）。
    判定2の除外先は破棄せずに別の決めごとへ送る（色つき・点線）。
    """
    nodes = [
        {"id": "origin", "label": "原典 ── 公式の情報 ・ 確立された設計論"},
        {"id": "judge1", "label": "何を確認すれば適合と判定できるかを、書けるか", "role": "focus"},
        {"id": "judge2", "label": "検証方法を実行する工程が、決まっているか", "role": "focus"},
        {"id": "goal", "label": "「Go の規則」に載る　＋　検証方法"},
        {"id": "drop1", "label": "載らない ── 原典を読めばよい", "role": "muted"},
        {"id": "drop2", "label": "除外しない ── 「工程を決める」まで保留", "role": "reroute"},
    ]
    edges = [
        {"from": "origin", "to": "judge1"},
        {"from": "judge1", "to": "judge2"},
        {"from": "judge2", "to": "goal"},
        {"from": "judge1", "to": "drop1", "dashed": True},
        {"from": "judge2", "to": "drop2", "dashed": True},
    ]
    lay = partial(layout_grid, at=_SIEVE_AT, cols=["a", "b", "c"], rows=[1, 2, 3])
    draw = figure_fragment if raw else render_figure
    return draw(nodes, edges, theme=theme, layout=lay)


# ── 2枚目：規則文書1本の内訳 ──────────────────────────────────────
# 文書 ＝ メタデータ ＋ 規則の並び。規則1件は3つを内包し、その3つが
# それぞれ何を内包するかまで見せる。内包の関係は入れ子の図でそのまま描く
# （囲みは辺の相手にならないので、中の1つへつなぎたい形には使わない）。
def doc(theme: dict, raw: bool = False):
    """規則文書1本が何を内包するか。検証方法の「最後に確認した日」が今回の追加。"""
    stack = "LR"   # 段が横に進む ＝ 段の中の並びは縦に積まれる
    row = "TB"     # 段が縦に進む ＝ 段の中の並びは横に伸びる

    meta = {"direction": row, "nodes": [
        {"id": "m1", "label": "軸と値"},
        {"id": "m2", "label": "承認"},
    ]}
    spec = {"direction": stack, "nodes": [
        {"id": "s1", "label": "遵守すべき内容"},
        {"id": "s2", "label": "水準"},
    ]}
    verify = {"direction": stack, "nodes": [
        {"id": "v1", "label": "何を確認すれば適合と判定できるか"},
        {"id": "v2", "label": "担当者"},
        {"id": "v3", "label": "最後に確認した日", "role": "added"},
    ]}
    source = {"direction": stack, "nodes": [
        {"id": "o1", "label": "引用した文字列"},
        {"id": "o2", "label": "取得日"},
    ]}
    rule = {"direction": row, "nodes": [
        {"id": "spec", "label": "規定", "figure": spec},
        {"id": "verify", "label": "検証方法", "figure": verify},
        {"id": "source", "label": "出典 ── 1つとは限らない", "figure": source},
    ]}
    nodes = [
        {"id": "meta", "label": "メタデータ", "figure": meta},
        {"id": "rule", "label": "規則 1件", "figure": rule},
    ]
    draw = figure_fragment if raw else render_figure
    return draw(nodes, [], groups=[{"label": "規則文書1本",
                                    "members": ["meta", "rule"]}],
                direction=row, theme=theme)


# figcheck が取り出すのは、大文字の名前が持つ SVG 文字列の組である。
SIEVE = (fit(sieve),)
DOC = (fit(doc),)

FIGURES = {"sieve.svg": SIEVE[0], "doc.svg": DOC[0]}


def main() -> int:
    """図を figures/ へ書き出し、エンジンの幾何検査に掛ける。"""
    from svg_engine import verify

    bad = 0
    for name, svg in FIGURES.items():
        OUT.joinpath(name).write_text(svg, encoding="utf-8")
        for check in (verify.check, verify.check_shapes, verify.check_attachment):
            found = check(svg)
            if found:
                bad += len(found)
                print(f"{name}: {check.__name__} ── {found}")
        print(f"{name}: 書き出した（{len(svg)} 文字）")
    print("幾何の食い違いは無い" if not bad else f"直すところが {bad} 件ある")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
