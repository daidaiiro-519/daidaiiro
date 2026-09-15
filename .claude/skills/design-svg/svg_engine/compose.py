"""合成 ── 宣言（節点・辺・囲み）から、1枚のSVGを組み立てる核。

ここが知っているのは「節点(nodes)・辺(edges)・囲み(groups)」という、どんな
グラフ図にも共通する一般名詞だけ。呼ぶ側が固有の語彙を持っていても、それを
この形へ直す役目は核の外（呼び出し側の変換）が担う。

**この核が守る性質** ── 判断が最も集まる場所なので、何を守っているのかを
ここに集める。個々の実測の記録は、それぞれの行のコメントに残してある。

1. **辺は必ず、両端のインクに着く。** 着き先は部品が申告した形ではなく、
   部品が実際に描いたインクから選ぶ。迂回で入り方が変わったら決め直す。
   （申告させると申告と実物がずれ、外接矩形で代用した部品では空白を指した）
2. **同じ節点へ集まる辺は、同じ点へ収束する。** 相手の方向へそのまま引かず、
   どの辺から出すかだけを相手の位置で決め、その辺の中央を狙う。
   （方向へ直接引くと、角のすぐ脇に着いて不自然に見える）
3. **線は曲げない。曲げるのは節点を避けるときだけ。** 札は動かせるので
   障害物に数えない ── 札を守るために曲げると、まっすぐでよい関係まで大回りする。
4. **迂回する車線は、節点どうしと同じ間隔だけ離れる。** 車線は仮の節点の列
   なので、節点と同じ間隔で並ぶのが筋。（線幅ぶんだけでは迂回に見えない）
5. **動かせるものが譲り、動かせないものは動かない。** 札（辺の札・囲みの札）が
   譲る側。譲れないときは不透明な帯で線を断って上に載る。
6. **札には逃げ場がある。** 段の間隔は、その間を通る辺の札が収まるだけ空ける。
   逃げ場が足りないと、動かせるはずの札が節点の名前へ重なる。
7. **画布は、描いたものを全部含む。** 節点だけでなく、外へはみ出す囲み・
   迂回した辺・その上に乗る札まで含めて取る。余白は四辺へ均等に。
8. **群の要素だけが、群の矩形の内側に居る。** これは nesting.py が保証する
   （群を先に畳み、親は1個として置く）。ここはその結果を使うだけ。

**描く順序**は 囲み → 辺 → 札 → 節点。札を辺より後に描くのは、避けられな
かったときでも帯が線を断って読めるようにするため（性質5）。

座標の解決は sugiyama.py（層状グラフ描画。サイクル・複数段またぎ・
交差する辺のいずれにも耐える本格版）に委ねる。環状・放射の木へ差し替える
ときは、同じ契約（LayoutResult を返す）の関数を layout 引数で渡す。
"""
from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

    from .layout_contract import LayoutResult

from .geometry import densify, ink_surface, nearest, segment_hits_rect
from .labels import place_edge_labels
from .text import text_width as _text_width
from .registry import Absolute, OwnOrigin, render_component, render_node
from .style import resolve_style
from .nesting import layout_nested
from .sugiyama import layout_graph
from .tokens import DEFAULT_THEME, num


def _centre(pos: tuple[float, float], size: tuple[float, float]) -> tuple[float, float]:
    return (pos[0] + size[0] / 2, pos[1] + size[1] / 2)


