"""教材2（ドメイン駆動設計編）の図。教材1と同じ部品・同じ配色を使う。"""
import sys, pathlib
# 部品は proposal/src/visuals.py と、教材1の図（material-1/src/）から借りる
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / 'proposal' / 'src'))
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1] / 'material-1' / 'src'))
from visuals import (text, rect, path, arrow, circle, person, icon, svg,
                     INK, DIM, LINE, PANEL, PAPER, ACCENT)
from lesson_visuals import cross, check, band

W = 1112


def d0_recap():
    """教材1で足した3文は、含めるものと含めないものを両方書いている。"""
    rows = [('意味', '課題とは、次の打ち合わせで決める必要があるもの',
             'すでに決まったものと、起きるかどうか分からないものは含めない'),
            ('範囲', '見るのは、今回の案件の定例、直近5回だけ',
             '合同会議と、前の案件の議事録は含めない'),
            ('条件', 'どの項目にも、誰が決めるかと、いつまでかを必ず入れる',
             'すでに決まったものは入れない')]
    a = text(0, 24, '教材1で足した3文　──　どれも、含めるものと含めないものを両方書いている', 18, DIM)
    a += rect(0, 48, 268, 58, PAPER, 10, LINE)
    a += text(134, 84, '「課題を整理して」', 20, INK, 700, 'middle')
    TOP = 124
    for i, (name, inc, exc) in enumerate(rows):
        y = TOP + i * 74
        a += rect(0, y, 86, 62, ACCENT, 8, ACCENT)
        a += text(43, y + 38, name, 18, PAPER, 700, 'middle')
        a += rect(98, y, 466, 62, PAPER, 8, LINE)
        a += text(118, y + 26, inc, 16, INK, 700)
        a += cross(128, y + 46, DIM, .5)
        a += text(144, y + 51, exc, 13, DIM)
    BOT = TOP + 2 * 74 + 62
    mid = (TOP + BOT) / 2
    a += arrow(580, mid, 620, mid, ACCENT)
    a += rect(634, TOP, 478, BOT - TOP, PANEL, 12)
    a += text(873, mid - 26, '別の議事録を渡しても、目的から外れない', 19, INK, 700, 'middle')
    a += text(873, mid + 10, '＝　再現性', 19, ACCENT, 700, 'middle')
    a += text(873, mid + 38, '教材1の締めである', 15, DIM, anchor='middle')
    a += band(BOT + 22, 'ただし、ここまでは依頼1つの中の話である ── 決めたのは、出した人の頭の中だけ')
    return svg('教材1で足した3文は、含めるものと含めないものを両方書いている', a, BOT + 90)


def d0_translation():
    """流れの中では通じるが、流れの外にいる相手には通じない。"""
    steps = [('定例で話す', '業務部門と開発'), ('議事録に書く', 'PM'),
             ('決めることを\n一覧にする', 'PM'), ('打ち合わせで決める', '関係者')]
    GAP, n = 28, 4
    w = (620 - (n - 1) * GAP) / n
    a = text(0, 22, '案件の進行管理　──　同じ会話の中にいる人たち', 18, DIM)
    for i, (name, who) in enumerate(steps):
        x = i * (w + GAP)
        hot = (i == 2)
        a += rect(x, 44, w, 88, ACCENT if hot else PAPER, 10, ACCENT if hot else LINE)
        lines = name.split('\n')
        a += text(x + w / 2, 72 if len(lines) == 1 else 64, lines, 14,
                  PAPER if hot else INK, 700, 'middle', gap=20)
        a += text(x + w / 2, 114, who, 13, PAPER if hot else DIM, anchor='middle')
        if i < n - 1:
            a += arrow(x + w + 4, 88, x + w + GAP - 4, 88)
    a += rect(0, 148, 620, 44, PANEL, 8)
    a += text(310, 176, '「課題」は、ここでは通じる ── 前後の事情を全員が知っているから', 16, INK, anchor='middle')
    a += text(0, 222, '教材1でやったのは、3つめの作業である', 16, ACCENT, 700)

    a += path('M660 20 V240', LINE, 1)

    a += text(700, 22, '流れの外にいる相手', 18, ACCENT, 700)
    for i, (who, why) in enumerate([('別の業務の人', '受注や請求では、課題の指すものが違う'),
                                    ('新しく入った人', 'その場の事情を知らない'),
                                    ('AI', '渡した言葉の中にあるものだけで決める')]):
        y = 48 + i * 64
        a += rect(700, y, 412, 52, PAPER, 10, LINE)
        a += text(724, y + 32, who, 17, INK, 700)
        a += text(856, y + 32, why, 14, DIM)
    a += rect(700, 240, 412, 0, PAPER, 0, 'none')
    a += text(700, 256, '頭の中で共有していたことが、ここでは伝わらない', 16, DIM)

    a += band(288, '業務の側で言葉を決めるのは、流れの外の相手にも通じるようにするためである')
    return svg('同じ流れの中では課題は通じるが、流れの外にいる相手には通じない', a, 356)


