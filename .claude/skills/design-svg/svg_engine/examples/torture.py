"""耐久試験 ── サイクル・複数段をまたぐ辺・交差する辺を、わざと1枚に混ぜる。

崩れずに描けるか（クラッシュしない、辺が節点を突っ切らない）を目で確かめる。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import render_figure  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

nodes = [{"id": n, "label": n} for n in
         ["A", "B", "C", "D", "E", "F", "G", "H"]]
edges: list[dict] = [
    {"from": "A", "to": "B"},
    {"from": "A", "to": "C"},
    {"from": "B", "to": "D"},
    {"from": "C", "to": "D"},
    {"from": "B", "to": "E"},   # 交差を誘発する斜めの辺
    {"from": "C", "to": "F", "label": "斜め"},
    {"from": "D", "to": "G"},
    {"from": "H", "to": "G"},
    {"from": "A", "to": "G", "label": "3段またぎ"},  # A(0段)→G(3段) を直結。仮節点を経由するはず
    {"from": "G", "to": "A", "label": "循環", "dashed": True},  # 意図的なサイクル
]

svg = render_figure(nodes, edges, direction="TB")
html = f"""<title>svg_engine 耐久試験</title>
<style>body{{font-family:sans-serif;padding:2rem;background:#fff}}
svg{{max-width:100%;height:auto;border:1px solid #eee}}</style>
<h2>サイクル・3段またぎ・交差する辺を混ぜた図</h2>
{svg}
"""
out_path = OUT / "torture.html"
out_path.write_text(html, encoding="utf-8")
print(f"wrote {out_path}")
