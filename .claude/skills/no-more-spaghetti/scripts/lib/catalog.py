# SPDX-License-Identifier: MIT
"""索引から規則ファイルを解決する。**索引は1か所である。**

規則ファイルを平らに並べると、成果物と言語が増えるたびに一覧が読めなくなる。
**名前から経路へ解決する層を1つ挟む** ── 呼ぶ側は名前だけを知り、置き場所の変更は
索引の中で閉じる。

    .coding-rules/index.json        唯一の索引
    .coding-rules/applied/…         このリポジトリで実行する規則（成果物ごと）
    .coding-rules/templates/…       他のリポジトリへ渡す雛形（ここでは実行しない）
    .coding-rules/sources/…         外を指す規則の原文
"""
from __future__ import annotations

import json
import pathlib

RULES_DIR = ".coding-rules"
APPLIED_DIR = "applied"
TEMPLATES_DIR = "templates"
INDEX_PATH = f"{RULES_DIR}/index.json"
"""区分は2つである。

    applied/     **索引に載り、このリポジトリで実行する**
    templates/   **索引に載せない** ── 他のリポジトリへ渡す雛形で、ここでは実行しない
"""


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
    """索引そのものを検査する。**名前の重複と、実在しない経路を見る。**"""
    findings: list[str] = []
    seen: set[str] = set()
    for e in load(root).get("entries", []):
        name = e.get("name", "")
        if name in seen:
            findings.append(f"名前が重複している: {name}")
        seen.add(name)
        if not (root / RULES_DIR / e.get("rules", "")).is_file():
            findings.append(f"規則ファイルが実在しない: {e.get('rules')} ── {name}")
    return findings
