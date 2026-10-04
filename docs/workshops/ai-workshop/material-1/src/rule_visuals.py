"""4本目（条件を決める）の試作の図。どの条件が、各項目のどこを作るか。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)

ROWS = [('承認の経路', '経理と開発', '9/30'),
        ('申請の締め日', '経理', '9/30'),
        ('旧システムの停止日', 'PM', '10/7')]


def s4_effect():
    """4つの条件が、各項目のどこを作るか。"""
    a = text(0, 22, 'プロンプトに追記する文', 17, DIM)
    a += rect(0, 38, 470, 206, PAPER, 12, ACCENT)
    a += text(24, 80, ['どの項目にも、誰が決めるかと、', 'いつまでかを必ず入れてください。',
                       'すでに決まったものは含めないでください。',
                       '決定の期限の早い順に並べてください。'], 18, INK, gap=30)
    a += text(24, 220, '条件は4つ', 15, ACCENT, 700)
    a += arrow(486, 140, 526, 140, ACCENT)
    a += text(546, 22, '各項目が、こうなる', 17, ACCENT, 700)
    a += text(600, 62, '項目', 14, DIM)
    a += text(830, 62, '決める担当', 14, ACCENT, 700)
    a += text(1004, 62, 'いつまでに', 14, ACCENT, 700)
    for i, (name, who, when) in enumerate(ROWS):
        y = 74 + i * 50
        a += rect(546, y, 566, 40, PAPER, 8, LINE)
        a += text(566, y + 26, str(i + 1), 14, ACCENT, 700)
        a += text(600, y + 26, name, 17, INK)
        a += text(830, y + 26, who, 16, INK)
        a += text(1004, y + 26, when, 16, INK)
    a += text(546, 240, '上から、期限の早い順 ── これも条件が決める', 16, ACCENT, 700)
    return svg('4つの条件が、各項目の内容と、載せる載せないと、並べる順を作る', a, 256)


def s4_after():
    """条件を足す前と、足したあとを並べる。"""
    a = text(0, 22, '足す前　意味と範囲だけを書いたプロンプト ── 各項目の内容が3通り', 17, DIM)
    lack = [('1回目', '項目名だけ　担当も期限もない'),
            ('2回目', '担当はある　期限がない'),
            ('3回目', '決まったものが混じる')]
    for i, (no, s) in enumerate(lack):
        x = i * 384
        a += rect(x, 38, 344, 48, PAPER, 10, LINE)
        a += text(x + 24, 68, no, 14, DIM)
        a += text(x + 80, 68, s, 15, DIM)
    a += text(0, 122, '追記したあと　条件を追記した ── どの項目にも担当と期限が入り、決まったものはない', 17, ACCENT, 700)
    counts = ['5件', '6件', '5件']
    for i, n in enumerate(counts):
        x = i * 384
        a += rect(x, 138, 344, 152, PAPER, 12, ACCENT)
        a += text(x + 24, 172, f'{i + 1}回目', 15, DIM)
        a += text(x + 320, 174, n, 22, ACCENT, 700, 'end')
        a += path(f'M{x + 24} 188 H{x + 320}', LINE, 1)
        for r, (name, who, when) in enumerate(ROWS[:2]):
            y = 216 + r * 30
            a += text(x + 24, y, name, 15, INK)
            a += text(x + 180, y, who, 12, DIM)
            a += text(x + 290, y, when, 12, DIM)
        a += text(x + 24, 276, 'ほか' + str(int(n[0]) - 2) + '件', 15, DIM)
    a += band(310, 'どの項目にも担当と期限が入り、すでに決まったものは落ちる')
    return svg('条件を足す前は中身が3通り、足したあとはどの項目にも同じものが在る', a, 378)
