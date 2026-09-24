"""唯一の入口。`check <根> <記録> [--json]`

終了コードは 0 正常 ／ 1 検出あり ／ 2 誤用である。
"""
from __future__ import annotations

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from lib.run import check  # noqa: E402


def human_form(res: dict) -> str:
    lines = []
    for x in res["data"]["規則"]:
        mark = {"合格": "合格　", "不合格": "不合格", "実行しない": "実行せず"}[x["判定"]]
        lines.append(f'  {印}　{x["名前"]}' + (f'　（{x["理由"]}）' if x.get("理由") else ""))
    for x in res["data"]["規則"]:
        if x["判定"] == "不合格" and x["出力"]:
            lines += ["", f'── {x["名前"]} の出力（道具のまま）', x["出力"]]
    d = res["data"]
    lines.append(f'\n合格 {d["合格"]} ／ 不合格 {d["不合格"]} ／ 実行せず {d["実行しない"]}')
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    as_json = "--json" in argv
    args = [a for a in argv if not a.startswith("--")]
    if len(args) != 2:
        print("使い方: cli.py <根> <記録> [--json]", file=sys.stderr)
        return 2
    res = check(pathlib.Path(args[0]), pathlib.Path(args[1]))
    print(json.dumps(res, ensure_ascii=False, indent=1) if as_json else human_form(res))
    return 0 if res["ok"] else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
