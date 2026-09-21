# SPDX-License-Identifier: MIT
"""デッキの骨組みを起こす。

**配色を枚の中に書かせない** ── 骨組みはテーマを貼る場所だけを持ち、
値は `references/themes/<名前>.css` の1か所から入る。
"""
from __future__ import annotations

import pathlib

from . import REFERENCES
from . import themes as _themes

TEMPLATE = REFERENCES / "deck-template.html"

# 骨組みがテーマを差し込む場所。**区切りは検査が見る印でもある** ──
# テーマの外に色の直書きが在るかを、この区切りで判定する。
OPEN, CLOSE = "/* ▼ テーマ", "/* ▲ テーマここまで */"


def create(out: pathlib.Path, theme: str = "warm-paper",
           title: str = "題を記入する") -> pathlib.Path:
    """骨組みにテーマを貼って、1枚の HTML を置く。"""
    if out.exists():
        raise FileExistsError(f"既に在る: {out} ── 作り直さない")
    css = pathlib.Path(_themes.theme_path(theme)).read_text(encoding="utf-8")
    body = TEMPLATE.read_text(encoding="utf-8")
    i, j = body.index(OPEN), body.index(CLOSE) + len(CLOSE)
    body = (body[:i] + f"{OPEN} {theme} ── references/themes/{theme}.css の中身 */\n"
            + css.rstrip() + f"\n  {CLOSE}" + body[j:])
    body = body.replace("<title>デッキの題名</title>", f"<title>{title}</title>")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(body, encoding="utf-8")
    return out
