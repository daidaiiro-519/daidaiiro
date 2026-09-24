# SPDX-License-Identifier: MIT
"""実行と判定を、事例で検証する。`python3 scripts/tests/test_run.py` で実行する。"""
import json
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import run, validate  # noqa: E402

件 = 0


def 検査(名: str, 条件: bool) -> None:
    global 件
    件 += 1
    if not 条件:
        raise AssertionError(名)


def 規則を書く(場所: pathlib.Path, 規則: list) -> pathlib.Path:
    p = 場所 / "rules.json"
    p.write_text(json.dumps({"rules": 規則}, ensure_ascii=False), encoding="utf-8")
    return p


def 道具(コマンド: list, 名: str = "試し") -> dict:
    return {"rule": 名, "source": {"meta": "試験"}, "scope": "試験",
            "check": {"tool": コマンド}}


with tempfile.TemporaryDirectory() as d:
    根 = pathlib.Path(d)

    # ── 終了コードが判定になる
    r = run.検査する(根, 規則を書く(根, [道具(["true"], "通る"), 道具(["false"], "落ちる")]))
    検査("合格が1件", r["pass"] == 1)
    検査("不合格が1件", r["fail"] == 1)
    検査("検出は不合格だけ", r["findings"] == ["落ちる ── 不合格"])

    # ── 道具が無いときは、合格にも不合格にもしない
    r = run.検査する(根, 規則を書く(根, [道具(["この道具は無い"], "無い")]))
    検査("実行しないが1件", r["skip"] == 1)
    検査("合格にしない", r["pass"] == 0)
    検査("理由を添える", r["rules"][0]["reason"] == "道具が見つからない")

    # ── 出力は道具のまま渡す
    r = run.検査する(根, 規則を書く(根, [道具(["sh", "-c", "echo ここが違反; exit 1"], "出す")]))
    検査("出力をそのまま持つ", r["rules"][0]["output"] == "ここが違反")

    # ── 早期終了しない
    r = run.検査する(根, 規則を書く(根, [道具(["false"], "1"), 道具(["true"], "2"), 道具(["false"], "3")]))
    検査("全件を実行する", len(r["rules"]) == 3)

    # ── 規則ファイルの形
    検査("道具が無ければ検出する",
         any("道具が無い" in x for x in validate.検査する(
             規則を書く(根, [{"rule": "無い", "source": {"meta": "x"}, "scope": "x",
                             "check": {}}]))))
    検査("道具が配列でなければ検出する",
         any("配列ではない" in x for x in validate.検査する(
             規則を書く(根, [{"rule": "文字列", "source": {"meta": "x"}, "scope": "x",
                             "check": {"tool": "gofmt -l ."}}]))))
    検査("出典が無ければ検出する",
         any("出典が無い" in x for x in validate.検査する(
             規則を書く(根, [{"rule": "出典なし", "scope": "x",
                             "check": {"tool": ["true"]}}]))))

    # ── 層の場所
    (根 / "internal").mkdir(exist_ok=True)
    p = 根 / "layers.json"
    p.write_text(json.dumps({"order": ["domain", "adapter"],
                            "layers": {"domain": "internal", "adapter": "無い場所"}},
                           ensure_ascii=False), encoding="utf-8")
    検出 = validate.層を検査する(p, 根)
    検査("実在しない層を検出する", any("実在しない" in x for x in 検出))
    検査("実在する層は検出しない", not any("domain" in x for x in 検出))

print(f"\n{件} 件すべて通った")
