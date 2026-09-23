"""記録から、既存の道具の設定を組む。**検査そのものは書かない。**

検査は import-linter が実施する。ここが持つのは、記録を道具の言葉へ直す変換だけである。
"""
import json
import pathlib
import sys

記録 = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
層 = [記録["層"][名] for 名 in reversed(記録["並び"])]   # 外側から内側の順で並べる
出 = ["[importlinter]", "root_packages =", *[f"    {p}" for p in 層], "",
      "[importlinter:contract:1]", "name = 内側は外側を参照しない",
      "type = layers", "layers ="]
出 += [f"    {p}" for p in 層]
pathlib.Path(sys.argv[2]).write_text("\n".join(出) + "\n", encoding="utf-8")
print("\n".join(出))
