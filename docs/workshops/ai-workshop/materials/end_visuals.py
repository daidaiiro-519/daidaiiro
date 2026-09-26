"""終わりに（6本の締め）の図。"""
from lesson_visuals import (text, rect, path, circle, arrow, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, check, band)

ADDED = [('意味', '決めないと案件の進行が止まるもの。運用と実装の中で解決できるものは含めない'),
         ('範囲', '今回の案件で、本番の切り替えまでに決めるもの。次のフェーズと別案件は含めない'),
         ('条件', 'どの項目にも、誰が決めるかといつまでかを入れる。決定の期限の早い順に並べる')]


def e_recap():
    """もとの依頼と、3文を足した依頼。"""
    a = text(0, 22, 'もとの依頼', 17, DIM)
    a += rect(0, 34, W, 60, PANEL, 12)
    a += text(556, 71, '「課題を整理して」', 21, INK, 700, 'middle')
    a += path('M556 94 V118 M550 112 l6 6 6 -6', ACCENT, 2)
    a += text(584, 112, '3文を足した', 15, ACCENT, 700)
    for i, (name, body) in enumerate(ADDED):
        y = 126 + i * 74
        a += rect(0, y, W, 62, PAPER, 12, ACCENT)
        a += rect(20, y + 14, 74, 34, ACCENT, 8, ACCENT)
        a += text(57, y + 37, name, 17, PAPER, 700, 'middle')
        a += text(114, y + 38, body, 17, INK)
    a += band(370, '足したのは3文だけ。見出しも、返ってくる形も指定していない')
    return svg('もとの依頼に、意味・範囲・条件の3文を足した', a, 438)


def e_gained():
    """最初の動画で言ったことと、できるようになったこと。"""
    a = text(0, 22, '最初の動画で言ったこと', 17, DIM)
    a += rect(0, 38, 536, 190, PANEL, 12)
    a += text(28, 86, '抽象と具体という見方', 25, INK, 700)
    a += path('M28 106 H508', LINE, 1)
    a += text(28, 140, ['広い言い方と狭い言い方のこと。', 'どちらがよい、という話ではない'], 16, DIM, gap=26)
    a += text(28, 204, 'ちょうどいい高さを見極める', 20, INK, 700)
    a += text(576, 22, 'できるようになったこと', 17, ACCENT, 700)
    a += rect(576, 38, 536, 190, PAPER, 12, ACCENT)
    for i, s in enumerate(['同じ依頼を、次の週も別の案件でも使える',
                           '渡すものが替わっても、目的から外れない',
                           'AIから返ってくるものに、再現性が出る']):
        y = 92 + i * 52
        a += check(608, y - 6, ACCENT, .8)
        a += text(638, y, s, 17, INK)
    a += rect(0, 252, W, 76, PANEL, 12)
    a += rect(0, 252, 5, 76, ACCENT, 2, ACCENT)
    a += text(32, 288, '広いか狭いかを見分けられると、その依頼が仕組みとして残せるかを判定できる', 18, INK)
    a += text(32, 314, '1回ごとの出来を運に任せず、目的から外れない範囲を自分で決める', 16, ACCENT, 700)
    return svg('最初の動画で言ったことが、ここで回収される', a, 338)


NEXT = [('経費の精算業務', '日々の精算が止まる要因'),
        ('案件の進行管理', '決めないと進行が止まるもの'),
        ('システムの開発', '実装が進まない要因')]


def e_next():
    """1つの業務の中から、業務をまたぐ側へ出る。"""
    a = text(0, 22, 'この6本は、1つの業務の中で閉じていた', 17, DIM)
    a += rect(0, 38, W, 72, PANEL, 12)
    a += rect(300, 50, 512, 48, PAPER, 10, ACCENT)
    a += text(556, 80, '案件の進行管理　の中の　課題を整理する', 18, INK, 700, 'middle')
    a += path('M556 110 V132 M550 126 l6 6 6 -6', ACCENT, 2)
    a += text(584, 128, 'ここから外へ出る', 15, ACCENT, 700)
    for i, (name, mean) in enumerate(NEXT):
        x = i * 377
        a += rect(x, 142, 358, 110, PAPER, 12, LINE)
        a += text(x + 24, 180, name, 19, INK, 700)
        a += path(f'M{x + 24} 196 H{x + 334}', LINE, 1)
        a += text(x + 24, 222, '「課題」とは', 13, ACCENT, 700)
        a += text(x + 24, 244, mean, 15, INK)
    a += band(272, 'どれも間違っていない。その業務の中では正しい意味である')
    a += text(0, 358, '次の教材では、業務の切れ目をどこに引き、その中で言葉をどう揃えるかを扱います', 17, INK, 700)
    return svg('1つの業務の中から、業務をまたぐ側へ出る', a, 374)
