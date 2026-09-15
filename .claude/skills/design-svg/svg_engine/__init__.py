"""svg_engine ── 構造化データからSVGを組み立てる、独立した描画エンジン。

構造（節点・辺・囲み）とスタイル（色・寸法・角丸等）を分けて持つ。新しい
描画部品は registry.py の台帳へ登録するだけで足せる。tokens.py のテーマを
差し替えれば、宣言を一切変えずに見た目だけを変えられる（CSSの:root差し替えに相当）。

このエンジンの契約は「構造化データを受け取る」ことだけである。受け取るのは
節点(nodes)・辺(edges)・囲み(groups)という、どんなグラフ図にも共通する一般名詞で、
そのデータが何を意味するか・何を言いたいかは知らない ── それは呼ぶ側が決める。

固有の語彙を持つ側から使うときは、この核の外に変換を置き、その語彙から
nodes/edges/groups へ直してから render_figure() を呼ぶ。何を受け取れるかは
catalog.py が目録として公開しているので、変換を書く人はそれだけを見れば済む。
"""
from . import (  # noqa: F401,E501
    shapes, shapes_decor, shapes_freeform, shapes_hex, shapes_interaction, shapes_quantity, shapes_table, shapes_titled,
)
from .canvas import render_canvas
from .radial import layout_radial
from .tree import layout_tree
from .compose import render_chart, render_figure
from .catalog import catalog, props_of, EXAMPLES
from .registry import known_kinds
from .tokens import DEFAULT_THEME

__all__ = ["render_figure", "render_chart", "render_canvas", "known_kinds",
           "DEFAULT_THEME", "layout_radial", "layout_tree",
           "catalog", "props_of", "EXAMPLES"]
