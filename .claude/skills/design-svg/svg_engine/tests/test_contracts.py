"""規約 ── 宣言した規律を、実物が守っているか。

規律を文章にだけ書くと、破れても何も起きない。実際このエンジンでは、
docstring で3箇所に宣言した契約が11ファイルで破れていた。だから宣言した規約は
すべてここで照合する ── 破った瞬間に落ちる。

「何を禁じるか」を書けるものだけを置く。書けないものは規約ではなく好みなので
置かない。
"""
from __future__ import annotations

import ast
import pathlib

import pytest

from svg_engine.registry import known_kinds, render_component
from svg_engine.style import resolve_style
from svg_engine.catalog import EXAMPLES

_ENGINE = pathlib.Path(__file__).resolve().parents[1]

# 責務の層。数字が小さいほど土台に近い。
# 依存の深さ（import の連鎖）とは別物である ── 深さは「呼ぶ順序」と「型をどこから
# 借りたか」を映すだけで、責務の層とは偶然おおむね揃っているにすぎない。だから
# 責務の側をここに明示し、揃い続けることを機械で確かめる。
LAYER: dict[str, int] = {
    # 0 語彙・台帳 ── 誰の都合も知らない。名前・数・形・登録簿
    "tokens": 0, "text": 0, "geometry": 0, "boolean": 0,
    "registry": 0, "lint_values": 0, "layout_contract": 0, "ids": 0,
    # 1 方針・配置 ── 値の解決と、座標の解き方
    "style": 1, "sugiyama": 1, "radial": 1, "tree": 1, "grid": 1,
    "nesting": 1, "labels": 1,
    # 2 部品 ── 1つの形を描く
    "shapes": 2, "shapes_decor": 2, "shapes_freeform": 2, "shapes_hex": 2,
    "shapes_interaction": 2, "shapes_quantity": 2, "shapes_table": 2, "shapes_titled": 2,
    # 3 合成 ── 全部を知ってよい唯一の場所
    "compose": 3, "canvas": 3, "catalog": 3,
}

# 層の数直線に載らないもの。生成物を外から検査する直交した軸で、
# 深さの順に並べた時点で位置づけを誤る。
ORTHOGONAL = {"verify"}


def _modules() -> dict[str, ast.Module]:
    out = {}
    for p in sorted(_ENGINE.glob("*.py")):
        if p.stem != "__init__":
            out[p.stem] = ast.parse(p.read_text(encoding="utf-8"))
    return out


def _imports(tree: ast.Module) -> set[str]:
    out = set()
    for n in ast.walk(tree):
        if isinstance(n, ast.ImportFrom) and n.level == 1 and n.module:
            out.add(n.module)
    return out


class Test規約1_層:
    """下から上を呼ばない。層を飛ばさない。"""

    def test_全てのモジュールが層に割り当てられている(self):
        # 割り当て漏れがあると、そのモジュールだけ検査の外に出る
        known = set(LAYER) | ORTHOGONAL | {"__main__"}
        assert set(_modules()) <= known, f"層の割り当てが無い: {sorted(set(_modules()) - known)}"

    def test_下から上を呼ばない(self):
        bad = []
        for name, tree in _modules().items():
            if name not in LAYER:
                continue
            for dep in _imports(tree):
                if dep in LAYER and LAYER[dep] > LAYER[name]:
                    bad.append(f"{name}(層{LAYER[name]}) → {dep}(層{LAYER[dep]})")
        assert not bad, "下から上への呼び出し: " + " / ".join(bad)

    def test_土台は誰も呼ばない(self):
        # 層0が層1以上を呼ぶと、土台が方針を知ることになる
        for name, tree in _modules().items():
            if LAYER.get(name) != 0:
                continue
            up = {d for d in _imports(tree) if LAYER.get(d, 0) > 0}
            assert not up, f"{name}(層0) が上を呼んでいる: {sorted(up)}"

    def test_依存に循環が無い(self):
        trees = _modules()
        dep = {n: {d for d in _imports(t) if d in trees} for n, t in trees.items()}
        settled: set[str] = set()
        for _ in range(len(dep) + 1):
            settled |= {n for n, d in dep.items() if d <= settled}
        assert settled == set(dep), f"循環に含まれる: {sorted(set(dep) - settled)}"


class Test規約3_契約の所有者:
    """契約は中立が所有し、実装が所有しない。"""

    def test_配置戦略が共有する契約を実装が持たない(self):
        # 以前は層状配置が戻り値の型を持ち、他の3戦略と合成が借りていた。
        # 実装の1つが所有すると、その実装を差し替える判断が借り手を巻き込む。
        owners = {n for n, t in _modules().items()
                  for node in ast.walk(t)
                  if isinstance(node, ast.ClassDef)
                  and node.name in ("LayoutResult", "UnsupportedByStrategy")}
        assert owners == {"layout_contract"}, f"契約を持っているのは {sorted(owners)}"

    def test_戦略は互いを知らない(self):
        strategies = {"sugiyama", "radial", "tree", "grid"}
        for name in strategies:
            others = _imports(_modules()[name]) & (strategies - {name})
            assert not others, f"{name} が他の戦略を呼んでいる: {sorted(others)}"


