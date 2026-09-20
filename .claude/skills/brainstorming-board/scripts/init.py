# SPDX-License-Identifier: MIT
# Copyright (c) 2026 daidaiiro
"""ブレストボードの置き場所を作る。

  python3 scripts/init.py <ブレストボードの名前> [--dir .brainstorming-board]

作るのは器だけである。**論点は書かない** ── 書くのは対話の側で、
この道具は置き場所と、組む雛形と、索引の行だけを用意する。

既に在る名前は作り直さない。上書きすると、書いた論点が消える。
"""
from __future__ import annotations

import argparse
import pathlib
import sys

BOARD = '''# SPDX-License-Identifier: MIT
"""「{題}」のブレストボードを組む。

    python3 board.py     board.html と topics.json を書き出す

**図はここでは描かない。**描くのは design-svg であり、figures/ に残るのはその成果物である。
"""
from __future__ import annotations

import dataclasses
import json
import pathlib
import sys

_R = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(_R / ".claude" / "skills" / "brainstorming-board" / "scripts"))

from build_board import Option, Topic, deck, write  # noqa: E402

_FIG = pathlib.Path(__file__).resolve().parent / "figures"
_CAP: dict[str, str] = {{
    # "図の名前": "その図が何を示すかを1文で。読み上げに使われる",
}}


def fig(name: str) -> tuple[str, str]:
    """figures/ に在る SVG をそのまま読む。"""
    return (_FIG.joinpath(f"{{name}}.svg").read_text(encoding="utf-8"), _CAP[name])


t1 = Topic(
    no=1, label="{{論点の短い名前}}", status="新規",
    question="{{問いの全文}}",
    answer="{{現在地の一覧に出る、いまの答え}}",
    note="{{この論点の背景。無ければ省く}}",
    pick=("A", "<b>A ── {{結論}}</b>{{そう決めた内容}}"),
    kept=[
        Option("A", "{{案の中身}}", "{{この案を採ったときに負担すること}}"),
        Option("B", "{{案の中身}}", "{{同上}}"),
    ],
    dropped=[("{{除外した案}}", "{{何が壊れるか}}")],
    found=["{{反証で分かったこと}}"],
    path=["{{そう判断するまでの1手}}"],
    grounds=[("{{答えのどこを支えるか}}", "{{もとにしたこと}}",
              "前提", "{{その出どころ}}")],
    costs=["{{負担すること}}"],
    weaknesses=["{{まだ弱いところ}}"],
)

TOPICS = [t1]

body = deck(
    theme="{題}",
    topics=TOPICS,
    intro="{{このボードが何を決めるか。前提に置くもの}}",
    extras=[
        ("いま見る論点", "<p>{{何件あり、どれを確認し、残りは何を待つか}}</p>"),
        ("決まりの一覧（承認後に決まりとなるもの）", "<ol><li>{{決まり}}</li></ol>"),
        ("次にすること", "<ul><li>{{次の1手}}</li></ul>"),
        ("保留 ── 決めていないと分かっていること", "<ul><li>{{保留}}</li></ul>"),
    ],
    board="{名前}",
    round_no=1,
)

here = pathlib.Path(__file__).resolve().parent
write(body, str(here / "board.html"), "{題}")
(here / "topics.json").write_text(json.dumps({{
    "theme": "{題}", "board": "{名前}", "round": 1,
    "topics": [dataclasses.asdict(t) for t in TOPICS],
}}, ensure_ascii=False, indent=1), encoding="utf-8")
print("書き出し:", here / "board.html", "／", here / "topics.json")
'''

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


def main() -> int:
    ap = argparse.ArgumentParser(description="ブレストボードの置き場所を作る")
    ap.add_argument("name", help="ブレストボードの名前（英小文字とハイフン）")
    ap.add_argument("--dir", default=".brainstorming-board", help="置き場所の親")
    ap.add_argument("--title", default="", help="題。省くと名前をそのまま使う")
    a = ap.parse_args()

    root = pathlib.Path(a.dir)
    board = root / a.name
    if board.exists():
        print(f"既に在る: {board} ── 作り直さない。上書きすると、書いた論点が消える",
              file=sys.stderr)
        return 1

    title = a.title or a.name
    (board / "figures").mkdir(parents=True)
    (board / "sources").mkdir()
    (board / "answers").mkdir()
    (board / "answers" / ".read").write_text("", encoding="utf-8")
    (board / "sources" / "README.md").write_text(SOURCES, encoding="utf-8")
    (board / "board.py").write_text(
        BOARD.format(題=title, 名前=a.name), encoding="utf-8")

    index = root / "README.md"
    if not index.exists():
        index.write_text(INDEX, encoding="utf-8")
    line = f"| `{a.name}/` | {title} | （未発行） | （未複製） |\n"
    if line not in index.read_text(encoding="utf-8"):
        with index.open("a", encoding="utf-8") as f:
            f.write(line)

    print(f"作った: {board}")
    for p in sorted(board.rglob("*")):
        print("  " + str(p.relative_to(root)))
    print(f"索引へ1行足した: {index}")
    print()
    print("次にすること ── board.py の {…} を埋めて、python3 board.py で組む。")
    print("図は design-svg に組ませ、返った SVG を figures/ へ置く。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
