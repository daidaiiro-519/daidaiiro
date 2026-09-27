"""はじめに（全教材に共通する前置き）の図。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)


def i0_case():
    """1つの案件に、3つの業務が関わっている。"""
    a = text(0, 24, '3本の教材が通して使う案件', 18, DIM)
    a += rect(356, 44, 400, 84, ACCENT, 12, ACCENT)
    a += text(556, 82, '経費精算システムを入れ替える', 21, PAPER, 700, 'middle')
    a += text(556, 110, '本番の切り替えは3か月後', 15, PAPER, anchor='middle')
    roles = [('経費の精算業務', '経理が、日々の精算と支払を担当している'),
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
    a += text(32, 352, '決まったことと、まだ決まっていないことを、書記が議事録にまとめる', 15, DIM)
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
    ucs = [('課題を整理する', True), ('スケジュールを引き直す', False),
           ('工数を見積もる', False), ('要員を調整する', False)]
    for i, (name, hot) in enumerate(ucs):
        y = 44 + i * 62
        a += rect(400, y, 712, 48, ACCENT if hot else PAPER, 10, ACCENT if hot else LINE)
        a += text(424, y + 31, name, 19, PAPER if hot else INK, 700)
        if hot:
            a += text(1088, y + 31, '教材1が扱う', 15, PAPER, anchor='end')
        if i < len(ucs) - 1:
            a += path(f'M440 {y + 48} V{y + 62} M435 {y + 57} l5 5 5 -5', ACCENT if hot else LINE, 1.6)
    a += text(1112, 24, '前の作業の結果を、次の作業が受け取る', 14, DIM, anchor='end')
    a += band(310, '精算業務にも開発にも、それぞれの作業がある')
    return svg('業務の中に、作業が並んでいる', a, 378)


def i0_usecase():
    """作業1つは、4つで書ける。"""
    a = text(0, 24, '教材1が扱う作業', 18, DIM)
    a += rect(0, 44, 536, 250, PAPER, 12, ACCENT)
    a += text(28, 88, '課題を整理する', 24, INK, 700)
    a += path('M28 106 H508', LINE, 1)
    rows = [('誰が', 'PM'), ('何のために', 'どの課題から決めるか、優先順位を付ける'),
            ('渡すもの', '定例の議事録'), ('返すもの', '課題管理表')]
    for i, (k, v) in enumerate(rows):
        y = 142 + i * 42
        a += text(28, y, k, 15, ACCENT, 700)
        a += text(150, y, v, 17, INK)
    a += text(596, 24, '作業から始める理由', 18, DIM)
    a += rect(596, 44, 516, 108, PANEL, 12)
    a += text(624, 80, '業務のまま', 15, DIM, 700)
    a += text(624, 108, '「本番まで案件を止めずに進める」', 17, INK)
    a += text(624, 134, '何を渡し、何を返してもらうかが決まらない', 15, DIM)
    a += rect(596, 168, 516, 126, PAPER, 12, ACCENT)
    a += text(624, 204, '作業にすると', 15, ACCENT, 700)
    a += text(624, 232, '渡すものは議事録、返すものは課題管理表', 17, INK)
    a += text(624, 258, 'そのまま優先順位を付けられるかで、', 15, DIM)
    a += text(624, 280, 'うまくいったかを判断できる', 15, DIM)
    a += band(316, '業務のままでは、AIに何を頼めばいいかが決まらない')
    return svg('作業1つは、誰が・何のために・何を受け取り・何を返すかで書ける', a, 384)


def i0_journey():
    """3本の並び。"""
    a = text(0, 24, '前の本で決めたことを、次の本が前提にする', 18, DIM)
    items = [('教材1', '課題の把握', 'PMの頭の中の基準を、どう指示に書くか',
              '意味・範囲・条件の3つを書く'),
             ('教材2', '業務の整理', '業務ごとに違う言葉を、どう揃えるか',
              '言葉を業務ごとに揃える手順'),
             ('教材3', '仕組みの構築', '言葉にしたことを、どう仕組みにするか',
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
    a += band(276, 'PMの業務 → 案件に関わる業務全体 → チームで使う仕組みへ広がる')
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
    a += text(556, 168, '業務で使う言葉を、それぞれの業務の中で揃える', 24, PAPER, 700, 'middle')
    a += text(556, 200, '揃えた言葉のまま、仕組みにする', 24, PAPER, 700, 'middle')
    a += text(556, 228, '全社で1つの意味に統一するのではなく、その業務での意味に揃える',
              14, PAPER, anchor='middle')
    a += text(0, 278, 'なぜ、いま土台に置くのか', 18, INK, 700)
    a += rect(0, 292, W, 76, PANEL, 12)
    a += rect(0, 292, 5, 76, ACCENT, 2, ACCENT)
    a += text(32, 328, '人どうしなら、言葉がずれていても、その場で聞き返して埋められる', 17, INK)
    a += text(32, 354, 'AIは聞き返さずに、そのまま実行する', 17, ACCENT, 700)
    return svg('3本の土台は、業務の言葉を業務ごとに揃え、そのまま仕組みにすることである', a, 378)


def i0_why():
    """いま起きていることと、このワークショップの先。"""
    a = text(0, 24, 'いま起きていること', 18, DIM)
    a += rect(0, 44, 520, 190, PAPER, 12, LINE)
    for i, s in enumerate(['人によって、AIへの指示の書き方が違う',
                           '同じ作業でも、返ってくるものが違う',
                           'うまくいった指示が、共有されない']):
        a += cross(32, 88 + i * 52, DIM, .6)
        a += text(62, 94 + i * 52, s, 17, INK)
    a += arrow(536, 139, 580, 139, ACCENT)
    a += text(596, 24, 'このワークショップのあと', 18, ACCENT, 700)
    a += rect(596, 44, 516, 190, PAPER, 12, ACCENT)
    for i, s in enumerate(['自分の業務の課題を、自分で見極める',
                           'AIで動く仕組みにする',
                           '同じ業務の人が、その仕組みを使う']):
        a += check(628, 88 + i * 52, ACCENT, .8)
        a += text(658, 94 + i * 52, s, 17, INK)
    a += rect(0, 258, W, 88, PANEL, 12)
    a += rect(0, 258, 5, 88, ACCENT, 2, ACCENT)
    a += text(32, 294, 'AIの使い方を、個人の工夫で終わらせない', 18, INK)
    a += text(32, 322, '誰が使っても同じように動く形にして、チームの進め方にする', 19, ACCENT, 700)
    return svg('いま起きていることと、このワークショップのあとに変わること', a, 356)


def i1_bridge():
    """前の動画で選んだ作業を、AIに頼む。"""
    a = text(0, 24, 'すでに決まっている4つ', 18, DIM)
    a += rect(0, 44, 536, 200, PAPER, 12, ACCENT)
    a += text(28, 86, '課題を整理する', 23, INK, 700)
    a += path('M28 104 H508', LINE, 1)
    for i, (k, v) in enumerate([('誰が', 'PM'), ('何のために', 'どの課題から決めるか、優先順位を付ける'),
                                ('渡すもの', '定例の議事録'), ('返すもの', '課題管理表')]):
        y = 132 + i * 32
        a += text(28, y, k, 14, ACCENT, 700)
        a += text(150, y, v, 16, INK)
    a += arrow(552, 144, 596, 144, ACCENT)
    a += rect(612, 44, 500, 88, PANEL, 12)
    a += text(636, 78, 'AIへの指示', 15, DIM)
    a += text(862, 112, '「課題を整理して」', 23, INK, 700, 'middle')
    a += path('M862 132 V152 M856 146 l6 6 6 -6', LINE, 2)
    a += rect(612, 156, 500, 88, PAPER, 12, LINE)
    a += text(636, 190, '返ってきたもの', 15, DIM)
    a += cross(650, 216, DIM, .7)
    a += text(678, 222, '頼むたびに違う', 21, INK, 700)
    a += band(268, '渡すものも、返してほしいものも決まっている。それでも、揃わない')
    return svg('前の動画で選んだ作業を、AIに頼む', a, 336)


def i1_goal():
    """この6本で身につけるもの。"""
    a = text(0, 22, 'この6本の動画で身につけるもの', 17, DIM)
    a += rect(0, 38, 536, 190, PAPER, 12, ACCENT)
    a += text(28, 86, '抽象と具体という見方', 25, ACCENT, 700)
    a += path('M28 106 H508', LINE, 1)
    a += text(28, 140, ['指示をどこまで決めて書くか、を見る見方。', 'どちらがよい、という話ではない'], 16, DIM, gap=26)
    a += text(28, 204, 'ちょうどいい抽象の高さを見極める', 20, INK, 700)
    a += text(576, 22, '最後まで観ると、こうなる', 17, ACCENT, 700)
    a += rect(576, 38, 536, 190, PANEL, 12)
    for i, s in enumerate(['一度書いた指示を、来週の議事録にも使える',
                           '渡す議事録が替わっても、目的から外れない']):
        y = 104 + i * 64
        a += check(608, y - 6, ACCENT, .8)
        a += text(638, y, s, 17, INK)
    a += band(252, '何を指すかは、原因を知る動画で、実際の指示を見ながら説明する')
    return svg('この6本で身につけるものと、最後まで観たときに変わること', a, 320)


def m1_task():
    """教材1のはじめに ── 6本が扱うのは、作業1つである。"""
    a = text(0, 24, '案件の進行管理の中の作業', 18, DIM)
    ucs = [('課題を整理する', True), ('スケジュールを引き直す', False),
           ('工数を見積もる', False), ('要員を調整する', False)]
    for i, (name, on) in enumerate(ucs):
        y = 48 + i * 58
        a += rect(0, y, 460, 46, ACCENT if on else PAPER, 10, ACCENT if on else LINE)
        a += text(24, y + 30, name, 19, PAPER if on else DIM, 700 if on else 400)
        if i < len(ucs) - 1:
            a += path(f'M40 {y + 46} V{y + 58} M35 {y + 53} l5 5 5 -5', ACCENT if on else LINE, 1.6)
    a += arrow(492, 140, 540, 140, ACCENT)
    a += text(596, 24, 'この6本が扱う作業', 18, ACCENT, 700)
    a += rect(596, 44, 516, 232, PAPER, 12, ACCENT)
    a += text(624, 88, '課題を整理する', 24, INK, 700)
    a += path('M624 106 H1084', LINE, 1)
    for i, (k, v) in enumerate([('誰が', 'PM'), ('何のために', 'どの課題から決めるか、優先順位を付ける'),
                                ('渡すもの', '定例の議事録'), ('返すもの', '課題管理表')]):
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
             ('揺らぎを直す', 'それでも揃わないとき'), ('抽象の高さを合わせる', '3つが何だったのか')]
    for i, (name, q) in enumerate(books):
        x = (i % 3) * 384
        y = 48 + (i // 3) * 124
        a += rect(x, y, 344, 100, PAPER, 12, LINE)
        a += text(x + 24, y + 34, f'{i + 1}', 15, ACCENT, 700)
        a += text(x + 56, y + 34, name, 20, INK, 700)
        a += text(x + 24, y + 72, q, 16, DIM)
    a += band(320, '前の本で分かったことを、次の本が前提にする')
    return svg('6本それぞれで分かること', a, 388)


def i0_scene():
    """経理と開発が、それぞれの意味で課題を挙げる。PMはそれを、案件の進行にとって何を意味するかで判断する。"""
    a = text(0, 22, '定例で挙がる課題', 17, DIM)
    for i, (who, body, mean) in enumerate([
            ('経理の課題', ['締め日の運用を変えないと、', '月末の支払が回らない'], 'ここでの課題　日々の精算が止まること'),
            ('開発の課題', ['連携の仕様が決まらず、', '作り始められない'], 'ここでの課題　実装が進まないこと')]):
        y = 38 + i * 128
        a += rect(0, y, 330, 112, PAPER, 12, LINE)
        a += text(22, y + 30, who, 17, INK, 700)
        a += text(22, y + 58, body, 15, INK, gap=22)
        a += text(22, y + 100, mean, 13, DIM)
    a += arrow(340, 150, 376, 150, ACCENT)
    a += text(386, 22, 'PMが1件ずつ判断する', 17, ACCENT, 700)
    a += rect(386, 38, 330, 240, PANEL, 12)
    for i, s_ in enumerate(['案件の進行にどう効くか', 'いつまでに決めないと止まるか', '誰が決めるのか']):
        y = 92 + i * 62
        a += check(412, y - 6, ACCENT, .7)
        a += text(442, y, s_, 16, INK)
    a += arrow(726, 150, 762, 150, ACCENT)
    a += text(772, 22, 'その結果として決まること', 17, ACCENT, 700)
    a += rect(772, 38, 340, 240, PAPER, 12, ACCENT)
    a += text(796, 84, '次に決めること', 17, INK, 700)
    a += text(796, 110, '何を、いつまでに、誰が', 15, DIM)
    a += text(796, 158, 'どれから決めるか', 17, INK, 700)
    a += text(796, 184, 'スケジュールと照らした優先順位', 15, DIM)
    a += path('M796 212 H1088', LINE, 1)
    a += text(796, 246, 'このあと、スケジュールを引き直す', 15, DIM)
    a += band(300, 'この読み解きと判断に、毎週かなりの時間がかかっている')
    return svg('経理と開発の課題を、PMが案件の進行にとって何を意味するかで判断する', a, 368)


def i0_dig():
    """本質、頭の中にしかないと起きること、解決するものと3本の教材。"""
    a = text(0, 22, '課題の本質', 17, ACCENT, 700)
    a += rect(0, 38, 260, 214, PAPER, 12, ACCENT)
    a += text(22, 78, ['業務ごとの課題が、', '案件の進行にとって', '何を意味するか。', 'その判断の基準が、', 'PMの頭の中にしかない'], 16, INK, 700, gap=30)
    a += arrow(270, 145, 302, 145, ACCENT)
    a += text(312, 22, '頭の中にしかないと、起きること', 17, DIM)
    a += rect(312, 38, 340, 214, PANEL, 12)
    a += text(334, 78, '1　PMが毎回その場で考え直す', 16, INK, 700)
    a += text(356, 104, '時間がかかる', 15, DIM)
    a += text(334, 150, '2　補佐にも後任にも渡せない', 16, INK, 700)
    a += text(356, 176, '人によって、指示に書くことが', 15, DIM)
    a += text(356, 200, '変わる', 15, DIM)
    a += arrow(662, 145, 694, 145, ACCENT)
    a += text(704, 22, '解決するもの　基準を言葉にし、仕組みにする', 17, ACCENT, 700)
    for i, (what, sub, no) in enumerate([('PMにとっての課題の捉え方', '何を課題とし、何を添え、どう並べるか', '教材1'),
                                         ('業務ごとの言葉の読み方', '経理や開発の課題が何を指すか', '教材2'),
                                         ('仕組みにする', '誰が使っても同じように動く形', '教材3')]):
        y = 38 + i * 74
        a += rect(704, y, 408, 64, PAPER, 10, ACCENT if i < 2 else LINE)
        a += text(724, y + 27, what, 16, INK, 700)
        a += text(724, y + 50, sub, 13, DIM)
        a += text(1094, y + 38, no, 15, ACCENT, 700, 'end')
    a += band(276, '3つの段階が、そのまま3本の教材になる')
    return svg('本質と、頭の中にしかないと起きることと、3本の教材で言葉にするもの', a, 344)
