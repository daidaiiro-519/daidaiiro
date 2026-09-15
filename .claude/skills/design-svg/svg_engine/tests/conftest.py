"""svg_engine は Waffle 本体の import グラフの外にある独立パッケージなので、
本体の tests/ とは別に、自分で自分の import 経路を通す。
"""
from __future__ import annotations

import pathlib
import sys

_ENGINE = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(_ENGINE.parent))       # 親ディレクトリ ── svg_engine を import する


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
