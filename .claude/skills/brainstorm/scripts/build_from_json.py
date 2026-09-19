#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 daidaiiro
"""ブレストの中身（JSON）を読んで、ブレストボードを組む。

    python3 build_from_json.py <ブレストのフォルダ>

そのフォルダの `topics.json` を読み、`board.html` と `index.html` を書く。
図は同じフォルダの `figures/<名前>.svg` から読む。

**論点の中身は、ここに置かない。**それはブレストごとの持ち物なので、
そのフォルダが `topics.json` として持つ ── ここが持つのは読み方だけである。
中身を組み立てる手順を外（作業用の一時領域）に置くと、セッションが変わった
ときに失われ、成果物から復元することになる。実際にそうなった。
"""
from __future__ import annotations

import json
import pathlib
import sys

from build_board import Option, Table, Topic, deck, write


def _topic(d: dict, figdir: pathlib.Path) -> Topic:
    figs = [(( figdir / f"{name}.svg").read_text(encoding="utf-8"), cap)
            for name, cap in d.get("figures", [])]
    return Topic(
        no=d["no"], label=d["label"], status=d.get("status", "未"),
        question=d["question"], answer=d.get("answer", ""), note=d.get("note"),
        figures=figs,
        pick=tuple(d["pick"]) if d.get("pick") else None,
        decision=[tuple(x) for x in d.get("decision", [])],
        path=d.get("path", []),
        grounds=[tuple(x) for x in d.get("grounds", [])],
        kept=[Option(*x) for x in d.get("kept", [])],
        tables=[Table(caption=t["caption"], columns=t["columns"], rows=t["rows"],
                      lead=t.get("lead"), plain=t.get("plain", False))
                for t in d.get("tables", [])],
        dropped=[tuple(x) for x in d.get("dropped", [])],
        found=d.get("found", []),
        costs=d.get("costs", []),
        weaknesses=d.get("weaknesses", []),
        extras=[tuple(x) for x in d.get("extras", [])],
    )


def main(folder: str) -> int:
    here = pathlib.Path(folder)
    data = json.loads((here / "topics.json").read_text(encoding="utf-8"))
    topics = [_topic(d, here / "figures") for d in data["topics"]]
    html = deck(data["theme"], topics,
                intro=data.get("intro"),
                extras=[tuple(x) for x in data.get("extras", [])],
                board=data.get("board", here.name),
                round_no=data.get("round", 1))
    write(html, str(here / "board.html"), data["theme"])
    (here / "index.html").write_text((here / "board.html").read_text(encoding="utf-8"),
                                     encoding="utf-8")
    n_open = sum(1 for t in topics if t.status != "決着" and (t.pick or t.decision))
    print(f"論点 {len(topics)} 件 ／ 開いている {n_open} 件 ／ 図 {sum(len(t.figures) for t in topics)} 枚")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__)
        raise SystemExit(1)
    raise SystemExit(main(sys.argv[1]))
