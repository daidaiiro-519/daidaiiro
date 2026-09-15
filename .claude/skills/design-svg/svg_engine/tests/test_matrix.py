"""層2 ── 組み立てたSVGを検査に通す。16主張 × 倍率 と組み合わせの全通り。

1件ずつ独立した試験にしてあるので、崩れたときにどの図のどの倍率かが
そのまま試験名に出る。組み立ては各試験の中で行う ── 表を作る時点で全部
組んでしまうと、1件の例外が収集ごと巻き込んで、何件通ったのかが分からない。
"""
from __future__ import annotations

import pytest

from svg_engine.examples.matrix_cases import cases
from svg_engine.verify import check, check_attachment, check_shapes

CASES = cases()


@pytest.mark.parametrize("scale,name,build",
                         CASES, ids=[f"{s}-{n}" for s, n, _ in CASES])
def test_崩れていない(scale, name, build):
    svg = build()
    faults = check(svg) + check_shapes(svg) + check_attachment(svg)
    assert faults == [], f"[{scale}] {name}: " + " / ".join(dict.fromkeys(faults))


def test_宣言した語が絵に出る():
    """仕様が必須と定めた欄を、変換が落としていないこと。

    幾何の検査では見つからない ── 欄が落ちても、重なりもはみ出しも起きない。
    実際に量の大小の2本目の軸と、分布の bins が落ちていた。
    """
    from svg_engine.examples._check_words import missing_words
    miss = missing_words()
    assert not miss, "宣言したのに絵へ出ていない語: " + "; ".join(
        f"{a}: {w}" for a, w in miss)
