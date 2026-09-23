"""テーマと骨組みが満たすこと。

**配色の正本は1つである** ── 図の側にも色を書くと、テーマを替えたときに
図だけが前の配色のまま残る（4配色のうち1つで、実際に文字が消えた）。
"""
from __future__ import annotations

import pathlib
import sys
import tempfile

import pytest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import render_deck as _render  # noqa: E402
from lib import themes as _themes  # noqa: E402
from lib import validate_input as _vi  # noqa: E402


class Testテーマ:
    def test_4本とも同じ鍵を持つ(self):
        sets = [set(_themes.tokens(f)) for f in _themes.theme_files()]
        assert sets and all(s == sets[0] for s in sets)

    def test_検出は0件である(self):
        """**0件にできるものだけを検査する。**"""
        assert _themes.findings([]) == []

    @pytest.mark.parametrize("name", _themes.theme_names())
    def test_役割ごとの色へ複製できる(self, name):
        t = _themes.as_roles(name)
        assert set(t) == set(_themes.FIGURE_ROLES)
        assert all(v.startswith("#") for v in t.values())

    def test_知らない名前は例外にする(self):
        with pytest.raises(ValueError):
            _themes.as_roles("無いテーマ")

    def test_複製した色は正本と同じである(self):
        """**複製であって、別の値ではない。**"""
        name = _themes.theme_names()[0]
        raw = _themes.tokens(_themes.theme_path(name))
        for role, css_key in _themes.FIGURE_ROLES.items():
            assert _themes.as_roles(name)[role] == raw[css_key]


class Test組み立て:
    """**3つで組む** ── 型が形を、入力が中身を、テーマが配色を持つ。"""

    def _deck(self, theme="warm-paper", title="ためし"):
        import json
        d = json.loads((pathlib.Path(__file__).resolve().parents[2]
                        / "references" / "deck-example.json").read_text(encoding="utf-8"))
        d["title"], d["theme"] = title, theme
        return d

    def test_テーマを貼って組める(self):
        body = _render.build(self._deck())
        assert "<title>ためし</title>" in body
        assert "▼ テーマ" in body and "▲ テーマここまで" in body
        assert _themes.tokens(_themes.theme_path("warm-paper"))["--ground"] in body

    def test_組んだ1枚は検査を通る(self):
        """**区切りが無いと、テーマの外の直書きを判定できない。**"""
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "deck.html"
            out.write_text(_render.build(self._deck("deep-navy")), encoding="utf-8")
            assert _themes.findings([str(out)]) == []

    def test_同じ入力からは同じ1枚が出る(self):
        """**冪等である** ── 日付も乱数も読まない。"""
        assert _render.build(self._deck()) == _render.build(self._deck())

    @pytest.mark.parametrize("name", _themes.theme_names())
    def test_どのテーマでも組める(self, name):
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "deck.html"
            out.write_text(_render.build(self._deck(name)), encoding="utf-8")
            assert _themes.findings([str(out)]) == []

    def test_枚の数がそのまま出る(self):
        body = _render.build(self._deck())
        assert body.count('<section class="slide') == 2
        assert '<span id="total">2</span>' in body

    def test_ラベルが枚と同じ並びで出る(self):
        """**0から数える** ── 先頭に空を足すと、全部の枚が1つ前のラベルを出す。"""
        d = self._deck()
        body = _render.build(d)
        assert ('const LABELS = ["' + d["slides"][0]["label"] + '"') in body


class Test入力の検査:
    """**形では書けない規則を、機械に見させる** ── 散文の規定は破れる。"""

    def _deck(self, **slide):
        base = {"label": "ためし", "layout": "single", "heading": "断定形の主張",
                "blocks": [{"kind": "text", "body": "本文"}]}
        base.update(slide)
        return {"title": "題", "theme": "warm-paper", "slides": [base]}

    def test_例は検査を通る(self):
        import json
        d = json.loads((pathlib.Path(__file__).resolve().parents[2]
                        / "references" / "deck-example.json").read_text(encoding="utf-8"))
        assert _vi.check(d) == []

    def test_大きい要素が4つで止まる(self):
        big = [{"kind": "stat", "value": str(i), "caption": "条件"} for i in range(4)]
        bad = _vi.check(self._deck(blocks=big))
        assert any("大きい要素" in e for e in bad)

    def test_強調が2か所で止まる(self):
        flow = {"kind": "flow", "rows": [{"title": "甲", "mark": True},
                                         {"title": "乙", "mark": True}]}
        assert any("強調" in e for e in _vi.check(self._deck(blocks=[flow])))

    def test_問いの見出しで止まる(self):
        assert any("問いの形" in e for e in _vi.check(self._deck(heading="移行は終わったか")))

    def test_見出しの無い枚で止まる(self):
        d = self._deck()
        del d["slides"][0]["heading"]
        assert any("見出しが無い" in e for e in _vi.check(d))

    def test_出典が本文に在ると止まる(self):
        blocks = [{"kind": "text", "body": "出典　arXiv 0000.00000"}]
        assert any("出典" in e for e in _vi.check(self._deck(blocks=blocks)))

    def test_列の役割が枚で食い違うと止まる(self):
        col = lambda r: {"role": r, "blocks": [{"kind": "text", "body": "本文"}]}
        d = self._deck()
        d["slides"] = [
            {"label": "甲", "layout": "cols", "heading": "主張",
             "columns": [col("いま"), col("提案")]},
            {"label": "乙", "layout": "cols", "heading": "主張",
             "columns": [col("提案"), col("いま")]}]
        assert any("対応づけを崩さない" in e for e in _vi.check(d))

    def test_無い要素は形で止まる(self):
        bad = _vi.check(self._deck(blocks=[{"kind": "無い要素"}]))
        assert bad and all(e.startswith("形:") for e in bad)

    def test_無いテーマで止まる(self):
        d = self._deck()
        d["theme"] = "無いテーマ"
        assert any("themes/ に無い" in e for e in _vi.check(d))
