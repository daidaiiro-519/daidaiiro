# SPDX-License-Identifier: MIT
"""デザイントークンを読み、CSS のカスタムプロパティへ組む。

    python3 scripts/cli.py tokens            検査して、通れば CSS を出す
    python3 scripts/cli.py tokens --check 1  検査だけを実行する

**トークンの正本は1つである。** この道具が読むのは
`references/tokens.json` だけで、色も寸法もここから出る。
2か所に書くと、どちらが正しいかを毎回確認することになる。

**段を跨いだ参照を、形の段で弾く。** 意味の段は基礎の鍵だけを参照し、
部品の段は意味か基礎の鍵だけを参照する。散文の規定では破れる。
"""
from __future__ import annotations

import json
import pathlib
import sys

from . import REFERENCES as _REF
TOKENS = _REF / "tokens.json"
SCHEMA = _REF / "tokens.schema.json"

# 明暗の3つの選択子。**1つの表から生成する** ──
# 手で3か所へ書くと必ずずれる（実際に --shadow が [data-theme] の側から脱落していた）
_LIGHT_SELECTOR = ":root"
_DARK_SELECTOR = ('@media (prefers-color-scheme:dark){:root:not([data-theme="light"])',
              ':root[data-theme="dark"]')


def load(path: pathlib.Path | None = None) -> dict:
    return json.loads((path or TOKENS).read_text(encoding="utf-8"))


def _raw(t: dict) -> dict[str, str]:
    """基礎の段を、1つの辞書へ畳む。鍵の重複はそこで判明する。"""
    out: dict[str, str] = {}
    for kind, table in t["base"].items():
        for k, v in table.items():
            if k in out:
                raise ValueError(f"基礎の鍵が重複している: {k}（{kind}）")
            out[k] = v
    return out


def validate(t: dict, *, with_schema: bool = True) -> list[str]:
    """形と、段を跨ぐ参照を検査する。**0件になるものだけを検査する。**"""
    err: list[str] = []
    if with_schema:
        try:
            import jsonschema
        except ModuleNotFoundError:
            err.append("jsonschema が無いので、形の検査を実行していない")
        else:
            v = jsonschema.Draft202012Validator(
                json.loads(SCHEMA.read_text(encoding="utf-8")))
            for e in sorted(v.iter_errors(t), key=lambda x: list(x.path)):
                err.append("形: " + "/".join(map(str, e.path)) + " ── " + e.message)
    try:
        base = _raw(t)
    except ValueError as e:
        return err + [str(e)]

    light, dark = t["semantic"]["light"], t["semantic"]["dark"]
    for k in sorted(set(light) ^ set(dark)):
        side = "light" if k in light else "dark"
        err.append(f"意味の鍵 --{k} が {side} にしか無い ── 明暗で同じ鍵集合にする")
    for side, table in (("light", light), ("dark", dark)):
        for k, ref in table.items():
            if ref not in base:
                err.append(f"意味 {side} の --{k} が、基礎に無い鍵「{ref}」を参照している")
    for k, ref in t["component"].items():
        if ref.startswith("calc(") or ref[0].isdigit() or ref[0] == ".":
            continue
        if ref not in base and ref not in light:
            err.append(f"部品 --{k} が、意味にも基礎に無い鍵「{ref}」を参照している")
    return err


def _lines(table: dict[str, str], base: dict[str, str]) -> str:
    return "".join(f"--{k}:{base[v]};" for k, v in table.items())


# 寸法の系は、名前のまま CSS 変数へ出す ── 色は意味の段を経由するが、
# 寸法・字寸・字送り・角丸・枠は役割ではなく段そのものが意味である
_SIZE_SCALE = ("space", "font_size", "tracking", "radius", "border")


def css(t: dict) -> str:
    """3つの選択子と、寸法の系と、部品の段を組む。"""
    base = _raw(t)
    size = "".join(f"--{k}:{v};" for kind in _SIZE_SCALE for k, v in t["base"][kind].items())
    parts = []
    for k, ref in t["component"].items():
        value = ref if (ref.startswith("calc(") or ref[0].isdigit() or ref[0] == ".") \
            else f"var(--{ref})"
        parts.append(f"--{k}:{value};")
    light = _lines(t["semantic"]["light"], base) + size + "".join(parts)
    dark = _lines(t["semantic"]["dark"], base)
    return (f"{_LIGHT_SELECTOR}{{{light}}}"
            f"{_DARK_SELECTOR[0]}{{{dark}}}}}"
            f"{_DARK_SELECTOR[1]}{{{dark}}}")


def refs_and_defs(built: str) -> tuple[set[str], set[str]]:
    """生成物から、参照した鍵と定義した鍵を抜く。差を0件にするために使う。"""
    import re
    # 代替値を持つ参照（var(--x,#fff)）は、定義が無くても崩壊しない ── 別に数える
    ref = {m.group(1) for m in re.finditer(r"var\(\s*(--[a-z0-9-]+)\s*([,)])",
                                           built) if m.group(2) == ")"}
    defs = set(re.findall(r"(--[a-z0-9-]+)\s*:", built))
    return ref, defs
