"""検証にかける図の表 ── 16主張 × 倍率 と、組み合わせ × 倍率。

1つの図を数通り試して通ったことは根拠にならない。部品はテーマを共有して
いるので、片方を直すと別が壊れる（実際に2度起きた）。だから全部を毎回かける。

手で走らせる入口（verify_matrix.py）と自動テスト（tests/test_matrix.py）が
同じ表を見るよう、表はここ1か所に置く。
"""
from __future__ import annotations

from typing import Callable  # noqa: F401

import functools

from svg_engine import DEFAULT_THEME, render_figure
from svg_engine.examples.all_claims import CLAIMS, convert


def scaled(k: float) -> dict:
    """全体倍率 k のテーマ。族ごとに効き方を変える（線は弱く、下限でクランプ）。"""
    t = dict(DEFAULT_THEME)
    for key, v in DEFAULT_THEME.items():
        if not isinstance(v, (int, float)) or isinstance(v, bool):
            continue
        if key.startswith("font.") or key.startswith("chart.") or key in (
                "size.box-h", "size.box-min-w", "size.box-pad-x", "size.box-radius"):
            t[key] = v * k
        elif key in ("size.gap-rank", "size.gap-order", "size.canvas-pad",
                     "size.label-pad-x"):
            t[key] = v * (k ** 0.6)
    t["size.stroke-width"] = max(DEFAULT_THEME["size.stroke-width"] * (k ** 0.4), 1.0)
    return t


SCALES = [0.8, 1.0, 1.6, 2.5]

# 組み合わせ ── 単体では出ない崩れを狙う
COMBOS: dict[str, dict[str, list]] = {
    "群の入れ子": dict(nodes=[{"id": c, "label": c.upper()} for c in "abcd"],
                       edges=[{"from": "a", "to": "b"}, {"from": "c", "to": "d"}],
                       groups=[{"label": "外", "members": ["a", "b", "c", "d"]},
                               {"label": "内", "members": ["c", "d"]}]),
    "群＋長いラベルの辺": dict(
                       nodes=[{"id": c, "label": f"名前が長い節点{c.upper()}"} for c in "abc"],
                       edges=[{"from": "a", "to": "b", "label": "とても長い辺のラベル"},
                              {"from": "b", "to": "c", "label": "こちらも長い"}],
                       groups=[{"label": "囲みのラベルも長い", "members": ["b", "c"]}]),
    "循環＋群": dict(nodes=[{"id": c, "label": c.upper()} for c in "abc"],
                       edges=[{"from": "a", "to": "b"}, {"from": "b", "to": "c"},
                              {"from": "c", "to": "a", "label": "戻る"}],
                       groups=[{"label": "輪の一部", "members": ["b", "c"]}]),
    "多段またぎ＋群": dict(
                       nodes=[{"id": c, "label": c.upper()} for c in "abcde"],
                       edges=[{"from": "a", "to": "b"}, {"from": "b", "to": "c"},
                              {"from": "c", "to": "d"}, {"from": "d", "to": "e"},
                              {"from": "a", "to": "e", "label": "飛ぶ"}],
                       groups=[{"label": "中ほど", "members": ["b", "c", "d"]}]),
}


def cases() -> list[tuple[str, str, "Callable[[], str]"]]:
    """全ての図を、まだ組み立てずに数え上げる。

    組み立てそのものを呼び出し側へ預けるのは、1つの図が組み立てに失敗した
    ときに、その1件だけを落として残りを走らせ切るため（表を先に全部組むと、
    1件の例外が表全体を巻き込んで、何件通ったのかが分からなくなる）。

    Returns:
        (倍率の名前, 図の名前, 呼ぶとSVGを返す関数) の並び。
    """
    out: list[tuple[str, str, "Callable[[], str]"]] = []
    for k in SCALES:
        theme = scaled(k)
        for d in CLAIMS:
            out.append((f"{k}倍", str(d["asserts"]),
                        functools.partial(convert, d, theme)))
        for name, kw in COMBOS.items():
            out.append((f"{k}倍", name,
                        functools.partial(render_figure, kw["nodes"], kw["edges"],
                                          groups=kw["groups"], direction="TB", theme=theme)))
    return out


def build_all() -> list[tuple[str, str, str]]:
    """全ての図を実際に組み立てる。手で走らせる入口が使う。

    Returns:
        (倍率の名前, 図の名前, 組み上がったSVG) の並び。
    """
    return [(s, n, f()) for s, n, f in cases()]
