"""層状グラフ描画 ── Graphvizの`dot`と同じ系統（Sugiyama法）の、本格版。

`layout.py`の簡易版に欠けていた4点を埋める:

1. サイクルの分断 ── DFSで逆流する辺を見つけ、段の計算時だけ向きを仮に反転する
2. 複数段をまたぐ辺の経路 ── またぐ段の分だけ「仮の節点」を挟み、隣接段だけを結ぶ
   単位辺の鎖にする。これが無いと、長い辺が中間の節点を突っ切って描かれる
3. 交差の最小化 ── 中央値法と転置法を、上り下り交互に何度も反復し、
   交差の数が一番少なかった並びを採用する
4. 座標の整列 ── 親の位置へ寄せる中央値ヒューリスティックを、下り・上り
   両方向で繰り返し平均する。1回ごとの詰め直しは近似ではなく厳密で、
   並びを保ったまま望んだ位置との差の総和を最小にする配置を等調回帰で解く。
   最後に、辺で繋がっていない塊どうしを詰める（目的から決まらない自由度は、
   詰める方に倒す）
5. 段の割り当て ── 本家と同じ網状単体法で、辺の長さの総和を最小にする

本家 Graphviz `dot` と同じ図で突き合わせた結果（tests/bench_layout.py）:
交差・辺の長さ・面積・縦横比のいずれもほぼ互角で、交差の多い図と段が決まらない
図は完全に一致する。多段をまたぐ図は自前のほうが短い。残る差は、交差の数が
2案件で1本多いこと。

比べるときは、本家が節点の縁から縁へ、自前が中心から中心へ辺を返すことに注意
する。揃えずに測ると、辺1本につき節点の高さのぶんだけ自前が長く出る（実測：
木の36辺で1440px、これだけで縦の差のほぼ全部を説明してしまう）。物差しの側で
両端を中心へ揃えてある。

どの段も、宣言（nodes/edges）を型として持つだけで、呼ぶ側の語彙は知らない。
"""
from __future__ import annotations

from dataclasses import dataclass, field

from .geometry import shift_to_origin
from .layout_contract import LayoutResult

# 入れ替えの空回りへの保険。切り値が負の辺を入れ替えるたびに解は改善するので
# 本来は止まるが、浮動小数の丸めで振動した場合に備えて上限を置く。辺の数に
# 比例させ、小さなグラフでも回数が確保されるよう下駄を履かせる。
_SWAP_LIMIT_PER_EDGE = 4
_SWAP_LIMIT_BASE = 16


# ── 1. サイクルの分断 ──────────────────────────────────────

def _break_cycles(nodes: list[str], edges: list[tuple[str, str]]) -> list[tuple[str, str, bool]]:
    """DFSで逆流する辺(後退辺)を見つけ、(from, to, reversed)の並びを返す。

    reversed=True の辺は、段の計算では (to, from) として扱う。実際の矢印の
    向きは元のまま (from→to) で描くので、呼び出し側はここのreversedを見て
    経路の点列を反転させ直す。
    """
    adj: dict[str, list[str]] = {n: [] for n in nodes}
    for a, b in edges:
        if a in adj and b in adj:
            adj[a].append(b)

    WHITE, GRAY, BLACK = 0, 1, 2
    out: list[tuple[str, str, bool]] = []

    # 明示的スタックで深さ優先を辿る（再帰深度の上限を気にせずに済むように）。
    state = {n: WHITE for n in nodes}
    for start in nodes:
        if state[start] != WHITE:
            continue
        stack = [(start, iter(adj[start]))]
        state[start] = GRAY
        while stack:
            u, it = stack[-1]
            advanced = False
            for v in it:
                if v not in state:
                    continue
                if state[v] == WHITE:
                    state[v] = GRAY
                    stack.append((v, iter(adj[v])))
                    advanced = True
                    break
                elif state[v] == GRAY:
                    out.append((v, u, True))  # 後退辺。段の計算では逆向きに使う
                else:
                    out.append((u, v, False))
            if not advanced:
                state[u] = BLACK
                stack.pop()
    # 上のループは「u→v」を辿るたびに1本ずつoutへ積むが、同じ(u,v)を後退辺
    # 判定のときと通常時とで二重に積まないよう、辺の集合から作り直す。
    result = []
    seen: set = set()
    rank_edges: dict[tuple[str, str], bool] = {}
    for a, b, rev in out:
        rank_edges[(a, b) if not rev else (b, a)] = rev
    for a, b in edges:
        if a not in adj or b not in adj:
            continue
        key = (a, b)
        # DFSで辿らなかった辺（多重辺等）は非後退として扱う
        rev = rank_edges.get(key, False)
        result.append((a, b, rev))
    return result


