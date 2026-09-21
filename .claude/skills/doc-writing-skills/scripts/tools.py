# SPDX-License-Identifier: MIT
"""doc-writing-skills の道具の宣言。**能力の正本はここである。**

計算は `gate.py` ・ `tails.py` ・ `fetch_sources.py` が持ち、この宣言は**呼び方だけ**を固定する。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from lib import fetch_sources  # noqa: E402
from lib import gate as _gate  # noqa: E402
from lib import tails as _tails  # noqa: E402
from contract import Arg, Tool, result  # noqa: E402



def check(paths: list | str, synonyms: str | None = None) -> dict:
    """10の判定を当てる（ゲート1）。**見つけるが、直さない。**"""
    if isinstance(paths, str):
        paths = [paths]
    if synonyms:
        _gate.SYNONYMS = [tuple(x.rstrip("\n").split("\t")[:2])
                          for x in open(synonyms, encoding="utf-8")
                          if x.strip() and not x.startswith("#") and "\t" in x]
    findings = [f"{pathlib.Path(a).name}:{f.line} [{f.basis}] {f.check}：{f.excerpt}"
                for a in paths for f in _gate.inspect(a)]
    return result(ok=True, findings=findings, paths=list(paths))


def _human_check(res: dict) -> str:
    return "\n".join(_gate.gate_lines(res["data"]["paths"])[0])


def checks() -> dict:
    """当てている判定の一覧を出す。"""
    return result(ok=True, checks=[c.__doc__ or c.__name__ for c in getattr(_gate, "CHECKS", [])])


def _human_checks(res: dict) -> str:
    _gate.print_checks()
    return ""


def tails(paths: list | str) -> dict:
    """述部の末尾を数える。**語彙の偏りは、数でしか見えない。**"""
    if isinstance(paths, str):
        paths = [paths]
    return result(ok=True, items=[{"tail": w, "count": n} for w, n in _tails.counted(paths)],
                  paths=list(paths))


def _human_tails(res: dict) -> str:
    return "\n".join(_tails.tails_lines(res["data"]["paths"]))


def sources(check: str = "") -> dict:
    """原文を取得する、または手元のものと目録の一致を検査する。"""
    code = fetch_sources.check() if check else fetch_sources.fetch()
    return result(ok=code == 0, findings=[] if code == 0 else ["目録と一致しない"], code=code)


def _human_sources(res: dict) -> str:
    return "一致している" if res["ok"] else "目録と一致しない"


TOOLS = [
    Tool(name="check", summary="10の判定を当てる（ゲート1）",
         args=[Arg("path", "対象のファイル", many=True, param="paths"),
               Arg("synonyms", "言い換えの一覧", required=False)],
         run=check, human=_human_check),
    Tool(name="checks", summary="当てている判定の一覧を出す", args=[],
         run=checks, human=_human_checks),
    Tool(name="tails", summary="述部の末尾を数える",
         args=[Arg("path", "対象のファイル", many=True, param="paths")],
         run=tails, human=_human_tails),
    Tool(name="sources", summary="原文を取得する／目録との一致を検査する",
         args=[Arg("check", "検査だけを行う", required=False)],
         run=sources, human=_human_sources),
]
