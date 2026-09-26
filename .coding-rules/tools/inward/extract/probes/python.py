# SPDX-License-Identifier: MIT
"""参照と抜け道を、標準の構文木から取る。**この言語自身の解析器である。**

    python3 python.py <根> <ファイル…>

辺は `{"from": 点, "to": 点, "at": 場所, "how": 書き方}` で出す。
相対の参照は、書かれた位置から解決して点の名前へ直す ── **解決しないと、同じ名前が
別の層を指す。**

**抜け道は、別名で束縛したものまで追う** ── `from importlib import import_module as im`
のあとの `im("x")` は、名前だけ見ると素通りする（実測で取りこぼした）。
"""
import ast
import json
import pathlib
import sys

BUILTINS = frozenset({"exec", "eval", "compile", "__import__"})
"""組み込みの名前。**修飾なしで呼ばれるものだけを見る** ── `re.compile` は別物である。"""

MODULE_FUNCS = frozenset({
    "import_module", "run_path", "run_module", "spec_from_file_location",
    "SourceFileLoader", "module_from_spec", "load_module", "find_module", "reload",
})
"""包みの中の名前。修飾されていてもよい。"""

DYNAMIC = BUILTINS | MODULE_FUNCS
"""図に現れない読み込みを起こす名前。**その言語の標準の面だけを並べる。**"""

FROM_MODULES = ("importlib", "runpy", "imp", "pkgutil")
"""この包みから束縛した名前は、別名でも追う。"""


def dotted(root: pathlib.Path, path: pathlib.Path) -> tuple[str, bool]:
    """点の名前と、それが包みかを返す。"""
    rel = path.relative_to(root).with_suffix("")
    parts = list(rel.parts)
    is_package = bool(parts) and parts[-1] == "__init__"
    if is_package:
        parts.pop()
    return ".".join(parts), is_package


def resolve(here: str, is_package: bool, level: int, module: str | None) -> str:
    """相対の参照を、点の名前へ直す。

    **level 1 は書かれた側の包みである。** 包みの中の `__init__` なら自分自身が包みで、
    module なら1つ上が包みである ── ここを取り違えると、層を越える参照が層の中に見える。
    """
    parts = here.split(".") if here else []
    if not is_package and parts:
        parts.pop()                      # module の包みは1つ上である
    up = level - 1
    keep = parts[: len(parts) - up] if up > 0 else parts
    return ".".join([x for x in (*keep, module) if x])


def dynamic_names(tree: ast.Module) -> set[str]:
    """図に現れない読み込みを起こす名前を、別名まで含めて集める。"""
    names = set(DYNAMIC)
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom):
            head = (node.module or "").split(".")[0]
            if head in FROM_MODULES:
                for a in node.names:
                    if a.name in DYNAMIC:
                        names.add(a.asname or a.name)
        elif isinstance(node, ast.Import):
            for a in node.names:
                if a.name.split(".")[0] in FROM_MODULES and a.asname:
                    names.add(a.asname)   # 包みの別名。呼び出しは <別名>.<名前> で出る
    return names


def escaping(node: ast.Call, alias: set[str]) -> str:
    """図に現れない読み込みなら、その名前を返す。そうでなければ空を返す。

    **組み込みは修飾なしのときだけ数える** ── `re.compile` を拾うと、正規表現を
    組む箇所が全部違反になる（実測で14件の誤検出）。
    """
    func = node.func
    if isinstance(func, ast.Name):
        name = func.id
        if name in BUILTINS or name in alias or name in MODULE_FUNCS:
            return name
        return ""
    if isinstance(func, ast.Attribute):
        name = func.attr
        head = ast.unparse(func).split(".")[0]
        if name in MODULE_FUNCS or (name in DYNAMIC and head in alias):
            return name
        return ""
    return ""


def main(argv: list[str]) -> int:
    if not argv:
        print("根を渡していない", file=sys.stderr)
        return 2
    root = pathlib.Path(argv[0]).resolve()
    edges, escapes, undecided = [], [], []
    for raw in argv[1:]:
        path = pathlib.Path(raw).resolve()
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"))
        except (SyntaxError, OSError, UnicodeDecodeError) as e:
            undecided.append(f"{raw} ── {e}")
            continue
        here, is_package = dotted(root, path)
        dynamic = dynamic_names(tree)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for a in node.names:
                    edges.append({"from": here, "to": a.name,
                                  "at": f"{raw}:{node.lineno}", "how": "import"})
            elif isinstance(node, ast.ImportFrom):
                to = (resolve(here, is_package, node.level, node.module) if node.level
                      else (node.module or ""))
                edges.append({"from": here, "to": to,
                              "at": f"{raw}:{node.lineno}", "how": "from"})
                for a in node.names:
                    # **包みから点を取り出す書き方も、点への参照である**
                    edges.append({"from": here, "to": f"{to}.{a.name}" if to else a.name,
                                  "at": f"{raw}:{node.lineno}", "how": "from-name"})
            elif isinstance(node, ast.Call):
                if name := escaping(node, dynamic):
                    escapes.append({"in": here, "at": f"{raw}:{node.lineno}",
                                    "how": f"図に現れない読み込み ── {name}"})
    print(json.dumps({"edges": edges, "escapes": escapes, "undecided": undecided},
                     ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