# ── 2. 段の割り当て ── 辺の長さの総和を最小にする（網状単体法） ────────

def _longest_path_ranks(nodes: list[str], edges: list[tuple[str, str]]) -> dict[str, int]:
    """どの辺も1段以上またぐ、いちばん素朴な割り当て。単体法の出発点にする。"""
    rank = {n: 0 for n in nodes}
    for _ in range(len(nodes) + 1):
        changed = False
        for a, b in edges:
            if rank[b] < rank[a] + 1:
                rank[b] = rank[a] + 1
                changed = True
        if not changed:
            break
    return rank


def _tight_tree(nodes, edges, rank, root):
    """ちょうど1段だけまたぐ辺（たるみ0）だけを辿って広がれるだけ広げる。

    Returns: (届いた節点の集合, 使った辺の添字の集合)
    """
    reached, used = {root}, set()
    grew = True
    while grew:
        grew = False
        for i, (a, b) in enumerate(edges):
            if i in used or rank[b] - rank[a] != 1:
                continue
            if (a in reached) != (b in reached):
                reached.add(a)
                reached.add(b)
                used.add(i)
                grew = True
    return reached, used


def _ranks_from_tree(comp_nodes, edges, tree, root):
    """木の辺がすべてちょうど1段になるように、段を振り直す。"""
    adj: dict[str, list[tuple[str, int]]] = {n: [] for n in comp_nodes}
    for i in tree:
        a, b = edges[i]
        adj[a].append((b, 1))
        adj[b].append((a, -1))
    rank = {root: 0}
    stack = [root]
    while stack:
        n = stack.pop()
        for m, step in adj[n]:
            if m not in rank:
                rank[m] = rank[n] + step
                stack.append(m)
    return rank


def _tail_side(comp_nodes, edges, tree, leaving):
    """木から1本抜いたとき、その辺の根元側に残る節点の集合。"""
    adj: dict[str, list[str]] = {n: [] for n in comp_nodes}
    for i in tree:
        if i == leaving:
            continue
        a, b = edges[i]
        adj[a].append(b)
        adj[b].append(a)
    start = edges[leaving][0]
    side, stack = {start}, [start]
    while stack:
        n = stack.pop()
        for m in adj[n]:
            if m not in side:
                side.add(m)
                stack.append(m)
    return side


