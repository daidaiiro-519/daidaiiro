# SPDX-License-Identifier: MIT
"""design-svg の道具の宣言。**能力の正本はここである。**

計算は `lib/svg_engine/` が保持し、この宣言は**呼び方だけ**を固定する。
CLI も MCP もここから組む ── 能力を2回記述すると、片方だけが古くなる。

**部品は宣言に載せない** ── `lib/` に在るものは読み込まれるものであり、
入口を保持しない。**入口は `cli.py` の1つだけである。**

**この Skill は、呼ぶ側の語彙を知らない。** 受けるのは節点・辺・囲みという
一般名詞と、部品・トークン・配置戦略の名前だけである ── 固有の語彙から
この宣言へ直す変換は、呼ぶ側が持つ。
"""
from __future__ import annotations

import json
import pathlib
import sys

_HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(_HERE))
sys.path.insert(0, str(_HERE / "lib"))

import svg_engine as _engine  # noqa: E402
from contract import Arg, Tool, result  # noqa: E402
from svg_engine.catalog import catalog as _catalog  # noqa: E402
from svg_engine import lint_values as _lint  # noqa: E402
from svg_engine import verify as _verify  # noqa: E402

# 配置戦略 ── 名前から実体へ。**呼ぶ側に関数を渡させない**
_LAYOUTS = {
    "graph": None,                       # 既定（層状）
    "radial": _engine.layout_radial,
    "tree": _engine.layout_tree,
}


def _read(path: str) -> dict | list:
    return json.loads(pathlib.Path(path).read_text(encoding="utf-8"))


def _write(svg: str, out: str) -> dict:
    """生成物を置く。**置き場所を渡されなければ、そのまま返す。**"""
    if out:
        pathlib.Path(out).write_text(svg, encoding="utf-8")
    return {"path": out, "svg": "" if out else svg, "bytes": len(svg)}


def _checked(svg: str) -> list[str]:
    """幾何の検査を3種とも当てる。**通ったことを返り値で示す。**"""
    return [x for name in ("check", "check_shapes", "check_attachment")
            for x in getattr(_verify, name)(svg)]


def catalog(out: str = "") -> dict:
    """目録を出す。**実装から導出される** ── 手で書いた一覧とずれない。"""
    d = _catalog()
    body = json.dumps(d, ensure_ascii=False, indent=1)
    if out:
        pathlib.Path(out).write_text(body + "\n", encoding="utf-8")
    return result(ok=True, path=out, parts=sorted(d["parts"]),
                  strategies=sorted(d["strategies"]), tokens=len(d["tokens"]),
                  body="" if out else body)


def _human_catalog(res: dict) -> str:
    d = res["data"]
    if d["body"]:
        return d["body"]
    return (f"目録を書き出した: {d['path']}　／　部品 {len(d['parts'])} ・ "
            f"配置戦略 {len(d['strategies'])} ・ トークン {d['tokens']}")


def _theme(d: dict) -> dict:
    """宣言の中の `theme` を、テーマの上書きとして受ける。

    **入口がテーマを通さないと、呼ぶ側は Python を書くことになる** ── そして
    その台本は呼ぶ側の作業場に残るだけで、成果物の隣には何も残らない
    （実測で、28枚の図が入力を保持しない状態で残った）。

    渡すのは差分だけでよい。既定のテーマへ重ねる。
    """
    over = d.get("theme")
    if not over:
        return {}
    if not isinstance(over, dict):
        raise ValueError("theme は名前と値の対でなければならない")
    未知 = sorted(k for k in over if k not in _engine.DEFAULT_THEME)
    if 未知:
        raise ValueError("知らないトークン: " + " ・ ".join(未知)
                         + " ── 目録（catalog）に在る名前だけを使う")
    return {"theme": dict(_engine.DEFAULT_THEME, **over)}


def figure(declaration: str, out: str = "", layout: str = "graph",
           direction: str = "TB") -> dict:
    """宣言（節点・辺・囲み）から図を組む。"""
    d = _read(declaration)
    # **宣言に書いたものを、入口が読む** ── 引数でしか渡せないと、
    # 宣言だけでは同じ図が組み直せない（実測で、direction が無視された）
    layout = d.get("layout", layout)
    direction = d.get("direction", direction)
    if layout not in _LAYOUTS:
        return result(ok=False, findings=[f"知らない配置戦略: {layout}。"
                                          f"使えるのは {'／'.join(_LAYOUTS)} である"])
    kw = {"direction": direction, **_theme(d)}
    if _LAYOUTS[layout]:
        kw["layout"] = _LAYOUTS[layout]
    svg = _engine.render_figure(d.get("nodes", []), d.get("edges", []),
                                d.get("groups"), **kw)
    return result(ok=True, findings=_checked(svg), **_write(svg, out))


