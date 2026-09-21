# SPDX-License-Identifier: MIT
"""データから決まる CSS の値だけを作る。

**見た目の正本はここではない** ── `references/board.css` である。
この側が作るのは、入力の中身から計算しないと決まらない値だけである。
"""
from __future__ import annotations


def drop_numbering(origin_by_number: dict[int, int]) -> str:
    """除外した案の記号を、通過した案の次から振る。

    **記号は候補の識別子である** ── 除外した案も候補だったのに、
    道具の側は（案, 何が壊れるか）の対しか保持しないので記号が無い。
    """
    return "".join(f"#p{n} table:has(span.n.out){{counter-reset:drop {v}}}"
                   for n, v in sorted(origin_by_number.items()))
