"""値の出どころを見る検査 ── 生の数値がコードに残っていないかを機械が探す。

幾何検査は「重なっているか」しか見ない。生の数値で置いた寸法でも、たまたま
重ならなければ緑になる。だから寸法の出どころは、規律だけが支えていた ──
そして実際に、量系の部品だけで9箇所の生値が見逃されたまま残っていた。

採用済みの規律では、コードに現れる数値は3つに分かれる。

    1. 設計上の選択（余白・線幅・書体の大きさ）  → トークンから注入する
    2. データから決まる量（文字幅・箱の大きさ）  → 毎回計算する
    3. 勘で置いた閾値                            → 存在してはいけない

この検査は3を探す。1と2はコードに数値として現れないので、**現れた数値は
原則すべて疑わしい**という立場を取る。

疑わないのは、次のどれかに当てはまるものだけである。どれも「誰かが選んだ値」では
なく、構造・数学・仕様が決めているか、あるいは既に名前を持っている。

    - 0・1・2・半分(0.5)・添字・角度(90/180/270/360)・割合(100)
    - 数の大小ではなく個数を確かめる比較（`len(points) >= 3` のような構造の条件）
    - 対応表そのもの（鍵から値を引く辞書。数はその表の中身であって、式の中に
      隠れた選択ではない）
    - 数値誤差の許容値
    - モジュールの直下で名前に束ねられた数（名前が付いている時点で、それは
      宣言であって式の中の magic number ではない）

誤検出する検査は無いより悪い ── 狼少年になって本物を見逃す。だから除外は
「たまたま鳴ってほしくない」ではなく、規則として書ける形にする。

使い方:
    python3 -m svg_engine.lint_values          # 一覧を出す
    from svg_engine.lint_values import findings  # 検査として使う
"""
from __future__ import annotations

import ast
import pathlib

# 構造上どうしても現れる数。これ以外の数値は報告する。
# 0.5 は「半分」── 2 で割るのと同じことを掛け算で書いただけで、選んだ値ではない。
_STRUCTURAL = {0, 1, 2, -1, -2, 0.5, -0.5}
# 角度と割合。座標系そのものが決めている数で、選んだ値ではない。
_GEOMETRIC = {90, 180, 270, 360, 100}


def _has_power(node: ast.AST) -> bool:
    """式のどこかに、整数のべき乗があるか。多項式の目印にする。"""
    return any(isinstance(n, ast.BinOp) and isinstance(n.op, ast.Pow)
               and isinstance(n.right, ast.Constant) and isinstance(n.right.value, int)
               and n.right.value >= 2
               for n in ast.walk(node))


class _Finder(ast.NodeVisitor):
    """関数ごとに、生の数値を集める。"""

    def __init__(self, src: str):
        self.lines = src.splitlines()
        self.func: list[str] = []
        self.found: list[tuple[int, str, str]] = []

    def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
        self.func.append(node.name)
        self.generic_visit(node)
        self.func.pop()

    def visit_Compare(self, node: ast.Compare) -> None:
        """個数を確かめる比較は構造の条件。`len(points) >= 3` の 3 は選んだ値ではない。"""
        parts = [node.left, *node.comparators]
        counting = any(isinstance(p, ast.Call) and getattr(p.func, "id", "") == "len"
                       for p in parts)
        for p in parts:
            if counting and isinstance(p, ast.Constant):
                continue
            self.visit(p)

    def visit_Dict(self, node: ast.Dict) -> None:
        """対応表の中身は、式の中に隠れた選択ではない。

        SVGの命令ごとの引数の個数のような、仕様が決めている表がこれにあたる。
        鍵がすべて定数の辞書だけを表とみなす ── その場で組み立てる辞書は除く。
        """
        if node.keys and all(isinstance(k, ast.Constant) for k in node.keys):
            return
        self.generic_visit(node)

    def visit_Module(self, node: ast.Module) -> None:
        """モジュール直下で名前に束ねた数は、宣言なので見ない。

        名前が付いている時点で「誰かが選んだ値」であることが読める。式の中に
        裸で現れることを禁じているのであって、名前を与えることまで禁じない。
        """
        for stmt in node.body:
            if isinstance(stmt, (ast.Assign, ast.AnnAssign)):
                continue
            self.visit(stmt)

    def visit_Subscript(self, node: ast.Subscript) -> None:
        # 添字（points[-1] など）の数は構造。中身だけ辿る
        self.visit(node.value)

    def visit_Call(self, node: ast.Call) -> None:
        name = getattr(node.func, "id", "")
        if name == "range":
            for a in node.args:
                if not isinstance(a, ast.Constant):
                    self.visit(a)
            return
        self.generic_visit(node)

    def visit_BinOp(self, node: ast.BinOp) -> None:
        # 平方根（** 0.5）は数学の書き方であって、選んだ値ではない
        if (isinstance(node.op, ast.Pow) and isinstance(node.right, ast.Constant)
                and node.right.value == 0.5):
            self.visit(node.left)
            return
        # べき乗を含む式は多項式。その中の整数は係数と指数であって、
        # 誰かが選んだ寸法ではない（ベジエ曲線の 3 が典型）。
        if _has_power(node):
            for sub in ast.walk(node):
                if isinstance(sub, ast.Constant) and isinstance(sub.value, float):
                    self.visit_Constant(sub)   # 小数はなお疑う
            return
        self.generic_visit(node)

    def visit_Constant(self, node: ast.Constant) -> None:
        v = node.value
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            return
        if v in _STRUCTURAL or v in _GEOMETRIC:
            return
        if isinstance(v, float) and 0 < abs(v) < 1e-6:
            return   # 数値誤差の許容値。座標の設計ではない
        line = self.lines[node.lineno - 1].strip() if node.lineno <= len(self.lines) else ""
        self.found.append((node.lineno, self.func[-1] if self.func else "(直下)", line))


def findings(root: pathlib.Path | None = None) -> list[tuple[str, int, str, str]]:
    """エンジンの各ファイルから、生の数値を探す。

    Args:
        root: 探すディレクトリ。省略するとこのパッケージ自身。

    Returns:
        (ファイル名, 行, 関数名, その行) の並び。行の昇順。

    Raises:
        なし。
    """
    root = root or pathlib.Path(__file__).parent
    out: list[tuple[str, int, str, str]] = []
    for path in sorted(root.glob("*.py")):
        if path.name in ("lint_values.py", "tokens.py", "__init__.py"):
            continue  # トークンの定義そのものは、値が書かれている場所である
        src = path.read_text(encoding="utf-8")
        f = _Finder(src)
        f.visit(ast.parse(src))
        for lineno, func, line in f.found:
            out.append((path.name, lineno, func, line))
    return out


def main() -> None:
    """一覧を印字する。

    Returns:
        なし。

    Raises:
        なし。
    """
    rows = findings()
    by_file: dict[str, int] = {}
    for name, _, _, _ in rows:
        by_file[name] = by_file.get(name, 0) + 1
    print(f"生の数値: {len(rows)} 箇所")
    for name, n in sorted(by_file.items(), key=lambda kv: -kv[1]):
        print(f"  {name:<26}{n:>3}")
    print()
    for name, lineno, func, line in rows:
        print(f"{name}:{lineno} [{func}] {line[:88]}")


if __name__ == "__main__":
    main()
