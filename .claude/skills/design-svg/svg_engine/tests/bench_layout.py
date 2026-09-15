"""層状配置の物差し ── 自前(sugiyama.py)と本家(Graphviz dot)を同じ宣言で測る。

測るのは4つ。どれも「良し悪し」ではなく「観測値」で、判断はこの外で行う。

1. 交差の数   ── 辺どうしが何回すれ違うか。並びの段(order)の出来を映す。
2. 辺の長さ   ── 通る点列の総延長。段の割り当て(rank)の出来を映す。
                 network-simplex が最小化しようとしているのがこれ。
3. 場所の大きさ ── 面積と縦横比。帯に伸びていないか。
4. 揺れ       ── 同じグラフを、入力の並び順だけ変えて解き直したとき、
                 節点がどれだけ動くか。これが「安定性」の正体で、
                 本家が入っていなくても自前だけで測れる。
"""
from __future__ import annotations

import itertools
import random
import subprocess
import sys
from pathlib import Path

# 自分の親（Skill 直下）を通す ── パッケージとして入れずに直接走らせるため
sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from svg_engine.sugiyama import LayoutResult, layout_graph  # noqa: E402

NODE = (90.0, 40.0)
GAP_RANK, GAP_ORDER = 60.0, 30.0


# ── 測る道具 ──────────────────────────────────────────────

def _crosses(p, q, r, s) -> bool:
    """線分 pq と rs が、端点を共有せずに交わるか。"""
    if len({p, q} & {r, s}) or len({p, q, r, s}) < 4:
        return False

    def side(a, b, c):
        v = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
        return (v > 1e-9) - (v < -1e-9)

    d1, d2 = side(p, q, r), side(p, q, s)
    d3, d4 = side(r, s, p), side(r, s, q)
    return d1 * d2 < 0 and d3 * d4 < 0


def crossings(paths: dict[int, list[tuple[float, float]]]) -> int:
    segs = [(pt[i], pt[i + 1]) for pt in paths.values() for i in range(len(pt) - 1)]
    return sum(1 for a, b in itertools.combinations(segs, 2) if _crosses(*a, *b))


