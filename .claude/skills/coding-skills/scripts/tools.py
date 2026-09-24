# SPDX-License-Identifier: MIT
"""道具の宣言。**能力の正本であり、CLI と MCP はここから組む。**

この Skill は道具を1つも持たない ── 実行するのは、規則に書いてあるコマンドである。
"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from contract import Arg, Tool, result  # noqa: E402
from lib import run as _run  # noqa: E402
from lib import init as _init  # noqa: E402
from lib import validate as _validate  # noqa: E402


def check(root: str, rules: str, timeout: str = "") -> dict:
    """規則を全件実行し、終了コードで判定する。

    **出力は道具のまま渡す。** 件数の集計も、違反の並べ替えも実施しない。
    **読めないファイルは誤用である** ── 検出（違反が在る）と同じ番号で返さない。
    """
    try:
        d = _run.check_rules(pathlib.Path(root), pathlib.Path(rules),
                         int(timeout) if timeout else _run.DEFAULT_TIMEOUT)
    except (OSError, ValueError) as e:
        return result(ok=False, findings=[f"規則ファイルを読めない ── {e}"],
                      file=rules, kind="rules")
    findings = d.pop("findings")
    return result(ok=True, findings=findings, **d)


def _human_check(res: dict) -> str:
    d = res["data"]
    if "rules" not in d:
        return "\n".join(res["findings"])
    mark = {"pass": "合格　", "fail": "不合格", "skip": "実行せず"}
    lines = [f'  {mark[x["verdict"]]}　{x["name"]}' + (f'　（{x["reason"]}）' if x.get("reason") else "")
          for x in d["rules"]]
    for x in d["rules"]:
        if x["verdict"] == "fail" and x["output"]:
            lines += ["", f'── {x["name"]} の出力（道具のまま）', x["output"]]
    if not d["rules"]:
        lines.append("  規則が0件である ── 1件も検査していない")
    lines.append(f'\n合格 {d["pass"]} ／ 不合格 {d["fail"]} ／ 実行せず {d["skip"]}')
    return "\n".join(lines)


def _unreadable(p: pathlib.Path) -> str:
    import json
    try:
        json.loads(p.read_text(encoding="utf-8"))
    except OSError as e:
        return f"ファイルを読めない ── {e}"
    except json.JSONDecodeError as e:
        return f"JSON として読めない ── {e}"
    return ""


def validate(rules: str, root: str = "") -> dict:
    """規則ファイルの形を検査する。**実行の前に見る。**

    根を渡すと、層の場所が実在するかも見る。
    **渡されたファイルの種類で、当てる契約が変わる** ── 規則 ・ 概念 ・ スキーマの3つ。
    入口を増やすと、呼ぶ側が形を推測することになる。
    """
    p = pathlib.Path(rules)
    problem = _unreadable(p)
    if problem:
        return result(ok=False, findings=[problem], file=rules, kind="rules")
    if _is_schema_file(p):
        return result(ok=True, findings=_validate.check_schema(p),
                      file=rules, kind="schema")
    if _is_concepts_file(p):
        return result(ok=True, findings=_validate.check_concepts(p),
                      file=rules, kind="concepts")
    findings = _validate.check_rules(p)
    if root:
        findings += _validate.check_layers(p, pathlib.Path(root))
    return result(ok=True, findings=findings, file=rules, kind="rules")


def _is_schema_file(p: pathlib.Path) -> bool:
    try:
        import json
        d = json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return False
    return isinstance(d, dict) and "properties" in d and "rules" not in d and "concepts" not in d


def _is_concepts_file(p: pathlib.Path) -> bool:
    try:
        import json
        d = json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return False
    return isinstance(d, dict) and "concepts" in d and "rules" not in d


KIND_LABEL = {"rules": "規則", "concepts": "概念", "schema": "スキーマ"}
"""**機械が分岐する値は ASCII である。** 画面へ出す語は、この対応表が持つ。"""


def init(root: str, layer: list | str) -> dict:
    """規則ファイルの雛形を `.coding/rules.json` へ置く。**中身は呼ぶ側が書く。**"""
    try:
        layers = _init.parse_layers(layer)
        path = _init.create(pathlib.Path(root), layers)
    except (OSError, ValueError) as e:
        return result(ok=False, findings=[str(e)], root=root)
    return result(ok=True, path=str(path), layers=layers,
                  findings=["道具が空である ── 検証方法を書く（x-prompt.write が案内する）",
                            "出典が空である ── 内を指す規則なので、モデルの識別子を record に書く"])


def _human_init(res: dict) -> str:
    if not res["ok"]:
        return " ／ ".join(res["findings"])
    d = res["data"]
    return (f'置いた: {d["path"]}\n  層 ' + " ・ ".join(f"{k}={v}" for k, v in d["layers"].items())
            + "\n\n次に書くもの\n  " + "\n  ".join(res["findings"]))


def plan(root: str, rules: str) -> dict:
    """何を、どこで実行するかを返す。**1件も実行しない。**"""
    try:
        d = _run.plan_rules(pathlib.Path(root), pathlib.Path(rules))
    except (OSError, ValueError) as e:
        return result(ok=False, findings=[f"規則ファイルを読めない ── {e}"],
                      file=rules, kind="rules")
    return result(ok=True, **d)


def _human_plan(res: dict) -> str:
    if not res["ok"]:
        return " ／ ".join(res["findings"])
    d = res["data"]
    lines = [f'  根       {d["root"]}']
    for x in d["plan"]:
        場所 = f'（{x["target"]}）' if x["target"] else "（根）"
        lines.append(f'  {x["name"]}\n      {" ".join(x["tool"])}　{場所}')
    lines.append(f'\n{d["count"]} 件を実行する。実行はしていない。')
    return "\n".join(lines)


def read_output(root: str, run: str, name: str, offset: str = "0",
                length: str = "") -> dict:
    """保持した出力の続きを読む。**道具は実行しない。**"""
    try:
        d = _run.read_output(pathlib.Path(root), run, name, int(offset or 0),
                             int(length or _run.OUTPUT_HEAD + _run.OUTPUT_TAIL))
    except (OSError, ValueError) as e:
        return result(ok=False, findings=[str(e)], run=run, name=name)
    return result(ok=True, **d)


def _human_read_output(res: dict) -> str:
    d = res["data"]
    if not res["ok"]:
        return " ／ ".join(res["findings"])
    尾 = (f'続きは offset={d["next_offset"]}' if d["next_offset"] is not None
          else "ここで終わりである")
    return f'{d["output"]}\n\n── {d["offset"]} から {len(d["output"])} 文字。{尾}'


def _human_validate(res: dict) -> str:
    name = KIND_LABEL[res["data"]["kind"]] + "ファイル"
    return (f"{name}の検査　通った" if not res["findings"]
            else f'{name}の検査　通っていない（{len(res["findings"])} 件）')


TOOLS = [
    Tool("check", "規則を全件実行し、終了コードで判定する",
         [Arg("root", "成果物の場所"), Arg("rules", "規則ファイルのパス"),
          Arg("timeout", "1件あたりの制限時間（秒）", required=False)],
         run=check, human=_human_check),
    Tool("init", "規則ファイルの雛形を .coding/rules.json へ置く",
         [Arg("root", "成果物の場所"),
          Arg("layer", "層を 名前=識別子 で渡す（複数可）", many=True)],
         run=init, human=_human_init),
    Tool("plan", "何を、どこで実行するかを返す（実行はしない）",
         [Arg("root", "成果物の場所"), Arg("rules", "規則ファイルのパス")],
         run=plan, human=_human_plan),
    Tool("output", "保持した出力の続きを読む（実行はしない）",
         [Arg("root", "成果物の場所"), Arg("run", "実行の識別子"),
          Arg("name", "規則の名前"),
          Arg("offset", "読み始める位置（バイト）", required=False),
          Arg("length", "読む量（バイト）", required=False)],
         run=read_output, human=_human_read_output),
    Tool("validate", "規則ファイルの形を検査する",
         [Arg("rules", "規則ファイルのパス"), Arg("root", "成果物の場所（層の宣言も見る）", required=False)],
         run=validate, human=_human_validate),
]
