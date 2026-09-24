# SPDX-License-Identifier: MIT
"""規則ファイルの形を検査する。**実行の前に、規則ファイルが契約を満たすかを確認する。**

規則を立てる条件は2つで、両方を満たすものだけを書く ──
無ければ外すか、コマンドで検査できるか。この側が見られるのは後者だけである。
"""
from __future__ import annotations

import json
import pathlib

from . import REFERENCES


def 検査する(規則ファイル: pathlib.Path) -> list[str]:
    検出: list[str] = []
    try:
        d = json.loads(規則ファイル.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        return [f"JSON として読めない ── {e}"]
    try:
        import jsonschema
    except ModuleNotFoundError:
        検出.append("jsonschema が無いので、形の検査を実行していない")
    else:
        形 = json.loads((REFERENCES / "rules.schema.json").read_text(encoding="utf-8"))
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


def 層を検査する(規則ファイル: pathlib.Path, 根: pathlib.Path) -> list[str]:
    """層の場所の規則が、実物と一致するかを見る。**存在だけを見る。**"""
    d = json.loads(規則ファイル.read_text(encoding="utf-8"))
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


def 概念を検査する(概念ファイル: pathlib.Path) -> list[str]:
    """概念の出典が契約を満たすかを見る。

    **原文は同梱しない。** したがってこの側が見られるのは、
    引用と、取り直すための4つの値（url ・ 日 ・ sha256 ・ 行）が揃っているかまでである。
    **引用が原文と一致するかは、取り直して照合する側が判定する。**
    """
    検出: list[str] = []
    try:
        d = json.loads(概念ファイル.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        return [f"JSON として読めない ── {e}"]
    try:
        import jsonschema
    except ModuleNotFoundError:
        検出.append("jsonschema が無いので、形の検査を実行していない")
    else:
        形 = json.loads((REFERENCES / "concepts.schema.json").read_text(encoding="utf-8"))
        v = jsonschema.Draft202012Validator(形)
        for e in sorted(v.iter_errors(d), key=lambda x: list(x.path)):
            検出.append("形: " + "/".join(map(str, e.path)) + " ── " + e.message)

    for i, c in enumerate(d.get("概念") or []):
        名 = c.get("概念") or f"{i}件目"
        出典 = c.get("出典") or {}
        if not 出典.get("引用"):
            検出.append(f"{名}: 引用が無い ── 原文を提示できない概念を、手順の根拠にしない")
        for 欄 in ("url", "日", "sha256", "行"):
            if not (出典.get("取得") or {}).get(欄):
                検出.append(f"{名}: 取得に {欄} が無い ── 同じ版かを、あとから判定できない")
    return 検出


def スキーマを検査する(スキーマ: pathlib.Path) -> list[str]:
    """スキーマ自身を実体として検証する。**案内の欠落を検出する。**

    案内は3つである ── `description`（概要）・ `x-prompt.read`（読み取り）・
    `x-prompt.write`（値を埋めるとき）。**持たせる深さは、最上位と `$defs` の各形の
    項目までである** ── 入れ子の奥の葉まで要求すると案内が肥大し、葉は親の案内が覆う。
    """
    検出: list[str] = []
    try:
        d = json.loads(スキーマ.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        return [f"JSON として読めない ── {e}"]
    try:
        import jsonschema
    except ModuleNotFoundError:
        return ["jsonschema が無いので、案内の検査を実行していない"]
    v = jsonschema.Draft202012Validator
    try:
        v.check_schema(d)
    except jsonschema.SchemaError as e:
        検出.append("スキーマとして無効 ── " + str(e).splitlines()[0])
    メタ = json.loads((REFERENCES / "schema-meta.schema.json").read_text(encoding="utf-8"))
    for e in sorted(v(メタ).iter_errors(d), key=lambda x: list(x.path)):
        検出.append("案内: " + "/".join(map(str, e.path)) + " ── " + e.message)
    return 検出
