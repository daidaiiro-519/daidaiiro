# SPDX-License-Identifier: MIT
"""見た目を、正本のファイルから読む。

**見た目の文字列をここに保持しない** ── 正本は `references/acdr.css` である。
この側が持つのはトークンの組み立てと、塊への分割だけである。

| 名前 | 何を着せるか |
|---|---|
| `TOKENS` | トークンの表。本体の先頭に置く |
| `TOKENS_EMBED` | 同じ表を、`:host` でも解決する形で。iframe と Shadow の中へ流し込む |
| `BASE` | 頁 ・ タブ ・ 面 |
| `CODE` | コードと差分 |
| `MARK` | 印と、押すと開くラベル。**中へも流し込む** |
| `SECTION` | 節（決定 ・ なぜ ・ 形の変化 …） |

塊の境目は、正本の中の `/* == 名前 == */` が示す ──
**この側が境目を決めない**。決めると、正本を割り直したときに気づけない。
"""
from __future__ import annotations

import re as _re

from . import REFERENCES as _REF
from . import tokens as _tokens

TOKENS = _tokens.css(_tokens.load())
TOKENS_EMBED = _tokens.css(_tokens.load(), host=True)

_CSS = (_REF / "acdr.css").read_text(encoding="utf-8")
_BLOCKS = {m.group(1): m.group(2) for m in
      _re.finditer(r"/\* == (\w+) == \*/(.*?)(?=/\* == |\Z)", _CSS, _re.S)}
if set(_BLOCKS) != {"BASE", "CODE", "MARK", "SECTION"}:
    raise ValueError(f"acdr.css の塊が4つでない: {sorted(_塊)}")

BASE, CODE, MARK, SECTION = (_BLOCKS["BASE"], _BLOCKS["CODE"], _BLOCKS["MARK"], _BLOCKS["SECTION"])
