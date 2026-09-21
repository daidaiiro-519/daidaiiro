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
from lib import deck as _deck  # noqa: E402
from lib import themes as _themes  # noqa: E402


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
        """**写しであって、別の値ではない。**"""
        name = _themes.theme_names()[0]
        raw = _themes.tokens(_themes.theme_path(name))
        for role, css_key in _themes.FIGURE_ROLES.items():
            assert _themes.as_roles(name)[role] == raw[css_key]


class Test骨組み:
    def test_テーマを貼って起こせる(self):
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "deck.html"
            _deck.create(out, "warm-paper", "ためし")
            body = out.read_text(encoding="utf-8")
            assert "<title>ためし</title>" in body
            assert _deck.OPEN in body and _deck.CLOSE in body
            assert _themes.tokens(_themes.theme_path("warm-paper"))["--ground"] in body

    def test_起こした骨組みは検査を通る(self):
        """**区切りが無いと、テーマの外の直書きを判定できない。**"""
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "deck.html"
            _deck.create(out, "deep-navy", "ためし")
            assert _themes.findings([str(out)]) == []

    def test_同じ名前では起こさない(self):
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "deck.html"
            _deck.create(out, "warm-paper", "ためし")
            with pytest.raises(FileExistsError):
                _deck.create(out, "warm-paper", "ためし")

    @pytest.mark.parametrize("name", _themes.theme_names())
    def test_どのテーマでも起こせる(self, name):
        with tempfile.TemporaryDirectory() as d:
            out = pathlib.Path(d) / "deck.html"
            _deck.create(out, name, "ためし")
            assert _themes.findings([str(out)]) == []
