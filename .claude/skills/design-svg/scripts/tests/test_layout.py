"""層1 ── 配置の計算そのもの。SVGを1文字も作らずに座標と個数を検証する。"""
from __future__ import annotations

import math

import pytest

from svg_engine.layout_contract import UnsupportedByStrategy
from svg_engine.nesting import layout_nested
from svg_engine.radial import layout_radial
from svg_engine.sugiyama import _assign_ranks, layout_graph
from svg_engine.tree import layout_tree

SIZE = (80.0, 40.0)


def _sizes(*ids: str) -> dict[str, tuple[float, float]]:
    return {i: SIZE for i in ids}


def _no_overlap(boxes: dict[str, tuple[float, float]],
                sizes: dict[str, tuple[float, float]]) -> bool:
    ks = list(boxes)
    for i, a in enumerate(ks):
        ax, ay = boxes[a]
        aw, ah = sizes[a]
        for b in ks[i + 1:]:
            bx, by = boxes[b]
            bw, bh = sizes[b]
            if ax < bx + bw and ax + aw > bx and ay < by + bh and ay + ah > by:
                return False
    return True


class TestRank:
    def test_layers_descend_along_edge_direction(self):
        """辺の向きに沿って層が下る。"""
        r = layout_graph(_sizes("a", "b", "c"), [("a", "b"), ("b", "c")], 40, 30)
        ys = [r.positions[k][1] for k in ("a", "b", "c")]
        assert ys[0] < ys[1] < ys[2]

    def test_horizontal_mode_descends_sideways(self):
        """横向きにすると層は横へ下る。"""
        r = layout_graph(_sizes("a", "b"), [("a", "b")], 40, 30, direction="LR")
        assert r.positions["a"][0] < r.positions["b"][0]
        assert r.positions["a"][1] == pytest.approx(r.positions["b"][1])


    def test_layers_minimize_total_edge_length(self):
        """層は辺の長さの総和が最小になるように決まる。"""
        # 長さの違う3本の道が1点へ合流する。層が構造から一意に決まらないので、
        # どこへ置くかに自由がある。短い道を上端へ寄せると辺が伸びる。
        edges = [("長1", "長2"), ("長2", "長3"), ("長3", "長4"), ("長4", "合"),
                 ("短1", "短2"), ("短2", "合"),
                 ("中1", "中2"), ("中2", "中3"), ("中3", "合")]
        rank = _assign_ranks(["長1", "長2", "長3", "長4", "短1", "短2",
                              "中1", "中2", "中3", "合"], edges)
        # どの辺も1層以上またぐので、総和は辺の本数を下回れない。等号＝最適。
        assert sum(rank[b] - rank[a] for a, b in edges) == len(edges)

    def test_disconnected_components_get_their_own_layers(self):
        """繋がっていない塊はそれぞれ独立に層が決まる。"""
        rank = _assign_ranks(["a", "b", "x", "y"], [("a", "b"), ("x", "y")])
        assert rank["a"] == rank["x"] == 0
        assert rank["b"] == rank["y"] == 1


class TestCycle:
    def test_cycles_resolve_without_overlapping_nodes(self):
        """輪になっていても解けて節点が重ならない。"""
        sizes = _sizes("a", "b", "c")
        r = layout_graph(sizes, [("a", "b"), ("b", "c"), ("c", "a")], 40, 30)
        assert len(r.positions) == 3
        assert _no_overlap(r.positions, sizes)

    def test_self_edge_still_resolves(self):
        """自分へ戻る辺があっても解ける。"""
        r = layout_graph(_sizes("a"), [("a", "a")], 40, 30)
        assert "a" in r.positions


class TestMultiRankEdge:
    def test_layer_skipping_edge_gets_a_bend(self):
        """仮節点を経由するから、2点の直線ではなくなる。"""
        r = layout_graph(_sizes("a", "b", "c"),
                         [("a", "b"), ("b", "c"), ("a", "c")], 40, 30)
        skip = r.edge_paths[2]
        assert len(skip) > 2


