"""抽象度の並びを描く共通の型。1本目と5本目が同じ形を使う。"""
from lesson_visuals import (text, rect, path, circle, svg, INK, DIM, LINE, PANEL, PAPER, ACCENT, W, band)

AX = 210
TOP, ROW, GAP = 24, 84, 24


def axis(rows, caption, note, poles, height=424):
    """左に抽象と具体、右に3つのプロンプト。rows は (プロンプト, 当てはまる範囲, 判定, 注記, 強調) の3件である。"""
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
    """縦の軸が抽象度である。上が抽象、下が具体。ちょうどいい高さは、そのあいだのどこか（目的で決まる）。"""
    X = 190                           # 軸の位置
    a = path(f'M{X} 348 V20 M{X - 7} 30 l7 -10 7 10', LINE, 2)
    # 軸の名前は上端に置く ── 中ほどに置くと、中ほどの1点が抽象度と読める
    a += text(X - 12, 24, '抽象度', 14, INK, 700, 'end')
    a += text(X - 12, 44, '高い', 13, DIM, anchor='end')
    a += text(X - 12, 348, '低い', 13, DIM, anchor='end')
    # 左の大きな2語が、この枚で覚えてほしい語である
    a += text(0, 72, '抽象', 26, INK, 700)
    a += text(0, 100, ['決めていない', 'ことが多い'], 14, DIM, gap=20)
    a += text(0, 300, '具体', 26, INK, 700)
    a += text(0, 328, '細かく決めてある', 14, DIM)
    rows = [(30, '決めていないプロンプト', '「課題を整理して」', ['実行するたびに変わる', 'どの週の議事録にも使える']),
            (266, '細かく決めたプロンプト', '「1行目に承認の経路、2行目に申請の締め日」', ['1つに決まる', 'この議事録にしか使えない'])]
    for y, kind, instr, facts in rows:
        a += circle(X, y + 42, 8, PAPER, LINE)
        a += rect(230, y, 882, 84, PAPER, 10, LINE)
        a += text(254, y + 30, kind, 13, DIM)
        a += text(254, y + 60, instr, 18, INK, 700)
        for i, f in enumerate(facts):
            a += text(1088, y + 34 + i * 26, f, 14, INK, anchor='end')
    # あいだ：ちょうどいい高さは、点ではなく範囲で示す。どこにあるかは目的で決まり、まだ言えない
    a += rect(X - 5, 134, 10, 112, ACCENT, 5, ACCENT)
    a += (f'<rect x="230" y="148" width="882" height="84" rx="10" fill="{PAPER}" stroke="{ACCENT}" '
          'stroke-width="2" stroke-dasharray="7 6"/>')
    a += text(254, 184, '目的に合う、ちょうどいい高さ', 18, ACCENT, 700)
    a += text(254, 212, 'このあいだのどこかにある。どこにあるかは目的で決まる', 14, INK)
    return svg('縦の軸が抽象度である。上が抽象、下が具体。ちょうどいい高さは、そのあいだのどこか', a, 356)


def h5_axis():
    """L6-S3　原因を知る動画で「？」だった作業の目的の段に、いまのプロンプトが入る。L1-S2 と同じ形にそろえる。"""
    from v1_visuals import _offaxis_row, _axis_rows
    a = _offaxis_row(0, '「課題を整理して」', '誰が何のために整理するかが書かれていない', (True, 'どの週にも使える'), (False, '毎回変わる'))
    a += _axis_rows([('いまのプロンプト', '1行目の目的 ＋ 意味 ・ 範囲 ・ 条件', (True, 'どの週にも使える'), (True, '目的から外れない'), True, False),
                     ('「1行目に承認の経路、2行目に申請の締め日」', '低すぎる目的', (False, 'この議事録だけ'), (True, '1つに決まる'), False, False)])
    return svg('原因を知る動画で「？」だった作業の目的の段に、1行目の目的と3つの決まりを書いたいまのプロンプトが入る。どの週にも使えて、目的から外れない', a, 376)