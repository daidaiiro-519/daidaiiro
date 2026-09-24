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

count = 0


def expect(name: str, 条件: bool) -> None:
    global count
    count += 1
    if not 条件:
        raise AssertionError(name)


def write_file(中身: dict) -> pathlib.Path:
    t = tempfile.mkdtemp()
    p = pathlib.Path(t) / "x.schema.json"
    p.write_text(json.dumps(中身, ensure_ascii=False), encoding="utf-8")
    return p


def load_canonical() -> dict:
    return json.loads((REFERENCES / "rules.schema.json").read_text(encoding="utf-8"))


def canonical_has_prompts() -> None:
    findings = validate.check_schema(REFERENCES / "rules.schema.json")
    expect("正本が案内を備える ── " + " ／ ".join(findings), not findings)


def non_generating_exempt() -> None:
    for name in ("concepts.schema.json", "schema-meta.schema.json"):
        findings = validate.check_schema(REFERENCES / name)
        expect(f"{name} は案内を要求されない ── " + " ／ ".join(findings), not findings)


def detects_missing_prompt() -> None:
    d = load_canonical()
    d["properties"]["layers"].pop("x-prompt")
    findings = validate.check_schema(write_file(d))
    expect("最上位の欠落を検出する", any("properties/layers" in x for x in findings))

    d = load_canonical()
    d["$defs"]["rule"]["properties"]["source"].pop("x-prompt")
    findings = validate.check_schema(write_file(d))
    expect("$defs の欠落を検出する", any("$defs/rule/properties/source" in x for x in findings))


def detects_empty_prompt() -> None:
    d = load_canonical()
    d["properties"]["layers"]["x-prompt"]["write"] = ""
    findings = validate.check_schema(write_file(d))
    expect("空文字を検出する", any("too short" in x for x in findings))


def detects_half_prompt() -> None:
    d = load_canonical()
    d["properties"]["layers"]["x-prompt"] = {"read": "読む"}
    findings = validate.check_schema(write_file(d))
    expect("write の欠落を検出する", any("write" in x for x in findings))


def detects_invalid_schema() -> None:
    d = load_canonical()
    d["properties"]["layers"]["description"] = {"read": "オブジェクトにした"}
    findings = validate.check_schema(write_file(d))
    expect("スキーマとして無効なことを検出する", any("無効" in x for x in findings))


def passes_without_x_generates() -> None:
    d = copy.deepcopy(load_canonical())
    d.pop("x-generates")
    d["properties"]["layers"].pop("x-prompt")
    expect("x-generates が無ければ、案内を要求しない",
        not validate.check_schema(write_file(d)))


if __name__ == "__main__":
    for f in (canonical_has_prompts, non_generating_exempt, detects_missing_prompt,
              detects_empty_prompt, detects_half_prompt, detects_invalid_schema,
              passes_without_x_generates):
        f()
    print(f"案内の検査　{count} 件　通った")
