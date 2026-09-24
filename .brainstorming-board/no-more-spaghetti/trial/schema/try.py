# SPDX-License-Identifier: MIT
"""論点8 の試作 ── スキーマから実体を生成し、案内の欠落を検出する。

    python3 try.py

3つを順に実行する ── ①案内の検査 ②欠落させたときの検出 ③実体の生成。
"""
import copy
import json
import pathlib

import jsonschema

here = pathlib.Path(__file__).resolve().parent
V = jsonschema.Draft202012Validator


def load(name: str) -> dict:
    return json.loads((here / name).read_text(encoding="utf-8"))


def check_prompt(schema: dict, meta: dict) -> list[str]:
    return ["/".join(map(str, e.path)) + " ── " + e.message
            for e in sorted(V(meta).iter_errors(schema), key=lambda x: list(x.path))]


def read_prompt(schema: dict, path: list[str]) -> dict:
    """項目1件の3つの案内を取り出す。**AI が読む導線である。**"""
    section = schema
    for x in path:
        section = section[x]
    p = section.get("x-prompt") or {}
    return {"概要": section.get("description"), "読む": p.get("read"), "埋める": p.get("write")}


def make_template(schema: dict, layer: dict[str, str]) -> dict:
    """スキーマから実体の雛形を組む。**項目の一覧を、この側に書かない。**"""
    shape = schema["$defs"]["rule"]["properties"]
    def rules(name: str) -> dict:
        out = {}
        for key in shape:
            out[key] = name if key == "rule" else ("" if shape[key]["type"] == "string" else {})
        out["check"] = {"tool": []}
        return out
    return {"order": list(layer), "layers": dict(layer),
            "rules": [rules("層の場所が、宣言した対応と一致する"),
                    rules("依存の向きが、内から外へ出ていない")]}


if __name__ == "__main__":
    schema, meta = load("rules.schema.json"), load("schema-meta.schema.json")

    print("── ① 案内の検査")
    findings = check_prompt(schema, meta)
    print("  " + ("\n  ".join(findings) if findings else "検出0件"))

    print("\n── ② 1件だけ案内を落として、検出されるかを見る")
    broken = copy.deepcopy(schema)
    broken["properties"]["layers"].pop("x-prompt")
    broken["$defs"]["rule"]["properties"]["source"]["x-prompt"]["write"] = ""
    for x in check_prompt(broken, meta):
        print("  " + x)

    print("\n── ③ AI が読む導線（層の項目）")
    for k, v in read_prompt(schema, ["properties", "layers"]).items():
        print(f"  {k}　{v}")

    print("\n── ④ 生成した実体 .coding/rules.json")
    template = make_template(schema, {"core": "internal/core", "app": "internal/app",
                          "adapter": "internal/adapter"})
    print(json.dumps(template, ensure_ascii=False, indent=2))

    print("\n── ⑤ 生成した実体を、契約で検証する")
    out = ["/".join(map(str, e.path)) + " ── " + e.message
          for e in sorted(V(schema).iter_errors(template), key=lambda x: list(x.path))]
    print("  " + ("\n  ".join(out) if out else "検出0件"))
