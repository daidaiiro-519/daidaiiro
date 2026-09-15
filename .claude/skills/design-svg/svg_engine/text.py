"""文字幅の見積り ── shapes.py と labels.py の両方が使うので、ここへ独立させる。

この見積りは**必ず実物以上**でなければならない。用途は「場所を取り置くこと」
（箱の幅・欄の幅・重なりの判定）で、少なく見積もれば文字が枠から出る。
だから比は平均ではなく、実測した中の最大を使う。
"""
from __future__ import annotations

from .tokens import DEFAULT_THEME

# テーマから引いた既定値。鍵が数を返すことは呼ぶ側が知っている。
_LATIN_RATIO = float(DEFAULT_THEME["font.latin-width-ratio"])  # type: ignore[arg-type]

# CJKの文字が始まる符号位置（Unicodeが決めている境目。選んだ値ではない）。
_CJK_START = 0x2E80


def text_width(s: str, size: float,
               latin_ratio: float = _LATIN_RATIO) -> float:
    """CJKは全角、それ以外は半角相当として幅を見積もる。

    既定値はテーマから引く。同じ数を2箇所に書くと、片方だけ直したときに
    見積りがずれる（実際に 0.58 が2箇所にあり、全角大文字を10%見誤っていた）。
    """
    return sum(size if ord(c) > _CJK_START else size * latin_ratio for c in str(s))


def column_width(texts, size: float, pad: float,
                 latin_ratio: float = _LATIN_RATIO) -> float:
    """文字が並ぶ欄の幅を、実際に入る文字から決める。

    欄の幅を決め打ちにすると、中身が短いときは無駄な空白が空き（円グラフの
    凡例で、値が項目名から遠く離れて見えた）、長いときははみ出す（空間の図で
    固定220の欄から名前がはみ出し、装飾と衝突した）。同じ「文字の欄」が
    5箇所で別々の定数を持っていたので、決め方をここへ1つにまとめる。

    Args:
        texts: 欄に入る文字（数値でもよい）。
        size: 文字の大きさ。
        pad: 欄の左右に空ける余白の合計。
        latin_ratio: 半角文字の幅の比。

    Returns:
        欄の幅。空なら pad だけ。

    Raises:
        なし。
    """
    widest = max((text_width(str(t), size, latin_ratio) for t in texts), default=0.0)
    return widest + pad
