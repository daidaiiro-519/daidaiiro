# SPDX-License-Identifier: MIT
"""skills-creator の道具の宣言。**能力の正本はここである。**"""
from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from contract import Arg, Tool, result  # noqa: E402

_TMPL = pathlib.Path(__file__).resolve().parents[1] / "references" / "tool-contract"


def scaffold(skill: str, path: str = ".claude/skills") -> dict:
    """道具の契約一式を、Skill のフォルダへ置く。**既に在るものは上書きしない。**"""
    root = pathlib.Path(path) / skill
    scripts = root / "scripts"
    (scripts / "lib").mkdir(parents=True, exist_ok=True)
    (scripts / "tests").mkdir(parents=True, exist_ok=True)
    written, kept = [], []
    pairs = [("contract.py", pathlib.Path(__file__).with_name("contract.py").read_text(encoding="utf-8")),
             ("tools.py", (_TMPL / "tools.py.tmpl").read_text(encoding="utf-8")),
             ("cli.py", (_TMPL / "cli.py.tmpl").read_text(encoding="utf-8")),
             ("mcp_server.py", (_TMPL / "mcp_server.py.tmpl").read_text(encoding="utf-8")),
             ("lib/__init__.py", (_TMPL / "lib__init__.py.tmpl").read_text(encoding="utf-8")),
             ("lib/template.py", (_TMPL / "template.py.tmpl").read_text(encoding="utf-8"))]
    for name, body in pairs:
        dst = scripts / name
        if dst.exists():
            kept.append(str(dst))
            continue
        dst.write_text(body.replace("{{Skill名}}", skill), encoding="utf-8")
        written.append(str(dst))
    # 型の雛形 ── 生成物を持つ Skill だけが使う。**形はここが持つ**
    tpl = root / "references" / f"{skill}.template.html"
    if not tpl.exists():
        tpl.parent.mkdir(parents=True, exist_ok=True)
        tpl.write_text((_TMPL / "template.html.tmpl").read_text(encoding="utf-8")
                       .replace("{{Skill名}}", skill), encoding="utf-8")
        written.append(str(tpl))

    frag = (_TMPL / "mcp.json.tmpl").read_text(encoding="utf-8")
    frag = frag.replace("{{Skill名}}", skill).replace("{{Skillの絶対パス}}", str(root.resolve()))
    (root / "mcp.json").write_text(frag, encoding="utf-8")
    written.append(str(root / "mcp.json"))
    return result(ok=True, findings=[f"既に在るので残した: {k}" for k in kept],
                  written=written, root=str(root))


def _legacy(tools_py: pathlib.Path) -> set[str]:
    """宣言が載せている、移行の互換の入口。

    **宣言に載っていない入口だけを、検査が拾う** ── 載せれば、移行の途中でも検査は通る。
    """
    if not tools_py.exists():
        return set()
    import ast
    try:
        tree = ast.parse(tools_py.read_text(encoding="utf-8"))
    except SyntaxError:
        # **雛形のままの Skill でも、検査は落ちない。** 記入前は差し込む場所が
        # 残っているので、Python として解析できない
        return set()
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(
                getattr(t, "id", "") == "LEGACY" for t in node.targets):
            try:
                return set(ast.literal_eval(node.value))
            except ValueError:
                return set()
    return set()


def _human_scaffold(res: dict) -> str:
    d = res["data"]
    lines = [f"置いた: {p}" for p in d["written"]] + res["findings"]
    lines.append("登録は mcp.json の断片を、ホストの設定へ差し込む ── "
                 "実装が無い環境では、CLI だけが動く")
    return "\n".join(lines)


# 入口の側に置いてよいもの。**これ以外は、部品か検証である**
ENTRY = ("contract.py", "tools.py", "cli.py", "mcp_server.py")
LIB, TESTS = "lib", "tests"


def _unparsable(scripts: pathlib.Path) -> list[str]:
    """Python として解析できない入口を並べる。**雛形のままなら、それも検出である。**"""
    import ast
    out = []
    for name in ENTRY:
        f = scripts / name
        if not f.exists():
            continue
        try:
            ast.parse(f.read_text(encoding="utf-8"))
        except SyntaxError:
            out.append(f"Python として解析できない: scripts/{name} ── 差し込む場所が残っている")
    return out


def check(path: str) -> dict:
    """契約を満たしているかを検査する。**見つけるが、直さない。**

    検査するのは2つである ── 入口が1つであることと、**置き場所が役割と一致すること**。
    役割ごとに階層が分かれていないと、どれが入口でどれが部品かを、
    中身を開かないと判定できない。
    """
    root = pathlib.Path(path)
    scripts = root / "scripts"
    findings = []
    for need in ENTRY:
        if not (scripts / need).exists():
            findings.append(f"入口の部品が無い: scripts/{need}")
    legacy = _legacy(scripts / "tools.py")
    findings += _unparsable(scripts)

    for f in sorted(scripts.glob("*.py")) if scripts.exists() else []:
        if f.name in ENTRY or f.name in legacy:
            continue
        kind = "検証" if f.name.startswith("test_") else "部品"
        place = TESTS if kind == "検証" else LIB
        findings.append(f"{kind}が入口の側に在る: scripts/{f.name} ── scripts/{place}/ へ移す")

    lib = scripts / LIB
    if lib.is_dir():
        if not (lib / "__init__.py").exists():
            findings.append(f"部品の包みが無い: scripts/{LIB}/__init__.py")
        for f in sorted(lib.rglob("*.py")):
            body = f.read_text(encoding="utf-8")
            if "__main__" in body or "sys.argv" in body:
                findings.append("部品が入口を保持している: "
                                f"scripts/{f.relative_to(scripts)} ── 入口は cli.py だけである")

    for f in sorted(scripts.rglob("test_*.py")) if scripts.exists() else []:
        if f.parent.name != TESTS:
            findings.append(f"検証が scripts/{TESTS}/ の外に在る: scripts/{f.relative_to(scripts)}")
    return result(ok=True, findings=findings, root=str(root))


def _human_check(res: dict) -> str:
    n = len(res["findings"])
    if not n:
        return "契約を満たしている ── 入口は1つ、道具は宣言の中に在る"
    return "\n".join([f"契約に合わない箇所　{n} 件"] + [f"  ・{x}" for x in res["findings"]])


TOOLS = [
    Tool(name="scaffold", summary="道具の契約一式を、Skill のフォルダへ置く",
         args=[Arg("skill", "Skill の名前"), Arg("path", "置き場所", required=False,
                                                default=".claude/skills")],
         run=scaffold, human=_human_scaffold),
    Tool(name="check", summary="契約を満たしているかを検査する",
         args=[Arg("path", "Skill のフォルダ")], run=check, human=_human_check),
]