def _network_simplex(comp_nodes: list[str],
                     edges: list[tuple[str, str]]) -> dict[str, int]:
    """辺の長さの総和が最小になる段の割り当てを解く。

    最長経路法は「どの辺も1段以上またぐ」を満たすだけで、長さは気にしない。
    その結果、下げても誰も困らない節点が下がったままになり、辺が伸びる。
    以前はここに「出て行く辺を持たない節点を子の1つ上まで引き上げる」という
    後始末を足していたが、それは1手先しか見ないので、間に節点が挟まると効かない。

    最小化そのものを解く。制約（どの辺も1段以上）を張った線形計画の双対は、
    たるみ0の辺だけで作った全域木の上を渡り歩く問題になる。木の辺を1本抜くと
    グラフが2つに割れ、その切り口を跨ぐ辺の重みの差（切り値）が負なら、その辺を
    別の辺と入れ替えると総延長が減る。負が無くなったら最適。

    段数（縦の長さ）は増えない ── 制約は最長経路法と同じで、その中で最短を選ぶだけ。
    """
    if not comp_nodes:
        return {}
    rank = _longest_path_ranks(comp_nodes, edges)
    root = comp_nodes[0]

    # たるみ0の辺だけでは全体に届かないうちは、いちばんたるみの小さい辺の分だけ
    # 木ごと動かして、その辺をたるみ0にする。届くまで繰り返す。
    while True:
        reached, tree = _tight_tree(comp_nodes, edges, rank, root)
        if len(reached) == len(comp_nodes):
            break
        # 添字とたるみは必ず一緒に決まる。別々の変数に置くと「片方が None なら
        # もう片方も None」という不変条件が型から見えなくなる。
        best: tuple[int, int] | None = None
        for i, (a, b) in enumerate(edges):
            if (a in reached) == (b in reached):
                continue
            slack = rank[b] - rank[a] - 1
            if best is None or slack < best[1]:
                best = (i, slack)
        if best is None:
            break  # 繋がっていない ── 呼び出し側が塊ごとに分けている前提
        best_edge, best_slack = best
        a, _b = edges[best_edge]
        shift = best_slack if a in reached else -best_slack
        for n in reached:
            rank[n] += shift

    limit = _SWAP_LIMIT_PER_EDGE * len(edges) + _SWAP_LIMIT_BASE
    for _ in range(limit):
        leaving = None
        for i in tree:
            side = _tail_side(comp_nodes, edges, tree, i)
            cut = sum(1 if (a in side) != (b in side) and a in side else
                      -1 if (a in side) != (b in side) else 0
                      for a, b in edges)
            if cut < 0:
                leaving = (i, side)
                break
        if leaving is None:
            break
        i, side = leaving
        # 切り口を逆向きに跨ぐ辺のうち、いちばんたるみの小さいものを入れる
        # ここも添字とたるみが一緒に決まる ── 1つにまとめる
        entering: tuple[int, int] | None = None
        for j, (a, b) in enumerate(edges):
            if j in tree or a in side or b not in side:
                continue
            slack = rank[b] - rank[a] - 1
            if entering is None or slack < entering[1]:
                entering = (j, slack)
        if entering is None:
            break
        tree = (tree - {i}) | {entering[0]}
        rank = _ranks_from_tree(comp_nodes, edges, tree, root)

    low = min(rank.values())
    return {n: rank[n] - low for n in comp_nodes}


def _assign_ranks(nodes: list[str], dag_edges: list[tuple[str, str]]) -> dict[str, int]:
    """段を割り当てる。繋がっていない塊は、それぞれ独立に解く。

    網状単体法は全域木を張るので、繋がっていない塊が混ざったままだと木を張れない。
    塊は互いの段を制約しないので、分けて解いて構わない。
    """
    adj: dict[str, list[str]] = {n: [] for n in nodes}
    for a, b in dag_edges:
        adj[a].append(b)
        adj[b].append(a)
    seen: set[str] = set()
    rank: dict[str, int] = {}
    for start in nodes:
        if start in seen:
            continue
        group, stack = [], [start]
        seen.add(start)
        while stack:
            n = stack.pop()
            group.append(n)
            for m in adj[n]:
                if m not in seen:
                    seen.add(m)
                    stack.append(m)
        members = set(group)
        rank.update(_network_simplex(
            [n for n in nodes if n in members],
            [(a, b) for a, b in dag_edges if a in members]))
    return rank


# ── 3. 複数段をまたぐ辺へ仮の節点を挟む ────────────────────────

@dataclass
class ExpandedGraph:
    all_nodes: list[str]
    real: set[str]
    rank: dict[str, int]
    unit_edges: list[tuple[str, str]]
    # 元の辺(通し番号)ごとの、通る節点id列（実節点＋仮節点、rank昇順）
    chains: dict[int, list[str]]
    reversed_flags: dict[int, bool]


def _expand(nodes: list[str], edges_with_rev: list[tuple[str, str, bool]],
            rank: dict[str, int]) -> ExpandedGraph:
    all_nodes = list(nodes)
    unit_edges: list[tuple[str, str]] = []
    chains: dict[int, list[str]] = {}
    reversed_flags: dict[int, bool] = {}
    vcount = 0

    for idx, (a, b, rev) in enumerate(edges_with_rev):
        ra, rb = rank[a], rank[b]
        lo, hi = (a, b) if ra <= rb else (b, a)
        chain = [lo]
        cur = lo
        for r in range(lo_r := rank[lo] + 1, rank[hi]):
            vid = f"__v{vcount}"
            vcount += 1
            rank[vid] = r
            all_nodes.append(vid)
            unit_edges.append((cur, vid))
            chain.append(vid)
            cur = vid
        unit_edges.append((cur, hi))
        chain.append(hi)
        if lo != a:
            chain = list(reversed(chain))
        chains[idx] = chain
        reversed_flags[idx] = rev

    return ExpandedGraph(all_nodes=all_nodes, real=set(nodes), rank=rank,
                          unit_edges=unit_edges, chains=chains, reversed_flags=reversed_flags)


