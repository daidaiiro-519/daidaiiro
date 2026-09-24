"""記録に書いた道具を実行し、終了コードで判定する。

**道具の名前を、この側に書かない。** 実行するものは記録から来る ──
言語ごとの違いは、記録が持つ。

**出力を解析しない。** 検出は道具の仕事で、意味の解釈は読み手が実施する。
解析すると、道具ごとに違う書式へ依存する。
"""
from __future__ import annotations

import json
import pathlib
import subprocess


def 読む(記録: pathlib.Path) -> list[dict]:
    """記録から規則の一覧を取り出す。1件だけの記録も受け取る。"""
    d = json.loads(記録.read_text(encoding="utf-8"))
    if isinstance(d.get("規則"), list):
        return d["規則"]
    return [d]


def 実行する(規則: dict, 根: pathlib.Path, 制限: int = 120) -> dict:
    """1件を実行する。**シェルを経由しない** ── 配列のまま渡す。"""
    道具 = (規則.get("検証方法") or {}).get("道具")
    名前 = 規則.get("規則") or 規則.get("名前") or "（名前が無い）"
    if not 道具:
        return {"名前": 名前, "道具": None, "終了コード": None, "出力": "",
                "判定": "実行しない", "理由": "検証方法に道具が無い"}
    try:
        p = subprocess.run(道具, cwd=根, capture_output=True, text=True, timeout=制限)
    except FileNotFoundError:
        return {"名前": 名前, "道具": 道具, "終了コード": None, "出力": "",
                "判定": "実行しない", "理由": "道具が見つからない"}
    except subprocess.TimeoutExpired:
        return {"名前": 名前, "道具": 道具, "終了コード": None, "出力": "",
                "判定": "実行しない", "理由": f"{制限}秒で終わらない"}
    出力 = (p.stdout + p.stderr).strip()
    return {"名前": 名前, "道具": 道具, "終了コード": p.returncode, "出力": 出力,
            "判定": "合格" if p.returncode == 0 else "不合格"}


def 検査する(根: pathlib.Path, 記録: pathlib.Path) -> dict:
    結果 = [実行する(r, 根) for r in 読む(記録)]
    数 = {k: sum(1 for x in 結果 if x["判定"] == k) for k in ("合格", "不合格", "実行しない")}
    検出 = [f'{x["名前"]} ── {x["判定"]}' + (f'（{x["理由"]}）' if x.get("理由") else "")
            for x in 結果 if x["判定"] != "合格"]
    return {"ok": not 検出, "findings": 検出,
            "data": {"規則": 結果, **数, "記録": str(記録)}}
