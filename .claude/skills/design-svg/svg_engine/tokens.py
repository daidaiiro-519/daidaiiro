"""デザイントークン ── CSSの`:root{--var}`に相当する。

ここに置くのは名前と値の対応だけ。色・寸法・角丸・書体は、コンポーネントの
コードに直書きしない。テーマを差し替えれば、全部品の見た目が一括で変わる。

値に意味のある範囲がある寸法トークン（負になれない・0だと崩れる・大きすぎる
と他の要素と衝突する等）は、TOKEN_RANGES にその範囲を持たせる。style.py の
resolve_style() がテーマ差し替え・インライン上書きのたびにこの範囲へ照らして
検査するので、範囲外の値を渡すと「レンダリングしてから崩れて気づく」のではなく
その場で例外になる。
"""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any

# 既定テーマ。figure-notation.html / figure-composition.html で使われてきた
# 配色を踏襲しつつ、ここでは「トークン」として名前を持たせる。
# トークンの値。色・書体名・部品名は文字列、寸法は数、系列の色は文字列の並び。
# 鍵ごとに型が決まっているが、鍵は122個あるので型では書き分けない ── 使う側は
# どの鍵が何を返すかを知っている（規約2で段の型を入れるときに、ここも型で縛る）。
TokenValue = str | int | float | list[str]

