# SPDX-License-Identifier: MIT
"""案内の検査を、事例で検証する。`python3 scripts/tests/test_schema.py` で実行する。

**案内を要求するのは、実体を生成するスキーマだけである** ── 最上位に
`x-generates` を持つものが対象になる。
"""
import copy
import json
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import REFERENCES, validate  # noqa: E402

件 = 0


def 検査(名: str, 条件: bool) -> None:
    global 件
    件 += 1
    if not 条件:
        raise AssertionError(名)


def 書く(中身: dict) -> pathlib.Path:
    t = tempfile.mkdtemp()
    p = pathlib.Path(t) / "x.schema.json"
    p.write_text(json.dumps(中身, ensure_ascii=False), encoding="utf-8")
    return p


def 正本を読む() -> dict:
    return json.loads((REFERENCES / "rules.schema.json").read_text(encoding="utf-8"))


def 正本は案内を備える() -> None:
    検出 = validate.スキーマを検査する(REFERENCES / "rules.schema.json")
    検査("正本が案内を備える ── " + " ／ ".join(検出), not 検出)


def 生成しないスキーマは要求されない() -> None:
    for 名 in ("concepts.schema.json", "schema-meta.schema.json"):
        検出 = validate.スキーマを検査する(REFERENCES / 名)
        検査(f"{名} は案内を要求されない ── " + " ／ ".join(検出), not 検出)


def 案内の欠落を検出する() -> None:
    d = 正本を読む()
    d["properties"]["layers"].pop("x-prompt")
    検出 = validate.スキーマを検査する(書く(d))
    検査("最上位の欠落を検出する", any("properties/layers" in x for x in 検出))

    d = 正本を読む()
    d["$defs"]["rule"]["properties"]["source"].pop("x-prompt")
    検出 = validate.スキーマを検査する(書く(d))
    検査("$defs の欠落を検出する", any("$defs/rule/properties/source" in x for x in 検出))


def 空の案内を検出する() -> None:
    d = 正本を読む()
    d["properties"]["layers"]["x-prompt"]["write"] = ""
    検出 = validate.スキーマを検査する(書く(d))
    検査("空文字を検出する", any("too short" in x for x in 検出))


def 片方だけの案内を検出する() -> None:
    d = 正本を読む()
    d["properties"]["layers"]["x-prompt"] = {"read": "読む"}
    検出 = validate.スキーマを検査する(書く(d))
    検査("write の欠落を検出する", any("write" in x for x in 検出))


def 無効なスキーマを検出する() -> None:
    d = 正本を読む()
    d["properties"]["layers"]["description"] = {"read": "オブジェクトにした"}
    検出 = validate.スキーマを検査する(書く(d))
    検査("スキーマとして無効なことを検出する", any("無効" in x for x in 検出))


def 生成しない宣言なら通る() -> None:
    d = copy.deepcopy(正本を読む())
    d.pop("x-generates")
    d["properties"]["layers"].pop("x-prompt")
    検査("x-generates が無ければ、案内を要求しない",
        not validate.スキーマを検査する(書く(d)))


if __name__ == "__main__":
    for f in (正本は案内を備える, 生成しないスキーマは要求されない, 案内の欠落を検出する,
              空の案内を検出する, 片方だけの案内を検出する, 無効なスキーマを検出する,
              生成しない宣言なら通る):
        f()
    print(f"案内の検査　{件} 件　通った")
