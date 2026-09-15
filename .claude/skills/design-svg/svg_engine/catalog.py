"""目録 ── このエンジンが受け取れるものを、外へ公開する。

利用側（変換器を書く人）は、ここだけを見れば済む。どの部品があり、それぞれが
どんな値を読み、どんなトークンで見た目が決まり、どの配置戦略が選べるか。

**手で書かない。** 手書きの目録は、実物とずれても誰も気づけない ── このエンジンでは
実際に、docstring で宣言した契約が3箇所に書かれたまま11ファイルで破れていた。
だからここで公開する `props` は、**部品の関数が実際に読んでいるキーを構文木から
拾ったもの**である。部品を直せば目録も直る。直し忘れは起こらない。

ずれが起こりうるのは見本（EXAMPLES）だけなので、そこは試験で縛る ──
見本が目録に無いキーを渡していたら落ちる。

**ここに、呼ぶ側の言い分の名前を置かない。** どんな言い分があり、それをどの部品で
どう組むかという対応表は、呼ぶ側の持ち物である。この目録が答えるのは
「何を受け取れるか」だけで、「何を表せるか」ではない。
"""
from __future__ import annotations

import ast
import inspect
import json
import textwrap

from .boolean import circle_polygon
from .registry import OwnOrigin, known_kinds, render_component, _REGISTRY
from .style import resolve_style
from .tokens import DEFAULT_THEME, PLAIN, ROLE_PREFIX, TOKEN_RANGES

_SLICES = [{"name": "文書", "value": 13}, {"name": "図", "value": 5}]

EXAMPLES: dict[str, dict] = {
    "box": {}, "hex": {}, "dot": {}, "icon": {"name": "check"},
    "donut": {"slices": _SLICES, "centre": "18"},
    "pie": {"slices": _SLICES, "centre": "18"},
    "boolean": {"shapes": [circle_polygon(40, 40, 34), circle_polygon(66, 40, 34)],
                "op": "subtract"},
    "gradient_rect": {"width": 90, "height": 48},
    "path": {"d": "M0,40 Q30,0 60,40 Q90,80 120,40"},
    "titled": {"of": "donut", "slices": _SLICES, "centre": "18"},
    "bars": {"bars": [{"name": "文書", "value": 13}, {"name": "図", "value": 5}]},
    "ranking": {"items": [{"name": "文書", "value": 13}, {"name": "図", "value": 5}]},
    "lanes": {"rows": [{"name": "設計", "bars": [{"from": 0, "to": 3}]}]},
    "scatter": {"points": [{"name": "a", "x": 1, "y": 2}, {"name": "b", "x": 3, "y": 4}]},
    "flow": {"links": [{"from": "A", "to": "B", "value": 5}]},
    "spatial": {"items": [{"name": "領域", "depth": 2}], "cols": 1},
    "table": {"headers": ["部品", "数"], "rows": [["形", "7"]]},
    "exchange": {"participants": ["甲", "乙"],
                 "steps": [{"from": "甲", "to": "乙", "label": "渡す"}]},
    "title": {"text": "見出し"}, "divider": {"width": 120},
    "frame": {"x": 0, "y": 0, "width": 80, "height": 40},
    "frame_label": {"x": 0, "y": 20, "label": "札"},
    "edge": {"points": [(0, 0), (60, 40)]},
}
"""部品ごとの、最小の動く入力。

利用側が「とりあえず1つ描いてみる」ための足がかりであり、同時に目録が実物と
繋がっている証拠でもある。部品を足したらここへ1行足す ── 足し忘れは試験で落ちる。
"""

EXAMPLES = {k: {"label": "名前", **v} for k, v in EXAMPLES.items()}


def _keys_read(fn, param: str) -> dict[str, dict]:
    """関数が、その引数からどの鍵を読んでいるかを構文木から拾う。

    `x["k"]` は必須、`x.get("k")` は任意として扱う ── 既定値を書けるのは
    無くてもよいときだけなので、書き方がそのまま要否を表している。

    Args:
        fn: 対象の関数。
        param: 読み取り元の引数名（部品なら "props"）。

    Returns:
        鍵の名前 → {"required": bool, "default": 既定値の文字列 or None}。
        同じ鍵を両方の書き方で読んでいたら、任意として扱う（1箇所でも
        既定値を持てるなら、無くても動く）。

    Raises:
        OSError: 元のソースが取れないとき（対話環境で定義された関数など）。
    """
    return _keys_read_tree(ast.parse(textwrap.dedent(inspect.getsource(fn))), param)


