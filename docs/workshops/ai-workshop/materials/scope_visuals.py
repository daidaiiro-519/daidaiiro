"""3本目（範囲を決める）の試作の図。議事録の中に混ざっている話を、どこまで対象にするか。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)

MIX = [('今回の案件で、切り替えまでに決めること', '承認の経路／申請の申請の締め日／旧システムの停止日'),
       ('次のフェーズで決めること', '交通費の自動取得／海外出張の通貨の換算'),
       ('定例で出た別案件の相談', '別案件の要員の手配'),
       ('全社の連絡事項', '研修の案内／勤怠の締め')]


def s3_mixed():
    """1本の議事録に、4種類の話が混ざっている。"""
    a = text(0, 22, '渡すのは、今回の案件の定例5週分', 17, DIM)
    a += rect(0, 38, 560, 212, PAPER, 12, LINE)
    for i, (name, ex) in enumerate(MIX):
        y = 62 + i * 48
        a += rect(20, y, 520, 40, PANEL if i == 0 else PAPER, 8, 'none' if i == 0 else LINE)
        a += text(40, y + 26, name, 16, INK if i == 0 else DIM, 700 if i == 0 else 400)
    a += text(0, 274, '4種類の話が、同じ議事録の中に並んでいる', 16, DIM)
    a += arrow(576, 140, 616, 140)
    a += rect(632, 106, 180, 68, PANEL, 10)
    a += text(722, 138, '同じ依頼', 19, INK, 700, 'middle')
    a += text(722, 160, '意味は書いてある', 14, DIM, anchor='middle')
    for i, (no, n, w) in enumerate([('1回目', '4件', 36), ('2回目', '9件', 82), ('3回目', '14件', 128)]):
        y = 44 + i * 68
        a += path(f'M812 140 H836 V{y + 24} H860', LINE)
        a += path(f'M851 {y + 18} l9 6 -9 6', LINE, 2)
        a += text(860, y + 30, no, 15, DIM)
        a += rect(912, y + 8, w, 32, ACCENT if i == 2 else PANEL, 8)
        a += text(912 + w + 12, y + 30, n, 19, INK, 700)
    a += text(860, 258, '拾う量が3倍以上違う', 16, DIM)
    return svg('1本の議事録に4種類の話が混ざり、拾う量が回ごとに変わる', a, 292)


def s3_ranges():
    """回ごとに、どこまでを対象にしたかが違う。"""
    a = text(0, 24, '回ごとに、どこまでを対象にしたか', 18, DIM)
    cols = ['切り替えまで', '次のフェーズ', '別案件', '全社の連絡']
    for i, name in enumerate(cols):
        a += text(300 + i * 196 + 88, 62, name, 15, INK, 700, 'middle')
    for r, (no, upto, n, extra) in enumerate([('1回目', 1, '4件', 'なし'),
                                              ('2回目', 2, '9件', '交通費の自動取得、通貨の換算'),
                                              ('3回目', 4, '14件', '別案件の要員、研修の案内')]):
        y = 82 + r * 66
        a += text(0, y + 32, no, 18, INK, 700)
        a += text(80, y + 32, n, 20, ACCENT if r == 2 else DIM, 700)
        for i in range(4):
            x = 300 + i * 196
            on = i < upto
            a += rect(x, y, 176, 46, PANEL if on else PAPER, 8, 'none' if on else LINE)
            a += text(x + 88, y + 30, '拾った' if on else '拾っていない', 15,
                      INK if on else DIM, 700 if on else 400, 'middle')
        a += text(0, y + 58, '増えた項目　' + extra, 13, DIM)
    a += band(296, '意味は守られている。それでも、量が揃わない')
    return svg('回ごとに、議事録の中のどこまでを対象にしたかが違う', a, 364)


def s3_two_roles():
    """意味と範囲は、決めるものが違う。"""
    a = rect(0, 30, 536, 168, PAPER, 12, LINE)
    a += text(32, 74, '意味', 24, ACCENT, 700)
    a += text(32, 112, '何を並べるか', 21, INK)
    a += text(32, 150, '並べるものの種類が決まる。', 17, DIM)
    a += text(32, 178, '決定しないと進行が止まるものだけを並べる。', 17, DIM)
    a += rect(576, 30, 536, 168, PAPER, 12, LINE)
    a += text(608, 74, '範囲', 24, ACCENT, 700)
    a += text(608, 112, 'どこから拾うか', 21, INK)
    a += text(608, 150, '拾ってくる場所が決まる。', 17, DIM)
    a += text(608, 178, '議事録の中の、どの話までを対象にするか。', 17, DIM)
    a += band(224, '片方を決めても、もう片方は決まらない。書かなければ、AIが決める')
    return svg('意味は並べるものの種類を決め、範囲は拾う話の範囲を決める', a, 292)


def s3_draw_line():
    """対象にする話と、対象にしない話を分ける。"""
    a = text(0, 22, '対象にする', 17, ACCENT, 700)
    a += rect(0, 38, 536, 62, PANEL, 10)
    a += text(24, 66, MIX[0][0], 17, INK, 700)
    a += text(24, 88, '例　' + MIX[0][1], 14, DIM)
    a += text(0, 132, '対象にしない', 17, DIM)
    for i, (name, ex) in enumerate(MIX[1:]):
        y = 148 + i * 58
        a += rect(0, y, 536, 46, PAPER, 8, LINE)
        a += cross(28, y + 22, DIM, .6)
        a += text(54, y + 28, name, 16, DIM, 700)
    a += arrow(552, 160, 592, 160, ACCENT)
    a += rect(608, 38, 504, 244, PAPER, 12, ACCENT)
    a += text(636, 78, '依頼に足す1文', 16, DIM)
    a += text(636, 116, ['見るのは、今回の案件で、本番の切り替え', 'までに決めるものです。次のフェーズの話',
                         'と、定例で出た別案件の話、全社の連絡事', '項は含めません。'], 18, INK, gap=30)
    a += path('M636 248 H1084', LINE, 1)
    a += text(636, 272, '対象にしないものも、並べて書く', 16, ACCENT, 700)
    return svg('議事録の中の、対象にする話と対象にしない話を分ける', a, 320)


def s3_after():
    """範囲を足す前と、足したあとを並べる。"""
    a = text(0, 22, '足す前　意味だけを書いた依頼 ── 対象にした話が3通り', 17, DIM)
    for i, (no, n, where) in enumerate([('1回目', '4件', '切り替えまでの話だけ'),
                                        ('2回目', '9件', '次のフェーズの話も'),
                                        ('3回目', '14件', '別案件と全社の連絡も')]):
        x = i * 384
        a += rect(x, 38, 344, 48, PAPER, 10, LINE)
        a += text(x + 24, 68, no, 14, DIM)
        a += text(x + 80, 68, where, 16, DIM)
        a += text(x + 320, 69, n, 18, DIM, 700, 'end')
    a += text(0, 122, '足したあと　範囲の1文を足した ── 対象にした話は3回とも同じ', 17, ACCENT, 700)
    for i, (no, n) in enumerate([('1回目', '4件'), ('2回目', '5件'), ('3回目', '4件')]):
        x = i * 384
        a += rect(x, 138, 344, 152, PAPER, 12, ACCENT)
        a += text(x + 24, 172, no, 15, DIM)
        a += text(x + 320, 174, n, 22, ACCENT, 700, 'end')
        a += path(f'M{x + 24} 188 H{x + 320}', LINE, 1)
        a += text(x + 24, 216, ['今回の案件、切り替えまで', '次のフェーズ　0件', '別案件・全社の連絡　0件'], 16, INK, gap=28)
    a += band(310, '14件になる回は、もう起きない')
    return svg('範囲を足す前は対象が3通り、足したあとは3回とも同じ', a, 378)
