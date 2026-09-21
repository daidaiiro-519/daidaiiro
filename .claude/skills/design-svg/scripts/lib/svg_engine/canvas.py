"""自由配置の合成 ── グラフのレイアウト解決を経由しない、もう一つの組み立て方。

`render_figure`（関係を自動配置する）・`render_chart`（値から単独で描く）が
「意味を持つ図」を組むための道なら、こちらは「1枚の絵」を組むための道。
背景・見出し・複数の部品を、好きな位置へ重ねて置けるだけの、薄い合成層。

レイヤーは木構造にできる ── 1つのレイヤーが`children`を持てば、それは
部品ではなく「グループ」として扱われ、自分の変形を子ぶんまとめてかける
（Illustratorの『グループ化して入れ子で変形する』に相当）。
レイヤーに`clip`を持たせると、その中身を指定した形で切り抜く
（クリッピングマスクに相当）。
"""
from __future__ import annotations


from .ids import stable_id
from .registry import render_component
from .style import resolve_style
from .tokens import DEFAULT_THEME


def _transform_of(layer: dict) -> str:
    parts = []
    if layer.get("x") or layer.get("y"):
        parts.append(f'translate({layer.get("x", 0):.1f},{layer.get("y", 0):.1f})')
    if layer.get("rotate"):
        parts.append(f'rotate({layer["rotate"]})')
    if layer.get("scale"):
        parts.append(f'scale({layer["scale"]})')
    return " ".join(parts)


def _render_layer(layer: dict, theme: dict) -> str:
    """1つのレイヤーを描く。`children`があればグループとして再帰する。"""
    if "children" in layer:
        inner = "".join(_render_layer(child, theme) for child in layer["children"])
    else:
        style = resolve_style(layer.get("role", "plain"), layer.get("style"), theme)
        r = render_component(layer["kind"], layer["props"], style)
        inner = r.svg

    transform = _transform_of(layer)
    g_attrs = f' transform="{transform}"' if transform else ""

    clip = layer.get("clip")
    if clip:
        # 実測で見つかった制約: このレンダラー(resvg)は、clipPathの中身が
        # <g>で包まれていると(transformの有無を問わず)クリップを丸ごと
        # 無視する。位置合わせの変形は、内側の<g>ではなく<clipPath>要素
        # 自身のtransform属性へ置く。だからclip側にはbox/pie等の内部で
        # <g>を持つ部品ではなく、裸の図形を返す部品(dot/path等)だけを使う。
        cid = stable_id("clip", clip.get("kind"), clip.get("props"),
                         clip.get("role"), clip.get("style"), _transform_of(clip))
        clip_style = resolve_style(clip.get("role", "plain"), clip.get("style"), theme)
        clip_r = render_component(clip["kind"], clip["props"], clip_style)
        clip_transform = _transform_of(clip)
        transform_attr = f' transform="{clip_transform}"' if clip_transform else ""
        defs = f'<clipPath id="{cid}"{transform_attr}>{clip_r.svg}</clipPath>'
        return f'<defs>{defs}</defs><g{g_attrs} clip-path="url(#{cid})">{inner}</g>'

    return f'<g{g_attrs}>{inner}</g>' if g_attrs else f'<g>{inner}</g>'


def render_canvas(width: float, height: float, layers: list[dict],
                   theme: dict | None = None, background: str | None = None) -> str:
    """好きな位置へ部品を重ねて、1枚のSVGに合成する。

    Args:
        width, height: 画布の大きさ。
        layers: 各要素は次のいずれか。
            - 部品レイヤー: {"kind": 部品名, "x": num, "y": num, "props": dict,
              "role": str(任意), "style": dict(任意), "rotate": num(任意, 度),
              "scale": num(任意), "clip": {"kind":..., "props":...}(任意)}
            - グループレイヤー: {"x":num, "y":num, "children": [layer, ...],
              "rotate": num(任意), "scale": num(任意), "clip": {...}(任意)}
              ── 自分の変形が子レイヤー全部にまとめてかかる。
        theme: DEFAULT_THEME を上書きするテーマ。
        background: 画布全体の背景色（トークン解決はしない。直値かCSS色名）。

    Returns:
        `<svg>...</svg>` 文字列。
    """
    theme = theme or DEFAULT_THEME
    body = []
    if background:
        body.append(f'<rect x="0" y="0" width="{width:.0f}" height="{height:.0f}" fill="{background}"/>')
    for layer in layers:
        body.append(_render_layer(layer, theme))
    return (f'<svg class="wf-fig" viewBox="0 0 {width:.0f} {height:.0f}" width="{width:.0f}" '
            f'height="{height:.0f}" role="img">{"".join(body)}</svg>')
