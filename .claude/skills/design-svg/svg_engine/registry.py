"""部品の登録台帳 ── 新しい描画パターンを、コアを直さずに足すための場所。

コンポーネントは「種別名」で引く関数として登録する。核（compose.py・layout.py）は
この台帳越しにしか部品を呼ばない。だから新しい部品（アセット）は、この台帳へ
1行足すだけで使えるようになり、核の側は一切変更しない（開放閉鎖の原則）。
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Callable

from .tokens import Style


@dataclass(frozen=True)
class Fragment:
    """描いた部品の中身と、それが申告すること。

    描く時点が「置く前」か「置いた後」かで、申告する大きさの意味が違う。だから
    フラグではなく2つの型に分ける ── OwnOrigin と Absolute。3つ目を作れないので、
    取り違えが構造として起きない。
    """

    svg: str
    """SVG断片。どの座標系で書かれているかは、この型が決める。"""

    width: float
    height: float
    """外側から見える大きさ。意味は型ごとに違う（各型の説明を見る）。"""

    labels_itself: bool = False
    """渡された名前を、この部品が自分で描いたか。

    名前を描く役目は1箇所にしか置けない。部品が描いたのに包む側も描くと、
    同じ名前が二重に出る（box・hex で実測）。包む側はこの申告を見て、
    自分では描かない。

    輪郭は申告しない ── 申告する形と実際に描いた形はずれる（11種のうち
    輪郭を申告していたのは3種だけで、残りは外接矩形で代用され、インクが
    矩形の一部にしか無い部品では辺が空白へ着いていた）。輪郭は
    geometry.outline_of() が、この svg そのものから導く。
    """


@dataclass(frozen=True)
class OwnOrigin(Fragment):
    """自分の原点(0,0)を基準に描いた断片。呼び出し側が置き場所を決める。

    節点になる部品はこちら。自分がどこへ置かれるかを知らない。呼び出し側が
    `<g transform="translate(x,y)">` で望みの位置へ移す。

    **満たすべき性質** ── 申告した大きさの中に、インクが収まっていなければ
    ならない。外へ出ると、配置は空でない場所を空きと見なし、置いたものどうしが
    重なる（実測：名前の字面が y=-3.6 まではみ出しており、帯を描いて初めて見えた）。

    「最小」であることは求めない。図（棒グラフ等）は作図領域の余白を大きさに
    含むのが正しく、実測でもインクが申告の32%しかない図がある。**含むことは
    課し、詰まっていることは課さない。**
    """


@dataclass(frozen=True)
class Absolute(Fragment):
    """既に置かれたものをまたいで、絶対座標で描いた断片。

    辺・囲み・囲みの札はこちら。置かれた節点の座標を受け取って描くので、
    描いた時点で位置が決まっている。

    申告する大きさは器ではなく、**既に決まった座標の広がりの記録**でしかない。
    だから OwnOrigin に課した「インクが収まる」性質は、こちらには課さない
    ── 課しても意味が無い（器として使われないため）。
    """


# 部品が返すもの。この2つ以外は無い。
ComponentResult = OwnOrigin | Absolute

ComponentFn = Callable[[dict, Style], ComponentResult]

_REGISTRY: dict[str, ComponentFn] = {}


def component(kind: str):
    """デコレータ。この関数を種別名 `kind` の部品として登録する。

    Args:
        kind: 部品の種別名。呼び出し側はこの名前で部品を指定する。
    """
    def deco(fn: ComponentFn) -> ComponentFn:
        if kind in _REGISTRY:
            raise ValueError(f"部品 '{kind}' は既に登録されています")
        _REGISTRY[kind] = fn
        return fn
    return deco


def render_component(kind: str, props: dict, style: Style) -> ComponentResult:
    """種別名から部品を引いて描く。

    Args:
        kind: 部品の種別名（例: "box", "edge"）。
        props: 構造の入力（例: label, from, to）。色・寸法を含まない。
        style: resolve_style() が解決した見た目の値。

    Returns:
        ComponentResult。

    Raises:
        KeyError: 台帳に無い種別名を指定したとき。
    """
    if kind not in _REGISTRY:
        raise KeyError(f"知らない部品です: {kind}（台帳: {sorted(_REGISTRY)}）")
    return _REGISTRY[kind](props, style)


def known_kinds() -> list[str]:
    return sorted(_REGISTRY)


def render_node(kind: str, props: dict, style: Style) -> OwnOrigin:
    """節点として置く部品を描く。自分の原点で描くものでなければならない。

    絶対座標で描く部品（辺・囲み・囲みの札）は、既に置かれたものをまたぐことを
    前提にしているので、節点としては置けない ── 包んでも、名前を載せても、
    座標が既に決まっているので動かせない。

    この前提は以前も暗黙にあったが、置き方が文字列の申告だったので、型の上では
    「どちらか分からないもの」を節点として扱っていた。分けた以上、ここで狭める。

    Args:
        kind: 部品の名前。
        props: 構造。
        style: 解決済みの見た目。

    Returns:
        自分の原点で描かれた断片。

    Raises:
        KeyError: 台帳に無い名前のとき。
        TypeError: その部品が絶対座標で描くものだったとき。
    """
    r = render_component(kind, props, style)
    if not isinstance(r, OwnOrigin):
        raise TypeError(f"部品 '{kind}' は絶対座標で描くので、節点としては置けない")
    return r