DEFAULT_THEME: dict[str, TokenValue] = {
    # 色 ── 役割(role)ごとの塗り・線
    "color.ink": "#171B23",
    "color.ink-soft": "#4B5563",
    "color.ink-faint": "#79828F",
    "color.line": "#C3CAD2",
    "color.box-fill": "#F5F7F9",
    "color.box-stroke": "#C3CAD2",
    "color.accent": "#16636B",
    "color.accent-bg": "#E2EFF0",
    "color.warn": "#9A4A21",
    "color.warn-bg": "#F7EAE2",

    # 寸法 ── コンポーネントの既定サイズ・余白
    "size.box-min-w": 60,
    "size.box-h": 32,
    "size.box-pad-x": 14,
    "size.box-radius": "size.radius",  # 尺度を指す。同じ概念に数を2つ置かない
    "size.gap-rank": 64,
    "size.gap-order": 24,
    "size.stroke-width": 1.2,
    "size.stroke-width-focus": 1.6,
    # 矢じりは線の終端なので、書体ではなく線幅に比例させる
    # （SVGのmarkerUnits="strokeWidth"と同じ考え方）。
    "size.arrowhead-w-ratio": 4.0,
    "size.arrowhead-len-ratio": 1.5,
    "size.arrowhead-min": 4.0,
    # 画布の余白と、文字の左右に取る余白。
    "size.canvas-pad": 12.0,
    "size.canvas-pad-tight": 4.0,
    "size.label-pad-x": 8.0,
    "size.label-line-h": 1.6,
    "size.frame-pad-ratio": 1.4,
    # 矢じりの開き（度）。形の選択なので、大きさとは別に持つ。
    "size.arrowhead-angle": 20.0,
    # 書体の計量 ── 中央に置いた文字のベースラインを、中心からどれだけ下げるか。
    # 半角の字幅が全角の何倍か。環境で実測できないので近似として持つ。
    "font.baseline-ratio": 0.36,
    # 字面の高さ（ベースラインから上へ、大文字・漢字が占める割合）。
    # 文字を帯の中で縦に揃えるとき、行の高さではなくこの高さを使う。
    "font.cap-ratio": 0.72,
    # 下ばね（ベースラインから下へ、かな・記号が出る割合）。
    "font.descender-ratio": 0.22,
    # 実測（Chromium・テーマの書体・10種の文字列）で 0.224〜0.644。
    # 見積りは場所の取り置きに使うので、平均(0.494)ではなく最大を採る。
    # 0.58 だったときは全角大文字を10%少なく見積もっていた。
    "font.latin-width-ratio": 0.65,
    # どの役割にどの部品を使うか。コアは名前を持たず、ここから引く。
    # 別の見た目にしたければ、部品を台帳へ足してここを差し替える。
    "parts.node": "box",
    "parts.edge": "edge",
    "parts.group": "frame",

    # 囲みのラベルを、枠の上へ乗せる札の形。
    "size.frame-label-rise": 0.9,     # 枠の線から、札の上端までを文字の何倍上げるか
    "size.frame-label-h": 1.5,        # 札の高さを文字の何倍にするか
    "size.frame-label-baseline": 1.1,  # 札の上端から、文字のベースラインまで

    # 書体
    "font.family": "Noto Sans JP, Hiragino Kaku Gothic ProN, sans-serif",
    "font.size": 12,
    "font.size-small": 10,
    "font.size-display": 29,          # 見出し。書体の大きさそのもので、比率ではない
    # 輪郭をインクから導くときの細かさ。向きをいくつに分けるか。刻み幅も
    # 曲線の分割数も、部品の大きさとこの値から導かれる。
    "size.outline-facets": 32,
    # 図の入れ子の深さの上限。段1 が「入れ子は深さに上限を置く」と定めている。
    "size.figure-depth-limit": 3,
    "size.dot-radius": 5,
    "size.label-band-h": 16,          # 辺の札の帯の高さ
    "size.decor-title-w": 640,        # 装飾の見出しの既定の幅
    "size.decor-icon": 24,
    "size.divider-amp": 3,            # 区切り線の波の高さ
    "size.divider-period": 18,        # 同・波の周期

    # 量を描く部品（円・棒・散布図など）が使う色の並び。系列や区分が増えたら
    # 順に使う。件数が上限を超えたら呼び出し側が同じ色を繰り返す。
    "chart.tones": ["#16636B", "#9A4A21", "#7A4368", "#8A8F98"],
    "chart.grid": "#E4E9EE",
    "chart.axis": "#C3CAD2",
    "chart.pad": 18,
    # 要素どうしの隙間の基準。個別の図ごとに数を置かず、ここから引く。
    "chart.gap": 10,
    "chart.legend-row-h": 26,
    "chart.rank-track-h": 14,
    "chart.lane-bar-inset": 6,
    "chart.lane-axis-h": 22,
    "chart.flow-node-gap": 8,
    "chart.ribbon-opacity": 0.45,
    "chart.table-pad-x": 14,
    "chart.table-row-h": 30,
    "chart.exchange-tab-h": 15,

    # pie（全体と部分）
    "chart.pie-radius": 52,
    "chart.pie-margin": 18,
    "chart.pie-donut-thickness": 24,

    # bars（量の大小・分布・偏差）
    "chart.bar-width": 62,
    "chart.bar-gap": 26,
    "chart.bar-plot-h": 150,
    "chart.bar-left-margin": 44,
    "chart.bar-bottom-margin": 34,

    # ranking（順位）
    "chart.rank-row-h": 30,
    "chart.rank-bar-w": 300,

    # lanes（時間変化）
    "chart.lane-track-w": 380,
    "chart.lane-row-h": 30,

    # scatter（相関）
    "chart.scatter-w": 420,
    "chart.scatter-h": 300,
    "chart.scatter-margin-left": 90,
    "chart.scatter-margin-top": 30,
    "chart.scatter-margin-bottom": 40,
    "chart.scatter-point-r": 4.5,
    "chart.scatter-edge-threshold": 50,

    # flow（流量）
    "chart.flow-w": 520,
    "chart.flow-h": 210,
    "chart.flow-margin": 90,
    "chart.flow-bar-w": 12,

    # spatial（空間）
    "chart.spatial-row-h": 42,
    "chart.spatial-gap": 10,
    "chart.spatial-pad-x": 20,

    # exchange（やり取り）
    "chart.exchange-col-w": 150,
    "chart.exchange-head-h": 40,
    "chart.exchange-row-h": 40,
    "chart.exchange-case-head-h": 28,
    "chart.exchange-box-h": 28,
    "chart.exchange-arrow-len": 6,

    # 自分へ戻る辺が作る輪の形。節点の高さと輪の半径に対する比で持つので、
    # 大きさが変わっても形が保たれる。
    "size.self-loop-attach": 0.3,   # 節点のどの高さから出入りするか（上端からの比）
    "size.self-loop-bulge": 1.2,    # 輪がいちばん外へ張り出す量（半径の何倍か）
    "size.self-loop-lift": 0.4,     # 出入り口の脇で、上下へどれだけ持ち上げるか

    # 尺度 ── 部品ごとに数を持たせず、段階に名前を付けて選ばせる。
    # 個別に名前を付けると「直書きの数」が「トークンという名の直書きの数」に
    # 変わるだけで、テーマを差し替えても全体の調子が揃わない。段階にしておけば、
    # 3つ動かすだけで図全体の角の丸みや線の重さが一斉に変わる。
    "size.radius-small": 3,     # 小さい要素（棒・升目）
    "size.radius": 5,           # 既定（箱）
    "size.radius-large": 8,     # 大きい容器（囲み）
    "size.stroke-width-thin": 1.1,   # 補助の線（囲みの破線）
    "size.rule-width": 2,            # 装飾の罫・下線
    "size.stroke-width-icon": 2.4,   # アイコンの線（24×24の座標系で描く）
    "font.weight-normal": "400",
    "font.weight-medium": "600",
    "font.weight-bold": "700",
    "opacity.soft": 0.75,       # 控えめに置く要素（囲み）
    "opacity.faint": 0.35,      # 背景へ沈める要素（図表の格子）

    # 役割(role) ── CSSの`.focus{...}`に相当する、トークンのひとまとまりの上書き。
    # `role.<役割名>.<トークン名>` という平らな名前で、他のトークンと同じ袋に置く。
    #
    # 別の入れ物に分けない。分けると、見た目を差し替える経路がトークンと役割で
    # 2本になり、テーマを渡しても役割だけは差し替えられない、という非対称が
    # 生まれる（実際そうなっていた）。同じ袋なら、テーマを1つ渡すだけで
    # 「どんな役割があるか」ごと入れ替わる。
    #
    # 新しい役割は、ここへ行を足すだけで増える。エンジンには触れない。
    "role.focus.color.box-fill": "color.accent-bg",   # 値がトークン名でもよい
    "role.focus.color.box-stroke": "color.accent",
    "role.focus.color.text": "color.accent",
    "role.focus.size.stroke-width": "size.stroke-width-focus",
    "role.focus.font.weight": "font.weight-medium",

    "role.muted.color.box-fill": "none",
    "role.muted.color.box-stroke": "color.ink-faint",
    "role.muted.color.text": "color.ink-faint",
    "role.muted.stroke-dasharray": "4 3",
}

