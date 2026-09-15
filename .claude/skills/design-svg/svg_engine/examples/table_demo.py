"""table部品の検証 ── 契約を実際になぞって足した新部品を試す。

1枚目: 既定テーマ。列によって文字の長さが大きく違う入力で、はみ出さないか。
2枚目: 同じ宣言、テーマだけ差し替え(構造とスタイルの分離が本当に効くか)。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import DEFAULT_THEME, render_chart  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

props = {
    "headers": ["部品", "分類", "見つかった不具合"],
    "rows": [
        ["box", "節点系", "（無し）"],
        ["frame", "関係系", "ラベルが枠内の要素と重なった"],
        ["scatter", "量系", "端の点のラベルが枠外へはみ出した"],
        ["spatial", "量系", "固定幅220pxで長いラベルがはみ出し、装飾アイコンと衝突した"],
        ["table", "節点系（今回追加）", "契約9番（import忘れ）を実際に再現して確認した"],
    ],
}

svg1 = render_chart("table", props)

ALT = dict(DEFAULT_THEME)
ALT.update({
    "color.accent": "#7A4368", "color.accent-bg": "#F3E9F0",
    "color.ink": "#2A1B26", "chart.grid": "#E7D9E3",
    "color.box-stroke": "#D8C3D2", "font.size": 13,
})
svg2 = render_chart("table", props, theme=ALT)

html = f"""<title>svg_engine table</title>
<style>body{{font-family:sans-serif;padding:2rem;background:#fff;display:flex;flex-direction:column;gap:1.5rem}}
.card{{border:1px solid #ddd;border-radius:8px;padding:1rem;max-width:820px}}
h2{{font-size:.95rem;margin:0 0 .7rem}} svg{{max-width:100%;height:auto;display:block}}</style>
<div class="card"><h2>table（既定テーマ）</h2>{svg1}</div>
<div class="card"><h2>同じ宣言、別テーマ</h2>{svg2}</div>
"""
out_path = OUT / "table_demo.html"
out_path.write_text(html, encoding="utf-8")
print(f"wrote {out_path}")
