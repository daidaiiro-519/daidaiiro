# SPDX-License-Identifier: MIT
"""デッキの入力（JSON）から、1枚の HTML を組む。

    python3 scripts/cli.py render <デッキの JSON>

**3つで組む。** 入力の形は `references/slide-deck.schema.json` が、
出来上がりの形は `references/slide-deck.template.html` が、
配色は `references/themes/<名前>.css` が持つ。ここが持つのは、
**どの値をどの部品へ差し込むか**だけである ── HTML の形をここへ書かない。

**同じ入力からは、同じ1枚が出る。** 日付も乱数も読まない。
"""
from __future__ import annotations

import json
import pathlib

from . import REFERENCES
from . import themes as _themes
from . import validate_input as _vi
from .template import part as _t

SCHEMA = REFERENCES / "slide-deck.schema.json"


def _esc(v: object) -> str:
    """文字列を、HTML の中へそのまま置ける形にする。

    **太字と強調だけは通す** ── `<b>` ・ `<i>` ・ `<em>` ・ `<mark>` ・ `<br>` は、
    枚の中で主従を付けるために要る。それ以外の札は文字として出る。
    """
    s = str(v).replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    for tag in ("b", "i", "em", "strong", "mark", "small"):
        s = s.replace(f"&lt;{tag}&gt;", f"<{tag}>").replace(f"&lt;/{tag}&gt;", f"</{tag}>")
    return s.replace("&lt;br&gt;", "<br>")


# 要素の種類と、それを組む部品。**種類を足すときは、型にも部品を足す** ──
# 片方だけに足すと、入力が通って出力が空になる
def _block(b: dict) -> str:
    kind = b["kind"]
    fn = _BLOCKS.get(kind)
    if fn is None:
        raise KeyError(f"知らない要素: {kind} ── 使えるのは {sorted(_BLOCKS)} である")
    return fn(b)


def _text(b: dict) -> str:
    return _t("text", body=_esc(b["body"]))


def _lede(b: dict) -> str:
    return _t("lede", body=_esc(b["body"]))


def _card(b: dict) -> str:
    rows = "".join(
        _t("card-row", text=_esc(r["text"]),
           note=_t("card-row-note", text=_esc(r["note"])) if r.get("note") else "")
        for r in b.get("rows", []))
    return _t("card", label=_esc(b.get("label", "")), claim=_esc(b["claim"]), rows=rows)


def _flow(b: dict) -> str:
    rows = "".join(
        _t("flow-row-mark" if r.get("mark") else "flow-row",
           title=_esc(r["title"]), note=_esc(r.get("note", "")))
        for r in b["rows"])
    return _t("flow", rows=rows)


def _stat(b: dict) -> str:
    return _t("stat", value=_esc(b["value"]),
              unit=_t("stat-unit", text=_esc(b["unit"])) if b.get("unit") else "",
              caption=_esc(b["caption"]),
              source=_t("stat-source", text=_esc(b["source"])) if b.get("source") else "")


def _boxes(b: dict) -> str:
    return _t("boxes", items="".join(
        _t("boxes-item", title=_esc(i["title"]), note=_esc(i.get("note", "")))
        for i in b["items"]))


def _pair(b: dict) -> str:
    def side(text: str, note: str) -> str:
        return _esc(text) + (_t("pair-note", text=_esc(note)) if note else "")
    return _t("pair", left=side(b["left"], b.get("left_note", "")),
              right=side(b["right"], b.get("right_note", "")),
              link=_esc(b.get("link", "")))


def _recap(b: dict) -> str:
    return _t("recap", items="".join(
        _t("recap-item", no=_esc(i["no"]), text=_esc(i["text"])) for i in b["items"]))


def _punch(b: dict) -> str:
    return _t("punch", body=_esc(b["body"]),
              sub=_t("punch-sub", text=_esc(b["sub"])) if b.get("sub") else "")


def _caveat(b: dict) -> str:
    return _t("caveat", body=_esc(b["body"]))


def _figure(b: dict) -> str:
    # 図は SVG の文字列のまま置く。**この Skill は図を描かない**
    return _t("figure", svg=b["svg"],
              caption=_t("figure-caption", text=_esc(b["caption"])) if b.get("caption") else "")