def chart(kind: str, data: str, out: str = "") -> dict:
    """量を描く部品を1つ選んで描く。"""
    d = _read(data)
    svg = _engine.render_chart(kind, d, **_theme(d))
    return result(ok=True, findings=_checked(svg), kind=kind, **_write(svg, out))


def canvas(layers: str, out: str = "", width: str = "", height: str = "") -> dict:
    """画布へ部品を重ねる。**意匠そのものを組むのはこれである。**

    渡す JSON は `{"width": 数, "height": 数, "layers": [...]}` か、層の並びだけ。
    """
    d = _read(layers)
    body = d["layers"] if isinstance(d, dict) else d
    w = float(width) if width else float(d.get("width", 0) if isinstance(d, dict) else 0)
    h = float(height) if height else float(d.get("height", 0) if isinstance(d, dict) else 0)
    if not w or not h:
        return result(ok=False, findings=["画布の大きさが無い ── width と height を渡す"])
    theme = d.get("theme") if isinstance(d, dict) else None
    kw = {"theme": dict(_engine.DEFAULT_THEME, **theme)} if theme else {}
    if isinstance(d, dict) and d.get("background"):
        kw["background"] = d["background"]
    svg = _engine.render_canvas(w, h, body, **kw)
    return result(ok=True, findings=_checked(svg), **_write(svg, out))


def _human_svg(res: dict) -> str:
    d = res["data"]
    head = [f"  × {x}" for x in res["findings"]]
    # **宣言が通らなかったときも、検出を読める形で出す** ──
    # 書き出しの欄が無い結果で例外にすると、原因が KeyError にすり替わる
    path = d.get("path")
    tail = (f"書き出し: {path}　／　{d['bytes']} 字" if path else d.get("svg", ""))
    note = ("幾何の検査　通った" if not res["findings"]
            else f"幾何の検査　通っていない（{len(res['findings'])} 件）")
    return "\n".join(head + ([note] if path else []) + ([tail] if tail else []))


def verify(svg: str) -> dict:
    """生成物の幾何を検査する。**文字の重なり・はみ出し・貫通・端点。**

    **この検査が見ないもの**が3つある ── 極端な縦横比、配置戦略の選び違い、
    詰まり・読みにくさ・配色の良し悪し。目視の代わりにはならない。
    """
    body = pathlib.Path(svg).read_text(encoding="utf-8")
    return result(ok=True, findings=_checked(body), svg=svg)


def _human_verify(res: dict) -> str:
    bad = res["findings"]
    return "\n".join([f"  × {x}" for x in bad]
                     + [f"幾何の検査　通っていない（{len(bad)} 件）" if bad
                        else "幾何の検査　通った"])


def lint(path: str = "") -> dict:
    """生の数値を探す。**設計上の選択はトークンから、量はデータから出す。**"""
    root = pathlib.Path(path) if path else _HERE / "lib" / "svg_engine"
    hits = _lint.findings(root)
    return result(ok=True, root=str(root),
                  findings=[f"{name}:{line} {func} ── {text.strip()}"
                            for name, line, func, text in hits])


def _human_lint(res: dict) -> str:
    n = len(res["findings"])
    return "\n".join([f"  × {x}" for x in res["findings"]] + [f"生の数値: {n} 箇所"])


TOOLS = [
    Tool(name="catalog", summary="目録を出す（部品・トークン・役割・配置戦略）",
         args=[Arg("out", "書き出し先。省くとそのまま出す", required=False)],
         run=catalog, human=_human_catalog),
    Tool(name="figure", summary="宣言（節点・辺・囲み）から図を組む",
         args=[Arg("declaration", "宣言の JSON"),
               Arg("out", "書き出し先の SVG", required=False),
               Arg("layout", "配置戦略（graph ／ radial ／ tree）", required=False,
                   default="graph"),
               Arg("direction", "向き（TB ／ LR）", required=False, default="TB")],
         run=figure, human=_human_svg),
    Tool(name="chart", summary="量を描く部品を1つ選んで描く",
         args=[Arg("kind", "部品の名前"), Arg("data", "データの JSON"),
               Arg("out", "書き出し先の SVG", required=False)],
         run=chart, human=_human_svg),
    Tool(name="canvas", summary="画布へ部品を重ねて、意匠そのものを組む",
         args=[Arg("layers", "層の JSON"),
               Arg("out", "書き出し先の SVG", required=False),
               Arg("width", "画布の幅", required=False),
               Arg("height", "画布の高さ", required=False)],
         run=canvas, human=_human_svg),
    Tool(name="verify", summary="生成物の幾何を検査する",
         args=[Arg("svg", "検査する SVG")], run=verify, human=_human_verify),
    Tool(name="lint", summary="生の数値を探す",
         args=[Arg("path", "探す場所。省くとエンジン全体", required=False)],
         run=lint, human=_human_lint),
]
