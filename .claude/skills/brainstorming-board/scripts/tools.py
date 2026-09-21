# SPDX-License-Identifier: MIT
"""brainstorming-board の道具の宣言。**能力の正本はここである。**

計算は `lib/` が保持し、この宣言は**呼び方だけ**を固定する。

**部品は宣言に載せない** ── `lib/` に在るものは読み込まれるものであり、
入口を保持しない。**入口は `cli.py` の1つだけである。**
"""
from __future__ import annotations

import io
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from lib import figcheck as _figcheck  # noqa: E402
from lib import init as _init  # noqa: E402
from lib import render_board as _render  # noqa: E402
from lib import serve as _serve  # noqa: E402
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


def validate(board: str) -> dict:
    """入力（board.json）を検査する。**組み上がりではなく、入力を検査する。**"""
    bad = _validate.check(pathlib.Path(board).resolve())
    return result(ok=True, findings=list(bad), board=board)


def _human_validate(res: dict) -> str:
    bad = res["findings"]
    return "\n".join(["  × " + e for e in bad]
                     + [f"入力の検査　通っていない（{len(bad)} 件）" if bad else "入力の検査　通った"])


def render(board: str, check: str = "") -> dict:
    """JSON からブレストボードを組む。`check` を渡すと、冪等だけを検査する。"""
    dir = pathlib.Path(board).resolve()
    body = _render.render(dir)
    if check:
        code, out, err = _capture(_render._check_idempotent, dir, body)
        # **通過の行は検出ではない。**検出は「×」の行だけである
        bad = [x.strip().lstrip("× ").strip() for x in (out + err).splitlines()
               if x.strip().startswith("×")]
        return result(ok=True, findings=bad, board=board, checked=True,
                      message=(out + err).strip().splitlines()[-1] if (out + err).strip() else "")
    title = json.loads((dir / "board.json").read_text(encoding="utf-8"))["title"]
    _render.write(body, str(dir / "board.html"), title)
    return result(ok=True, board=board, path=str(dir / "board.html"), bytes=len(body))


def _human_render(res: dict) -> str:
    d = res["data"]
    if d.get("checked"):
        return "\n".join(["  × " + x for x in res["findings"]] + [d.get("message", "")]).strip()
    return f"書き出し: {d['path']}"


def freeze(board: str) -> dict:
    """いまの姿を、前の回の基準として保存する。**組み直しでは前進させない。**"""
    code, out, err = _capture(_render.freeze, pathlib.Path(board).resolve())
    return result(ok=code == 0, findings=[], board=board,
                  message=(out + err).strip())


def _human_freeze(res: dict) -> str:
    return res["data"]["message"]


def figures(module: list | str) -> dict:
    """図の中の重なり・はみ出し・貫通を検査する。**絶対座標の図だけが対象である。**"""
    mods = [module] if isinstance(module, str) else list(module)
    code, out, err = _capture(_figcheck.count, mods)
    return result(ok=True, findings=[x for x in (out + err).splitlines() if x.strip()],
                  modules=mods, count=code)


def _human_figures(res: dict) -> str:
    n = res["data"]["count"]
    return "\n".join(res["findings"]
                     + ["食い違いは無い" if not n else f"直すところが {n} 件ある"])


def init(name: str, dir: str = ".brainstorming-board", title: str = "") -> dict:
    """ブレストボードの置き場所と雛形と索引の行を作る。**論点は書かない。**"""
    try:
        lines = _init.create(name, dir, title)
    except FileExistsError as e:
        # **検出であって、誤用ではない** ── 既に在ることは、呼び方の誤りではない
        return result(ok=True, findings=[str(e)], name=name)
    return result(ok=True, name=name, path=str(pathlib.Path(dir) / name), lines=lines)


def _human_init(res: dict) -> str:
    return "\n".join(res["data"].get("lines") or res["findings"])


def serve(root: str = ".", answers: str = "answers", port: str = "8731") -> dict:
    """ブレストボードを配り、押された回答を蓄積する。**止められるまで返らない。**"""
    code = _serve.run(root, answers, int(port))
    return result(ok=code == 0, root=root, answers=answers, port=int(port))


def _human_serve(res: dict) -> str:
    return "止めました" if res["ok"] else "配れなかった"


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
    Tool(name="init", summary="ブレストボードの置き場所と雛形と索引の行を作る",
         args=[Arg("name", "ブレストボードの名前（英小文字とハイフン）"),
               Arg("dir", "置き場所の親", required=False, default=".brainstorming-board"),
               Arg("title", "題。省くと名前をそのまま使う", required=False)],
         run=init, human=_human_init),
    Tool(name="validate", summary="入力（board.json）を検査する",
         args=[Arg("board", "ブレストボードのディレクトリ")],
         run=validate, human=_human_validate),
    Tool(name="render", summary="JSON からブレストボードを組む",
         args=[Arg("board", "ブレストボードのディレクトリ"),
               Arg("check", "冪等だけを検査する", required=False)],
         run=render, human=_human_render),
    Tool(name="tokens", summary="トークンを検査し、CSS を出す",
         args=[Arg("check", "検査だけを行う", required=False)],
         run=tokens, human=_human_tokens),
    Tool(name="freeze", summary="いまの姿を、前の回の基準として保存する",
         args=[Arg("board", "ブレストボードのディレクトリ")],
         run=freeze, human=_human_freeze),
    Tool(name="figures", summary="図の重なり・はみ出し・貫通を検査する",
         args=[Arg("module", "図を持つモジュール名", many=True, param="module")],
         run=figures, human=_human_figures),
    Tool(name="serve", summary="ブレストボードを配り、押された回答を蓄積する",
         args=[Arg("root", "画面を置いてあるディレクトリ", required=False, default="."),
               Arg("answers", "回答の蓄積先", required=False, default="answers"),
               Arg("port", "待ち受ける番号", required=False, default="8731")],
         run=serve, human=_human_serve),
]