def _table(b: dict) -> str:
    head = (_t("table-head", cells="".join(_t("table-th", text=_esc(c)) for c in b["head"]))
            if b.get("head") else "")
    rows = "".join(_t("table-row", cells="".join(_t("table-td", text=_esc(c)) for c in r))
                   for r in b["rows"])
    return _t("table", head=head, rows=rows)


def _list(b: dict) -> str:
    return _t("list", items="".join(_t("list-item", text=_esc(i)) for i in b["items"]))


def _who(b: dict) -> str:
    return _t("who", items="".join(_t("who-item", text=_esc(i)) for i in b["items"]))


_BLOCKS = {"text": _text, "lede": _lede, "card": _card, "flow": _flow, "stat": _stat,
           "boxes": _boxes, "pair": _pair, "recap": _recap, "punch": _punch,
           "caveat": _caveat, "figure": _figure, "table": _table, "list": _list,
           "who": _who}


def _column(c: dict) -> str:
    return _t("column",
              role=_t("column-role", text=_esc(c["role"])) if c.get("role") else "",
              blocks="".join(_block(b) for b in c["blocks"]),
              close=_t("column-close", text=_esc(c["close"])) if c.get("close") else "")


def _kicker(s: dict, *, center: bool) -> str:
    k = s.get("kicker")
    if not k:
        return ""
    tone = " warm" if k.get("tone") == "warm" else ""
    return _t("kicker-center" if center else "kicker", text=_esc(k["text"]), tone=tone)


def slide(s: dict) -> str:
    """枚を1つ組む。**並べ方は layout が決める** ── 枚ごとに器を選ばせない。"""
    layout = s["layout"]
    if layout == "cover":
        return _t("slide-cover", kicker=_kicker(s, center=True),
                  heading=_esc(s.get("heading", "")),
                  lede=_t("cover-lede", text=_esc(s["lede"])) if s.get("lede") else "",
                  byline=_t("byline", text=_esc(s["byline"])) if s.get("byline") else "",
                  submitted=(_t("submitted", text=_esc(s["submitted"]))
                             if s.get("submitted") else ""))
    if layout in ("cols", "cols-3"):
        body = _t(layout, columns="".join(_column(c) for c in s["columns"]))
    else:
        body = _t("stack", blocks="".join(_block(b) for b in s["blocks"]))
    return _t("slide-center" if layout == "center" else "slide",
              kicker=_kicker(s, center=(layout == "center")),
              heading=(_t("heading", text=_esc(s["heading"])) if s.get("heading") else ""),
              body=body,
              note=_t("note", text=_esc(s["note"])) if s.get("note") else "")


def build(deck: dict) -> str:
    """デッキ1本を組む。**入力が通っていなければ、1バイトも出さない。**"""
    bad = _vi.check(deck)
    if bad:
        raise SystemExit("入力の検査が通っていない ── HTML は書き出さない:\n  "
                         + "\n  ".join("× " + e for e in bad))
    theme = pathlib.Path(_themes.theme_path(deck["theme"])).read_text(encoding="utf-8")
    slides = deck["slides"]
    # **札は0から数える** ── めくる仕掛けが見るのは `LABELS[i]` で、i は0から始まる。
    # 先頭に空を足すと、全部の枚が1つ前の札を出す
    labels = json.dumps([s["label"] for s in slides], ensure_ascii=False)
    return _t("page", title=_esc(deck["title"]), theme_name=_esc(deck["theme"]),
              theme=theme.rstrip(), total=len(slides), labels=labels,
              slides="\n".join(slide(s) for s in slides))


def load(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def build_deck(source: pathlib.Path, out: pathlib.Path, *,
               check_only: bool = False) -> tuple[int, str]:
    """入力から1枚を書き出す。`check_only` なら、差が無いかだけを検査する。"""
    body = build(load(source))
    if check_only:
        same = out.exists() and out.read_text(encoding="utf-8") == body
        return (0 if same else 1), ("同一" if same else "差が在る")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(body, encoding="utf-8")
    return 0, f"{len(body)} 字"