def _keys_read_tree(tree, param: str) -> dict[str, dict]:
    """構文木から、その名前の変数が読んでいる鍵を拾う（`_keys_read` の本体）。"""
    found: dict[str, dict] = {}

    def note(key: str, required: bool, default=None) -> None:
        cur = found.get(key)
        if cur is None:
            found[key] = {"required": required, "default": default}
        elif not required:
            cur["required"] = False
            if default is not None:
                cur["default"] = default

    for node in ast.walk(tree):
        if (isinstance(node, ast.Subscript) and isinstance(node.value, ast.Name)
                and node.value.id == param and isinstance(node.slice, ast.Constant)
                and isinstance(node.slice.value, str)):
            note(node.slice.value, True)
        if (isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
                and node.func.attr == "get" and isinstance(node.func.value, ast.Name)
                and node.func.value.id == param and node.args
                and isinstance(node.args[0], ast.Constant)
                and isinstance(node.args[0].value, str)):
            dflt = None
            if len(node.args) > 1:
                try:
                    dflt = repr(ast.literal_eval(node.args[1]))
                except (ValueError, SyntaxError):
                    dflt = ast.unparse(node.args[1])
            note(node.args[0].value, False, dflt)
    return dict(sorted(found.items()))


def _keys_read_module(module, param: str) -> dict[str, dict]:
    """モジュール全体から、その名前の変数が読んでいる鍵を拾う。

    関数を1つずつ指定すると、読む場所が増えたときに黙って取りこぼす
    （実際、囲みの `members` は群を畳む関数の中の内包表記で読まれていて、
    入口の関数だけを見ても出てこなかった）。取りこぼしは利用側が必須の鍵を
    知れないことを意味するので、広く取って落とすほうを選ぶ。
    """
    tree = ast.parse(inspect.getsource(module))
    fake = ast.FunctionDef(name="_", args=ast.arguments(
        posonlyargs=[], args=[], kwonlyargs=[], kw_defaults=[], defaults=[]),
        body=tree.body, decorator_list=[])
    return _keys_read_tree(fake, param)


def props_of(kind: str) -> dict[str, dict]:
    """その部品が読む値の一覧。

    Args:
        kind: 部品の名前（`known_kinds()` が返すもの）。

    Returns:
        鍵の名前 → {"required": bool, "default": ...}。

    Raises:
        KeyError: 台帳に無い名前のとき。
    """
    if kind not in _REGISTRY:
        raise KeyError(f"台帳に無い部品です: {kind}（使えるのは {known_kinds()}）")
    return _keys_read(_REGISTRY[kind], "props")


def forwards_of(kind: str) -> str | None:
    """その部品が、受け取った値をそのまま別の部品へ渡すなら、その渡し先。

    渡す部品がある ── `pie` は輪そのものを `donut` に描かせ、`titled` は
    `of` で指された部品を包む。どちらも自分では読まない鍵を受け取れるので、
    渡し先を書かないと利用側は「渡せる鍵」を知れない（実際、`pie` の説明は
    `centre` を受けると書いていたが、読んでいたのは渡した先だった）。

    Returns:
        渡し先の部品名。渡し先が値で決まるなら `"props:<鍵>"`。渡さないなら None。
    """
    tree = ast.parse(textwrap.dedent(inspect.getsource(_REGISTRY[kind])))
    for node in ast.walk(tree):
        if not (isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
                and node.func.id in ("render_component", "render_node")
                and len(node.args) >= 2):
            continue
        # 第2引数が props そのものでなければ、組み直して渡している＝素通しではない
        if not (isinstance(node.args[1], ast.Name) and node.args[1].id == "props"):
            continue
        target = node.args[0]
        if isinstance(target, ast.Constant) and isinstance(target.value, str):
            return target.value
        if (isinstance(target, ast.Subscript) and isinstance(target.value, ast.Name)
                and target.value.id == "props" and isinstance(target.slice, ast.Constant)):
            return "props:" + str(target.slice.value)
    return None