class TestNested:
    def test_nested_group_fully_contains_the_inner_one(self):
        """入れ子の群は外側が内側を完全に含む。"""
        sizes = _sizes("a", "b", "c", "d")
        _, groups, _, _, _ = layout_nested(
            sizes, [("a", "b"), ("c", "d")],
            [{"label": "外", "members": ["a", "b", "c", "d"]},
             {"label": "内", "members": ["c", "d"]}],
            gap_rank=40, gap_order=30, direction="TB", frame_pad=12, label_h=16)
        boxes = sorted(groups.values(), key=lambda b: b.width * b.height)
        inner, outer = boxes[0], boxes[-1]
        assert outer.x <= inner.x and outer.y <= inner.y
        assert outer.x + outer.width >= inner.x + inner.width
        assert outer.y + outer.height >= inner.y + inner.height

    def test_non_nested_overlap_is_refused(self):
        """1つの節点が2つの群に半端に属す形は、この戦略では解けない。"""
        with pytest.raises(UnsupportedByStrategy):
            layout_nested(_sizes("a", "b", "c"), [],
                          [{"label": "甲", "members": ["a", "b"]},
                           {"label": "乙", "members": ["b", "c"]}],
                          gap_rank=40, gap_order=30, direction="TB",
                          frame_pad=12, label_h=16)


class TestRadial:
    def test_nodes_are_evenly_spaced_on_one_circle(self):
        """節点は同じ輪の上に等間隔で並ぶ。"""
        sizes = _sizes("a", "b", "c", "d")
        r = layout_radial(sizes, [("a", "b"), ("b", "c"), ("c", "d"), ("d", "a")], 40, 30)
        cx = [r.positions[k][0] + SIZE[0] / 2 for k in sizes]
        cy = [r.positions[k][1] + SIZE[1] / 2 for k in sizes]
        ox, oy = sum(cx) / 4, sum(cy) / 4
        radii = [((x - ox) ** 2 + (y - oy) ** 2) ** 0.5 for x, y in zip(cx, cy)]
        assert max(radii) - min(radii) < 1.0

    def test_bigger_nodes_give_a_bigger_circle(self):
        """半径を決め打ちしていないことを、大きさを変えて検証する。"""
        small = layout_radial({k: (40.0, 20.0) for k in "abcd"}, [], 40, 30)
        big = layout_radial({k: (200.0, 100.0) for k in "abcd"}, [], 40, 30)
        assert big.width > small.width

    def test_nodes_never_overlap_at_any_size(self):
        """どの大きさでも節点は重ならない。"""
        for w, h in [(40.0, 20.0), (200.0, 100.0), (300.0, 30.0)]:
            sizes = {k: (w, h) for k in "abcde"}
            r = layout_radial(sizes, [], 40, 30)
            assert _no_overlap(r.positions, sizes)

    def test_same_return_shape_as_layered(self):
        """戦略を差し替えられる、という主張の根拠。"""
        sizes = _sizes("a", "b")
        a = layout_graph(sizes, [("a", "b")], 40, 30)
        b = layout_radial(sizes, [("a", "b")], 40, 30)
        assert type(a) is type(b)
        assert set(a.positions) == set(b.positions)

    def test_circle_order_comes_from_edges_not_declaration(self):
        """宣言順に従うと、隣り合うべき節点が輪の反対側へ行き、
        絵はもつれた星になる。幾何的な破綻は出ないので検査は通ってしまう。"""
        keys = [f"n{i}" for i in range(8)]
        cyc = [(keys[i], keys[(i + 1) % 8]) for i in range(8)]
        shuffled = keys[::2] + keys[1::2]
        r = layout_radial({k: SIZE for k in shuffled}, cyc, 40, 30)
        ox = sum(r.positions[k][0] for k in keys) / 8 + SIZE[0] / 2
        oy = sum(r.positions[k][1] for k in keys) / 8 + SIZE[1] / 2

        def angle(k):
            return math.atan2(r.positions[k][1] + SIZE[1] / 2 - oy,
                              r.positions[k][0] + SIZE[0] / 2 - ox)

        # 輪でつながっている隣どうしは、輪の上でも隣（1周／8 の角度）に居る
        step = 2 * math.pi / 8
        for a, b in cyc:
            d = abs(angle(a) - angle(b)) % (2 * math.pi)
            d = min(d, 2 * math.pi - d)
            assert d == pytest.approx(step, abs=step * 0.1)

    def test_shape_that_cannot_fit_the_circle_is_refused(self):
        """1つの節点から3方向以上へ分かれる木は、輪の上に並べきれない。"""
        sizes = {"root": SIZE}
        edges = []
        for i in range(3):
            sizes[f"a{i}"] = SIZE
            edges.append(("root", f"a{i}"))
            for j in range(3):
                sizes[f"b{i}{j}"] = SIZE
                edges.append((f"a{i}", f"b{i}{j}"))
        with pytest.raises(UnsupportedByStrategy):
            layout_radial(sizes, edges, 40, 30)


