# SPDX-License-Identifier: MIT
"""規則ファイルの雛形を出す。**中身は呼ぶ側が書く。**

置き場所は成果物の `.coding/rules.json` である ── **この Skill の中に規則を置かない**。
層の名前と識別子は成果物ごとに違うので、ここへ置くと2つ目の成果物で衝突する。

**道具の名前を1つも持たない。** 雛形は契約（`rules.schema.json`）の形から組み、
何を入れるかは各項目の `x-prompt.write` が案内する。
"""
from __future__ import annotations

import json
import pathlib

from . import REFERENCES

RULES_PATH = ".coding/rules.json"
INNER_RULES = ("層の場所が、宣言した対応と一致する",
               "依存の向きが、内から外へ出ていない")
"""内を指す規則2件。**出典はモデルであり、原典を要さない**（論点4）。"""


def skeleton(layers: dict[str, str]) -> dict:
    """契約の形から雛形を組む。**項目の一覧を、この側に書かない。**"""
    schema = json.loads((REFERENCES / "rules.schema.json").read_text(encoding="utf-8"))
    shape = schema["$defs"]["rule"]["properties"]

    def rule(name: str) -> dict:
        out: dict = {}
        for key in shape:
            if key == "rule":
                out[key] = name
            elif key == "check":
                out[key] = {"tool": []}
            else:
                out[key] = "" if shape[key].get("type") == "string" else {}
        return out

    return {"$schema": "…/coding-skills/references/rules.schema.json",
            "order": list(layers), "layers": dict(layers),
            "rules": [rule(x) for x in INNER_RULES]}


def create(root: pathlib.Path, layers: dict[str, str]) -> pathlib.Path:
    """`.coding/rules.json` を置く。**既に在れば作り直さない** ── 書いた規則が消える。"""
    if not layers:
        raise ValueError("層を1つ以上渡す ── 層の無い規則ファイルは、何も検査できない")
    path = root / RULES_PATH
    if path.exists():
        raise FileExistsError(f"既に在る: {path} ── 作り直さない")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(skeleton(layers), ensure_ascii=False, indent=2) + "\n",
                    encoding="utf-8")
    return path


def parse_layers(pairs: list[str] | str) -> dict[str, str]:
    """`名前=識別子` の並びを読む。**識別子の形は問わない** ── 言語ごとに違う。"""
    items = [pairs] if isinstance(pairs, str) else list(pairs)
    out: dict[str, str] = {}
    for item in items:
        name, sep, ident = str(item).partition("=")
        if not sep or not name.strip() or not ident.strip():
            raise ValueError(f"層は 名前=識別子 で渡す ── {item}")
        out[name.strip()] = ident.strip()
    return out