# ── 4. 段内の並び順 ── 中央値法＋転置法を反復し、交差が最少の結果を採る ──

def _block_of(node: str, groups: list[list[str]]) -> int:
    """その節点が属する群の番号。属さないなら -1。

    入れ子（群の中の群）は、より内側＝より後に宣言された群を優先する。
    """
    found = -1
    for i, members in enumerate(groups):
        if node in members:
            found = i
    return found


def _order_within_ranks(expanded: ExpandedGraph,
                         groups: list[list[str]] | None = None) -> dict[str, int]:
    """段内の並びを解く。

    群があるときは階層的に解く ── まず同じ群の要素をひとつの塊として扱い、
    塊どうしの並びを交差が減るよう決め、次に塊の中の並びを同じやり方で決める。
    こうすると「同じ群の要素が隣り合う」ことが構造的に保証されたうえで、
    交差の最小化が群の外と内の両方で効く。

    隣り合うことは優先度ではなく要件である ── 離れて置かれた要素を箱で囲むと、
    間に居る非メンバーまで囲んでしまい、その囲みは意味を成さない。
    """
    groups = groups or []
    by_rank: dict[int, list[str]] = {}
    for n in expanded.all_nodes:
        by_rank.setdefault(expanded.rank[n], []).append(n)

    incoming: dict[str, list[str]] = {n: [] for n in expanded.all_nodes}
    outgoing: dict[str, list[str]] = {n: [] for n in expanded.all_nodes}
    for a, b in expanded.unit_edges:
        outgoing[a].append(b)
        incoming[b].append(a)

    # 初期の並びは、宣言順ではなく辺をたどった順にする。
    #
    # 中央値法は初期の並びから山を下るだけなので、始まりが悪いと途中の谷で
    # 止まる。宣言順から始めると、互いに繋がっていない塊どうしが交互に
    # 並んだままになり、塊が互いを横切り続ける（実測：2つの塊に分かれる図で
    # 28交差。塊を分けて並べれば、塊の中だけで済んで10交差になる）。
    # 辺をたどれば、繋がっているものが自然に隣り合う。
    #
    # たどり始める節点は宣言順に選ぶ。宣言の順は読み手が意図した順であり、
    # 交差が同じなら尊重する。
    seen: set[str] = set()
    walked: list[str] = []
    for start in expanded.all_nodes:
        if start in seen:
            continue
        seen.add(start)
        queue = [start]
        while queue:
            n = queue.pop(0)
            walked.append(n)
            for m in outgoing[n] + incoming[n]:
                if m not in seen:
                    seen.add(m)
                    queue.append(m)
    rank_of_walk = {n: i for i, n in enumerate(walked)}
    order: dict[str, int] = {}
    for r in sorted(by_rank):
        by_rank[r].sort(key=lambda n: rank_of_walk[n])
        for i, n in enumerate(by_rank[r]):
            order[n] = i

    def median_pass(neighbours: dict[str, list[str]], ranks_seq):
        for r in ranks_seq:
            row = by_rank[r]

            def med(n):
                ords = sorted(order[p] for p in neighbours[n] if p in order)
                if not ords:
                    return order[n]
                m = len(ords) // 2
                if len(ords) % 2 == 1:
                    return float(ords[m])
                if len(ords) == 2:
                    return (ords[0] + ords[1]) / 2
                left = ords[m - 1] - ords[0]
                right = ords[-1] - ords[m]
                if left + right == 0:
                    return (ords[m - 1] + ords[m]) / 2
                return (ords[m - 1] * right + ords[m] * left) / (left + right)

            if groups:
                # 階層的に解く。塊（＝同じ群の要素の集まり）を単位に並べてから、
                # 塊の中をもう一度並べる。塊は分断されないので隣接が保たれる。
                blocks: dict[int, list[str]] = {}
                for n in row:
                    blocks.setdefault(_block_of(n, groups), []).append(n)
                free = [(-1, [n]) for n in blocks.pop(-1, [])]
                units = free + [(gi, members) for gi, members in blocks.items()]

                def unit_med(unit):
                    _, members = unit
                    return sum(med(m) for m in members) / len(members)

                keyed = []
                for _, members in sorted(units, key=unit_med):
                    keyed.extend(sorted(members, key=med) if len(members) > 1 else members)
            else:
                keyed = sorted(row, key=med)
            for i, n in enumerate(keyed):
                order[n] = i

    def transpose():
        """隣り合う2つを入れ替えて交差が減るなら採用する。局所改善のダメ押し。

        ただし群をまたぐ入れ替えはしない ── それをすると塊が分断され、
        階層的に解いて得た隣接が壊れる。
        """
        improved = True
        while improved:
            improved = False
            for r in sorted(by_rank):
                row = by_rank[r]
                for i in range(len(row) - 1):
                    n1, n2 = row[i], row[i + 1]
                    if groups and _block_of(n1, groups) != _block_of(n2, groups):
                        continue
                    before = _local_crossings(n1, n2, order, incoming, outgoing)
                    after = _local_crossings(n2, n1, order, incoming, outgoing)
                    if after < before:
                        row[i], row[i + 1] = n2, n1
                        order[n1], order[n2] = i + 1, i
                        improved = True

    ranks_sorted = sorted(by_rank)
    best_order = dict(order)
    best_score = _crossing_total(expanded, by_rank, order)
    # 回数は決め打ちにしない。良くならなくなったら止める ── 交差が0になるか、
    # 上りと下りを1往復しても並びが変わらなくなった（不動点に達した）ら終わり。
    # 上限は節点の数から取る。交差の入れ替えは1回につき少なくとも1組しか
    # 直さないので、節点の数を超えて改善が続くことはない（保険であって閾値ではない）。
    limit = max(len(expanded.rank), 1)
    it = 0
    while it < limit * 2 and best_score > 0:
        before_cycle = dict(order)
        for direction_pass in (0, 1):
            seq = ranks_sorted if direction_pass == 0 else list(reversed(ranks_sorted))
            median_pass(incoming if direction_pass == 0 else outgoing, seq)
            transpose()
            score = _crossing_total(expanded, by_rank, order)
            if score < best_score:
                best_score = score
                best_order = dict(order)
        if order == before_cycle:
            break            # 一往復して何も動かなかった＝これ以上良くならない
        it += 1
    return best_order


