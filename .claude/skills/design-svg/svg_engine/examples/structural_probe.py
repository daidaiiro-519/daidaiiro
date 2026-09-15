"""構造の検査 ── 「図ごとにコアを触らなくてよい」と言えるかを、2点で試す。

①寸法がテーマに比例するか ── 書体と間隔を大きく／小さくしても崩れないか。
  絶対値の定数が残っていれば、ここで露見する。
②配置が任意の位相を扱えるか ── 未検証の形で壊れないか。

通ることではなく、どこで壊れるかを見るための試験。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import DEFAULT_THEME, render_figure  # noqa: E402
from svg_engine.tokens import num  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)

BASE_NODES = [{"id": c, "label": {"a": "受け口", "b": "ユースケース", "c": "業務サービス",
                                  "d": "モデル"}[c]} for c in "abcd"]
BASE_EDGES = [{"from": "a", "to": "b", "label": "呼ぶ"},
              {"from": "b", "to": "c"}, {"from": "c", "to": "d"}]
BASE_GROUPS = [{"label": "領域の内側", "members": ["b", "c"]}]


def scaled(k: float) -> dict:
    """書体と間隔を一律に k 倍したテーマ。色は変えない。"""
    t = dict(DEFAULT_THEME)
    for key in ("font.size", "font.size-small", "size.box-h", "size.box-min-w",
                "size.box-pad-x", "size.gap-rank", "size.gap-order", "size.box-radius"):
        t[key] = num(DEFAULT_THEME, key) * k
    return t


cases: list[tuple[str, str]] = []

# ① テーマの倍率を振る
for k, name in [(0.75, "0.75倍"), (1.0, "等倍"), (2.5, "2.5倍")]:
    svg = render_figure(BASE_NODES, BASE_EDGES, groups=BASE_GROUPS,
                        direction="TB", theme=scaled(k))
    cases.append((f"① 書体と間隔 {name}", svg))

# ② 位相の際どい形
TOPO: dict[str, dict[str, list]] = {
    "非連結（辺が無い塊が2つ）": dict(
        nodes=[{"id": c, "label": c.upper()} for c in "abcd"],
        edges=[{"from": "a", "to": "b"}, {"from": "c", "to": "d"}], groups=[]),
    "自己ループ": dict(
        nodes=[{"id": c, "label": c.upper()} for c in "ab"],
        edges=[{"from": "a", "to": "a", "label": "自分へ"}, {"from": "a", "to": "b"}],
        groups=[]),
    "多重辺（同じ2点を2本）": dict(
        nodes=[{"id": c, "label": c.upper()} for c in "ab"],
        edges=[{"from": "a", "to": "b", "label": "往"},
               {"from": "a", "to": "b", "label": "復"}], groups=[]),
    "1節点が2つの群に属す": dict(
        nodes=[{"id": c, "label": c.upper()} for c in "abc"],
        edges=[],
        groups=[{"label": "甲", "members": ["a", "b"]},
                {"label": "乙", "members": ["b", "c"]}]),
    "空の群": dict(
        nodes=[{"id": c, "label": c.upper()} for c in "ab"],
        edges=[], groups=[{"label": "空", "members": []}]),
    "節点1つだけ": dict(
        nodes=[{"id": "a", "label": "ひとつ"}], edges=[], groups=[]),
}

for name, kw in TOPO.items():
    try:
        svg = render_figure(kw["nodes"], kw["edges"], groups=kw["groups"], direction="TB")
        cases.append((f"② {name}", svg))
    except Exception as e:  # noqa: BLE001 — どこで壊れるかを見たいので握って記録する
        cases.append((f"② {name}", f'<p style="color:#8C2F39">{type(e).__name__}: {e}</p>'))

cards = "".join(f'<section class="c"><h2>{n}</h2><div class="s">{s}</div></section>'
                for n, s in cases)
html = f"""<title>構造の検査</title>
<style>body{{font-family:sans-serif;padding:2rem;background:#fff;display:flex;
flex-direction:column;gap:1.1rem}}
.c{{border:1px solid #ddd;border-radius:8px;padding:1rem}}
h2{{font-size:.88rem;margin:0 0 .6rem}} .s{{overflow-x:auto}} svg{{max-width:100%;height:auto}}</style>
{cards}
"""
(OUT / "structural_probe.html").write_text(html, encoding="utf-8")
print(f"{len(cases)} 件")
for n, s in cases:
    print(("  失敗 " if s.startswith("<p") else "  描けた ") + n)
