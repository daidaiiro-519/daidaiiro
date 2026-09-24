# SPDX-License-Identifier: MIT
"""道具の契約 ── **能力は1つ、呼び出し方は複数**。

道具は `TOOLS` として宣言する。CLI も MCP も、**この宣言から組む** ──
能力を2回書くと、片方だけが古くなる。

戻り値は3つの欄を持つ。
  ok       正常に終わったか
  findings 検出したもの（**誤りではない**。0件でも正常である）
  data     機械が読む本体
"""
from __future__ import annotations

import json
import sys
from dataclasses import dataclass, field
from typing import Callable


@dataclass
class Arg:
    """引数1つ。**位置引数と旗を区別しない** ── 呼ぶ側の形は CLI が決める。

    `name` は**呼ぶ側が使う名前**、`param` は実体の引数名である ──
    実装の都合（予約語の回避など）を、呼ぶ側の名前へ漏らさない。
    """
    name: str
    summary: str
    required: bool = True
    many: bool = False
    default: str | None = None
    param: str | None = None

    @property
    def key(self) -> str:
        """実体へ渡すときの名前。"""
        return self.param or self.name


@dataclass
class Tool:
    """道具1つ。`run` は dict を返す ── 印字はしない。

    **印字と終了コードは入口が持つ。**道具が印字すると、MCP から呼んだとき
    戻り値が空になる。
    """
    name: str
    summary: str
    args: list[Arg] = field(default_factory=list)
    run: Callable[..., dict] | None = None
    human: Callable[[dict], str] | None = None


def result(ok: bool = True, findings: list | None = None, **data) -> dict:
    """戻り値を組む。**findings は検出であって、誤りではない。**"""
    return {"ok": ok, "findings": list(findings or []), "data": data}


def exit_code(res: dict) -> int:
    """0 正常 ／ 1 検出あり ／ 2 誤用。**検出を誤用と同じ番号にしない。**"""
    if not res.get("ok", False):
        return 2
    return 1 if res.get("findings") else 0


def emit(res: dict, as_json: bool, human: Callable[[dict], str] | None) -> int:
    """人向けと機械向けの、**同じ内容を2つの形で出す**。"""
    if as_json:
        print(json.dumps(res, ensure_ascii=False, indent=1))
    else:
        print(human(res) if human else json.dumps(res, ensure_ascii=False, indent=1))
    return exit_code(res)


def main(tools: list[Tool], argv: list[str] | None = None) -> int:
    """唯一の入口。`<動詞> [対象…] [--json]`"""
    argv = list(sys.argv[1:] if argv is None else argv)
    table = {t.name: t for t in tools}
    as_json = "--json" in argv
    argv = [a for a in argv if a != "--json"]

    if not argv or argv[0] in ("-h", "--help", "help"):
        print("道具の一覧")
        for t in tools:
            need = " ".join(f"<{a.name}>" if a.required else f"[{a.name}]" for a in t.args)
            print(f"  {t.name} {need}\n      {t.summary}")
        print("\n  どれも --json を付けると、機械が読む形で出る")
        return 0 if argv else 2

    verb, rest = argv[0], argv[1:]
    if verb not in table:
        print(f"その動詞は無い: {verb} ── 一覧は help である", file=sys.stderr)
        return 2

    t = table[verb]
    # 旗は `--名前=値` と `--名前 値` の両方を受ける ──
    # **書き方を1つに強制すると、既存の手順が壊れる。**
    byname = {a.name: a for a in t.args}
    kw, pos, i = {}, [], 0
    while i < len(rest):
        a = rest[i]
        if a.startswith("--"):
            body = a[2:]
            if "=" in body:
                k, _, v = body.partition("=")
            else:
                k = body
                nxt = rest[i + 1] if i + 1 < len(rest) else None
                # **値を伴わない旗は、立っている。** 空にすると、`--check` が
                # 黙って逆の意味になり、検査のつもりで正本を書き換える。
                # 次が別の旗なら、それは値ではない ── 後ろの引数も呑み込まない
                if nxt is None or nxt.startswith("--"):
                    v = "1"
                else:
                    v = nxt
                    i += 1
            spec = byname.get(k) or byname.get(k.replace("-", "_"))
            kw[spec.key if spec else k.replace("-", "_")] = v
        else:
            pos.append(a)
        i += 1

    need = [a for a in t.args if a.required]
    if len(pos) < len(need) and not all(a.key in kw for a in need):
        print(f"引数が足りない: {t.name} は {' '.join(a.name for a in need)} を要する",
              file=sys.stderr)
        return 2

    # **まとめて受ける引数は、並びのどこに在ってもよい。**
    # 最後に在ると決めると、その後ろに任意の引数を置けなくなる。
    many = next((i for i, a in enumerate(t.args) if a.many), None)
    if many is None:
        for a, v in zip(t.args, pos):
            kw.setdefault(a.key, v)
    else:
        for a, v in zip(t.args[:many], pos):
            kw.setdefault(a.key, v)
        tail = pos[many:]
        if tail:
            kw.setdefault(t.args[many].key, tail)

    try:
        res = t.run(**kw) if t.run else result(ok=False)
    except TypeError as e:
        print(f"引数が合わない: {e}", file=sys.stderr)
        return 2
    except (OSError, json.JSONDecodeError) as e:
        # **読めないものを渡すのは誤用である。** 検出（1）と同じ番号で返すと、
        # 呼ぶ側は「違反が在った」と解釈する。
        print(f"読めない: {e}", file=sys.stderr)
        return 2
    return emit(res, as_json, t.human)
