#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 daidaiiro
"""盤面のHTMLを、文章だけに落とす。

    python3 to_prose.py <ブレストのフォルダ>

そのフォルダの `board.html` を読み、`board-prose.md` を書く。
用途は2つ ── 文章の検査（doc-writing-skills の gate）にかけること、
そして盤面の中身を、ブラウザを開かずに読み返せるようにすることである。

**この取り出しを、作業用の一時領域に置かない。**セッションが変わると
失われ、成果物から書き起こすことになる。実際にそうなった。
"""
from __future__ import annotations

import html as _h
import pathlib
import re
import sys

_DROP = re.compile(r"<(script|style)\b.*?</\1>", re.S | re.I)
_H2 = re.compile(r"<h2\b[^>]*>(.*?)</h2>", re.S | re.I)


def to_prose(page: str) -> str:
    s = _DROP.sub("", page)
    s = re.sub(r"<h1\b[^>]*>(.*?)</h1>", r"\n\n# \1\n", s, flags=re.S | re.I)
    s = _H2.sub(lambda m: "\n\n## " + m.group(1) + "\n", s)
    s = re.sub(r"<summary\b[^>]*>(.*?)</summary>", r"\n\n**\1**\n", s, flags=re.S | re.I)
    s = re.sub(r"<li\b[^>]*>", "\n- ", s, flags=re.I)
    s = re.sub(r"<blockquote\b[^>]*>", "\n> ", s, flags=re.I)
    s = re.sub(r"</(p|tr|div|section|table|ul|details|figcaption)>", "\n", s, flags=re.I)
    s = re.sub(r"</t[dh]>", "\n", s, flags=re.I)
    s = re.sub(r"<[^>]+>", "", s)
    s = _h.unescape(s)
    s = re.sub(r"[ \t　]+\n", "\n", s)
    s = re.sub(r"\n{3,}", "\n\n", s)
    return s.strip() + "\n"


def main(folder: str) -> int:
    here = pathlib.Path(folder)
    out = here / "board-prose.md"
    out.write_text(to_prose((here / "board.html").read_text(encoding="utf-8")), encoding="utf-8")
    print(f"{out} ── {out.stat().st_size} バイト")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        raise SystemExit(1)
    raise SystemExit(main(sys.argv[1]))
