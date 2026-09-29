"""現行の図から派生させた図。実行時の書き換えをやめ、実体として置く。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, card, band)


def t1_fit():
    """目的に合った高さの指示なら、渡す議事録を替えても、返ってくるものがどれも目的の中に収まる。"""
    names = ['今週の議事録', '来週の議事録', '別の案件の議事録']
    a = text(0, 24, '渡すものを替える', 15, DIM)
    for i, name in enumerate(names):
        y = 40 + i * 74
        a += rect(0, y, 230, 58, PAPER, 10, LINE)
        a += icon(20, y + 14, 'doc', DIM, .55)
        a += text(62, y + 36, name, 16, INK, 700)
        a += path(f'M230 {y + 29} H262 V148 H290', LINE)
    a += path('M281 142 l9 6 -9 6', LINE, 2)
    a += rect(292, 108, 240, 80, PANEL, 10)
    a += text(412, 142, '目的に合った', 16, INK, 700, 'middle')
    a += text(412, 168, '高さの指示', 16, INK, 700, 'middle')
    a += arrow(540, 148, 612, 148)
    # 右：目的の円。3つの議事録から返ってきたものが、どれも円の中に収まる
    cx, cy, r = 760, 148, 128
    a += text(cx, 12, '目的　どの課題から決めるか、優先順位を付ける', 15, ACCENT, 700, 'middle')
    a += circle(cx, cy, r, PAPER, ACCENT)
    # 3つの結果は、幅も位置も少しずつ違う ── 中身も件数も違うが、どれも円の中に収まる
    for i, (name, w, dx) in enumerate(zip(['今週の結果', '来週の結果', '別の案件の結果'], [150, 196, 168], [-26, 8, -14])):
        y = cy - 76 + i * 52
        a += rect(cx - w / 2 + dx, y, w, 38, PAPER, 8, LINE)
        a += text(cx + dx, y + 25, name, 14, INK, 700, 'middle')
    a += text(912, 132, ['中身も件数も違う', 'それでも、どれも', '目的の中に収まる'], 15, INK, gap=24)
    return svg('目的に合った高さの指示なら、渡す議事録を替えても、返ってくるものがどれも目的の中に収まる', a, 290)


def t1_height():
    """2つの指示は、どこまで決めて書くかの両端にある。困り方は、その両端で1つずつ起きる。"""
    L, R, CW = 140, 652, 460          # 左の列 ・ 右の列の左端と、列の幅
    # 上：どこまで決めて書くかの軸。両端に2つの指示が来る
    a = text(L, 18, '決めていない', 14, DIM, 700)
    a += text(R + CW, 18, '細かく決める', 14, DIM, 700, 'end')
    a += text((L + R + CW) / 2, 18, 'どこまで決めて書くか', 14, DIM, anchor='middle')
    a += path(f'M{L + 100} 12 H{(L + R + CW) / 2 - 90} M{(L + R + CW) / 2 + 90} 12 H{R + CW - 110}', LINE, 1.5)
    a += path(f'M{L + 108} 6 l-8 6 8 6 M{R + CW - 118} 6 l8 6 -8 6', LINE, 1.5)
    # 1段目：2つの指示
    for x, who, what in [(L, '決めていない指示', '「課題を整理して」'),
                         (R, '細かく決めた指示', '「1行目に承認の経路、2行目に申請の締め日」')]:
        a += rect(x, 32, CW, 72, PANEL, 10)
        a += text(x + 20, 58, who, 14, DIM)
        a += text(x + 20, 88, what, 17 if len(what) > 12 else 20, INK, 700)
    # 2段目 ・ 3段目：同じ形のマス。困り方のマスだけ、強調色の枠と左上の札を持つ
    cells = [
        (0, L, '頼むたびに変わる', '業務の流れの順 ・ 決定の期限の順 ・ 機能ごと、どれも指示どおり', '困り方1'),
        (0, R, '1つに決まる', '指定どおりのものが返る', None),
        (1, L, 'どの週の議事録にも使える', 'どの案件の議事録にも使える', None),
        (1, R, 'この議事録にしか使えない', '来週、承認の話が出ていなければ使えない', '困り方2'),
    ]
    for row, name in enumerate(['返ってくるもの', '使える議事録']):
        a += text(0, 172 + row * 110, name, 15, DIM, 700)
    for row, x, head, sub, trouble in cells:
        y = 120 + row * 110
        a += rect(x, y, CW, 98, PAPER, 10, ACCENT if trouble else LINE)
        if trouble:
            a += rect(x + 20, y + 14, 64, 24, ACCENT, 6, ACCENT)
            a += text(x + 52, y + 31, trouble, 13, PAPER, 700, 'middle')
        a += text(x + 20, y + 60, head, 18, INK, 700)
        a += text(x + 20, y + 84, sub, 14, INK)
    a += band(350, '2つの困り方は、この両端で起きていた')
    return svg('2つの指示はどこまで決めて書くかの両端にあり、困り方はその両端で1つずつ起きる', a, 414)


def t1_order():
    """5本の順序と、各回で決めること。"""
    a = text(0, 24, 'この順で決める', 18, DIM)
    steps = [('次', '意味を決める', '何を並べるか', PAPER),
             ('そのあと', '範囲を決める', 'どこから拾うか', PAPER),
             ('そのあと', '条件を決める', '入れる・落とす・並べる順', PAPER),
             ('そのあと', '揺らぎを直す', '揃わないとき', PANEL),
             ('最後', '抽象の高さを合わせる', '3つを書いた意味', ACCENT)]
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
    a += band(206, '決める → 3回試す → ずれた1つを直す。この繰り返しで、抽象の高さを近づける')
    return svg('このあと5本で決めることと、その順序', a, 274)


def t1_symptoms():
    """2つの困り方を、左右同じ形で並べる。どちらも、同じ指示を何度か使うと起きる。"""
    def badge(x, y, t):
        return rect(x, y, 64, 24, ACCENT, 6, ACCENT) + text(x + 32, y + 17, t, 13, PAPER, 700, 'middle')

    panels = [
        (0, '困り方1', '毎回、違うものが返ってくる', '「課題を整理して」', '同じ議事録で3回頼む',
         [('1回目', '業務の流れの順に並ぶ', False), ('2回目', '決定の期限の順に並ぶ', False),
          ('3回目', '機能ごとに並ぶ', False)], '並べ方が、毎回違う'),
        (572, '困り方2', 'その1回にしか使えない', '「1行目に承認の経路、2行目に申請の締め日」', '週を替えて頼む',
         [('今週の議事録', 'そのとおりに返る', False), ('来週の議事録', '承認の話が無く、指定が合わない', True)],
         '毎週、指示を書き直すことになる'),
    ]
    a = path('M556 0 V340', LINE, 1)
    for x, tag, title, instr, how, rows, end in panels:
        a += badge(x, 2, tag)
        a += text(x + 76, 20, title, 18, INK, 700)
        a += rect(x, 42, 540, 56, PANEL, 10)
        a += text(x + 20, 76, instr, 16 if len(instr) > 12 else 18, INK, 700)
        a += text(x, 128, how, 14, DIM)
        for i, (label, result, bad) in enumerate(rows):
            y = 142 + i * 50
            a += text(x, y + 26, label, 14, DIM)
            a += rect(x + 110, y, 430, 40, PAPER, 8, LINE)
            if bad:
                a += cross(x + 132, y + 20, DIM, .6)
            a += text(x + (150 if bad else 130), y + 26, result, 15, INK, 700)
        a += path(f'M{x} 300 H{x + 540}', LINE, 1)
        a += text(x, 330, end, 18, INK, 700)
    return svg('2つの困り方 ── 毎回違うものが返ることと、その1回にしか使えないこと', a, 342)


def t3_after():
    """範囲を足す前と、足したあとを並べる。"""
    a = text(0, 22, '足す前　意味だけを書いた指示 ── 読む範囲が3通り', 17, DIM)
    for i, (no, n, where) in enumerate([('1回目', '4件', '今回の定例だけ'),
                                        ('2回目', '11件', '合同会議まで'),
                                        ('3回目', '18件', '前の案件まで')]):
        x = i * 384
        a += rect(x, 38, 344, 48, PAPER, 10, LINE)
        a += text(x + 24, 68, no, 14, DIM)
        a += text(x + 80, 68, where, 17, DIM)
        a += text(x + 320, 69, n, 18, DIM, 700, 'end')
    a += text(0, 122, '書き足したあと　範囲を書き足した ── 読む範囲は3回とも同じ', 17, ACCENT, 700)
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
    a += text(624, 80, '指示に書き足す文', 16, DIM)
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
    a += text(475, 134, '同じ指示', 20, INK, 700, 'middle')
    a += text(475, 156, '意味は書いてある', 15, DIM, anchor='middle')
    for i, (no, n, w) in enumerate([('1回目', '4件', 68), ('2回目', '11件', 186), ('3回目', '18件', 304)]):
        y = 44 + i * 74
        a += path(f'M588 136 H616 V{y + 26} H648', LINE)
        a += path(f'M639 {y + 20} l9 6 -9 6', LINE, 2)
        a += text(648, y + 32, no, 16, DIM)
        a += rect(706, y + 10, w, 34, ACCENT if i == 2 else PANEL, 8)
        a += text(706 + w + 14, y + 33, n, 20, INK, 700)
    a += text(648, 266, '同じ指示でも、拾ってくる量が4倍以上違う', 17, DIM)
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
    return svg('件数は違うが、どの項目にも決める担当と期限が入っている', a, 302)


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
        a += text(900, y + 28, '決める担当', 15, ACCENT)
        a += text(1010, y + 28, 'いつまでに', 15, ACCENT)
    a += rect(696, 232, 416, 56, ACCENT, 10, ACCENT)
    a += text(904, 258, '条件', 19, PAPER, 700, 'middle')
    a += text(904, 280, '入れるもの・落とすもの・並べる順', 15, PAPER, anchor='middle')
    a += path('M904 232 V214 M898 220 l6 -6 6 6', ACCENT, 2)
    a += text(0, 250, '条件には、1件ずつに入れるものと、一覧の全体に効くものがある', 16, DIM)
    return svg('意味と範囲は一覧に効き、条件は1件ずつに効く', a, 300)


def t4_missing():
    """項目は合っているのに、1件ずつの中身が揃わない。"""
    head = ['項目', '決める担当', 'いつまでに', '決まったもの', '並び順']
    rows = [('1回目', ['合っている', 'ない', 'ない', '混じらない', 'ばらばら']),
            ('2回目', ['合っている', 'ある', 'ない', '混じらない', 'ばらばら']),
            ('3回目', ['合っている', 'ある', 'ある', '混じる', 'ばらばら'])]
    a = ''
    xs = [132, 324, 516, 708, 900]
    for x, h in zip(xs, head):
        a += text(x, 38, h, 16, DIM)
    a += path('M0 52 H1112', LINE, 1)
    for r, (no, cells) in enumerate(rows):
        y = 68 + r * 62
        a += text(0, y + 30, no, 18, INK, 700)
        for x, c in zip(xs, cells):
            ng = c in ('ない', 'ばらばら', '混じる')
            a += rect(x - 16, y, 180, 44, ACCENT if ng else PAPER, 8, ACCENT if ng else LINE)
            a += text(x + 74, y + 29, c, 17, PAPER if ng else INK, 700 if ng else 400, 'middle')
    a += band(256, 'どの回も指示どおりではある。それでも、期待したものとは違う')
    return svg('項目は合っているのに、1件ずつの中身が回ごとに欠ける', a, 324)


def t4_unwritten():
    """頭の中にあった期待が、指示には書かれていない。"""
    a = text(0, 24, '頭の中にあった期待', 18, DIM)
    a += rect(0, 40, 520, 224, PAPER, 12, LINE)
    for i, (what, why) in enumerate([('決める担当', '誰に確認するかが決まらない'),
                                     ('いつまでに決めるか', 'どれを先に決めるか比べられない'),
                                     ('決まったものは入れない', '1件ずつ確かめ直すことになる'),
                                     ('決定の期限の順に並べる', '上から優先順位を付けていけない')]):
        y = 70 + i * 50
        a += text(28, y, what, 17, INK, 700)
        a += text(28, y + 22, 'ないと　' + why, 14, DIM)
    a += text(596, 24, '指示に書いてあったこと', 18, DIM)
    a += rect(596, 40, 516, 224, PANEL, 12)
    a += text(624, 82, '意味', 18, INK, 700)
    a += text(700, 82, '決定しないと案件の進行が止まるもの', 17, DIM)
    a += text(624, 122, '範囲', 18, INK, 700)
    a += text(700, 122, '今回の案件で、本番の切り替えまでに決めるもの', 17, DIM)
    a += text(624, 170, '条件', 18, DIM, 700)
    a += cross(714, 164, ACCENT, .7)
    a += text(740, 170, '書いていない', 17, ACCENT, 700)
    a += text(624, 220, '書いていない期待は、満たされる回と', 16, DIM)
    a += text(624, 244, '満たされない回が出る', 16, DIM)
    a += band(288, 'AIの調子が悪いのではない。書いていないことは、頼んでいないのと同じである')
    return svg('必ず入れてほしい4つが、頭の中にはあり、指示には書かれていない', a, 356)


def t5_axis():
    """抽象の高さの3つの位置を、縦に並べて比べる。"""
    AX = 52
    a = path(f'M{AX} 330 V22 M{AX - 7} 32 l7 -10 7 10', LINE, 2)
    a += text(0, 18, '高い', 16, DIM, 700)
    a += text(0, 348, '低い', 16, DIM, 700)
    rows = [('高すぎる', '広い書き方', '「課題を整理して」', '何が返るか決まらない', False),
            ('いまの指示', '目的に合う抽象の高さ', '意味・範囲・条件を書いた', '渡すものを替えても目的の中に収まる', True),
            ('低すぎる', '狭い書き方', '1行目に通知の方式、2行目に締めの日', 'その1回にしか使えない', False)]
    for i, (name, kind, ex, note, hot) in enumerate(rows):
        y = 22 + i * 108
        a += circle(AX, y + 42, 13, ACCENT if hot else PAPER, ACCENT if hot else LINE)
        a += rect(96, y, 1016, 84, PAPER if hot else PANEL, 12, ACCENT if hot else 'none')
        a += text(124, y + 36, name, 20, ACCENT if hot else INK, 700)
        a += text(124, y + 64, kind, 15, DIM)
        a += text(330, y + 36, ex, 19, INK)
        a += text(330, y + 64, note, 16, DIM)
    return svg('高すぎる側と低すぎる側の間に、いまの指示がある', a, 356)


def t5_grown():
    """もとの指示に、3つだけを書き足した。"""
    a = text(0, 24, 'もとの指示', 18, DIM)
    a += rect(0, 40, 300, 64, PAPER, 10, LINE)
    a += text(150, 80, '「課題を整理して」', 20, INK, 700, 'middle')
    a += text(0, 132, '書き足した3つ', 18, ACCENT, 700)
    for i, (name, body) in enumerate([('意味', '決定しないと案件の進行が止まるもの（精算業務と開発の課題は含めない）'),
                                      ('範囲', '今回の案件で、本番の切り替えまでに決めるもの'),
                                      ('条件', '担当と期限を入れ、決まったものは落とし、期限の順に並べる')]):
        y = 150 + i * 56
        a += rect(0, y, 86, 44, ACCENT, 8, ACCENT)
        a += text(43, y + 29, name, 18, PAPER, 700, 'middle')
        a += rect(98, y, 574, 44, PAPER, 8, LINE)
        a += text(118, y + 29, body, 16, INK)
    a += rect(712, 40, 400, 278, PANEL, 12)
    a += text(736, 78, '指定していないこと', 18, DIM)
    for i, s in enumerate(['見出しの文言', '表にするかどうか', '1件あたりの字数', '個別の項目名']):
        a += cross(752, 112 + i * 40, DIM, .6)
        a += text(778, 118 + i * 40, s, 17, DIM)
    return svg('もとの指示に、意味・範囲・条件の3つだけを書き足した', a, 330)


def t5_swap():
    """指示を固定して、材料のほうを替える。"""
    a = text(0, 24, '指示は固定', 18, DIM)
    a += rect(0, 100, 268, 84, ACCENT, 12, ACCENT)
    a += text(134, 136, 'いまの指示', 20, PAPER, 700, 'middle')
    a += text(134, 162, '3つを書き足したもの', 15, PAPER, anchor='middle')
    a += text(310, 24, '替えたのは渡すもの', 18, DIM)
    for i, (mat, n, body) in enumerate([('今週までの5週分', '5件', '承認の経路ほか'),
                                        ('来週までの5週分', '6件', '移行の範囲ほか'),
                                        ('別の案件の5週分', '9件', '連携の方式ほか')]):
        y = 40 + i * 84
        a += path(f'M268 142 H296 V{y + 30} H328', LINE, 2 if i == 1 else 2)
        a += path(f'M319 {y + 24} l9 6 -9 6', LINE, 2)
        a += rect(328, y, 246, 60, PAPER, 10, LINE)
        a += icon(348, y + 16, 'doc', DIM, .45)
        a += text(386, y + 36, mat, 18, INK, 700)
        a += arrow(584, y + 30, 620, y + 30)
        a += rect(630, y, 482, 60, PANEL, 10)
        a += text(654, y + 36, n, 20, ACCENT, 700)
        a += text(700, y + 36, '期限の順に並び、決める担当と期限が付いている', 16, INK)
        a += text(1088, y + 36, '', 15, DIM, anchor='end')
    a += band(300, '中身も件数も違う。返ってくるものの形は、どれも同じである')
    return svg('指示を固定して渡すものを替えても、返ってくるものの形は同じになる', a, 368)


def t5_three_and_height():
    """3つを決める作業が、抽象の高さを合わせる作業だった。"""
    a = text(0, 24, '2本目から4本目まで、やってきたこと', 18, DIM)
    for i, (name, did, effect) in enumerate([
            ('意味', '何を並べるか', '並ぶものの種類が揃う'),
            ('範囲', 'どこから拾うか', '拾ってくる量が揃う'),
            ('条件', '入れるもの・落とすもの・並べる順', '1件ずつの中身が揃う')]):
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
    a += text(1007, 146, '抽象の高さが合う', 20, PAPER, 700, 'middle')
    a += text(1007, 172, '目的に対して', 15, PAPER, anchor='middle')
    a += band(254, '抽象の高さを上げ下げする、では何をするかが決まらない。この3つなら書ける')
    return svg('言葉・範囲・条件を決める作業が、抽象の高さを合わせる作業だった', a, 322)


def t5_three_and_height2():
    """3つを決める作業が、抽象の高さを合わせる作業だった。"""
    a = text(0, 24, 'ここまでの3本の動画で、やってきたこと', 18, DIM)
    for i, (name, did, effect) in enumerate([
            ('意味', '何を並べるか', '並ぶものの種類が揃う'),
            ('範囲', 'どこから拾うか', '拾ってくる量が揃う'),
            ('条件', '入れるもの・落とすもの・並べる順', '1件ずつの中身が揃う')]):
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
    a += text(1007, 146, '抽象の高さが合う', 20, PAPER, 700, 'middle')
    a += text(1007, 172, '目的に対して', 15, PAPER, anchor='middle')
    a += band(254, '抽象の高さを上げ下げする、では何をするかが決まらない。この3つなら書ける')
    return svg('言葉・範囲・条件を決める作業が、抽象の高さを合わせる作業だった', a, 322)


def t6_one_at_a_time():
    """1文だけ変えて、3回で確かめる。"""
    a = text(0, 24, '直し方', 18, DIM)
    a += rect(0, 44, 268, 92, ACCENT, 12, ACCENT)
    a += text(134, 84, '1か所だけ直す', 20, PAPER, 700, 'middle')
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
    a += band(176, '1回返ってきたものだけでは、揺らいでいるかどうかを見分けられない')
    return svg('1文だけ変え、同じものを渡して3回試して揃い方を見る', a, 252)


def t6_symptoms():
    """3つを足しても残る、3つの症状。"""
    a = ''
    # 並びは、次の枚の 意味 ・ 範囲 ・ 条件 の順と1対1にする
    items = [('種類が変わる', '課題管理表に、', '相談したいだけの項目が混じった'),
             ('量が変わる', '前の週には出てきた項目が、', '今週は出てこない'),
             ('書き方が揃わない', '期限が「今月中」と', '「9月30日」で混ざった')]
    for i, (name, l1, l2) in enumerate(items):
        x = i * 384
        a += rect(x, 16, 344, 148, PAPER, 12, LINE)
        a += text(x + 24, 60, name, 21, ACCENT, 700)
        a += path(f'M{x + 24} 78 H{x + 320}', LINE, 1)
        a += text(x + 24, 114, [l1, l2], 17, INK, gap=28)
    a += band(186, '指示の全体を書き直すと、効いていた文まで一緒に消える')
    return svg('3つを足しても残る、3つの症状', a, 254)


def t6_two_goals():
    """目的が2つ入った指示は、逆向きの作業へ引かれる。"""
    a = rect(340, 24, 432, 68, ACCENT, 12, ACCENT)
    a += text(556, 66, '「課題を整理して、打ち手も考えて」', 20, PAPER, 700, 'middle')
    a += path('M470 92 V126 H286 V158', ACCENT)
    a += path('M280 152 l6 6 6 -6', ACCENT, 2)
    a += path('M642 92 V126 H826 V158', ACCENT)
    a += path('M820 152 l6 6 6 -6', ACCENT, 2)
    for x, name, sub in [(0, '課題を整理する', '漏れなく並べる'), (572, '打ち手を考える', '絞り込んで深く考える')]:
        a += rect(x, 164, 540, 92, PAPER, 12, LINE)
        a += text(x + 270, 204, name, 21, INK, 700, 'middle')
        a += text(x + 270, 234, sub, 17, DIM, anchor='middle')
    a += band(278, '目的が2つなら、指示も2つに分ける')
    a += text(0, 370, '見分け方　何のためにするのかを1文で書く。「と」「そして」でつながれば、目的は2つ', 16, DIM)
    return svg('目的が2つ入った指示は、逆向きの作業へ引かれる', a, 390)


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
    """表紙の印。前置き ・ 教材1のはじめに ・ 締めは3つの教材の道筋を、本編は6本の中での位置を示す。"""
    if no in (0, 1, 8):
        steps = [('教材1', '目的の把握', '目的を捉え、指示に書く', 'search', 49),
                 ('教材2', '言葉の定義', '言葉が何を指すかを定める', 'book', 54),
                 ('教材3', '仕組みの構築', '目的から外れない仕組みにする', 'robot', 60)]
        a = ''
        for i, (label, verb, sub, kind, iw) in enumerate(steps):
            x = i * 386
            # 教材1の始まりと締めでは、教材1を際立たせる
            on = (no in (1, 8) and i == 0)
            a += rect(x, 0, 340, 180, PAPER, 12, ACCENT if on else LINE)
            a += text(x + 20, 28, label, 14, ACCENT if on else DIM, 700)
            if kind == 'robot':
                # ロボットは腕が左右へ出るので、描いた幅の中心を箱の中心へ合わせる
                a += _robot(x + 146, 43, ACCENT)
            else:
                a += icon(x + (340 - iw * 1.2) / 2, 40, kind, ACCENT, 1.2)
            a += text(x + 170, 128, verb, 22, INK, 700, 'middle')
            a += text(x + 170, 160, sub, 15, DIM, anchor='middle')
            if i < 2:
                a += arrow(x + 348, 90, x + 378, 90)
        return svg('3つの教材の道筋', a, 190)
    # 本編は、6本の名前を順に並べ、いまの1本だけを塗る。はじめにの「6本の動画で、この順に進みます」と同じ名前と順である
    names = ['原因を知る', '意味を決める', '範囲を決める', '条件を決める', '揺らぎを直す', '抽象の高さを合わせる']
    a = ''
    for i, name in enumerate(names):
        x = i * 188
        on = (i + 2) == no
        a += rect(x, 0, 172, 44, ACCENT if on else PANEL, 8)
        a += text(x + 86, 28, name, 14, PAPER if on else DIM, 700 if on else 400, 'middle')
        if i < 5:
            a += path(f'M{x + 176} 22 H{x + 184}', LINE, 1.5)
    return svg('6本の中での位置', a, 48)

