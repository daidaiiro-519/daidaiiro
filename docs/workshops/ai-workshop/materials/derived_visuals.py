"""現行の図から派生させた図。実行時の書き換えをやめ、実体として置く。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, card, band)


def t1_fit():
    """材料を替えても、返ってくるものが目的の中に収まる。"""
    a = text(0, 24, '渡すものを替える', 18, DIM)
    for i, name in enumerate(['今週の定例', '来週の定例', '別の案件']):
        y = 44 + i * 74
        a += rect(0, y, 210, 58, PAPER, 10, LINE)
        a += icon(22, y + 14, 'doc', DIM, .55)
        a += text(66, y + 36, name, 19, INK, 700)
        a += path(f'M210 {y + 29} H260 V142 H300', LINE)
    a += path('M291 136 l9 6 -9 6', LINE, 2)
    a += rect(300, 108, 214, 68, PANEL, 10)
    a += text(407, 148, '同じ依頼', 21, INK, 700, 'middle')
    a += arrow(524, 142, 606, 142)
    a += circle(730, 142, 112, PAPER, ACCENT)
    a += text(730, 14, '目的　次の打ち合わせで決めることを並べる', 17, ACCENT, 700, 'middle')
    for cx, cy, label in [(686, 120, '6件'), (742, 172, '4件'), (778, 112, '9件')]:
        a += circle(cx, cy, 30, ACCENT, ACCENT)
        a += text(cx, cy + 7, label, 18, PAPER, 700, 'middle')
    a += text(730, 288, '中身も件数も違う。どれも目的の中にある', 17, DIM, anchor='middle')
    a += text(876, 116, '毎回おなじ文章が', 17, DIM)
    a += text(876, 142, '返るわけではない', 17, DIM)
    return svg('渡すものを替えても、返ってくるものが目的の中に収まる', a, 300)


def t1_height():
    """同じ議事録の依頼でも、書き方によって当てはまる場面の数が違う。"""
    a = rect(0, 44, 296, 76, PANEL, 10)
    a += text(24, 74, '広い書き方', 17, DIM)
    a += text(24, 104, '「課題を整理して」', 21, INK, 700)
    a += arrow(306, 82, 342, 82)
    for i, name in enumerate(['業務の流れの順', '決定の期限の順', '機能ごと', '3つを混ぜたもの']):
        x = 354 + i * 155
        a += rect(x, 52, 143, 60, PAPER, 10, LINE)
        a += text(x + 71, 88, name, 15, INK, anchor='middle')
    a += text(0, 148, '当てはまる場面　4つとも', 17, ACCENT, 700)
    a += text(240, 148, 'どれが返ってくるかは決まらない', 17, DIM)

    a += path('M0 176 H1112', LINE, 1)

    a += rect(0, 200, 296, 76, PANEL, 10)
    a += text(24, 228, '狭い書き方', 17, DIM)
    a += text(24, 252, ['「1行目に通知の方式、', '　2行目に締めの日」'], 16, INK, 700, gap=24)
    a += arrow(306, 238, 342, 238)
    a += rect(354, 208, 143, 60, PAPER, 10, LINE)
    a += text(425, 244, '指定どおりの1つ', 15, INK, anchor='middle')
    a += text(0, 304, '当てはまる場面　1つだけ', 17, ACCENT, 700)
    a += text(240, 304, '来週の議事録では、この指定が合わない', 17, DIM)
    return svg('広い書き方は4つとも当てはまり、狭い書き方は1つだけに当てはまる', a, 322)


def t1_order():
    """5本の順序と、各回で決めること。"""
    a = text(0, 24, 'この順で決める', 18, DIM)
    steps = [('次', '意味を決める', '何を並べるか', PAPER),
             ('そのあと', '範囲を決める', 'どこから拾うか', PAPER),
             ('そのあと', '条件を決める', '必ず入れるもの', PAPER),
             ('そのあと', '揺らぎを直す', '収まらないとき', PANEL),
             ('最後', '高さを合わせる', '3つを書いた意味', ACCENT)]
    W5, GAP = 196, 33
    for i, (no, name, sub, fill) in enumerate(steps):
        x = i * (W5 + GAP)
        ink = PAPER if fill is ACCENT else INK
        dim = PAPER if fill is ACCENT else DIM
        a += rect(x, 46, W5, 96, fill, 12, ACCENT if fill is ACCENT else (LINE if fill is PAPER else 'none'))
        a += text(x + W5 / 2, 84, name, 19, ink, 700, 'middle')
        a += text(x + W5 / 2, 114, sub, 16, dim, anchor='middle')
        a += text(x + W5 / 2, 166, no, 16, DIM, anchor='middle')
        if i < 4:
            a += arrow(x + W5 + 6, 94, x + W5 + GAP - 6, 94)
    a += band(206, '決める → 3回試す → ずれた1つを直す。この繰り返しで、高さを近づける')
    return svg('このあと5本で決めることと、その順序', a, 274)


def t1_symptoms():
    """2つの困り方を、番号を付けて左右に分ける。"""
    def head(x, no, name):
        a = rect(x, 0, 32, 26, ACCENT, 6, ACCENT)
        a += text(x + 16, 19, no, 15, PAPER, 700, 'middle')
        a += text(x + 44, 19, name, 19, INK, 700)
        return a

    a = head(0, '1', '毎回、違うものが返ってくる')
    a += text(0, 58, '同じものを渡して、同じ依頼を3回', 16, DIM)
    a += card(0, 74, 208, 76, '「課題を整理して」')
    for i, (no, name) in enumerate([('1回目', '業務の流れの順に並ぶ'),
                                    ('2回目', '決定の期限の順に並ぶ'),
                                    ('3回目', '機能ごとに並ぶ')]):
        y = 70 + i * 66
        a += path(f'M208 112 H236 V{y + 25} H262', LINE)
        a += path(f'M{262 - 9} {y + 25 - 6} l9 6 -9 6', LINE, 2)
        a += rect(262, y, 300, 50, PAPER, 10, LINE)
        a += text(282, y + 31, no, 15, DIM)
        a += text(336, y + 31, name, 18, INK, 700)
    a += text(0, 294, '並べ方が、毎回違う', 17, ACCENT, 700)

    a += path('M590 0 V308', LINE, 1)

    a += head(616, '2', 'その1回にしか使えない')
    a += text(616, 58, '行ごとに中身を指定した依頼', 16, DIM)
    a += rect(616, 74, 228, 118, PAPER, 10, LINE)
    a += text(638, 106, ['1行目に承認の経路', '2行目に申請の締め日', '3行目に停止日'], 18, INK, gap=32)
    a += arrow(854, 133, 888, 133)
    a += rect(898, 98, 214, 70, PAPER, 10, LINE)
    a += check(926, 133)
    a += text(950, 140, 'そのとおり返る', 18, INK, 700)
    a += text(616, 232, '次の週の議事録に、同じ依頼を出す', 16, DIM)
    a += rect(616, 248, 228, 60, PAPER, 10, LINE)
    a += text(730, 285, '承認の話が出ていない', 17, INK, anchor='middle')
    a += arrow(854, 278, 888, 278)
    a += rect(898, 248, 214, 60, PAPER, 10, LINE)
    a += cross(926, 278)
    a += text(950, 285, 'その指定が合わない', 17, INK, 700)
    return svg('2つの困り方 ── 毎回違うものが返ることと、その1回にしか使えないこと', a, 320)


def t3_after():
    """範囲を足す前と、足したあとを並べる。"""
    a = text(0, 22, '足す前　意味だけを書いた依頼 ── 読む範囲が3通り', 17, DIM)
    for i, (no, n, where) in enumerate([('1回目', '4件', '今回の定例だけ'),
                                        ('2回目', '11件', '合同会議まで'),
                                        ('3回目', '18件', '前の案件まで')]):
        x = i * 384
        a += rect(x, 38, 344, 48, PAPER, 10, LINE)
        a += text(x + 24, 68, no, 14, DIM)
        a += text(x + 80, 68, where, 17, DIM)
        a += text(x + 320, 69, n, 18, DIM, 700, 'end')
    a += text(0, 122, '足したあと　範囲の1文を足した ── 読む範囲は3回とも同じ', 17, ACCENT, 700)
    for i, (no, n) in enumerate([('1回目', '4件'), ('2回目', '5件'), ('3回目', '4件')]):
        x = i * 384
        a += rect(x, 138, 344, 152, PAPER, 12, ACCENT)
        a += text(x + 24, 172, no, 15, DIM)
        a += text(x + 320, 174, n, 22, ACCENT, 700, 'end')
        a += path(f'M{x + 24} 188 H{x + 320}', LINE, 1)
        a += text(x + 24, 216, ['今回の案件の定例、直近5回', '合同会議　0件', '前の案件　0件'], 17, INK, gap=28)
    a += band(310, '18件になる回は、もう起きない')
    return svg('範囲を足す前は読む範囲が3通り、足したあとは3回とも同じ', a, 378)


def t3_draw_line():
    """範囲の線を引くと、外側は拾われない。"""
    a = text(0, 24, '見る範囲', 18, ACCENT, 700)
    a += rect(0, 40, 520, 150, PANEL, 12)
    for i, name in enumerate(['今回の案件の定例　第5回', '今回の案件の定例　第4回', '同じ定例の　第3回・第2回・第1回']):
        a += rect(24, 56 + i * 44, 472, 36, PAPER, 8, LINE)
        a += text(48, 80 + i * 44, name, 16, INK, 700)
    a += text(0, 216, '含めないもの', 18, DIM)
    for i, (name, why) in enumerate([('合同会議', '他部署の業務の目的で整理してある'),
                                     ('前の案件の議事録', 'もう終わった案件の話')]):
        y = 232 + i * 54
        a += rect(0, y, 520, 44, PAPER, 8, LINE)
        a += cross(28, y + 22, DIM, .7)
        a += text(54, y + 28, name, 17, DIM, 700)
        a += text(256, y + 28, why, 15, DIM)
    a += arrow(536, 132, 580, 132, ACCENT)
    a += rect(596, 40, 516, 246, PAPER, 12, ACCENT)
    a += text(624, 80, '依頼に足す1文', 16, DIM)
    a += text(624, 118, ['見るのは、今回の案件の定例、', '直近5回だけです。合同会議と、', '前の案件の議事録は含めません。'], 19, INK, gap=30)
    a += path('M624 222 H1084', LINE, 1)
    a += text(624, 256, '範囲の外には、別の業務の整理が入る', 17, ACCENT, 700)
    return svg('範囲の線を引くと、その外側からは拾われない', a, 330)


def t3_folder():
    """フォルダごと渡すと、拾う量が毎回違う。"""
    a = text(0, 24, '渡したもの', 18, DIM)
    a += rect(0, 40, 300, 192, PAPER, 12, LINE)
    for i, (name, n) in enumerate([('今回の案件の定例', '5回分'), ('他部署との合同会議', '2回分'),
                                   ('前の案件の議事録', '3回分')]):
        y = 76 + i * 56
        a += icon(24, y - 18, 'doc', DIM, .5)
        a += text(66, y + 6, name, 18, INK, 700)
        a += text(276, y + 6, n, 16, DIM, anchor='end')
    a += arrow(310, 136, 352, 136)
    a += rect(362, 102, 226, 68, PANEL, 10)
    a += text(475, 134, '同じ依頼', 20, INK, 700, 'middle')
    a += text(475, 156, '意味は書いてある', 15, DIM, anchor='middle')
    for i, (no, n, w) in enumerate([('1回目', '4件', 68), ('2回目', '11件', 186), ('3回目', '18件', 304)]):
        y = 44 + i * 74
        a += path(f'M588 136 H616 V{y + 26} H648', LINE)
        a += path(f'M639 {y + 20} l9 6 -9 6', LINE, 2)
        a += text(648, y + 32, no, 16, DIM)
        a += rect(706, y + 10, w, 34, ACCENT if i == 2 else PANEL, 8)
        a += text(706 + w + 14, y + 33, n, 20, INK, 700)
    a += text(648, 266, '同じ依頼でも、拾ってくる量が4倍以上違う', 17, DIM)
    return svg('フォルダごと渡すと、拾ってくる量が毎回違う', a, 282)


def t4_after():
    """件数も並びも違うが、どの項目にも同じ列が在る。"""
    a = ''
    for i, (no, n, rows) in enumerate([
            ('1回目', '3件', [('通知の方式', '業務部門', '9/30'), ('締めを流す日', '開発', '9/30'),
                            ('兼務の権限', 'PM', '10/7')]),
            ('2回目', '5件', [('通知の宛先', '業務部門', '9/30'), ('締めの日', '開発', '9/30'),
                            ('ほか3件', '', '')]),
            ('3回目', '4件', [('通知の方式', '業務部門', '10/2'), ('権限の単位', 'PM', '10/7'),
                            ('ほか2件', '', '')])]):
        x = i * 384
        a += rect(x, 16, 344, 196, PAPER, 12, LINE)
        a += text(x + 24, 52, no, 17, DIM)
        a += text(x + 320, 54, n, 22, ACCENT, 700, 'end')
        a += text(x + 24, 84, '課題', 14, DIM)
        a += text(x + 190, 84, '誰が', 14, ACCENT)
        a += text(x + 262, 84, 'いつまで', 14, ACCENT)
        a += path(f'M{x + 24} 94 H{x + 320}', LINE, 1)
        for r, (name, who, when) in enumerate(rows):
            y = 122 + r * 30
            a += text(x + 24, y, name, 16, INK)
            a += text(x + 190, y, who, 15, DIM)
            a += text(x + 262, y, when, 15, DIM)
    a += band(234, '件数も書き方も違う。それでも、同じ列が在り、期限の順に並ぶ')
    return svg('件数も並びも違うが、どの項目にも誰が・いつまでが在る', a, 302)


def t4_layers():
    """意味と範囲は一覧に効き、条件は1件ずつに効く。"""
    a = text(0, 24, '決まるものが違う', 18, DIM)
    a += rect(0, 44, 250, 68, PAPER, 10, LINE)
    a += text(125, 76, '意味', 20, INK, 700, 'middle')
    a += text(125, 98, '並べるものの種類', 15, DIM, anchor='middle')
    a += rect(0, 124, 250, 68, PAPER, 10, LINE)
    a += text(125, 156, '範囲', 20, INK, 700, 'middle')
    a += text(125, 178, '拾ってくる場所', 15, DIM, anchor='middle')
    a += path('M250 78 H300 V118 M250 158 H300 V118 H336', LINE)
    a += path('M327 112 l9 6 -9 6', LINE, 2)
    a += rect(336, 76, 300, 84, PANEL, 10)
    a += text(486, 112, '返ってくる一覧', 20, INK, 700, 'middle')
    a += text(486, 138, 'どういう一覧になるか', 15, DIM, anchor='middle')
    a += arrow(646, 118, 686, 118)
    for i in range(3):
        y = 44 + i * 58
        a += rect(696, y, 416, 44, PAPER, 8, LINE)
        a += text(716, y + 28, ['承認の経路', '申請の締め日', '旧システムの停止日'][i], 17, INK)
        a += text(900, y + 28, '誰が決めるか', 15, ACCENT)
        a += text(1010, y + 28, 'いつまでに', 15, ACCENT)
    a += rect(696, 232, 416, 56, ACCENT, 10, ACCENT)
    a += text(904, 258, '条件', 19, PAPER, 700, 'middle')
    a += text(904, 280, '必ず入れるものと、並べる順', 15, PAPER, anchor='middle')
    a += path('M904 232 V214 M898 220 l6 -6 6 6', ACCENT, 2)
    a += text(0, 250, '書いてあっても、書き方まで決めていないと揃わない', 16, DIM)
    return svg('意味と範囲は一覧に効き、条件は1件ずつに効く', a, 300)


def t4_missing():
    """項目は合っているのに、1件ずつの中身が揃わない。"""
    head = ['項目', '誰が決めるか', 'いつまでに', '並び順']
    rows = [('1回目', ['合っている', 'ない', 'ない', 'ばらばら']),
            ('2回目', ['合っている', 'ある', 'ない', 'ばらばら']),
            ('3回目', ['合っている', 'ある', 'ある', 'ばらばら'])]
    a = ''
    xs = [140, 400, 660, 880]
    for x, h in zip(xs, head):
        a += text(x, 38, h, 16, DIM)
    a += path('M0 52 H1112', LINE, 1)
    for r, (no, cells) in enumerate(rows):
        y = 68 + r * 62
        a += text(0, y + 30, no, 18, INK, 700)
        for x, c in zip(xs, cells):
            ng = c in ('ない', 'ばらばら')
            a += rect(x - 16, y, 212, 44, ACCENT if ng else PAPER, 8, ACCENT if ng else LINE)
            a += text(x + 90, y + 29, c, 17, PAPER if ng else INK, 700 if ng else 400, 'middle')
    a += band(256, 'どの回も依頼に違反していない。それでも、期待したものとは違う')
    return svg('項目は合っているのに、1件ずつの中身が回ごとに欠ける', a, 324)


def t4_unwritten():
    """頭の中に在った約束が、依頼には書かれていない。"""
    a = text(0, 24, '頭の中にあった約束', 18, DIM)
    a += rect(0, 40, 520, 214, PAPER, 12, LINE)
    for i, (what, why) in enumerate([('誰が決めるか', 'その場に呼ぶ人が決まらない'),
                                     ('いつまでに決めるか', '今回扱うかが決まらない'),
                                     ('決定の期限の順に並べる', 'どれを先に扱うかが分からない')]):
        y = 74 + i * 62
        a += text(28, y, what, 19, INK, 700)
        a += text(28, y + 26, 'ないと　' + why, 16, DIM)
    a += text(596, 24, '依頼に書いてあったこと', 18, DIM)
    a += rect(596, 40, 516, 214, PANEL, 12)
    a += text(624, 82, '意味', 18, INK, 700)
    a += text(700, 82, '決定しないと案件の進行が止まるもの', 17, DIM)
    a += text(624, 122, '範囲', 18, INK, 700)
    a += text(700, 122, '今回の案件の定例、直近5回', 17, DIM)
    a += text(624, 170, '条件', 18, DIM, 700)
    a += cross(714, 164, ACCENT, .7)
    a += text(740, 170, '書いていない', 17, ACCENT, 700)
    a += text(624, 216, '書いていない約束は、守られる回と', 16, DIM)
    a += text(624, 240, '守られない回が出る', 16, DIM)
    a += band(280, 'AIが手を抜いたのではない。書いていないことは、頼んでいないのと同じである')
    return svg('必ず入れてほしい3つが、頭の中には在り、依頼には書かれていない', a, 348)


def t5_axis():
    """高さの3つの位置を、縦に並べて比べる。"""
    AX = 52
    a = path(f'M{AX} 330 V22 M{AX - 7} 32 l7 -10 7 10', LINE, 2)
    a += text(0, 18, '高い', 16, DIM, 700)
    a += text(0, 348, '低い', 16, DIM, 700)
    rows = [('高すぎる', '広い書き方', '「課題を整理して」', '何が返るか決まらない', False),
            ('いまの依頼', '目的に合う高さ', '意味・範囲・条件を書いた', '渡すものを替えても目的の中に収まる', True),
            ('低すぎる', '狭い書き方', '1行目に通知の方式、2行目に締めの日', 'その1回にしか使えない', False)]
    for i, (name, kind, ex, note, hot) in enumerate(rows):
        y = 22 + i * 108
        a += circle(AX, y + 42, 13, ACCENT if hot else PAPER, ACCENT if hot else LINE)
        a += rect(96, y, 1016, 84, PAPER if hot else PANEL, 12, ACCENT if hot else 'none')
        a += text(124, y + 36, name, 20, ACCENT if hot else INK, 700)
        a += text(124, y + 64, kind, 15, DIM)
        a += text(330, y + 36, ex, 19, INK)
        a += text(330, y + 64, note, 16, DIM)
    return svg('高すぎる側と低すぎる側の間に、いまの依頼がある', a, 356)


def t5_grown():
    """もとの一言に、3文だけを足した。"""
    a = text(0, 24, 'もとの依頼', 18, DIM)
    a += rect(0, 40, 300, 64, PAPER, 10, LINE)
    a += text(150, 80, '「課題を整理して」', 20, INK, 700, 'middle')
    a += text(0, 132, '足した3文', 18, ACCENT, 700)
    for i, (name, body) in enumerate([('意味', '決定しないと案件の進行が止まるもの（精算業務と開発の課題は含めない）'),
                                      ('範囲', '今回の案件の定例、直近5回だけを見る'),
                                      ('条件', '誰が決めるかと、いつまでかを入れ、決定の期限の順に並べる')]):
        y = 150 + i * 56
        a += rect(0, y, 86, 44, ACCENT, 8, ACCENT)
        a += text(43, y + 29, name, 18, PAPER, 700, 'middle')
        a += rect(98, y, 574, 44, PAPER, 8, LINE)
        a += text(118, y + 29, body, 16, INK)
    a += rect(712, 40, 400, 278, PANEL, 12)
    a += text(736, 78, '指定していないこと', 18, DIM)
    for i, s in enumerate(['見出しの文言', '返ってくる形', '項目の並び順', '表にするかどうか', '1件あたりの字数']):
        a += cross(752, 112 + i * 40, DIM, .6)
        a += text(778, 118 + i * 40, s, 17, DIM)
    return svg('もとの依頼に、意味・範囲・条件の3文だけを足した', a, 330)


def t5_swap():
    """依頼を固定して、材料のほうを替える。"""
    a = text(0, 24, '依頼は固定', 18, DIM)
    a += rect(0, 100, 268, 84, ACCENT, 12, ACCENT)
    a += text(134, 136, 'いまの依頼', 20, PAPER, 700, 'middle')
    a += text(134, 162, '3文を足したもの', 15, PAPER, anchor='middle')
    a += text(310, 24, '替えたのは渡すもの', 18, DIM)
    for i, (mat, n, body) in enumerate([('今週の定例', '6件', '通知の方式ほか'),
                                        ('来週の定例', '4件', '移行の範囲ほか'),
                                        ('別の案件の定例', '9件', '帳票の様式ほか')]):
        y = 40 + i * 84
        a += path(f'M268 142 H296 V{y + 30} H328', LINE, 2 if i == 1 else 2)
        a += path(f'M319 {y + 24} l9 6 -9 6', LINE, 2)
        a += rect(328, y, 246, 60, PAPER, 10, LINE)
        a += icon(348, y + 16, 'doc', DIM, .45)
        a += text(386, y + 36, mat, 18, INK, 700)
        a += arrow(584, y + 30, 620, y + 30)
        a += rect(630, y, 482, 60, PANEL, 10)
        a += text(654, y + 36, n, 20, ACCENT, 700)
        a += text(700, y + 36, '決定の期限の順に並び、誰が決めるか付き', 16, INK)
        a += text(1088, y + 36, '', 15, DIM, anchor='end')
    a += band(300, '中身も件数も違う。返ってくるものの形は、どれも同じである')
    return svg('依頼を固定して渡すものを替えても、返ってくるものの形は同じになる', a, 368)


def t5_three_and_height():
    """3つを決める作業が、高さを合わせる作業だった。"""
    a = text(0, 24, '2本目から4本目まで、やってきたこと', 18, DIM)
    for i, (name, did, effect) in enumerate([
            ('意味', '何を並べるか', '広さの上限が決まる'),
            ('範囲', 'どこから拾うか', '読む範囲が決まる'),
            ('条件', 'どの項目にも入れるものと、並べる順', '1件ずつの中身が決まる')]):
        y = 44 + i * 66
        a += rect(0, y, 120, 52, ACCENT, 10, ACCENT)
        a += text(60, y + 33, name, 19, PAPER, 700, 'middle')
        a += rect(132, y, 340, 52, PAPER, 10, LINE)
        a += text(154, y + 33, did, 17, INK)
        a += arrow(482, y + 26, 518, y + 26)
        a += rect(528, y, 300, 52, PANEL, 10)
        a += text(678, y + 33, effect, 17, INK, anchor='middle')
    a += path('M838 70 H876 M838 136 H876 M838 202 H876', ACCENT)
    a += path('M876 70 V202', ACCENT)
    a += path('M876 136 H902', ACCENT)
    a += path('M893 130 l9 6 -9 6', ACCENT, 2)
    a += rect(902, 108, 210, 76, ACCENT, 12, ACCENT)
    a += text(1007, 146, '高さが合う', 20, PAPER, 700, 'middle')
    a += text(1007, 172, '目的に対して', 15, PAPER, anchor='middle')
    a += band(254, '抽象度を上げる、では何をするかが決まらない。この3つなら書ける')
    return svg('言葉・範囲・条件を決める作業が、高さを合わせる作業だった', a, 322)


def t5_three_and_height2():
    """3つを決める作業が、高さを合わせる作業だった。"""
    a = text(0, 24, 'ここまでの3本で、やってきたこと', 18, DIM)
    for i, (name, did, effect) in enumerate([
            ('意味', '何を並べるか', '広さの上限が決まる'),
            ('範囲', 'どこから拾うか', '読む範囲が決まる'),
            ('条件', 'どの項目にも入れるものと、並べる順', '1件ずつの中身が決まる')]):
        y = 44 + i * 66
        a += rect(0, y, 120, 52, ACCENT, 10, ACCENT)
        a += text(60, y + 33, name, 19, PAPER, 700, 'middle')
        a += rect(132, y, 340, 52, PAPER, 10, LINE)
        a += text(154, y + 33, did, 17, INK)
        a += arrow(482, y + 26, 518, y + 26)
        a += rect(528, y, 300, 52, PANEL, 10)
        a += text(678, y + 33, effect, 17, INK, anchor='middle')
    a += path('M838 70 H876 M838 136 H876 M838 202 H876', ACCENT)
    a += path('M876 70 V202', ACCENT)
    a += path('M876 136 H902', ACCENT)
    a += path('M893 130 l9 6 -9 6', ACCENT, 2)
    a += rect(902, 108, 210, 76, ACCENT, 12, ACCENT)
    a += text(1007, 146, '高さが合う', 20, PAPER, 700, 'middle')
    a += text(1007, 172, '目的に対して', 15, PAPER, anchor='middle')
    a += band(254, '抽象度を上げる、では何をするかが決まらない。この3つなら書ける')
    return svg('言葉・範囲・条件を決める作業が、高さを合わせる作業だった', a, 322)


def t6_one_at_a_time():
    """1文だけ変えて、3回で確かめる。"""
    a = text(0, 24, '直し方', 18, DIM)
    a += rect(0, 44, 268, 92, ACCENT, 12, ACCENT)
    a += text(134, 84, '1文だけ変える', 20, PAPER, 700, 'middle')
    a += text(134, 110, '2つ同時に直さない', 15, PAPER, anchor='middle')
    a += arrow(278, 90, 318, 90)
    a += rect(328, 44, 268, 92, PAPER, 12, LINE)
    a += text(462, 84, '同じものを渡して3回', 20, INK, 700, 'middle')
    a += text(462, 110, '1回では分からない', 15, DIM, anchor='middle')
    a += arrow(606, 90, 646, 90)
    a += rect(656, 44, 456, 92, PANEL, 12)
    a += check(692, 78)
    a += text(716, 85, '揃った　　効いている', 18, INK, 700)
    a += cross(692, 114, DIM, .7)
    a += text(716, 121, '揃わない　その文を戻して、別の場所を見る', 17, DIM)
    a += band(176, 'この教材でずっと3回頼んできたのは、1回の出来ではなく揃い方を見るためである')
    return svg('1文だけ変え、同じものを渡して3回試して揃い方を見る', a, 252)


def t6_symptoms():
    """3つを足しても残る、3つの症状。"""
    a = ''
    items = [('形が違う', '課題の一覧に、', '相談したいことが混じった'),
             ('書き方が揃わない', '期限が「今月中」と', '「9月30日」で混ざった'),
             ('量が違う', '前の週には出てきた項目が、', '今週は出てこない')]
    for i, (name, l1, l2) in enumerate(items):
        x = i * 384
        a += rect(x, 16, 344, 148, PAPER, 12, LINE)
        a += text(x + 24, 60, name, 21, ACCENT, 700)
        a += path(f'M{x + 24} 78 H{x + 320}', LINE, 1)
        a += text(x + 24, 114, [l1, l2], 17, INK, gap=28)
    a += band(186, 'ここで依頼の全体を書き直すと、たいてい前より悪くなる')
    return svg('3つを足しても残る、3つの症状', a, 254)


def t6_two_goals():
    """目的が2つ入った依頼は、逆向きの作業へ引かれる。"""
    a = rect(340, 24, 432, 68, ACCENT, 12, ACCENT)
    a += text(556, 66, '「課題を整理して、次の打ち手も出して」', 20, PAPER, 700, 'middle')
    a += path('M470 92 V126 H286 V158', ACCENT)
    a += path('M280 152 l6 6 6 -6', ACCENT, 2)
    a += path('M642 92 V126 H826 V158', ACCENT)
    a += path('M820 152 l6 6 6 -6', ACCENT, 2)
    for x, name, sub in [(0, '決めることを並べる', '漏れなく並べる'), (572, '案を出す', '絞って深く考える')]:
        a += rect(x, 164, 540, 92, PAPER, 12, LINE)
        a += text(x + 270, 204, name, 21, INK, 700, 'middle')
        a += text(x + 270, 234, sub, 17, DIM, anchor='middle')
    a += band(278, '目的が2つなら、依頼も2つに分ける')
    return svg('目的が2つ入った依頼は、逆向きの作業へ引かれる', a, 346)


def t6_where():
    """症状で、見る場所が決まる。"""
    a = text(0, 24, '症状', 18, DIM)
    a += text(596, 24, '見る場所', 18, ACCENT, 700)
    rows = [('返ってくる形が違う', '意味', '意味が2つ以上ある語が残っていないか'),
            ('拾ってくる量が違う', '範囲', '渡すもののどこまでを見るかが書いてあるか'),
            ('中身が欠ける・揃わない', '条件', '必ず入れるものと、その書き方を決めたか')]
    for i, (sym, where, ask) in enumerate(rows):
        y = 44 + i * 78
        a += rect(0, y, 440, 60, PAPER, 12, LINE)
        a += text(24, y + 38, sym, 19, INK, 700)
        a += arrow(452, y + 30, 580, y + 30, ACCENT)
        a += rect(596, y, 120, 60, ACCENT, 12, ACCENT)
        a += text(656, y + 38, where, 20, PAPER, 700, 'middle')
        a += rect(728, y, 384, 60, PANEL, 12)
        a += text(752, y + 37, ask, 16, INK)
    a += band(288, 'まず症状を見る。直すのは、そのあとである')
    return svg('3つの症状から、見る場所が1つずつ決まる', a, 356)


def _robot(x, y, color):
    """AIを指す図形。検査が座標を読めるよう、絶対座標で描く。"""
    a = path(f'M{x + 24} {y + 12} V{y + 3}', color, 2.3)
    a += circle(x + 24, y + 2, 3, 'none', color)
    a += rect(x + 2, y + 12, 44, 32, 'none', 9, color)
    a += circle(x + 16, y + 27, 3.5, color, color)
    a += circle(x + 32, y + 27, 3.5, color, color)
    a += path(f'M{x + 17} {y + 36} H{x + 31}', color, 2.3)
    a += path(f'M{x + 2} {y + 26} H{x - 6} M{x + 46} {y + 26} H{x + 54}', color, 2.3)
    a += path(f'M{x + 13} {y + 44} V{y + 52} H{x + 35} V{y + 44}', color, 2.3)
    return a


def t_cover(no):
    """表紙の印。前置きと締めは3つの教材の道筋を、本編は6本の中での位置を示す。"""
    if no in (0, 8):
        steps = [('教材1', '課題の把握', 'AIに課題を整理させるとき、何を揃えるか', 'search', 49),
                 ('教材2', '業務の整理', '業務の言葉を、通じる区切りごとに揃える', 'book', 54),
                 ('教材3', '仕組みの構築', '揃えた言葉を、AIに任せる仕組みにする', 'robot', 60)]
        a = ''
        for i, (label, verb, sub, kind, iw) in enumerate(steps):
            x = i * 386
            on = (no == 8 and i == 0)
            a += rect(x, 0, 340, 164, PAPER, 12, ACCENT if on else LINE)
            a += text(x + 20, 28, label, 12, ACCENT if on else DIM, 700)
            if kind == 'robot':
                # ロボットは腕が左右へ出るので、描いた幅の中心を箱の中心へ合わせる
                a += _robot(x + 146, 43, ACCENT)
            else:
                a += icon(x + (340 - iw * 1.2) / 2, 40, kind, ACCENT, 1.2)
            a += text(x + 170, 130, verb, 22, INK, 700, 'middle')
            a += text(x + 170, 154, sub, 12, DIM, anchor='middle')
            if i < 2:
                a += arrow(x + 348, 82, x + 378, 82)
        return svg('3つの教材の道筋', a, 174)
    a = ''
    for i in range(6):
        # 本編は該当する1本を、教材1のはじめには どれも光らせない
        a += rect(i * 100, 0, 88, 10, ACCENT if (i + 2) == no else LINE, 5)
    return svg('6本の中での位置', a, 24)

