"""1枚絵の実演 ── 上限側。実務図ではなく、装飾込みの完成イメージ1枚を組む。

配色・活字は「決め打ちの飾り」ではなく、この画布専用のテーマ(POSTER_THEME)
として渡す。box/pie等の実務部品と、gradient_rect/title/divider/icon の
装飾部品を、同じ`render_canvas`の上で自由に重ねられることを見せる。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import DEFAULT_THEME, render_canvas  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

# ── この1枚だけの配色。teal/rust の実務配色とは別の palette を、ここで選ぶ ──
POSTER_THEME = dict(DEFAULT_THEME)
POSTER_THEME.update({
    "color.ink": "#EDE6D6",
    "color.ink-soft": "#C9C1AE",
    "color.ink-faint": "#8E8874",
    "color.title": "#C9A24B",
    "color.accent": "#4C7A79",
    "color.accent-bg": "#2A3B42",
    "color.box-fill": "#1E2732",
    "color.box-stroke": "#3A4657",
    "chart.tones": ["#C9A24B", "#4C7A79", "#8E8874"],
    "chart.grid": "#3A4657",
    "chart.axis": "#5B6779",
    "font.size-display": 34,
})

w, h = 800, 480
layers = [
    {"kind": "gradient_rect", "x": 0, "y": 0,
     "props": {"width": w, "height": h, "direction": "radial",
               "stops": [(0.0, "#232E3C"), (1.0, "#12161E")]}},
    {"kind": "icon", "x": 40, "y": 44, "props": {"name": "spark", "size": 22}},
    {"kind": "title", "x": 74, "y": 40,
     "props": {"text": "描ける図は、部品次第。", "subtitle": "16の主張 × 関係7・量9", "width": 500}},
    {"kind": "divider", "x": 40, "y": 132, "props": {"width": 460, "kind": "wave"}},
    {"kind": "pie", "x": 470, "y": 60,
     "props": {"slices": [{"name": "関係", "value": 7}, {"name": "量", "value": 9}],
               "centre": "16"}},
    {"kind": "icon", "x": 40, "y": 170, "props": {"name": "ring", "size": 18}},
]

# 下段: 部品の内訳を、実務部品(spatial)を装飾部品と同じ画布に置いて共存させる
layers.append({"kind": "spatial", "x": 40, "y": 220,
              "props": {"items": [{"name": "自動配置（つながり・階層・包含・順序・循環）"},
                                  {"name": "縦のライフライン（やり取り）"},
                                  {"name": "入れ子の図（対応）"},
                                  {"name": "値から直接描く（量9種）", "role": "focus"}]}})

svg = render_canvas(w, h, layers, theme=POSTER_THEME)
html = f"""<title>svg_engine 1枚絵</title>
<style>body{{margin:0;padding:2rem;background:#0b0e13}} svg{{display:block;border-radius:12px}}</style>
{svg}
"""
out_path = OUT / "poster_demo.html"
out_path.write_text(html, encoding="utf-8")
print(f"wrote {out_path}")
