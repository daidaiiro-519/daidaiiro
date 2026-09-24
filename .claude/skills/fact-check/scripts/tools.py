# SPDX-License-Identifier: MIT
"""fact-check の道具の宣言。**能力の正本はここである。**

計算は `source.py` が持ち、この宣言は**呼び方だけ**を固定する ──
能力を2か所に書くと、片方だけが古くなる。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from lib import source  # noqa: E402
from contract import Arg, Tool, result  # noqa: E402



def fetch(url: str, dir: str = source.DEFAULT_DIR) -> dict:
    """原文を取得して保存する。"""
    path = source.fetch(url, dir)
    return result(ok=path is not None, findings=[] if path else [f"取得できなかった: {url}"],
                  path=path, url=url, dir=dir)


def _human_fetch(res: dict) -> str:
    d = res["data"]
    return f"取得した: {d['path']}" if res["ok"] else f"取得できなかった: {d['url']}"


def listing(dir: str = source.DEFAULT_DIR) -> dict:
    """取得したものを並べる。"""
    rows = source.listing(dir)
    return result(ok=True, items=[{"fetched_at": d, "lines": n, "url": u} for d, n, u in rows],
                  dir=dir)


def _human_listing(res: dict) -> str:
    return "\n".join(source.lst_lines(res["data"]["dir"]))


def verify(path: str, needles: list | str | None = None, as_: str = "",
           near: str | None = None, within: str | int = 40,
           from_: str | None = None) -> dict:
    """原文の文字列で照合する。**照合の種類を渡さなければ止まる。**

    照合するものは、並べて渡すか `--from` で渡す ──
    **引数どうしの関係は、契約ではなく道具が判定する。**
    """
    if not needles and not from_:
        return result(ok=False, findings=["照合するものを渡す"], path=path)
    if from_:
        needles = [x.strip() for x in open(from_, encoding="utf-8") if x.strip()]
    if isinstance(needles, str):
        needles = [needles]
    s = source.scan(path, list(needles), how=as_, near=near, within=int(within))
    findings = ([f"読めた範囲に無い: {getattr(x, 'needle', x)}" for x in s.missing]
                + [f"読めなかった: {name}　{why}" for name, why in s.unreadable])
    return result(ok=True, findings=findings, path=path, how=as_, near=near,
                  within=int(within),
                  hits={r.needle: [{"doc": h.doc, "line": h.line} for h in r.hits]
                        for r in s.results})


def _human_verify(res: dict) -> str:
    if not res["ok"]:
        return "\n".join(res["findings"])
    d = res["data"]
    s = source.scan(d["path"], list(d["hits"].keys()), how=d["how"], near=d["near"],
                    within=d["within"])
    return "\n".join(source.report_lines(d["path"], s, d["near"], d["within"]))


TOOLS = [
    Tool(name="fetch", summary="原文を取得して保存する",
         args=[Arg("url", "取得する先"), Arg("dir", "保存先", required=False)],
         run=fetch, human=_human_fetch),
    Tool(name="list", summary="取得したものを並べる",
         args=[Arg("dir", "保存先", required=False)], run=listing, human=_human_listing),
    Tool(name="verify", summary="原文の文字列で照合する",
         args=[Arg("path", "原文"), Arg("needle", "照合するもの（`--from` で渡してもよい）", required=False,
                   many=True, param="needles"),
               Arg("as", "照合の種類（identifier ／ quote ／ text）", required=False, param="as_"),
               Arg("near", "アンカー", required=False),
               Arg("within", "アンカーからの行数", required=False),
               Arg("from", "照合するものを並べたファイル", required=False, param="from_")],
         run=verify, human=_human_verify),
]
