# SPDX-License-Identifier: MIT
"""規則に書いた道具を実行し、終了コードで判定する。

**道具の名前を、この側に書かない。** 実行するものは規則から来る ──
言語ごとの違いは規則が持つので、この Skill の依存は0件である。

**出力を解析しない。** 検出は道具の仕事で、意味の解釈は読み手が実施する ──
解析すると、道具ごとに違う書式へ依存する。
"""
from __future__ import annotations

import hashlib
import json
import pathlib
import secrets
import shutil
import subprocess
import tempfile

DEFAULT_TIMEOUT = 120
OUTPUT_HEAD = 8_000
OUTPUT_TAIL = 8_000
"""道具の出力から読む量（バイト）。**先頭と末尾の両方を読む。**

読み手は人だけではない ── MCP から呼ぶ側は、**この出力を見て次の手を決める**。
だから落としてよいのは真ん中だけである ── 翻訳器は先頭に、試験の実行器は末尾に
要るものを置く。**全文は保存し、続きは `read_output` が読む** ── 呼ぶ側が
MCP しか持たない環境でも、続きを取れるようにするためである。

解析ではない ── 読む量を限るだけである。**出力はファイルへ流す** ── まとめて
受け取ると、上限で切っても最大常駐は減らない（実測 2026-09-24、100MB で 306MB）。"""

VERDICT_LABEL = {"pass": "合格", "fail": "不合格", "skip": "実行しない"}
"""**機械が分岐する値は ASCII である。** 画面へ出す語は、この対応表が持つ。"""


def runs_dir(root: pathlib.Path) -> pathlib.Path:
    """この根の保存先。**根ごとに分ける** ── 別の成果物の実行を消さないためである。"""
    key = hashlib.sha256(str(root.resolve()).encode("utf-8")).hexdigest()[:16]
    return pathlib.Path(tempfile.gettempdir()) / "coding-skills-runs" / key


def load_rules(rules_file: pathlib.Path) -> list[dict]:
    """規則ファイルから規則の一覧を取り出す。規則が1件だけの形も受け取る。"""
    d = json.loads(rules_file.read_text(encoding="utf-8"))
    if isinstance(d.get("rules"), list):
        return d["rules"]
    return [d]


def run_one(rules: dict, root: pathlib.Path, timeout: int = DEFAULT_TIMEOUT,
            save_to: pathlib.Path | None = None, index: int = 0) -> dict:
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
    target = (rules.get("check") or {}).get("target") or ""
    cwd = (root / target).resolve()
    # **根の外を指す対象を実行しない。** 解決してから、根の中かを確認する ──
    # `../` や絶対のパスで、成果物の外を検査したことになるのを防ぐ。
    try:
        cwd.relative_to(root.resolve())
    except ValueError:
        return {**base, "verdict": "skip", "reason": f"対象が根の外を指す ── {target}"}
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
                   f"── 続きは read_output で読む（offset={OUTPUT_HEAD}）\n\n"
                   + tail.lstrip())
            if save_to is not None:
                save_to.mkdir(parents=True, exist_ok=True)
                keep = str(save_to / f"{index}.log")
                sink.seek(0)
                shutil.copyfileobj(sink, open(keep, "wb"))
    finally:
        sink.close()
        pathlib.Path(sink.name).unlink(missing_ok=True)
    return {**base, "exit": p.returncode, "output": out, "output_size": size,
            "saved": bool(keep),
            "verdict": "pass" if p.returncode == 0 else "fail"}


def check_rules(root: pathlib.Path, rules_file: pathlib.Path, timeout: int = DEFAULT_TIMEOUT) -> dict:
    """規則を全件実行する。**早期終了しない** ──
    途中で止めると、実行しなかった規則が合格と区別できない。
    """
    rules = load_rules(rules_file)
    # **保持するのは直近の1回だけである。** 文脈へ載れば、保持は要らない ──
    # 溜め続けると置き場所が膨らみ、時間で消すと読んでいる最中に消える。
    run = secrets.token_hex(4)
    save_to = runs_dir(root)
    shutil.rmtree(save_to, ignore_errors=True)
    results = [run_one(r, root, timeout, save_to / run, i) for i, r in enumerate(rules)]
    counts = {k: sum(1 for x in results if x["verdict"] == k) for k in ("pass", "fail", "skip")}
    findings = [f'{x["name"]} ── {VERDICT_LABEL[x["verdict"]]}' + (f'（{x["reason"]}）' if x.get("reason") else "")
            for x in results if x["verdict"] != "pass"]
    if not rules:
        # **1件も検査していない状態を、合格と同じ姿で返さない。**
        # 呼ぶ側が終了コードだけを読むと、検査した結果として受け取る。
        findings.append("規則が0件である ── 1件も検査していない")
    if any(x.get("saved") for x in results):
        (save_to / run / "index.json").write_text(json.dumps(
            {"run": run, "rules": [x["name"] for x in results]}, ensure_ascii=False),
            encoding="utf-8")
    return {"rules": results, **counts, "run": run, "rules_file": str(rules_file),
            "root": str(root), "findings": findings}


def plan_rules(root: pathlib.Path, rules_file: pathlib.Path) -> dict:
    """**実行しない。** 何を、どこで実行するかを返すだけである。

    原典は、拒める人が居ることを求めている ── `there SHOULD always be a human in
    the loop with the ability to deny tool invocations`
    （specification/2025-06-18/server/tools:26、2026-09-24 取得）。
    **承認はホストが持つ**ので、こちらは判断の材料を返す。
    """
    out = []
    for r in load_rules(rules_file):
        check = r.get("check") or {}
        out.append({"name": r.get("rule") or "（名前が無い）",
                    "tool": check.get("tool") or [],
                    "target": check.get("target") or ""})
    return {"root": str(root.resolve()), "rules_file": str(rules_file),
            "plan": out, "count": len(out)}


def read_output(root: pathlib.Path, run: str, name: str, offset: int = 0,
                length: int = OUTPUT_HEAD + OUTPUT_TAIL) -> dict:
    """保存した出力を読む。**道具は実行しない。**

    **合わない識別子には、明示して断る** ── 別の実行の中身を返すと、読む側は
    それを今回の結果として受け取る。
    """
    saved = runs_dir(root) / run
    index = saved / "index.json"
    if not index.exists():
        raise FileNotFoundError(
            f"その実行の出力は保持していない ── run={run}。check を実行し直す")
    names = json.loads(index.read_text(encoding="utf-8"))["rules"]
    if name not in names:
        raise FileNotFoundError(f"その名前は、この実行に無い ── {name}")
    path = saved / f"{names.index(name)}.log"
    if not path.exists():
        raise FileNotFoundError(f"その規則の出力は保持していない ── {name}")
    size = path.stat().st_size
    with path.open("rb") as f:
        f.seek(max(0, offset))
        body = f.read(max(1, length))
    nxt = offset + len(body)
    return {"run": run, "name": name, "offset": offset,
            "output": body.decode("utf-8", "replace"),
            "output_size": size, "next_offset": nxt if nxt < size else None}
