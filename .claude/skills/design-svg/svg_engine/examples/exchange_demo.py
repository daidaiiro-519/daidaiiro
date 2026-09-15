"""やり取り部品の実演 ── 普通の往復と、分かれ(cases)を持つ版の2枚。"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import render_chart  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

plain = render_chart("exchange", {
    "participants": ["Orchestrator", "Waffle", "advisor"],
    "steps": [
        {"from": "Orchestrator", "to": "Waffle", "label": "骨格を作る"},
        {"from": "Waffle", "to": "Orchestrator", "label": "書き方の指針", "kind": "return"},
        {"from": "Orchestrator", "to": "advisor", "label": "敵対的に確かめる"},
        {"from": "advisor", "to": "Orchestrator", "label": "反証、または支持", "kind": "return"},
    ],
})

branching = render_chart("exchange", {
    "participants": ["呼ぶ側", "文書", "型"],
    "steps": [
        {"from": "呼ぶ側", "to": "文書", "label": "逸脱していないか判定する"},
        {"from": "文書", "to": "型", "label": "読み方の指針を引く"},
        {"from": "文書", "to": "呼ぶ側", "label": "適合を返す", "kind": "return"},
        {"from": "文書", "to": "呼ぶ側", "label": "適合しない箇所を返す", "kind": "return"},
    ],
    "groups": [
        {"label": "判定の結果で分かれる", "cases": [
            {"name": "適合しているとき", "span": [2, 2]},
            {"name": "適合していないとき", "span": [3, 3]},
        ]},
    ],
})

html = f"""<title>svg_engine やり取り</title>
<style>body{{font-family:sans-serif;padding:2rem;background:#fff;display:flex;flex-direction:column;gap:2rem}}
.card{{border:1px solid #ddd;border-radius:8px;padding:1rem;max-width:640px}}
h2{{font-size:.95rem;margin:0 0 .8rem}} svg{{max-width:100%;height:auto;display:block}}</style>
<div class="card"><h2>やり取り（往復）</h2>{plain}</div>
<div class="card"><h2>やり取り＋分かれ(cases)</h2>{branching}</div>
"""
out_path = OUT / "exchange_demo.html"
out_path.write_text(html, encoding="utf-8")
print(f"wrote {out_path}")
