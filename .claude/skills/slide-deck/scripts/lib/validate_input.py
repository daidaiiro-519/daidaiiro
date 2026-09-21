# SPDX-License-Identifier: MIT
"""デッキの入力を検査する。**組み立てより前に、1件でも出れば止まる。**

形は `references/slide-deck.schema.json` が見る。ここが見るのは、
**形では書けない規則**である ── 設計規則（`references/design-rules.md`）のうち、
数えれば判定できるものを機械に見させる。散文の規定は破れる。
"""
from __future__ import annotations

import json
import re

from . import REFERENCES
from . import themes as _themes

SCHEMA = REFERENCES / "slide-deck.schema.json"

# 大きい要素。**4つ以上あると、どれが主張か消える**（設計規則 §2）
BIG = {"stat", "figure", "punch", "card", "pair", "recap"}
BIG_MAX = 3

# 見出しは1〜2行に収める（§1）。h2 は 36px で、1枚の幅に約30字が入る
HEADING_MAX = 60
# 締めは1〜2行（§2）
CLOSE_MAX = 120

# 強調は1か所だけ（§2・§8 の4）
MARK_MAX = 1


def _blocks(slide: dict):
    """枚が持つ要素を、列の内も外も同じ並びで返す。"""
    yield from slide.get("blocks", [])
    for c in slide.get("columns", []):
        yield from c["blocks"]


def shape(deck: dict) -> list[str]:
    """形を検査する。**jsonschema が無ければ、無いと報告する** ── 黙って通さない。"""
    try:
        import jsonschema
    except ModuleNotFoundError:
        return ["jsonschema が無いので、形の検査を実行していない"]
    v = jsonschema.Draft202012Validator(json.loads(SCHEMA.read_text(encoding="utf-8")))
    return ["形: " + "/".join(map(str, e.path)) + " ── " + e.message
            for e in sorted(v.iter_errors(deck), key=lambda x: list(x.path))]


def wiring(deck: dict) -> list[str]:
    """並べ方と、渡した中身が噛み合っているかを検査する。"""
    bad = []
    if deck.get("theme") and deck["theme"] not in _themes.theme_names():
        bad.append(f'テーマ「{deck["theme"]}」が references/themes/ に無い ── '
                   f'在るのは {" ・ ".join(_themes.theme_names())} である')
    for i, s in enumerate(deck.get("slides", []), 1):
        at = f"{i}枚目（{s.get('label', '名前なし')}）"
        layout = s.get("layout")
        if layout in ("cols", "cols-3") and not s.get("columns"):
            bad.append(f"{at}: {layout} なのに columns が無い")
        if layout in ("single", "center") and not s.get("blocks"):
            bad.append(f"{at}: {layout} なのに blocks が無い")
        if layout == "cols-3" and len(s.get("columns", [])) != 3:
            bad.append(f"{at}: cols-3 の列が {len(s.get('columns', []))} 本である")
        if layout == "cols" and len(s.get("columns", [])) != 2:
            bad.append(f"{at}: cols の列が {len(s.get('columns', []))} 本である")
        if layout != "cover" and s.get("blocks") and s.get("columns"):
            bad.append(f"{at}: blocks と columns の両方を渡している ── どちらか1つにする")
        if layout != "cover" and not s.get("heading"):
            bad.append(f"{at}: 見出しが無い ── 枚で言い切ることを1文で書く")
        if layout == "cover" and (s.get("blocks") or s.get("columns")):
            bad.append(f"{at}: 表紙は要素を持たない")
    return bad


def limits(deck: dict) -> list[str]:
    """設計規則のうち、数えれば判定できるものを検査する。"""
    bad = []
    for i, s in enumerate(deck.get("slides", []), 1):
        at = f"{i}枚目（{s.get('label', '名前なし')}）"
        head = s.get("heading", "")
        if len(head) > HEADING_MAX:
            bad.append(f"{at}: 見出しが {len(head)} 字 ── {HEADING_MAX} 字（2行）に収める")
        if head.rstrip().endswith(("か", "か？", "?", "？")):
            bad.append(f"{at}: 見出しが問いの形である ── 断定形で書く")
        big = [b["kind"] for b in _blocks(s) if b["kind"] in BIG]
        if len(big) > BIG_MAX:
            bad.append(f"{at}: 大きい要素が {len(big)} つ（{' ・ '.join(big)}）── "
                       f"{BIG_MAX} つまでにする。どれが主張か消える")
        marks = sum(1 for b in _blocks(s) if b["kind"] == "flow"
                    for r in b["rows"] if r.get("mark"))
        if s.get("kicker", {}).get("tone") == "warm":
            marks += 1
        if marks > MARK_MAX:
            bad.append(f"{at}: 強調が {marks} か所 ── 1か所だけにする")
        for c in s.get("columns", []):
            if len(c.get("close", "")) > CLOSE_MAX:
                bad.append(f"{at}: 締めが {len(c['close'])} 字 ── "
                           f"{CLOSE_MAX} 字（2行）に収める")
        # **出典を、要素から離さない**（§2・§8 の3）
        for b in _blocks(s):
            if b["kind"] == "text" and re.search(r"(出典|arXiv|https?://)", b.get("body", "")):
                bad.append(f"{at}: 出典が本文に在る ── 支える要素（card の note ・ "
                           f"stat の source）の直下へ置く")
    return bad


def roles(deck: dict) -> list[str]:
    """列に与えた役割が、枚をまたいで同じかを検査する（§5）。

    **役割を与えた枚どうしだけを比べる** ── 与えていない枚は、
    役割を持たない枚であって、違反ではない。
    """
    seen: dict[int, list[str]] = {}
    bad = []
    for i, s in enumerate(deck.get("slides", []), 1):
        cols = s.get("columns") or []
        got = [c.get("role", "") for c in cols]
        if not any(got):
            continue
        n = len(cols)
        if n not in seen:
            seen[n] = got
        elif got != seen[n]:
            bad.append(f"{i}枚目（{s.get('label', '名前なし')}）: 列の役割が "
                       f"{' ／ '.join(got)} ── 他の枚は {' ／ '.join(seen[n])} である。"
                       f"対応づけを崩さない")
    return bad


def check(deck: dict) -> list[str]:
    """すべての検査を、同じ並びで実行する。**形が通らなければ、その先は見ない。**"""
    bad = shape(deck)
    if any(e.startswith("形:") for e in bad):
        return bad
    return bad + wiring(deck) + limits(deck) + roles(deck)
