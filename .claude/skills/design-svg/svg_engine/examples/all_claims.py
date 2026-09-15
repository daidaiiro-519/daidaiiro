"""16の主張すべてを、宣言から絵まで通す。

ここに書いた `convert()` が、ADRで言う「アダプタの内側」の実体。
ホストの語彙（asserts/items/links/frame）を受け取り、描画エンジンの
呼び出しへ写す。主張ごとの組み方は design-svg の記法に従う。
"""
from __future__ import annotations

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2]))

from svg_engine import layout_radial, render_chart, render_figure  # noqa: E402

OUT = pathlib.Path(__file__).resolve().parent / "out"
OUT.mkdir(exist_ok=True)


# ── アダプタの内側 ── 主張ごとに、どの部品でどう組むかを解決する ──────────

def _nodes(items, key="key", label="name"):
    return [{"id": it[key], "label": it[label],
             **({"role": it["role"]} if it.get("role") else {})} for it in items]


def _edges(links):
    return [{"from": lk["from"], "to": lk["to"],
             **({"label": lk["name"]} if lk.get("name") else {})} for lk in links]


def _flatten(items, parent=None, nodes=None, edges=None):
    """入れ子の置くものを、節点と親子の辺へ展開する。"""
    nodes = nodes if nodes is not None else []
    edges = edges if edges is not None else []
    for it in items:
        nodes.append({"id": it["key"], "label": it["name"],
                      **({"role": it["role"]} if it.get("role") else {})})
        if parent:
            edges.append({"from": parent, "to": it["key"], "arrow": "none"})
        if it.get("children"):
            _flatten(it["children"], it["key"], nodes, edges)
    return nodes, edges


def convert(d: dict, theme: dict | None = None) -> str:
    """図の宣言を受け取り、描かれた結果を返す。theme を渡すと見た目だけが変わる。"""
    a = d["asserts"]
    items = d.get("items", [])
    links = d.get("links", [])
    frame = d.get("frame", {})

    # ── 関係を主張するもの（7） ──
    if a == "つながり":
        return render_figure(_nodes(items), _edges(links), direction="TB", theme=theme)
    if a == "階層":
        n, e = _flatten(items)
        return render_figure(n, e, direction="TB", theme=theme)
    if a == "包含":
        n, _ = _flatten(items)
        parents = {it["key"] for it in items}
        groups = [{"label": it["name"], "members": [c["key"] for c in it.get("children", [])]}
                  for it in items if it.get("children")]
        n = [x for x in n if x["id"] not in parents]
        return render_figure(n, [], groups=groups, direction="TB", theme=theme)
    if a == "順序":
        keys = [it["key"] for it in items]
        e = [{"from": x, "to": y} for x, y in zip(keys, keys[1:])]
        return render_figure(_nodes(items), e, direction="LR", theme=theme)
    if a == "循環":
        # 並びが閉じることを主張するので、層状に並べず輪に置く。
        # 縦一列＋長い戻り線では、閉じていることが図から読めない。
        keys = [it["key"] for it in items]
        e = [{"from": keys[i], "to": keys[(i + 1) % len(keys)]} for i in range(len(keys))]
        return render_figure(_nodes(items), e, theme=theme, layout=layout_radial)
    if a == "やり取り":
        return render_chart("exchange", {
            "participants": [it["name"] for it in items],
            "steps": [{"from": _name(items, lk["from"]), "to": _name(items, lk["to"]),
                       "label": lk.get("name"), "kind": lk.get("kind", "call")} for lk in links],
            "groups": frame.get("groups", []),
        }, theme=theme)
    if a == "対応":
        cols = frame.get("axes", [{"unit": ""}, {"unit": ""}])
        rows = [[it["name"] for it in pair] for pair in _pairs(items)]
        return render_chart("table", {"headers": [c["unit"] for c in cols], "rows": rows}, theme=theme)

    # ── 量を主張するもの（9） ──
    vals = [{"name": it["name"], "value": it.get("value", 0)} for it in items]
    if a == "全体と部分":
        return render_chart("pie", {"slices": vals,
                                    "centre": str(sum(v["value"] for v in vals))}, theme=theme)
    if a == "量の大小":
        # 軸は2本。1本目が値、2本目が品目そのものを表す
        return render_chart("bars", {"bars": vals,
                                     "axis_label": _unit(frame, 0),
                                     "item_axis_label": _unit(frame, 1)}, theme=theme)
    if a == "分布":
        # 品目の軸は bins が名づける ── 何を区切っているか。値は度数
        return render_chart("bars", {"bars": vals,
                                     "axis_label": _unit(frame, 0),
                                     "item_axis_label": frame.get("bins")}, theme=theme)
    if a == "偏差":
        return render_chart("bars", {"bars": vals, "baseline": frame.get("baseline", 0),
                                     "axis_label": _unit(frame, 0)}, theme=theme)
    if a == "順位":
        return render_chart("ranking", {"items": vals}, theme=theme)
    if a == "時間変化":
        return render_chart("lanes", {
            "rows": [{"name": it["name"], "bars": [{"from": it["span"][0],
                                                    "to": it["span"][0] + it["span"][1]}]}
                     for it in items],
            "axis_label": _unit(frame, 0)}, theme=theme)
    if a == "相関":
        return render_chart("scatter", {
            "points": [{"name": it["name"], "x": it["at"][0], "y": it["at"][1]} for it in items],
            "x_label": _unit(frame, 0), "y_label": _unit(frame, 1)}, theme=theme)
    if a == "流量":
        return render_chart("flow", {
            "links": [{"from": _name(items, lk["from"]), "to": _name(items, lk["to"]),
                       "value": lk["weight"]} for lk in links]}, theme=theme)
    if a == "空間":
        return render_chart("spatial", {
            "items": [{"name": it["name"], "depth": i,
                       **({"role": it["role"]} if it.get("role") else {})}
                      for i, it in enumerate(items)]}, theme=theme)
    raise ValueError(f"知らない主張です: {a}")


