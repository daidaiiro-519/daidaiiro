# SPDX-License-Identifier: MIT
"""記録の形を検査する。**実行の前に、記録が契約を満たすかを見る。**

規則を立てる条件は2つで、両方を満たすものだけを書く ──
無ければ外すか、コマンドで検査できるか。この側が見られるのは後者だけである。
"""
from __future__ import annotations

import json
import pathlib

from . import REFERENCES


def 検査する(記録: pathlib.Path) -> list[str]:
    検出: list[str] = []
    try:
        d = json.loads(記録.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        return [f"JSON として読めない ── {e}"]
    try:
        import jsonschema
    except ModuleNotFoundError:
        検出.append("jsonschema が無いので、形の検査を実行していない")
    else:
        形 = json.loads((REFERENCES / "record.schema.json").read_text(encoding="utf-8"))
        v = jsonschema.Draft202012Validator(形)
        for e in sorted(v.iter_errors(d), key=lambda x: list(x.path)):
            検出.append("形: " + "/".join(map(str, e.path)) + " ── " + e.message)

    規則 = d["規則"] if isinstance(d.get("規則"), list) else [d]
    for i, r in enumerate(規則):
        名 = r.get("規則") or r.get("名前") or f"{i}件目"
        道具 = (r.get("検証方法") or {}).get("道具")
        if not 道具:
            検出.append(f"{名}: 検証方法に道具が無い ── コマンドで検査できない規則は立てない")
        elif not isinstance(道具, list):
            検出.append(f"{名}: 道具が配列ではない ── 殻を経由すると、展開が実行する殻に依存する")
        if not r.get("出典"):
            検出.append(f"{名}: 出典が無い ── 外（原典）か内（記録）かを書く")
    return 検出


def 層を検査する(記録: pathlib.Path, 根: pathlib.Path) -> list[str]:
    """層の場所の記録が、実物と一致するかを見る。**存在だけを見る。**"""
    d = json.loads(記録.read_text(encoding="utf-8"))
    層 = d.get("層")
    if not isinstance(層, dict):
        return []
    検出 = []
    for 名, 場所 in 層.items():
        if not (根 / 場所).exists():
            検出.append(f"層 {名} の場所が実在しない ── {場所}")
    並び = d.get("並び") or []
    足りない = [x for x in 並び if x not in 層]
    for x in 足りない:
        検出.append(f"並びの {x} が、層に無い")
    return 検出
