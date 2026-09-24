# SPDX-License-Identifier: MIT
"""概念の出典を、事例で検証する。`python3 scripts/tests/test_concepts.py` で実行する。

**この Skill は、DDD ・ クリーン ・ ヘキサゴナルの知識を散文で保持しない。**
手順が依拠する概念は `references/concepts.json` が持ち、1件ずつ引用と取得元を備える。
"""
import json
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import REFERENCES, validate  # noqa: E402

count = 0


def expect(name: str, ok: bool) -> None:
    global count
    count += 1
    if not ok:
        raise AssertionError(name)


def write_file(cwd: pathlib.Path, body: dict) -> pathlib.Path:
    p = cwd / "concepts.json"
    p.write_text(json.dumps(body, ensure_ascii=False), encoding="utf-8")
    return p


ONE = {
    "concept": "試しの概念",
    "applies": "試験でのみ使う",
    "source": {
        "meta": "試験",
        "quote": "Gather together the things that change for the same reasons.",
        "fetched": {"url": "https://example.invalid/", "date": "2026-09-22",
                    "sha256": "0" * 64, "line": 1},
    },
}


def canonical_meets_contract() -> None:
    findings = validate.check_concepts(REFERENCES / "concepts.json")
    expect("正本の概念ファイルが契約を満たす ── " + " ／ ".join(findings), not findings)


def all_three_present() -> None:
    d = json.loads((REFERENCES / "concepts.json").read_text(encoding="utf-8"))
    name = [c["concept"] for c in d["concepts"]]
    for x in ("変更理由で分ける", "依存の向き", "ポートとアダプタ"):
        expect(f"{x} が正本に在る", x in name)


def detects_missing_quote() -> None:
    with tempfile.TemporaryDirectory() as t:
        c = json.loads(json.dumps(ONE))
        c["source"].pop("quote")
        findings = validate.check_concepts(write_file(pathlib.Path(t), {"concepts": [c]}))
        expect("引用の欠落を検出する", any("引用が無い" in x or "引用" in x for x in findings))


def detects_missing_fetch_field() -> None:
    for field in ("url", "date", "sha256", "line"):
        with tempfile.TemporaryDirectory() as t:
            c = json.loads(json.dumps(ONE))
            c["source"]["fetched"].pop(field)
            findings = validate.check_concepts(write_file(pathlib.Path(t), {"concepts": [c]}))
            expect(f"取得の {field} の欠落を検出する", bool(findings))


def detects_unreadable() -> None:
    with tempfile.TemporaryDirectory() as t:
        p = pathlib.Path(t) / "concepts.json"
        p.write_text("{", encoding="utf-8")
        expect("JSON として読めないことを検出する", bool(validate.check_concepts(p)))


if __name__ == "__main__":
    for f in (canonical_meets_contract, all_three_present, detects_missing_quote,
              detects_missing_fetch_field, detects_unreadable):
        f()
    print(f"概念の検査　{count} 件　通った")
