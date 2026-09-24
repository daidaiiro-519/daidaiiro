# SPDX-License-Identifier: MIT
"""規則ファイルの形を検査する。**実行の前に、規則ファイルが契約を満たすかを確認する。**

規則を立てる条件は2つで、両方を満たすものだけを書く ──
無ければ外すか、コマンドで検査できるか。この側が見られるのは後者だけである。
"""
from __future__ import annotations

import json
import pathlib

from . import REFERENCES


def check_rules(rules_file: pathlib.Path) -> list[str]:
    findings: list[str] = []
    try:
        d = json.loads(rules_file.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        return [f"JSON として読めない ── {e}"]
    try:
        import jsonschema
    except ModuleNotFoundError:
        findings.append("jsonschema が無いので、形の検査を実行していない")
    else:
        schema = json.loads((REFERENCES / "rules.schema.json").read_text(encoding="utf-8"))
        v = jsonschema.Draft202012Validator(schema)
        for e in sorted(v.iter_errors(d), key=lambda x: list(x.path)):
            findings.append("形: " + "/".join(map(str, e.path)) + " ── " + e.message)

    rules = d["rules"] if isinstance(d.get("rules"), list) else [d]
    for i, r in enumerate(rules):
        name = r.get("rule") or f"{i}件目"
        tool = (r.get("check") or {}).get("tool")
        if not tool:
            findings.append(f"{name}: 検証方法に道具が無い ── コマンドで検査できない規則は立てない")
        elif not isinstance(tool, list):
            findings.append(f"{name}: 道具が配列ではない ── 殻を経由すると、展開が実行する殻に依存する")
        if not r.get("source"):
            findings.append(f"{name}: 出典が無い ── 外（原典）か内（記録）かを書く")
    return findings


def check_layers(rules_file: pathlib.Path, root: pathlib.Path) -> list[str]:
    """層の宣言が、構造として成立するかを見る。

    **値をファイルの場所として検査しない** ── 層を識別する文字列は言語ごとに形が違う
    （モジュールパス ・ パッケージ ・ dotted path ・ crate 名）。値が正しいかは、
    **依存の向きの道具が実行できるかで判明する**。
    """
    d = json.loads(rules_file.read_text(encoding="utf-8"))
    layers = d.get("layers")
    if not isinstance(layers, dict):
        return []
    findings = []
    order = d.get("order") or []
    missing = [x for x in order if x not in layers]
    for x in missing:
        findings.append(f"並びの {x} が、層に無い")
    return findings


def check_concepts(concepts_file: pathlib.Path) -> list[str]:
    """概念の出典が契約を満たすかを見る。

    **原文は同梱しない。** したがってこの側が見られるのは、
    引用と、取り直すための4つの値（url ・ 日 ・ sha256 ・ 行）が揃っているかまでである。
    **引用が原文と一致するかは、取り直して照合する側が判定する。**
    """
    findings: list[str] = []
    try:
        d = json.loads(concepts_file.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        return [f"JSON として読めない ── {e}"]
    try:
        import jsonschema
    except ModuleNotFoundError:
        findings.append("jsonschema が無いので、形の検査を実行していない")
    else:
        schema = json.loads((REFERENCES / "concepts.schema.json").read_text(encoding="utf-8"))
        v = jsonschema.Draft202012Validator(schema)
        for e in sorted(v.iter_errors(d), key=lambda x: list(x.path)):
            findings.append("形: " + "/".join(map(str, e.path)) + " ── " + e.message)

    for i, c in enumerate(d.get("concepts") or []):
        name = c.get("concept") or f"{i}件目"
        source = c.get("source") or {}
        if not source.get("quote"):
            findings.append(f"{name}: 引用が無い ── 原文を提示できない概念を、手順の根拠にしない")
        for field in ("url", "date", "sha256", "line"):
            if not (source.get("fetched") or {}).get(field):
                findings.append(f"{name}: 取得に {field} が無い ── 同じ版かを、あとから判定できない")
    return findings


def check_schema(schema_path: pathlib.Path) -> list[str]:
    """スキーマ自身を実体として検証する。**案内の欠落を検出する。**

    案内は3つである ── `description`（概要）・ `x-prompt.read`（読み取り）・
    `x-prompt.write`（値を埋めるとき）。**持たせる深さは、最上位と `$defs` の各形の
    項目までである** ── 入れ子の奥の葉まで要求すると案内が肥大し、葉は親の案内が覆う。
    """
    findings: list[str] = []
    try:
        d = json.loads(schema_path.read_text(encoding="utf-8"))
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
        findings.append("スキーマとして無効 ── " + str(e).splitlines()[0])
    meta = json.loads((REFERENCES / "schema-meta.schema.json").read_text(encoding="utf-8"))
    for e in sorted(v(meta).iter_errors(d), key=lambda x: list(x.path)):
        findings.append("案内: " + "/".join(map(str, e.path)) + " ── " + e.message)
    return findings
