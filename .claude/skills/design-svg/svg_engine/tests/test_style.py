"""層1 ── 見た目の解決。カスケードの優先順と、範囲外の値を弾くこと。"""
from __future__ import annotations

import pytest

from svg_engine.registry import known_kinds, render_component
from svg_engine.style import (IncompleteThemeError, TokenRangeError,
                              UnknownRoleError, UnknownTokenError, resolve_style)
from svg_engine.tokens import DEFAULT_THEME, TOKEN_RANGES


class TestCascade:
    def test_何も指定しなければテーマの値が出る(self):
        s = resolve_style()
        assert s.num("font.size") == DEFAULT_THEME["font.size"]

    def test_役割はテーマより強い(self):
        assert resolve_style("focus") != resolve_style("plain")

    def test_その場の上書きは役割より強い(self):
        s = resolve_style("focus", {"size.stroke-width": 3.0})
        assert s.num("size.stroke-width") == 3.0

    def test_差し替えたテーマが土台になる(self):
        theme = dict(DEFAULT_THEME, **{"font.size": 20.0})
        assert resolve_style(theme=theme).num("font.size") == 20.0

    def test_トークン名を指す値は指し先までたどる(self):
        s = resolve_style(overrides={"color.ink": "color.accent"})
        assert s.text("color.ink") == DEFAULT_THEME["color.accent"]


class Test役割はテーマが持つ:
    """役割はCSSのクラスに相当する。数を増やすのにエンジンを触らせない。"""

    def test_テーマへ足すだけで新しい役割が増える(self):
        theme = dict(DEFAULT_THEME, **{"role.危険.color.box-fill": "#FBE9E7",
                                       "role.危険.color.box-stroke": "color.warn"})
        s = resolve_style("危険", None, theme)
        assert s.text("color.box-fill") == "#FBE9E7"
        assert s.text("color.box-stroke") == DEFAULT_THEME["color.warn"]

    def test_テーマが知らない役割は描く前に例外になる(self):
        # 黙って既定で描くと綴り違いに気づけない。範囲外のトークン値を
        # その場で弾いているのと同じ扱いにする。
        with pytest.raises(UnknownRoleError) as e:
            resolve_style("知らない役割")
        assert "focus" in str(e.value)  # 使える役割を挙げて返す

    def test_何も上書きしない役割は常に通る(self):
        resolve_style("plain", None, dict(DEFAULT_THEME))

    def test_役割の定義そのものは解決結果へ漏れない(self):
        assert not [k for k in resolve_style("focus").values if k.startswith("role.")]


class Test綴り違いは描く前に落ちる:
    """未知の役割は例外にするのに、未知のトークン名は素通りしていた。

    範囲外の値をその場で弾いているのだから、名前の間違いだけ通すのは筋が通らない。
    """

    def test_テーマに無い名前で上書きしたら落ちる(self):
        with pytest.raises(UnknownTokenError) as e:
            resolve_style(overrides={"size.box-hight": 40})   # height の綴り違い
        assert "size.box-hight" in str(e.value)

    def test_ある名前での上書きは通る(self):
        assert resolve_style(overrides={"size.box-h": 40}).num("size.box-h") == 40

    def test_鍵の欠けたテーマは描く前に落ちる(self):
        # 欠けたまま描き始めると、その鍵を引く部品に当たった時点で
        # 組みかけのSVGを捨てることになる
        with pytest.raises(IncompleteThemeError):
            resolve_style(theme={"font.size": 12})


class TestRanges:
    @pytest.mark.parametrize("key", sorted(TOKEN_RANGES))
    def test_範囲の下と上を外れたら描く前に例外になる(self, key):
        lo, hi = TOKEN_RANGES[key]
        with pytest.raises(TokenRangeError):
            resolve_style(overrides={key: lo - 1})
        with pytest.raises(TokenRangeError):
            resolve_style(overrides={key: hi + 1})

    @pytest.mark.parametrize("key", sorted(TOKEN_RANGES))
    def test_範囲の中の値は通る(self, key):
        lo, hi = TOKEN_RANGES[key]
        assert resolve_style(overrides={key: (lo + hi) / 2}).num(key) == (lo + hi) / 2

    def test_既定のテーマ自身が範囲を守っている(self):
        resolve_style()


class TestRegistry:
    def test_台帳に無い部品は名前を挙げて断る(self):
        with pytest.raises(KeyError):
            render_component("そんな部品はない", {}, resolve_style())

    def test_例のファイルを読み込まなくても基本の部品は登録済み(self):
        assert {"box", "edge", "frame"} <= set(known_kinds())
