# SPDX-License-Identifier: MIT
"""部品が外側を参照していないかを、構文木で検査する。

    python3 .coding-rules/tools/inward.py <部品のディレクトリ> <禁じる名前>...

**この道具はリポジトリが持つ。** Skill の側は、規則に書いたコマンドを実行するだけで、
どの言語かを認知しない ── 言語ごとに検査の形が違うので、Skill へ置くと言語が混ざる。

**名前の解決を実行しない。** 見るのは、その位置に書かれた最上位の名前だけである ──
禁じる名前は同じディレクトリの最上位の名前なので、解決が要らない。相対の書き方
（`from . import x`）は同じ層の中なので、違反として数えない。

**文字列で読み込む箇所は、別の検出として返す** ── 構文木に依存の辺が現れないので、
そこを通れば向きの検査を素通りできる。
"""
from __future__ import annotations

import ast
import pathlib
import sys

DYNAMIC = ("import_module", "__import__")
"""文字列でモジュールを読み込む呼び方。**辺が構文木に現れない。**"""


def _head(name: str) -> str:
    return (name or "").split(".")[0]


def outward(parts: pathlib.Path, forbidden: tuple[str, ...]) -> list[str]:
    """部品から外側への参照を並べる。**見つけるが、直さない。**"""
    found: list[str] = []
    for f in sorted(parts.rglob("*.py")):
        try:
            tree = ast.parse(f.read_text(encoding="utf-8"))
        except SyntaxError as e:
            found.append(f"{f}:{e.lineno} 構文として読めない")
            continue
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                found += [f"{f}:{node.lineno} 外側を参照している ── import {a.name}"
                          for a in node.names if _head(a.name) in forbidden]
            elif isinstance(node, ast.ImportFrom):
                # 相対の書き方は同じ層の中である ── level が0のものだけを見る
                if node.level == 0 and _head(node.module) in forbidden:
                    found.append(f"{f}:{node.lineno} 外側を参照している ── "
                                 f"from {node.module}")
    return found


def dynamic(parts: pathlib.Path) -> list[str]:
    """文字列でモジュールを読み込む箇所を並べる。"""
    found: list[str] = []
    for f in sorted(parts.rglob("*.py")):
        try:
            tree = ast.parse(f.read_text(encoding="utf-8"))
        except SyntaxError:
            continue
        for node in ast.walk(tree):
            if isinstance(node, ast.Call):
                name = ast.unparse(node.func)
                if name.split(".")[-1] in DYNAMIC:
                    found.append(f"{f}:{node.lineno} 文字列で読み込んでいる ── {name}")
    return found


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2
    parts = pathlib.Path(argv[0])
    if not parts.is_dir():
        print(f"部品のディレクトリが無い ── {parts}", file=sys.stderr)
        return 2
    findings = outward(parts, tuple(argv[1:])) + dynamic(parts)
    for x in findings:
        print(x)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
