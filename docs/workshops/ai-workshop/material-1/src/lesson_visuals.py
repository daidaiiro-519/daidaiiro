"""教材スライドの図。企画デッキと同じ部品・同じ配色を使う。"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))
from visuals import (text, rect, path, arrow, circle, person, icon, svg,
                     INK, DIM, LINE, PANEL, PAPER, ACCENT)

W = 1112


def cross(x, y, color=DIM, s=1):
    return path(f'M{x-9*s} {y-9*s} L{x+9*s} {y+9*s} M{x+9*s} {y-9*s} L{x-9*s} {y+9*s}', color, 2.6)


def check(x, y, color=ACCENT, s=1):
    return path(f'M{x-10*s} {y} L{x-3*s} {y+8*s} L{x+11*s} {y-9*s}', color, 3)


def card(x, y, w, h, head, sub=None, *, hot=False, head_size=20, sub_size=16):
    fill = ACCENT if hot else PAPER
    ink = PAPER if hot else INK
    dim = PAPER if hot else DIM
    a = rect(x, y, w, h, fill, 10, ACCENT if hot else LINE)
    a += text(x + w / 2, y + (h / 2 + 7 if sub is None else h / 2 - 4), head, head_size, ink, 700, 'middle')
    if sub:
        a += text(x + w / 2, y + h / 2 + 22, sub, sub_size, dim, anchor='middle')
    return a


def band(y, msg, h=56):
    return rect(0, y, W, h, PANEL, 12) + text(W / 2, y + h / 2 + 8, msg, 21, ACCENT, 700, 'middle')


# ───────────────── 1本目 ─────────────────

def l1_symptoms():
    """同じ指示で返りが3通りになることと、細かく書くと次に使えないこと。"""
    a = text(0, 22, '同じ材料で、同じ指示を3回', 18, DIM)
    a += card(0, 38, 208, 76, '「課題を整理して」')
    for i, (no, name) in enumerate([('1回目', 'リスクの一覧'),
                                    ('2回目', '決まっていないことの一覧'),
                                    ('3回目', '決まったことの一覧')]):
        y = 34 + i * 66
        a += path(f'M208 76 H236 V{y + 25} H262', LINE)
        a += path(f'M{262 - 9} {y + 25 - 6} l9 6 -9 6', LINE, 2)
        a += rect(262, y, 300, 50, PAPER, 10, LINE)
        a += text(282, y + 31, no, 15, DIM)
        a += text(336, y + 31, name, 18, INK, 700)
    a += text(0, 258, '返ってくるものの形が、毎回違う', 18, DIM)

    a += path('M590 10 V272', LINE, 1)

    a += text(616, 22, '見出しも項目も指定した指示', 18, DIM)
    a += rect(616, 38, 228, 118, PAPER, 10, LINE)
    a += text(638, 70, ['1章に体制図', '2章に担当表', '3章に日程'], 18, INK, gap=32)
    a += arrow(854, 97, 888, 97)
    a += rect(898, 62, 214, 70, PAPER, 10, LINE)
    a += check(926, 97)
    a += text(950, 104, 'そのとおり返る', 18, INK, 700)
    a += text(616, 196, '次の週の議事録', 18, DIM)
    a += rect(616, 212, 228, 60, PAPER, 10, LINE)
    a += text(730, 249, '体制図も担当表もない', 17, INK, anchor='middle')
    a += arrow(854, 242, 888, 242)
    a += rect(898, 212, 214, 60, PAPER, 10, LINE)
    a += cross(926, 242)
    a += text(950, 249, 'その指定が合わない', 17, INK, 700)
    return svg('同じ指示では返りの形が毎回違い、細かく指定した指示は次の材料で合わない', a, 282)


def l1_height():
    """同じ議事録の指示でも、書き方によって当てはまる場面の数が違う。"""
    a = rect(0, 44, 296, 76, PANEL, 10)
    a += text(24, 74, '広い書き方', 17, DIM)
    a += text(24, 104, '「課題を整理して」', 21, INK, 700)
    a += arrow(306, 82, 342, 82)
    for i, name in enumerate(['リスクの一覧', '決まっていないこと', '決まったことの一覧', '3つを混ぜた一覧']):
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


def l1_fit():
    """材料を替えても、返ってくるものが目的の中に収まる。"""
    a = text(0, 24, '材料を替える', 18, DIM)
    for i, name in enumerate(['今週の定例', '来週の定例', '別の案件']):
        y = 44 + i * 74
        a += rect(0, y, 210, 58, PAPER, 10, LINE)
        a += icon(22, y + 14, 'doc', DIM, .55)
        a += text(66, y + 36, name, 19, INK, 700)
        a += path(f'M210 {y + 29} H260 V142 H300', LINE)
    a += path('M291 136 l9 6 -9 6', LINE, 2)
    a += rect(300, 108, 214, 68, PANEL, 10)
    a += text(407, 148, '同じ指示', 21, INK, 700, 'middle')
    a += arrow(524, 142, 606, 142)
    a += circle(730, 142, 112, PAPER, ACCENT)
    a += text(730, 14, '目的　次の打ち合わせで決める材料をつくる', 17, ACCENT, 700, 'middle')
    for cx, cy, label in [(686, 120, '6件'), (742, 172, '4件'), (778, 112, '9件')]:
        a += circle(cx, cy, 30, ACCENT, ACCENT)
        a += text(cx, cy + 7, label, 18, PAPER, 700, 'middle')
    a += text(730, 288, '中身も件数も違う。どれも目的の中にある', 17, DIM, anchor='middle')
    a += text(876, 116, '毎回おなじ文章が', 17, DIM)
    a += text(876, 142, '返るわけではない', 17, DIM)
    return svg('材料を替えても、返ってくるものが目的の中に収まる', a, 300)


def l1_three():
    """決めなかった3つを、相手が埋めている。"""
    a = text(0, 24, '指示に書いて決めること', 18, DIM)
    items = [('意味', '課題が何を指すか', '書かないと、並ぶものの種類が変わる'),
             ('範囲', 'どこまでを見るか', '書かないと、拾ってくる量が変わる'),
             ('条件', '何を入れ、落とし、並べるか', '書かないと、1件ずつが揃わない')]
    for i, (name, q, who) in enumerate(items):
        x = i * 384
        a += rect(x, 44, 344, 150, PAPER, 12, LINE)
        a += text(x + 24, 84, name, 24, ACCENT, 700)
        a += text(x + 24, 118, q, 19, INK)
        a += path(f'M{x + 24} 140 H{x + 320}', LINE, 1)
        a += text(x + 24, 174, who, 16, DIM)
    a += band(216, 'AIの決め方は毎回同じとは限らない。回ごとに変わる ── これを、揺らぐと呼ぶ')
    return svg('言葉・範囲・条件を決めないと、その3つを相手が埋める', a, 284)


def l1_order():
    """5本の順序と、各回で決めること。"""
    a = text(0, 24, 'この順で決める', 18, DIM)
    steps = [('2本目', '意味', '何を並べるか', PAPER), ('3本目', '範囲', 'どこから拾うか', PAPER),
             ('4本目', '条件', '必ず入れるもの', PAPER),
             ('5本目', '抽象の高さを合わせる', '3つを決めた意味', PANEL),
             ('6本目', '揺らぎを直す', '収まらないとき', ACCENT)]
    W5, GAP = 196, 33
    for i, (no, name, sub, fill) in enumerate(steps):
        x = i * (W5 + GAP)
        ink = PAPER if fill is ACCENT else INK
        dim = PAPER if fill is ACCENT else DIM
        a += rect(x, 46, W5, 96, fill, 12, ACCENT if fill is ACCENT else (LINE if fill is PAPER else 'none'))
        a += text(x + W5 / 2, 84, name, 21 if len(name) < 9 else 18, ink, 700, 'middle')
        a += text(x + W5 / 2, 114, sub, 16, dim, anchor='middle')
        a += text(x + W5 / 2, 166, no, 16, DIM, anchor='middle')
        if i < 4:
            a += arrow(x + W5 + 6, 94, x + W5 + GAP - 6, 94)
    a += band(206, '決める → 3回試す → ずれた1つを直す。この繰り返しで、抽象の高さを近づける')
    return svg('2本目から6本目までの順序と、各回で決めること', a, 274)


# ───────────────── 2本目 ─────────────────

def l2_three_returns():
    """1本の議事録と1つの指示から、3通りの形が返る。"""
    a = text(0, 24, '渡した材料は1本だけ', 18, DIM)
    a += rect(0, 44, 276, 154, PAPER, 12, LINE)
    a += icon(24, 62, 'doc', DIM, .7)
    a += text(78, 90, '定例の議事録', 19, INK, 700)
    a += text(24, 124, '出席　業務部門 ・ 開発 ・ PM', 13, ACCENT, 700)
    a += text(24, 152, ['通知の方式／締めの日／権限の区分', '決まったこと2件・リスク1件'], 13, DIM, gap=20)
    a += arrow(286, 118, 320, 118)
    a += rect(330, 76, 198, 86, PANEL, 10)
    a += text(429, 108, '「課題を整理して」', 20, INK, 700, 'middle')
    a += text(429, 136, '指示を書くのはPM', 14, DIM, anchor='middle')
    for i, (no, name, ex) in enumerate([
            ('1回目', 'リスクの一覧', '繁忙期が重なる'),
            ('2回目', '決まっていないことの一覧', '通知の方式'),
            ('3回目', '決まったことの一覧', '試作を次の工程へ')]):
        y = 34 + i * 84
        a += path(f'M528 119 H560 V{y + 32} H592', LINE)
        a += path(f'M583 {y + 26} l9 6 -9 6', LINE, 2)
        a += rect(592, y, 520, 64, PAPER, 10, LINE)
        a += text(614, y + 40, no, 16, DIM)
        a += text(668, y + 32, name, 19, INK, 700)
        a += text(668, y + 54, '例　' + ex, 15, DIM)
    return svg('同じ議事録と同じ指示から、3通りの違う形の一覧が返る', a, 292)


def l2_sorting():
    """議事録の中の3種類を、打ち合わせでの扱いで分ける。"""
    a = text(0, 24, '同じ定例の議事録に、3種類が並んでいる ── 発言したのは、別々の人である', 18, DIM)
    kinds = [('リスク', '打ち合わせでは監視する', '業務部門', '来月、繁忙期と重なって確認が取れない', '1回目が拾った'),
             ('未決事項', '打ち合わせで決定する', '業務部門と開発', '通知をメールにするか、画面だけにするか', '2回目が拾った'),
             ('決定事項', '担当が実行する', 'PM', '画面の試作を次の工程へ進める', '3回目が拾った')]
    for i, (name, how, said, ex, who) in enumerate(kinds):
        x = i * 384
        a += rect(x, 44, 344, 190, PAPER, 12, LINE)
        a += text(x + 24, 82, name, 21, INK, 700)
        a += text(x + 24, 110, how, 16, ACCENT, 700)
        a += path(f'M{x + 24} 128 H{x + 320}', LINE, 1)
        a += person(x + 40, 158, .6, DIM)
        a += text(x + 70, 162, said, 15, INK, 700)
        a += text(x + 24, 190, ex, 14, DIM)
        a += rect(x + 24, 200, 148, 26, PANEL, 8)
        a += text(x + 98, 218, who, 13, DIM, anchor='middle')
    a += band(256, '3種類とも、この議事録に記載がある。どれを拾っても指示に違反しない')
    return svg('議事録に並ぶ3種類を、打ち合わせでの扱いと発言者で分ける', a, 324)


def l2_ambiguous():
    """1つの語が、扱いの異なる3種類を指している。"""
    a = rect(400, 30, 312, 72, ACCENT, 12, ACCENT)
    a += text(556, 76, '「課題」', 26, PAPER, 700, 'middle')
    a += text(0, 76, ['職場では、3種類とも', '「課題」と呼称する'], 17, DIM, gap=26)
    for i, (name, how) in enumerate([('リスク', '監視する'), ('未決事項', '決定する'), ('決定事項', '実行する')]):
        x = i * 384
        a += path(f'M556 102 C556 140 {x + 172} 130 {x + 172} 162', ACCENT)
        a += path(f'M{x + 172 - 6} 156 l6 8 6 -8', ACCENT, 2)
        a += rect(x, 168, 344, 86, PAPER, 12, LINE)
        a += text(x + 172, 204, name, 21, INK, 700, 'middle')
        a += text(x + 172, 232, f'打ち合わせでの扱い　{how}', 16, DIM, anchor='middle')
    a += text(556, 286, 'どれを拾うかを記載していない。だからAIが選定する', 19, INK, 700, 'middle')
    return svg('1つの語が、扱いの異なる3種類を指している', a, 300)


def l2_pick():
    """1つの意味を選び、残りを除く1文を書く。"""
    a = text(0, 24, '目的は、次の打ち合わせで決める材料をつくること', 18, DIM)
    picks = [('リスク', False), ('未決事項', True), ('決定事項', False)]
    for i, (name, on) in enumerate(picks):
        x = i * 200
        a += rect(x, 44, 184, 64, ACCENT if on else PAPER, 10, ACCENT if on else LINE)
        a += text(x + 92, 84, name, 20, PAPER if on else DIM, 700, 'middle')
        if not on:
            a += cross(x + 154, 76, DIM, .7)
    a += rect(632, 30, 480, 168, PANEL, 12)
    a += text(656, 62, '指示に書き足す文', 16, DIM)
    a += text(656, 104, ['課題とは、次の打ち合わせで決める', '必要があるものです。すでに決まった', 'ものや、起きるかどうか分からない', 'ものは含めません。'], 17, INK, gap=26)
    a += band(220, '目的の言葉で書く ── 含めないものも並べて、どちらとも取れるものをAIに任せない')
    return svg('3つの意味のうち1つを選び、目的の言葉で書き直す', a, 288)


def l2_after():
    """件数は違うが、並んでいるものの種類は同じ。"""
    a = ''
    for i, (no, n, items) in enumerate([
            ('1回目', '4件', ['通知の方式', '締めを流す日', '兼務の権限', '試験環境の開始']),
            ('2回目', '6件', ['通知の方式', '通知の宛先', '締めを流す日', 'ほか3件']),
            ('3回目', '5件', ['通知の方式と宛先', '締めを流す日', '兼務の権限', 'ほか2件'])]):
        x = i * 384
        a += rect(x, 44, 344, 176, PAPER, 12, LINE)
        a += text(x + 24, 80, no, 17, DIM)
        a += text(x + 320, 82, n, 22, ACCENT, 700, 'end')
        a += path(f'M{x + 24} 96 H{x + 320}', LINE, 1)
        a += text(x + 24, 124, items, 17, INK, gap=28)
    a += band(240, '件数は毎回違う。どの回も、決める必要があるものだけが並ぶ')
    return svg('件数は毎回違うが、並んでいるものの種類は同じになる', a, 308)


# ───────────────── 3本目 ─────────────────

def l3_folder():
    """フォルダごと渡すと、拾う量が毎回違う。"""
    a = text(0, 24, '渡した材料', 18, DIM)
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


def l3_read_range():
    """読んだ範囲が、回ごとに違う。"""
    a = text(0, 24, 'どこまでを読んだか', 18, DIM)
    cols = [('今回の定例', 0), ('合同会議', 1), ('前の案件', 2)]
    for name, i in cols:
        a += text(330 + i * 244 + 110, 66, name, 18, INK, 700, 'middle')
    for r, (no, upto, n, extra) in enumerate([('1回目', 1, '4件', 'なし'),
                                              ('2回目', 2, '11件', '共通基盤の対応範囲、窓口の担当'),
                                              ('3回目', 3, '18件', '帳票の様式、運用の担当部署')]):
        y = 86 + r * 64
        a += text(0, y + 34, no, 18, INK, 700)
        a += text(80, y + 34, n, 20, ACCENT if r == 2 else DIM, 700)
        for name, i in cols:
            x = 330 + i * 244
            on = i < upto
            a += rect(x, y, 220, 48, PANEL if on else PAPER, 8, 'none' if on else LINE)
            a += text(x + 110, y + 31, '読んだ' if on else '読んでいない', 17,
                      INK if on else DIM, 700 if on else 400, 'middle')
        a += text(0, y + 58, '増えた項目　' + extra, 14, DIM)
    a += band(290, '読む範囲が広がった分だけ、拾う量が増えている')
    return svg('読んだ範囲が回ごとに違うので、拾う量が変わる', a, 360)


def l3_two_roles():
    """意味と範囲は、決めるものが違う。"""
    a = rect(0, 30, 536, 168, PAPER, 12, LINE)
    a += text(32, 74, '意味', 24, ACCENT, 700)
    a += text(32, 112, '何を並べるか', 21, INK)
    a += text(32, 150, '並べるものの種類が決まる。', 17, DIM)
    a += text(32, 178, 'リスクではなく、未決のものを並べる。', 17, DIM)
    a += rect(576, 30, 536, 168, PAPER, 12, LINE)
    a += text(608, 74, '範囲', 24, ACCENT, 700)
    a += text(608, 112, 'どこから拾うか', 21, INK)
    a += text(608, 150, '拾ってくる場所が決まる。', 17, DIM)
    a += text(608, 178, 'このフォルダのうち、どこまでを見るか。', 17, DIM)
    a += band(224, '片方を決めても、もう片方は決まらない。書かなければ、AIが決める')
    return svg('意味は並べるものの種類を決め、範囲は拾う場所を決める', a, 292)


def l3_draw_line():
    """範囲の線を引くと、外側は拾われない。"""
    a = text(0, 24, '見る範囲', 18, ACCENT, 700)
    a += rect(0, 40, 520, 150, PANEL, 12)
    for i, name in enumerate(['今回の案件の定例　第5回', '今回の案件の定例　第4回', '同じ定例の　第3回・第2回・第1回']):
        a += rect(24, 56 + i * 44, 472, 36, PAPER, 8, LINE)
        a += text(48, 80 + i * 44, name, 16, INK, 700)
    a += text(0, 216, '含めないもの', 18, DIM)
    for i, (name, why) in enumerate([('合同会議', '他部署にとっての課題を指す'),
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
    a += text(624, 256, '同じ語でも、範囲の外では別の意味になる', 17, ACCENT, 700)
    return svg('範囲の線を引くと、その外側からは拾われない', a, 330)


def l3_after():
    """範囲を決めたあとは、どの回も線の内側から拾う。"""
    a = ''
    for i, (no, n) in enumerate([('1回目', '4件'), ('2回目', '5件'), ('3回目', '4件')]):
        x = i * 384
        a += rect(x, 44, 344, 150, PAPER, 12, LINE)
        a += text(x + 24, 82, no, 17, DIM)
        a += text(x + 320, 84, n, 22, ACCENT, 700, 'end')
        a += path(f'M{x + 24} 98 H{x + 320}', LINE, 1)
        a += text(x + 24, 128, '拾った場所', 15, DIM)
        a += text(x + 24, 156, '定例2本の中だけ', 18, INK, 700)
        a += text(x + 24, 180, '合同会議・前の案件は0件', 15, DIM)
    a += band(216, '18件になる回は、もう起きない。次は、1件ずつの中身を揃える')
    return svg('範囲を決めたあとは、どの回も線の内側だけから拾う', a, 284)


# ───────────────── 4本目 ─────────────────

def l4_missing():
    """項目は合っているのに、1件ずつの中身が揃わない。"""
    head = ['項目', '決める担当', 'いつまでに', '決まったもの']
    rows = [('1回目', ['合っている', 'ない', 'ない', 'なし']),
            ('2回目', ['合っている', 'ある', 'ない', 'なし']),
            ('3回目', ['合っている', 'ある', 'ある', '1件入った'])]
    a = ''
    xs = [140, 400, 660, 880]
    for x, h in zip(xs, head):
        a += text(x, 38, h, 16, DIM)
    a += path('M0 52 H1112', LINE, 1)
    for r, (no, cells) in enumerate(rows):
        y = 68 + r * 62
        a += text(0, y + 30, no, 18, INK, 700)
        for x, c in zip(xs, cells):
            ng = c in ('ない', '1件入った')
            a += rect(x - 16, y, 212, 44, ACCENT if ng else PAPER, 8, ACCENT if ng else LINE)
            a += text(x + 90, y + 29, c, 17, PAPER if ng else INK, 700 if ng else 400, 'middle')
    a += band(256, 'どの回も指示に違反していない。それでも、期待したものとは違う')
    return svg('項目は合っているのに、1件ずつの中身が回ごとに欠ける', a, 324)


def l4_unwritten():
    """頭の中に在った約束が、指示には書かれていない。"""
    a = text(0, 24, '頭の中にあった約束', 18, DIM)
    a += rect(0, 40, 520, 214, PAPER, 12, LINE)
    for i, (what, why) in enumerate([('決める担当', '誰に確認するかが決まらない'),
                                     ('いつまでに決めるか', 'どれを先に決めるか比べられない'),
                                     ('決まったものは入れない', '1件ずつ確認し直すことになる')]):
        y = 74 + i * 62
        a += text(28, y, what, 19, INK, 700)
        a += text(28, y + 26, 'ないと　' + why, 16, DIM)
    a += text(596, 24, '指示に書いてあったこと', 18, DIM)
    a += rect(596, 40, 516, 214, PANEL, 12)
    a += text(624, 82, '意味', 18, INK, 700)
    a += text(700, 82, '次の打ち合わせで決める必要があるもの', 17, DIM)
    a += text(624, 122, '範囲', 18, INK, 700)
    a += text(700, 122, '今回の案件の定例、直近5回', 17, DIM)
    a += text(624, 170, '条件', 18, DIM, 700)
    a += cross(714, 164, ACCENT, .7)
    a += text(740, 170, '書いていない', 17, ACCENT, 700)
    a += text(624, 216, '書いていない約束は、守られる回と', 16, DIM)
    a += text(624, 240, '守られない回が出る', 16, DIM)
    a += band(280, 'AIが手を抜いたのではない。書いていないことは、頼んでいないのと同じである')
    return svg('必ず入れてほしい3つが、頭の中には在り、指示には書かれていない', a, 348)


def l4_layers():
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
        a += text(716, y + 28, ['通知の方式', '締めを流す日', '兼務の権限'][i], 17, INK)
        a += text(900, y + 28, '決める担当', 15, ACCENT)
        a += text(1010, y + 28, 'いつまでに', 15, ACCENT)
    a += rect(696, 218, 416, 56, ACCENT, 10, ACCENT)
    a += text(904, 244, '条件', 19, PAPER, 700, 'middle')
    a += text(904, 266, 'どの項目にも必ず入れるもの', 15, PAPER, anchor='middle')
    a += path('M904 218 V196 M898 202 l6 -6 6 6', ACCENT, 2)
    a += text(0, 250, '書いてあっても、書き方まで決めていないと揃わない', 16, DIM)
    return svg('意味と範囲は一覧に効き、条件は1件ずつに効く', a, 292)


def l4_scope_rule():
    """条件は、決めた範囲の中でだけ成り立つ。"""
    a = text(0, 24, '決めた範囲の中', 18, ACCENT, 700)
    a += rect(0, 40, 536, 176, PANEL, 12)
    a += rect(24, 64, 488, 60, PAPER, 10, LINE)
    a += text(268, 100, '今回の案件の定例', 19, INK, 700, 'middle')
    a += path('M268 124 V148 M262 142 l6 6 6 -6', LINE, 2)
    a += rect(24, 152, 488, 46, PAPER, 10, ACCENT)
    a += check(56, 175)
    a += text(84, 182, '決める担当を、必ず書ける', 18, INK, 700)
    a += text(0, 246, '出席者の中に、決める担当がいる', 16, DIM)

    a += text(576, 24, '範囲を広げると', 18, DIM)
    a += rect(576, 40, 536, 176, PAPER, 12, LINE)
    a += rect(600, 64, 488, 60, PANEL, 10)
    a += text(844, 100, '合同会議まで含める', 19, INK, 700, 'middle')
    a += path('M844 124 V148 M838 142 l6 6 6 -6', LINE, 2)
    a += rect(600, 152, 488, 46, PAPER, 10, LINE)
    a += cross(632, 175, DIM, .8)
    a += text(660, 182, '他部署が決めることは、書けない', 18, DIM, 700)
    a += text(576, 246, '同じ条件が、そのままでは成り立たなくなる', 16, DIM)
    a += band(276, '範囲を変えれば、条件も変わる。だから、範囲を決めてから条件を決める')
    return svg('条件は、決めた範囲の中でだけ成り立つ', a, 344)


def l4_after():
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
        a += text(x + 24, 84, '未決事項', 14, DIM)
        a += text(x + 190, 84, '誰が', 14, ACCENT)
        a += text(x + 262, 84, 'いつまで', 14, ACCENT)
        a += path(f'M{x + 24} 94 H{x + 320}', LINE, 1)
        for r, (name, who, when) in enumerate(rows):
            y = 122 + r * 30
            a += text(x + 24, y, name, 16, INK)
            a += text(x + 190, y, who, 15, DIM)
            a += text(x + 262, y, when, 15, DIM)
    a += band(234, '件数も並び順も書き方も違う。それでも、どの項目にも同じ列がある')
    return svg('件数も並びも違うが、どの項目にも誰が・いつまでが在る', a, 302)


# ───────────────── 5本目 ─────────────────

def l5_grown():
    """もとの指示に、3つだけを書き足した。"""
    a = text(0, 24, 'もとの指示', 18, DIM)
    a += rect(0, 40, 300, 64, PAPER, 10, LINE)
    a += text(150, 80, '「課題を整理して」', 20, INK, 700, 'middle')
    a += text(0, 132, '書き足した3つ', 18, ACCENT, 700)
    for i, (name, body) in enumerate([('意味', '次の打ち合わせで決める必要があるもの（決定事項とリスクは含めない）'),
                                      ('範囲', '今回の案件の定例、直近5回だけを見る'),
                                      ('条件', 'どの項目にも、誰が決めるかと、いつまでかを必ず入れる')]):
        y = 150 + i * 56
        a += rect(0, y, 86, 44, ACCENT, 8, ACCENT)
        a += text(43, y + 29, name, 18, PAPER, 700, 'middle')
        a += rect(98, y, 574, 44, PAPER, 8, LINE)
        a += text(118, y + 29, body, 16, INK)
    a += rect(712, 40, 400, 278, PANEL, 12)
    a += text(736, 78, '指定していないこと', 18, DIM)
    for i, s in enumerate(['見出しの文言', '出力の形式', '項目の並び順', '表にするかどうか', '1件あたりの字数']):
        a += cross(752, 112 + i * 40, DIM, .6)
        a += text(778, 118 + i * 40, s, 17, DIM)
    return svg('もとの指示に、意味・範囲・条件の3つだけを書き足した', a, 330)


def l5_swap():
    """指示を固定して、材料のほうを替える。"""
    a = text(0, 24, '指示は固定', 18, DIM)
    a += rect(0, 100, 268, 84, ACCENT, 12, ACCENT)
    a += text(134, 136, 'いまの指示', 20, PAPER, 700, 'middle')
    a += text(134, 162, '3つを足したもの', 15, PAPER, anchor='middle')
    a += text(310, 24, '替えたのは材料', 18, DIM)
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
        a += text(700, y + 36, '誰が・いつまで付きの、未決事項の一覧', 16, INK)
        a += text(1088, y + 36, '', 15, DIM, anchor='end')
    a += band(300, '中身も件数も違う。返ってくるものの形は、どれも同じである')
    return svg('指示を固定して材料を替えても、返ってくるものの形は同じになる', a, 368)


def l5_axis():
    """抽象の高さの3つの位置を、縦に並べて比べる。"""
    AX = 52
    a = path(f'M{AX} 330 V22 M{AX - 7} 32 l7 -10 7 10', LINE, 2)
    a += text(0, 18, '高い', 16, DIM, 700)
    a += text(0, 348, '低い', 16, DIM, 700)
    rows = [('高すぎる', '広い書き方', '「課題を整理して」', '何が返るか決まらない', False),
            ('いまの指示', '目的に合う抽象の高さ', '意味・範囲・条件を書いた', '材料を替えても目的の中に収まる', True),
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


def l5_three_and_height():
    """3つを決める作業が、抽象の高さを合わせる作業だった。"""
    a = text(0, 24, '2本目から4本目まで、やってきたこと', 18, DIM)
    for i, (name, did, effect) in enumerate([
            ('意味', '何を並べるか', '広さの上限が決まる'),
            ('範囲', 'どこから拾うか', '読む範囲が決まる'),
            ('条件', 'どの項目にも必ず入れるもの', '1件ずつの中身が決まる')]):
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
    a += band(254, '抽象度を上げる、では何をするかが決まらない。この3つなら書ける')
    return svg('言葉・範囲・条件を決める作業が、抽象の高さを合わせる作業だった', a, 322)


def l5_fit():
    """目指すことと、目指さないこと。"""
    a = text(0, 24, '目指すこと', 18, ACCENT, 700)
    a += rect(0, 40, 536, 178, PAPER, 12, ACCENT)
    a += check(36, 84)
    a += text(64, 91, '返ってくるものが、目的から外れない', 19, INK, 700)
    a += check(36, 138)
    a += text(64, 145, '同じ指示を、次の週も別の案件でも使える', 19, INK, 700)
    a += text(36, 190, '中身も件数も毎回違ってよい。', 16, DIM)
    a += text(576, 24, '目指さないこと', 18, DIM)
    a += rect(576, 40, 536, 178, PANEL, 12)
    a += cross(612, 84, DIM, .8)
    a += text(640, 91, '毎回おなじ文章が返ってくる', 19, DIM, 700)
    a += cross(612, 138, DIM, .8)
    a += text(640, 145, '1回ごとに指示を書き直す', 19, DIM, 700)
    a += text(612, 190, '揃えにいくほど、その1回にしか使えなくなる。', 16, DIM)
    a += band(248, '毎回書き直さずに、同じ指示を使える')
    return svg('目指すのは目的の中に収まることで、同じ文章が返ることではない', a, 316)


# ───────────────── 6本目 ─────────────────

def l6_symptoms():
    """3つを足しても残る、3つの症状。"""
    a = ''
    items = [('形が違う', '未決事項の一覧に、', '相談したいことが混じった'),
             ('書き方が揃わない', '期限が「今月中」と', '「9月30日」で混ざった'),
             ('量が違う', '前の週には出てきた項目が、', '今週は出てこない')]
    for i, (name, l1, l2) in enumerate(items):
        x = i * 384
        a += rect(x, 16, 344, 148, PAPER, 12, LINE)
        a += text(x + 24, 60, name, 21, ACCENT, 700)
        a += path(f'M{x + 24} 78 H{x + 320}', LINE, 1)
        a += text(x + 24, 114, [l1, l2], 17, INK, gap=28)
    a += band(186, 'ここで指示の全体を書き直すと、たいてい前より悪くなる')
    return svg('3つを足しても残る、3つの症状', a, 254)


def l6_where():
    """症状で、見る場所が決まる。"""
    a = text(0, 24, '症状', 18, DIM)
    a += text(596, 24, '見る場所', 18, ACCENT, 700)
    rows = [('返ってくる形が違う', '意味', '意味が2つ以上ある語が残っていないか'),
            ('拾ってくる量が違う', '範囲', '材料のどこまでを見るかが書いてあるか'),
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


def l6_one_at_a_time():
    """1文だけ変えて、3回で確かめる。"""
    a = text(0, 24, '直し方', 18, DIM)
    a += rect(0, 44, 268, 92, ACCENT, 12, ACCENT)
    a += text(134, 84, '1文だけ変える', 20, PAPER, 700, 'middle')
    a += text(134, 110, '2つ同時に直さない', 15, PAPER, anchor='middle')
    a += arrow(278, 90, 318, 90)
    a += rect(328, 44, 268, 92, PAPER, 12, LINE)
    a += text(462, 84, '同じ材料で3回', 20, INK, 700, 'middle')
    a += text(462, 110, '1回では分からない', 15, DIM, anchor='middle')
    a += arrow(606, 90, 646, 90)
    a += rect(656, 44, 456, 92, PANEL, 12)
    a += check(692, 78)
    a += text(716, 85, '揃った　　効いている', 18, INK, 700)
    a += cross(692, 114, DIM, .7)
    a += text(716, 121, '揃わない　その文を戻して、別の場所を見る', 17, DIM)
    a += band(176, '1回返ってきたものだけでは、揺らいでいるかどうかを見分けられない')
    return svg('1文だけ変え、同じ材料で3回試して揃い方を見る', a, 252)


def l6_two_goals():
    """目的が2つ入った指示は、逆向きの作業へ引かれる。"""
    a = rect(340, 24, 432, 68, ACCENT, 12, ACCENT)
    a += text(556, 66, '「課題を整理して、次の打ち手も出して」', 20, PAPER, 700, 'middle')
    a += path('M470 92 V126 H286 V158', ACCENT)
    a += path('M280 152 l6 6 6 -6', ACCENT, 2)
    a += path('M642 92 V126 H826 V158', ACCENT)
    a += path('M820 152 l6 6 6 -6', ACCENT, 2)
    for x, name, sub in [(0, '決める材料を作る', '漏れなく並べる'), (572, '案を出す', '絞って深く考える')]:
        a += rect(x, 164, 540, 92, PAPER, 12, LINE)
        a += text(x + 270, 204, name, 21, INK, 700, 'middle')
        a += text(x + 270, 234, sub, 17, DIM, anchor='middle')
    a += band(278, '目的が2つなら、指示も2つに分ける')
    return svg('目的が2つ入った指示は、逆向きの作業へ引かれる', a, 346)


def l6_whole():
    """6本ぶんを1枚にする。"""
    a = text(0, 24, '返ってくるものが揺らいだら', 18, DIM)
    W4, G4 = 236, 56
    for i, (name, sym) in enumerate([('意味', '種類が変わる'), ('範囲', '量が変わる'), ('条件', '中身が揃わない')]):
        x = i * (W4 + G4)
        a += rect(x, 44, W4, 88, PAPER, 12, ACCENT)
        a += text(x + W4 / 2, 82, name, 22, INK, 700, 'middle')
        a += text(x + W4 / 2, 110, sym, 16, DIM, anchor='middle')
        a += arrow(x + W4 + 10, 88, x + W4 + G4 - 10, 88)
    x = 3 * (W4 + G4)
    a += rect(x, 44, W4, 88, ACCENT, 12, ACCENT)
    a += text(x + W4 / 2, 82, '目的が1つか', 21, PAPER, 700, 'middle')
    a += text(x + W4 / 2, 110, 'それでも揃わなければ', 15, PAPER, anchor='middle')
    a += text(0, 176, '直すときは', 18, DIM)
    a += rect(0, 192, 536, 60, PANEL, 12)
    a += text(268, 230, '直すのは1か所、確かめるのは3回', 20, INK, 700, 'middle')
    a += rect(576, 192, 536, 60, PANEL, 12)
    a += text(844, 230, '目的が2つなら、指示を2つに分ける', 18, INK, anchor='middle')
    a += band(282, '症状は手がかり。前の1つが決まっていなければ、次を直しても揃わない')
    return svg('揺らいだら3つを順に見て、揃わなければ目的を見る', a, 350)


def cover(no):
    """表紙。6本の中で、いまどこに在るかを示す。"""
    names = ['抽象の高さを知る', '意味を決める', '範囲を決める', '条件を決める', '抽象の高さを合わせる', '揺らぎを直す']
    a = ''
    for i, name in enumerate(names):
        x = i * 188
        on = (i + 1) == no
        a += rect(x, 30, 168, 74, ACCENT if on else PAPER, 10, ACCENT if on else LINE)
        a += text(x + 84, 62, f'{i + 1}本目', 15, PAPER if on else DIM, anchor='middle')
        a += text(x + 84, 88, name, 19, PAPER if on else INK, 700, 'middle')
        if i < 5:
            a += path(f'M{x + 168} 67 H{x + 188}', LINE, 1)
    return svg(f'6本のうち、{no}本目', a, 116)


def l1_ladder():
    """同じ指示を3つの抽象の高さで書くと、上が抽象、下が具体になる。"""
    AX = 210
    a = path(f'M{AX} 320 V24 M{AX - 7} 34 l7 -10 7 10', LINE, 2)
    a += text(AX + 16, 20, '高い', 15, ACCENT, 700)
    a += text(AX + 16, 344, '低い', 15, ACCENT, 700)

    a += rect(0, 24, 176, 130, PANEL, 12)
    a += text(88, 74, '抽象', 30, ACCENT, 700, 'middle')
    a += text(88, 108, '当てはまる', 15, DIM, anchor='middle')
    a += text(88, 130, '場面が広い', 15, DIM, anchor='middle')
    a += rect(0, 196, 176, 130, PANEL, 12)
    a += text(88, 246, '具体', 30, ACCENT, 700, 'middle')
    a += text(88, 280, '当てはまる', 15, DIM, anchor='middle')
    a += text(88, 302, '場面が狭い', 15, DIM, anchor='middle')

    rows = [('「いい感じにまとめて」', 'どんな資料にも当てはまる', '何が返ってくるかは決まらない'),
            ('「課題を整理して」', '課題らしきものを並べる指示に当てはまる', 'どれを拾うかはAIが決める'),
            ('「1行目に通知の方式、2行目に締めの日」', 'この議事録の、この1回だけ', '来週の議事録には使えない')]
    for i, (ask, where, note) in enumerate(rows):
        y = 24 + i * 108
        a += circle(AX, y + 42, 11, PAPER, LINE)
        a += rect(266, y, 846, 84, PAPER, 12, LINE)
        a += text(294, y + 38, ask, 21, INK, 700)
        a += text(294, y + 66, where, 16, DIM)
        a += text(1084, y + 52, note, 16, DIM, anchor='end')
    a += band(356, 'この並びのどこにあるかが、抽象の高さである')
    return svg('同じ議事録への指示を、当てはまる場面の広い順に3つ並べる。上が抽象、下が具体である', a, 424)


def l6_reproducible():
    """抽象の高さが合っていないと返りは散り、合っていると毎回目的の中に収まる。"""
    def target(cx, label, pts, hot):
        a = circle(cx, 150, 104, PAPER, ACCENT if hot else LINE)
        a += text(cx, 76, label, 17, ACCENT if hot else DIM, 700, 'middle')
        for dx, dy in pts:
            a += circle(cx + dx, 150 + dy, 22, ACCENT if hot else DIM, ACCENT if hot else DIM)
        return a

    a = text(0, 24, '抽象の高さが合っていない指示', 18, DIM)
    a += target(246, '目的', [(-30, -20), (-150, 60), (140, -70)], False)
    a += text(246, 290, '目的の外へ出る回がある', 17, DIM, anchor='middle')

    a += path('M556 20 V300', LINE, 1)

    a += text(612, 24, 'ちょうどいい抽象の高さの指示', 18, ACCENT, 700)
    a += target(866, '目的', [(-40, -30), (20, 40), (50, -20)], True)
    a += text(866, 290, '中身は毎回違う。それでも、どれも目的の中', 17, DIM, anchor='middle')
    a += band(322, '同じ文章が返ることではない。目的に対して毎回外さないこと ── これを再現性と呼びます')
    return svg('抽象の高さが合っていないと目的の外へ出る回があり、合っていると毎回目的の中に収まる', a, 390)