def _local_crossings(n1, n2, order, incoming, outgoing) -> int:
    """n1がn2の左（順序が小さい）に居るとして、隣接段との間で何本交差するか。"""
    c = 0
    for neigh in (incoming, outgoing):
        for p1 in neigh.get(n1, []):
            for p2 in neigh.get(n2, []):
                if order.get(p1, 0) > order.get(p2, 0):
                    c += 1
    return c


def _crossing_total(expanded: ExpandedGraph, by_rank: dict[int, list[str]],
                     order: dict[str, int]) -> int:
    total = 0
    ranks = sorted(by_rank)
    edges_by_top_rank: dict[int, list[tuple[str, str]]] = {r: [] for r in ranks}
    for a, b in expanded.unit_edges:
        ra = expanded.rank[a]
        edges_by_top_rank.setdefault(ra, []).append((a, b))
    for r in ranks[:-1]:
        pairs = [(order[a], order[b]) for a, b in edges_by_top_rank.get(r, [])]
        for i in range(len(pairs)):
            for j in range(i + 1, len(pairs)):
                (a1, b1), (a2, b2) = pairs[i], pairs[j]
                if (a1 - a2) * (b1 - b2) < 0:
                    total += 1
    return total


def _median(vals: list[float]) -> float:
    """並びの真ん中。偶数個なら中2つの平均 ── どちらへも同じだけ動けるように。"""
    s = sorted(vals)
    m = len(s) // 2
    return s[m] if len(s) % 2 else (s[m - 1] + s[m]) / 2



