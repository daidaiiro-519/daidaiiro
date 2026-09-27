"""5本目（揺らぎを直す）の試作の図。直すときも、決めたときと同じ順で見る。"""
from lesson_visuals import (text, rect, path, circle, arrow, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, band)

STEPS = [('意味', '何を並べるかが書いてあるか', '並ぶものの種類が変わるとき'),
         ('範囲', '渡したもののどこまでを見るかが書いてあるか', '拾ってくる量が変わるとき'),
         ('条件', '必ず入れるもの・落とすもの・並べる順が書いてあるか', '中身が欠ける・書き方が揃わないとき')]


def f5_order():
    """決めたときと同じ順で、上から見る。"""
    a = text(0, 22, '書き足したときと同じ順で、上から見る', 17, DIM)
    a += text(768, 22, 'ここを疑う手がかり', 17, ACCENT, 700)
    for i, (name, ask, sign) in enumerate(STEPS):
        y = 38 + i * 88
        a += rect(0, y, 1112, 72, PANEL if i else PAPER, 12, ACCENT if i == 0 else 'none')
        a += circle(44, y + 36, 17, ACCENT, ACCENT)
        a += text(44, y + 43, str(i + 1), 18, PAPER, 700, 'middle')
        a += text(84, y + 32, name, 21, INK, 700)
        a += text(168, y + 32, ask, 17, DIM)
        a += path(f'M744 {y + 16} V{y + 56}', LINE, 1)
        a += text(768, y + 32, sign, 16, INK)
        if i < 2:
            a += path(f'M44 {y + 72} V{y + 88} M38 {y + 82} l6 6 6 -6', LINE, 2)
    a += band(310, '書き足したときの順と、直すときの順は同じである')
    return svg('直すときも、意味・範囲・条件の順に上から見る', a, 378)
