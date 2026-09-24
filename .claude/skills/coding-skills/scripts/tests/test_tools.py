# SPDX-License-Identifier: MIT
"""雛形の生成 ・ 提示 ・ 続きの読み取りを、事例で検証する。

    python3 scripts/tests/test_tools.py
"""
import json
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import init as _init  # noqa: E402
from lib import run  # noqa: E402

count = 0


def expect(name: str, ok: bool) -> None:
    global count
    count += 1
    if not ok:
        raise AssertionError(name)


def write_rules(root: pathlib.Path, rules: list) -> pathlib.Path:
    p = root / "rules.json"
    p.write_text(json.dumps({"rules": rules}, ensure_ascii=False), encoding="utf-8")
    return p


def tool(cmd: list, name: str = "試し", target: str = "") -> dict:
    check = {"tool": cmd}
    if target:
        check["target"] = target
    return {"rule": name, "source": {"record": "x"}, "scope": "x", "check": check}


def init_places_the_skeleton() -> None:
    """雛形を .coding/rules.json へ置く。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        path = _init.create(root, {"core": "internal/core", "app": "app.core"})
        expect("置き場所は .coding/rules.json である", path == root / ".coding" / "rules.json")
        d = json.loads(path.read_text(encoding="utf-8"))
        expect("並びが層と一致する", d["order"] == ["core", "app"])
        expect("識別子の形を問わない", d["layers"]["app"] == "app.core")
        expect("内を指す規則を2件置く", len(d["rules"]) == 2)
        expect("道具は空である", d["rules"][0]["check"]["tool"] == [])


def init_refuses_to_overwrite() -> None:
    """既に在れば作り直さない。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        _init.create(root, {"core": "x"})
        try:
            _init.create(root, {"core": "y"})
            expect("既に在れば断る", False)
        except FileExistsError:
            expect("既に在れば断る", True)


def init_needs_layers() -> None:
    """層が無ければ断る。"""
    with tempfile.TemporaryDirectory() as t:
        try:
            _init.create(pathlib.Path(t), {})
            expect("層が無ければ断る", False)
        except ValueError:
            expect("層が無ければ断る", True)
    for bad in ("core", "=x", "core="):
        try:
            _init.parse_layers([bad])
            expect(f"層の形が違えば断る ── {bad}", False)
        except ValueError:
            expect(f"層の形が違えば断る ── {bad}", True)


def plan_runs_nothing() -> None:
    """plan は1件も実行しない。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        mark = root / "実行した"
        p = write_rules(root, [tool(["python3", "-c", f"open({str(mark)!r}, 'w')"])])
        d = run.plan_rules(root, p)
        expect("実行する内容を返す", d["plan"][0]["tool"][0] == "python3")
        expect("件数を返す", d["count"] == 1)
        expect("1件も実行していない", not mark.exists())


def target_outside_root_is_refused() -> None:
    """根の外を指す対象は実行しない。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        r = run.run_one(tool(["true"], target="../"), root)
        expect("根の外は実行しない", r["verdict"] == "skip")
        expect("理由を添える", "根の外" in r["reason"])


def output_is_readable_in_slices() -> None:
    """保持した出力を、位置を変えて読む。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        n = run.OUTPUT_HEAD + run.OUTPUT_TAIL
        cmd = ["python3", "-c", f"print('あ' + 'x' * {n * 2} + 'ん')"]
        p = write_rules(root, [tool(cmd, "長い")])
        d = run.check_rules(root, p)
        run_id = d["run"]
        expect("実行の識別子を返す", bool(run_id))
        expect("全体の大きさを返す", d["rules"][0]["output_size"] > n)
        expect("続きの読み方を書く", "read_output" in d["rules"][0]["output"])

        a = run.read_output(root, run_id, "長い", 0, 100)
        expect("先頭から読める", a["output"].startswith("あ"))
        expect("次の位置を返す", a["next_offset"] == 100)
        b = run.read_output(root, run_id, "長い", a["output_size"] - 10, 100)
        expect("末尾まで読める", b["output"].rstrip().endswith("ん"))
        expect("末尾では次の位置を返さない", b["next_offset"] is None)


def stale_run_is_refused() -> None:
    """合わない識別子には、明示して断る。"""
    with tempfile.TemporaryDirectory() as t:
        root = pathlib.Path(t)
        n = run.OUTPUT_HEAD + run.OUTPUT_TAIL
        p = write_rules(root, [tool(["python3", "-c", f"print('x' * {n * 2})"], "長い")])
        first = run.check_rules(root, p)["run"]
        second = run.check_rules(root, p)["run"]
        expect("実行ごとに識別子が変わる", first != second)
        try:
            run.read_output(root, first, "長い")
            expect("前の実行は断る", False)
        except FileNotFoundError as e:
            expect("前の実行は断る", "保持していない" in str(e))
        expect("直近の実行は読める", bool(run.read_output(root, second, "長い")["output"]))
        try:
            run.read_output(root, second, "無い規則")
            expect("無い名前は断る", False)
        except FileNotFoundError:
            expect("無い名前は断る", True)


if __name__ == "__main__":
    for f in (init_places_the_skeleton, init_refuses_to_overwrite, init_needs_layers,
              plan_runs_nothing, target_outside_root_is_refused,
              output_is_readable_in_slices, stale_run_is_refused):
        f()
    print(f"道具の検査　{count} 件　通った")
