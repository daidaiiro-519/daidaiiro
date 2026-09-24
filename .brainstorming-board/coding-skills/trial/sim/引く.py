"""記録から、既存の道具の設定を組む。**検査そのものは書かない。**

検査は import-linter が実施する。ここが持つのは、記録を道具の言葉へ直す変換だけである。
"""
import json
import pathlib
import sys

record = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
layer = [record["層"][name] for name in reversed(record["並び"])]   # 外側から内側の順で並べる
out = ["[importlinter]", "root_packages =", *[f"    {p}" for p in layer], "",
      "[importlinter:contract:1]", "name = 内側は外側を参照しない",
      "type = layers", "layers ="]
out += [f"    {p}" for p in layer]
pathlib.Path(sys.argv[2]).write_text("\n".join(out) + "\n", encoding="utf-8")
print("\n".join(out))
