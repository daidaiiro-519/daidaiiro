# SPDX-License-Identifier: MIT
"""組み上がり（`topics.json`）の形を検査する。**散文を弾くのが目的である。**

**全ての升を検査する。** 一部の欄だけを見ると、検査していない欄へ散文が寄る。

    python3 validate_topics.py <topics.json>

道具が例外で止めるのは3つだけである ── 根拠の欠け ・ 札に無い出どころ ・ 案が1件。
**答えと結論は自由な文字列なので、散文が通る。** ここが型の契約を持つ。

検査は2段である。
  1 形    `references/topics.schema.json` に一致するか（jsonschema が在れば適用する）
  2 散文  1つの升に主張が複数入っていないか ・ 1文が長すぎないか

**2 は概念から導ける。** 情報の型が表現の形を決めるので、
主張が2つ以上並ぶ升は、箇条書きか表に割れる。
"""
from __future__ import annotations

import collections
import json
import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
SCHEMA = HERE.parent / "references" / "topics.schema.json"

SEP = "──"          # 主張と説明を分ける区切り
MAX_SENT = 120      # 1文の上限（字）
MAX_CLAIM = 1       # 1つの升に入れてよい「主張 ── 説明」の数

LIST = re.compile(r"<(?:ul|ol|table|li|tr)\b", re.I)
TAG = re.compile(r"<[^>]+>")


def _plain(x: str) -> str:
    return TAG.sub("", x)


QUOTE = re.compile(r"<blockquote\b.*?</blockquote>", re.S | re.I)


def prose(where: str, text: str) -> list[str]:
    """1つの升を検査する。**箇条書きか表に割ってあれば、区切りの数は数えない。**

    **引用は検査しない** ── 原文を書き換えてはならない（差し戻しの理由は利用者の言葉である）。
    """
    out = []
    if not isinstance(text, str) or not text.strip():
        return out
    text = QUOTE.sub("", text)
    if not text.strip():
        return out
    if not LIST.search(text):
        n = _plain(text).count(SEP)
        if n > MAX_CLAIM:
            out.append(f"{where}: 1つの升に区切り「{SEP}」が {n} 個ある ── "
                       f"主張が {n} 件同居している。箇条書きか表に割る")
    # 箇条書きの項目と段落も、文の境界として扱う ── 句点を持たない項目が1文に繋がる
    flat = re.sub(r"</(?:li|p|td|th|tr|h[1-6])>|<br\s*/?>", "。", text, flags=re.I)
    for s in re.split(r"[。］]", _plain(flat)):
        s = s.strip()
        if len(s) > MAX_SENT:
            out.append(f"{where}: 1文が {len(s)} 字ある（上限 {MAX_SENT}）── "
                       f"「{s[:30]}…」")
    return out


def by_schema(doc: dict) -> list[str]:
    try:
        import jsonschema
    except ImportError:
        return ["（jsonschema が無いので、形の検査は実施していない）"]
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    v = jsonschema.Draft202012Validator(schema)
    return [f"形: {'/'.join(map(str, e.path))} ── {e.message}"
            for e in sorted(v.iter_errors(doc), key=lambda e: list(e.path))]


def 升たち(t: dict):
    """1つの論点の中の**全ての升**を並べる。

    **一部の欄だけを検査すると、検査していない欄へ散文が寄る。**
    実際に、答えと結論だけを見ていたので、前提と扱わない範囲へ散文が残存した。
    """
    no = t.get("no")
    yield f"論点{no} の問い", t.get("question", "")
    yield f"論点{no} の答え", t.get("answer", "")
    yield f"論点{no} の前置き", t.get("note") or ""
    if t.get("pick"):
        yield f"論点{no} の結論", t["pick"][1]
    for i, o in enumerate(t.get("kept", [])):
        yield f"論点{no} の案[{o.get('name', i)}]", o.get("gist", "")
        yield f"論点{no} の案[{o.get('name', i)}]の代償", o.get("cost", "")
    for i, d in enumerate(t.get("dropped", [])):
        yield f"論点{no} の除外した案[{i}]", d[0]
        yield f"論点{no} の除外した案[{i}]の反証", d[1]
    for i, x in enumerate(t.get("found", [])):
        yield f"論点{no} の分かったこと[{i}]", x
    for i, x in enumerate(t.get("path", [])):
        yield f"論点{no} の道筋[{i}]", x
    for i, g in enumerate(t.get("grounds", [])):
        yield f"論点{no} の前提[{i}]の支える先", g[0]
        yield f"論点{no} の前提[{i}]のもとにしたこと", g[1]
    for i, x in enumerate(t.get("costs", [])):
        yield f"論点{no} の要求事項[{i}]", x
    for i, w in enumerate(t.get("weaknesses", [])):
        pair = w if isinstance(w, (list, tuple)) else [w]
        yield f"論点{no} の扱わない範囲[{i}]", pair[0]
        if len(pair) > 1:
            yield f"論点{no} の扱わない範囲[{i}]の扱い", pair[1]
    for i, d in enumerate(t.get("defects", [])):
        yield f"論点{no} の未修正の誤り[{i}]", d[0]
        yield f"論点{no} の未修正の誤り[{i}]の現状", d[1]
    for i, tb in enumerate(t.get("tables", [])):
        yield f"論点{no} の表[{i}]の題", tb.get("caption", "")
        for k, row in (tb.get("rows") or {}).items():
            for c, cell in enumerate(row):
                yield f"論点{no} の表[{i}]の升（{k} / {c}）", cell
    for i, dec in enumerate(t.get("decision", [])):
        yield f"論点{no} の決定[{i}]", dec[1]
    for i, ex in enumerate(t.get("extras", [])):
        # 畳んだ節は段落で割る ── 前の答えの記録は、段落1つが1つの升である
        for k, para in enumerate(re.split(r"</p>|</blockquote>", ex[1])):
            yield f"論点{no} の{_plain(ex[0])[:14]}[{k}]", para


