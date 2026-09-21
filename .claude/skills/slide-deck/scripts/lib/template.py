# SPDX-License-Identifier: MIT
"""組み立ての型 ── HTML の形は、コードではなくテンプレートが持つ。

**形をコードの中の文字列に散らすと、枚ごとに違う形が出る**。ここが読むのは `references/slide-deck.template.html` 1枚で、
差し込む場所（`{{名前}}`）も、部品の名前も、そのファイルが決める。

    from .template import part
    part("flow-row", title="見出しになる1行", note="その補足")

**差し込む場所の過不足を、その場で例外にする** ── 埋め忘れも、余分な値も、
出てから気づく形にしない。
"""
from __future__ import annotations

import re

from . import REFERENCES

TEMPLATE = REFERENCES / "slide-deck.template.html"
_SLOT = re.compile(r"\{\{([a-z0-9_]+)\}\}")
_OPEN = re.compile(r'<template data-part="([a-z0-9-]+)">')


def _load() -> dict[str, str]:
    """部品の並びを読む。**1か所からしか読まない。**"""
    body = TEMPLATE.read_text(encoding="utf-8")
    at = [(m.start(), m.end(), m.group(1)) for m in _OPEN.finditer(body)]
    if not at:
        raise ValueError(f"{TEMPLATE} に部品が1つも無い")
    parts: dict[str, str] = {}
    for i, (_, end, name) in enumerate(at):
        stop = at[i + 1][0] if i + 1 < len(at) else len(body)
        chunk = body[end:stop]
        # **入れ子の <template> を、部品の切れ目と取り違えない** ──
        # 面の雛形は中に <template> を持つ。最後の閉じだけが切れ目である
        cut = chunk.rfind("</template>")
        if cut < 0:
            raise ValueError(f"部品 {name} が閉じていない")
        if name in parts:
            raise ValueError(f"部品の名前が重複している: {name}")
        parts[name] = chunk[:cut]
    return parts


_PARTS = _load()


def names() -> list[str]:
    """テンプレートが持つ部品の名前。"""
    return sorted(_PARTS)


def part(name: str, **slots: object) -> str:
    """部品1つを組む。**差し込む場所と、渡した値が、過不足なく一致する。**"""
    if name not in _PARTS:
        raise KeyError(f"テンプレートに無い部品: {name}。使えるのは {names()} である")
    frag = _PARTS[name]
    need = set(_SLOT.findall(frag))
    got = set(slots)
    if need - got:
        raise KeyError(f"部品 {name}: 差し込む値が足りない {sorted(need - got)}")
    if got - need:
        raise KeyError(f"部品 {name}: テンプレートに無い値を渡した {sorted(got - need)}")
    return _SLOT.sub(lambda m: str(slots[m.group(1)]), frag)