ROLE_PREFIX = "role."
PLAIN = "plain"  # 何も上書きしない役割。常に有効

# 値に妥当な範囲があるトークンだけ、ここへ (最小, 最大) を持たせる。
# 無いトークンは無制限（色・文字列トークンや、崩れても致命的でない値）。
TOKEN_RANGES: dict[str, tuple[float, float]] = {
    "size.stroke-width": (0.4, 6.0),
    "size.stroke-width-focus": (0.4, 8.0),
    "size.stroke-width-thin": (0.2, 4.0),
    "size.rule-width": (0.4, 8.0),
    "size.stroke-width-icon": (0.4, 8.0),
    "size.self-loop-attach": (0.05, 0.45),
    "size.self-loop-bulge": (1.0, 3.0),
    "size.self-loop-lift": (0.0, 1.5),
    "size.radius-small": (0.0, 20.0),
    "size.radius": (0.0, 30.0),
    "size.radius-large": (0.0, 60.0),
    "opacity.soft": (0.0, 1.0),
    "opacity.faint": (0.0, 1.0),
    "size.box-radius": (0.0, 30.0),
    "size.box-h": (16.0, 80.0),
    "font.size": (8.0, 40.0),
    "font.size-small": (6.0, 32.0),
    "size.outline-facets": (8, 256),
    "size.figure-depth-limit": (1, 8),
    "chart.pad": (4.0, 60.0),
    "chart.pie-radius": (16.0, 200.0),
    "chart.pie-donut-thickness": (4.0, 60.0),
    "chart.bar-width": (10.0, 160.0),
    "chart.bar-plot-h": (40.0, 500.0),
    "chart.rank-row-h": (16.0, 80.0),
    "chart.lane-row-h": (16.0, 80.0),
    "chart.scatter-point-r": (2.0, 14.0),
}


