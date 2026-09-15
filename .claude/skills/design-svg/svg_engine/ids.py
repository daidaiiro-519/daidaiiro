"""SVGの中で使う識別子 ── 中身から決める。

識別子は文書の中で衝突しなければよく、順番である必要はない。通し番号にすると
同じ入力から違う出力が出て、部品が決定的であるという契約を破る（規約の試験で
実際に落ちた ── 同じ値で2回描くと違う識別子が出た）。

中身から決めれば、同じものは同じ識別子になり ── 定義が重なるが指す先は同じなので
害が無い ── 違うものは違う識別子になる。
"""
from __future__ import annotations

import hashlib


def stable_id(prefix: str, *parts: object) -> str:
    """中身から決まる識別子を返す。

    Args:
        prefix: 用途が読める短い接頭辞（"grad" / "clip" など）。
        *parts: 識別子を決める材料。同じ材料からは同じ識別子が出る。

    Returns:
        接頭辞つきの識別子。
    """
    return prefix + hashlib.sha1(repr(parts).encode()).hexdigest()[:8]
