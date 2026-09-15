"""量9種の実演 ── 全体と部分・量の大小・順位・時間変化・分布・偏差・相関・
流量・空間を、それぞれ render_chart() で1枚ずつ描く。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import render_chart  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

charts = {
    "全体と部分 (pie)": render_chart("pie", {
        "slices": [{"name": "文章の部品", "value": 10}, {"name": "図の部品", "value": 5}],
        "centre": "15",
    }),
    "量の大小 (bars)": render_chart("bars", {
        "bars": [{"name": "x-prompt-write", "value": 623}, {"name": "x-prompt-query", "value": 216},
                 {"name": "x-render", "value": 157}, {"name": "x-render-order", "value": 156}],
        "axis_label": "使用回数",
    }),
    "順位 (ranking)": render_chart("ranking", {
        "items": [{"name": "条件による選択", "value": 62}, {"name": "やり取りの順序", "value": 49},
                  {"name": "向きのある関係", "value": 30}, {"name": "区画への所属", "value": 21},
                  {"name": "一列に並ぶ", "value": 7}],
    }),
    "時間変化 (lanes)": render_chart("lanes", {
        "rows": [{"name": "調べる", "bars": [{"from": 0, "to": 3}]},
                 {"name": "決める", "bars": [{"from": 3, "to": 5}]},
                 {"name": "引き継ぐ", "bars": [{"from": 5, "to": 6}]},
                 {"name": "作る", "bars": [{"from": 6, "to": 11}]}],
        "axis_label": "日",
    }),
    "分布 (bars)": render_chart("bars", {
        "bars": [{"name": "2-4", "value": 38}, {"name": "5-8", "value": 64},
                 {"name": "9-12", "value": 15}, {"name": "13-16", "value": 8}],
        "axis_label": "節点の数",
    }),
    "偏差 (bars+baseline)": render_chart("bars", {
        "bars": [{"name": "x-prompt-write", "value": 19}, {"name": "x-render", "value": 74},
                 {"name": "x-render-level", "value": 2}, {"name": "x-render-order", "value": 0}],
        "baseline": 24,
    }),
    "相関 (scatter)": render_chart("scatter", {
        "points": [{"name": "文書の検証", "x": 8, "y": 7}, {"name": "描画", "x": 5, "y": 6},
                   {"name": "設定の読み込み", "x": 2, "y": 2}, {"name": "配置の検査", "x": 7, "y": 3}],
        "x_label": "差別化の大きさ", "y_label": "複雑さ",
    }),
    "流量 (flow)": render_chart("flow", {
        "links": [{"from": "構造化データ", "to": "Markdown", "value": 7},
                  {"from": "構造化データ", "to": "HTML", "value": 3}],
    }),
    "空間 (spatial)": render_chart("spatial", {
        "items": [{"name": "受け口", "depth": 0}, {"name": "ユースケース", "depth": 1},
                  {"name": "業務サービス", "role": "focus", "depth": 2}, {"name": "モデル", "depth": 3}],
    }),
}

cards = "".join(f'<div class="card"><h2>{k}</h2>{v}</div>' for k, v in charts.items())
html = f"""<title>svg_engine 量9種</title>
<style>
  body{{font-family:sans-serif;background:#fff;padding:2rem;display:flex;flex-wrap:wrap;gap:1.5rem}}
  .card{{border:1px solid #ddd;border-radius:8px;padding:1rem;max-width:560px}}
  h2{{font-size:.95rem;margin:0 0 .6rem}}
  svg{{max-width:100%;height:auto;display:block}}
</style>
{cards}
"""
out_path = OUT / "quantity_demo.html"
out_path.write_text(html, encoding="utf-8")
print(f"wrote {out_path}")
