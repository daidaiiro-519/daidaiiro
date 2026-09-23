"""唯一の入口。`check <根> <記録> [--json]`

終了コードは 0 正常 ／ 1 検出あり ／ 2 誤用である。
"""
from __future__ import annotations

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from lib.run import 検査する  # noqa: E402


def 人が読む形(res: dict) -> str:
    行 = []
    for x in res["data"]["規則"]:
        印 = {"合格": "合格　", "不合格": "不合格", "実行しない": "実行せず"}[x["判定"]]
        行.append(f'  {印}　{x["名前"]}' + (f'　（{x["理由"]}）' if x.get("理由") else ""))
    for x in res["data"]["規則"]:
        if x["判定"] == "不合格" and x["出力"]:
            行 += ["", f'── {x["名前"]} の出力（道具のまま）', x["出力"]]
    d = res["data"]
    行.append(f'\n合格 {d["合格"]} ／ 不合格 {d["不合格"]} ／ 実行せず {d["実行しない"]}')
    return "\n".join(行)


def main(argv: list[str]) -> int:
    機械 = "--json" in argv
    引数 = [a for a in argv if not a.startswith("--")]
    if len(引数) != 2:
        print("使い方: cli.py <根> <記録> [--json]", file=sys.stderr)
        return 2
    res = 検査する(pathlib.Path(引数[0]), pathlib.Path(引数[1]))
    print(json.dumps(res, ensure_ascii=False, indent=1) if 機械 else 人が読む形(res))
    return 0 if res["ok"] else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
