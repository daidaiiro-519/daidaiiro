# SPDX-License-Identifier: MIT
"""実行と判定を、事例で検証する。`python3 scripts/tests/test_run.py` で実行する。"""
import json
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import run, validate  # noqa: E402

count = 0


def expect(name: str, ok: bool) -> None:
    global count
    count += 1
    if not ok:
        raise AssertionError(name)


def write_rules(cwd: pathlib.Path, rules: list) -> pathlib.Path:
    p = cwd / "rules.json"
    p.write_text(json.dumps({"rules": rules}, ensure_ascii=False), encoding="utf-8")
    return p


def tool(cmd: list, name: str = "試し") -> dict:
    return {"rule": name, "source": {"meta": "試験"}, "scope": "試験",
            "check": {"tool": cmd}}


with tempfile.TemporaryDirectory() as d:
    root = pathlib.Path(d)

    # ── 終了コードが判定になる
    r = run.check_rules(root, write_rules(root, [tool(["true"], "通る"), tool(["false"], "落ちる")]))
    expect("合格が1件", r["pass"] == 1)
    expect("不合格が1件", r["fail"] == 1)
    expect("検出は不合格だけ", r["findings"] == ["落ちる ── 不合格"])

    # ── 道具が無いときは、合格にも不合格にもしない
    r = run.check_rules(root, write_rules(root, [tool(["この道具は無い"], "無い")]))
    expect("実行しないが1件", r["skip"] == 1)
    expect("合格にしない", r["pass"] == 0)
    expect("理由を添える", r["rules"][0]["reason"] == "道具が見つからない")

    # ── 出力は道具のまま渡す
    r = run.check_rules(root, write_rules(root, [tool(["sh", "-c", "echo ここが違反; exit 1"], "出す")]))
    expect("出力をそのまま持つ", r["rules"][0]["output"] == "ここが違反")

    # ── 早期終了しない
    r = run.check_rules(root, write_rules(root, [tool(["false"], "1"), tool(["true"], "2"), tool(["false"], "3")]))
    expect("全件を実行する", len(r["rules"]) == 3)

    # ── 規則ファイルの形
    expect("道具が無ければ検出する",
         any("道具が無い" in x for x in validate.check_rules(
             write_rules(root, [{"rule": "無い", "source": {"meta": "x"}, "scope": "x",
                             "check": {}}]))))
    expect("道具が配列でなければ検出する",
         any("配列ではない" in x for x in validate.check_rules(
             write_rules(root, [{"rule": "文字列", "source": {"meta": "x"}, "scope": "x",
                             "check": {"tool": "gofmt -l ."}}]))))
    expect("出典が無ければ検出する",
         any("出典が無い" in x for x in validate.check_rules(
             write_rules(root, [{"rule": "出典なし", "scope": "x",
                             "check": {"tool": ["true"]}}]))))

    # ── 層の場所
    (root / "internal").mkdir(exist_ok=True)
    p = root / "layers.json"
    p.write_text(json.dumps({"order": ["domain", "adapter", "足りない層"],
                            "layers": {"domain": "internal",
                                       "adapter": "com.example.adapter"}},
                           ensure_ascii=False), encoding="utf-8")
    findings = validate.check_layers(p, root)
    expect("パッケージ名を、場所として検査しない", not any("com.example" in x for x in findings))
    expect("並びに在って層に無いものを検出する", any("足りない層" in x for x in findings))

    # ── 対象
    (root / "sub").mkdir(exist_ok=True)
    r = run.run_one({"rule": "狭めた範囲", "check": {"tool": ["pwd"], "target": "sub"}}, root)
    expect("対象の場所で実行する", r["output"].endswith("sub"))
    r = run.run_one({"rule": "無い範囲", "check": {"tool": ["pwd"], "target": "無い"}}, root)
    expect("実在しない対象を、合格に寄せない", r["verdict"] == "skip")

    # ── 1件も検査していない状態
    p = root / "空.json"
    p.write_text('{"rules": []}', encoding="utf-8")
    r = run.check_rules(root, p)
    expect("規則が0件なら、検出として出す", any("0件" in x for x in r["findings"]))

    # ── 標準入力を待つ道具
    r = run.run_one({"rule": "入力を待つ", "check": {"tool": ["cat"]}}, root, 5)
    expect("標準入力を待つ道具が、制限時間まで止まらない", r["verdict"] == "pass")

    # ── 出力の上限。**先頭と末尾の両方を残す** ── 読む側は、この出力で次の手を決める
    n = run.OUTPUT_HEAD + run.OUTPUT_TAIL
    long_tool = ["python3", "-c",
                 f"print('先頭の手がかり'); print('x' * {n * 2}); print('末尾の手がかり')"]
    r = run.run_one({"rule": "長い出力", "check": {"tool": long_tool}}, root)
    expect("出力を上限で切る", len(r["output"]) < n * 2)
    expect("先頭を残す", "先頭の手がかり" in r["output"])
    expect("末尾を残す", "末尾の手がかり" in r["output"])
    expect("中略したと書く", "中略" in r["output"])
    expect("続きの読み方を書く", "read_output" in r["output"])
    expect("全体の大きさを返す", r["output_size"] > n * 2)

    r = run.run_one({"rule": "短い出力", "check": {"tool": ["python3", "-c", "print(1)"]}}, root)
    expect("切っていなければ、保持しない", not r["saved"])

print(f"\n{count} 件すべて通った")