記号 = re.compile(r"^(?:<b>)?([A-Z])(?:<sup>[^<]*</sup>)?\s*──")
LI = re.compile(r"<li>(.*?)</li>", re.S)


def ui(t: dict) -> list[str]:
    """**見せ方の欠陥を検出する。** どれも、実際に出してしまったものである。

      1 記号が二重に出る    —— 道具が記号を別に出すのに、本文の先頭でも書いた
      2 升の中が未整形      —— 箇条書きの外側に整形を当て、項目の中の区切りが残った
      3 整形が入れ子になった —— 割った直後に箇条書きが来て、主張が見出しに化けた
    """
    out, no = [], t.get("no")
    if t.get("pick"):
        letter, body = t["pick"][0], t["pick"][1]
        m = 記号.match(body.strip())
        if m and m.group(1) == letter:
            out.append(f"論点{no} の結論: 記号「{letter}」が本文の先頭にも在る ── "
                       "道具が別に出すので、二重に表示される")
        if body.strip().startswith('<b class="lead-s">') and "<ul>" in body:
            out.append(f"論点{no} の結論: 整形が箇条書きの外側に当たっている ── "
                       "項目1つずつに当てる")
    for where, text in 升たち(t):
        if not isinstance(text, str) or "<li>" not in text:
            continue
        for k, item in enumerate(LI.findall(text)):
            if 'class="lead-s"' in item:
                continue
            if _plain(QUOTE.sub("", item)).count(SEP) >= 1:
                out.append(f"{where} の項目[{k}]: 升の中に区切り「{SEP}」が残っている ── "
                           "箇条書きの外側ではなく、項目1つずつに整形を当てる")
    return out


VOID = {"br", "img", "hr", "input", "meta", "path", "circle", "rect", "line",
        "polygon", "use", "stop", "g", "svg", "text", "tspan", "defs", "marker"}


def 入れ子(where: str, x: str) -> list[str]:
    """開いた要素と閉じた要素の数を突き合わせる。**入れ子の壊れを検出する。**

    整形を重ねて当てると要素が入れ子になって壊れる ── 実際に2度発生した。
    以降の本文が太字に化けるので、読む側は強調の意味を取り違える。
    """
    if not isinstance(x, str) or "<" not in x:
        return []
    op = collections.Counter(t for t in re.findall(r"<(\w+)(?:\s[^>]*)?>", x)
                             if t not in VOID)
    cl = collections.Counter(t for t in re.findall(r"</(\w+)>", x) if t not in VOID)
    bad = {k: (op[k], cl[k]) for k in set(op) | set(cl) if op[k] != cl[k]}
    if not bad:
        return []
    detail = " ・ ".join(f"{k}: 開き {a} ／ 閉じ {b}" for k, (a, b) in bad.items())
    return [f"{where}: 要素の数が合わない（{detail}）── "
            "整形を重ねて当てていないかを確認する"]


def by_concept(doc: dict) -> list[str]:
    out = []
    for t in doc.get("topics", []):
        for where, text in 升たち(t):
            out += prose(where, text)
        # 入れ子は**丸ごとの欄**にだけ当てる ── 段落へ割った断片は、開きと閉じが揃わない
        no = t.get("no")
        out += 入れ子(f"論点{no} の答え", t.get("answer", ""))
        out += 入れ子(f"論点{no} の実例", t.get("example", ""))
        if t.get("pick"):
            out += 入れ子(f"論点{no} の結論", t["pick"][1])
        for i, ex in enumerate(t.get("extras", [])):
            out += 入れ子(f"論点{no} の畳んだ節[{i}]", ex[1])
        out += ui(t)
    return out


def main(argv: list[str]) -> int:
    path = pathlib.Path(argv[1])
    doc = json.loads(path.read_text(encoding="utf-8"))
    bad = by_schema(doc) + by_concept(doc)
    hard = [x for x in bad if not x.startswith("（")]
    for x in bad:
        print(("  △ " if x.startswith("（") else "  × ") + x)
    print(f"型の検査　{'通った' if not hard else f'通っていない（{len(hard)} 件）'}"
          f"　／　論点 {len(doc.get('topics', []))} 件")
    return 0 if not hard else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
