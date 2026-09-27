"""抽象の高さの並びを描く共通の型。1本目と5本目が同じ形を使う。"""
from lesson_visuals import (text, rect, path, circle, svg, INK, DIM, LINE, PANEL, PAPER, ACCENT, W, band)

AX = 210
TOP, ROW, GAP = 24, 84, 24


def axis(rows, caption, note, poles, height=424):
    """左に抽象と具体、右に3つの指示。rows は (指示, 当てはまる範囲, 判定, 注記, 強調) の3件である。"""
    bottom = TOP + 3 * (ROW + GAP) - GAP
    a = path(f'M{AX} {bottom} V{TOP} M{AX - 7} {TOP + 10} l7 -10 7 10', LINE, 2)
    a += text(AX + 16, TOP - 4, '高い', 15, ACCENT, 700)
    a += text(AX + 16, bottom + 20, '低い', 15, ACCENT, 700)

    a += rect(0, TOP, 176, 130, PANEL, 12)
    a += text(88, TOP + 50, '抽象', 30, ACCENT, 700, 'middle')
    a += text(88, TOP + 84, poles[0][0], 15, DIM, anchor='middle')
    a += text(88, TOP + 106, poles[0][1], 15, DIM, anchor='middle')
    a += rect(0, bottom - 130, 176, 130, PANEL, 12)
    a += text(88, bottom - 80, '具体', 30, ACCENT, 700, 'middle')
    a += text(88, bottom - 46, poles[1][0], 15, DIM, anchor='middle')
    a += text(88, bottom - 24, poles[1][1], 15, DIM, anchor='middle')

    for i, (ask, where, judge, tail, hot) in enumerate(rows):
        y = TOP + i * (ROW + GAP)
        a += circle(AX, y + 42, 12, ACCENT if hot else PAPER, ACCENT if hot else LINE)
        a += rect(266, y, 846, ROW, PAPER, 12, ACCENT if hot else LINE)
        a += text(294, y + 36, ask, 20, ACCENT if hot else INK, 700)
        a += text(294, y + 64, where, 15, DIM)
        a += text(1084, y + 36, judge, 16, ACCENT if hot else DIM, 700, 'end')
        if tail:
            a += text(1084, y + 64, tail, 15, DIM, anchor='end')
    a += band(bottom + 32, note)
    return svg(caption, a, height)


def h1_axis():
    """1本目 ── 真ん中は、まだ決まっていない。"""
    return axis([
        ('「課題を整理して」', 'どの議事録にも使える',
         '高すぎる', '何が返ってくるか決まらない', False),
        ('これから決める指示', 'この6本の動画で探していきます', 'ちょうどいい抽象の高さ', '', True),
        ('「1行目に承認の経路、2行目に申請の締め日」', 'この議事録の、この1回だけ',
         '低すぎる', '来週の議事録には使えない', False),
    ], '指示を、決めていないことの多い順に3つ並べる。真ん中はこれから探す',
       'この並びのどこにあるかが、抽象の高さである',
       (('決めていない', 'ことが多い'), ('細かく', '決めてある')))


def h5_axis():
    """5本目 ── 真ん中が埋まった。"""
    return axis([
        ('「課題を整理して」', 'どの議事録にも使える',
         '高すぎる', '何が返ってくるか決まらない', False),
        ('いまの指示', '意味・範囲・条件の3つを書いた', 'ちょうどいい抽象の高さ',
         '渡すものを替えても目的の中に収まる', True),
        ('「1行目に承認の経路、2行目に申請の締め日」', 'この議事録の、この1回だけ',
         '低すぎる', '来週の議事録には使えない', False),
    ], '高すぎる側と低すぎる側の間に、いまの指示がある',
       '個別の項目は書いていない。決まりごとだけを書いている',
       (('決めていない', 'ことが多い'), ('細かく', '決めてある')))
