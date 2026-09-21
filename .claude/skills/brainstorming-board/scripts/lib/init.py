# SPDX-License-Identifier: MIT
# Copyright (c) 2026 daidaiiro
"""ブレストボードの置き場所を作る。

  python3 scripts/cli.py init <ブレストボードの名前> [--dir .brainstorming-board]

作るのは器だけである。**論点は書かない** ── 書くのは対話の側で、
この道具は置き場所と、入力の雛形（`board.json`）と、索引の行だけを用意する。

**ブレストボードの下に Python を置かない。** 正本は JSON であり、
組み立てるのは Skill 側の `render_board.py` である。

既に在る名前は作り直さない。上書きすると、書いた論点が消える。
"""
from __future__ import annotations

import argparse
import pathlib
import sys

BOARD = """{
 "$schema": "../../.claude/skills/brainstorming-board/references/board.schema.json",
 "title": "{title}",
 "board": "{name}",
 "round": 1,
 "topics": [
  {
   "no": 1,
   "name": "論点の短い名前",
   "status": "open",
   "question": "何を決めるか",
   "answer": "反証を通過して残った答え"
  }
 ]
}
"""

INDEX = """# ブレストボードの出どころ

**ブレストボードの正本は Artifact 側にある**。ここに置いてあるのは、その時点の複製である。
続きを進めるときは、先に Artifact を読んで、新しい版が出ていないかを確認する。

| ブレストボード | 題 | Artifact | 複製した時点 |
|---|---|---|---|
"""

SOURCES = """# 取得した原文の取り直し方

**原文の実体は置かない。** 置くのは `*.meta.json` だけで、url ・ 取得日 ・ `sha256` ・
バイト数 ・ 行数が入っている。取り直して `sha256` が一致すれば、照合した当時と同じ中身である。

```
python3 <source-fidelity>/scripts/source.py fetch <url> --dir .
```

PDF は本文を起こしてから照合する。ブレストボードの根拠が指す行番号は、起こした `.txt` の行番号である。

```
pdftotext -layout <名前>.pdf <名前>.txt
```
"""


def create(name: str, dir: str = ".brainstorming-board", title: str = "") -> list[str]:
    """置き場所・雛形・索引の行を作り、報告の行を返す。**印字はしない。**

    既に在る名前は作り直さない ── 上書きすると、書いた論点が消える。
    作り直さなかったことは、例外で伝える。
    """
    root = pathlib.Path(dir)
    board = root / name
    if board.exists():
        raise FileExistsError(
            f"既に在る: {board} ── 作り直さない。上書きすると、書いた論点が消える")

    title = title or name
    (board / "figures").mkdir(parents=True)
    (board / "rounds").mkdir()
    (board / "sources").mkdir()
    (board / "answers").mkdir()
    (board / "answers" / ".read").write_text("", encoding="utf-8")
    (board / "sources" / "README.md").write_text(SOURCES, encoding="utf-8")
    (board / "board.json").write_text(
        BOARD.replace("{title}", title).replace("{name}", name), encoding="utf-8")

    index = root / "README.md"
    if not index.exists():
        index.write_text(INDEX, encoding="utf-8")
    line = f"| `{name}/` | {title} | （未発行） | （未複製） |\n"
    if line not in index.read_text(encoding="utf-8"):
        with index.open("a", encoding="utf-8") as f:
            f.write(line)

    lines = [f"作った: {board}"]
    lines += ["  " + str(p.relative_to(root)) for p in sorted(board.rglob("*"))]
    lines += [f"索引へ1行足した: {index}", "",
              "次にすること ── board.json へ論点を書き、"
              "python3 scripts/cli.py render <この置き場所> で組む。",
              "図は design-svg に組ませ、返った SVG を figures/ へ置く。"]
    return lines


def main() -> int:
    ap = argparse.ArgumentParser(description="ブレストボードの置き場所を作る")
    ap.add_argument("name", help="ブレストボードの名前（英小文字とハイフン）")
    ap.add_argument("--dir", default=".brainstorming-board", help="置き場所の親")
    ap.add_argument("--title", default="", help="題。省くと名前をそのまま使う")
    a = ap.parse_args()
    try:
        lines = create(a.name, a.dir, a.title)
    except FileExistsError as e:
        print(e, file=sys.stderr)
        return 1
    for line in lines:
        print(line)
    return 0
