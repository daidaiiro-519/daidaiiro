"""はじめに（全教材に共通する前置き）の図。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)


def i0_case():
    """1つの案件に、3つの業務が関わっている。"""
    a = text(0, 24, 'この教材が通して使う案件', 18, DIM)
    a += rect(356, 44, 400, 84, ACCENT, 12, ACCENT)
    a += text(556, 82, '経費精算システムを入れ替える', 21, PAPER, 700, 'middle')
    a += text(556, 110, '本番の切り替えは3か月後', 15, PAPER, anchor='middle')
    roles = [('経費の精算業務', '経理が、日々の精算と支払を回している'),
             ('案件の進行管理', 'PMが、切り替えまで案件を止めずに進める'),
             ('システムの開発', '開発が、画面と連携を作っている')]
    for i, (name, what) in enumerate(roles):
        x = i * 384
        a += path(f'M556 128 V152 H{x + 172} V176', LINE)
        a += path(f'M{x + 172 - 6} 170 l6 8 6 -8', LINE, 2)
        # 教材1が扱う業務を際立たせる。並び順ではなく名前で決める
        a += rect(x, 182, 344, 96, PAPER, 12, ACCENT if name == '案件の進行管理' else LINE)
        a += person(x + 44, 222, .62, DIM)
        a += text(x + 80, 226, name, 19, INK, 700)
        a += text(x + 24, 260, what, 15, DIM)
    a += rect(0, 300, W, 62, PANEL, 12)
    a += text(32, 330, '週に1回、3つの業務の人が集まる定例がある', 19, INK, 700)
    a += text(32, 352, '議事録を書くのはPM', 15, DIM)
    return svg('1つの案件に、3つの業務が関わっている', a, 372)


def i0_usecases():
    """業務は、いくつもの作業でできている。"""
    a = text(0, 24, 'PMが受け持っている業務', 18, ACCENT, 700)
    a += rect(0, 44, 320, 244, PANEL, 12)
    a += text(160, 92, '案件の進行管理', 21, INK, 700, 'middle')
    a += text(160, 126, '本番の切り替えまで', 15, DIM, anchor='middle')
    a += text(160, 150, '案件を止めずに進める', 15, DIM, anchor='middle')
    a += path('M40 178 H280', LINE, 1)
    a += text(160, 212, 'これだけでは', 15, DIM, anchor='middle')
    a += text(160, 240, '何をするかが決まらない', 15, DIM, anchor='middle')
    a += arrow(336, 162, 380, 162, ACCENT)
    ucs = [('課題を整理する', True), ('次回の議題を組む', False),
           ('進捗を報告する', False), ('要員の割り当てを決める', False)]
    for i, (name, hot) in enumerate(ucs):
        y = 44 + i * 62
        a += rect(400, y, 712, 48, ACCENT if hot else PAPER, 10, ACCENT if hot else LINE)
        a += text(424, y + 31, name, 19, PAPER if hot else INK, 700)
        if hot:
            a += text(1088, y + 31, '教材1が扱う', 15, PAPER, anchor='end')
    a += band(310, '精算業務にも開発にも、それぞれの作業が在る')
    return svg('業務の中に、作業が並んでいる', a, 378)


def i0_usecase():
    """作業1つは、4つで書ける。"""
    a = text(0, 24, '教材1が扱う作業', 18, DIM)
    a += rect(0, 44, 536, 250, PAPER, 12, ACCENT)
    a += text(28, 88, '課題を整理する', 24, INK, 700)
    a += path('M28 106 H508', LINE, 1)
    rows = [('誰が', 'PM'), ('何のために', '次回の議題を組成し、誰を招集するかを確定する'),
            ('渡すもの', '定例の議事録'), ('返すもの', '課題の一覧')]
    for i, (k, v) in enumerate(rows):
        y = 142 + i * 42
        a += text(28, y, k, 15, ACCENT, 700)
        a += text(150, y, v, 17, INK)
    a += text(596, 24, 'ここから入る理由', 18, DIM)
    a += rect(596, 44, 516, 250, PANEL, 12)
    for i, (head, body) in enumerate([
            ('誰が実行するかが1つに定まる', '実行する人が変われば、別の作業になる'),
            ('何のためかが1つに定まる', '目的が2つあれば、作業も2つに分かれる'),
            ('渡すものと返すものが決まる', 'だから、返ってきたものが正しいかを判定できる')]):
        y = 90 + i * 74
        a += text(624, y, head, 18, INK, 700)
        a += text(624, y + 26, body, 15, DIM)
    a += band(316, '4つが決まる単位なので、ここから始める')
    return svg('作業1つは、誰が・何のために・何を受け取り・何を返すかで書ける', a, 384)


def i0_journey():
    """3本の並び。"""
    a = text(0, 24, '前の本で決めたことを、次の本が前提にする', 18, DIM)
    items = [('教材1', '課題の把握', '作業1つを、AIに正しく実行させる',
              '意味・範囲・条件の3つを書く'),
             ('教材2', '業務の整理', '業務が違えば、同じ言葉が別のものを指す',
              '言葉を区切りごとに揃える手順'),
             ('教材3', '仕組みの構築', '人が承認できる形に、どう固定するか',
              'Skill・Agent・MCP・Hook・ハーネス')]
    for i, (no, name, q, out) in enumerate(items):
        x = i * 384
        hot = (i == 0)
        a += rect(x, 44, 344, 208, PAPER, 12, ACCENT if hot else LINE)
        a += rect(x, 44, 344, 46, ACCENT if hot else PANEL, 12)
        a += text(x + 24, 74, no, 17, PAPER if hot else DIM, 700)
        a += text(x + 100, 74, name, 19, PAPER if hot else INK, 700)
        a += text(x + 24, 126, '扱う問い', 14, DIM)
        a += text(x + 24, 152, q.split('、'), 16, INK, gap=24)
        a += path(f'M{x + 24} 186 H{x + 320}', LINE, 1)
        a += text(x + 24, 212, '身につくもの', 14, DIM)
        a += text(x + 24, 236, out, 15, ACCENT, 700)
        if i < 2:
            a += path(f'M{x + 348} 148 H{x + 376}', LINE, 2)
            a += path(f'M{x + 367} 142 l9 6 -9 6', LINE, 2)
    a += band(276, 'この3本は、同じ1つの案件を使う')
    return svg('3本で、扱う範囲が1つずつ広がる', a, 344)


def i0_base():
    """3本に共通する土台。"""
    a = text(0, 24, '3つの教材が、同じ土台に載る', 18, DIM)
    for i, name in enumerate(['教材1', '教材2', '教材3']):
        x = 156 + i * 280
        a += rect(x, 44, 244, 60, PAPER, 10, LINE)
        a += text(x + 122, 82, name, 19, INK, 700, 'middle')
        a += path(f'M{x + 122} 104 V124', LINE, 1)
    a += rect(0, 130, W, 116, ACCENT, 12, ACCENT)
    a += text(556, 168, '業務で使う言葉を、通じる区切りごとに揃える', 24, PAPER, 700, 'middle')
    a += text(556, 200, 'そのまま、作るものの形にする', 24, PAPER, 700, 'middle')
    a += text(556, 228, '同じ言葉でも、業務が違えば別のものを指す。どこまで通じるかと、その言葉が守る決まりまで揃える',
              14, PAPER, anchor='middle')
    a += text(0, 278, 'なぜ、いま土台に置くのか', 18, INK, 700)
    a += text(W, 278, 'この考え方には名前がある。教材2で扱う', 15, DIM, anchor='end')
    a += rect(0, 292, W, 76, PANEL, 12)
    a += rect(0, 292, 5, 76, ACCENT, 2, ACCENT)
    a += text(32, 328, '人どうしなら、言葉がずれていても、その場で聞き返して埋められる', 17, INK)
    a += text(32, 354, 'AIは聞き返さずに、そのまま実行する', 17, ACCENT, 700)
    return svg('3本の土台は、業務の言葉を区切りごとに揃え、そのまま作るものの形にすることである', a, 378)


def i0_why():
    """いま起きていることと、この教材の先。"""
    a = text(0, 24, 'いま起きていること', 18, DIM)
    a += rect(0, 44, 520, 190, PAPER, 12, LINE)
    for i, s in enumerate(['AIを使う人と、使わない人に分かれる',
                           '工夫が本人の手元に留まり、共有されない',
                           '次の人が、また同じところから始める']):
        a += cross(32, 88 + i * 52, DIM, .6)
        a += text(62, 94 + i * 52, s, 17, INK)
    a += arrow(536, 139, 580, 139, ACCENT)
    a += text(596, 24, 'この教材のあと', 18, ACCENT, 700)
    a += rect(596, 44, 516, 190, PAPER, 12, ACCENT)
    for i, s in enumerate(['自分の業務の課題を、自分で見極める',
                           'AIで動く仕組みにする',
                           '同じ業務の人が、その仕組みを使う']):
        a += check(628, 88 + i * 52, ACCENT, .8)
        a += text(658, 94 + i * 52, s, 17, INK)
    a += rect(0, 258, W, 88, PANEL, 12)
    a += rect(0, 258, 5, 88, ACCENT, 2, ACCENT)
    a += text(32, 294, '案件が終わると、作った仕組みはその案件に残る', 18, INK)
    a += text(32, 322, '自分に残るのは、課題をAIで解決する力である', 19, ACCENT, 700)
    return svg('いま起きていることと、この教材のあとに変わること', a, 356)


def i0_loop():
    """出来事から、本質と、それを解決するものを取り出す。"""
    a = text(0, 24, '起きている出来事から、順に取り出す', 18, DIM)
    steps = [('出来事', ['定例のたびに、決めること', 'を並べるのに時間がかかる'], False),
             ('課題の本質', ['議事録から、決める必要が', 'あるものだけを取り出せて', 'いない'], True),
             ('解決するもの', ['同じ基準で、毎回同じよう', 'に取り出せる状態'], True),
             ('AIで動く仕組み', ['その基準をAIに渡して、', '毎回実行させる'], True)]
    for i, (name, body, hot) in enumerate(steps):
        x = i * 286
        a += rect(x, 48, 254, 168, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 24, 86, name, 19, ACCENT if hot else INK, 700)
        a += path(f'M{x + 24} 104 H{x + 230}', LINE, 1)
        a += text(x + 24, 134, body, 14, DIM, gap=24)
        if i < 3:
            a += arrow(x + 258, 132, x + 282, 132, LINE if i == 0 else ACCENT)
    a += path('M286 232 V248 H1112 V232', ACCENT, 1)
    a += text(699, 276, 'ここまでが、この教材で練習することである', 19, ACCENT, 700, 'middle')
    return svg('出来事から、課題の本質と、それを解決するものを取り出す', a, 300)


def i1_bridge():
    """前の動画で選んだ作業を、AIに頼む。"""
    a = text(0, 24, '「はじめに」で決めた4つ', 18, DIM)
    a += rect(0, 44, 536, 200, PAPER, 12, ACCENT)
    a += text(28, 86, '課題を整理する', 23, INK, 700)
    a += path('M28 104 H508', LINE, 1)
    for i, (k, v) in enumerate([('誰が', 'PM'), ('何のために', '次回の議題を組成し、誰を招集するかを確定する'),
                                ('渡すもの', '定例の議事録'), ('返すもの', '課題の一覧')]):
        y = 132 + i * 32
        a += text(28, y, k, 14, ACCENT, 700)
        a += text(150, y, v, 16, INK)
    a += arrow(552, 144, 596, 144, ACCENT)
    a += rect(612, 44, 500, 88, PANEL, 12)
    a += text(636, 78, 'AIへの依頼', 15, DIM)
    a += text(862, 112, '「課題を整理して」', 23, INK, 700, 'middle')
    a += path('M862 132 V152 M856 146 l6 6 6 -6', LINE, 2)
    a += rect(612, 156, 500, 88, PAPER, 12, LINE)
    a += text(636, 190, '返ってきたもの', 15, DIM)
    a += cross(650, 216, DIM, .7)
    a += text(678, 222, '思ったとおりではない', 21, INK, 700)
    a += band(268, '渡すものも、返してほしいものも決まっている。それでも、揃わない')
    return svg('前の動画で選んだ作業を、AIに頼む', a, 336)


def i1_goal():
    """この6本で身につけるもの。"""
    a = text(0, 22, 'この6本の動画で身につけるもの', 17, DIM)
    a += rect(0, 38, 536, 190, PAPER, 12, ACCENT)
    a += text(28, 86, '抽象と具体という見方', 25, ACCENT, 700)
    a += path('M28 106 H508', LINE, 1)
    a += text(28, 140, ['広い言い方と狭い言い方のこと。', 'どちらがよい、という話ではない'], 16, DIM, gap=26)
    a += text(28, 204, 'ちょうどいい高さを見極める', 20, INK, 700)
    a += text(576, 22, '最後まで観ると、こうなる', 17, ACCENT, 700)
    a += rect(576, 38, 536, 190, PANEL, 12)
    for i, s in enumerate(['同じ依頼を、次の週も別の案件でも使える',
                           '渡すものが替わっても、目的から外れない',
                           'AIから返ってくるものに、再現性が出る']):
        y = 92 + i * 52
        a += check(608, y - 6, ACCENT, .8)
        a += text(638, y, s, 17, INK)
    a += band(252, '名前より先に、実際に何が起きているかを見ます')
    return svg('この6本で身につけるものと、最後まで観たときに変わること', a, 320)


def m1_task():
    """教材1のはじめに ── 6本が扱うのは、作業1つである。"""
    a = text(0, 24, '案件の進行管理の中の作業', 18, DIM)
    ucs = [('課題を整理する', True), ('次回の議題を組む', False),
           ('進捗を報告する', False), ('要員の割り当てを決める', False)]
    for i, (name, on) in enumerate(ucs):
        y = 48 + i * 58
        a += rect(0, y, 460, 46, ACCENT if on else PAPER, 10, ACCENT if on else LINE)
        a += text(24, y + 30, name, 19, PAPER if on else DIM, 700 if on else 400)
    a += arrow(492, 140, 540, 140, ACCENT)
    a += text(596, 24, 'この6本が扱う作業', 18, ACCENT, 700)
    a += rect(596, 44, 516, 232, PAPER, 12, ACCENT)
    a += text(624, 88, '課題を整理する', 24, INK, 700)
    a += path('M624 106 H1084', LINE, 1)
    for i, (k, v) in enumerate([('誰が', 'PM'), ('何のために', '次回の議題を組成すること'),
                                ('渡すもの', '定例の議事録'), ('返すもの', '課題の一覧')]):
        y = 142 + i * 36
        a += text(624, y, k, 15, ACCENT, 700)
        a += text(750, y, v, 17, INK)
    a += band(300, '6本とも、この1つの作業で進む')
    return svg('6本が扱うのは、案件の進行管理の中の作業1つである', a, 368)


def m1_map():
    """教材1のはじめに ── 6本ごとに、何が分かるか。"""
    a = text(0, 24, '1本ごとに分かること', 18, DIM)
    books = [('原因を知る', 'なぜ揃わないのか'), ('意味を決める', '何を並べてほしいか'),
             ('範囲を決める', 'どこまでを見てほしいか'), ('条件を決める', '必ず入れてほしいもの'),
             ('揺らぎを直す', 'それでも揃わないとき'), ('高さを合わせる', '3つが何だったのか')]
    for i, (name, q) in enumerate(books):
        x = (i % 3) * 384
        y = 48 + (i // 3) * 124
        a += rect(x, y, 344, 100, PAPER, 12, LINE)
        a += text(x + 24, y + 34, f'{i + 1}', 15, ACCENT, 700)
        a += text(x + 56, y + 34, name, 20, INK, 700)
        a += text(x + 24, y + 72, q, 16, DIM)
    a += band(320, '前の本で分かったことを、次の本が前提にする')
    return svg('6本それぞれで分かること', a, 388)
