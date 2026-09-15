"""宣言した語が、すべて絵に出ているか。

仕様が必須と定めた欄を変換で落とすと、絵は成立しているのに宣言の一部が消える
── 幾何の検査では見つからない（重なりもはみ出しも起きないため）。実際に、
量の大小の2本目の軸と、分布の bins が落ちていた。

例の側に置くのは、ここで読む宣言が呼ぶ側のデータだからである。描く側の試験には
置かない。
"""
from __future__ import annotations

from .all_claims import CLAIMS, convert


def missing_words() -> list[tuple[str, list[str]]]:
    """言い分ごとに、宣言したのに絵へ出ていない語を返す。"""
    out = []
    for d in CLAIMS:
        svg = convert(d)
        frame = d.get("frame", {})
        words = [frame.get("bins")]
        words += [a.get("unit") for a in frame.get("axes", [])]
        words += [g.get("name") for g in frame.get("groups", [])]
        words += [it.get("name") for it in d.get("items", [])]
        miss = [str(w) for w in words if w and str(w) not in svg]
        if miss:
            out.append((d["asserts"], miss))
    return out