def _mindmap(branches: int, leaves: int = 3):
    ids = ["root"]
    edges = []
    for i in range(branches):
        a = f"a{i}"
        ids.append(a)
        edges.append(("root", a))
        for j in range(leaves):
            b = f"b{i}{j}"
            ids.append(b)
            edges.append((a, b))
    return {i: SIZE for i in ids}, edges


class TestTree:
    def test_root_is_at_the_center(self):
        """根が中心に来る。"""
        sizes, edges = _mindmap(4)
        r = layout_tree(sizes, edges, 40, 30)
        cx = [r.positions[k][0] + SIZE[0] / 2 for k in sizes]
        cy = [r.positions[k][1] + SIZE[1] / 2 for k in sizes]
        rx = r.positions["root"][0] + SIZE[0] / 2
        ry = r.positions["root"][1] + SIZE[1] / 2
        assert abs(rx - sum(cx) / len(cx)) < SIZE[0]
        assert abs(ry - sum(cy) / len(cy)) < SIZE[1]

    def test_deeper_nodes_are_farther_from_center(self):
        """深さを輪で表す、ということの座標での言い方。"""
        sizes, edges = _mindmap(4)
        r = layout_tree(sizes, edges, 40, 30)
        ox = r.positions["root"][0] + SIZE[0] / 2
        oy = r.positions["root"][1] + SIZE[1] / 2

        def dist(k):
            return ((r.positions[k][0] + SIZE[0] / 2 - ox) ** 2
                    + (r.positions[k][1] + SIZE[1] / 2 - oy) ** 2) ** 0.5

        assert max(dist(f"a{i}") for i in range(4)) < min(
            dist(f"b{i}{j}") for i in range(4) for j in range(3))

    def test_nodes_do_not_overlap(self):
        """節点は重ならない。"""
        for br in (2, 4, 10):
            sizes, edges = _mindmap(br)
            r = layout_tree(sizes, edges, 40, 30)
            assert _no_overlap(r.positions, sizes)

    def test_more_branches_do_not_form_a_band(self):
        """層状配置はここで細長い帯になる。放射木は正方形に近いまま。

        比べるのは形（縦横比）と大きさの向きだけで、何倍という数は置かない
        ── 配置が良くなると倍率は変わるが、主張は変わらないため。
        """
        sizes, edges = _mindmap(10)
        t = layout_tree(sizes, edges, 40, 30)
        g = layout_graph(sizes, edges, 40, 30)
        tree_ratio = max(t.width, t.height) / min(t.width, t.height)
        graph_ratio = max(g.width, g.height) / min(g.width, g.height)
        assert tree_ratio < 2                    # 正方形に近い
        assert graph_ratio > tree_ratio * 2      # 層状は明らかに細長い
        assert max(t.width, t.height) < max(g.width, g.height)

    def test_cycles_still_resolve(self):
        """木に入らない辺があっても失敗しない。"""
        sizes = _sizes("a", "b", "c")
        r = layout_tree(sizes, [("a", "b"), ("b", "c"), ("c", "a")], 40, 30)
        assert len(r.positions) == 3

    def test_unreachable_nodes_are_placed_too(self):
        """根から届かない節点も置かれる。"""
        sizes = _sizes("a", "b", "x")
        r = layout_tree(sizes, [("a", "b")], 40, 30)
        assert set(r.positions) == {"a", "b", "x"}

    def test_same_return_shape_as_layered(self):
        """層状配置と同じ戻り値の形をしている。"""
        sizes = _sizes("a", "b")
        assert type(layout_tree(sizes, [("a", "b")], 40, 30)) is type(
            layout_graph(sizes, [("a", "b")], 40, 30))


