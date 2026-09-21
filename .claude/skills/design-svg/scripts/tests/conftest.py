"""svg_engine は Waffle 本体の import グラフの外にある独立パッケージなので、
本体の tests/ とは別に、自分で自分の import 経路を設定する。
"""
from __future__ import annotations

import pathlib
import sys

# 部品の置き場所（scripts/lib）を道に載せる ── そこに svg_engine が在る
_LIB = pathlib.Path(__file__).resolve().parents[1] / "lib"
sys.path.insert(0, str(_LIB))


import pytest


@pytest.fixture
def style():
    """既定のテーマを解決したもの。部品を単体で描くときに要る。"""
    from svg_engine.style import resolve_style
    from svg_engine.tokens import DEFAULT_THEME
    return resolve_style("plain", None, DEFAULT_THEME)


@pytest.fixture
def sample_props():
    """台帳の全部品を1つずつ描くための、最小の入力。

    実体は目録が持つ ── 見本は試験の道具であると同時に、利用側が最初に
    動かすための足がかりでもある。試験の中に置くと、外から使えない。
    """
    from svg_engine.catalog import EXAMPLES
    return dict(EXAMPLES)