def _avoid(pts, obstacles, direction: str, margin: float, keep_out=None):
    """経路が箱を突っ切るなら、ぶつかった箱の脇を回る点を挟む。

    どの箱をどれだけ避けるかは、ぶつかった箱の外接矩形から毎回決める。
    特定の図のための細工を置かない。

    Args:
        pts: 経路の点列。
        obstacles: 突っ切ってはいけない矩形の並び。
        direction: "TB" または "LR"。
        margin: 障害物から空ける量。
        keep_out: 迂回路が「中を並走してはいけない」矩形の並び（囲みの内側）。
            囲みを跨ぐのは正常なので障害物にはしないが、その縁に沿って
            長く並走すると枠線と見分けが付かなくなる。迂回の位置だけ外へ出す。
    """
    out = [pts[0]]
    for a, b in zip(pts, pts[1:]):
        hit = [o for o in obstacles if segment_hits_rect(a, b, o, margin)]
        if hit:
            lo = min(o[0] for o in hit), min(o[1] for o in hit)
            hi = max(o[2] for o in hit), max(o[3] for o in hit)
            if direction == "TB":
                # 近い側の外へ寄せて、縦に迂回する
                left = abs(a[0] - lo[0]) + abs(b[0] - lo[0])
                right = abs(a[0] - hi[0]) + abs(b[0] - hi[0])
                x = (lo[0] - margin * 2) if left <= right else (hi[0] + margin * 2)
                x = _push_out(x, a[1], b[1], keep_out, margin, axis=0)
                out.append((x, a[1]))
                out.append((x, b[1]))
            else:
                top = abs(a[1] - lo[1]) + abs(b[1] - lo[1])
                bottom = abs(a[1] - hi[1]) + abs(b[1] - hi[1])
                y = (lo[1] - margin * 2) if top <= bottom else (hi[1] + margin * 2)
                y = _push_out(y, a[0], b[0], keep_out, margin, axis=1)
                out.append((a[0], y))
                out.append((b[0], y))
        out.append(b)
    return out



def _push_out(v: float, s0: float, s1: float, keep_out, margin: float, axis: int) -> float:
    """迂回の位置が囲みの内側なら、近いほうの縁の外へ出す。

    枠線から margin*2 だけ離す ── 節点から離す量と同じ根拠で、線幅から導く。
    """
    if not keep_out:
        return v
    lo_s, hi_s = (s0, s1) if s0 <= s1 else (s1, s0)
    other = 1 - axis
    for r in keep_out:
        # 迂回が伸びる向きに、この囲みと重なりがあるか
        if hi_s < r[other] or lo_s > r[other + 2]:
            continue
        if r[axis] < v < r[axis + 2]:
            near_lo = v - r[axis]
            near_hi = r[axis + 2] - v
            v = (r[axis] - margin * 2) if near_lo <= near_hi else (r[axis + 2] + margin * 2)
    return v


def _self_loop(pos, size, gap: float, style):
    """自分から自分へ戻る辺の経路。節点の脇へ小さな輪を作る。

    普通の辺として扱うと始点と終点が同じ位置になり、線も矢じりも
    向きを持てない。輪の大きさは節点の大きさから決める。

    形を決める3つの比はトークンから引く ── 節点の高さと輪の半径に対する比なので、
    大きさが変わっても形が保たれる。直書きすると、テーマを差し替えても輪だけが
    取り残される。
    """
    x, y = pos
    w, h = size
    attach = style.num("size.self-loop-attach")
    bulge = style.num("size.self-loop-bulge")
    lift = style.num("size.self-loop-lift")
    r = min(w, h) / 2 + gap
    cx, cy = x + w, y + h / 2
    top, bottom = y + h * attach, y + h * (1 - attach)
    return [(x + w, top), (cx + r, top - r * lift),
            (cx + r * bulge, cy), (cx + r, bottom + r * lift),
            (x + w, bottom)]


