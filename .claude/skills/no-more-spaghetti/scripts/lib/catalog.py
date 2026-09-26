# SPDX-License-Identifier: MIT
"""索引から規則ファイルを解決する。**索引は1か所である。**

規則ファイルを平らに並べると、成果物と言語が増えるたびに一覧が読めなくなる。
**名前から経路へ解決する層を1つ挟む** ── 呼ぶ側は名前だけを知り、置き場所の変更は
索引の中で閉じる。

    .coding-rules/index.json        唯一の索引
    .coding-rules/applied/…         実行する規則（成果物ごと）
    .coding-rules/sources/…         外を指す規則の原文
"""
from __future__ import annotations

import json
import pathlib

RULES_DIR = ".coding-rules"
APPLIED_DIR = "applied"
INDEX_PATH = f"{RULES_DIR}/index.json"


def load(root: pathlib.Path) -> dict:
    """索引を読む。**無ければ、経路を添えて断る。**"""
    path = root / INDEX_PATH
    if not path.exists():
        raise FileNotFoundError(
            f"索引が無い: {INDEX_PATH} ── init が作る（root と成果物の場所を渡す）")
    return json.loads(path.read_text(encoding="utf-8"))


def names(root: pathlib.Path) -> list[str]:
    """索引に在る名前を、並びのまま返す。"""
    return [e["name"] for e in load(root).get("entries", [])]


def resolve(root: pathlib.Path, name_or_path: str) -> pathlib.Path:
    """名前か経路から、規則ファイルの経路を得る。

    **経路をそのまま渡せる** ── 索引に載せる前の規則ファイルも検査できる。
    """
    direct = pathlib.Path(name_or_path)
    if direct.is_file():
        return direct
    if (root / name_or_path).is_file():
        return root / name_or_path
    for e in load(root).get("entries", []):
        if e["name"] == name_or_path:
            return root / RULES_DIR / e["rules"]
    raise FileNotFoundError(
        f"その名前は索引に無い: {name_or_path} ── 在るのは {' ・ '.join(names(root))}")


def check_index(root: pathlib.Path) -> list[str]:
    """索引そのものを検査する。

    **名前の重複 ・ 実在しない経路 ・ 解決しない `$schema` を見る** ── 置き場所を
    変えたとき、規則ファイルの中の相対の経路は一緒に動かない（実測 2026-09-26、
    applied/ へ移した11件が全部壊れた）。
    """
    findings: list[str] = []
    seen: set[str] = set()
    for e in load(root).get("entries", []):
        name = e.get("name", "")
        if name in seen:
            findings.append(f"名前が重複している: {name}")
        seen.add(name)
        path = root / RULES_DIR / e.get("rules", "")
        if not path.is_file():
            findings.append(f"規則ファイルが実在しない: {e.get('rules')} ── {name}")
            continue
        findings += check_schema_path(path)
    return findings


def check_schema_path(path: pathlib.Path) -> list[str]:
    """`$schema` の相対の経路が解決するかを見る。**外を指すものは対象にしない。**"""
    try:
        ref = json.loads(path.read_text(encoding="utf-8")).get("$schema", "")
    except (OSError, ValueError) as e:
        return [f"読めない: {path} ── {e}"]
    if not ref or ref.startswith("http"):
        return []
    if (path.parent / ref).exists():
        return []
    return [f"$schema が解決しない: {ref} ── {path}"]
