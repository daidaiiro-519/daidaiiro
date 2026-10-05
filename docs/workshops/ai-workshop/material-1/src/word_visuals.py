"""2本目（意味を決める）の図。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)
from intro_visuals import _robot_at

# 業務 ／ 何のために整理するか ／ 整理の仕方 ／ 例 ／ 何回目の返りか
KINDS = [
    ('経費の精算業務', '日々の精算が回るように', '業務の流れの順に並べ、影響を受ける担当を添える',
     '月次の締めと重なり、確認の時間が取れない', '1回目に出力された'),
    ('案件の進行管理', '案件の進行が止まらないように', '決定の期限の順に並べ、誰が決めるかを添える',
     '承認の経路が決まらず、設計が止まる', '2回目に出力された'),
    ('システムの開発', '実装が進むように', '機能ごとに並べ、技術的な依存関係を添える',
     '試験環境が1つで、並行して試験できない', '3回目に出力された'),
]


def _sheet(x, y, w, h, stroke=LINE):
    """角を折った書類の形。"""
    d = f'M{x} {y} H{x + w - 22} L{x + w} {y + 22} V{y + h} H{x} Z M{x + w - 22} {y} V{y + 22} H{x + w}'
    return path(d, stroke, 2, PAPER)


def t2_three_returns():
    """同じ議事録と、作業の目的が書かれていないプロンプトで3回実行すると、表の作りが毎回違い、抽出する課題も変わる。出力された課題管理表を、書類の形で3枚並べる。"""
    # 上：頼んだこと。議事録 → プロンプト → 3回
    a = rect(0, 0, 1112, 60, PAPER, 12, LINE)
    a += icon(250, 10, 'doc', DIM, .62) + text(288, 37, '今週の定例の議事録', 16, INK, 700)
    a += arrow(450, 30, 492, 30)
    a += rect(500, 10, 210, 40, PAPER, 20, ACCENT) + text(605, 36, '「課題を整理して」', 16, INK, 700, 'middle')
    a += arrow(710, 30, 752, 30)
    a += _robot_at(770, 8, INK) + text(836, 37, 'AIで3回実行する', 16, INK, 700)
    tables = [('精算の担当者が使う表', '業務の流れの順に並ぶ', '影響を受ける担当',
               [('申請の締め日', '申請者'), ('承認の経路', '承認者'), ('差し戻しの扱い', '申請者 ・ 承認者'),
                ('既存データの移行', '経理'), ('旧システムの停止日', '全員'), ('月次の締めの進め方', '経理')]),
              ('PMが使う表', '決定の期限の順に並ぶ', '誰が決めるか',
               [('承認の経路', '経理と開発'), ('申請の締め日', '経理'), ('差し戻しの扱い', '経理'),
                ('旧システムの停止日', 'PM'), ('既存データの移行', '開発')]),
              ('開発者が使う表', '機能ごとに並ぶ', '依存する機能',
               [('承認の経路', '権限の設定'), ('差し戻しの扱い', '承認の経路'), ('申請の締め日', '締めの処理'),
                ('既存データの移行', '旧システム'), ('旧システムの停止日', 'データの移行'), ('試験環境の準備', 'すべての機能')])]
    for i, (who, how, col, rows) in enumerate(tables):
        x, y = i * 380, 70
        a += _sheet(x, y, 352, 270)
        a += text(x + 18, y + 28, f'課題管理表　{i + 1}回目', 13, DIM, 700)
        a += text(x + 18, y + 54, who, 17, INK, 700) + text(x + 18, y + 76, how, 12, DIM)
        a += path(f'M{x + 18} {y + 92} H{x + 334}', LINE, 1.5)
        a += text(x + 18, y + 112, '課題', 12, DIM, 700) + text(x + 190, y + 112, col, 12, ACCENT, 700)
        a += path(f'M{x + 18} {y + 122} H{x + 334}', LINE, 1)
        for j, (item, val) in enumerate(rows):
            ry = y + 144 + j * 22
            extra = j == 5
            a += text(x + 18, ry, ('＋ ' if extra else '') + item, 13, ACCENT if extra else INK, 700 if extra else 400)
            a += text(x + 190, ry, val, 13, INK)
    return svg('同じ議事録と同じプロンプトで3回実行すると、精算の担当者が使う表、PMが使う表、開発者が使う表と、表の作りが毎回違い、抽出する課題も変わる', a, 346)


def t2_sorting():
    """3つの表は、それぞれ別の業務の目的に合っている。使う人 → その業務の目的 → 出力された表、を人ごとに縦に並べる。PMが欲しい表だけ強調する。"""
    people = [('精算の担当者', '経費の精算業務', '日々の精算が回るように', '1回目の表', '業務の流れの順', False),
              ('PM', '案件の進行管理', '案件の進行が止まらないように', '2回目の表', '決定の期限の順', True),
              ('開発者', 'システムの開発', '実装が進むように', '3回目の表', '機能ごと', False)]
    a = ''
    for i, (who, biz, goal, tbl, how, hot) in enumerate(people):
        cx = 185 + i * 371
        col = ACCENT if hot else LINE
        a += rect(cx - 170, 0, 340, 356, PAPER, 14, col)
        a += circle(cx, 52, 34, PAPER, col) + person(cx, 58, .58, ACCENT if hot else INK)
        a += text(cx, 112, who, 18, INK, 700, 'middle') + text(cx, 134, biz, 14, DIM, anchor='middle')
        # 吹き出し：その人の業務の目的
        a += rect(cx - 150, 156, 300, 56, PAPER, 28, col) + path(f'M{cx - 10} 157 L{cx} 146 L{cx + 10} 157', col, 2, PAPER)
        a += text(cx, 190, goal, 16, ACCENT if hot else INK, 700, 'middle')
        a += path(f'M{cx} 214 V242', col, 2) + path(f'M{cx - 7} 235 l7 8 7 -8', col, 2)
        a += _sheet(cx - 110, 248, 220, 66, col)
        a += text(cx - 90, 276, tbl, 16, INK, 700) + text(cx - 90, 300, how, 14, DIM)
    a += rect(486, 322, 140, 26, ACCENT, 13, ACCENT) + text(556, 340, 'PMが欲しい表', 13, PAPER, 700, 'middle')
    return svg('3回出力された表は、精算の担当者、PM、開発者の、それぞれの業務の目的に合っている。PMが欲しいのは、案件の進行管理の表', a, 360)


def t2_ambiguous():
    """PMの頭の中には作業の目的があったが、当たり前なのでプロンプトに書かなかった。左から右へ、頭の中 → 書いたプロンプト → AIに届くもの、と読む。"""
    # 左：PMと、頭の中の吹き出し
    a = rect(0, 0, 440, 230, PAPER, 36, LINE)
    a += text(32, 42, 'PMの頭の中', 14, DIM, 700)
    for i, (k, v) in enumerate([('やりたいこと', '優先順位を付けたい'), ('PMの業務', '案件の進行管理'), ('整理といえば', '期限の順に並べる')]):
        y = 86 + i * 40
        a += text(32, y, k, 14, DIM) + text(144, y, v, 17, INK, 700)
    a += text(32, 206, '当たり前なので、書こうと思わなかった', 15, ACCENT, 700)
    a += circle(120, 246, 8, PAPER, LINE) + circle(100, 266, 5, PAPER, LINE)
    a += rect(0, 276, 200, 60, PAPER, 30, LINE)
    a += circle(36, 306, 18, PAPER, LINE) + person(36, 309, .34, INK) + text(66, 312, 'PM', 17, INK, 700)
    a += arrow(440, 116, 484, 116)
    # 真ん中：実際に書いたプロンプト。書いていないところは破線の空欄
    a += rect(492, 56, 320, 96, PAPER, 12, LINE)
    a += text(512, 84, '書いたプロンプト', 14, DIM, 700) + text(512, 128, '「課題を整理して」', 24, INK, 700)
    box = rect(492, 168, 320, 52, PAPER, 10, LINE)
    a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 5"')
    a += cross(514, 194, DIM, .55) + text(536, 200, '何のための整理かが、書かれていない', 15, DIM)
    a += arrow(812, 116, 856, 116)
    # 右：AIに届くのは、書いたプロンプトだけ
    a += rect(864, 40, 248, 152, PAPER, 14, ACCENT)
    a += _robot_at(964, 62, INK)
    a += text(988, 142, 'AIに届くのは、', 16, ACCENT, 700, 'middle') + text(988, 168, 'このプロンプトだけ', 16, ACCENT, 700, 'middle')
    return svg('PMの頭の中には作業の目的があったが、当たり前なのでプロンプトに書かなかった。AIに届くのは、書いたプロンプトだけ', a, 340)


def t2_pick():
    """意味は、作業の目的から書く。業務では分けない。1枚の表の上の行に意味の1文、下に2つの業務から出た話を、進行が止まるかで振り分けた結果を置く。"""
    a = rect(0, 0, 1112, 330, PAPER, 12, ACCENT)
    a += text(24, 30, 'プロンプトに追記する意味', 13, DIM, 700)
    a += text(24, 64, '課題とは、次の打ち合わせで決定しないと、案件の進行が止まるものです。', 21, INK, 700)
    a += path('M16 88 H1096', ACCENT, 1.5) + path('M556 100 V318', LINE, 1)
    groups = [('経費の精算業務から出た話', [('承認の経路', True), ('差し戻しの扱い', True), ('申請の締め日', True), ('月次の締めの進め方', False)]),
              ('システムの開発から出た話', [('既存データの移行', True), ('旧システムの停止日', True), ('試験環境の準備', False)])]
    for i, (head, rows) in enumerate(groups):
        x = i * 556
        a += text(x + 24, 124, head, 16, INK, 700)
        for j, (item, keep) in enumerate(rows):
            y = 164 + j * 40
            a += (check(x + 34, y - 6, ACCENT, .5) if keep else cross(x + 34, y - 6, DIM, .6))
            a += text(x + 58, y, item, 16, INK if keep else DIM, 700 if keep else 400)
            a += text(x + 532, y, '進行が止まる' if keep else 'その業務の中で解決できる', 14, ACCENT if keep else DIM, 700 if keep else 400, 'end')
    return svg('意味は作業の目的から書く。業務では分けず、どの業務から出た話でも、決定しないと案件の進行が止まるなら課題に含める', a, 334)


def t2_after():
    """意味を追記する前と後の3回ずつを、課題 × 回の表で並べる。●はその回に並んだ課題。前は月次の締めと試験環境が混じる回があり、後はどの回にも5件だけが並ぶ。"""
    items = ['承認の経路', '差し戻しの扱い', '申請の締め日', '旧システムの停止日', '既存データの移行',
             '月次の締めの進め方', '試験環境の準備']
    before = [{0, 1, 2, 3, 4, 5}, {0, 1, 2, 3, 4}, {0, 1, 2, 3, 4, 6}]
    BX, AX = [400, 490, 580], [740, 830, 920]
    a = rect(0, 0, 1112, 324, PAPER, 12, LINE)
    a += text(490, 32, '追記する前', 15, INK, 700, 'middle') + text(830, 32, '追記したあと', 15, ACCENT, 700, 'middle')
    a += rect(620, 14, 80, 26, ACCENT, 13, ACCENT) + text(660, 32, '＋意味', 13, PAPER, 700, 'middle')
    for xs in (BX, AX):
        for k, x in enumerate(xs):
            a += text(x, 62, f'{k + 1}回目', 12, DIM, anchor='middle')
    a += text(24, 62, '課題', 12, DIM, 700)
    a += path('M24 74 H1088', LINE, 1) + path('M660 48 V226', LINE, 1)
    # 違いが出る2行を、強調色の枠で囲む
    a += rect(14, 234, 1084, 74, PAPER, 10, ACCENT)
    for i, t in enumerate(items):
        y = 96 + i * 30 + (10 if i >= 5 else 0)
        extra = i >= 5
        a += text(24, y + 5, t, 15 if extra else 14, ACCENT if extra else INK, 700 if extra else 400)
        for k, x in enumerate(BX):
            if i in before[k]:
                a += circle(x, y, 10, ACCENT, ACCENT) if extra else circle(x, y, 5, DIM, DIM)
        for x in AX:
            if extra:
                a += circle(x, y, 10, PAPER, LINE).replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="3 3"')
            else:
                a += circle(x, y, 5, DIM, DIM)
        if extra:
            a += text(1084, y + 5, '外れた', 15, ACCENT, 700, 'end')
    a += rect(0, 340, 1112, 46, PAPER, 10, LINE)
    a += text(24, 369, '件数は 4件 ・ 6件 ・ 5件。2件を1行にまとめた回と、1件を2行に分けた回があるだけで、並ぶ課題は同じ', 14, INK)
    return svg('意味を追記する前は、1回目に月次の締め、3回目に試験環境の話が混じっていた。追記したあとは、3回とも同じ5件だけが並ぶ', a, 390)


def t6_domains():
    """6本目の締めに足す1枚 ── 整理の目的は、業務ごとに違う。"""
    a = text(0, 24, '同じプロンプトを、3つの業務がそれぞれ受け取ったとする', 18, DIM)
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