class Test規約_語彙:
    """このエンジンの契約は「構造化データを受け取る」ことだけ。

    受け取ったデータが何を意味するか・何を言いたいかは呼ぶ側が決める。だから
    配る側のモジュールは、呼ぶ側の名前も、呼ぶ側が使う言い分の語彙も持たない。

    かつては持っていた ── モジュールの分割線（量の部品が9つあること）も、
    配置戦略の存在理由も、見た目のトークンの注記も、呼ぶ側の数え方で書かれていた。
    実害は「呼ぶ側が言い分を1つ増やすと、このエンジンのファイルを触ることになる」
    こと。動かすのに要らないのに、変更の速度だけが同期させられる。

    例と試験は対象外。呼ぶ側の宣言から作った回帰の表を今も借りていて、その
    引っ越しは変換を書くときの作業だからである（配布物には入らない）。
    """

    # 呼ぶ側を指す語。一般の日本語として使う語（「読み方」など）は入れない
    # ── 誤検出する検査は無いより悪い。
    FORBIDDEN = ("Waffle", "waffle", "主張", "asserts", "ユビキタス")

    def test_配る側は呼ぶ側の語彙を持たない(self):
        bad = []
        for p in sorted(_ENGINE.glob("*.py")):
            body = p.read_text(encoding="utf-8")
            hit = [w for w in self.FORBIDDEN if w in body]
            if hit:
                bad.append(f"{p.name}: {hit}")
        assert not bad, "呼ぶ側の語彙が残っている: " + " / ".join(bad)


class Test規約2_段は飛ばせない:
    """選んだ戦略が、途中の経路で黙って捨てられないこと。

    かつては捨てられていた ── 群を渡すと、呼び出し側が選んだ配置戦略が例外も
    警告も無しに無視され、常に層状配置で描かれていた。「4つの戦略が同じ契約で
    差し替えられる」という主張の反例が、その1経路にあった。

    型で「ありえない組み合わせを作れなくする」のが規約2だが、この件は組み合わせ
    自体は正しく、届いていないことが問題だった。だから届くことを縛る。
    """

    def test_群があっても選んだ戦略が使われる(self):
        from svg_engine.compose import figure_fragment
        from svg_engine.sugiyama import layout_graph

        called = []

        def spy(*args, **kwargs):
            called.append(True)
            return layout_graph(*args, **kwargs)

        figure_fragment(
            [{"id": "a", "label": "甲"}, {"id": "b", "label": "乙"}],
            [{"from": "a", "to": "b"}],
            groups=[{"label": "束", "members": ["a", "b"]}],
            layout=spy)
        assert called, "群を渡すと、選んだ戦略が使われずに捨てられている"

    def test_群が無いときも同じ戦略が使われる(self):
        from svg_engine.compose import figure_fragment
        from svg_engine.sugiyama import layout_graph

        called = []

        def spy(*args, **kwargs):
            called.append(True)
            return layout_graph(*args, **kwargs)

        figure_fragment([{"id": "a", "label": "甲"}], [], layout=spy)
        assert called


class Test規約_値の出どころ:
    """コードに現れる数値は、設計上の選択かデータから決まる量のどちらかである。

    3種目 ── 勘で置いた閾値 ── は存在してはいけない。残すと、図ごとにその数字を
    調整することになり、汎用性が失われる。

    この検査は実装されていたのに、どこからも呼ばれず45件が放置されていた。
    書いてあるだけの規約は守られない、という今日いちばん高くついた教訓の実物である。
    """

    def test_勘で置いた数値が残っていない(self):
        from svg_engine.lint_values import findings
        found = findings()
        assert not found, "値の出どころが不明な数値: " + " / ".join(
            f"{f[0]}:{f[1]} {f[3][:40]}" for f in found[:8])


class Test規約2_置き方は型:
    """描く時点が「置く前」か「置いた後」かを、フラグではなく型で持つ。

    かつては placement という文字列の申告だった。文字列だと3つ目の値を書けて
    しまい、型の上では「どちらか分からないもの」を節点として扱うことになる。
    2つの型に分ければ、取り違えが構造として起きない。
    """

    def test_置き方は2つの型しかない(self):
        from svg_engine.registry import Absolute, Fragment, OwnOrigin
        assert {c.__name__ for c in Fragment.__subclasses__()} == {"OwnOrigin", "Absolute"}

    def test_絶対座標の部品は節点として置けない(self):
        from svg_engine.registry import render_node
        from svg_engine.style import resolve_style
        from svg_engine.catalog import EXAMPLES
        with pytest.raises(TypeError) as e:
            render_node("edge", EXAMPLES["edge"], resolve_style())
        assert "edge" in str(e.value)

    def test_自分の原点で描く部品は節点として置ける(self):
        from svg_engine.registry import OwnOrigin, render_node
        from svg_engine.style import resolve_style
        from svg_engine.catalog import EXAMPLES
        assert isinstance(render_node("box", EXAMPLES["box"], resolve_style()), OwnOrigin)

    def test_申告した大きさにインクが収まるのは自分の原点の側だけ(self):
        # 絶対座標の断片は、大きさが器ではなく広がりの記録でしかないので、
        # この性質を課しても意味が無い。型が「どちらに課すか」を持つ。
        from svg_engine.registry import Absolute, OwnOrigin
        assert "インクが収まって" in (OwnOrigin.__doc__ or "")
        assert "課さない" in (Absolute.__doc__ or "")


class Test部品の契約:
    def test_部品はSVGのルートを返さない(self):
        # ルートを持つと、他の部品と合成したとき二重の svg / viewBox が生まれる
        style = resolve_style()
        for kind in known_kinds():
            r = render_component(kind, EXAMPLES[kind], style)
            assert "<svg" not in r.svg, f"{kind} がルートタグを返している"

    @pytest.mark.parametrize("kind", known_kinds())
    def test_部品は決定的(self, kind):
        # 同じ入力から常に同じ出力。乱数も時刻も使わない
        style = resolve_style()
        first = render_component(kind, EXAMPLES[kind], style)
        second = render_component(kind, EXAMPLES[kind], style)
        assert first.svg == second.svg
        assert (first.width, first.height) == (second.width, second.height)