class TestLabelClearance:
    """層の間隔は、その間を通る辺のラベルが収まるだけ空ける。

    「ラベルどうしが重ならない」だけを性質にしていたため、逃げ場が足りるかを
    誰も見ていなかった。実測：3節点の鎖に長いラベルを付けると、ラベルが節点の名前へ
    重なった（4件）。ラベルを短くすると0件になるので、足りないのは逃げ場の側。
    """

    def _faults(self, label, direction):
        from svg_engine.compose import render_figure
        from svg_engine.verify import check, check_attachment, check_shapes
        svg = render_figure(
            nodes=[{"id": "a", "label": "概念"}, {"id": "b", "label": "満たすべき性質"},
                   {"id": "c", "label": "アルゴリズム"}],
            edges=[{"from": "a", "to": "b", "label": label},
                   {"from": "b", "to": "c", "label": label}],
            direction=direction)
        return check(svg) + check_shapes(svg) + check_attachment(svg)

    def test_short_labels_do_not_overlap(self):
        """短いラベルは重ならない。"""
        assert self._faults("保証", "LR") == []

    def test_labels_longer_than_layer_gap_do_not_overlap(self):
        """既定の層の間隔（48）より明らかに長いラベル。"""
        assert self._faults("何が成り立てばその概念かを言う", "LR") == []

    def test_vertical_mode_does_not_overlap(self):
        """縦向きでも重ならない。"""
        assert self._faults("何が成り立てばその概念かを言う", "TB") == []


class TestNestedFigures:
    """図を部品として置く ── 決定「図を、部品として置けるようにする」の裏づけ。

    子図の中で守られている性質が入れ子にしても壊れないこと、親が子図の実際の
    大きさを知って場所を取ることを検証する。
    """

    CHILD = {"nodes": [{"id": "x", "label": "子1"}, {"id": "y", "label": "子2"}],
             "edges": [{"from": "x", "to": "y"}]}

    def _faults(self, svg):
        from svg_engine.verify import check, check_attachment, check_shapes
        return check(svg) + check_shapes(svg) + check_attachment(svg)

    def test_assembly_adds_no_wrapper(self):
        """図が部品として使えるのは、器づけと分かれているから。"""
        from svg_engine.compose import figure_fragment
        r = figure_fragment([{"id": "a", "label": "A"}])
        assert "<svg" not in r.svg
        assert r.width > 0 and r.height > 0

    def test_declared_size_contains_the_ink(self):
        """部品に課している契約を、図も満たす。"""
        from svg_engine.compose import figure_fragment
        from svg_engine.geometry import sample_ink
        r = figure_fragment([{"id": "a", "label": "A"}, {"id": "b", "label": "B"}],
                            [{"from": "a", "to": "b", "label": "渡す"}])
        pts = [p for p, _ in sample_ink(r.svg, max(min(r.width, r.height), 1) / 32)]
        assert pts
        assert min(x for x, _ in pts) >= -1 and min(y for _, y in pts) >= -1
        assert max(x for x, _ in pts) <= r.width + 1
        assert max(y for _, y in pts) <= r.height + 1

    def test_child_figure_can_be_placed_as_a_node(self):
        """子図を節点として置ける。"""
        from svg_engine.compose import render_figure
        svg = render_figure([{"id": "a", "label": "親"}, {"id": "b", "figure": self.CHILD}],
                            [{"from": "a", "to": "b"}])
        assert self._faults(svg) == []
        assert "子1" in svg and "子2" in svg

    def test_nesting_depth_is_bounded(self):
        """層1 が「入れ子は深さに上限を置く」と定めている。"""
        import pytest
        from svg_engine.compose import render_figure
        from svg_engine.tokens import DEFAULT_THEME, num
        deep: dict = {"nodes": [{"id": "leaf", "label": "葉"}]}
        for _ in range(int(num(DEFAULT_THEME, "size.figure-depth-limit")) + 1):
            deep = {"nodes": [{"id": "n", "figure": deep}]}
        with pytest.raises(ValueError, match="深すぎる"):
            render_figure([{"id": "a", "figure": deep}])