def d0_volume():
    """量が少ないうちは気づけるが、増えると気づけない。"""
    a = text(0, 24, '1つの語に2つの意味が入ったまま出力される', 18, DIM)

    a += rect(0, 48, 528, 196, PAPER, 12, LINE)
    a += text(28, 88, '人が書く量', 20, INK, 700)
    a += text(150, 88, '10件', 24, INK, 700)
    for i in range(10):
        a += rect(28 + (i % 10) * 48, 112, 36, 24, PANEL, 4)
    a += check(36, 186)
    a += text(64, 193, '読み返すと、揺れに気づける', 17, INK)

    a += rect(584, 48, 528, 196, PAPER, 12, ACCENT)
    a += text(612, 88, 'AIが書く量', 20, INK, 700)
    a += text(742, 88, '200件', 24, ACCENT, 700)
    for i in range(40):
        a += rect(612 + (i % 10) * 48, 112 + (i // 10) * 14, 36, 10, PANEL, 3)
    a += cross(620, 186, DIM, .8)
    a += text(648, 193, '読み返せない。気づかないまま先へ進む', 17, INK)

    a += band(270, '先に語を決めておくほうが安く済む ── ただし、これは原則ではなく、この教材の考察である')
    return svg('量が少ないうちは揺れに気づけるが、増えると気づけない', a, 338)


def d0_words():
    """3語を、定義できる順に並べる。"""
    a = text(0, 24, '本題の前に、3つだけ決めておく', 18, DIM)
    items = [('ユースケース', '作業1つ', '誰が ・ 何を使って ・ 何のために'),
             ('業務領域', 'ユースケースの集まり', '同じ人が、同じデータを扱う作業の集まり'),
             ('業務エキスパート', 'その領域に詳しい人', '使う言葉は、この人たちから採る')]
    for i, (name, short, body) in enumerate(items):
        y = 48 + i * 86
        a += rect(0, y, 300, 68, ACCENT, 10, ACCENT)
        a += text(150, y + 32, name, 20, PAPER, 700, 'middle')
        a += text(150, y + 56, short, 15, PAPER, anchor='middle')
        a += arrow(310, y + 34, 346, y + 34)
        a += rect(358, y, 754, 68, PAPER, 10, LINE)
        a += text(384, y + 41, body, 18, INK)
        if i < 2:
            a += path(f'M150 {y + 68} V{y + 86}', LINE, 2)
            a += path(f'M144 {y + 80} l6 6 6 -6', LINE, 2)
    a += band(316, '作業が決まらないと集まりを言えず、集まりが決まらないと、誰が詳しい人かも決まらない')
    return svg('ユースケース ・ 業務領域 ・ 業務エキスパートを、定義できる順に並べる', a, 384)


def d0_order():
    """教材1の語と、ドメイン駆動設計の用語の対応。"""
    a = text(0, 24, '教材1の語から、名前を与える', 18, DIM)
    rows = [('意味', 'ユビキタス言語', '同じ言葉', '業務全体で、語の意味を1つに決める'),
            ('範囲', '境界づけられたコンテキスト', '区切られた文脈', 'その意味が通用する範囲を決める'),
            ('抽象の高さ', 'モデル', '目的のための模型', '目的ごとに、別の模型を持つ'),
            ('条件', '集約と不変条件', 'ドメインモデルの部品', '常に成り立つ約束を決める')]
    for i, (old, new, gloss, body) in enumerate(rows):
        y = 48 + i * 70
        a += rect(0, y, 190, 56, PANEL, 10)
        a += text(95, y + 34, f'教材1「{old}」', 17, INK, 700, 'middle')
        a += arrow(200, y + 28, 236, y + 28, ACCENT)
        a += rect(248, y, 340, 56, ACCENT, 10, ACCENT)
        a += text(418, y + 26, new, 18, PAPER, 700, 'middle')
        a += text(418, y + 47, gloss, 14, PAPER, anchor='middle')
        a += text(614, y + 34, body, 17, DIM)
        a += text(1112, y + 34, f'{i + 1}本目', 15, DIM, anchor='end')
    a += band(340, '名前が変わるだけで、決めることは教材1と同じである')
    return svg('教材1の4語と、ドメイン駆動設計編で与える用語の対応', a, 408)


def d0_roles():
    """同じ定例に出ていても、役割ごとに「課題」の指すものが違う。"""
    a = text(0, 22, '同じ定例で、3人が「課題」と言う', 18, DIM)
    who = [('業務部門', '困っていること', '月末の締めが間に合わない'),
           ('開発', '技術的に決まっていないこと', '通知の方式が決まらない'),
           ('PM', '決めないと進まないこと', '誰がいつまでに決めるか')]
    for i, (name, mean, ex) in enumerate(who):
        x = i * 384
        a += person(x + 40, 76, .8, DIM)
        a += text(x + 78, 66, name, 19, INK, 700)
        a += rect(x, 104, 344, 88, PAPER, 10, LINE)
        a += text(x + 24, 136, '「課題」＝', 15, DIM)
        a += text(x + 104, 136, mean, 16, ACCENT, 700)
        a += text(x + 24, 168, '例　' + ex, 15, DIM)
    a += rect(0, 216, 1112, 56, PANEL, 12)
    a += text(556, 250, '議事録には、3つとも「課題」と書かれる', 20, INK, 700, 'middle')
    a += text(0, 304, '教材1で3回頼んで3通り返ってきたのは、この3つが同じ語で混ざっていたからである', 17, ACCENT, 700)
    a += band(328, '同じ会議に出ていても、頭にあるものは違う ── だから、業務の側で1つに決める')
    return svg('同じ定例でも、業務部門 ・ 開発 ・ PMで「課題」の指すものが違う', a, 396)


def d2_handoff():
    """扱いは同じでも、何について決めるかが業務ごとに違う。"""
    a = text(0, 22, '3つとも、扱いは同じ ── 次の打ち合わせで決定する未決事項である', 18, DIM)
    who = [('受注管理', '何について決めるか　商談の条件', '値引きをどこまで認めるか'),
           ('案件の進行管理', '何について決めるか　仕様と進行', '通知をメールにするか、画面だけにするか'),
           ('問い合わせ対応', '何について決めるか　個別の不具合', '恒久の対応をいつ入れるか')]
    for i, (name, obj, ex) in enumerate(who):
        x = i * 384
        a += rect(x, 44, 344, 128, PAPER, 12, LINE)
        a += text(x + 24, 80, name, 18, INK, 700)
        a += text(x + 24, 110, obj, 15, ACCENT, 700)
        a += text(x + 24, 140, '例', 14, DIM)
        a += text(x + 48, 140, ex, 14, DIM)
        if i < 2:
            a += cross(x + 364, 108, DIM, .8)
    a += text(556, 206, '同じ「課題」という語で、別の対象を指している', 17, DIM, anchor='middle')
    a += rect(0, 226, 1112, 62, PANEL, 12)
    a += text(556, 264, '営業が顧客へ「課題はありません」と報告する裏で、保守に未解決の不具合が残存する', 19, INK, 700, 'middle')
    a += text(0, 324, 'どの業務も、自分の中では正しく通用している ── 相違それ自体は、是正すべき誤りではない', 17, DIM)
    a += band(348, 'つなぎ方を決める必要がある ── その型は、この教材では扱わない')
    return svg('扱いは同じでも、何について決めるかが業務ごとに相違する', a, 416)
