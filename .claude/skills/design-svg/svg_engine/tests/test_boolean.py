"""層1 ── ブーリアン演算。輪郭の個数と、点がどちら側に落ちるかで確かめる。"""
from __future__ import annotations

import pytest

from svg_engine.boolean import (_point_in_polygon, boolean_op, circle_polygon,
                                rect_polygon)


def _area(poly) -> float:
    s = 0.0
    for i in range(len(poly)):
        x1, y1 = poly[i]
        x2, y2 = poly[(i + 1) % len(poly)]
        s += x1 * y2 - x2 * y1
    return abs(s) / 2


class TestSubtract:
    def test_内側に収まる形を抜くと穴として2つの輪郭になる(self):
        outer = circle_polygon(70, 70, 55)
        inner = circle_polygon(70, 70, 25)
        res = boolean_op([outer, inner], "subtract")
        assert len(res) == 2

    def test_穴の中心は塗りの外側と判定される(self):
        """偶奇規則で塗ったとき実際に穴になる、ということの座標での言い方。"""
        res = boolean_op([circle_polygon(70, 70, 55), circle_polygon(70, 70, 25)],
                         "subtract")
        crossings = sum(1 for poly in res if _point_in_polygon((70, 70), poly))
        assert crossings % 2 == 0

    def test_交わる形を抜くと面積が減る(self):
        a = circle_polygon(60, 60, 45)
        b = circle_polygon(100, 60, 45)
        res = boolean_op([a, b], "subtract")
        assert len(res) == 1
        assert _area(res[0]) < _area(a)

    def test_丸ごと包まれる側を抜くと何も残らない(self):
        res = boolean_op([circle_polygon(70, 70, 25), circle_polygon(70, 70, 55)],
                         "subtract")
        assert res == []

    def test_離れている形を抜いても元のまま(self):
        a = rect_polygon(0, 0, 20, 20)
        res = boolean_op([a, rect_polygon(100, 100, 20, 20)], "subtract")
        assert len(res) == 1
        assert _area(res[0]) == pytest.approx(_area(a))


class TestUnionIntersect:
    def test_交わる2つの和は両方より大きい(self):
        a = circle_polygon(60, 60, 45)
        b = circle_polygon(100, 60, 45)
        res = boolean_op([a, b], "union")
        assert sum(_area(p) for p in res) > _area(a)

    def test_交わる2つの積は両方より小さい(self):
        a = circle_polygon(60, 60, 45)
        b = circle_polygon(100, 60, 45)
        res = boolean_op([a, b], "intersect")
        assert sum(_area(p) for p in res) < _area(a)

    def test_離れた2つの積は空(self):
        res = boolean_op([rect_polygon(0, 0, 20, 20), rect_polygon(100, 0, 20, 20)],
                         "intersect")
        assert res == []

    def test_離れた2つの和は2つの輪郭のまま(self):
        res = boolean_op([rect_polygon(0, 0, 20, 20), rect_polygon(100, 0, 20, 20)],
                         "union")
        assert len(res) == 2


class TestGuards:
    def test_形が1つだと断る(self):
        with pytest.raises(ValueError):
            boolean_op([rect_polygon(0, 0, 10, 10)], "union")

    def test_知らない演算名は断る(self):
        with pytest.raises(ValueError):
            boolean_op([rect_polygon(0, 0, 10, 10), rect_polygon(5, 5, 10, 10)], "xor")