class TestGridLayout:
    """座標のとおりに置く4つ目の戦略。"""

    def test_nodes_follow_the_given_coordinates(self):
        """座標のとおりに並ぶ。"""
        from functools import partial
        from svg_engine.compose import render_figure
        from svg_engine.grid import layout_grid
        from svg_engine.verify import check, check_attachment, check_shapes
        at = {"a": ("左", "上"), "b": ("右", "上"), "c": ("右", "下")}
        nodes = [{"id": k, "label": k} for k in at]
        svg = render_figure(nodes, [], layout=partial(layout_grid, at=at))
        assert check(svg) + check_shapes(svg) + check_attachment(svg) == []

    def test_order_can_be_supplied(self):
        """見出しを端へ置くために、列と行の並びを外から決められる。"""
        from functools import partial
        from svg_engine.grid import layout_grid
        at = {"h": ("見出し", "上"), "a": ("左", "上")}
        sizes = {"h": (40.0, 20.0), "a": (40.0, 20.0)}
        left = layout_grid(sizes, [], 10, 10, at=at, cols=["見出し", "左"], rows=["上"])
        right = layout_grid(sizes, [], 10, 10, at=at, cols=["左", "見出し"], rows=["上"])
        assert left.positions["h"][0] < left.positions["a"][0]
        assert right.positions["h"][0] > right.positions["a"][0]

    def test_node_without_coordinates_is_refused(self):
        """座標の無い節点は拒む。"""
        import pytest
        from svg_engine.grid import layout_grid
        with pytest.raises(KeyError):
            layout_grid({"a": (10.0, 10.0)}, [], 10, 10, at={})

    def test_elbow_edge_has_one_corner(self):
        """角の置き方で、辺がどの辺から出入りするかが変わる。"""
        from svg_engine.grid import layout_grid
        at = {"a": (0, 0), "b": (1, 1)}
        sizes = {"a": (100.0, 32.0), "b": (80.0, 32.0)}
        args = (sizes, [("a", "b")], 20, 20, "LR")
        straight = layout_grid(*args, at=at)
        down = layout_grid(*args, at=at, elbow={("a", "b"): "vertical"})
        across = layout_grid(*args, at=at, elbow={("a", "b"): "horizontal"})
        assert len(straight.edge_paths[0]) == 2
        # 真下へ降りてから横へ ── 角は出る側の縦の位置を保つ
        assert down.edge_paths[0][1][0] == down.edge_paths[0][0][0]
        # 横へ出てから降りる ── 角は出る側の横の位置を保つ
        assert across.edge_paths[0][1][1] == across.edge_paths[0][0][1]

    def test_no_corner_when_it_coincides_with_an_end(self):
        """同じ点が2つ並ぶと向きが決まらず、辺の着き先が定まらない。"""
        from svg_engine.grid import layout_grid
        at = {"a": (0, 0), "b": (0, 1)}
        sizes = {"a": (80.0, 32.0), "b": (80.0, 32.0)}
        r = layout_grid(sizes, [("a", "b")], 20, 20, "LR", at=at,
                        elbow={("a", "b"): "vertical"})
        assert len(r.edge_paths[0]) == 2

    def test_elbow_requires_an_existing_edge(self):
        """辺に無いものを鍵線にできない。"""
        import pytest
        from svg_engine.grid import layout_grid
        at = {"a": (0, 0), "b": (1, 1)}
        sizes = {"a": (10.0, 10.0), "b": (10.0, 10.0)}
        with pytest.raises(KeyError):
            layout_grid(sizes, [("a", "b")], 10, 10, at=at,
                        elbow={("a", "c"): "vertical"})

    def test_only_two_corner_orientations(self):
        """角の置き方は2つだけ。"""
        import pytest
        from svg_engine.grid import layout_grid
        at = {"a": (0, 0), "b": (1, 1)}
        sizes = {"a": (10.0, 10.0), "b": (10.0, 10.0)}
        with pytest.raises(ValueError):
            layout_grid(sizes, [("a", "b")], 10, 10, at=at,
                        elbow={("a", "b"): "ななめ"})
