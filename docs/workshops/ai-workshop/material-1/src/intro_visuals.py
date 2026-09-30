"""はじめに（全教材に共通する前置き）の図。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)


def i0_case():
    """案件の目的はAIに頼むには大きすぎる。関わる業務ごとの目的に分け、PMの目的に絞る。担当者は定例に集まり、書記が議事録にまとめる。"""
    # 上：案件の目的。このままではAIが何をすればいいか決まらない
    a = rect(0, 0, 640, 92, PAPER, 12, INK)
    a += text(24, 32, '案件の目的', 14, DIM, 700)
    a += text(24, 66, '経費精算システムを入れ替え、本番を切り替える', 20, INK, 700)
    a += text(616, 32, '本番の切り替えは3か月後', 13, DIM, anchor='end')
    a += arrow(652, 46, 700, 46)
    a += _robot_at(716, 20, DIM)
    a += text(790, 40, '大きすぎて、AIが', 16, DIM, 700)
    a += text(790, 66, '何をすればいいか決まらない', 16, DIM, 700)
    # 業務ごとの目的に分ける
    a += path('M320 92 V118 M176 118 H936 M176 118 V136 M556 118 V136 M936 118 V136', LINE, 2)
    for x in (176, 556, 936):
        a += path(f'M{x - 6} 130 l6 8 6 -8', LINE, 2)
    a += text(340, 112, '関わる業務ごとの目的に分ける', 13, DIM)
    roles = [('経費の精算業務', '経理', ['切り替えの前後も、', '精算と支払を止めない'], False),
             ('システムの開発', '開発', ['新しいシステムと連携を、', '切り替えまでに作る'], False),
             ('案件の進行管理', 'PM', ['本番の切り替えまで、', '案件を止めずに進める'], True)]
    for k, (name, who, goal, hot) in enumerate(roles):
        x = k * 380
        col = ACCENT if hot else INK
        a += rect(x, 142, 352, 132, PAPER, 12, ACCENT if hot else LINE)
        a += person(x + 36, 184, .62, col)
        a += text(x + 70, 176, name, 18, col, 700)
        a += text(x + 70, 200, '担当：' + who, 14, DIM)
        a += text(x + 24, 236, '目的：' + goal[0], 15, INK, 700)
        a += text(x + 72, 260, goal[1], 15, INK, 700)
        if hot:
            a += text(x + 328, 176, '教材の例', 13, ACCENT, 700, 'end')
    # 担当者が定例に集まり、書記が議事録にまとめる
    for k in range(3):
        a += path(f'M{k * 380 + 176} 274 V296', LINE)
    a += path('M176 296 H936 M380 296 V312 M374 306 l6 8 6 -8', LINE, 2)
    a += rect(170, 318, 420, 72, PAPER, 12, LINE)
    a += icon(194, 332, 'chat', INK, .8)
    a += text(254, 348, '週に1回の定例', 18, INK, 700)
    a += text(254, 374, '3つの業務の担当者が集まる', 15, INK)
    a += arrow(598, 354, 634, 354)
    a += rect(642, 318, 330, 72, PAPER, 12, LINE)
    a += icon(664, 332, 'doc', INK, .8)
    a += text(720, 348, '議事録', 18, INK, 700)
    a += text(720, 374, '書記が、決定と未決をまとめる', 15, INK)
    return svg('案件の目的は大きすぎるので、関わる業務ごとの目的に分け、PMの目的に絞る。担当者は定例に集まり、書記が議事録にまとめる', a, 398)


def i0_usecases():
    """業務の目的はそのままではAIに頼めない。目的を達成するための作業に分けると、作業ごとの目的は1行で書け、AIに頼める。"""
    # 上：業務と、その目的。このままAIに頼んでも、何を返せばいいか決まらない
    a = rect(0, 0, 700, 88, PAPER, 12, INK)
    a += person(34, 40, .62, INK)
    a += text(68, 34, '業務：案件の進行管理（PM）', 16, DIM, 700)
    a += text(68, 66, '目的：本番の切り替えまで、案件を止めずに進める', 19, INK, 700)
    a += arrow(712, 44, 752, 44)
    a += _robot_at(768, 18, DIM) + cross(846, 44, DIM, .6)
    a += text(870, 38, 'このまま頼んでも、', 15, DIM, 700)
    a += text(870, 62, '何を返せばいいか決まらない', 15, DIM, 700)
    # 目的を達成するための作業に分ける
    a += path('M350 88 V112', LINE, 2) + path('M344 106 l6 8 6 -8', LINE, 2)
    a += text(366, 108, '目的を達成するための作業に分ける', 13, DIM)
    works = [('課題を整理する', ['どれから決めるか、', '優先順位を付ける'], True),
             ('スケジュールを引き直す', ['切り替えの日に', '間に合う計画にする'], False),
             ('工数を見積もる', ['必要な人手と', '期間を把握する'], False),
             ('要員を調整する', ['足りない作業に', '要員を配置する'], False)]
    cw, gap = 254, 32
    for k, (name, goal, hot) in enumerate(works):
        x = k * (cw + gap)
        a += rect(x, 122, cw, 150, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 18, 150, '作業', 13, ACCENT if hot else DIM, 700)
        if hot:
            a += text(x + cw - 18, 150, '教材1で扱う', 13, ACCENT, 700, 'end')
        a += text(x + 18, 182, name, 18, ACCENT if hot else INK, 700)
        a += text(x + 18, 216, '目的：' + goal[0], 14, INK)
        a += text(x + 60, 240, goal[1], 14, INK)
        if k < 3:
            a += arrow(x + cw + 4, 197, x + cw + gap - 4, 197)
    a += text(cw + gap // 2, 290, '課題管理表を受け取る', 12, DIM, anchor='middle')
    # 下：作業の目的なら、1行で指示に書ける
    a += _robot_at(0, 306, INK) + check(78, 332, ACCENT, .55)
    a += text(104, 330, '作業の目的なら、1行で指示に書ける。AIに頼む1回の単位は、作業である', 17, INK, 700)
    a += text(104, 356, '作業の目的を1つずつ達成すると、業務の目的に近づく', 14, DIM)
    return svg('業務の目的はそのままではAIに頼めない。目的を達成するための作業に分けると、作業ごとの目的は1行で書け、AIに頼める', a, 366)


def _table_icon(x, y, color=INK):
    """表の形。課題管理表を示す。"""
    a = rect(x, y, 30, 32, PAPER, 3, color)
    a += path(f'M{x} {y + 10} H{x + 30} M{x} {y + 21} H{x + 30} M{x + 11} {y + 10} V{y + 32}', color, 1.5)
    return a


def i0_usecase():
    """目的には大きさがある。業務の目的では大きすぎ、作業の目的まで小さくすると、渡すものと返してもらうものが決まる。教材1で学ぶ3つを添える。"""
    # 左：目的の大きさ。上ほど大きい。段ごとに、何の目的かを絵で示す
    a = text(0, 18, '目的の大きさ', 15, DIM, 700)
    a += path('M14 34 V222 M7 215 l7 8 7 -8', LINE, 2)
    a += text(24, 46, '大きい', 12, DIM) + text(24, 222, '小さい', 12, DIM)
    rows = [(70, 'calendar', '案件の目的', '経費精算システムを入れ替える', False),
            (136, 'person', '業務の目的', '本番の切り替えまで、案件を止めずに進める', False),
            (202, 'table', '作業の目的', '優先順位を付けるために、課題を整理する', True)]
    for y, ic, tag, what, hot in rows:
        col = ACCENT if hot else INK
        a += rect(80, y - 34, 440, 56, PAPER, 10, ACCENT if hot else LINE)
        if ic == 'person':
            a += person(106, y - 4, .5, col)
        elif ic == 'table':
            a += _table_icon(92, y - 20, col)
        else:
            a += icon(92, y - 22, ic, col, .6)
        a += text(136, y - 12, tag, 12, ACCENT if hot else DIM, 700)
        a += text(136, y + 12, what, 15, col, 700)
        a += (check(500, y - 4, ACCENT, .45) if hot else cross(500, y - 4, DIM, .45))
    a += text(80, 250, '業務の目的では大きすぎて、何を返せばいいかが決まらない', 13, DIM)
    # 右：作業の目的をプロンプトに書き、議事録を添えて渡す。AIが解釈して、課題管理表を返す
    a += arrow(530, 198, 566, 198)
    a += rect(574, 0, 538, 236, PAPER, 12, ACCENT)
    a += text(598, 32, '教材1の例', 15, ACCENT, 700)
    a += rect(594, 50, 214, 168, PAPER, 8, INK)
    a += text(610, 76, '指示（いわゆるプロンプト）', 13, ACCENT, 700)
    a += text(610, 104, ['目的：優先順位を', '付けるために、', '課題を整理する'], 14, INK, 700, gap=22)
    a += path('M610 164 H792', LINE, 1)
    a += icon(612, 174, 'doc', INK, .5) + text(644, 200, '議事録を添えて渡す', 13, INK)
    a += arrow(814, 134, 858, 134)
    a += _robot_at(872, 108, ACCENT)
    a += text(896, 190, 'AIが', 13, ACCENT, 700, 'middle') + text(896, 210, '解釈する', 13, ACCENT, 700, 'middle')
    a += arrow(934, 134, 1000, 134)
    a += _table_icon(1016, 118, INK)
    a += text(1031, 190, '課題管理表', 14, INK, 700, 'middle') + text(1031, 210, '返ってくるもの', 12, DIM, 400, 'middle')
    # 下：教材1で学ぶこと
    a += text(0, 300, '教材1で学ぶこと', 15, DIM, 700)
    learn = [('search', ['返ってくるものが、', '目的からずれる原因を知る']),
             ('book', ['言葉の意味 ・ 対象の範囲 ・', '必ず入れる条件を、指示に書く']),
             ('check', ['指示をどこまで決めて書くかを、', '目的に合わせる'])]
    for k, (ic, ls) in enumerate(learn):
        x = k * 380
        a += rect(x, 314, 352, 72, PAPER, 10, LINE)
        a += icon(x + 18, 332 if ic != 'check' else 338, ic, INK, .6 if ic != 'check' else .7)
        a += text(x + 70, 344, ls, 14, INK, gap=22)
    return svg('目的には大きさがある。業務の目的では大きすぎ、作業の目的まで小さくすると、渡すものと返してもらうものが決まる。教材1で学ぶ3つ', a, 392)


def _mini_table(x, y, rows, head_col, filled):
    """課題管理表の小さい形。filled が偽なら、工数と遅延時の影響を「？」にする。"""
    cols = [('課題', 0), ('担当', 96), ('期限', 146), ('工数', 196), ('遅延時の影響', 256)]
    a = rect(x, y, 404, 150, PAPER, 10, head_col)
    for name, dx in cols:
        a += text(x + 12 + dx, y + 26, name, 12, DIM, 700)
    a += path(f'M{x} {y + 38} H{x + 404}', LINE, 1)
    for r, row in enumerate(rows):
        yy = y + 72 + r * 50
        for k, (name, dx) in enumerate(cols):
            val = row[k]
            if k >= 3 and not filled:
                a += rect(x + 8 + dx, yy - 20, 40 if k == 3 else 130, 28, PAPER, 6, ACCENT)
                a += text(x + 8 + dx + (20 if k == 3 else 65), yy, '？', 14, ACCENT, 700, 'middle')
            else:
                a += text(x + 12 + dx, yy, val, 13, ACCENT if (k >= 3) else INK, 700 if k in (0, 3, 4) else 400)
    return a


def i0_next2():
    """教材1の課題管理表は、工数と遅延時の影響が空で、優先順位を付ける精度に届かない。業務ごとの言葉の意味をチームで統一すると、表が埋まり、チームでも話が通じる。"""
    rows_b = [('連携の仕様', '開発', '9/16', '', ''), ('申請の締め日', '経理', '9/30', '', '')]
    rows_a = [('連携の仕様', '開発', '9/16', '3人日', '連携の開発が止まる'), ('申請の締め日', '経理', '9/30', '1人日', '運用の準備が遅れる')]
    a = text(0, 18, '教材1のあとの課題管理表', 14, DIM, 700)
    a += _mini_table(0, 30, rows_b, LINE, False)
    a += cross(18, 208, DIM, .45) + text(36, 213, '工数と影響が空で、優先順位を付けられない', 13, DIM)
    # 中央：チームで言葉の意味を統一する
    a += arrow(410, 105, 434, 105)
    a += rect(440, 0, 232, 226, PAPER, 12, INK)
    for k in range(3):
        a += person(486 + k * 70, 34, .45, INK)
    a += text(556, 80, 'チームで「課題」の', 14, INK, 700, 'middle')
    a += text(556, 100, '意味を統一する', 14, INK, 700, 'middle')
    for k, (who, what) in enumerate([('経理', '精算が止まる要因'), ('開発', '実装が進まない要因'), ('PM', '進行が止まる要因')]):
        y = 132 + k * 28
        a += text(458, y, who, 12, DIM, 700) + text(496, y, what, 13, INK)
    a += text(556, 216, '定義を、指示に書く', 12, ACCENT, 700, 'middle')
    a += arrow(678, 105, 702, 105)
    a += text(708, 18, '教材2のあとの課題管理表', 14, ACCENT, 700)
    a += _mini_table(708, 30, rows_a, ACCENT, True)
    a += check(726, 208, ACCENT, .45) + text(744, 213, '工数と影響が入り、優先順位に裏付けが付く', 13, ACCENT, 700)
    # 下：言葉の意味を統一すると、よいこと
    a += text(0, 262, '言葉の意味を統一すると', 15, DIM, 700)
    goods = [('robot', ['AIが、業務ごとの言葉を', '正しく読み分ける']),
             ('table', ['課題管理表の期限と', '優先順位に、裏付けが付く']),
             ('chat', ['チームの中でも、', '業務をまたいで話が通じる'])]
    for k, (ic, ls) in enumerate(goods):
        x = k * 380
        a += rect(x, 276, 352, 76, PAPER, 10, LINE)
        if ic == 'robot':
            a += _robot_at(x + 16, 290, INK)
        elif ic == 'table':
            a += _table_icon(x + 22, 298, INK)
        else:
            a += icon(x + 18, 298, ic, INK, .6)
        a += text(x + 80, 306, ls, 14, INK, gap=22)
    return svg('教材1の課題管理表は、工数と遅延時の影響が空で、優先順位を付ける精度に届かない。業務ごとの言葉の意味をチームで統一すると、表が埋まり、チームでも話が通じる', a, 358)


def i0_next3():
    """教材1と教材2で決めたことを、Skill ・ MCP ・ Hook ・ エージェントの4つの部品で仕組みにする。どれも特定のAIツールに依存しない。"""
    # 左：教材1と教材2で決めたこと
    a = text(0, 18, '教材1 ・ 2で決めたこと', 14, DIM, 700)
    a += rect(0, 30, 300, 292, PAPER, 12, INK)
    decided = [('目的', '優先順位を付けるために整理する'),
               ('言葉の意味', '「課題」が業務ごとに指すもの'),
               ('対象の範囲', '定例の議事録とスケジュール'),
               ('必ず入れる条件', '期限 ・ 工数 ・ 遅延時の影響')]
    ys = [76, 146, 216, 286]
    for (k_, v_), y in zip(decided, ys):
        a += text(20, y - 14, k_, 13, DIM, 700)
        a += text(20, y + 10, v_, 14, INK, 700)
    # 右：4つの部品。決めたことのどれを受け持つかを線で示す
    parts = [('doc', 'Skill', '作業ごとの指示と、言葉の意味をまとめる', 1),
             ('branch', 'MCP', 'AIが参照する情報源に、決めた範囲でつなぐ', 2),
             ('check', 'Hook', '決まった時点で動き、条件を満たすかを確認する', 3),
             ('robot', 'エージェント', '業務の目的のために、どのSkillをいつ使うかを判断する', 0)]
    for k, (ic, name, what, src) in enumerate(parts):
        y = 30 + k * 76
        hot = name == 'エージェント'
        a += path(f'M300 {ys[src]} C 340 {ys[src]}, 350 {y + 32}, 390 {y + 32}', LINE, 1.5)
        a += rect(394, y, 718, 64, PAPER, 10, ACCENT if hot else LINE)
        if ic == 'robot':
            a += _robot_at(412, y + 8, ACCENT)
        else:
            a += icon(414, y + 16 if ic != 'check' else y + 22, ic, INK, .7)
        a += text(484, y + 30, name, 18, ACCENT if hot else INK, 700)
        a += text(484, y + 54, what, 14, INK)
    # 下：どのAIツールでも使える。最後は人が確認して承認する
    a += text(0, 362, 'どのAIツールでも使える、仕組みの考え方として学ぶ', 16, INK, 700)
    a += _table_icon(620, 340, INK) + person(684, 356, .45, INK)
    a += text(708, 362, '返ってきた課題管理表は、PMが確認して承認する', 14, DIM)
    return svg('教材1と教材2で決めたことを、Skill ・ MCP ・ Hook ・ エージェントの4つの部品で仕組みにする。どれも特定のAIツールに依存しない', a, 380)


def i0_journey():
    """教材を1つ終えるごとに、AIでできることが1つずつ増える。段を上がるごとに、課題管理表と仕組みが育つ。"""
    steps = [('教材1　目的の把握', 'search', '議事録を渡すと、目的に合った', '課題管理表が返る', '議事録が替わっても、同じ指示で使える', 150),
             ('教材2　言葉の定義', 'book', '＋ 工数と影響まで入り、', '優先順位に裏付けが付く', 'チームで、言葉の意味が統一される', 90),
             ('教材3　仕組みの構築', 'robot', '＋ 誰が使っても、', '同じように動く', 'エージェントがSkillを使い分け、業務を支える', 30)]
    a = text(0, 20, '教材を終えると、AIでできること', 15, DIM, 700)
    base = 350
    for k, (tag, ic, l1, l2, sub, top) in enumerate(steps):
        x = k * 380
        hot = k == 2
        a += rect(x, top, 352, base - top, PAPER, 12, ACCENT if hot else LINE)
        if ic == 'robot':
            a += _robot_at(x + 22, top + 18, ACCENT)
        else:
            a += icon(x + 22, top + 20, ic, INK, .8)
        a += text(x + 88, top + 46, tag, 15, ACCENT if hot else DIM, 700)
        a += text(x + 22, top + 104, [l1, l2], 18, ACCENT if hot else INK, 700, gap=28)
        a += text(x + 22, top + 170, sub, 14, DIM)
        if k < 2:
            a += arrow(x + 356, top + 40, x + 376, top + 40)
    return svg('教材を1つ終えるごとに、AIでできることが1つずつ増える。段を上がるごとに、課題管理表と仕組みが育つ', a, base + 6)


def _bubble_r(x, y, w, h, tail_y):
    """右に尾を持つ吹き出し。右側の人の発言を示す。"""
    return (rect(x, y, w, h, PAPER, 12, LINE)
            + path(f'M{x + w - 1} {tail_y - 8} L{x + w + 14} {tail_y} L{x + w - 1} {tail_y + 8}', LINE, 2, PAPER))


def i0_base():
    """目的を言語化し、AIで仕組みを作り、目的を達成する。3つを、自分が実現したい目的で進める。"""
    a = ''
    cards = [('doc', '1　目的を言語化する', '準備期間 ・ 参加するとき',
              ['AIで実現したいことを書く', 'AIに頼める大きさまで定める', '達成に何が必要かを考える'], False),
             ('robot', '2　AIで仕組みを作る', 'ワークショップの3か月',
              ['Skillとエージェントで作る', '自分のチームで使ってもらう', '振り返りの場で直す'], False),
             ('check', '3　目的を達成する', 'ワークショップの終わり ・ 最終共有会',
              ['目的をAIで達成する', '作った仕組みを見せ合う'], True)]
    for k, (ic, head, when, items, hot) in enumerate(cards):
        x = k * 380
        col = ACCENT if hot else INK
        a += rect(x, 0, 352, 272, PAPER, 12, ACCENT if hot else LINE)
        if ic == 'robot':
            a += _robot_at(x + 22, 18, col)
        else:
            a += icon(x + 22, 20 if ic == 'doc' else 32, ic, col, .8)
        a += text(x + 22, 106, head, 20, col, 700)
        a += text(x + 22, 134, when, 14, DIM, 700)
        for j_, s_ in enumerate(items):
            y = 176 + j_ * 36
            a += circle(x + 28, y - 6, 4, LINE, LINE) + text(x + 44, y, s_, 15, INK)
        if k < 2:
            a += arrow(x + 356, 136, x + 376, 136)
    a += text(556, 318, '見るのは、何を作ったかではなく、目的をAIで達成できたか', 20, ACCENT, 700, 'middle')
    return svg('目的を言語化し、AIで仕組みを作り、目的を達成する。3つを、自分が実現したい目的で進める', a, 330)


def i0_effect():
    """業務ごとにエージェントがあり、その下に作業ごとのSkillが並ぶ。これができると、成果物の品質と作業の効率が上がる。"""
    cols = [('PM ・ PL', '案件を止めずに進める', ['課題を整理する', '進捗報告を作る', '遅れを検知する']),
            ('開発', '仕様どおりに、品質よく作る', ['仕様をまとめる', 'コードをレビューする', 'テスト項目を作る']),
            ('事務 ・ 管理', '申請と問い合わせを処理する', ['申請内容を確認する', '問い合わせに回答する', '定型の書類を作る'])]
    a = ''
    for k, (role, goal, skills) in enumerate(cols):
        x = k * 380
        a += rect(x, 0, 352, 88, PAPER, 12, INK)
        a += _robot_at(x + 20, 18, INK)
        a += text(x + 88, 38, role + 'のエージェント', 17, INK, 700)
        a += text(x + 88, 66, '業務：' + goal, 14, DIM)
        a += path(f'M{x + 44} 88 V{112 + 2 * 48 + 20}', LINE, 2)
        for m, sk in enumerate(skills):
            y = 112 + m * 48
            a += path(f'M{x + 44} {y + 20} H{x + 70}', LINE, 2)
            a += rect(x + 70, y, 282, 40, PAPER, 8, LINE)
            a += text(x + 86, y + 26, 'Skill', 13, DIM, 700) + text(x + 130, y + 26, sk, 15, INK)
    # 3つの業務から、効果へ
    a += path('M176 268 V288 M556 268 V288 M936 268 V288 M176 288 H936 M556 288 V304 m-7 -7 l7 7 7 -7', LINE, 2)
    effects = ['成果物の品質が上がる', '作業の効率が上がる', '誰が担当しても、同じ品質で出せる']
    for k, e in enumerate(effects):
        x = k * 380
        a += rect(x, 312, 352, 56, PAPER, 12, ACCENT)
        a += check(x + 30, 340, ACCENT, .5) + text(x + 54, 346, e, 17, ACCENT, 700)
    return svg('業務ごとにエージェントがあり、その下に作業ごとのSkillが並ぶ。これができると、成果物の品質と作業の効率が上がり、誰が担当しても同じ品質で出せる', a, 374)


def i0_flow():
    """10月から3月の進め方。準備期間に教材を見て目的を考え、12月に参加を受け付け、1月から3か月のワークショップで目的を達成する。"""
    cw = 1112 / 6
    a = ''
    for k, m in enumerate(['10月', '11月', '12月', '1月', '2月', '3月']):
        a += text(k * cw + cw / 2, 18, m, 14, DIM, 700, 'middle')
        if k:
            a += path(f'M{k * cw:.0f} 28 V84', LINE, 1)
    a += rect(0, 32, 552, 44, PAPER, 8, LINE) + text(18, 60, '準備期間（希望者）', 15, INK, 700)
    a += rect(560, 32, 552, 44, PAPER, 8, ACCENT) + text(578, 60, 'ワークショップ（先着15名）', 15, ACCENT, 700)
    # 節目
    a += path('M0 98 H1112', LINE, 1)
    marks = [(8, 116, '教材1を公開', 'start'),
             (278, 146, '教材2 ・ 3を順に公開', 'middle'),
             (463, 116, '参加の受付（12月中）', 'middle'),
             (566, 146, 'ワークショップ開始', 'start'),
             (834, 116, 'チームで使い、振り返る', 'middle'),
             (1104, 146, '最終共有会', 'end')]
    for x, y, label, anc in marks:
        a += circle(x, 98, 5, ACCENT if x > 552 else INK, ACCENT if x > 552 else INK)
        if y > 130:
            a += path(f'M{x} 103 V132', LINE, 1)
        a += text(x, y + 8, label, 13, INK, 700, anc)
    # 期間ごとにすること
    a += rect(0, 176, 540, 150, PAPER, 12, LINE)
    a += icon(22, 196, 'book', INK, .6) + text(70, 214, '教材をポータルで順に見て、', 16, INK, 700)
    a += text(70, 240, 'AIで実現したい目的を考える', 16, INK, 700)
    a += icon(22, 270, 'chat', INK, .55) + text(70, 294, '分からないことは、Teamsで相談できる', 14, DIM)
    a += rect(572, 176, 540, 150, PAPER, 12, ACCENT)
    a += _robot_at(594, 192, ACCENT) + text(660, 214, '目的を達成する仕組みを、', 16, ACCENT, 700)
    a += text(660, 240, 'AIで作る', 16, ACCENT, 700)
    a += text(596, 280, 'KiroのアカウントとAWS環境を発行する', 14, DIM)
    a += text(596, 306, '自分で達成するコースか、チームに広げるコースを選ぶ', 14, DIM)
    return svg('10月から3月の進め方。準備期間に教材を見て目的を考え、12月に参加を受け付け、1月から3か月のワークショップで目的を達成する', a, 334)


def _robot_at(x, y, color):
    """AIを指す図形。絶対座標で描く（derived_visuals._robot と同じ形）。"""
    a = path(f'M{x + 24} {y + 12} V{y + 3}', color, 2.3)
    a += circle(x + 24, y + 2, 3, 'none', color)
    a += rect(x + 2, y + 12, 44, 32, 'none', 9, color)
    a += circle(x + 16, y + 27, 3.5, color, color)
    a += circle(x + 32, y + 27, 3.5, color, color)
    a += path(f'M{x + 17} {y + 36} H{x + 31}', color, 2.3)
    a += path(f'M{x + 2} {y + 26} H{x - 6} M{x + 46} {y + 26} H{x + 54}', color, 2.3)
    a += path(f'M{x + 13} {y + 44} V{y + 52} H{x + 35} V{y + 44}', color, 2.3)
    return a


def i0_why():
    """目的を定め、言葉の意味を統一し、仕組みにする。3つを教材で1つずつ身につけ、目的をAIで達成する。"""
    steps = [('教材1　目的の把握', 'search', '目的を定める', 'AIに頼める大きさで、的確に',
              ['例：優先順位を付けるために、', '課題を整理する']),
             ('教材2　言葉の定義', 'book', '言葉の意味を統一する', '同じ言葉が指すものを定める',
              ['例：「課題」は、', '案件の進行が止まる要因']),
             ('教材3　仕組みの構築', 'robot', '仕組みにする', 'Skillに書き、エージェントが使う',
              ['例：誰が頼んでも、', '同じ課題管理表が返る'])]
    a = ''
    cw, gap = 280, 36
    for k, (tag, ic, name, what, ex) in enumerate(steps):
        x = k * (cw + gap)
        a += rect(x, 0, cw, 316, PAPER, 12, LINE)
        a += text(x + 22, 34, tag, 14, DIM, 700)
        if ic == 'robot':
            a += _robot_at(x + 22, 58, INK)
        else:
            a += icon(x + 22, 58, ic, INK, 1)
        a += text(x + 22, 152, name, 22, INK, 700)
        a += text(x + 22, 184, what, 15, INK)
        a += path(f'M{x + 22} 214 H{x + cw - 22}', LINE, 1)
        a += text(x + 22, 246, ex, 14, DIM, gap=24)
        a += arrow(x + cw + 6, 158, x + cw + gap - 6, 158)
    cx, cy, r = 1112 - 84, 158, 84
    a += circle(cx, cy, r, ACCENT, ACCENT)
    a += text(cx, cy - 8, '目的を', 21, PAPER, 700, 'middle')
    a += text(cx, cy + 24, 'AIで達成する', 21, PAPER, 700, 'middle')
    return svg('目的を定め、言葉の意味を統一し、仕組みにする。3つを教材で1つずつ身につけ、目的をAIで達成する', a, 324)


def i1_bridge():
    """今週の定例の議事録を渡し、「課題を整理して」とだけ頼む。返ってくる課題管理表は、頼むたびに違う。"""
    # 左：渡すもの。議事録の中身は、意味を決める動画の1枚目の原稿と同じにする
    a = text(0, 20, '渡すもの', 15, DIM)
    a += path('M0 32 H420 L452 64 V300 H0 Z', LINE, 2, PAPER)
    a += path('M420 32 V64 H452', LINE, 2)
    a += text(24, 64, '定例の議事録　今週の1回分', 17, INK, 700)
    a += text(24, 88, '出席　経理 ・ 開発 ・ PM', 13, DIM)
    a += path('M24 102 H428', LINE, 1)
    for i, line in enumerate(['承認の経路をどうするか', '差し戻しの扱いをどうするか', '申請の締め日をいつにするか',
                              '旧システムをいつ止めるか', '既存データをどう移すか', '月次の締めの進め方',
                              '試験環境の準備']):
        y = 130 + i * 24
        a += circle(30, y - 5, 2.5, INK, INK)
        a += text(42, y, line, 14, INK)
    # 中央の列は、左の議事録の右端（452）と右の枠の左端（812）のちょうど中間（632）を中心に置く
    # 矢印は、AIの左右の端に届かせる（AIは x 602〜662、中心の高さは 152）
    a += arrow(462, 152, 594, 152)
    # 中央：指示とAI
    a += text(490, 20, 'AIへの指示', 15, DIM)
    a += rect(490, 32, 284, 60, PANEL, 12)
    a += text(632, 70, '「課題を整理して」', 20, INK, 700, 'middle')
    a += path('M632 92 V118 M626 112 l6 7 6 -7', LINE, 2)
    a += _robot_at(608, 126, INK)
    a += text(632, 206, 'AI', 13, DIM, 700, 'middle')
    a += arrow(670, 152, 804, 152)
    # 右：返ってくるもの
    a += text(812, 20, '返ってくるもの', 15, DIM)
    a += rect(812, 32, 300, 268, PAPER, 12, LINE)
    a += _table_icon(836, 56)
    a += text(878, 78, '課題管理表', 17, INK, 700)
    a += path('M836 110 H1088', LINE, 1)
    a += cross(846, 146, ACCENT, .7)
    a += text(868, 152, '頼むたびに違う', 18, ACCENT, 700)
    a += text(836, 190, ['何度か頼むと、', '並べ方が毎回変わる'], 15, INK, gap=24)
    a += text(836, 270, '3回の中身は、次の枚で比べる', 13, DIM)
    return svg('今週の定例の議事録を渡し、課題を整理してとだけ頼む。返ってくる課題管理表は、頼むたびに違う', a, 308)


def i1_goal():
    """この教材で書くのは意味・範囲・条件の3つ。手がかりは抽象と具体という見方。最後まで観ると2つができる。"""
    a = text(0, 20, 'この教材で書くもの', 15, DIM)
    for i, (k, sub) in enumerate([('意味', '何を並べるか'), ('範囲', 'どこから拾うか'), ('条件', '必ず入れるもの')]):
        x = i * 216
        a += rect(x, 32, 200, 92, PAPER, 10, ACCENT)
        a += text(x + 100, 72, k, 22, INK, 700, 'middle')
        a += text(x + 100, 102, sub, 15, INK, anchor='middle')
    # 3つの下に、手がかりの見方を置く。3つを支えるので、3つの幅いっぱいに渡す
    a += path('M100 124 V144 M316 124 V144 M532 124 V144 M100 144 H532 M316 144 V158', LINE, 1.5)
    a += rect(0, 162, 632, 104, PANEL, 12)
    a += text(24, 190, '3つを書くときの手がかりになる見方', 14, DIM)
    a += text(24, 222, '抽象と具体', 22, INK, 700)
    a += text(24, 250, '指示を、どこまで決めて書くか。何を指すかは、原因を知る動画で扱う', 14, INK)
    a += text(680, 20, '最後まで観ると、できるようになること', 15, DIM)
    a += rect(680, 32, 432, 234, PAPER, 12, LINE)
    for i, (t, sub) in enumerate([('一度書いた指示を、', '来週の議事録にもそのまま使える'),
                                  ('渡す議事録が替わっても、', '返ってくるものが目的から外れない')]):
        y = 64 + i * 100
        a += circle(716, y + 8, 14, PANEL, PANEL)
        a += text(716, y + 14, str(i + 1), 16, INK, 700, 'middle')
        a += text(744, y + 14, [t, sub], 16, INK, gap=26)
    a += text(744, 244, 'この2つめを、再現性と呼ぶ', 15, ACCENT, 700)
    return svg('この教材で書くのは意味・範囲・条件の3つ。手がかりは抽象と具体という見方。最後まで観ると2つができる', a, 276)


def m1_task():
    """「課題を整理して」に、3つを1つずつ書き足すと、返ってくる課題管理表の揃うところが増えていく。"""
    a = text(130, 20, '指示', 14, DIM)
    a += text(344, 20, '書き足す3つ', 14, ACCENT, 700)
    a += text(716, 20, '返ってくる課題管理表', 14, DIM)
    rows = [('原因を知る', [], '頼むたびに並べ方が違う'), ('意味を決める', ['意味'], '並べるものが揃う'),
            ('範囲を決める', ['意味', '範囲'], '見る範囲が揃う'),
            ('条件を決める', ['意味', '範囲', '条件'], '1件ずつの形も揃う')]
    for i, (video, added, result) in enumerate(rows):
        y = 34 + i * 68
        a += text(0, y + 32, video, 14, DIM)
        a += rect(130, y, 200, 52, PAPER, 8, LINE)
        a += text(230, y + 32, '「課題を整理して」', 15, INK, 700, 'middle')
        for j, k in enumerate(added):
            x = 344 + j * 90
            a += rect(x, y + 6, 80, 40, PAPER, 8, ACCENT)
            a += text(x + 40, y + 32, '＋' + k, 15, ACCENT, 700, 'middle')
        # 矢印は、その行の最後の札のすぐ右から出す
        tail = 344 + len(added) * 90 - 4 if added else 338
        a += arrow(tail, y + 26, 708, y + 26)
        a += rect(716, y, 396, 52, PANEL if i == 0 else PAPER, 8, LINE)
        a += _table_icon(732, y + 10)
        a += text(776, y + 32, result, 16, DIM if i == 0 else INK, 400 if i == 0 else 700)
    a += text(0, 318, 'どの動画でも、書き足す前と後で3回ずつ頼み、返ってきたものを比べる', 14, DIM)
    return svg('課題を整理してに3つを1つずつ書き足すと、返ってくる課題管理表の揃うところが増えていく', a, 328)


def m1_map():
    """6本を順に並べ、4つの区切り（原因 ・ 3つを書き足す ・ 揃わないとき ・ 振り返る）を上に示す。"""
    books = [('原因を知る', ['なぜ揃わないのか']), ('意味を決める', ['何を並べるか']),
             ('範囲を決める', ['どこから拾うか']), ('条件を決める', ['必ず入れるもの']),
             ('揺らぎを直す', ['それでも揃わないとき']), ('抽象の高さを', ['3つが何だったのか'])]
    w, gap = 166, 23
    xs = [i * (w + gap) for i in range(6)]
    # 上：4つの区切り。区切りの幅は、含む動画の幅に合わせる
    groups = [('原因', 0, 0), ('指示に3つを書き足す', 1, 3), ('揃わないときの直し方', 4, 4), ('振り返る', 5, 5)]
    for name, i, j in groups:
        x0, x1 = xs[i], xs[j] + w
        a_ = path(f'M{x0 + 4} 40 V32 H{x1 - 4} V40', LINE, 1.5)
        a_ += text((x0 + x1) / 2, 22, name, 14, ACCENT if i == 1 else DIM, 700, 'middle')
        if name == groups[0][0]:
            a = a_
        else:
            a += a_
    for i, (name, q) in enumerate(books):
        x = xs[i]
        a += rect(x, 52, w, 132, PAPER, 10, ACCENT if 1 <= i <= 3 else LINE)
        a += text(x + 16, 80, f'{i + 1}', 15, DIM, 700)
        title = [name, '合わせる'] if i == 5 else [name]
        a += text(x + 16, 108, title, 16, INK, 700, gap=22)
        a += text(x + 16, 168, q, 13, INK)
        if i < 5:
            a += path(f'M{x + w + 5} 118 H{x + w + 17} M{x + w + 12} 113 l5 5 -5 5', LINE, 1.5)
    a += band(212, '前の動画で分かったことを、次の動画で前提にする')
    return svg('6本を順に並べ、4つの区切りを上に示す', a, 278)


def _dash(d, color=LINE, width=2):
    """破線。渡せない経路を示す。"""
    return (f'<path d="{d}" stroke="{color}" stroke-width="{width}" fill="none" '
            'stroke-dasharray="6 6" stroke-linecap="round"/>')


def _bubble(x, y, w, h, tail_y):
    """左に尾を持つ吹き出し。発言であることを示す。"""
    return (rect(x, y, w, h, PAPER, 12, LINE)
            + path(f'M{x + 1} {tail_y - 8} L{x - 14} {tail_y} L{x + 1} {tail_y + 8}', LINE, 2, PAPER))


def i0_scene():
    """経理と開発の発言を、PMは本番切り替えまでのスケジュールに置いて、どの作業が止まるかで判断する。"""
    a = text(0, 22, '定例で挙がる課題', 17, DIM)
    speakers = [('経理', ['「締め日の運用を変えないと、', '　月末の支払が回らない」'], '日々の精算が止まること', 108),
                ('開発', ['「連携の仕様が決まらず、', '　作り始められない」'], '実装が進まないこと', 220)]
    for who, body, mean, y in speakers:
        a += person(24, y + 30, .7, DIM)
        a += text(24, y + 70, who, 14, DIM, 700, 'middle')
        a += _bubble(66, y, 300, 66, y + 30)
        a += text(84, y + 28, body, 15, INK, gap=22)
        a += text(66, y + 90, '指すもの', 13, DIM)
        a += text(130, y + 90, mean, 14, INK, 700)
    # 右：PMが見ている、本番切り替えまでのスケジュール
    x0, x1 = 470, 1050            # 9/1 と 12/1 の位置
    def at(m, d):                  # 月日を横位置へ
        days = {9: 0, 10: 30, 11: 61, 12: 91}[m] + d - 1
        return round(x0 + (x1 - x0) * days / 91)
    a += text(410, 22, 'PMが見ているもの', 17, DIM)
    a += rect(410, 34, 702, 290, PAPER, 12, ACCENT)
    a += person(442, 70, .6, INK)
    a += text(468, 66, 'PMにとっての課題', 13, DIM)
    a += text(468, 88, '本番の切り替えまでに、進行が止まるかどうか', 17, ACCENT, 700)
    for m, lab in [(9, '9月'), (10, '10月'), (11, '11月')]:
        a += text(at(m, 1) + 6, 116, lab, 13, DIM)
        a += path(f'M{at(m, 1)} 104 V310', PANEL, 1.5)
    a += path(f'M{x1} 104 V310', ACCENT, 2)
    a += text(x1 - 6, 116, '本番切り替え', 13, ACCENT, 700, 'end')
    # 行の中心を、左の吹き出しの中心と同じ高さに置く
    rows = [('運用の準備と確認', (9, 30), (11, 25), '②', '9/30までに、経理が申請の締め日を決める', 128),
            ('連携の開発', (9, 16), (11, 15), '①', '9/16までに、開発が連携の仕様を決める', 240)]
    for name, st, en, no, what, y in rows:
        xs, xe = at(*st), at(*en)
        a += rect(xs, y, xe - xs, 26, PANEL, 6)
        a += text(xs + 12, y + 18, name, 14, INK, 700)
        a += path(f'M{xs} {y - 6} V{y + 32}', ACCENT, 3)
        a += text(xs - 8, y + 18, no, 16, ACCENT, 700, 'end')
        a += text(xs, y + 50, what, 14, INK)
    for y in (141, 253):
        a += arrow(372, y, 404, y)
    a += band(344, 'この読み解きと判断に、毎週かなりの時間がかかっている')
    return svg('経理と開発の発言を、PMは本番切り替えまでのスケジュールに置いて、どの作業が止まるかで判断する', a, 410)


def _ray(x1, y1, x2, y2, color=INK, width=2, dashed=False):
    """斜めの矢印。矢じりは (x2, y2) を指す。dashed なら軸だけ破線にする。"""
    import math
    L = math.hypot(x2 - x1, y2 - y1)
    ux, uy = (x2 - x1) / L, (y2 - y1) / L
    bx, by = x2 - 11 * ux, y2 - 11 * uy
    wing = path(f'M{bx - 6 * uy:.1f} {by + 6 * ux:.1f} L{x2} {y2} L{bx + 6 * uy:.1f} {by - 6 * ux:.1f}', color, width)
    shaft = _dash(f'M{x1} {y1} L{x2} {y2}', color, width) if dashed else path(f'M{x1} {y1} L{x2} {y2}', color, width)
    return shaft + wing


def i0_dig():
    """いまは基準がPMの頭の中にしかなく、補佐にも後任にもAIにも届かない。言葉にすれば届く。左右は同じ部品を同じ位置に置き、線の届き方だけを変える。"""
    RY = [88, 164, 240]   # 受け手の中心の高さ

    def receivers(x):
        out = ''
        for who, y in zip(['補佐', '後任', 'AI'], RY):
            if who == 'AI':
                out += _robot_at(x - 24, y - 30, INK)
            else:
                out += person(x, y + 4, .62, INK)
            out += text(x + 40, y + 6, who, 15, INK, 700)
        return out
    # 左：いま。点線は途中で止まり、受け手に届かない
    a = text(0, 22, 'いま', 17, DIM)
    a += rect(0, 34, 530, 262, PAPER, 12, LINE)
    a += rect(24, 56, 200, 64, PAPER, 32, ACCENT)
    a += text(124, 82, '判断の基準', 17, ACCENT, 700, 'middle')
    a += text(124, 104, 'PMの頭の中にしかない', 13, DIM, anchor='middle')
    a += circle(116, 134, 6, PAPER, ACCENT) + circle(110, 152, 4, PAPER, ACCENT)
    a += person(110, 198, .9, INK)
    a += text(110, 250, 'PM', 15, INK, 700, 'middle')
    a += text(110, 276, '毎回その場で考え直す', 14, DIM, anchor='middle')
    for y in RY:
        a += _ray(150, 188, 380, y, LINE, 2, dashed=True)
    a += receivers(420)
    for y in RY:
        a += cross(506, y, DIM, .7)
    a += arrow(540, 165, 574, 165)
    # 右：言葉にすると。実線の矢印が受け手に届く
    a += text(582, 22, '基準を言葉にすると', 17, DIM)
    a += rect(582, 34, 530, 262, PAPER, 12, LINE)
    a += rect(604, 64, 236, 202, PAPER, 8, INK)
    a += text(622, 96, '言葉にした基準', 16, INK, 700)
    a += path('M622 110 H822', LINE, 1)
    a += text(622, 144, '整理の目的を定める', 15, INK)
    a += text(822, 144, '教材1', 13, DIM, 700, 'end')
    a += text(622, 182, '言葉の意味を統一する', 15, INK)
    a += text(822, 182, '教材2', 13, DIM, 700, 'end')
    a += path('M622 210 H822', LINE, 1)
    a += text(622, 240, '仕組みにする', 14, DIM)
    a += text(822, 240, '教材3', 13, DIM, 700, 'end')
    for y in RY:
        a += _ray(848, 165, 960, y, INK, 2)
    a += receivers(1000)
    for y in RY:
        a += check(1086, y, ACCENT, .7)
    a += band(318, '頭の中の基準を言葉にすれば、ほかの人にもAIにも渡せる')
    return svg('いまは基準がPMの頭の中にしかなく、補佐にも後任にもAIにも届かない。言葉にすれば届く', a, 384)


def i0_books():
    """教材の考え方のもとにした3冊を、表紙と、どの教材の考え方を深められるかで紹介する。"""
    from visuals import cover, BOOKS
    notes = [('教材1', ['目的を捉える高さを、', 'システム開発の場面で説明する']),
             ('教材1', ['目的のレベルを分けて、', '何を実現したいかを書く']),
             ('教材2', ['業務の言葉の意味を統一し、', 'ソフトウェアの設計と結びつける'])]
    a = ''
    for k, ((isbn, title, author, pub), (lesson, note)) in enumerate(zip(BOOKS, notes)):
        x = k * 384
        a += cover(x + 94, 0, 154, 220, isbn)
        a += text(x + 171, 256, ''.join(title), 16, INK, 700, 'middle')
        a += text(x + 171, 282, f'{author}（{pub}）', 13, DIM, anchor='middle')
        a += rect(x + 20, 302, 302, 86, PAPER, 10, LINE)
        a += text(x + 40, 328, lesson + 'を深める', 13, DIM, 700)
        a += text(x + 40, 352, note, 14, INK, gap=22)
    return svg('教材の考え方のもとにした3冊。システム開発と「具体と抽象」とユースケース実践ガイドは教材1を、ドメイン駆動設計をはじめようは教材2を深められる', a, 396)
