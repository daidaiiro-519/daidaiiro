"""2本目（意味を決める）の図。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)

# 業務 ／ 何のために整理するか ／ 整理の仕方 ／ 例 ／ 何回目の返りか
KINDS = [
    ('経費の精算業務', '日々の精算が回るように', '業務の流れの順に並べ、影響を受ける担当を添える',
     '月次の締めと重なり、確認の時間が取れない', '1回目に返ってきた'),
    ('案件の進行管理', '案件の進行が止まらないように', '決定の期限の順に並べ、誰が決めるかを添える',
     '承認の経路が決まらず、設計が止まる', '2回目に返ってきた'),
    ('システムの開発', '実装が進むように', '機能ごとに並べ、技術の依存を添える',
     '試験環境が1つで、並行して試験できない', '3回目に返ってきた'),
]


def t2_three_returns():
    """1本の議事録と1つの依頼から、3通りの整理の仕方が返る。"""
    a = text(0, 24, '渡したのは議事録1本だけ', 18, DIM)
    a += rect(0, 44, 276, 154, PAPER, 12, LINE)
    a += icon(24, 62, 'doc', DIM, .7)
    a += text(78, 90, '定例の議事録', 19, INK, 700)
    a += text(24, 124, '出席　経理 ・ 開発 ・ PM', 13, ACCENT, 700)
    a += text(24, 152, ['承認の経路／申請の締め日', '旧システムの停止日／月次の締め', '試験環境の話も1件ずつ'], 12, DIM, gap=18)
    a += arrow(286, 121, 320, 121)
    a += rect(330, 78, 198, 86, PANEL, 10)
    a += text(429, 110, '「課題を整理して」', 20, INK, 700, 'middle')
    a += text(429, 138, '依頼を出すのはPM', 14, DIM, anchor='middle')
    for i, (no, order, add) in enumerate([
            ('1回目', '業務の流れの順に並んでいる', '影響を受ける担当が添えてある'),
            ('2回目', '決定の期限の順に並んでいる', '誰が決めるかが添えてある'),
            ('3回目', '機能ごとに並んでいる', '技術の依存が添えてある')]):
        y = 36 + i * 84
        a += path(f'M528 121 H560 V{y + 32} H592', LINE)
        a += path(f'M583 {y + 26} l9 6 -9 6', LINE, 2)
        a += rect(592, y, 520, 64, PAPER, 10, LINE)
        a += text(614, y + 43, no, 16, DIM)
        a += text(668, y + 32, order, 19, INK, 700)
        a += text(668, y + 54, add, 15, DIM)
    a += band(302, '拾った項目はほぼ同じ。違うのは、並べ方と、添えてあるものである')
    return svg('同じ議事録と同じ依頼から、3通りの整理の仕方が返る', a, 370)


def t2_sorting():
    """整理には目的がある。目的が業務ごとに違う。"""
    a = text(0, 24, '整理の目的は、その業務が何を達成しようとしているかで決まる', 18, DIM)
    for i, (name, why, how, ex, who) in enumerate(KINDS):
        x = i * 384
        hot = (i == 1)
        a += rect(x, 44, 344, 216, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 24, 82, name, 19, INK, 700)
        a += text(x + 24, 110, why, 15, ACCENT if hot else DIM, 700 if hot else 400)
        a += path(f'M{x + 24} 128 H{x + 320}', LINE, 1)
        a += text(x + 24, 156, '整理の仕方', 13, DIM)
        a += text(x + 24, 182, how.split('、'), 15, INK, gap=24)
        a += rect(x + 24, 224, 176, 26, PANEL, 8)
        a += text(x + 112, 242, who, 13, DIM, anchor='middle')
    a += band(282, '3つとも整理してある。どれも、その業務では正しい')
    return svg('整理の目的が業務ごとに違うので、整理の仕方も違う', a, 350)


def t2_ambiguous():
    """何のために整理するかを書いていない。"""
    a = text(0, 26, 'AIから見ると、3つとも同じ資格の候補である', 17, DIM)
    a += rect(400, 52, 312, 66, ACCENT, 12, ACCENT)
    a += text(556, 95, '「整理して」', 26, PAPER, 700, 'middle')
    for i, (name, why, how, ex, who) in enumerate(KINDS):
        x = i * 384
        hot = (i == 1)
        a += path(f'M556 118 C556 152 {x + 172} 144 {x + 172} 174', ACCENT)
        a += path(f'M{x + 172 - 6} 168 l6 8 6 -8', ACCENT, 2)
        a += rect(x, 180, 344, 86, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 172, 216, name, 19, INK, 700, 'middle')
        a += text(x + 172, 246, why, 15, DIM, anchor='middle')
        if hot:
            a += text(x + 172, 292, 'PMが意図したのは、これ1つ', 16, ACCENT, 700, 'middle')
        else:
            a += cross(x + 172, 286, DIM, .6)
    a += band(312, '書いていないので、選定するのはAIになる')
    return svg('何のために整理するかを書いていないので、AIが選定する', a, 380)


def t2_pick():
    """1つの整理の仕方を選び、目的とともに書く。"""
    a = ''
    rows = [('経費の精算業務', '日々の精算が回るように', False),
            ('案件の進行管理', '案件の進行が止まらないように', True),
            ('システムの開発', '実装が進むように', False)]
    for i, (name, why, on) in enumerate(rows):
        y = 30 + i * 78
        a += rect(0, y, 560, 64, ACCENT if on else PAPER, 10, ACCENT if on else LINE)
        a += text(24, y + 40, name, 19, PAPER if on else INK, 700)
        a += text(232, y + 40, why, 15, PAPER if on else DIM)
        a += check(524, y + 32, PAPER, .8) if on else cross(524, y + 32, DIM, .7)
    a += rect(632, 30, 480, 220, PANEL, 12)
    a += text(656, 62, '依頼に足す1文', 16, DIM)
    a += text(656, 104, ['課題とは、次の打ち合わせで決定しない', 'と、案件の進行が止まるものです。',
                         '経費の精算業務の中で解決できるものと、',
                         'システムの開発の中で解決できるものは含めません。'], 17, INK, gap=26)
    a += text(656, 222, '目的　次回の議題を組成すること', 15, ACCENT, 700)
    a += band(282, '含めないものも並べて、どちらとも取れるものをAIに任せない')
    return svg('3つの整理の仕方のうち1つを選び、目的の言葉で書き直す', a, 350)


def t2_after():
    """足す前と足したあとを、同じ3回で上下に並べる。"""
    a = text(0, 22, '足す前　依頼は「課題を整理して」だけ ── 並べ方が3通り', 17, DIM)
    for i, before in enumerate(['業務の流れの順に並ぶ', '決定の期限の順に並ぶ', '機能ごとに並ぶ']):
        x = i * 384
        a += rect(x, 38, 344, 48, PAPER, 10, LINE)
        a += text(x + 24, 68, f'{i + 1}回目', 14, DIM)
        a += text(x + 80, 68, before, 17, DIM)
    a += text(0, 122, '足したあと　意味の1文を足した ── 並ぶものは3回とも同じ種類', 17, ACCENT, 700)
    after = [('4件', ['承認の経路', '申請の締め日', 'ほか2件']),
             ('6件', ['承認の経路', '承認の差し戻し先', 'ほか4件']),
             ('5件', ['承認の経路と差し戻し先', '申請の締め日', 'ほか3件'])]
    for i, (n, items) in enumerate(after):
        x = i * 384
        a += rect(x, 138, 344, 152, PAPER, 12, ACCENT)
        a += text(x + 24, 172, f'{i + 1}回目', 15, DIM)
        a += text(x + 320, 174, n, 22, ACCENT, 700, 'end')
        a += path(f'M{x + 24} 188 H{x + 320}', LINE, 1)
        a += text(x + 24, 216, items, 17, INK, gap=28)
    a += band(310, 'どの回も、決めないと進行が止まるものだけが並ぶ')
    return svg('足す前は並べ方が3通り、足したあとは3回とも同じ種類が並ぶ', a, 378)


def t6_domains():
    """6本目の締めに足す1枚 ── 整理の目的は、業務ごとに違う。"""
    a = text(0, 24, '同じ依頼を、3つの業務がそれぞれ受け取ったとする', 18, DIM)
    a += rect(400, 40, 312, 52, PANEL, 10)
    a += text(556, 73, '「課題を整理して」', 20, INK, 700, 'middle')
    for i, (name, why) in enumerate([('経費の精算業務', '精算が回るように整理する'),
                                     ('案件の進行管理', '進行が止まらないように整理する'),
                                     ('システムの開発', '実装が進むように整理する')]):
        x = i * 384
        hot = (i == 1)
        a += path(f'M556 92 V102 H{x + 172} V112', LINE)
        a += path(f'M{x + 172 - 6} 106 l6 8 6 -8', LINE, 2)
        a += rect(x, 118, 344, 96, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 172, 156, name, 20, INK, 700, 'middle')
        a += text(x + 172, 186, why, 15, ACCENT if hot else DIM, anchor='middle')
    a += text(0, 258, '整理の仕方は、業務ごとに違う', 21, ACCENT, 700)
    a += rect(0, 280, W, 78, PANEL, 12)
    a += rect(0, 280, 5, 78, ACCENT, 2, ACCENT)
    a += text(32, 316, '他の業務へ依頼するときは、その業務での整理の目的を先に把握する', 19, INK, 700)
    a += text(32, 342, '何が目的を決定するのかは、次の教材で扱う', 15, DIM)
    return svg('整理の目的は、業務ごとに違う', a, 368)
