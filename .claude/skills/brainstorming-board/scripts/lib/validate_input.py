# SPDX-License-Identifier: MIT
"""入力（board.json）を検査する。**組み上がりではなく、入力を検査する。**

    python3 scripts/cli.py validate <ボードのディレクトリ>

生成物を検査する形では、写しが正しく生成されたことしか判明しない。
しかも不合格の時点で HTML は既に出ている ── それはゲートではなく注記である。
**不合格なら HTML を1バイトも出さない。**

検査は4系統である ── 形（スキーマ）・散文・段の要素の混入・参照の解決。
"""
from __future__ import annotations

import json
import pathlib
import re
import sys

from . import REFERENCES as _REF
SCHEMA = _REF / "board.schema.json"

sep = "──"
longest = 120          # 1文の字数の上限
cohabit = 1            # 1つの升に置ける主張の数
quote = re.compile(r"「[^」]*」|<code>.*?</code>", re.S)
level_items = re.compile(r"</?(p|h[1-6]|ul|ol|li|pre|div|table|tr|td|th|details|summary|"
                    r"figure|blockquote)\b", re.I)


def _cells(d: dict):
    """検査する升を、場所の名前つきで全部列挙する。**取りこぼしを作らない。**"""
    def declare(bs, place):
        for i, b in enumerate(bs or []):
            k, p = b.get("kind"), f"{place}[{i}]{b.get('kind','')}"
            if k in ("para", "note", "heading", "card"):
                yield p, b.get("text") or b.get("heading", "")
            if k == "card":
                for j, e in enumerate(b.get("events", [])):
                    yield f"{p}/events[{j}]{e['tag']}", e["text"]
            if k == "list":
                for j, x in enumerate(b["items"]):
                    yield f"{p}/items[{j}]", x["text"]
                    yield from declare(x.get("nested"), f"{p}/items[{j}]")
            if k == "table":
                for a, vs in b["rows"]:
                    for v in vs:
                        yield f"{p}/{a}", str(v)
            if k == "grid":
                for r in b["rows"]:
                    for v in r:
                        yield f"{p}", str(v)
            if k == "previous":
                yield f"{p}/body", b["body"]
                yield f"{p}/why", b["why"]
            if k == "fold":
                yield from declare(b["body"], f"{p}/{b['heading']}")
            if k in ("para", "note"):
                yield from declare(b.get("nested"), p)

    yield from declare(d.get("intro"), "intro")
    for e in d.get("panels", []):
        yield from declare(e["body"], f"panels/{e['heading']}")
    for q in d.get("queue", []):
        yield f"queue/{q['no']}", q["why"]
    for t in d["topics"]:
        n = f"topics[{t['no']}]"
        yield f"{n}/answer", t["answer"]
        if t.get("intro"):
            yield f"{n}/intro", t["intro"]
        if t.get("decision"):
            yield from declare(t["decision"]["text"], f"{n}/decision")
        yield from declare(t.get("example"), f"{n}/example")
        yield from declare(t.get("path"), f"{n}/path")
        for i, o in enumerate(t.get("passed", [])):
            yield f"{n}/passed[{o['letter']}]", o["body"]
            yield f"{n}/passed[{o['letter']}]/cost", o["cost"]
        for i, o in enumerate(t.get("dropped", [])):
            yield f"{n}/dropped[{i}]", o["body"]
            yield f"{n}/dropped[{i}]/reason", o["reason"]
        for i, g in enumerate(t.get("grounds", [])):
            yield f"{n}/grounds[{i}]", g["basis"]
        for i, x in enumerate(t.get("requirements", [])):
            yield f"{n}/requirements[{i}]", x
        for i, x in enumerate(t.get("findings", [])):
            yield f"{n}/findings[{i}]", x
        for i, w in enumerate(t.get("out_of_scope", [])):
            yield f"{n}/out_of_scope[{i}]", w["item"]
            if w.get("treatment"):
                yield f"{n}/out_of_scope[{i}]/treatment", w["treatment"]
        for e in t.get("panels", []):
            yield from declare(e["body"], f"{n}/panels/{e['heading']}")


def prose(place: str, s: str) -> list[str]:
    """1つの升に2つのことが入っていないか、1文が長すぎないかを見る。

    **引用は除外する** ── 原文の形を変えないと決めているためである。
    **箇条書きごと除外してはならない** ── 以前の検査は箇条書きを含む升を
    丸ごと素通しにしていた。列挙はこの道具が升へ割ってから渡す。
    """
    bad = []
    raw = quote.sub("", re.sub(r"<[^>]+>", "", s))
    if raw.count(sep) > cohabit:
        bad.append(f"{place}: 1つの升に区切り「{sep}」が {raw.count(sep)} 個ある")
    for sentence in re.split(r"(?<=。)", raw):
        sentence = sentence.strip()
        if len(sentence) > longest:
            bad.append(f"{place}: 1文が {len(sentence)} 字ある（上限 {longest}）── {sentence[:34]}…")
    return bad


def level_mix(place: str, s: str) -> list[str]:
    """**入力は宣言だけを保持する。** 段の要素が入っていれば、宣言の外に構造がある。"""
    m = level_items.search(s)
    return [f"{place}: 段の要素「{m.group(0)}」が升の中に在る ── 宣言へ割る"] if m else []


def ref(dir: pathlib.Path, d: dict) -> list[str]:
    """図の参照先が実在するかを見る。"""
    bad, seen = [], set()
    def walk(x):
        if isinstance(x, dict):
            if x.get("kind") == "figures" and x["name"] not in seen:
                seen.add(x["name"])
                if not (dir / "figures" / f"{x['name']}.svg").exists():
                    bad.append(f"図 figures/{x['name']}.svg が無い")
            for v in x.values():
                walk(v)
        elif isinstance(x, list):
            for v in x:
                walk(v)
    walk(d)
    return bad


def shape(d: dict) -> list[str]:
    try:
        import jsonschema
    except ModuleNotFoundError:
        return ["jsonschema が無いので、形の検査を実行していない"]
    v = jsonschema.Draft202012Validator(json.loads(SCHEMA.read_text(encoding="utf-8")))
    return ["形: " + "/".join(map(str, e.path)) + " ── " + e.message
            for e in sorted(v.iter_errors(d), key=lambda x: list(x.path))]


def check(dir: pathlib.Path) -> list[str]:
    d = json.loads((dir / "board.json").read_text(encoding="utf-8"))
    bad = shape(d) + ref(dir, d)
    for place, s in _cells(d):
        bad += prose(place, s) + level_mix(place, s)
    return bad
