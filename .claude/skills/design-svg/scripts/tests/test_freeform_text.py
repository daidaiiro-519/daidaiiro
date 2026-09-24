"""自由なパスと、文字・人の部品が満たすこと。

**受けると宣言した文法は、実際に受けられるかを機械で照合する** ──
宣言しただけの契約は守られない（この engine が既に踏んだ）。
"""
from __future__ import annotations

import pytest

from svg_engine import render_canvas, verify
from svg_engine.registry import render_component
from svg_engine.shapes_decor import icon_names as _names
from svg_engine.shapes_freeform import path_bounds, path_points
from svg_engine.style import resolve_style
from svg_engine.tokens import DEFAULT_THEME


def _style():
    return resolve_style("plain", None, DEFAULT_THEME)


class TestPathSyntax:
    """SVG のパスの文法をそのまま受ける。"""

    @pytest.mark.parametrize("d", [
        "M0,40 Q30,0 60,40 Q90,80 120,40",      # 絶対の2次曲線
        "M-24 28 v-6 c0 -27 48 -27 48 0 v6",    # 相対の3次曲線
        "M35 17 a17 17 0 1 1 -34 0 a17 17 0 1 1 34 0",   # 円弧
        "M0 0 H56 V36 H0 Z",                    # 水平垂直と閉じ
        "M0,0 L10,0 S20,0 20,10 T30,20",        # 省略形
    ])
    def test_parses(self, d):
        """読める。"""
        x0, y0, w, h = path_bounds(d)
        assert w > 0 and h > 0

    def test_arc_includes_bulge_beyond_endpoints(self):
        """円弧は端点の外側の膨らみを含む。"""
        # 端点だけを見ると幅0になる半円。弧の頂点まで含めて初めて大きさが出る
        _, _, w, h = path_bounds("M0,0 a10 10 0 0 1 0,20")
        assert w == pytest.approx(10.0, abs=0.5)
        assert h == pytest.approx(20.0, abs=0.5)

    def test_close_returns_to_start(self):
        """閉じは開始点へ戻る。"""
        pts = path_points("M10,10 L20,10 Z")
        assert pts[-1] == (10.0, 10.0)

    @pytest.mark.parametrize("d", ["M0,0 L", "M0,0 X10,10"])
    def test_unparsable_raises(self, d):
        """読めないものは例外にする。"""
        with pytest.raises(ValueError):
            path_bounds(d)

    def test_coordinates_are_not_rewritten(self):
        """**d は原文のまま置く** ── 書き換えると円弧や省略形の意味が変わる。"""
        d = "M-24 28 v-6 c0 -27 48 -27 48 0 v6"
        svg = render_canvas(200, 120, [{"kind": "path", "x": 10, "y": 10,
                                        "props": {"d": d, "filled": False}}])
        assert d in svg


class TestTextAndGlyphs:
    """箱にもラベルにも属さない文字と、絵記号。"""

    def test_accepts_a_sequence_of_lines(self):
        """行の並びを受ける。"""
        svg = render_canvas(300, 120, [{"kind": "text", "x": 10, "y": 10,
                                        "props": {"text": ["1行目", "2行目"]}}])
        assert svg.count("<text") == 2

    @pytest.mark.parametrize("align", ["start", "middle", "end"])
    def test_alignment_does_not_move_the_bounding_box(self, align):
        """揃え方を変えても外接矩形は動かない。"""
        style = _style()
        base = render_component("text", {"text": "あいうえお"}, style)
        other = render_component("text", {"text": "あいうえお", "align": align}, style)
        assert (base.width, base.height) == (other.width, other.height)

    def test_person_is_one_of_the_glyphs(self):
        """**同じ枠・同じ線の決まりで描くものを、2つの仕組みに分けない。**"""
        style = _style()
        person = render_component("icon", {"name": "person", "size": 40}, style)
        doc = render_component("icon", {"name": "doc", "size": 40}, style)
        assert (person.width, person.height) == (doc.width, doc.height) == (40, 40)
        assert "person" in _names()

    def test_scaling_does_not_thicken_strokes(self):
        """拡大しても線は太らない。"""
        style = _style()
        small = render_component("icon", {"name": "doc", "size": 24}, style)
        big = render_component("icon", {"name": "doc", "size": 96}, style)
        import re
        def effective(frag):
            scale = float(re.search(r"scale\(([\d.]+)\)", frag.svg).group(1))
            width = float(re.search(r'stroke-width="([\d.]+)"', frag.svg).group(1))
            return round(scale * width, 3)
        assert effective(small) == effective(big)

    def test_unknown_glyph_name_raises(self):
        """知らない絵の名前は例外にする。"""
        style = _style()
        with pytest.raises(ValueError):
            render_component("icon", {"name": "無い絵"}, style)

    def test_glyphs_come_from_the_registry(self):
        """**一覧を2か所に持たない。**"""
        assert "doc" in _names() and "spark" in _names()

    def test_geometry_stays_sound_when_tiled(self):
        """並べても幾何は破綻しない。"""
        layers = [{"kind": "icon", "x": 10 + i * 40, "y": 10,
                   "props": {"name": n, "size": 32}} for i, n in enumerate(_names())]
        layers.append({"kind": "icon", "x": 10, "y": 60,
                       "props": {"name": "person", "size": 48}})
        layers.append({"kind": "text", "x": 70, "y": 60,
                       "props": {"text": "人と絵", "size": 16, "weight": "bold"}})
        svg = render_canvas(560, 130, layers)
        assert verify.check(svg) == [] and verify.check_shapes(svg) == []
