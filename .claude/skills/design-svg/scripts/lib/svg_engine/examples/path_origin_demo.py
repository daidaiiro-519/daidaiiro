"""path部品の座標正規化を検証 ── 原点から遠く離れた場所に書いたdでも、
render_chart()単体でちゃんと画布内に収まって見えるか。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import render_chart  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

# わざと(0,0)から遠い(500,300)付近の座標で書く。以前はこれが画布の外へ
# 出て見えなかった。
far_blob = ("M500,340 C500,310 530,290 560,300 C595,312 600,345 580,365 "
           "C560,385 520,385 505,365 C495,352 500,350 500,340 Z")

svg = render_chart("path", {"d": far_blob})
html = f"""<title>svg_engine path原点正規化</title>
<style>body{{font-family:sans-serif;padding:2rem;background:#fff}}
svg{{border:1px solid #eee;border-radius:8px}}</style>
<h2>原点(0,0)から遠い座標で書いたdを、render_chart単体で描く</h2>
{svg}
"""
out_path = OUT / "path_origin_demo.html"
out_path.write_text(html, encoding="utf-8")
print(f"wrote {out_path}")
