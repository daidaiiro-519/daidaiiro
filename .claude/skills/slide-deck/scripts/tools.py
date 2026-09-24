# SPDX-License-Identifier: MIT
"""slide-deck の道具の宣言。**能力の正本はここである。**

計算は `lib/` が保持し、この宣言は**呼び方だけ**を固定する。
CLI も MCP もここから組む ── 能力を2回記述すると、片方だけが古くなる。

**この Skill は図を描かない。** 渡すのは配色だけである ── `theme` が、
テーマのキーを**図の中の役割の名前**へ複製して出す。誰に組ませるかは配線表が決める。
"""
from __future__ import annotations

import json
import pathlib
import sys

_HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(_HERE))

from contract import Arg, Tool, result  # noqa: E402
from lib import render_deck as _render  # noqa: E402
from lib import themes as _themes  # noqa: E402


def check(deck: list | str = "") -> dict:
    """テーマの形を検査する。**見た目は見ない。**

    キーがすべてのテーマで一致しているか ・ 適合条件（文字と地の比）を満たすか ・
    テーマの外に色の直書きが残っていないかの3つである。
    """
    decks = [deck] if isinstance(deck, str) and deck else list(deck or [])
    bad = _themes.findings(decks)
    return result(ok=True, findings=bad, themes=len(_themes.theme_files()),
                  decks=len(decks))


def _human_check(res: dict) -> str:
    d, bad = res["data"], res["findings"]
    return "\n".join([f"  × {x}" for x in bad]
                     + [f'テーマの検査　{"通った" if not bad else f"通っていない（{len(bad)} 件）"}'
                        f'　／　テーマ {d["themes"]} 本、デッキ {d["decks"]} 本'])


def theme(name: str = "", out: str = "") -> dict:
    """テーマを、図の中の役割ごとの色へ複製して出す。

    名前を省くと、置いてあるテーマの一覧を出す。**色の正本は1つである** ──
    図の側にも色を書くと、テーマを替えたときに図だけが前の配色のまま残る。
    """
    if not name:
        return result(ok=True, names=_themes.theme_names(), tokens={})
    try:
        tokens = _themes.as_roles(name)
    except ValueError as e:
        return result(ok=True, findings=[str(e)], names=_themes.theme_names(), tokens={})
    if out:
        pathlib.Path(out).write_text(
            json.dumps(tokens, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    return result(ok=True, name=name, tokens=tokens, path=out,
                  names=_themes.theme_names())


def _human_theme(res: dict) -> str:
    d = res["data"]
    if res["findings"]:
        return "\n".join(f"  × {x}" for x in res["findings"])
    if not d["tokens"]:
        return "テーマ: " + " ・ ".join(d["names"])
    if d.get("path"):
        return f'書き出し: {d["path"]}　／　{len(d["tokens"])} 件'
    return json.dumps(d["tokens"], ensure_ascii=False, indent=1)


def new(out: str, theme: str = "warm-paper", title: str = "題を記入する") -> dict:
    """デッキの入力（JSON）を起こす。**同じ名前が在れば起こさない。**

    **書くのは中身だけである** ── 形は型が、配色はテーマが持つ。
    """
    path = pathlib.Path(out)
    if path.exists():
        return result(ok=True, findings=[f"既に在る: {out} ── 作り直さない"], out=out)
    deck = json.loads((_HERE.parent / "references" / "deck-example.json")
                      .read_text(encoding="utf-8"))
    deck["title"], deck["theme"] = title, theme
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(deck, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    return result(ok=True, out=str(path), theme=theme)


def _human_new(res: dict) -> str:
    d = res["data"]
    if res["findings"]:
        return "\n".join(f"  × {x}" for x in res["findings"])
    return "\n".join([
        f'作った: {d["out"]}　／　テーマ {d["theme"]}',
        "次にすること ── 枚を書き、図は組ませて、返った SVG を figure へ置く。",
        f'組む: `cli.py render {d["out"]} <出力.html>`',
        f'配色は `cli.py theme {d["theme"]}` が出す役割ごとの色を、描く側へ渡す。'])


def render(deck: str, out: str = "", check: str = "") -> dict:
    """入力（JSON）から1枚の HTML を組む。**同じ入力からは、同じ1枚が出る。**"""
    source = pathlib.Path(deck)
    dest = pathlib.Path(out) if out else source.with_suffix(".html")
    try:
        code, note = _render.build_deck(source, dest, check_only=bool(check))
    except SystemExit as e:
        return result(ok=True, findings=str(e).splitlines()[1:], deck=deck)
    return result(ok=True, findings=[] if code == 0 else [f"{dest}: {note}"],
                  deck=deck, out=str(dest), note=note)


def _human_render(res: dict) -> str:
    d, bad = res["data"], res["findings"]
    if bad:
        return "\n".join([x if x.strip().startswith("×") else "  × " + x for x in bad]
                          + [f"組めていない（{len(bad)} 件）"])
    return f'組んだ: {d["out"]}　／　{d["note"]}'


TOOLS = [
    Tool(name="new", summary="デッキの入力（JSON）を起こす",
         args=[Arg("out", "書き出し先の JSON"),
               Arg("theme", "テーマの名前", required=False, default="warm-paper"),
               Arg("title", "題", required=False, default="題を記入する")],
         run=new, human=_human_new),
    Tool(name="render", summary="入力（JSON）から1枚の HTML を組む",
         args=[Arg("deck", "デッキの入力（JSON）"),
               Arg("out", "書き出し先の HTML。省くと入力と同じ名前", required=False),
               Arg("check", "組み直さず、差が無いかだけを検査する", required=False)],
         run=render, human=_human_render),
    Tool(name="check", summary="テーマの形を検査する",
         args=[Arg("deck", "デッキの HTML", required=False, many=True, param="deck")],
         run=check, human=_human_check),
    Tool(name="theme", summary="テーマを、図の中の役割ごとの色へ複製して出す",
         args=[Arg("name", "テーマの名前。省くと一覧", required=False),
               Arg("out", "書き出し先の JSON", required=False)],
         run=theme, human=_human_theme),
]