def _pack_components(expanded: "ExpandedGraph", by_rank: dict[int, list[str]],
                     cross: dict[str, float], size_of, gap_order: float,
                     direction: str) -> None:
    """繋がっていない塊どうしを、隣り合うまで詰める。

    塊と塊の間は、どの辺も跨いでいない。辺の長さを目的にする限り、塊の相対
    位置は目的から決まらない ── どこへ置いても目的の値は変わらない。決まらない
    ものを反復に委ねると、寄せる力が無いまま離れていく（実測：繋がっていない
    2つの塊を持つ図で、節点を1つ足すと幅が570から1290へ広がった）。

    決まらないなら詰める、と決める。塊の中の配置は一切動かさないので、
    反復が解いた結果は保たれる。

    どの段でも塊が途切れず、かつ塊どうしの前後関係が段をまたいで一致している
    ときだけ詰める。そうでない段があるなら、詰めると段の並びが壊れる。
    """
    seen: set[str] = set()
    comp: dict[str, int] = {}
    adj: dict[str, list[str]] = {n: [] for n in expanded.all_nodes}
    for a, b in expanded.unit_edges:
        adj[a].append(b)
        adj[b].append(a)
    for start in expanded.all_nodes:
        if start in seen:
            continue
        cid = len(set(comp.values()))
        seen.add(start)
        queue = [start]
        while queue:
            n = queue.pop(0)
            comp[n] = cid
            for m in adj[n]:
                if m not in seen:
                    seen.add(m)
                    queue.append(m)
    if len(set(comp.values())) < 2:
        return

    sequences = []
    for r in by_rank:
        row = sorted(by_rank[r], key=lambda n: cross[n])
        seq = [comp[row[0]]]
        for n in row[1:]:
            if comp[n] != seq[-1]:
                if comp[n] in seq:
                    return  # 塊が段の中で途切れている
                seq.append(comp[n])
        sequences.append(seq)
    order_seen: dict[int, int] = {}
    for seq in sequences:
        for i, c in enumerate(seq):
            if c in order_seen and order_seen[c] != i and len(seq) == len(sequences[0]):
                return  # 塊どうしの前後関係が段によって違う
        for i, c in enumerate(seq):
            order_seen.setdefault(c, i)

    span = lambda n: size_of(n)[0 if direction == "TB" else 1]
    extents: dict[int, tuple[float, float]] = {}
    for n, c in comp.items():
        lo, hi = cross[n] - span(n) / 2, cross[n] + span(n) / 2
        prev = extents.get(c)
        extents[c] = (min(lo, prev[0]), max(hi, prev[1])) if prev else (lo, hi)

    cursor = None
    for c in sorted(extents, key=lambda c: extents[c][0]):
        lo, hi = extents[c]
        if cursor is None:
            cursor = hi + gap_order
            continue
        shift = cursor - lo
        if shift < 0:
            for n, cc in comp.items():
                if cc == c:
                    cross[n] += shift
            lo, hi = lo + shift, hi + shift
        cursor = hi + gap_order



# ── 5. 座標の整列 ── 上下の中央値へ寄せる反復平均（簡略版） ──────────

