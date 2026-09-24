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


def load(record: pathlib.Path) -> list[dict]:
    """記録から規則の一覧を取り出す。1件だけの記録も受け取る。"""
    d = json.loads(record.read_text(encoding="utf-8"))
    if isinstance(d.get("規則"), list):
        return d["規則"]
    return [d]


def run_one(rules: dict, root: pathlib.Path, timeout: int = 120) -> dict:
    """1件を実行する。**シェルを経由しない** ── 配列のまま渡す。"""
    tool = (rules.get("検証方法") or {}).get("道具")
    name = rules.get("規則") or rules.get("名前") or "（名前が無い）"
    if not tool:
        return {"名前": name, "道具": None, "終了コード": None, "出力": "",
                "判定": "実行しない", "理由": "検証方法に道具が無い"}
    try:
        p = subprocess.run(tool, cwd=root, capture_output=True, text=True, timeout=timeout)
    except FileNotFoundError:
        return {"名前": name, "道具": tool, "終了コード": None, "出力": "",
                "判定": "実行しない", "理由": "道具が見つからない"}
    except subprocess.TimeoutExpired:
        return {"名前": name, "道具": tool, "終了コード": None, "出力": "",
                "判定": "実行しない", "理由": f"{制限}秒で終わらない"}
    output = (p.stdout + p.stderr).strip()
    return {"名前": name, "道具": tool, "終了コード": p.returncode, "出力": output,
            "判定": "合格" if p.returncode == 0 else "不合格"}


def check(root: pathlib.Path, record: pathlib.Path) -> dict:
    result = [run_one(r, root) for r in load(record)]
    counts = {k: sum(1 for x in result if x["判定"] == k) for k in ("合格", "不合格", "実行しない")}
    findings = [f'{x["名前"]} ── {x["判定"]}' + (f'（{x["理由"]}）' if x.get("理由") else "")
            for x in result if x["判定"] != "合格"]
    return {"ok": not findings, "findings": findings,
            "data": {"規則": result, **counts, "記録": str(record)}}
