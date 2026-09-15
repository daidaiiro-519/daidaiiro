"""総当たりの検証を手で走らせる入口 ── 中身は svg_engine.verify にある。

自動テスト（tests/test_matrix.py）と同じ表を使う。こちらは崩れた図の名前を
その場で読みたいときのため。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

from svg_engine.verify import check, check_shapes  # noqa: E402,F401
from svg_engine.examples.matrix_cases import build_all  # noqa: E402

rows = [(scale, name, check(svg) + check_shapes(svg)) for scale, name, svg in build_all()]
fail = sum(1 for _, _, f in rows if f)
print(f"{len(rows)} 通り、崩れ {fail} 件")
for scale, name, f in rows:
    if f:
        print(f"  NG [{scale}] {name}")
        for x in list(dict.fromkeys(f))[:3]:
            print(f"       {x}")