def edge_length(paths) -> float:
    return sum(((b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2) ** 0.5
               for pt in paths.values() for a, b in zip(pt, pt[1:]))


def measure(res) -> dict:
    w, h = max(res.width, 1e-9), max(res.height, 1e-9)
    return {"交差": crossings(res.edge_paths),
            "辺の長さ": round(edge_length(res.edge_paths)),
            "面積": round(w * h / 1000),
            "縦横比": round(max(w, h) / min(w, h), 1)}


def nudge(nodes, edges) -> dict:
    """節点を1つ足して、元から居た節点がどれだけ動くかを測る。

    実務で効く安定性はこちら。図を少し直すたびに全体が組み替わると、
    読み手は前回どこに何があったかを覚えていられない。足した1つの周りだけが
    動くのが望ましい。並び順への感度(wobble)と違い、こちらに「順を尊重して
    いるだけ」という言い訳は立たない ── 宣言は足した分しか変わっていない。
    """
    sizes = {n: NODE for n in nodes}
    base = layout_graph(dict(sizes), list(edges), GAP_RANK, GAP_ORDER)
    target = edges[len(edges) // 2][0]
    grown = dict(sizes) | {"新": NODE}
    after = layout_graph(grown, list(edges) + [(target, "新")], GAP_RANK, GAP_ORDER)
    moved = {k: abs(after.positions[k][0] - base.positions[k][0])
                + abs(after.positions[k][1] - base.positions[k][1]) for k in base.positions}
    return {"最も動いた既存節点": round(max(moved.values())),
            "動いた既存節点": f"{sum(1 for v in moved.values() if v > 0.5)}/{len(moved)}",
            "外形の変化": f"{round(base.width)}x{round(base.height)}→{round(after.width)}x{round(after.height)}"}


def wobble(nodes, edges, rounds=8, seed=0) -> dict:
    """入力の並び順だけを変えて解き直し、節点がどれだけ動くかを測る。

    宣言は同じで、辞書に入れた順だけが違う。読み手から見て同じ図なので、
    動かないのが正しい。動くなら、結果が並び順という無関係な事情に依存している。
    """
    rng = random.Random(seed)
    base = layout_graph(dict(nodes), list(edges), GAP_RANK, GAP_ORDER)
    moves, shapes = [], []
    for _ in range(rounds):
        ns = list(nodes.items())
        rng.shuffle(ns)
        es = list(edges)
        rng.shuffle(es)
        r = layout_graph(dict(ns), es, GAP_RANK, GAP_ORDER)
        moves.append(max(abs(r.positions[k][0] - base.positions[k][0])
                         + abs(r.positions[k][1] - base.positions[k][1]) for k in base.positions))
        shapes.append((round(r.width), round(r.height)))
    return {"最も動いた節点": round(max(moves)),
            "全く動かない回": sum(1 for m in moves if m < 0.5),
            "外形の種類": len(set(shapes))}


# ── 本家へ渡す ────────────────────────────────────────────

def to_dot(name, nodes, edges) -> str:
    lines = [f"digraph {name} {{", "  graph [rankdir=TB];",
             f"  node [shape=box fixedsize=true width={NODE[0]/72:.4f} height={NODE[1]/72:.4f}];",
             f"  graph [nodesep={GAP_ORDER/72:.4f} ranksep={GAP_RANK/72:.4f}];"]
    lines += [f'  "{n}";' for n in nodes]
    lines += [f'  "{a}" -> "{b}";' for a, b in edges]
    return "\n".join(lines + ["}"])


def run_dot(src: str) -> dict | None:
    """dot に解かせて、同じ物差しで測れる形へ読み替える。

    -Tplain は「node 名 x y 幅 高さ …」「edge 尾 頭 点数 x y x y …」を吐く。
    単位はインチなので、宣言に使った px へ戻す。

    本家は節点の縁から縁へ、自前は中心から中心へ辺を返す。そのまま長さを
    比べると、辺1本につき節点の高さのぶんだけ自前が長く出てしまう（実測：
    木の36辺で1440px、これだけで縦の差のほぼ全部）。両端を節点の中心へ
    置き換えて、同じ土俵に載せる。
    """
    try:
        out = subprocess.run(["dot", "-Tplain"], input=src, capture_output=True,
                             text=True, check=True).stdout
    except (FileNotFoundError, subprocess.CalledProcessError):
        return None
    positions: dict[str, tuple[float, float]] = {}
    paths: dict[int, list[tuple[float, float]]] = {}
    ends: list[tuple[str, str]] = []
    w = h = 0.0
    for line in out.splitlines():
        f = line.split()
        if f[0] == "graph":
            w, h = float(f[2]) * 72, float(f[3]) * 72
        elif f[0] == "node":
            positions[f[1].strip('"')] = (float(f[2]) * 72, float(f[3]) * 72)
        elif f[0] == "edge":
            n = int(f[3])
            pts = [(float(f[4 + 2 * i]) * 72, float(f[5 + 2 * i]) * 72) for i in range(n)]
            ends.append((f[1].strip('"'), f[2].strip('"')))
            paths[len(paths)] = pts
    for k, (tail, head) in enumerate(ends):
        if tail in positions and head in positions:
            paths[k] = [positions[tail]] + paths[k][1:-1] + [positions[head]]
    return {"positions": positions, "edge_paths": paths, "width": w, "height": h}


# ── 測る対象 ──────────────────────────────────────────────

def tree(width=12, depth=2):
    nodes, edges = ["根"], []
    for i in range(width):
        c = f"枝{i}"
        nodes.append(c)
        edges.append(("根", c))
        for j in range(depth):
            g = f"葉{i}_{j}"
            nodes.append(g)
            edges.append((c, g))
    return nodes, edges


def skipping(n=8):
    nodes = [f"段{i}" for i in range(n)]
    edges = [(nodes[i], nodes[i + 1]) for i in range(n - 1)]
    edges += [(nodes[0], nodes[n - 1]), (nodes[1], nodes[5]), (nodes[2], nodes[7])]
    return nodes, edges


def tangled(a=5, b=5):
    left = [f"左{i}" for i in range(a)]
    right = [f"右{j}" for j in range(b)]
    edges = [(l, r) for i, l in enumerate(left) for j, r in enumerate(right) if (i + j) % 2 == 0]
    return left + right, edges


def cyclic():
    nodes = [f"環{i}" for i in range(6)] + ["入口", "出口"]
    edges = [(f"環{i}", f"環{(i + 1) % 6}") for i in range(6)]
    edges += [("入口", "環0"), ("環3", "出口"), ("入口", "環4")]
    return nodes, edges


def uneven():
    """段が構造から決まらない図 ── 長さの違う道が1点へ合流する。

    他の4案件はどれも段が一意に決まってしまい、段の割り当ての出来を測れない。
    ここでは短い道の起点をどこへ置くかに自由があり、上端へ置くと辺が伸びる。
    """
    edges = [("長1", "長2"), ("長2", "長3"), ("長3", "長4"), ("長4", "合流"),
             ("短1", "短2"), ("短2", "合流"),
             ("中1", "中2"), ("中2", "中3"), ("中3", "合流"),
             ("合流", "出口")]
    nodes = ["長1", "長2", "長3", "長4", "短1", "短2", "中1", "中2", "中3", "合流", "出口"]
    return nodes, edges


CASES = {"枝の多い木": tree(), "多段をまたぐ辺": skipping(),
         "交差の多いグラフ": tangled(), "サイクルを含む": cyclic(),
         "段が決まらない図": uneven()}


def main():
    header = ("案件", "手", "交差", "辺の長さ", "面積", "縦横比")
    print("{:<16}{:<8}{:>6}{:>10}{:>8}{:>8}".format(*header))
    print("-" * 56)
    for name, (nodes, edges) in CASES.items():
        sizes = {n: NODE for n in nodes}
        mine = measure(layout_graph(dict(sizes), list(edges), GAP_RANK, GAP_ORDER))
        print("{:<16}{:<8}{:>6}{:>10}{:>8}{:>8}".format(
            name, "自前", mine["交差"], mine["辺の長さ"], mine["面積"], mine["縦横比"]))

        got = run_dot(to_dot("g", nodes, edges))
        if got:
            theirs = measure(LayoutResult({}, got["edge_paths"], got["width"], got["height"]))
            print("{:<16}{:<8}{:>6}{:>10}{:>8}{:>8}".format(
                "", "本家", theirs["交差"], theirs["辺の長さ"], theirs["面積"], theirs["縦横比"]))
        else:
            print("{:<16}{:<8}{:>6}".format("", "本家", "未導入"))

    print("\n揺れ ── 宣言は同じで、入力の並び順だけを変えて8回解き直す")
    print("{:<16}{:>16}{:>16}{:>12}".format("案件", "最も動いた節点", "全く動かない回", "外形の種類"))
    print("-" * 60)
    for name, (nodes, edges) in CASES.items():
        w = wobble({n: NODE for n in nodes}, edges)
        print("{:<16}{:>16}{:>16}{:>12}".format(
            name, w["最も動いた節点"], f'{w["全く動かない回"]}/8', w["外形の種類"]))

    print("\n継ぎ足し ── 節点を1つ足したとき、元から居た節点がどれだけ動くか")
    print("{:<16}{:>18}{:>16}{:>22}".format("案件", "最も動いた既存節点", "動いた既存節点", "外形の変化"))
    print("-" * 72)
    for name, (nodes, edges) in CASES.items():
        n = nudge(nodes, edges)
        print("{:<16}{:>18}{:>16}{:>22}".format(
            name, n["最も動いた既存節点"], n["動いた既存節点"], n["外形の変化"]))


if __name__ == "__main__":
    main()