def _cardinal(pos, size, ink, toward, flow=None):
    """相手が居る側を上下左右のどれかに決め、その側のインクへ着ける。

    相手の方向へそのまま引くと、輪郭の角の近くへ着いて不自然に見える
    （実測：箱の左辺の下端すれすれに矢印が刺さった）。どの辺から出すかだけを
    相手の位置で決め、その辺の中央を狙う ── 図として規則的になり、同じ節点へ
    集まる線が同じ点へ収束する。

    狙った先そのものではなく、**そこにいちばん近い実際のインク**へ着ける。
    輪郭（閉じた多角形）を経由すると、頂点を結んだ弦がインクの隙間をまたぎ、
    波やチェック記号では見た目に空白を指す（実測：波で13.7の隔たり）。
    インクの点から選べば、どんな形でも必ずインクの上に着く。

    Args:
        pos: 部品の左上の絶対座標。
        size: (width, height)。
        ink: 部品が描いたインクの表面の点（原点基準）。
        toward: 相手の位置。

    Returns:
        接続点の絶対座標。

    Raises:
        なし。
    """
    cx, cy = pos[0] + size[0] / 2, pos[1] + size[1] / 2
    dx, dy = toward[0] - cx, toward[1] - cy
    if dx == 0 and dy == 0:
        return (cx, cy)
    # 段が進む向きで離れているなら、その向きの辺から出す。
    #
    # 箱の形だけで決めると、横長の箱では斜めが浅くなり、真下にある相手まで
    # 「横にある」と判定される。すると辺が箱の列の中へ入り、間の箱を避けて
    # 長く迂回する（実測：枝10本の木で、根から両端の枝への辺が列の上を
    # 横切って引かれた）。段の進む向きは、その図で構造が伸びている向きなので、
    # そちら側から出入りするほうが読み手の期待と合う。
    #
    # 「離れている」は閾値ではなく、この箱自身の半分で測る ── 相手が箱の
    # 内側に収まる高さにいるなら、それは横に並んでいるということである。
    if flow is not None:
        d_flow = dy if flow == 1 else dx
        if abs(d_flow) > size[flow] / 2:
            aim = ((cx, pos[1] + (size[1] if dy >= 0 else 0.0)) if flow == 1
                   else (pos[0] + (size[0] if dx >= 0 else 0.0), cy))
            here = nearest(ink, (aim[0] - pos[0], aim[1] - pos[1]))
            return (pos[0] + here[0], pos[1] + here[1])
    if abs(dx) * size[1] >= abs(dy) * size[0]:
        aim = (pos[0] + (size[0] if dx >= 0 else 0.0), cy)
    else:
        aim = (cx, pos[1] + (size[1] if dy >= 0 else 0.0))
    here = nearest(ink, (aim[0] - pos[0], aim[1] - pos[1]))
    return (pos[0] + here[0], pos[1] + here[1])


def _detour_aim(pos, size, routed, direction: str):
    """迂回する辺が「どちら側を回るか」を返す。まっすぐなら None。

    中継点のうち、節点の中心から横（TBなら左右、LRなら上下）へ最も離れた
    ものを見る。その隔たりが節点の半分を超えていれば、そちら側を回っている。

    Args:
        pos: 節点の左上の絶対座標。
        size: (width, height)。
        routed: 経路の点列。
        direction: "TB" または "LR"。

    Returns:
        向きを表す点。迂回していなければ None。

    Raises:
        なし。
    """
    if len(routed) <= 2:
        return None
    axis = 0 if direction == "TB" else 1
    centre = pos[axis] + size[axis] / 2
    far = max(routed[1:-1], key=lambda p: abs(p[axis] - centre))
    if abs(far[axis] - centre) <= size[axis] / 2:
        return None
    other = 1 - axis
    aim = [0.0, 0.0]
    aim[axis] = far[axis]
    aim[other] = pos[other] + size[other] / 2
    return (aim[0], aim[1])


def _nested(decl: dict, theme: dict, depth: int, label: str | None = None) -> OwnOrigin:
    """節点の中身として置く子図を組み立てる。

    深さに上限を置くのは、段1 が入れ子の文法にそう定めているため。上限が
    無いと、自分を指す宣言で終わらなくなる。

    Args:
        decl: 子図の宣言（nodes / edges / groups / direction を持つ）。
        theme: 親と同じテーマ。子図だけ別の見た目にはしない。
        depth: いまの深さ。
        label: 入れ子に付ける名前。あれば囲んで名札を付ける。

    Returns:
        ComponentResult。部品と同じ契約なので、親は他の部品と区別せず置ける。

    Raises:
        ValueError: 深さが上限を超えたとき。
    """
    limit = int(theme["size.figure-depth-limit"])
    if depth >= limit:
        raise ValueError(f"図の入れ子が深すぎる（上限 {limit}）")
    inner = figure_fragment(decl.get("nodes", []), decl.get("edges"), decl.get("groups"),
                            decl.get("direction", "TB"), theme, _depth=depth + 1)
    if not label:
        return inner

    # 名前を持つ入れ子は、囲んで名札を付ける。付けないと、子の節点が親と同じ
    # 平面に並んでいるようにしか見えず、入れ子であることが絵から読めない
    # （実測：名前を落としたまま描くと、親→子図の辺が子の先頭の節点を
    # 指しているようにしか見えなかった）。
    #
    # 囲みと名札は群のために既にある部品を使う。同じ「塊に名前を付ける」ことを
    # 2つの方法で描くと、テーマを差し替えたときに見た目が揃わなくなる。
    style = resolve_style("plain", None, theme)
    pad = style.num("font.size-small") * style.num("size.frame-pad-ratio")
    label_h = style.num("font.size-small") * style.num("size.label-line-h")
    w, h = inner.width + pad * 2, inner.height + pad * 2
    frame = render_component(style.text("parts.group"), {
        "x": 0, "y": label_h, "width": w, "height": h, "label": None}, style)
    tag = render_component("frame_label", {
        "x": style.num("size.label-pad-x"), "y": label_h, "label": label}, style)
    return OwnOrigin(
        svg=(f'{frame.svg}<g transform="translate({pad:.1f},{label_h + pad:.1f})">'
             f'{inner.svg}</g>{tag.svg}'),
        width=w, height=h + label_h, labels_itself=True)


