# SPDX-License-Identifier: MIT
"""論点8 の試作 ── スキーマから実体を生成し、案内の欠落を検出する。

    python3 try.py

3つを順に実行する ── ①案内の検査 ②欠落させたときの検出 ③実体の生成。
"""
import copy
import json
import pathlib

import jsonschema

ここ = pathlib.Path(__file__).resolve().parent
V = jsonschema.Draft202012Validator


def 読む(名: str) -> dict:
    return json.loads((ここ / 名).read_text(encoding="utf-8"))


def 案内を検査する(スキーマ: dict, メタ: dict) -> list[str]:
    return ["/".join(map(str, e.path)) + " ── " + e.message
            for e in sorted(V(メタ).iter_errors(スキーマ), key=lambda x: list(x.path))]


def 案内を引く(スキーマ: dict, 道: list[str]) -> dict:
    """項目1件の3つの案内を取り出す。**AI が読む導線である。**"""
    節 = スキーマ
    for x in 道:
        節 = 節[x]
    p = 節.get("x-prompt") or {}
    return {"概要": 節.get("description"), "読む": p.get("read"), "埋める": p.get("write")}


def 雛形を出す(スキーマ: dict, 層: dict[str, str]) -> dict:
    """スキーマから実体の雛形を組む。**項目の一覧を、この側に書かない。**"""
    形 = スキーマ["$defs"]["rule"]["properties"]
    def 規則(名: str) -> dict:
        出 = {}
        for 鍵 in 形:
            出[鍵] = 名 if 鍵 == "rule" else ("" if 形[鍵]["type"] == "string" else {})
        出["check"] = {"tool": []}
        return 出
    return {"order": list(層), "layers": dict(層),
            "rules": [規則("層の場所が、宣言した対応と一致する"),
                    規則("依存の向きが、内から外へ出ていない")]}


if __name__ == "__main__":
    スキーマ, メタ = 読む("rules.schema.json"), 読む("schema-meta.schema.json")

    print("── ① 案内の検査")
    検出 = 案内を検査する(スキーマ, メタ)
    print("  " + ("\n  ".join(検出) if 検出 else "検出0件"))

    print("\n── ② 1件だけ案内を落として、検出されるかを見る")
    壊す = copy.deepcopy(スキーマ)
    壊す["properties"]["layers"].pop("x-prompt")
    壊す["$defs"]["rule"]["properties"]["source"]["x-prompt"]["write"] = ""
    for x in 案内を検査する(壊す, メタ):
        print("  " + x)

    print("\n── ③ AI が読む導線（層の項目）")
    for k, v in 案内を引く(スキーマ, ["properties", "layers"]).items():
        print(f"  {k}　{v}")

    print("\n── ④ 生成した実体 .coding/rules.json")
    雛形 = 雛形を出す(スキーマ, {"core": "internal/core", "app": "internal/app",
                          "adapter": "internal/adapter"})
    print(json.dumps(雛形, ensure_ascii=False, indent=2))

    print("\n── ⑤ 生成した実体を、契約で検証する")
    出 = ["/".join(map(str, e.path)) + " ── " + e.message
          for e in sorted(V(スキーマ).iter_errors(雛形), key=lambda x: list(x.path))]
    print("  " + ("\n  ".join(出) if 出 else "検出0件"))