def _assign_coordinates(expanded: ExpandedGraph, order: dict[str, int],
                         sizes: dict[str, tuple[float, float]],
                         gap_rank: float, gap_order: float,
                         direction: str, tolerance: float,
                         groups: list[list[str]] | None = None,
                         group_margin: float = 0.0) -> dict[str, tuple[float, float]]:
    groups = groups or []
    by_rank: dict[int, list[str]] = {}
    for n in expanded.all_nodes:
        by_rank.setdefault(expanded.rank[n], []).append(n)
    for r in by_rank:
        by_rank[r].sort(key=lambda n: order[n])

    def size_of(n):
        return sizes.get(n, (2.0, 2.0))

    def gap_between(prev: str, cur: str) -> float:
        """隣り合う2つの間に空ける量。群の境目だけ、枠が入るぶんを余分に空ける。

        全体の間隔を一律に広げると、群の内側まで間延びする。境目にだけ足す。
        """
        if groups and _block_of(prev, groups) != _block_of(cur, groups):
            return gap_order + group_margin
        return gap_order

    # 初期位置 ── 段内をそのまま均等配置
    cross = {}
    for r in sorted(by_rank):
        row = by_rank[r]
        cursor = 0.0
        for i, n in enumerate(row):
            w = size_of(n)[0 if direction == "TB" else 1]
            if i > 0:
                cursor += gap_between(row[i - 1], n) - gap_order
            cross[n] = cursor + w / 2
            cursor += w + gap_order

    incoming: dict[str, list[str]] = {n: [] for n in expanded.all_nodes}
    outgoing: dict[str, list[str]] = {n: [] for n in expanded.all_nodes}
    for a, b in expanded.unit_edges:
        outgoing[a].append(b)
        incoming[b].append(a)

    def resolve_overlaps(row):
        """段の並びを保ったまま、望んだ位置に最も近い座標へ詰める。

        望んだ位置（上下の隣の中央値）は隣との最小間隔を知らないので、
        そのままでは重なる。ここで直すのだが、右へ押すだけでは足りない
        ── 押す力しか無いと、一度開いた隙間が二度と閉じない（実測：
        交差の多い図に節点を1つ足しただけで、段の中に1380pxの空白が残り、
        幅が570から1974へ膨らんだ）。

        開くのと閉じるのを、1つの規則で同時に扱う。並び順を保ったまま
        Σ|座標 − 望んだ位置| を最小にする配置は、隣接の違反を塊へ併合して
        いく等調回帰（PAVA）で厳密に解ける。塊の座標は、その塊が望んだ
        位置の中央値。決め打ちの数も、押した量を平均して戻す補正も要らない
        ── 左右どちらへも同じだけ動けるので、偏り自体が起きない。
        """
        if not row:
            return
        span = [size_of(n)[0 if direction == "TB" else 1] for n in row]
        # 最小間隔を座標から抜いておくと、制約は「並びが逆転しない」だけになる
        base = [0.0]
        for i in range(1, len(row)):
            base.append(base[-1] + span[i - 1] / 2 + span[i] / 2
                        + gap_between(row[i - 1], row[i]))
        stack: list[tuple[float, list[float]]] = []
        for i, n in enumerate(row):
            block = [cross[n] - base[i]]
            centre = block[0]
            while stack and stack[-1][0] >= centre:
                prev_centre, prev_block = stack.pop()
                block = prev_block + block
                centre = _median(block)
            stack.append((centre, block))
        i = 0
        for centre, block in stack:
            for _ in block:
                cross[row[i]] = centre + base[i]
                i += 1

    ranks_sorted = sorted(by_rank)
    # 回数は決め打ちにしない。1回の往復で動いた最大の量が、描いても見えない
    # 大きさ（線の太さ）を下回ったら止める。上限は節点の数（保険）。
    limit = max(len(order), 1)
    it = -1
    moved = tolerance + 1
    while it + 1 < limit * 2 and moved > tolerance:
        it += 1
        snapshot = dict(cross)
        neigh = incoming if it % 2 == 0 else outgoing
        seq = ranks_sorted if it % 2 == 0 else list(reversed(ranks_sorted))
        for r in seq:
            row = by_rank[r]
            for n in row:
                ns = neigh.get(n, [])
                if ns:
                    cross[n] = sum(cross[p] for p in ns) / len(ns)
            # 位置で並べ替え直すと、順序の段で解いた群の隣接が壊れる。
            # 群があるときは塊ごと動かし、塊の中だけを並べ替える。
            #
            # 並べ替えを丸ごとやめて順序の段の並びを固定する案を測ったが、
            # 交差は減らず（28のまま）、サイクルを含む図で1→2に増え、
            # 枝の多い木の継ぎ足しの揺れが30→120pxへ悪化した。
            # 交差の差の出所はここではない。
            if groups:
                units: list[tuple[int, list[str]]] = []
                for n in row:
                    gi = _block_of(n, groups)
                    if units and units[-1][0] == gi and gi != -1:
                        units[-1][1].append(n)
                    else:
                        units.append((gi, [n]))
                units.sort(key=lambda u: sum(cross[m] for m in u[1]) / len(u[1]))
                row[:] = [m for _, members in units
                          for m in sorted(members, key=lambda n: cross[n])]
            else:
                row.sort(key=lambda n: cross[n])
            resolve_overlaps(row)
        moved = max((abs(cross[k] - snapshot[k]) for k in cross), default=0.0)

    _pack_components(expanded, by_rank, cross, size_of, gap_order, direction)

    main = {}
    cursor = 0.0
    for r in ranks_sorted:
        row = by_rank[r]
        extent = max(size_of(n)[1 if direction == "TB" else 0] for n in row)
        for n in row:
            main[n] = cursor
        cursor += extent + gap_rank

    positions = {}
    for n in expanded.all_nodes:
        w, h = size_of(n)
        if direction == "TB":
            positions[n] = (cross[n] - w / 2, main[n])
        else:
            positions[n] = (main[n], cross[n] - h / 2)
    return positions


# ── 公開API ────────────────────────────────────────────