def figure_fragment(nodes: list[dict], edges: list[dict] | None = None,
                   groups: list[dict] | None = None, direction: str = "TB",
                   theme: dict | None = None,
                   layout: "Callable[..., LayoutResult] | None" = None,
                   nested_layout: "Callable[..., tuple] | None" = None,
                   _depth: int = 0) -> OwnOrigin:
    """節点・辺・囲みの宣言から、**部品として置ける断片**を組み立てる。

    ルートタグを被せない。返すのは中身と、それを囲む大きさ ── つまり部品と
    同じ契約である。だから図を他の図の中へ置ける。器を被せた1枚が欲しいときは
    render_figure() を呼ぶ。

    大きさは、囲みのはみ出し・迂回した辺・その上に乗る札まで含めて外形を出し、
    原点を左上へ寄せてから決める。実測：15通りの図すべてで、インクがこの
    大きさから出た量は 0.0 だった。

    Args:
        nodes: [{"id": str, "label": str, "role": str(任意), "style": dict(任意)}, ...]
        edges: [{"from": str, "to": str, "label": str(任意), "dashed": bool(任意),
                 "arrow": str(任意)}, ...]。サイクル・複数段をまたぐ辺・
                交差する辺のいずれを含んでもよい。
        groups: [{"label": str, "members": [id, ...]}, ...]（任意）
        direction: "TB" または "LR"。
        theme: DEFAULT_THEME を上書きするテーマ（CSSの:root差し替えに相当）。
        layout / nested_layout: 座標を解く手を差し替える。省略すると層状配置を使う。
            別の配置（力学配置など）を試すときに、コアを直さずに差し替えられる。
            まだ実装が1つしか無いので、登録の仕組みは置かず引数だけにしてある。

    Returns:
        `<svg>...</svg>` 文字列。role/theme が変わっても、宣言（nodes/edges/groups）は
        一切変えずに見た目だけが変わる。

    Raises:
        KeyError: 宣言が存在しない節点idを指したとき。
    """
    edges = edges or []
    groups = groups or []
    theme = theme or DEFAULT_THEME
    depth = _depth
    # 段が進む向き。層状のときだけ辺の出入りに効かせるので、既定値で埋める前に見る
    # ── 埋めた後だと、戦略を選ばなかったことが分からなくなる。
    flow = (1 if direction == "TB" else 0) if layout is None else None
    layout = layout or layout_graph
    nested_layout = nested_layout or layout_nested

    rendered = {}
    for n in nodes:
        style = resolve_style(n.get("role", "plain"), n.get("style"), theme)
        if n.get("figure"):
            # 節点の中身が図。子図を先に組み立てて、大きさの分かった1つにする
            # ── 返るのは部品と同じ（中身・幅・高さ）なので、以降は他の部品と
            # 区別せず扱える。描き上がったものを外から渡す形にはしない
            # （props は構造だけ、という契約を破らないため）。
            rendered[n["id"]] = _nested(n["figure"], theme, depth, n.get("label"))
        else:
            rendered[n["id"]] = render_node(style.text("parts.node"), n, style)

    sizes = {nid: (r.width, r.height) for nid, r in rendered.items()}
    # 辺の着き先は部品に申告させず、部品が描いたインクそのものから選ぶ。
    # 申告させると、申告した形と描いた形がずれる（11種のうち輪郭を申告して
    # いたのは3種だけで、残りは外接矩形で代用され、インクが矩形の一部にしか
    # 無い部品では辺が空白へ着いていた）。導出が1本なら、部品を足しても
    # 着き先の決め方が増えない。
    fineness = int(num(theme, "size.outline-facets"))
    inks = {nid: ink_surface(r.svg, r.width, r.height, fineness)
            for nid, r in rendered.items()}
    edge_pairs = [(e["from"], e["to"]) for e in edges]

    # 囲みの余白とラベルの高さは、書体のトークンから導く。図ごとの決め打ちを置かない。
    frame_style = resolve_style("plain", None, theme)
    frame_pad = frame_style.num("font.size-small") * frame_style.num("size.frame-pad-ratio")
    label_h = frame_style.num("font.size-small") * frame_style.num("size.label-line-h")

    # 段の間隔は、その間を通る辺の札が収まるだけ空ける。札は動かせるが、
    # 逃げ場が段の間隔しかないので、札がその間隔より長いと逃げ切れず、
    # 節点の名前に重なる（実測：3節点の鎖で4件。札を短くすると0件）。
    # 「札どうしが重ならない」だけを性質にしていたので、逃げ場が足りるかを
    # 誰も見ていなかった。ここで逃げ場の側を保証する。
    gap_rank = num(theme, "size.gap-rank")
    if edges:
        fs = resolve_style("plain", None, theme).num("font.size-small")
        pad = num(theme, "size.label-pad-x")
        need = max((_text_width(str(e["label"]), fs) + pad for e in edges if e.get("label")),
                   default=0.0)
        if direction == "LR":
            gap_rank = max(gap_rank, need + num(theme, "size.gap-order"))
        else:
            # 縦に進む辺では、札は帯の高さぶんしか段を占めない
            gap_rank = max(gap_rank, num(theme, "size.label-band-h") + num(theme, "size.gap-order"))

    if groups:
        # 群があるときは、群を先に解いて1つの大きさへ畳み、親はそれを1個として置く。
        # こうしないと、段をまたぐ群の外接矩形が間の非メンバーを飲み込む。
        node_boxes, group_boxes, nested_paths, total_w, total_h = nested_layout(
            sizes, edge_pairs, groups, gap_rank,
            num(theme, "size.gap-order"), direction, frame_pad, label_h,
            layout=layout)
        coords = {nid: (b.x, b.y) for nid, b in node_boxes.items()}
        # 経路が解けなかった辺（群の内側で完結する等）だけ、両端を直結する
        edge_paths = {i: nested_paths.get(
            i, [_centre(coords[a], sizes[a]), _centre(coords[b], sizes[b])])
            for i, (a, b) in enumerate(edge_pairs)}
    else:
        result = layout(sizes, edge_pairs, gap_rank,
                              num(theme, "size.gap-order"), direction)
        coords = result.positions
        group_boxes = {}
        total_w, total_h = result.width, result.height
        edge_paths = result.edge_paths

    body: list[str] = []
    # 囲みは節点の外側へはみ出す（余白ぶんと、枠の上に乗るラベルぶん）。
    # 画布の大きさを節点だけから決めると、この分が切れる。
    frame_bounds = [(group_boxes[f"__g{i}"].x, group_boxes[f"__g{i}"].y,
                     group_boxes[f"__g{i}"].x + group_boxes[f"__g{i}"].width,
                     group_boxes[f"__g{i}"].y + group_boxes[f"__g{i}"].height)
                    for i in range(len(groups))]

    edge_style = resolve_style("plain", None, theme)

    edge_points: dict[int, list[tuple[float, float]]] = {}
    for idx, e in enumerate(edges):
        a, b = e["from"], e["to"]
        if a == b:
            # 自分へ戻る辺は、配置の解いた経路（同じ点が2つ）では表せない
            edge_points[idx] = _self_loop(coords[a], sizes[a],
                                          edge_style.num("size.gap-order") / 2,
                                          edge_style)
            continue
        pts = list(edge_paths[idx])
        axis = 1 if direction == "TB" else 0
        backward = coords[b][axis] < coords[a][axis]
        # 接続点は、部品が申告した輪郭と、相手へ向かう向きの交点で決める。
        nxt = pts[1] if len(pts) > 1 else pts[-1]
        prv = pts[-2] if len(pts) > 1 else pts[0]
        # 接続点は、相手へ向かう向きと自分の輪郭の交点。輪郭は描いたインクから
        # 導いてあるので、箱でも輪でも波でも同じ扱いでよい。
        ia, ib = inks[a], inks[b]
        pts[0] = _cardinal(coords[a], sizes[a], ia, nxt, flow)
        pts[-1] = _cardinal(coords[b], sizes[b], ib, prv, flow)
        # 端の2つ以外が占めている領域のうち、動かせないもの（節点）だけが
        # 障害物。札は動かせるので、線を曲げさせず札のほうを後で避けさせる
        # （札を守るために線を曲げると、まっすぐでよい関係まで大回りする）。
        obstacles = [(coords[n][0], coords[n][1],
                      coords[n][0] + sizes[n][0], coords[n][1] + sizes[n][1])
                     for n in coords if n not in (a, b)]
        routed = _avoid(pts, obstacles, direction, edge_style.num("size.stroke-width") * 2,
                        keep_out=frame_bounds)
        # 迂回で入り方が変わったら、接続点も決め直す。曲げる前の向きで決めた
        # ままだと、横から来た線が底辺の中点へ刺さり、矢じりが箱へめり込む。
        if len(routed) > 1:
            # まっすぐな辺は相手の方を向く。迂回する辺（中継点を持つ）は、
            # 回る側の辺から出入りする ── すぐ隣の中継点だけを見ると、
            # 1段下の中継点に引かれて底辺から出てしまい、同じ「迂回」なのに
            # 図によって出る辺が変わる（実測：循環は左辺、多段またぎは底辺）。
            aim_a = _detour_aim(coords[a], sizes[a], routed, direction)
            aim_b = _detour_aim(coords[b], sizes[b], routed, direction)
            routed[0] = _cardinal(coords[a], sizes[a], ia, aim_a or routed[1], flow)
            routed[-1] = _cardinal(coords[b], sizes[b], ib, aim_b or routed[-2], flow)
            # 横の辺から出入りするなら、まず横へ抜けてから曲がる。辺から出て
            # すぐ真下へ折れると、出た向きと進む向きが食い違い、矢じりも
            # 辺と直交しない（実測：右辺から出て矢じりが下を向いた）。
            axis = 0 if direction == "TB" else 1
            # 迂回する車線は、両端の節点からも離す。配置が置いた仮節点の列は
            # 隣の段の節点との間隔しか見ていないので、端の節点をかすめること
            # がある（実測：4.3px しか離れず、迂回に見えなかった）。離す量は
            # 節点どうしを離す量と同じにする ── 車線は仮の節点の列なので、
            # 節点と同じ間隔で並ぶのが筋。線幅ぶんだけでは、端の節点の縁を
            # なぞって迂回に見えない（実測 4.8px）。
            clear = num(theme, "size.gap-order")
            for aim, node in ((aim_a, a), (aim_b, b)):
                if aim is None:
                    continue
                centre = coords[node][axis] + sizes[node][axis] / 2
                need = sizes[node][axis] / 2 + clear
                lane = aim[axis]
                if abs(lane - centre) < need:
                    lane = centre + need if lane >= centre else centre - need
                    for i in range(1, len(routed) - 1):
                        pt = list(routed[i])
                        if (pt[axis] - centre) * (lane - centre) > 0:
                            pt[axis] = lane
                            routed[i] = tuple(pt)
                    aim_a = (lane, aim_a[1]) if aim_a is not None and axis == 0 else aim_a
                    aim_b = (lane, aim_b[1]) if aim_b is not None and axis == 0 else aim_b
            if aim_a is not None:
                corner = list(routed[0])
                corner[axis] = aim_a[axis]
                routed.insert(1, tuple(corner))
            if aim_b is not None:
                corner = list(routed[-1])
                corner[axis] = aim_b[axis]
                routed.insert(len(routed) - 1, tuple(corner))
        edge_points[idx] = routed

    # 囲みの札を、線を避けた位置へ置く。札は枠の上辺のどこへ置いてもよいので、
    # 動かせない線のほうを優先し、札が譲る。左端から順に試して、どの線とも
    # 重ならない最初の位置を採る（どこも空いていなければ左端に戻す）。
    frame_svgs: list[str] = []
    top_labels: list[str] = []
    frame_label_areas: list[tuple[float, float, float, float]] = []
    # 線の太さより粗く刻むと、線を跳び越して「当たっていない」と誤判定する。
    probe = max(frame_style.num("size.stroke-width"), 0.5)
    all_points = [pt for pts in edge_points.values() for pt in densify(pts, probe)]
    for i, g in enumerate(groups):
        b = group_boxes[f"__g{i}"]
        has_label = bool(g.get("label"))
        pad_x = frame_style.num("size.label-pad-x")
        label_x = b.x + pad_x
        if has_label:
            lw = (_text_width(g["label"], frame_style.num("font.size-small"),
                              frame_style.num("font.latin-width-ratio")) + pad_x)
            # 札の縦位置は、部品が実際に描く位置と同じ式から出す。ここを
            # ずらすと、当たり判定が実物と別の場所を見ることになる。
            fs = frame_style.num("font.size-small")
            top = b.y + label_h - fs * frame_style.num("size.frame-label-rise")
            bottom = top + fs * frame_style.num("size.frame-label-h")
            # 置き場所を試す刻みも、避ける相手（線）の太さに合わせる。
            step = probe
            n_steps = max(int((b.width - lw - pad_x * 2) / step), 0)
            for k in range(n_steps + 1):
                cx = b.x + pad_x + step * k
                if not any(cx < x < cx + lw and top < y < bottom for x, y in all_points):
                    label_x = cx
                    break
            frame_label_areas.append((label_x, top, label_x + lw, bottom))
        r = render_component(frame_style.text("parts.group"), {
            "x": b.x, "y": b.y + (label_h if has_label else 0),
            "width": b.width, "height": b.height - (label_h if has_label else 0),
            "label": None,
        }, frame_style)
        frame_svgs.append(r.svg)
        if has_label:
            # 札は辺より後に描く。避けられなかったときでも、帯が線を断って読める。
            top_labels.append(render_component("frame_label", {
                "x": label_x, "y": b.y + label_h, "label": g["label"],
            }, frame_style).svg)
    body = frame_svgs + body

    # ラベルどうしの重なりは、辺1本ずつでは判断できない（近くを通る別の辺の
    # ラベルと衝突しうるため）。まとめて渡し、重ならない置き場所を決める。
    labelled = [{"index": i, "points": edge_points[i], "label": e["label"]}
                for i, e in enumerate(edges) if e.get("label")]
    # 節点そのものも、札が避けるべき相手。節点は動かせないので札が譲る。
    # ここへ渡していなかったときは、札は他の札と囲みの枠しか見ておらず、
    # 節点の名前の上に重なった（実測：3節点の鎖で4件）。
    node_areas = [(coords[n][0], coords[n][1],
                   coords[n][0] + sizes[n][0], coords[n][1] + sizes[n][1])
                  for n in coords]
    # 囲みの枠線が占める領域（線の太さぶんの細い帯4本）。札は枠線を隠してはいけない。
    sw = frame_style.num("size.stroke-width") * 2
    frame_line_areas: list[tuple[float, float, float, float]] = []
    for x0, y0, x1, y1 in frame_bounds:
        frame_line_areas += [(x0 - sw, y0 - sw, x1 + sw, y0 + sw),
                             (x0 - sw, y1 - sw, x1 + sw, y1 + sw),
                             (x0 - sw, y0 - sw, x0 + sw, y1 + sw),
                             (x1 - sw, y0 - sw, x1 + sw, y1 + sw)]
    label_at = (place_edge_labels(labelled, edge_style,
                                  occupied=frame_label_areas + frame_line_areas + node_areas)
                if labelled else {})

    for idx, e in enumerate(edges):
        r = render_component(edge_style.text("parts.edge"), {
            "points": edge_points[idx], "label": e.get("label"),
            "label_at": label_at.get(idx),
            "dashed": e.get("dashed", False), "arrow": e.get("arrow", "head"),
        }, edge_style)
        body.append(r.svg)

    body.extend(top_labels)

    for n in nodes:
        x, y = coords[n["id"]]
        # 節点であることに印を付ける。検査は「辺の終端が節点のインクに着いて
        # いるか」を見るので、どこからどこまでが節点なのかを外から判る形で
        # 残す必要がある（位置の付け方から推測させると、囲みや辺と混ざる）。
        body.append(f'<g class="wf-node" transform="translate({x:.1f},{y:.1f})">'
                    f'{rendered[n["id"]].svg}</g>')

    # 画布は、節点だけでなく囲みのはみ出しも含めて取る。左と上へはみ出す場合は
    # 原点をずらし、全体を右下へ寄せてから枠を決める。
    pad = frame_style.num("size.canvas-pad")
    # 辺は迂回で節点の外側へ回るし、ラベルの札はその上に乗る。節点と囲みだけを
    # 見て画布を決めると、それらが切れる（実測で見つかった不具合）。
    fs_small = edge_style.num("font.size-small")
    lab_h = fs_small * edge_style.num("size.label-line-h")
    edge_bounds: list[tuple[float, float, float, float]] = []
    for idx, e in enumerate(edges):
        xs = [p[0] for p in edge_points[idx]]
        ys = [p[1] for p in edge_points[idx]]
        edge_bounds.append((min(xs), min(ys), max(xs), max(ys)))
        if e.get("label"):
            cx, cy = label_at.get(idx, (sum(xs) / len(xs), sum(ys) / len(ys)))
            lw = (_text_width(e["label"], fs_small,
                              edge_style.num("font.latin-width-ratio"))
                  + edge_style.num("size.label-pad-x"))
            edge_bounds.append((cx - lw / 2, cy - lab_h, cx + lw / 2, cy + lab_h / 2))
    min_x = min([0.0] + [b[0] for b in frame_bounds] + [b[0] for b in edge_bounds])
    min_y = min([0.0] + [b[1] for b in frame_bounds] + [b[1] for b in edge_bounds])
    max_x = max([total_w] + [b[2] for b in frame_bounds] + [b[2] for b in edge_bounds])
    max_y = max([total_h] + [b[3] for b in frame_bounds] + [b[3] for b in edge_bounds])
    # 余白は四辺へ均等に取る。右下にだけ足すと、いちばん幅の広い要素が
    # 左端・上端に貼り付く（六角形の部品を試したときに目で見つけた）。
    half = pad / 2
    w = max_x - min_x + pad
    h = max_y - min_y + pad
    dx, dy = half - min_x, half - min_y
    inner = (f'<g transform="translate({dx:.1f},{dy:.1f})">{"".join(body)}</g>'
             if (dx or dy) else "".join(body))
    return OwnOrigin(svg=inner, width=w, height=h)


