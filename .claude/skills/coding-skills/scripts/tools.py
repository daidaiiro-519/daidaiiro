# SPDX-License-Identifier: MIT
"""道具の宣言。**能力の正本であり、CLI と MCP はここから組む。**

この Skill は道具を1つも持たない ── 実行するのは、規則に書いてあるコマンドである。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from contract import Arg, Tool, result  # noqa: E402
from lib import run as _run  # noqa: E402
from lib import validate as _validate  # noqa: E402


def check(根: str, 規則: str, 制限: str = "") -> dict:
    """規則を全件実行し、終了コードで判定する。

    **出力は道具のまま渡す。** 件数の集計も、違反の並べ替えも実施しない。
    """
    d = _run.検査する(pathlib.Path(根), pathlib.Path(規則),
                     int(制限) if 制限 else _run.制限の既定)
    検出 = d.pop("検出")
    return result(ok=True, findings=検出, **d)


def _human_check(res: dict) -> str:
    d = res["data"]
    印 = {"合格": "合格　", "不合格": "不合格", "実行しない": "実行せず"}
    行 = [f'  {印[x["判定"]]}　{x["名前"]}' + (f'　（{x["理由"]}）' if x.get("理由") else "")
          for x in d["規則"]]
    for x in d["規則"]:
        if x["判定"] == "不合格" and x["出力"]:
            行 += ["", f'── {x["名前"]} の出力（道具のまま）', x["出力"]]
    行.append(f'\n合格 {d["合格"]} ／ 不合格 {d["不合格"]} ／ 実行せず {d["実行しない"]}')
    return "\n".join(行)


def validate(規則: str, 根: str = "") -> dict:
    """規則ファイルの形を検査する。**実行の前に見る。**

    根を渡すと、層の場所が実在するかも見る。
    **渡されたファイルの種類で、当てる契約が変わる** ── 規則 ・ 概念 ・ スキーマの3つ。
    入口を増やすと、呼ぶ側が形を推測することになる。
    """
    p = pathlib.Path(規則)
    if _スキーマのファイルか(p):
        return result(ok=True, findings=_validate.スキーマを検査する(p),
                      ファイル=規則, 種類="スキーマ")
    if _概念のファイルか(p):
        return result(ok=True, findings=_validate.概念を検査する(p),
                      ファイル=規則, 種類="概念")
    検出 = _validate.検査する(p)
    if 根:
        検出 += _validate.層を検査する(p, pathlib.Path(根))
    return result(ok=True, findings=検出, ファイル=規則, 種類="規則", 規則ファイル=規則)


def _スキーマのファイルか(p: pathlib.Path) -> bool:
    try:
        import json
        d = json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return False
    return isinstance(d, dict) and "properties" in d and "規則" not in d and "概念" not in d


def _概念のファイルか(p: pathlib.Path) -> bool:
    try:
        import json
        d = json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return False
    return isinstance(d, dict) and "概念" in d and "規則" not in d


def _human_validate(res: dict) -> str:
    名 = res["data"]["種類"] + "ファイル"
    return (f"{名}の検査　通った" if not res["findings"]
            else f'{名}の検査　通っていない（{len(res["findings"])} 件）')


TOOLS = [
    Tool("check", "規則を全件実行し、終了コードで判定する",
         [Arg("根", "成果物の場所"), Arg("規則", "規則ファイルのパス"),
          Arg("制限", "1件あたりの制限時間（秒）", required=False)],
         run=check, human=_human_check),
    Tool("validate", "規則ファイルの形を検査する",
         [Arg("規則", "規則ファイルのパス"), Arg("根", "成果物の場所（層の実在も見る）", required=False)],
         run=validate, human=_human_validate),
]
