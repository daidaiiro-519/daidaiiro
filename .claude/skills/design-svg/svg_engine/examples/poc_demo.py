"""PoC実演 ── 同じ宣言(nodes/edges)を、違うテーマで2通りに描いて見せる。

構造とスタイルが本当に分離できているかは、「宣言を1文字も変えずに、
テーマだけ差し替えて別の見た目になるか」で確かめる。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import DEFAULT_THEME, known_kinds, render_figure  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

# ── 図1: つながり（figure-notation.html の実例と同じ題材） ──────────
tsunagari_nodes = [
    {"id": "cli", "label": "CLI"},
    {"id": "mcp", "label": "MCP"},
    {"id": "usecase", "label": "ユースケース", "role": "focus"},
    {"id": "service", "label": "業務サービス"},
    {"id": "model", "label": "モデル"},
]
tsunagari_edges = [
    {"from": "cli", "to": "usecase"},
    {"from": "mcp", "to": "usecase"},
    {"from": "usecase", "to": "service", "label": "呼ぶ"},
    {"from": "service", "to": "model"},
]

# ── 図2: 階層（親子は「辺」として渡す ── children構造への変換はアダプタの仕事） ──
kaisou_nodes = [
    {"id": "domain", "label": "Domain"},
    {"id": "aggregate", "label": "集約", "role": "focus"},
    {"id": "entity", "label": "エンティティ"},
    {"id": "vo", "label": "値オブジェクト"},
]
kaisou_edges = [
    {"from": "domain", "to": "aggregate", "arrow": "none"},
    {"from": "aggregate", "to": "entity", "arrow": "none"},
    {"from": "aggregate", "to": "vo", "arrow": "none"},
]

# ── 別テーマ ── 宣言(nodes/edges)は一切変えず、トークンだけ差し替える ─────
ALT_THEME = dict(DEFAULT_THEME)
ALT_THEME.update({
    "color.box-fill": "#FFF7ED",
    "color.box-stroke": "#C2410C",
    "color.accent": "#9A3412",
    "color.accent-bg": "#FFEDD5",
    "color.ink": "#431407",
    "size.box-radius": 16,
    "size.box-h": 40,
    "font.size": 13,
})

svg1a = render_figure(tsunagari_nodes, tsunagari_edges, direction="TB")
svg1b = render_figure(tsunagari_nodes, tsunagari_edges, direction="TB", theme=ALT_THEME)
svg2 = render_figure(kaisou_nodes, kaisou_edges, direction="TB")
svg3 = render_figure(  # つながり + 囲み（frame）の組み合わせ実演
    tsunagari_nodes, tsunagari_edges,
    groups=[{"label": "ここは領域の内側", "members": ["usecase", "service"]}],
    direction="TB",
)

html = f"""<title>svg_engine PoC ── 構造とスタイルの分離</title>
<style>
  body{{font-family:sans-serif;background:#fff;padding:2rem;display:flex;flex-direction:column;gap:2rem}}
  .card{{border:1px solid #ddd;border-radius:8px;padding:1rem;max-width:640px}}
  h2{{font-size:1rem;margin:0 0 .8rem}}
  svg{{max-width:100%;height:auto;display:block}}
</style>
<div class="card"><h2>1a. つながり（既定テーマ）</h2>{svg1a}</div>
<div class="card"><h2>1b. 同じ宣言、別テーマ（構造は無改変）</h2>{svg1b}</div>
<div class="card"><h2>2. 階層（同じ核・同じ辺の仕組みで表現）</h2>{svg2}</div>
<div class="card"><h2>3. つながり + 囲み（frame部品の組み合わせ）</h2>{svg3}</div>
<p>登録済み部品: {", ".join(known_kinds())}</p>
"""

out_path = OUT / "poc_demo.html"
out_path.write_text(html, encoding="utf-8")
print(f"wrote {out_path}")
print(f"registered components: {known_kinds()}")
