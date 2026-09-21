# SPDX-License-Identifier: MIT
"""acdr の道具の宣言。**能力の正本はここである。**

計算は `lib/` が保持し、
この宣言は**呼び方だけ**を固定する。CLI も MCP もここから組む ──
能力を2回記述すると、片方だけが古くなる。

**部品は宣言に載せない** ── `lib/` に在るものは読み込まれるものであり、
入口を保持しない。**入口は `cli.py` の1つだけである。**
"""
from __future__ import annotations

import io
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from lib import render_acdr as _render  # noqa: E402
from lib import tokens as _tokens  # noqa: E402
from lib import validate_input as _validate  # noqa: E402
from contract import Arg, Tool, result  # noqa: E402

def _capture(fn, *a, **kw) -> tuple[int, str, str]:
    """印字する道具から、行と終了コードを取り出す。

    **道具の側を書き換えずに包む** ── 互換の入口の振る舞いを保つためである。
    """
    out, err = io.StringIO(), io.StringIO()
    keep = sys.stdout, sys.stderr
    sys.stdout, sys.stderr = out, err
    try:
        code = fn(*a, **kw)
    finally:
        sys.stdout, sys.stderr = keep
    return code, out.getvalue(), err.getvalue()


def new(record: str, title: str = "題を記入する") -> dict:
    """雛形から記録のフォルダを起こす。**同じ名前が在れば起こさない。**"""
    folder = pathlib.Path(record)
    try:
        code, out, err = _capture(_render.new, folder, title)
    except SystemExit as e:
        return result(ok=True, findings=[str(e)], record=record)
    return result(ok=True, record=record, path=str(folder / "acdr.json"),
                  lines=(out + err).strip().splitlines())


def _human_new(res: dict) -> str:
    return "\n".join(res["data"].get("lines") or res["findings"])


def validate(record: str) -> dict:
    """入力（acdr.json）を検査する。**生成物ではなく、入力を検査する。**"""
    folder = pathlib.Path(record).resolve()
    bad = _validate.check(folder, _render.repo_root(folder))
    return result(ok=True, findings=list(bad), record=record)


def _human_validate(res: dict) -> str:
    bad = res["findings"]
    return "\n".join(["  × " + e for e in bad]
                     + [f"入力の検査　通っていない（{len(bad)} 件）" if bad
                        else "入力の検査　通った"])


def render(record: str, check: str = "", force: str = "") -> dict:
    """acdr.json から1枚を組む。`check` を渡すと、差が無いかだけを検査する。"""
    code, out, err = _capture(_render.build_record, pathlib.Path(record),
                              check_only=bool(check), force=bool(force))
    lines = (out + err).strip().splitlines()
    # **通過の行は検出ではない。** 検出は、検査に落ちた行 ・ 差が在る行 ・ 拒否した行である
    bad = [x.strip()[4:].strip() for x in lines if x.strip().startswith("NG")]
    bad += [x.strip() for x in lines if "差が在る" in x or "拒否" in x]
    return result(ok=True, findings=bad, record=record, code=code, lines=lines,
                  path=str(pathlib.Path(record) / "index.html"))


def _human_render(res: dict) -> str:
    return "\n".join(res["data"]["lines"])


def tokens(check: str = "") -> dict:
    """トークンを検査し、CSS を出す。**直値を書かず、トークンを参照する。**"""
    t = _tokens.load()
    bad = _tokens.validate(t)
    if check:
        return result(ok=not bad, findings=list(bad), checked=True,
                      base=len(_tokens._raw(t)), meaning=len(t["semantic"]["light"]),
                      parts=len(t["component"]))
    return result(ok=not bad, findings=list(bad), css=_tokens.css(t) if not bad else "")


def _human_tokens(res: dict) -> str:
    d = res["data"]
    if res["findings"]:
        return "\n".join(["  × " + e for e in res["findings"]]
                         + [f"トークンの検査　通っていない（{len(res['findings'])} 件）"])
    if d.get("checked"):
        return (f"トークンの検査　通った　／　基礎 {d['base']} ・ "
                f"意味 {d['meaning']} ・ 部品 {d['parts']}")
    return d["css"]


TOOLS = [
    Tool(name="new", summary="雛形から記録のフォルダを起こす",
         args=[Arg("record", "記録のフォルダ（.acdr/<番号>-<短い名詞句>）"),
               Arg("title", "題。短い名詞句", required=False, default="題を記入する")],
         run=new, human=_human_new),
    Tool(name="validate", summary="入力（acdr.json）を検査する",
         args=[Arg("record", "記録のフォルダ")],
         run=validate, human=_human_validate),
    Tool(name="render", summary="acdr.json から1枚（index.html）を組む",
         args=[Arg("record", "記録のフォルダ"),
               Arg("check", "組み直さず、差が無いかだけを検査する", required=False),
               Arg("force", "承認済みの記録でも組み直す", required=False)],
         run=render, human=_human_render),
    Tool(name="tokens", summary="トークンを検査し、CSS を出す",
         args=[Arg("check", "検査だけを実行する", required=False)],
         run=tokens, human=_human_tokens),
]
