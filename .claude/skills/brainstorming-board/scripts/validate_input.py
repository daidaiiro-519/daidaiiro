# SPDX-License-Identifier: MIT
"""入力（board.json）を検査する。**組み上がりではなく、入力を検査する。**

    python3 validate_input.py <ボードのディレクトリ>

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

_REF = pathlib.Path(__file__).resolve().parent.parent / "references"
SCHEMA = _REF / "board.schema.json"

区切り = "──"
最長 = 120          # 1文の字数の上限
同居 = 1            # 1つの升に置ける主張の数
引用 = re.compile(r"「[^」]*」|<code>.*?</code>", re.S)
段の要素 = re.compile(r"</?(p|h[1-6]|ul|ol|li|pre|div|table|tr|td|th|details|summary|"
                    r"figure|blockquote)\b", re.I)


def _升たち(d: dict):
    """検査する升を、場所の名前つきで全部列挙する。**取りこぼしを作らない。**"""
    def 宣言(bs, 場所):
        for i, b in enumerate(bs or []):
            k, p = b.get("種類"), f"{場所}[{i}]{b.get('種類','')}"
            if k in ("文", "注記", "見出し", "カード"):
                yield p, b.get("本文") or b.get("見出し", "")
            if k == "カード":
                for j, e in enumerate(b.get("出来事", [])):
                    yield f"{p}/出来事[{j}]{e['札']}", e["本文"]
            if k == "箇条":
                for j, x in enumerate(b["項目"]):
                    yield f"{p}/項[{j}]", x["本文"]
                    yield from 宣言(x.get("入れ子"), f"{p}/項[{j}]")
            if k == "表":
                for a, vs in b["行"]:
                    for v in vs:
                        yield f"{p}/{a}", str(v)
            if k == "卓":
                for r in b["行"]:
                    for v in r:
                        yield f"{p}", str(v)
            if k == "前の答え":
                yield f"{p}/中身", b["中身"]
                yield f"{p}/なぜ", b["なぜ"]
            if k == "開閉":
                yield from 宣言(b["中身"], f"{p}/{b['見出し']}")
            if k in ("文", "注記"):
                yield from 宣言(b.get("入れ子"), p)

    yield from 宣言(d.get("前書き"), "前書き")
    for e in d.get("面", []):
        yield from 宣言(e["中身"], f"面/{e['見出し']}")
    for q in d.get("行列", []):
        yield f"行列/{q['番号']}", q["なぜ"]
    for t in d["論点"]:
        n = f"論点{t['番号']}"
        yield f"{n}/答え", t["答え"]
        if t.get("前書き"):
            yield f"{n}/前書き", t["前書き"]
        if t.get("決定"):
            yield from 宣言(t["決定"]["本文"], f"{n}/決定")
        yield from 宣言(t.get("実例"), f"{n}/実例")
        yield from 宣言(t.get("道筋"), f"{n}/道筋")
        for i, o in enumerate(t.get("通過", [])):
            yield f"{n}/通過[{o['記号']}]", o["中身"]
            yield f"{n}/通過[{o['記号']}]代償", o["代償"]
        for i, o in enumerate(t.get("除外", [])):
            yield f"{n}/除外[{i}]", o["中身"]
            yield f"{n}/除外[{i}]理由", o["理由"]
        for i, g in enumerate(t.get("前提", [])):
            yield f"{n}/前提[{i}]", g["もと"]
        for i, x in enumerate(t.get("要求", [])):
            yield f"{n}/要求[{i}]", x
        for i, x in enumerate(t.get("判明事項", [])):
            yield f"{n}/判明事項[{i}]", x
        for i, w in enumerate(t.get("扱わない", [])):
            yield f"{n}/扱わない[{i}]", w["事項"]
            if w.get("扱い"):
                yield f"{n}/扱わない[{i}]扱い", w["扱い"]
        for e in t.get("面", []):
            yield from 宣言(e["中身"], f"{n}/面/{e['見出し']}")


def 散文(場所: str, s: str) -> list[str]:
    """1つの升に2つのことが入っていないか、1文が長すぎないかを見る。

    **引用は除外する** ── 原文の形を変えないと決めているためである。
    **箇条書きごと除外してはならない** ── 以前の検査は箇条書きを含む升を
    丸ごと素通しにしていた。列挙はこの道具が升へ割ってから渡す。
    """
    悪 = []
    素 = 引用.sub("", re.sub(r"<[^>]+>", "", s))
    if 素.count(区切り) > 同居:
        悪.append(f"{場所}: 1つの升に区切り「{区切り}」が {素.count(区切り)} 個ある")
    for 文 in re.split(r"(?<=。)", 素):
        文 = 文.strip()
        if len(文) > 最長:
            悪.append(f"{場所}: 1文が {len(文)} 字ある（上限 {最長}）── {文[:34]}…")
    return 悪


def 段の混入(場所: str, s: str) -> list[str]:
    """**入力は宣言だけを保持する。** 段の要素が入っていれば、宣言の外に構造がある。"""
    m = 段の要素.search(s)
    return [f"{場所}: 段の要素「{m.group(0)}」が升の中に在る ── 宣言へ割る"] if m else []


def 参照(dir: pathlib.Path, d: dict) -> list[str]:
    """図の参照先が実在するかを見る。"""
    悪, 見た = [], set()
    def 巡る(x):
        if isinstance(x, dict):
            if x.get("種類") == "図" and x["名前"] not in 見た:
                見た.add(x["名前"])
                if not (dir / "figures" / f"{x['名前']}.svg").exists():
                    悪.append(f"図 figures/{x['名前']}.svg が無い")
            for v in x.values():
                巡る(v)
        elif isinstance(x, list):
            for v in x:
                巡る(v)
    巡る(d)
    return 悪


def 形(d: dict) -> list[str]:
    try:
        import jsonschema
    except ModuleNotFoundError:
        return ["jsonschema が無いので、形の検査を実行していない"]
    v = jsonschema.Draft202012Validator(json.loads(SCHEMA.read_text(encoding="utf-8")))
    return ["形: " + "/".join(map(str, e.path)) + " ── " + e.message
            for e in sorted(v.iter_errors(d), key=lambda x: list(x.path))]


def check(dir: pathlib.Path) -> list[str]:
    d = json.loads((dir / "board.json").read_text(encoding="utf-8"))
    悪 = 形(d) + 参照(dir, d)
    for 場所, s in _升たち(d):
        悪 += 散文(場所, s) + 段の混入(場所, s)
    return 悪


def main() -> int:
    dir = pathlib.Path(sys.argv[1]).resolve()
    悪 = check(dir)
    for e in 悪:
        print("  ×", e, file=sys.stderr)
    print("入力の検査　通った" if not 悪 else
          f"入力の検査　通っていない（{len(悪)} 件）",
          file=sys.stdout if not 悪 else sys.stderr)
    return 1 if 悪 else 0


if __name__ == "__main__":
    sys.exit(main())
