"""囲みの耐久試験 ── 「同じ段に固まって並ぶ2群」以外の形でも成り立つか。

先の修正（囲みがあるとき節点の間隔を広げる）が、たまたま今の例に効いた
だけの調整でないかを確かめる。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import render_figure  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

CASES: dict[str, dict[str, list]] = {}

# ① 3群を横に並べる（2群でしか試していなかった）
CASES["3群を横に"] = dict(
    nodes=[{"id": c, "label": c.upper()} for c in "abcdef"],
    edges=[],
    groups=[{"label": "甲", "members": ["a", "b"]},
            {"label": "乙", "members": ["c", "d"]},
            {"label": "丙", "members": ["e", "f"]}])

# ② 群が段をまたぐ（縦方向の枠の重なりは、横の間隔を広げても直らないはず）
CASES["群が段をまたぐ"] = dict(
    nodes=[{"id": c, "label": c.upper()} for c in "abcd"],
    edges=[{"from": "a", "to": "b"}, {"from": "b", "to": "c"}, {"from": "c", "to": "d"}],
    groups=[{"label": "上の群", "members": ["a", "b"]},
            {"label": "下の群", "members": ["c", "d"]}])

# ③ 群の要素が飛び飛びになる（並び順が群でまとまらない場合）
CASES["群が飛び飛び"] = dict(
    nodes=[{"id": c, "label": c.upper()} for c in "abcd"],
    edges=[{"from": "a", "to": "c"}, {"from": "b", "to": "d"}],
    groups=[{"label": "甲", "members": ["a", "d"]},
            {"label": "乙", "members": ["b", "c"]}])

# ④ 群に属さない節点が混ざる
CASES["群外の節点が混ざる"] = dict(
    nodes=[{"id": c, "label": c.upper()} for c in "abcde"],
    edges=[],
    groups=[{"label": "甲", "members": ["a", "b"]},
            {"label": "乙", "members": ["d", "e"]}])

# ⑤ 1つの節点だけの群が並ぶ
CASES["単独の群が並ぶ"] = dict(
    nodes=[{"id": c, "label": c.upper()} for c in "abc"],
    edges=[],
    groups=[{"label": "甲", "members": ["a"]},
            {"label": "乙", "members": ["b"]},
            {"label": "丙", "members": ["c"]}])

cards = []
for name, kw in CASES.items():
    svg = render_figure(kw["nodes"], kw["edges"], groups=kw["groups"], direction="TB")
    cards.append(f'<section class="c"><h2>{name}</h2><div class="s">{svg}</div></section>')

html = f"""<title>囲みの耐久試験</title>
<style>body{{font-family:sans-serif;padding:2rem;background:#fff;display:flex;
flex-direction:column;gap:1.2rem}}
.c{{border:1px solid #ddd;border-radius:8px;padding:1rem}}
h2{{font-size:.9rem;margin:0 0 .7rem}} .s{{overflow-x:auto}} svg{{max-width:100%;height:auto}}</style>
{"".join(cards)}
"""
(OUT / "group_stress.html").write_text(html, encoding="utf-8")
print(f"wrote {len(CASES)} cases")