def render_figure(nodes: list[dict], edges: list[dict] | None = None,
                  groups: list[dict] | None = None, direction: str = "TB",
                  theme: dict | None = None,
                  layout: "Callable[..., LayoutResult] | None" = None,
                  nested_layout: "Callable[..., tuple] | None" = None) -> str:
    """図を1枚の完結したSVGとして描く。組み立ては figure_fragment() が行う。

    ここがするのは器を被せることだけ。分けてあるのは、図を他の図の中へ
    置けるようにするため ── 器を被せた時点で、それはもう部品ではない。

    Args:
        nodes / edges / groups / direction / theme / layout / nested_layout:
            figure_fragment() と同じ。

    Returns:
        `<svg>...</svg>` 文字列。

    Raises:
        KeyError: 宣言が存在しない節点idを指したとき。
    """
    r = figure_fragment(nodes, edges, groups, direction, theme, layout, nested_layout)
    return (f'<svg class="wf-fig" viewBox="0 0 {r.width:.0f} {r.height:.0f}" '
            f'width="{r.width:.0f}" height="{r.height:.0f}" role="img">{r.svg}</svg>')


def render_chart(kind: str, props: dict, role: str = "plain",
                  style_overrides: dict | None = None, theme: dict | None = None) -> str:
    """自動配置を要らない部品(pie/bars/ranking/lanes/scatter/flow/spatial)を、
    それ単独で1枚のSVGへ描く。値から座標が一意に決まる部品はグラフの
    レイアウト解決を経由しない、という段1の整理をそのまま反映している。

    Args:
        kind: 台帳に登録された部品名（例: "pie", "bars"）。
        props: その部品が要求する構造の入力。
        role: スタイル解決に使う役割（既定は"plain"）。
        style_overrides: その場限りのインライン上書き。
        theme: DEFAULT_THEME を上書きするテーマ。

    Returns:
        `<svg>...</svg>` 文字列。
    """
    style = resolve_style(role, style_overrides, theme or DEFAULT_THEME)
    r = render_component(kind, props, style)
    pad = style.num("size.canvas-pad-tight")
    w, h = r.width + pad, r.height + pad
    return (f'<svg class="wf-fig" viewBox="0 0 {w:.0f} {h:.0f}" width="{w:.0f}" '
            f'height="{h:.0f}" role="img">{r.svg}</svg>')