def accepted_props(kind: str, _seen: frozenset[str] = frozenset()) -> dict[str, dict]:
    """その部品へ渡せる値の全部。素通しの先が読むものも含む。

    渡し先が値で決まる部品（`titled`）は、先が定まらないので自分の分だけを返す
    ── 利用側は `of` で指す部品の目録を併せて見る。
    """
    out = dict(props_of(kind))
    nxt = forwards_of(kind)
    if nxt and not nxt.startswith("props:") and nxt not in _seen:
        for k, v in accepted_props(nxt, _seen | {kind}).items():
            out.setdefault(k, v)
    return dict(sorted(out.items()))


def parts() -> dict[str, dict]:
    """部品ごとの、受け取る値・置き方・名前を自分で描くか・見本の大きさ。

    置き方と名前の申告は、実際に見本で描いて確かめた結果を載せる ── 部品が
    返すものなので、呼ばずには分からない。
    """
    out: dict[str, dict] = {}
    for kind in known_kinds():
        entry: dict = {"props": accepted_props(kind), "reads_itself": sorted(props_of(kind))}
        fwd = forwards_of(kind)
        if fwd:
            entry["forwards_to"] = fwd
        example = EXAMPLES.get(kind)
        if example is not None:
            r = render_component(kind, example, resolve_style())
            entry["example"] = {k: v for k, v in example.items()}
            entry["placement"] = ("own-origin" if isinstance(r, OwnOrigin)
                                  else "absolute")
            entry["labels_itself"] = r.labels_itself
            entry["example_size"] = [round(r.width, 1), round(r.height, 1)]
        out[kind] = entry
    return out


def tokens() -> dict[str, dict]:
    """見た目を決める値の一覧。範囲を持つものは、その範囲も添える。"""
    return {k: {"default": v, "range": list(TOKEN_RANGES[k]) if k in TOKEN_RANGES else None}
            for k, v in sorted(DEFAULT_THEME.items()) if not k.startswith(ROLE_PREFIX)}


def roles() -> dict[str, dict]:
    """役割ごとの、上書きするトークン。テーマへ行を足せば増える。"""
    out: dict[str, dict] = {PLAIN: {}}
    for k, v in DEFAULT_THEME.items():
        if not k.startswith(ROLE_PREFIX):
            continue
        name, _, token = k[len(ROLE_PREFIX):].partition(".")
        out.setdefault(name, {})[token] = v
    return dict(sorted(out.items()))


def _merge(*parts: dict[str, dict]) -> dict[str, dict]:
    """同じ鍵を複数の場所で読んでいたら、1つにまとめる。任意が優先。"""
    out: dict[str, dict] = {}
    for part in parts:
        for key, info in part.items():
            cur = out.get(key)
            if cur is None:
                out[key] = dict(info)
            elif not info["required"]:
                cur["required"] = False
                cur["default"] = cur["default"] or info["default"]
    return dict(sorted(out.items()))


def declaration() -> dict[str, dict]:
    """図の宣言そのものが受け取る鍵。読んでいる実物すべてから拾う。

    1つの関数だけを見ると足りない ── 囲みの `members` は合成では読まれず、
    群を畳む側の内包表記の中で読まれている。だから群についてはモジュール全体を
    走査する。
    """
    from . import compose, nesting
    return {"nodes": _keys_read(compose.figure_fragment, "n"),
            "edges": _keys_read(compose.figure_fragment, "e"),
            "groups": _merge(_keys_read(compose.figure_fragment, "g"),
                             _keys_read_module(nesting, "g"))}


def strategies() -> dict[str, str]:
    """選べる配置戦略と、それが何を根拠に位置を決めるか。"""
    return {"layout_graph": "辺の向きから段を決める（層状）",
            "layout_radial": "順が巡って戻る（環状）",
            "layout_tree": "中心から枝分かれする（放射の木）",
            "layout_grid": "宣言が持つ座標のとおりに置く（格子）"}


def catalog() -> dict:
    """目録の全体。これ1つで、利用側は変換器を書ける。"""
    return {"declaration": declaration(), "parts": parts(),
            "tokens": tokens(), "roles": roles(), "strategies": strategies()}


def main() -> None:
    """目録をJSONで吐く。言語を問わず読めるようにするため。"""
    print(json.dumps(catalog(), ensure_ascii=False, indent=2, default=str))


if __name__ == "__main__":
    main()
