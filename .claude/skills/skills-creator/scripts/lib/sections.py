# SPDX-License-Identifier: MIT
"""節の構成が、対応する雛形を満たすかを見る。

**節の名前は、雛形が持つ。** この側に一覧を書かない ── 書くと、雛形を直した瞬間に
食い違う。

**並び順は問わない。** 見るのは有無だけである ── 順序まで固定すると、節を1つ足した
Skill が全部不合格になる。
"""
from __future__ import annotations

import pathlib
import re

HEADING = re.compile(r"^##\s+(.+?)\s*$")
PLACEHOLDER = re.compile(r"\{\{.*?\}\}")


def headings(path: pathlib.Path) -> list[str]:
    """`## ` の節の名前を、並びのまま返す。"""
    return [m.group(1) for line in path.read_text(encoding="utf-8").split("\n")
            if (m := HEADING.match(line))]


def missing(document: pathlib.Path, template: pathlib.Path) -> list[str]:
    """雛形に在って、文書に無い節を返す。

    **差し込む場所を持つ節は要求しない** ── 雛形の `{{…}}` は名前ではない。
    """
    required = [x for x in headings(template) if not PLACEHOLDER.search(x)]
    present = set(headings(document))
    return [x for x in required if x not in present]
