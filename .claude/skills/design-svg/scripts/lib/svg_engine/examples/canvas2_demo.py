"""グループの入れ子・クリップ・自由曲線の実演。"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import DEFAULT_THEME, render_canvas  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

THEME = dict(DEFAULT_THEME)
THEME.update({"color.shape-fill": "#2F6F6B", "color.shape-stroke": "#2F6F6B"})

# ① グループの入れ子 ── 3つのbox+iconの組を、1つのグループとしてまとめて回転する。
group_demo = {
    "x": 90, "y": 90,
    "rotate": 12,
    "children": [
        {"kind": "box", "x": 0, "y": 0, "props": {"label": "A"}},
        {"kind": "box", "x": 90, "y": 0, "props": {"label": "B"}},
        {"kind": "icon", "x": 30, "y": 44, "props": {"name": "spark", "size": 20}},
    ],
}

# ② クリップ ── 大きな自由曲線の塊(ブロブ)を、丸い枠でクリップする。
blob_path = ("M0,40 C0,10 30,-10 60,0 C95,12 100,45 80,65 "
            "C60,85 20,85 5,65 C-5,52 0,50 0,40 Z")
clipped_blob = {
    "kind": "path", "x": 260, "y": 40,
    "props": {"d": blob_path},
    "clip": {"kind": "dot", "x": 20, "y": 20, "props": {"radius": 34}},
}

# ③ 自由曲線 ── クリップ無しでそのまま。輪郭の線だけにする。
outline_blob = {
    "kind": "path", "x": 380, "y": 40,
    "props": {"d": blob_path, "filled": False},
}

svg = render_canvas(560, 220, [group_demo, clipped_blob, outline_blob], theme=THEME,
                    background="#FAFAF8")
html = f"""<title>svg_engine 入れ子・クリップ・自由曲線</title>
<style>body{{font-family:sans-serif;padding:2rem;background:#fff}}
svg{{border:1px solid #eee;border-radius:8px}}</style>
<h2>① グループ回転　② クリップ　③ 自由曲線(輪郭)</h2>
{svg}
"""
out_path = OUT / "canvas2_demo.html"
out_path.write_text(html, encoding="utf-8")
print(f"wrote {out_path}")
