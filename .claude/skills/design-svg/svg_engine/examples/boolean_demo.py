"""ブーリアン演算の実演 ── 重なる2つの円で、和・積・差を確かめる。"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import render_chart  # noqa: E402
from svg_engine.boolean import circle_polygon  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

a = circle_polygon(60, 60, 45)
b = circle_polygon(100, 60, 45)

svgs = {op: render_chart("boolean", {"shapes": [a, b], "op": op})
       for op in ("union", "intersect", "subtract")}

cards = "".join(f'<div class="card"><h2>{k}</h2>{v}</div>' for k, v in svgs.items())
html = f"""<title>svg_engine ブーリアン演算</title>
<style>body{{font-family:sans-serif;padding:2rem;background:#fff;display:flex;gap:1.5rem}}
.card{{border:1px solid #ddd;border-radius:8px;padding:1rem}}
h2{{font-size:.9rem;margin:0 0 .6rem}} svg{{display:block}}</style>
{cards}
"""
out_path = OUT / "boolean_demo.html"
out_path.write_text(html, encoding="utf-8")
print(f"wrote {out_path}")
