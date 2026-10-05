"""4本目（条件を決める）の試作の図。どの条件が、各項目のどこを作るか。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)

ROWS = [('承認の経路', '経理と開発', '9/30'),
        ('申請の締め日', '経理', '9/30'),
        ('旧システムの停止日', 'PM', '10/7')]


def s4_effect():
    """L4-S4　条件を3文で追記したプロンプト（主役）と、条件が範囲の中でしか成り立たないこと。"""
    from v1_visuals import _sheet
    # 左：追記したあとのプロンプト
    a = _sheet(0, 0, 620, 330)
    a += text(24, 34, '追記したあとのプロンプト', 14, DIM, 700)
    for k, (tag, body) in enumerate([('＋意味', '決定しないと案件の進行が止まるもの'), ('＋範囲', '今回の案件、12月末の切り替えまで')]):
        y = 50 + k * 40
        a += rect(24, y, 88, 28, PAPER, 8, LINE) + text(68, y + 19, tag, 14, DIM, 700, 'middle')
        a += text(128, y + 19, body, 15, DIM)
    a += rect(24, 136, 88, 28, ACCENT, 8, ACCENT) + text(68, 155, '＋条件', 14, PAPER, 700, 'middle')
    a += rect(24, 176, 572, 136, PAPER, 10, ACCENT)
    for k, (tag, lines) in enumerate([('入れる', ['どの項目にも、誰が決めるかと、いつまでかを', '必ず入れてください。']),
                                      ('落とす', ['すでに決まったものは含めないでください。']),
                                      ('並べる', ['決定の期限の早い順に並べてください。'])]):
        y = [192, 250, 282][k]
        a += text(44, y + 15, tag, 14, ACCENT, 700)
        a += text(112, y + 15, lines, 17, INK, 700, gap=24)
    # 右：条件は範囲の中で成り立つ
    a += rect(660, 0, 452, 330, PAPER, 12, LINE)
    a += text(684, 36, '条件は、範囲の中で成り立つ', 18, INK, 700)
    a += path('M684 56 H1088', LINE, 1)
    a += text(684, 90, '範囲の中', 14, DIM, 700)
    a += text(684, 120, '承認の経路', 17, INK, 700) + check(904, 114, INK, .7)
    a += text(924, 120, '誰が：経理と開発', 16, INK)
    a += path('M684 146 H1088', LINE, 1)
    a += text(684, 180, '範囲を広げると', 14, DIM, 700)
    a += text(684, 210, '別案件の要員の手配', 17, INK, 700) + cross(904, 204, INK, .6)
    a += text(924, 210, '誰が：書けない', 16, INK)
    a += text(684, 238, 'この定例の議事録には、誰が決めるのかが書かれていない', 14, DIM)
    a += path('M684 262 H1088', LINE, 1)
    a += text(684, 300, '範囲を決めてから、条件を書く', 18, INK, 700)
    return svg('意味と範囲の下に、入れるもの・落とすもの・並び順を3文で追記する。条件は範囲の中でしか成り立たないので、範囲を決めてから書く', a, 334)


def s4_after():
    """L4-S5　条件を追記する前と後を、確かめる4つ × 回の表で比べる。主役は、追記したあとの3列がそろって埋まること。"""
    a = rect(0, 0, W, 340, PAPER, 12, LINE)
    a += rect(668, 14, 312, 262, PANEL, 10)
    a += text(480, 40, '追記する前', 16, DIM, 700, 'middle')
    a += text(824, 40, '追記したあと　＋条件', 16, ACCENT, 700, 'middle')
    before, after = [380, 480, 580], [724, 824, 924]
    for cx in before + after:
        a += text(cx, 68, f'{(before + after).index(cx) % 3 + 1}回目', 14, DIM, 700, 'middle')
    a += path('M24 82 H1088', LINE, 1)
    rows = [('誰が決めるかが入る', [0, 1, 1]),
            ('いつまでかが入る', [0, 0, 1]),
            ('決まったものが入っていない', [1, 1, 0]),
            ('期限の早い順に並ぶ', [0, 0, 0])]
    for k, (name, ok) in enumerate(rows):
        y = 118 + k * 44
        a += text(24, y + 6, name, 17, INK, 700)
        for cx, v in zip(before, ok):
            a += check(cx, y, INK, .7) if v else cross(cx, y, INK, .55)
        for cx in after:
            a += check(cx, y, ACCENT, .8)
        if k < 3:
            a += path(f'M24 {y + 22} H1088', LINE, 1)
    a += path('M24 294 H1088', LINE, 1)
    a += text(24, 324, '件数は、追記したあと 5件 ・ 6件 ・ 5件。6件の回は、1件を2行に分けているだけで、並ぶ課題は同じ', 15, DIM)
    return svg('条件を追記する前は、担当 ・ 期限 ・ 決まったものの除外 ・ 期限の順が回ごとに欠けていた。追記したあとは、3回とも欠けが無い', a, 344)
