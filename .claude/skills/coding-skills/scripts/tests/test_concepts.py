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

件 = 0


def 検査(名: str, 条件: bool) -> None:
    global 件
    件 += 1
    if not 条件:
        raise AssertionError(名)


def 書く(場所: pathlib.Path, 中身: dict) -> pathlib.Path:
    p = 場所 / "concepts.json"
    p.write_text(json.dumps(中身, ensure_ascii=False), encoding="utf-8")
    return p


一件 = {
    "概念": "試しの概念",
    "適用": "試験でのみ使う",
    "出典": {
        "meta": "試験",
        "引用": "Gather together the things that change for the same reasons.",
        "取得": {"url": "https://example.invalid/", "日": "2026-09-22",
                 "sha256": "0" * 64, "行": 1},
    },
}


def 正本は契約を満たす() -> None:
    検出 = validate.概念を検査する(REFERENCES / "concepts.json")
    検査("正本の概念ファイルが契約を満たす ── " + " ／ ".join(検出), not 検出)


def 三件そろっている() -> None:
    d = json.loads((REFERENCES / "concepts.json").read_text(encoding="utf-8"))
    名 = [c["概念"] for c in d["概念"]]
    for x in ("変更理由で分ける", "依存の向き", "ポートとアダプタ"):
        検査(f"{x} が正本に在る", x in 名)


def 引用が無ければ検出する() -> None:
    with tempfile.TemporaryDirectory() as t:
        c = json.loads(json.dumps(一件))
        c["出典"].pop("引用")
        検出 = validate.概念を検査する(書く(pathlib.Path(t), {"概念": [c]}))
        検査("引用の欠落を検出する", any("引用が無い" in x or "引用" in x for x in 検出))


def 取得の欄が欠けていれば検出する() -> None:
    for 欄 in ("url", "日", "sha256", "行"):
        with tempfile.TemporaryDirectory() as t:
            c = json.loads(json.dumps(一件))
            c["出典"]["取得"].pop(欄)
            検出 = validate.概念を検査する(書く(pathlib.Path(t), {"概念": [c]}))
            検査(f"取得の {欄} の欠落を検出する", bool(検出))


def 読めなければ検出する() -> None:
    with tempfile.TemporaryDirectory() as t:
        p = pathlib.Path(t) / "concepts.json"
        p.write_text("{", encoding="utf-8")
        検査("JSON として読めないことを検出する", bool(validate.概念を検査する(p)))


if __name__ == "__main__":
    for f in (正本は契約を満たす, 三件そろっている, 引用が無ければ検出する,
              取得の欄が欠けていれば検出する, 読めなければ検出する):
        f()
    print(f"概念の検査　{件} 件　通った")
