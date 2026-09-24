# SPDX-License-Identifier: MIT
"""規則に書いた道具を実行し、終了コードで判定する。

**道具の名前を、この側に書かない。** 実行するものは規則から来る ──
言語ごとの違いは規則が持つので、この Skill の依存は0件である。

**出力を解析しない。** 検出は道具の仕事で、意味の解釈は読み手が実施する ──
解析すると、道具ごとに違う書式へ依存する。
"""
from __future__ import annotations

import json
import pathlib
import subprocess
import tempfile

DEFAULT_TIMEOUT = 120
OUTPUT_HEAD = 8_000
OUTPUT_TAIL = 8_000
"""道具の出力から読む量（バイト）。**頭と尻の両方を読む。**

読み手は人だけではない ── MCP から呼ぶ側は、**この出力を見て次の手を決める**。
だから落としてよいのは真ん中だけである ── 翻訳器は先頭に、試験の実行器は末尾に
要るものを置く。**全文はファイルへ残し、その場所を返す** ── 足りなければ、
読む側が自分で開く。

解析ではない ── 読む量を限るだけである。**出力はファイルへ流す** ── まとめて
受け取ると、上限で切っても最大常駐は減らない（実測 2026-09-24、100MB で 306MB）。"""

VERDICT_LABEL = {"pass": "合格", "fail": "不合格", "skip": "実行しない"}
"""**機械が分岐する値は ASCII である。** 画面へ出す語は、この対応表が持つ。"""


def load_rules(rules_file: pathlib.Path) -> list[dict]:
    """規則ファイルから規則の一覧を取り出す。規則が1件だけの形も受け取る。"""
    d = json.loads(rules_file.read_text(encoding="utf-8"))
    if isinstance(d.get("rules"), list):
        return d["rules"]
    return [d]


def run_one(rules: dict, root: pathlib.Path, timeout: int = DEFAULT_TIMEOUT) -> dict:
    """1件を実行する。**殻を経由しない** ── 配列のまま渡す。

    文字列1本で渡すと、`./...` の展開が実行する殻に依存する。
    **`check.target` が在れば、そこで実行する** ── 範囲を狭めるのは、規則を緩めるのでは
    なく、見ている範囲を書くことである。実在しなければ「実行しない」で、合格に寄せない。
    """
    tool = (rules.get("check") or {}).get("tool")
    name = rules.get("rule") or "（名前が無い）"
    base = {"name": name, "tool": tool, "exit": None, "output": ""}
    if not tool:
        return {**base, "verdict": "skip", "reason": "検証方法に道具が無い"}
    if not isinstance(tool, list):
        return {**base, "verdict": "skip", "reason": "道具が配列ではない"}
    cwd = root / ((rules.get("check") or {}).get("target") or "")
    if not cwd.is_dir():
        return {**base, "verdict": "skip", "reason": f"対象が実在しない ── {cwd}"}
    # **出力をファイルへ流し、上限までしか読まない。**
    # まとめて受け取ると、道具が出した量がそのままこの側の記憶に載る
    # （実測 2026-09-24、100MB を出す道具で最大常駐 306MB）。
    sink = tempfile.NamedTemporaryFile(prefix="coding-skills-", suffix=".log",
                                       delete=False)
    keep = ""
    try:
        try:
            p = subprocess.run(tool, cwd=cwd, stdin=subprocess.DEVNULL,
                               stdout=sink, stderr=sink, timeout=timeout)
        except FileNotFoundError:
            return {**base, "verdict": "skip", "reason": "道具が見つからない"}
        except subprocess.TimeoutExpired:
            return {**base, "verdict": "skip", "reason": f"{timeout}秒で終わらない"}
        size = sink.tell()
        sink.seek(0)
        if size <= OUTPUT_HEAD + OUTPUT_TAIL:
            out = sink.read(size).decode("utf-8", "replace").strip()
        else:
            head = sink.read(OUTPUT_HEAD).decode("utf-8", "replace")
            sink.seek(size - OUTPUT_TAIL)
            tail = sink.read(OUTPUT_TAIL).decode("utf-8", "replace")
            out = (head.rstrip() + f"\n\n── 中略（全 {size} バイトのうち "
                   f"{OUTPUT_HEAD + OUTPUT_TAIL} バイトを表示）\n"
                   f"── 全文: {sink.name}\n\n" + tail.lstrip())
            keep = sink.name
    finally:
        sink.close()
        if not keep:
            pathlib.Path(sink.name).unlink(missing_ok=True)
    return {**base, "exit": p.returncode, "output": out, "output_file": keep,
            "verdict": "pass" if p.returncode == 0 else "fail"}


def check_rules(root: pathlib.Path, rules_file: pathlib.Path, timeout: int = DEFAULT_TIMEOUT) -> dict:
    """規則を全件実行する。**早期終了しない** ──
    途中で止めると、実行しなかった規則が合格と区別できない。
    """
    rules = load_rules(rules_file)
    results = [run_one(r, root, timeout) for r in rules]
    counts = {k: sum(1 for x in results if x["verdict"] == k) for k in ("pass", "fail", "skip")}
    findings = [f'{x["name"]} ── {VERDICT_LABEL[x["verdict"]]}' + (f'（{x["reason"]}）' if x.get("reason") else "")
            for x in results if x["verdict"] != "pass"]
    if not rules:
        # **1件も検査していない状態を、合格と同じ姿で返さない。**
        # 呼ぶ側が終了コードだけを読むと、検査した結果として受け取る。
        findings.append("規則が0件である ── 1件も検査していない")
    return {"rules": results, **counts, "rules_file": str(rules_file), "root": str(root),
            "findings": findings}