def layout_graph(node_sizes: dict[str, tuple[float, float]],
                  edges: list[tuple[str, str]],
                  gap_rank: float, gap_order: float,
                  direction: str = "TB",
                  tolerance: float = 1.0,
                  groups: list[list[str]] | None = None,
                  group_margin: float = 0.0) -> LayoutResult:
    """点と辺から、実座標つきの完全なレイアウトを解く。

    サイクル・複数段をまたぐ辺・交差する辺のいずれにも耐える
    （耐える、とは＝クラッシュしない、かつ辺が節点を突っ切らないことを指す。
    Graphviz本家と全く同じ美しさになる保証はしない）。

    Args:
        node_sizes: 実節点idごとの (width, height)。
        edges: (from, to) の並び。
        gap_rank: 段と段の間隔。
        gap_order: 段内の要素どうしの間隔。
        direction: "TB" または "LR"。
        tolerance: 座標の整列を止める動きの大きさ。描いても見えない大きさ
            （線の太さ）を渡す。回数ではなく、この量で打ち切る。
        groups: 隣り合わせる識別子の集合の並び（囲みの要素）。与えると、
            各段でその要素が必ず隣り合うよう階層的に順序を解く。箱で囲む以上
            隣接は要件であり、離れて置くと非メンバーまで囲んでしまう。
        group_margin: 群の境目にだけ余分に空ける量（枠の線と余白が入る幅）。

    Returns:
        LayoutResult。edge_pathsのキーは edges の通し番号(0始まり)で、
        値はその辺が実際に通る点列（元のfrom→toの向きで、始点・終点とも
        節点の中心座標）。

    Raises:
        なし。
    """
    nodes = list(node_sizes)
    edges_with_rev = _break_cycles(nodes, edges)
    dag_edges = [((b, a) if rev else (a, b)) for a, b, rev in edges_with_rev]
    rank = _assign_ranks(nodes, dag_edges)
    expanded = _expand(nodes, edges_with_rev, dict(rank))
    order = _order_within_ranks(expanded, groups)

    sizes = dict(node_sizes)
    for n in expanded.all_nodes:
        if n not in sizes:
            sizes[n] = (1.0, 1.0)  # 仮節点は点として扱う

    positions = _assign_coordinates(expanded, order, sizes, gap_rank, gap_order,
                                     direction, tolerance, groups, group_margin)

    # 段ごとに重心へ戻すと、左（または上）へはみ出すことがある。原点を左上へ
    # 取り直す ── 呼び出し側は座標が0から始まる前提で画布を決めるため。
    positions, _ = shift_to_origin(positions)

    def centre(n):
        x, y = positions[n]
        w, h = sizes[n]
        return (x + w / 2, y + h / 2)

    # 仮節点は点（1×1）なので、その段が背の高いもの（畳んだ群など）を含むと、
    # 段の上端だけを経由して斜めに降り、間の箱を切ってしまう。仮節点では
    # 段の入口と出口の2点を出し、辺がその段の高さぶん並走するようにする。
    axis = 1 if direction == "TB" else 0
    rank_lo: dict[int, float] = {}
    rank_hi: dict[int, float] = {}
    for n in nodes:
        r = expanded.rank[n]
        lo = positions[n][axis]
        hi = lo + sizes[n][1 if axis == 1 else 0]
        rank_lo[r] = min(rank_lo.get(r, lo), lo)
        rank_hi[r] = max(rank_hi.get(r, hi), hi)

    edge_paths: dict[int, list[tuple[float, float]]] = {}
    for idx, chain in expanded.chains.items():
        pts: list[tuple[float, float]] = []
        for n in chain:
            if n in expanded.real:
                pts.append(centre(n))
                continue
            r = expanded.rank[n]
            cx, cy = centre(n)
            lo = rank_lo.get(r, cy if axis == 1 else cx)
            hi = rank_hi.get(r, cy if axis == 1 else cx)
            if axis == 1:
                pts.append((cx, lo))
                pts.append((cx, hi))
            else:
                pts.append((lo, cy))
                pts.append((hi, cy))
        edge_paths[idx] = pts

    real_positions = {n: positions[n] for n in nodes}
    max_x = max((positions[n][0] + sizes[n][0] for n in expanded.all_nodes), default=0)
    max_y = max((positions[n][1] + sizes[n][1] for n in expanded.all_nodes), default=0)
    return LayoutResult(positions=real_positions, edge_paths=edge_paths,
                         width=max_x, height=max_y)
