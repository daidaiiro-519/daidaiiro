"""現行の図から派生させた図。実行時の書き換えをやめ、実体として置く。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, card, band)


def t1_fit():
    """目的レベルが明確なプロンプトなら、入力する議事録を替えても、出力がどれも目的から外れない。1つずつの議事録が具体、どの議事録にも共通する目的が抽象。"""
    def badge(x, y, t):
        return rect(x, y, 52, 26, ACCENT, 6, ACCENT) + text(x + 26, y + 18, t, 14, PAPER, 700, 'middle')
    # 左：具体。1つずつ違う議事録
    a = rect(0, 0, 300, 292, PAPER, 12, LINE)
    a += badge(20, 18, '具体') + text(84, 37, '1つずつ違う議事録', 16, INK, 700)
    for i, name in enumerate(['今週の議事録', '来週の議事録', '別の案件の議事録']):
        y = 64 + i * 72
        a += rect(20, y, 260, 56, PAPER, 10, LINE)
        a += icon(36, y + 13, 'doc', DIM, .55)
        a += text(76, y + 35, name, 16, INK, 700)
    a += arrow(306, 146, 340, 146)
    a += rect(348, 104, 240, 84, PAPER, 10, LINE)
    a += text(468, 140, '目的レベルが', 16, INK, 700, 'middle') + text(468, 166, '明確なプロンプト', 16, INK, 700, 'middle')
    a += arrow(594, 146, 636, 146)
    # 右：抽象。どの議事録にも共通する目的。出力は、どれもその中に収まる
    a += rect(644, 0, 468, 292, PAPER, 12, ACCENT)
    a += badge(664, 18, '抽象') + text(728, 37, 'どの議事録にも共通する目的', 16, INK, 700)
    a += text(664, 76, '優先順位を付ける', 20, ACCENT, 700)
    for i, name in enumerate(['今週の結果', '来週の結果', '別の案件の結果']):
        y = 100 + i * 62
        a += rect(664, y, 428, 48, PAPER, 10, LINE)
        a += text(684, y + 30, name, 15, INK, 700)
        a += check(888, y + 24, ACCENT, .5) + text(908, y + 30, '目的から外れない', 15, INK, 700)
    a += rect(0, 312, 1112, 52, PAPER, 10, LINE)
    a += text(24, 344, '中身も件数も、議事録ごとに違う。それでも、どれも優先順位を付けるのに使える', 16, INK, 700)
    return svg('1つずつ違う議事録（具体）に、目的レベルが明確なプロンプトを使うと、出力はどれも、どの議事録にも共通する目的（抽象）である、優先順位を付けることから外れない', a, 368)


def t1_height():
    """2つのプロンプトは、目的レベルの両端にあり、どちらも作業に合った目的レベルが明確になっていない。上のプロンプトは多くに当てはまって出力が決まらず、具体に寄りすぎたプロンプトは1つに決まって、その議事録にしか使えない。"""
    def badge(x, y, t):
        return rect(x, y, 64, 24, ACCENT, 6, ACCENT) + text(x + 32, y + 17, t, 13, PAPER, 700, 'middle')
    a = text(100, 18, 'プロンプト', 14, DIM) + text(612, 18, '出力', 14, DIM) + text(860, 18, '使える議事録', 14, DIM)
    a += text(0, 52, '抽象', 18, INK, 700) + text(0, 72, 'なぜ', 13, DIM)
    a += text(0, 300, '具体', 18, INK, 700) + text(0, 322, 'どのように', 13, DIM)
    a += path('M80 328 V30 m-6 6 l6 -6 6 6', LINE, 2)
    rows = [(32, '作業の目的が書かれていないプロンプト', '「課題を整理して」', '業務の流れの順 ・ 期限の順 ・ 機能ごと、どれも当てはまる',
             ('困り方1', '実行するたびに変わる'), ('ok', 'どの週の議事録にも使える')),
            (232, '具体に寄りすぎたプロンプト', '「1行目に承認の経路、2行目に申請の締め日」', '当てはまるものが、この議事録の1つしかない',
             ('ok', '1つに決まる'), ('困り方2', 'この議事録にしか使えない'))]
    for y, kind, quote, why, ret, use in rows:
        cy = y + 44
        a += circle(80, cy, 6, PAPER, LINE) + path(f'M86 {cy} H100', LINE, 1)
        a += rect(100, y, 496, 88, PAPER, 10, LINE)
        a += text(120, y + 26, kind, 13, DIM, 700) + text(120, y + 52, quote, 16, INK, 700) + text(120, y + 76, why, 13, DIM)
        a += rect(612, y, 496, 88, PAPER, 10, LINE) + path(f'M860 {y + 14} V{y + 74}', LINE, 1)
        for x, (tag, t) in ((612, ret), (860, use)):
            bad = tag != 'ok'
            if bad:
                a += badge(x + 18, y + 16, tag) + text(x + 18, y + 68, t, 16, INK, 700)
            else:
                a += check(x + 30, cy, ACCENT, .5) + text(x + 50, cy + 6, t, 15, INK, 700)
    # 真ん中：作業の目的。どちらのプロンプトも、ここで書かれていない
    box = rect(100, 140, 1008, 72, PAPER, 10, ACCENT)
    a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 5"')
    a += circle(80, 176, 7, ACCENT, ACCENT) + path('M87 176 H100', LINE, 1)
    a += text(120, 168, '作業の目的', 13, ACCENT, 700)
    a += text(120, 194, 'ここで目的レベルが明確なプロンプトは、まだない', 17, ACCENT, 700)
    return svg('2つのプロンプトは目的レベルの両端にあり、どちらも作業に合った目的レベルが明確になっていない。作業の目的が書かれていないプロンプトは実行するたびに変わり、具体に寄りすぎたプロンプトはこの議事録にしか使えない', a, 332)


def t1_order():
    """意味 ・ 範囲 ・ 条件の順に追記すると、プロンプトの目的レベルが明確になる。前の1つが決まっていないと、次の1つを決められない。具体に寄りすぎない。"""
    rows = [(None, 'もとのプロンプト', '「課題を整理して」', 'ここから始める', '作業の目的が書かれていない'),
            ('意味', '課題が何を指すか', '「課題とは、決定しないと案件の進行が止まるもの」',
             '最初に決める', '何を並べるかが決まらないと、抽出するものが決まらない'),
            ('範囲', 'どこまでを見るか', '「見るのは、今回の案件で12月末の切り替えまでに決めるもの」',
             '意味を決めてから', '範囲は、その意味が通じるところまでを決める'),
            ('条件', '何を入れ、落とし、並べるか', '「どの項目にも、誰が決めるかと、いつまでかを入れる」',
             '範囲を決めてから', '範囲を広げると、誰が決めるかを書けなくなる')]
    a = text(100, 18, 'プロンプトに追記するもの', 14, DIM) + text(724, 18, 'この順である理由', 14, DIM)
    a += text(0, 48, '抽象', 18, INK, 700) + text(0, 68, 'なぜ', 13, DIM)
    a += text(0, 352, '具体', 18, INK, 700) + text(0, 374, 'どのように', 13, DIM)
    a += path('M80 378 V28 m-6 6 l6 -6 6 6', LINE, 2)
    for i, (name, sub, quote, why, because) in enumerate(rows):
        y = 30 + i * 76
        cy = y + 31
        last = i == len(rows) - 1
        a += circle(80, cy, 6, ACCENT if last else PAPER, ACCENT if last else LINE) + path(f'M86 {cy} H100', LINE, 1)
        a += rect(100, y, 600, 62, PAPER, 10, ACCENT if last else LINE)
        if name:
            a += circle(124, cy, 13, ACCENT, ACCENT) + text(124, cy + 5, str(i), 14, PAPER, 700, 'middle')
            a += text(148, y + 25, '＋' + name, 16, ACCENT, 700) + text(212, y + 25, sub, 13, DIM)
            a += text(148, y + 50, quote, 15, INK, 700)
        else:
            a += text(124, y + 25, sub, 13, DIM) + text(124, y + 50, quote, 16, INK, 700)
        a += rect(724, y, 388, 62, PAPER, 10, LINE)
        a += text(744, y + 26, why, 15, INK, 700) + text(744, y + 49, because, 13, DIM)
    a += text(0, 296, '作業の目的', 13, ACCENT, 700)
    # 下：項目の指定。ここまでは下りない
    box = rect(100, 336, 1012, 46, PAPER, 10, LINE)
    a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 5"')
    a += circle(80, 359, 6, PAPER, LINE) + path('M86 359 H100', LINE, 1)
    a += text(124, 365, '具体に寄りすぎない ── どう並べるかまで書くと、その1回にしか使えないプロンプトに戻る', 15, DIM)
    return svg('意味・範囲・条件の順に追記すると、プロンプトの目的レベルが明確になる。前の1つが決まっていないと次の1つを決められない。具体に寄りすぎない', a, 392)


def t1_symptoms():
    """2つの困り方を、左右同じ形で並べる。どちらも、同じプロンプトを何度か使うと起きる。"""
    def badge(x, y, t):
        return rect(x, y, 64, 24, ACCENT, 6, ACCENT) + text(x + 32, y + 17, t, 13, PAPER, 700, 'middle')

    panels = [
        (0, '困り方1', '毎回、違うものが出力される', ('作業の目的が書かれていないプロンプト', '「課題を整理して」'), '同じ議事録で3回実行する',
         [('1回目', '業務の流れの順に並ぶ', False), ('2回目', '決定の期限の順に並ぶ', False),
          ('3回目', '機能ごとに並ぶ', False)], '並び順が、毎回違う'),
        (572, '困り方2', 'その1回にしか使えない', ('具体に寄りすぎたプロンプト', '「1行目に承認の経路、2行目に申請の締め日」'), '週を替えて頼む',
         [('今週の議事録', 'そのとおりに返る', False), ('来週の議事録', '承認の話が無く、指定が合わない', True)],
         '毎週、プロンプトを書き直すことになる'),
    ]
    a = path('M556 0 V364', LINE, 1)
    for x, tag, title, instr, how, rows, end in panels:
        a += badge(x, 2, tag)
        a += text(x + 76, 20, title, 18, INK, 700)
        kind, instr = instr
        a += rect(x, 40, 540, 66, PAPER, 10, LINE)
        a += text(x + 20, 64, kind, 14, DIM, 700)
        a += text(x + 20, 92, instr, 16 if len(instr) > 12 else 18, INK, 700)
        a += text(x, 136, how, 14, DIM)
        for i, (label, result, bad) in enumerate(rows):
            y = 150 + i * 50
            a += rect(x, y, 540, 40, PAPER, 8, LINE)
            a += text(x + 20, y + 26, label, 14, DIM)
            if bad:
                a += cross(x + 142, y + 20, DIM, .6)
            a += text(x + (162 if bad else 140), y + 26, result, 15, INK, 700)
        a += rect(x, 312, 540, 52, PAPER, 10, ACCENT)
        a += text(x + 20, 344, end, 18, ACCENT, 700)
    return svg('2つの困り方 ── 毎回違うものが返ることと、その1回にしか使えないこと', a, 366)


def t3_after():
    """範囲を足す前と、足したあとを並べる。"""
    a = text(0, 22, '足す前　意味だけを書いたプロンプト ── 読む範囲が3通り', 17, DIM)
    for i, (no, n, where) in enumerate([('1回目', '4件', '今回の定例だけ'),
                                        ('2回目', '11件', '合同会議まで'),
                                        ('3回目', '18件', '前の案件まで')]):
        x = i * 384
        a += rect(x, 38, 344, 48, PAPER, 10, LINE)
        a += text(x + 24, 68, no, 14, DIM)
        a += text(x + 80, 68, where, 17, DIM)
        a += text(x + 320, 69, n, 18, DIM, 700, 'end')
    a += text(0, 122, '追記したあと　範囲を追記した ── 読む範囲は3回とも同じ', 17, ACCENT, 700)
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
    """範囲の線を引くと、外側は抽出されない。"""
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
    a += text(624, 80, 'プロンプトに追記する文', 16, DIM)
    a += text(624, 118, ['見るのは、今回の案件の定例、', '直近5回だけです。合同会議と、', '前の案件の議事録は含めません。'], 19, INK, gap=30)
    a += path('M624 222 H1084', LINE, 1)
    a += text(624, 256, '範囲の外には、別の業務の整理が入る', 17, ACCENT, 700)
    return svg('範囲の線を引くと、その外側からは抽出されない', a, 330)


def t3_folder():
    """フォルダごと入力すると、抽出する量が毎回違う。"""
    a = text(0, 24, '入力したもの', 18, DIM)
    a += rect(0, 40, 300, 192, PAPER, 12, LINE)
    for i, (name, n) in enumerate([('今回の案件の定例', '5回分'), ('他部署との合同会議', '2回分'),
                                   ('前の案件の議事録', '3回分')]):
        y = 76 + i * 56
        a += icon(24, y - 18, 'doc', DIM, .5)
        a += text(66, y + 6, name, 18, INK, 700)
        a += text(276, y + 6, n, 16, DIM, anchor='end')
    a += arrow(310, 136, 352, 136)
    a += rect(362, 102, 226, 68, PANEL, 10)
    a += text(475, 134, '同じプロンプト', 20, INK, 700, 'middle')
    a += text(475, 156, '意味は書いてある', 15, DIM, anchor='middle')
    for i, (no, n, w) in enumerate([('1回目', '4件', 68), ('2回目', '11件', 186), ('3回目', '18件', 304)]):
        y = 44 + i * 74
        a += path(f'M588 136 H616 V{y + 26} H648', LINE)
        a += path(f'M639 {y + 20} l9 6 -9 6', LINE, 2)
        a += text(648, y + 32, no, 16, DIM)
        a += rect(706, y + 10, w, 34, ACCENT if i == 2 else PANEL, 8)
        a += text(706 + w + 14, y + 33, n, 20, INK, 700)
    a += text(648, 266, '同じプロンプトでも、抽出する量が4倍以上違う', 17, DIM)
    return svg('フォルダごと入力すると、抽出する量が毎回違う', a, 282)


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
    """L4-S3　意味と範囲でどの課題を抽出するかが決まり、条件で表に何を入れ、何を落とし、どう並べるかが決まる。主役は条件のカードと、それが作る表の部分。"""
    from v1_visuals import _down
    # 左上：意味と範囲
    a = rect(0, 0, 340, 140, PAPER, 12, LINE)
    a += text(24, 36, '意味と範囲　どの課題を抽出するか', 18, INK, 700)
    for k, (tag, body) in enumerate([('意味', '並ぶ課題の種類'), ('範囲', '抽出する場所')]):
        y = 56 + k * 42
        a += rect(24, y, 72, 30, PAPER, 8, LINE) + text(60, y + 21, tag, 15, INK, 700, 'middle')
        a += text(112, y + 21, body, 16, INK)
    # 左下：条件（主役）
    a += rect(0, 160, 340, 180, PAPER, 12, ACCENT)
    a += text(24, 196, '条件　表にどう書くか', 18, ACCENT, 700)
    for k, (tag, body) in enumerate([('入れる', '誰が決めるか・いつまでか'), ('落とす', 'すでに決まったもの'), ('並べる', '期限の早い順')]):
        y = 214 + k * 40
        a += rect(24, y, 72, 30, ACCENT, 8, ACCENT) + text(60, y + 21, tag, 15, PAPER, 700, 'middle')
        a += text(112, y + 21, body, 16, INK)
    a += arrow(340, 70, 400, 70) + arrow(340, 230, 400, 230, ACCENT)
    # 右：出力される課題管理表
    a += rect(400, 0, 712, 340, PAPER, 12, LINE)
    a += text(424, 36, '出力される課題管理表', 18, INK, 700)
    a += text(424, 72, '課題', 15, DIM, 700)
    a += text(660, 72, '決める担当', 15, ACCENT, 700) + text(800, 72, '期限', 15, ACCENT, 700)
    a += path('M424 84 H1088', LINE, 1)
    rows = [('承認の経路', '経理と開発', '9/30'), ('申請の締め日', '経理', '9/30'), ('旧システムの停止日', 'PM', '10/7')]
    for k, (name, who, when) in enumerate(rows):
        y = 118 + k * 46
        a += text(424, y, name, 17, INK) + text(660, y, who, 17, INK) + text(800, y, when, 17, INK)
    # 落とした行
    y = 118 + 3 * 46
    a += text(424, y, '申請画面の試作', 17, DIM) + text(660, y, '決定済み', 17, DIM)
    a += path(f'M424 {y - 6} H860', ACCENT, 2)
    a += text(900, y, '落とす', 16, ACCENT, 700)
    # 並べる順
    a += _down(916, 104, 210, ACCENT)
    a += text(940, 150, '期限の', 16, ACCENT, 700) + text(940, 174, '早い順', 16, ACCENT, 700)
    a += text(424, 318, '赤い部分は、条件で決まる', 15, DIM)
    return svg('意味と範囲は何を抽出するかを決め、条件は担当と期限を入れ、決まったものを落とし、期限の順に並べる', a, 344)


def _doc(x, y, w, h):
    """角を折った書類の形。"""
    d = f'M{x} {y} H{x + w - 22} L{x + w} {y + 22} V{y + h} H{x} Z M{x + w - 22} {y} V{y + 22} H{x + w}'
    return path(d, LINE, 2, PAPER) + rect(x, y, w, h, 'none', 0, 'none')


def t4_missing():
    """意味と範囲を追記したプロンプトで3回実行した課題管理表。抽出した課題は合っているが、各項目の担当と期限が欠け、決定済みが混じり、期限の順にも並ばない。"""
    runs = [('担当も期限も入っていない', [('承認の経路', None, None), ('申請の締め日', None, None), ('既存データの移行', None, None), ('旧システムの停止日', None, None)]),
            ('期限が入っていない', [('既存データの移行', '開発', None), ('承認の経路', '経理と開発', None), ('旧システムの停止日', 'PM', None), ('申請の締め日', '経理', None)]),
            ('決定済みが混じり、期限の順でない', [('旧システムの停止日', 'PM', '10/7'), ('承認の経路', '経理と開発', '9/30'), ('申請画面の試作', '決定済み', '―'),
                                         ('申請の締め日', '経理', '9/30'), ('既存データの移行', '開発', '10/7')])]
    a = ''
    for i, (what, rows) in enumerate(runs):
        x = i * 380
        a += _doc(x, 0, 352, 300)
        a += text(x + 18, 28, f'出力　{i + 1}回目', 13, DIM, 700)
        a += text(x + 18, 54, what, 15, ACCENT, 700)
        a += text(x + 18, 88, '課題', 12, DIM, 700) + text(x + 178, 88, '決める担当', 12, DIM, 700) + text(x + 284, 88, '期限', 12, DIM, 700)
        a += path(f'M{x + 18} 98 H{x + 334}', LINE, 1)
        for j, (item, who, due) in enumerate(rows):
            y = 126 + j * 32
            done = who == '決定済み'
            a += text(x + 18, y, item, 14, ACCENT if done else INK, 700 if done else 400)
            a += text(x + 178, y, who or 'なし', 14, ACCENT if (who is None or done) else INK, 700 if (who is None or done) else 400)
            a += text(x + 284, y, due or 'なし', 14, ACCENT if due is None else INK, 700 if due is None else 400)
    return svg('意味と範囲を追記したプロンプトで3回実行すると、抽出した課題は合っているが、担当や期限が欠ける回、決定済みが混じり期限の順に並ばない回がある', a, 304)


def t4_unwritten():
    """L4-S2　PMの頭の中にあった4つの期待が、プロンプトには書かれていない。主役は「条件　書いていない」。"""
    from v1_visuals import _sheet, _robot_at
    # 左：PMの頭の中
    a = rect(0, 0, 520, 284, PAPER, 12, LINE)
    a += person(36, 22, .45, INK) + text(64, 40, 'PMの頭の中にあった期待', 18, INK, 700)
    a += text(24, 84, '期待', 14, DIM, 700) + text(256, 84, '書いていないと困ること', 14, DIM, 700)
    for i, (what, why) in enumerate([('誰が決めるか', '誰に確認すればいいか分からない'),
                                     ('いつまでに決めるか', 'どれを先に決めるか比べられない'),
                                     ('決まったものは入れない', '全部の項目を確かめ直す'),
                                     ('期限の順に並べる', '上から順に優先順位を付けられない')]):
        y = 96 + i * 46
        a += path(f'M24 {y} H496', LINE, 1)
        a += text(24, y + 30, what, 17, INK, 700) + text(256, y + 30, why, 15, DIM)
    # 真ん中：頭の中からプロンプトへ、渡っていない
    a += arrow(520, 157, 600, 157) + cross(560, 157, ACCENT, .6)
    # 右：いまのプロンプト
    a += _sheet(600, 0, 512, 284)
    a += text(624, 34, 'いまのプロンプト', 14, DIM, 700)
    for i, (tag, body) in enumerate([('意味', '決定しないと案件の進行が止まるもの'),
                                     ('範囲', '今回の案件、12月末の切り替えまで')]):
        y = 54 + i * 44
        a += rect(624, y, 88, 30, PAPER, 8, LINE) + text(668, y + 21, '＋' + tag, 14, INK, 700, 'middle')
        a += text(728, y + 21, body, 16, INK)
    a += rect(624, 142, 88, 30, PAPER, 8, ACCENT) + text(668, 163, '＋条件', 14, ACCENT, 700, 'middle')
    a += cross(742, 157, ACCENT, .6) + text(762, 164, '書いていない', 20, ACCENT, 700)
    a += path('M624 198 H1088', LINE, 1)
    a += _robot_at(632, 214, INK) + text(704, 238, 'AIは、書いてあることだけで実行する', 16, INK, 700)
    a += text(704, 262, '期待が入る回と、入らない回が出る', 14, DIM)
    # 下：言い切り
    a += rect(0, 304, W, 52, PAPER, 12, LINE)
    a += text(W / 2, 337, 'AIの調子が悪いのではない。書いていないことは、頼んでいないのと同じ', 19, INK, 700, 'middle')
    return svg('PMの頭の中にあった4つの期待が、プロンプトには書かれていないので、AIは期待が入る回と入らない回を出す', a, 360)


def t5_axis():
    """抽象度の3つの位置を、縦に並べて比べる。"""
    AX = 52
    a = path(f'M{AX} 330 V22 M{AX - 7} 32 l7 -10 7 10', LINE, 2)
    a += text(0, 18, '高い', 16, DIM, 700)
    a += text(0, 348, '低い', 16, DIM, 700)
    rows = [('高すぎる', '広い書き方', '「課題を整理して」', '何が返るか決まらない', False),
            ('いまのプロンプト', '目的に合う抽象度', '意味・範囲・条件を書いた', '入力する議事録を替えても目的の中に収まる', True),
            ('低すぎる', '狭い書き方', '1行目に通知の方式、2行目に締めの日', 'その1回にしか使えない', False)]
    for i, (name, kind, ex, note, hot) in enumerate(rows):
        y = 22 + i * 108
        a += circle(AX, y + 42, 13, ACCENT if hot else PAPER, ACCENT if hot else LINE)
        a += rect(96, y, 1016, 84, PAPER if hot else PANEL, 12, ACCENT if hot else 'none')
        a += text(124, y + 36, name, 20, ACCENT if hot else INK, 700)
        a += text(124, y + 64, kind, 15, DIM)
        a += text(330, y + 36, ex, 19, INK)
        a += text(330, y + 64, note, 16, DIM)
    return svg('高すぎる側と低すぎる側の間に、いまのプロンプトがある', a, 356)


def t5_grown():
    """もとのプロンプトに、3つだけを追記した。"""
    a = text(0, 24, 'もとのプロンプト', 18, DIM)
    a += rect(0, 40, 300, 64, PAPER, 10, LINE)
    a += text(150, 80, '「課題を整理して」', 20, INK, 700, 'middle')
    a += text(0, 132, '追記した3つ', 18, ACCENT, 700)
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
    return svg('もとのプロンプトに、意味・範囲・条件の3つだけを追記した', a, 330)


def t5_swap():
    """プロンプトを固定して、材料のほうを替える。"""
    a = text(0, 24, 'プロンプトは固定', 18, DIM)
    a += rect(0, 100, 268, 84, ACCENT, 12, ACCENT)
    a += text(134, 136, 'いまのプロンプト', 20, PAPER, 700, 'middle')
    a += text(134, 162, '3つを追記したもの', 15, PAPER, anchor='middle')
    a += text(310, 24, '替えたのは入力するもの', 18, DIM)
    for i, (mat, n, body) in enumerate([('今週の議事録', '5件', '承認の経路ほか'),
                                        ('来週の議事録', '6件', '移行の範囲ほか'),
                                        ('別の案件の議事録', '9件', '連携の方式ほか')]):
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
    a += band(300, '中身も件数も違う。出力の形は、どれも同じである')
    return svg('プロンプトを固定して入力する議事録を替えても、出力の形は同じになる', a, 368)


def t5_three_and_height():
    """3つを決める作業が、目的レベルを合わせる作業だった。"""
    a = text(0, 24, '2本目から4本目まで、やってきたこと', 18, DIM)
    for i, (name, did, effect) in enumerate([
            ('意味', '何を並べるか', '並ぶものの種類が安定する'),
            ('範囲', 'どこから抽出するか', '抽出する量が安定する'),
            ('条件', '入れるもの・落とすもの・並べる順', '各項目の内容が安定する')]):
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
    a += text(1007, 146, '抽象度が合う', 20, PAPER, 700, 'middle')
    a += text(1007, 172, '目的に対して', 15, PAPER, anchor='middle')
    a += band(254, '抽象度を上げ下げする、では何をするかが決まらない。この3つなら書ける')
    return svg('言葉・範囲・条件を決める作業が、目的レベルを合わせる作業だった', a, 322)


def t5_three_and_height2():
    """3つを決める作業が、目的レベルを合わせる作業だった。"""
    a = text(0, 24, 'ここまでの3本の動画で、やってきたこと', 18, DIM)
    for i, (name, did, effect) in enumerate([
            ('意味', '何を並べるか', '並ぶものの種類が安定する'),
            ('範囲', 'どこから抽出するか', '抽出する量が安定する'),
            ('条件', '入れるもの・落とすもの・並べる順', '各項目の内容が安定する')]):
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
    a += text(1007, 146, '抽象度が合う', 20, PAPER, 700, 'middle')
    a += text(1007, 172, '目的に対して', 15, PAPER, anchor='middle')
    a += band(254, '抽象度を上げ下げする、では何をするかが決まらない。この3つなら書ける')
    return svg('言葉・範囲・条件を決める作業が、目的レベルを合わせる作業だった', a, 322)


def t6_one_at_a_time():
    """1文だけ変えて、3回で確かめる。"""
    a = text(0, 24, '直し方', 18, DIM)
    a += rect(0, 44, 268, 92, ACCENT, 12, ACCENT)
    a += text(134, 84, '1か所だけ直す', 20, PAPER, 700, 'middle')
    a += text(134, 110, '2つ同時に直さない', 15, PAPER, anchor='middle')
    a += arrow(278, 90, 318, 90)
    a += rect(328, 44, 268, 92, PAPER, 12, LINE)
    a += text(462, 84, '同じものを入力して3回', 20, INK, 700, 'middle')
    a += text(462, 110, '1回では分からない', 15, DIM, anchor='middle')
    a += arrow(606, 90, 646, 90)
    a += rect(656, 44, 456, 92, PANEL, 12)
    a += check(692, 78)
    a += text(716, 85, '安定した　　効いている', 18, INK, 700)
    a += cross(692, 114, DIM, .7)
    a += text(716, 121, '安定しない　その文を戻して、別の場所を見る', 17, DIM)
    a += band(176, '1回出力だけでは、ばらついているかどうかを見分けられない')
    return svg('1文だけ変え、同じものを入力して3回試して安定し方を見る', a, 252)


def t6_symptoms():
    """3つを足しても残る、3つの症状。"""
    a = ''
    # 並びは、次の枚の 意味 ・ 範囲 ・ 条件 の順と1対1にする
    items = [('種類が変わる', '課題管理表に、', '相談したいだけの項目が混じった'),
             ('量が変わる', '前の週には出てきた項目が、', '今週は出てこない'),
             ('書き方が安定しない', '期限が「今月中」と', '「9月30日」で混ざった')]
    for i, (name, l1, l2) in enumerate(items):
        x = i * 384
        a += rect(x, 16, 344, 148, PAPER, 12, LINE)
        a += text(x + 24, 60, name, 21, ACCENT, 700)
        a += path(f'M{x + 24} 78 H{x + 320}', LINE, 1)
        a += text(x + 24, 114, [l1, l2], 17, INK, gap=28)
    a += band(186, 'プロンプトの全体を書き直すと、効いていた文まで一緒に消える')
    return svg('3つを足しても残る、3つの症状', a, 254)


def t6_two_goals():
    """目的が2つ入ったプロンプトは、逆向きの作業へ引かれる。"""
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
    a += band(278, '目的が2つなら、プロンプトも2つに分ける')
    a += text(0, 370, '見分け方　何のためにするのかを1文で書く。「と」「そして」でつながれば、目的は2つ', 16, DIM)
    return svg('目的が2つ入ったプロンプトは、逆向きの作業へ引かれる', a, 390)


def t6_where():
    """症状で、見る場所が決まる。"""
    a = text(0, 24, '症状', 18, DIM)
    a += text(596, 24, '見る場所', 18, ACCENT, 700)
    rows = [('出力される形が違う', '意味', '意味が2つ以上ある語が残っていないか'),
            ('抽出する量が違う', '範囲', '入力するもののどこまでを見るかが書いてあるか'),
            ('中身が欠ける・安定しない', '条件', '必ず入れるものと、その書き方を決めたか')]
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
        steps = [('教材1', '目的の把握', '目的を、AIに頼める大きさで定める', 'search', 49),
                 ('教材2', '言葉の定義', '言葉の意味を、チームで統一する', 'book', 54),
                 ('教材3', '仕組みの構築', 'Skillに書き、エージェントが使う', 'robot', 60)]
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
    names = ['原因を知る', '意味を決める', '範囲を決める', '条件を決める', '出力のばらつきを直す', '目的レベルを合わせる']
    a = ''
    for i, name in enumerate(names):
        x = i * 188
        on = (i + 2) == no
        a += rect(x, 0, 172, 44, ACCENT if on else PANEL, 8)
        a += text(x + 86, 28, name, 14, PAPER if on else DIM, 700 if on else 400, 'middle')
        if i < 5:
            a += path(f'M{x + 176} 22 H{x + 184}', LINE, 1.5)
    return svg('6本の中での位置', a, 48)