def _name(items, key):
    return next(it["name"] for it in items if it["key"] == key)


def _unit(frame, i):
    ax = frame.get("axes") or []
    return ax[i].get("unit") if i < len(ax) else None


def _pairs(items):
    return [items[i:i + 2] for i in range(0, len(items), 2)]


# ── 16の宣言（ホストの語彙で書いたもの） ────────────────────────────
CLAIMS: list[dict] = [
 {"asserts": "つながり", "reading": "受け口はユースケースを呼び、モデルへ届く。",
  "items": [{"key": "cli", "name": "CLI"}, {"key": "mcp", "name": "MCP"},
            {"key": "uc", "name": "ユースケース", "role": "focus"},
            {"key": "svc", "name": "業務サービス"}, {"key": "mdl", "name": "モデル"}],
  "links": [{"from": "cli", "to": "uc"}, {"from": "mcp", "to": "uc"},
            {"from": "uc", "to": "svc", "name": "呼ぶ"}, {"from": "svc", "to": "mdl"}]},

 {"asserts": "階層", "reading": "語彙は器・意味・共通の欄に分かれる。",
  "items": [{"key": "v", "name": "描く語彙", "children": [
      {"key": "k", "name": "器・経路"},
      {"key": "m", "name": "意味", "role": "focus", "children": [
          {"key": "t", "name": "文字4"}, {"key": "f", "name": "図8"}]},
      {"key": "c", "name": "共通の欄"}]}]},

 {"asserts": "包含", "reading": "段1は実装に属し、段2・段3は文書に属する。",
  "items": [{"key": "impl", "name": "実装", "children": [{"key": "s", "name": "Schema"}]},
            {"key": "doc", "name": "文書", "children": [{"key": "ty", "name": "型"},
                                                        {"key": "en", "name": "実体"}]}]},

 {"asserts": "順序", "reading": "調べ、決め、引き継ぎ、作る。",
  "items": [{"key": "a", "name": "調べる"}, {"key": "b", "name": "決める"},
            {"key": "c", "name": "引き継ぐ"}, {"key": "d", "name": "作る", "role": "focus"}]},

 {"asserts": "循環", "reading": "生成し、測り、直し、知識へ戻る。",
  "items": [{"key": "a", "name": "生成"}, {"key": "b", "name": "測定"},
            {"key": "c", "name": "修正"}, {"key": "d", "name": "知識へ還元"}]},

 {"asserts": "やり取り", "reading": "判定の結果で応答が分かれる。",
  "items": [{"key": "c", "name": "呼ぶ側"}, {"key": "d", "name": "文書"}, {"key": "t", "name": "型"}],
  "links": [{"from": "c", "to": "d", "name": "判定する"},
            {"from": "d", "to": "t", "name": "指針を引く"},
            {"from": "d", "to": "c", "name": "適合を返す", "kind": "return"},
            {"from": "d", "to": "c", "name": "不適合を返す", "kind": "return"}],
  "frame": {"groups": [{"label": "結果で分かれる", "cases": [
      {"name": "適合しているとき", "span": [2, 2]},
      {"name": "適合していないとき", "span": [3, 3]}]}]}},

 {"asserts": "対応", "reading": "仕様の要素と、実装で確かめるものが対応する。",
  "items": [{"key": "a1", "name": "受け入れ基準"}, {"key": "b1", "name": "保証シナリオ"},
            {"key": "a2", "name": "業務ユースケース"}, {"key": "b2", "name": "操作の契約"},
            {"key": "a3", "name": "集約"}, {"key": "b3", "name": "不変条件"}],
  "frame": {"axes": [{"unit": "仕様の要素"}, {"unit": "実装で確かめるもの"}]}},

 {"asserts": "全体と部分", "reading": "16の主張は関係7と量9に分かれる。",
  "items": [{"key": "r", "name": "関係", "value": 7}, {"key": "q", "name": "量", "value": 9}]},

 {"asserts": "量の大小", "reading": "最も使われている宣言が、最も説明されていない。",
  "items": [{"key": "a", "name": "x-prompt-write", "value": 623},
            {"key": "b", "name": "x-prompt-query", "value": 216},
            {"key": "c", "name": "x-render", "value": 157}],
  "frame": {"axes": [{"unit": "使用回数"}, {"unit": "宣言の種別"}]}},

 {"asserts": "順位", "reading": "描画部品の使われ方には偏りがある。",
  "items": [{"key": "a", "name": "条件による選択", "value": 62},
            {"key": "b", "name": "やり取りの順序", "value": 49},
            {"key": "c", "name": "向きのある関係", "value": 30},
            {"key": "d", "name": "一列に並ぶ", "value": 7}]},

 {"asserts": "時間変化", "reading": "決定は日をまたいで積み上がった。",
  "items": [{"key": "a", "name": "調べる", "span": [0, 3]},
            {"key": "b", "name": "決める", "span": [3, 2]},
            {"key": "c", "name": "引き継ぐ", "span": [5, 1]},
            {"key": "d", "name": "作る", "span": [6, 5]}],
  "frame": {"axes": [{"unit": "日"}]}},

 {"asserts": "分布", "reading": "節点の数は5〜8に集中している。",
  "items": [{"key": "a", "name": "2-4", "value": 38}, {"key": "b", "name": "5-8", "value": 64},
            {"key": "c", "name": "9-12", "value": 15}, {"key": "d", "name": "13-16", "value": 8}],
  "frame": {"bins": "節点の数", "axes": [{"unit": "図の枚数"}]}},

 {"asserts": "偏差", "reading": "言及回数は、平均から大きく下振れしている。",
  "items": [{"key": "a", "name": "x-prompt-write", "value": 19},
            {"key": "b", "name": "x-render", "value": 74},
            {"key": "c", "name": "x-render-level", "value": 2},
            {"key": "d", "name": "x-render-order", "value": 0}],
  "frame": {"baseline": 24, "axes": [{"unit": "平均からの差"}]}},

 {"asserts": "相関", "reading": "使われている宣言ほど説明されている、とは言えない。",
  "items": [{"key": "a", "name": "文書の検証", "at": [8, 7]},
            {"key": "b", "name": "描画", "at": [5, 6]},
            {"key": "c", "name": "設定の読み込み", "at": [2, 2]},
            {"key": "d", "name": "配置の検査", "at": [7, 3]}],
  "frame": {"axes": [{"unit": "差別化の大きさ"}, {"unit": "複雑さ"}]}},

 {"asserts": "流量", "reading": "描画部品の多くは、文字の側へ流れている。",
  "items": [{"key": "s", "name": "構造化データ"}, {"key": "m", "name": "Markdown"},
            {"key": "h", "name": "HTML"}],
  "links": [{"from": "s", "to": "m", "weight": 7}, {"from": "s", "to": "h", "weight": 3}]},

 {"asserts": "空間", "reading": "段は外側から内側へ並ぶ。位置が深さを表す。",
  "items": [{"key": "a", "name": "受け口"}, {"key": "b", "name": "ユースケース"},
            {"key": "c", "name": "業務サービス", "role": "focus"}, {"key": "d", "name": "モデル"}]},
]

# 読み込むだけで図を描かない。CLAIMS と convert() は他（検証・テスト）から
# import されるので、ここで描いてしまうと、部品1つの例外が import ごと
# 巻き込み、テストが1件も走らないまま収集で落ちる（実際にそうなった）。
if __name__ == "__main__":
    results: list[dict] = [{"declaration": d, "svg": convert(d)} for d in CLAIMS]
    (OUT / "all_claims.json").write_text(
        json.dumps(results, ensure_ascii=False), encoding="utf-8")
    print(f"通した主張: {len(results)}件")
    for r in results:
        print(" ", dict(r["declaration"])["asserts"])