def num(theme: dict, key: str) -> float:
    """数を返す鍵から、数として取り出す。

    トークンの値は鍵ごとに型が決まっているが、鍵が122個あるので型では書き分け
    ていない。数として使う場所でこれを通すと、静的検査が通るだけでなく、
    差し替えたテーマが数のはずの鍵へ文字列を入れていた場合に、描いて崩れる前に
    その場で落ちる。

    Args:
        theme: テーマ、または解決済みのスタイル。
        key: トークン名。

    Returns:
        数。

    Raises:
        KeyError: その鍵がテーマに無いとき。
        TypeError: その鍵の値が数でないとき。
    """
    v = theme[key]
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        raise TypeError(f"トークン '{key}' は数のはずだが {type(v).__name__} が入っている: {v!r}")
    return float(v)


@dataclass(frozen=True)
class Style:
    """解決済みの見た目 ── 鍵ごとに型が決まっているので、取り出すときに型を選ぶ。

    辞書のまま渡していた頃は、値が「文字列か数か文字列の並び」のどれかという
    ことしか型に書けず、数として使う場所で静的検査が通らなかった。取り出し方に
    型を持たせると、鍵と型の対応が呼ぶ側のコードに現れる。

    鍵で引く以外の使い方（反復・複製・展開）は持たせない。解決済みの見た目は
    「引いて使うもの」であって、組み替えるものではない ── 組み替えたいなら
    resolve_style() をもう一度呼ぶ。
    """

    values: dict[str, Any]

    def num(self, key: str) -> float:
        """数を返す鍵から取り出す。

        Raises:
            KeyError: その鍵が無いとき。
            TypeError: その鍵の値が数でないとき。
        """
        v = self.values[key]
        if isinstance(v, bool) or not isinstance(v, (int, float)):
            raise TypeError(f"トークン '{key}' は数のはずだが {type(v).__name__}: {v!r}")
        return float(v)

    def text(self, key: str, default: str | None = None) -> str:
        """文字列を返す鍵から取り出す。既定値を渡すと、鍵が無いときにそれを返す。

        Raises:
            KeyError: その鍵が無く、既定値も渡されていないとき。
            TypeError: その鍵の値が文字列でないとき。
        """
        if key not in self.values:
            if default is None:
                raise KeyError(f"トークン '{key}' がテーマに無い")
            return default
        v = self.values[key]
        if not isinstance(v, str):
            raise TypeError(f"トークン '{key}' は文字列のはずだが {type(v).__name__}: {v!r}")
        return v

    def opt_text(self, key: str) -> str | None:
        """あれば文字列、無ければ None。破線の刻みのように、無いことが既定の鍵に使う。"""
        v = self.values.get(key)
        return v if isinstance(v, str) else None

    def tones(self, key: str) -> list[str]:
        """文字列の並びを返す鍵から取り出す（系列の色など）。

        Raises:
            KeyError: その鍵が無いとき。
            TypeError: その鍵の値が文字列の並びでないとき。
        """
        v = self.values[key]
        if not isinstance(v, list):
            raise TypeError(f"トークン '{key}' は並びのはずだが {type(v).__name__}: {v!r}")
        return list(v)
