# SPDX-License-Identifier: MIT
"""入力（acdr.json）を検査する。**組み上がりではなく、入力を検査する。**

    python3 cli.py validate <記録のフォルダ>

生成物を検査する形では、複製が正しく生成されたことしか判明しない。
しかも不合格の時点で HTML は既に出ている ── それはゲートではなく注記である。
**不合格なら HTML を1バイトも出さない。**

検査は4系統である ── 形（スキーマ）・欄の完備（節と3つ組）・散文・参照の解決。
"""
from __future__ import annotations

import json
import pathlib
import re

from . import REFERENCES as _REF
SCHEMA = _REF / "acdr.schema.json"

# 承認の状態。**これ以外を書かせない** ── 状態が自由文になると、
# 承認を得ているかを読む側が判定することになる
STATUS = ("proposed", "accepted", "superseded")

# 節。**欠けたら止まる** ── 欠けた記録は、あとから誰も補完できない
SECTIONS = ("decision", "why", "applies_to")


def shape(spec: dict) -> list[str]:
    """形を検査する。**jsonschema が無い環境では、検査していないと報告する。**"""
    try:
        import jsonschema
    except ModuleNotFoundError:
        return ["jsonschema が無いので、形の検査を実行していない"]
    v = jsonschema.Draft202012Validator(json.loads(SCHEMA.read_text(encoding="utf-8")))
    return ["形: " + "/".join(map(str, e.path)) + " ── " + e.message
            for e in sorted(v.iter_errors(spec), key=lambda x: list(x.path))]


def fields(spec: dict) -> list[str]:
    """欄の完備を検査する。**3つ組が完備していない変更を、記録に包含しない。**"""
    bad: list[str] = []
    for k in ("no", "title", "date", "status", *SECTIONS):
        if not str(spec.get(k, "")).strip():
            bad.append(f"必須の欄が欠けている: {k}")
    if spec.get("status") and spec["status"] not in STATUS:
        bad.append(f"状態が「{spec['status']}」。使えるのは {'／'.join(STATUS)} である")
    for i, r in enumerate(spec.get("shift", [])):
        for k in ("what", "from", "to"):
            if k not in r:
                bad.append(f"shift[{i}] に {k} が無い")
    for i, r in enumerate(spec.get("alternatives", [])):
        for k in ("option", "why_not"):
            if k not in r:
                bad.append(f"alternatives[{i}] に {k} が無い")
    for d in spec.get("docs", []):
        for k in ("key", "tab", "file"):
            if k not in d:
                bad.append(f"docs の項目に {k} が無い")
        for i, m in enumerate(d.get("marks", [])):
            for k in ("find", "before", "why"):
                if not str(m.get(k, "")).strip():
                    bad.append(f"{d.get('key', '?')} の変更[{i}] に {k} が無い ── "
                               "3つ組が完備していない変更は、記録に含めない")
    return bad


sep = "──"
longest = 120          # 1文の字数の上限
cohabit = 1            # 1つの升に置ける主張の数
quote = re.compile(r"「[^」]*」|<code>.*?</code>", re.S)
level_items = re.compile(r"</?(p|h[1-6]|ul|ol|li|pre|div|table|tr|td|th|details|summary|"
                         r"figure|blockquote)\b", re.I)


def _cells(spec: dict):
    """検査する升を、場所の名前つきで全部列挙する。**取りこぼしを作らない。**

    **原文は列挙しない** ── 変更前（`before`）は原典であり、形を変えない。
    照合する文字列（`find`）も、散文ではない。
    """
    for k in ("decision", "why", "how", "applies_to", "supersedes", "figure_caption"):
        if spec.get(k):
            yield k, str(spec[k])
    for i, r in enumerate(spec.get("shift", [])):
        for k in ("what", "from", "to"):
            yield f"shift[{i}]/{k}", str(r.get(k, ""))
    for i, r in enumerate(spec.get("alternatives", [])):
        for k in ("option", "why_not"):
            if r.get(k):
                yield f"alternatives[{i}]/{k}", str(r[k])
    for i, x in enumerate(spec.get("after_approval", [])):
        yield f"after_approval[{i}]", str(x)
    for d in spec.get("docs", []):
        for i, m in enumerate(d.get("marks", [])):
            yield f"{d.get('key', '?')} の変更[{i}]/why", str(m.get("why", ""))


def prose(place: str, s: str) -> list[str]:
    """1つの升に2つのことが入っていないか、1文が長すぎないかを見る。

    **引用は除外する** ── 原文の形を変えないと決めているためである。
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


def refs(folder: pathlib.Path, spec: dict, root: pathlib.Path | None = None) -> list[str]:
    """参照先が実在するかを検査する。図と、対象の文書である。"""
    bad = []
    fig = spec.get("figure")
    if fig and not (folder / fig).exists():
        bad.append(f"図が無い: {folder / fig} ── design-svg に組ませて置くか、"
                   '"figure" の欄を削除する')
    for d in spec.get("docs", []):
        path = pathlib.Path(d["file"]) if root is None else root / d["file"]
        if not path.exists():
            bad.append(f"対象の文書が無い: {path}")
    return bad


def check(folder: pathlib.Path, root: pathlib.Path | None = None) -> list[str]:
    """記録のフォルダ1つを検査する。**呼ぶ側は、0件のときだけ組む。**"""
    spec = json.loads((folder / "acdr.json").read_text(encoding="utf-8"))
    bad = fields(spec) + shape(spec)
    for place, cell in _cells(spec):
        bad += prose(place, cell) + level_mix(place, cell)
    return bad + refs(folder, spec, root)
