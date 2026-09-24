# SPDX-License-Identifier: MIT
"""規則に書いた道具を実行し、終了コードで判定する。

**道具の名前を、この側に書かない。** 実行するものは規則から来る ──
言語ごとの違いは規則が持つので、この Skill の依存は0件である。

**出力を解析しない。** 検出は道具の仕事で、意味の解釈は読み手が実施する ──
解析すると、道具ごとに違う書式へ依存する。
"""
from __future__ import annotations

import json
import pathlib
import subprocess

制限の既定 = 120
出力の上限 = 200_000
"""道具の出力を保持する上限（文字）。**超えた分は切り落とし、切ったことを書く。**

解析ではない ── 読む先を限るだけである。上限が無いと、道具が出した量がそのまま
この側の記憶に載る（実測 2026-09-24、100MB を出す道具で最大常駐 406MB）。"""

判定の語 = {"pass": "合格", "fail": "不合格", "skip": "実行しない"}
"""**機械が分岐する値は ASCII である。** 画面へ出す語は、この対応表が持つ。"""


def 読む(規則ファイル: pathlib.Path) -> list[dict]:
    """規則ファイルから規則の一覧を取り出す。規則が1件だけの形も受け取る。"""
    d = json.loads(規則ファイル.read_text(encoding="utf-8"))
    if isinstance(d.get("rules"), list):
        return d["rules"]
    return [d]


def 実行する(規則: dict, 根: pathlib.Path, 制限: int = 制限の既定) -> dict:
    """1件を実行する。**殻を経由しない** ── 配列のまま渡す。

    文字列1本で渡すと、`./...` の展開が実行する殻に依存する。
    **`check.target` が在れば、そこで実行する** ── 範囲を狭めるのは、規則を緩めるのでは
    なく、見ている範囲を書くことである。実在しなければ「実行しない」で、合格に寄せない。
    """
    道具 = (規則.get("check") or {}).get("tool")
    名前 = 規則.get("rule") or "（名前が無い）"
    素 = {"name": 名前, "tool": 道具, "exit": None, "output": ""}
    if not 道具:
        return {**素, "verdict": "skip", "reason": "検証方法に道具が無い"}
    if not isinstance(道具, list):
        return {**素, "verdict": "skip", "reason": "道具が配列ではない"}
    場所 = 根 / ((規則.get("check") or {}).get("target") or "")
    if not 場所.is_dir():
        return {**素, "verdict": "skip", "reason": f"対象が実在しない ── {場所}"}
    try:
        p = subprocess.run(道具, cwd=場所, stdin=subprocess.DEVNULL,
                           capture_output=True, text=True, timeout=制限)
    except FileNotFoundError:
        return {**素, "verdict": "skip", "reason": "道具が見つからない"}
    except subprocess.TimeoutExpired:
        return {**素, "verdict": "skip", "reason": f"{制限}秒で終わらない"}
    出 = (p.stdout + p.stderr).strip()
    if len(出) > 出力の上限:
        出 = 出[:出力の上限] + f"\n── ここで切った（{len(出)} 文字のうち {出力の上限} 文字）"
    return {**素, "exit": p.returncode, "output": 出,
            "verdict": "pass" if p.returncode == 0 else "fail"}


def 検査する(根: pathlib.Path, 規則ファイル: pathlib.Path, 制限: int = 制限の既定) -> dict:
    """規則を全件実行する。**早期終了しない** ──
    途中で止めると、実行しなかった規則が合格と区別できない。
    """
    規則 = 読む(規則ファイル)
    結果 = [実行する(r, 根, 制限) for r in 規則]
    数 = {k: sum(1 for x in 結果 if x["verdict"] == k) for k in ("pass", "fail", "skip")}
    検出 = [f'{x["name"]} ── {判定の語[x["verdict"]]}' + (f'（{x["reason"]}）' if x.get("reason") else "")
            for x in 結果 if x["verdict"] != "pass"]
    if not 規則:
        # **1件も検査していない状態を、合格と同じ姿で返さない。**
        # 呼ぶ側が終了コードだけを読むと、検査した結果として受け取る。
        検出.append("規則が0件である ── 1件も検査していない")
    return {"rules": 結果, **数, "rules_file": str(規則ファイル), "root": str(根),
            "findings": 検出}
