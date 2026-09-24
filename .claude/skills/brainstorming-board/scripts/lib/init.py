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
 "intro": [
  {
   "kind": "para",
   "text": "この板が何を決めるかを1文で書く。無ければこの欄ごと消す"
  }
 ],
 "panels": [
  {
   "heading": "いま見る論点",
   "body": [
    {
     "kind": "para",
     "text": "なぜこの論点をいま開いたかを書く"
    }
   ]
  }
 ],
 "queue": [
  {
   "no": 1,
   "why": "前提が片付いたので開いた ── 何が確定したかを書く"
  }
 ],
 "topics": [
  {
   "no": 1,
   "name": "論点の短い名前",
   "status": "open",
   "question": "何を決めるか",
   "answer": "反証を通過して残った答え",
   "intro": "この論点が何を縛るかを書く。**読み手が最初に読む1行である**",
   "decision": {
    "letter": "A",
    "text": [
     {
      "kind": "para",
      "text": "決めたことを書く。**手段ではなく、何を採るかである**"
     },
     {
      "kind": "list",
      "ordered": true,
      "items": [
       {
        "text": "決めたことの1つ目"
       },
       {
        "text": "決めたことの2つ目"
       }
      ]
     }
    ]
   },
   "example": [
    {
     "kind": "para",
     "text": "決めた語を導入したら、その実例をここに置く"
    }
   ],
   "figures": [
    {
     "kind": "figure",
     "name": "example",
     "caption": "この図が何を示すかを1文で書く。**答えの直下に、開いたまま置く**"
    }
   ],
   "passed": [
    {
     "name": "残った案の名前",
     "body": "何をする案か",
     "cost": "採ると何を負担するか"
    },
    {
     "name": "もう1つの案",
     "body": "何をする案か",
     "cost": "採ると何を負担するか"
    }
   ],
   "tables": [
    {
     "caption": "案ごとの帰結を並べる表",
     "cols": [
      "どうなるか"
     ],
     "rows": [
      [
       "残った案の名前",
       [
        "この案を採ったときに起きること"
       ]
      ]
     ]
    }
   ],
   "dropped": [
    {
     "body": "除外した案",
     "reason": "何が壊れるか"
    }
   ],
   "path": [
    {
     "kind": "card",
     "letter": "A",
     "heading": "前の答え。差し戻されたら、ここへ積む",
     "events": [
      {
       "tag": "returned",
       "text": "利用者の言葉を、そのまま置く"
      },
      {
       "tag": "obsolete",
       "text": "その差し戻しで、何が失効したか"
      },
      {
       "tag": "finding",
       "text": "そこで何が判明したか"
      }
     ]
    },
    {
     "kind": "para",
     "text": "差し戻しを伴わない手は、文として置く"
    }
   ],
   "findings": [
    "反証で判明したこと。**1行に1つだけ置く**"
   ],
   "grounds": [
    {
     "supports": "答えのどこを支えるか",
     "basis": "もとにしたこと",
     "tag": "primary",
     "source": "その出どころ"
    }
   ],
   "requirements": [
    "この答えを採ると、何を用意することになるか"
   ],
   "out_of_scope": [
    {
     "item": "この答えが扱わない事項",
     "treatment": "later",
     "note": "いつ、どこで決めるか"
    }
   ],
   "panels": [
    {
     "heading": "補足",
     "body": [
      {
       "kind": "para",
       "text": "論点ごとの補足を置く。無ければこの欄ごと消す"
      }
     ]
    }
   ]
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
python3 <fact-check>/scripts/source.py fetch <url> --dir .
```

PDF は本文を起こしてから照合する。ブレストボードの根拠が指す行番号は、起こした `.txt` の行番号である。

```
pdftotext -layout <名前>.pdf <名前>.txt
```
"""


# 雛形の図。**組むのはこの道具ではない** ── 置き換えるまでの場所取りである。
FIGURE = ('<svg viewBox="0 0 320 96" role="img">'
          '<rect x="1" y="1" width="318" height="94" rx="8" fill="var(--card)" '
          'stroke="var(--rule)"/>'
          '<text x="160" y="52" text-anchor="middle" font-size="13" fill="var(--dim)">'
          '図は design-svg に組ませ、この場所へ置く</text></svg>\n')


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
    # 完成イメージの置き場所を、見本ごと作る ── 図は design-svg に組ませ、ここへ置く
    (board / "figures" / "example.svg").write_text(FIGURE, encoding="utf-8")
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
