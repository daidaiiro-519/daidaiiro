"""はじめに（全教材に共通する前置き）の図。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)
from theme import T

MARK = T['mark']  # 安定したところの塗り。強調ではないので、赤茶を使わない


def i0_case():
    """案件の目的はAIに頼むには大きすぎる。関わる業務ごとの目的に分け、PMの目的に絞る。担当者は定例に集まり、書記が議事録にまとめる。"""
    # 上：案件の目的。このままではAIが任せたい作業がまだ決まっていない
    a = rect(0, 0, 640, 92, PAPER, 12, INK)
    a += text(24, 32, '案件の目的', 14, DIM, 700)
    a += text(24, 66, '経費精算システムを入れ替え、本番を切り替える', 20, INK, 700)
    a += text(616, 32, '本番の切り替えは3か月後', 13, DIM, anchor='end')
    a += arrow(652, 46, 700, 46)
    a += _robot_at(716, 20, DIM)
    a += text(790, 40, '関わる業務と作業が多く、', 16, DIM, 700)
    a += text(790, 66, '任せたい作業がまだ決まっていない', 16, DIM, 700)
    # 業務ごとの目的に分ける
    a += path('M320 92 V118 M176 118 H936 M176 118 V136 M556 118 V136 M936 118 V136', LINE, 2)
    for x in (176, 556, 936):
        a += path(f'M{x - 6} 130 l6 8 6 -8', LINE, 2)
    a += text(340, 112, '関わる人ごとに、目的が違う', 13, DIM)
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
    a += text(720, 374, '書記が、決定事項と未決事項を書く', 15, INK)
    return svg('案件の目的は大きすぎるので、関わる業務ごとの目的に分け、PMの目的に絞る。担当者は定例に集まり、書記が議事録にまとめる', a, 398)


def i0_usecases():
    """業務の目的はそのままではAIに頼めない。目的を達成するための作業に分けると、作業ごとの目的は1行で書け、AIに頼める。"""
    # 上：業務と、その目的。このままAIに頼んでも、何を返せばいいか決まらない
    a = rect(0, 0, 700, 88, PAPER, 12, INK)
    a += person(34, 40, .62, INK)
    a += text(68, 34, '業務：案件の進行管理（PM）', 16, DIM, 700)
    a += text(68, 66, '目的：本番の切り替えまで、案件を止めずに進める', 19, INK, 700)
    a += arrow(712, 44, 752, 44)
    a += _robot_at(768, 18, DIM)
    a += text(870, 38, 'この目的には、', 15, DIM, 700)
    a += text(870, 62, '課題整理以外の作業も含まれる', 15, DIM, 700)
    # 目的を達成するための作業に分ける
    a += path('M350 88 V112', LINE, 2) + path('M344 106 l6 8 6 -8', LINE, 2)
    a += text(366, 108, '目的を達成するための作業に分ける', 13, DIM)
    works = [('課題を整理する', ['どれから決めるか、', '優先順位を付ける'], True),
             ('スケジュールを見直す', ['切り替えの日に', '間に合う計画にする'], False),
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
    a += text(0, 290, '業務の目的に関わる作業の例。状況に応じて関連する。', 14, DIM)
    # 下：作業の目的なら、1行でプロンプトに書ける
    a += _robot_at(0, 306, INK) + check(78, 332, ACCENT, .55)
    a += text(104, 330, '任せたい作業を決め、その目的と必要な内容をAIに伝える', 17, INK, 700)
    a += text(104, 356, '作業の目的を1つずつ達成すると、業務の目的に近づく', 14, DIM)
    return svg('業務の目的はそのままではAIに頼めない。目的を達成するための作業に分けると、作業ごとの目的は1行で書け、AIに頼める', a, 366)


def _table_icon(x, y, color=INK):
    """表の形。課題管理表を示す。"""
    a = rect(x, y, 30, 32, PAPER, 3, color)
    a += path(f'M{x} {y + 10} H{x + 30} M{x} {y + 21} H{x + 30} M{x + 11} {y + 10} V{y + 32}', color, 1.5)
    return a


def i0_usecase():
    """目的には大きさがある。業務の目的では大きすぎ、作業の目的まで小さくすると、入力と出力が決まる。教材1で学ぶ3つを添える。"""
    # 左：目的の大きさ。上ほど大きい。段ごとに、何の目的かを絵で示す
    a = text(0, 18, '目的の大きさ', 15, DIM, 700)
    a += path('M14 34 V222 M7 215 l7 8 7 -8', LINE, 2)
    a += text(24, 46, '大きい', 12, DIM) + text(24, 222, '小さい', 12, DIM)
    rows = [(90, 'person', 'PMの業務の目的', '本番の切り替えまで、案件を止めずに進める', False),
            (190, 'table', '作業の目的', '優先順位を付けるために、課題を整理する', True)]
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
        a += (check(500, y - 4, ACCENT, .45) if hot else '')
    a += text(80, 250, '今回は、毎週の課題整理に合う大きさで目的を伝える', 13, DIM)
    # 右：作業の目的をプロンプトに書き、議事録を添えて入力する。AIが解釈して、課題管理表を出力する
    a += arrow(530, 198, 566, 198)
    a += rect(574, 0, 538, 236, PAPER, 12, ACCENT)
    a += text(598, 32, '教材1の例', 15, ACCENT, 700)
    a += rect(594, 50, 214, 168, PAPER, 8, INK)
    a += text(610, 76, 'プロンプト', 13, ACCENT, 700)
    a += text(610, 99, ['優先順位を付けるため', '課題を整理する。', '担当と期限を入れる。'], 14, INK, 700, gap=22)
    a += path('M610 164 H792', LINE, 1)
    a += icon(612, 174, 'doc', INK, .5) + text(644, 200, '議事録と一緒に入力する', 13, INK)
    a += arrow(814, 134, 858, 134)
    a += _robot_at(872, 108, ACCENT)
    a += text(896, 190, 'AIが', 13, ACCENT, 700, 'middle') + text(896, 210, '解釈する', 13, ACCENT, 700, 'middle')
    a += arrow(934, 134, 1000, 134)
    a += _table_icon(1016, 118, INK)
    a += text(1031, 190, '課題管理表', 14, INK, 700, 'middle') + text(1031, 210, '出力', 12, DIM, 400, 'middle')
    # 下：教材1で学ぶこと
    a += text(0, 300, '教材1で学ぶこと', 15, DIM, 700)
    learn = [('search', ['出力が、', '目的からずれる原因を知る']),
             ('book', ['作業の目的を伝え、', '意味 ・ 範囲 ・ 条件を決めて伝える']),
             ('check', ['その週の項目まで、', '決めすぎない'])]
    for k, (ic, ls) in enumerate(learn):
        x = k * 380
        a += rect(x, 314, 352, 72, PAPER, 10, LINE)
        a += icon(x + 18, 332 if ic != 'check' else 338, ic, INK, .6 if ic != 'check' else .7)
        a += text(x + 70, 344, ls, 14, INK, gap=22)
    return svg('目的には大きさがある。業務の目的では大きすぎ、作業の目的まで小さくすると、入力と出力が決まる。教材1で学ぶ3つ', a, 392)


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
    """教材1の課題管理表は、工数と遅延時の影響が空で、優先順位を付ける精度に届かない。業務ごとの言葉の意味と違いを定義すると、表が埋まり、チームでも話が通じる。"""
    rows_b = [('連携の仕様', '開発', '9/16', '', ''), ('申請の締め日', '経理', '9/30', '', '')]
    rows_a = [('連携の仕様', '開発', '9/16', '3人日', '連携の開発が止まる'), ('申請の締め日', '経理', '9/30', '1人日', '運用の準備が遅れる')]
    a = text(0, 18, '教材1のあとの課題管理表', 14, DIM, 700)
    a += _mini_table(0, 30, rows_b, LINE, False)
    a += text(0, 213, '担当・期限が入る。工数・影響は次に確認', 13, DIM)
    # 中央：チームで言葉の意味を統一する
    a += arrow(410, 105, 434, 105)
    a += rect(440, 0, 232, 252, PAPER, 12, INK)
    for k in range(3):
        a += person(486 + k * 70, 34, .45, INK)
    a += text(556, 80, '「課題」が指すものを言語化し、', 14, INK, 700, 'middle')
    a += text(556, 100, '同じチームの人と合わせる', 14, INK, 700, 'middle')
    for k, (who, what) in enumerate([('経理', '精算が止まる要因'), ('開発', '実装が進まない要因'), ('PM', '進行が止まる要因')]):
        y = 132 + k * 28
        a += text(458, y, who, 12, DIM, 700) + text(496, y, what, 13, INK)
    a += text(556, 216, '定義をAIに伝える', 12, ACCENT, 700, 'middle')
    a += text(556, 242, '＋各業務の記録を入力する', 14, INK, 700, 'middle')
    a += arrow(678, 105, 702, 105)
    a += text(708, 18, '教材2のあとの課題管理表', 14, ACCENT, 700)
    a += _mini_table(708, 30, rows_a, ACCENT, True)
    a += check(726, 208, ACCENT, .45) + text(744, 213, '工数と影響が入り、優先順位に裏付けが付く', 13, ACCENT, 700)
    # 下：言葉の意味を統一すると、よいこと
    a += text(0, 262, '意味を合わせ、必要な記録もAIに入力すると', 15, DIM, 700)
    goods = [('robot', ['業務ごとの言葉の意味に', '合っているか確認できる']),
             ('table', ['工数と遅れの影響を', '優先順位の判断に使える']),
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
    return svg('教材1の課題管理表は、工数と遅延時の影響が空で、優先順位を付ける精度に届かない。業務ごとの言葉の意味と違いを定義すると、表が埋まり、チームでも話が通じる', a, 358)


def i0_next3():
    """教材1と教材2で決めたことを、Skill ・ MCP ・ Hook ・ エージェントの4つの部品で仕組みにする。どれも特定のAIツールに依存しない。"""
    # 左：教材1と教材2で決めたこと
    a = text(0, 18, '教材1 ・ 2で決めたこと', 14, DIM, 700)
    a += rect(0, 30, 300, 292, PAPER, 12, INK)
    decided = [('作業の目的', '優先順位を付けるために整理する'),
               ('言葉の意味', '「課題」が何を指すか'),
               ('対象の範囲', '今回の案件、12月末の切り替えまで'),
               ('必ず入れる条件', '誰が決めるか ・ 期限 ・ 工数 ・ 影響')]
    ys = [76, 146, 216, 286]
    for (k_, v_), y in zip(decided, ys):
        a += text(20, y - 14, k_, 13, DIM, 700)
        a += text(20, y + 10, v_, 14, INK, 700)
    # 右：4つの部品と、それぞれの目的。決めたことはSkillに書く
    parts = [('doc', 'Skill', '作業の手順と知識をまとめ、その作業のときに読み込ませる', (0, 1, 2, 3)),
             ('branch', 'MCP', 'AIを外のデータやツールにつなぐ共通の決まり。使える情報が広がる', ()),
             ('check', 'Hook', '決まった出来事が起きたら、呼び出さなくても毎回自動で動く', ()),
             ('robot', 'エージェント', '業務の目的を受け持ち、どのSkillをいつ使うかを決める', ())]
    for k, (ic, name, what, src) in enumerate(parts):
        y = 30 + k * 76
        hot = name == 'エージェント'
        for s_ in src:
            a += path(f'M300 {ys[s_]} C 340 {ys[s_]}, 350 {y + 32}, 390 {y + 32}', LINE, 1.5)
        a += rect(394, y, 718, 64, PAPER, 10, ACCENT if hot else LINE)
        if ic == 'robot':
            a += _robot_at(412, y + 8, ACCENT)
        else:
            a += icon(414, y + 16 if ic != 'check' else y + 22, ic, INK, .7)
        a += text(484, y + 30, name, 18, ACCENT if hot else INK, 700)
        a += text(484, y + 54, what, 14, INK)
    # 下：どのAIツールでも使える。最後は人が確認して承認する
    a += text(0, 362, '使って見つかったずれを直す人を決め、改善し続ける', 16, INK, 700)
    a += _table_icon(620, 340, INK) + person(684, 356, .45, INK)
    a += text(708, 362, '出力された課題管理表は、PMが確認して承認する', 14, DIM)
    return svg('教材1と教材2で決めたことを、Skill ・ MCP ・ Hook ・ エージェントの4つの部品で仕組みにする。どれも特定のAIツールに依存しない', a, 380)


def i0_journey():
    """教材を1つ終えるごとに、AIでできることが1つずつ増える。段を上がるごとに、課題管理表と仕組みが育つ。"""
    steps = [('教材1　目的の把握', 'search', '議事録から、担当と期限を含む', '課題管理表を作る', '別の案件では、見る案件と時期を変える', 120),
             ('教材2　言葉の定義', 'book', '＋ 工数と影響まで入り、', '優先順位に裏付けが付く', '言葉の意味を言語化し、チームで合う', 60),
             ('教材3　仕組みの構築', 'robot', '＋ 使いながら改善でき、', '共通の基準で使える', 'エージェントがSkillを使い分け、業務を支える', 0)]
    a = ''
    base = 320
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
    a += rect(0, 340, W, 52, PAPER, 12, LINE)
    a += text(W / 2, 373, '言葉の意味を言語化せずに仕組みだけ作ると、使う人で結果が変わる。合わなければ前に戻って直す', 18, INK, 700, 'middle')
    return svg('教材を1つ終えるごとに、AIでできることが1つずつ増える。教材2を飛ばして道具だけ使うと、仕組みは作った人に依存する', a, 396)


def _bubble_r(x, y, w, h, tail_y):
    """右に尾を持つ吹き出し。右側の人の発言を示す。"""
    return (rect(x, y, w, h, PAPER, 12, LINE)
            + path(f'M{x + w - 1} {tail_y - 8} L{x + w + 14} {tail_y} L{x + w - 1} {tail_y + 8}', LINE, 2, PAPER))


def i0_base():
    """目的を言語化し、AIで仕組みを作り、目的を達成する。3つを、自分が実現したい目的で進める。"""
    a = ''
    cards = [('doc', '1　目的を言語化する', '準備期間 ・ 参加するとき',
              ['何を実現するか ・ 誰のためかを書く', '何が変われば達成かを決める', '達成に何が必要かを考える'], False),
             ('robot', '2　AIで仕組みを作る', 'ワークショップの3か月',
              ['Skillとエージェントで作る', '使う人に合わせて使う', 'うまくいかない所を直す'], False),
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
    cols = [('PM ・ PL', '案件を止めずに進める', ['課題を整理する', '進捗報告を作る', '遅れを見つける']),
            ('開発', '仕様を満たし、品質を確保する', ['仕様をまとめる', 'コードをレビューする', 'テスト項目を作る']),
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
    a += path('M176 260 V278 M556 260 V278 M936 260 V278 M176 278 H936 M556 278 V296 m-7 -7 l7 7 7 -7', LINE, 2)
    a += rect(0, 300, W, 44, PAPER, 12, ACCENT)
    a += check(30, 322, ACCENT, .5) + text(54, 329, '目的と基準を共有し、AIの出力を確認して使う', 18, ACCENT, 700)
    for k, e in enumerate(['手直しと依頼し直しが減り、判断に時間を使える', '担当が替わっても、同じ基準で確認できる']):
        x = k * 566
        a += rect(x, 354, 546, 38, PAPER, 10, LINE)
        a += text(x + 20, 378, 'その結果', 12, DIM, 700) + text(x + 100, 379, e, 15, INK, 700)
    return svg('業務ごとにエージェントがあり、その下に作業ごとのSkillが並ぶ。これができると、目的と基準を共有し、AIの出力を確認して使う。その結果、手直しが減り、人が替わっても同じ品質で続く', a, 396)


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
             (834, 116, '使って直す', 'middle'),
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
    a += text(596, 306, '目的ごとに、自分で使うか、チームで使うかを決める', 14, DIM)
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
    """作業の目的と言葉の意味を言語化し、仕組みにして改善する。3つの方法を教材で1つずつ学び、目的をAIで達成する。"""
    steps = [('教材1　目的の把握', 'search', '作業の目的を言語化する', '目的と必要な内容をAIに伝える',
              ['例：優先順位を付けるために、', '課題を整理する']),
             ('教材2　言葉の定義', 'book', '言葉の意味を言語化する', '人や業務で違う意味を書き、合わせる',
              ['例：「課題」が経理 ・ 開発 ・ PMで', '何を指すかを言葉にする']),
             ('教材3　仕組みの構築', 'robot', '仕組みにして改善する', '目的と言葉の意味を仕組みに入れる',
              ['例：使いながらずれを直し、', '課題管理表を確認して改善する'])]
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
    return svg('目的を定め、言葉の意味を定義し、仕組みにする。3つを教材で1つずつ身につけ、目的をAIで達成する', a, 324)


def i1_bridge():
    """もう決まっている4つ（誰が ・ 何のため ・ 何を入力する ・ 何を出力する）。今週の定例の議事録を入力し、何のためかを書かずに「課題を整理して」とだけ頼む。出力される課題管理表は、実行するたびに違う。"""
    # 上：この作業で、もう決まっている4つ
    a = rect(0, 0, 1112, 50, PAPER, 10, LINE)
    a += text(20, 31, '決まっていること', 13, DIM, 700)
    for x, k, v in [(176, '誰が', 'PM'), (320, '何のため', '優先順位を付けるため'), (640, '何を入力する', '定例の議事録'), (880, '何を出力する', '課題管理表')]:
        a += text(x, 31, k, 13, DIM, 700) + text(x + 13 * len(k) + 10, 31, v, 15, INK, 700)
    Y = 70   # 下の段は、上の段の下から始める（絶対座標で描く ── 図の検査は transform を見ない）
    # 左：入力するもの。議事録の中身は、意味を決める動画の1枚目の原稿と同じにする
    a += text(0, Y + 20, '入力するもの', 15, DIM)
    a += path(f'M0 {Y + 32} H420 L452 {Y + 64} V{Y + 300} H0 Z', LINE, 2, PAPER)
    a += path(f'M420 {Y + 32} V{Y + 64} H452', LINE, 2)
    a += text(24, Y + 64, '定例の議事録　今週の1回分', 17, INK, 700)
    a += text(24, Y + 88, '出席　経理 ・ 開発 ・ PM', 13, DIM)
    a += path(f'M24 {Y + 102} H428', LINE, 1)
    for i, line in enumerate(['承認の経路をどうするか', '差し戻しの扱いをどうするか', '申請の締め日をいつにするか',
                              '旧システムをいつ止めるか', '既存データをどう移すか', '月次の締めの進め方',
                              '試験環境の準備']):
        y = Y + 130 + i * 24
        a += circle(30, y - 5, 2.5, INK, INK)
        a += text(42, y, line, 14, INK)
    # 中央：プロンプトとAI。左の議事録の右端（452）と右の枠の左端（812）の中間（632）を中心に置く
    a += arrow(462, Y + 152, 594, Y + 152)
    a += text(490, Y + 20, 'プロンプト', 15, DIM)
    a += rect(490, Y + 32, 284, 60, PAPER, 12, LINE)
    a += text(632, Y + 58, '「課題を整理して」', 20, INK, 700, 'middle')
    a += text(632, Y + 82, '何のためかは書いていない', 14, DIM, anchor='middle')
    a += path(f'M632 {Y + 92} V{Y + 118} M626 {Y + 112} l6 7 6 -7', LINE, 2)
    a += _robot_at(608, Y + 126, INK)
    a += text(632, Y + 206, 'AI', 13, DIM, 700, 'middle')
    a += arrow(670, Y + 152, 804, Y + 152)
    # 右：出力
    a += text(812, Y + 20, '出力', 15, DIM)
    a += rect(812, Y + 32, 300, 268, PAPER, 12, LINE)
    a += _table_icon(836, Y + 56)
    a += text(878, Y + 78, '課題管理表', 17, INK, 700)
    a += path(f'M836 {Y + 110} H1088', LINE, 1)
    a += cross(846, Y + 146, ACCENT, .7)
    a += text(868, Y + 152, '実行するたびに違う', 18, ACCENT, 700)
    a += text(836, Y + 190, ['何度か実行すると、', '並び順が毎回変わる'], 15, INK, gap=24)
    a += text(836, Y + 270, '3回の中身は、次の枚で比べる', 13, DIM)
    return svg('もう決まっている4つ（誰が・何のため・何を入力する・何を出力する）を上に置く。今週の定例の議事録を入力し、何のためかを書かずに課題を整理してとだけ頼むと、出力される課題管理表は実行するたびに違う', a, 378)


def i1_goal():
    """目的未記載の文と、業務目的・作業目的・個別指定を比較する。今回の作業に合う目的を示す。"""
    # 最初の文の不足は、目的の大きさと分けて示す
    a = rect(0, 0, W, 64, PANEL, 12)
    a += text(24, 26, 'よく書くプロンプト', 13, DIM, 700)
    a += text(24, 50, '「課題を整理して」', 18, INK, 700)
    a += text(236, 50, '何をするかは書いてあるが、誰が ・ 何のために整理するかが書いていない', 15, INK)
    # 業務目的・作業目的・個別指定
    a += text(100, 98, '目的レベル', 13, DIM, 700) + text(624, 98, '今回の課題整理との関係', 13, DIM, 700)
    a += text(0, 132, 'なぜ', 13, DIM, 700) + text(0, 300, 'どのように', 13, DIM, 700)
    a += path('M80 302 V112 m-6 6 l6 -6 6 6 M74 296 l6 6 6 -6', LINE, 2)
    rows = [('業務の目的', '本番の切り替えまで、案件を止めずに進める', '何か月もかかる',
             None, '課題整理以外の作業も含む、大きな目的'),
            ('作業の目的', 'どれから決めるか、優先順位を付けるために、課題を整理する', 'PMが定例のあと1回で終え、次の作業へ移れる',
             ('「PMが、どれから決めるか優先順位を付けるために、', '課題を整理する」 ── この大きさで伝える'), None),
            ('細かすぎるプロンプト', 'どの項目を表に入れるか', 'その週の項目を名指ししてしまう',
             ('「承認の経路と申請の締め日を、課題として表に入れて」', 'この議事録にしか使えない'), None)]
    for i, (level, purpose, note_, box, plain) in enumerate(rows):
        y = 110 + i * 70
        cy = y + 30
        hot = i == 1
        a += circle(80, cy, 6, INK if hot else PAPER, INK if hot else LINE) + path(f'M86 {cy} H100', LINE, 1)
        a += rect(100, y, 492, 60, PAPER, 10, LINE)
        a += text(120, y + 22, level, 13, DIM, 700) + text(592 - 16, y + 22, note_, 12, DIM, anchor='end')
        a += text(120, y + 47, purpose, 15, INK, 700)
        a += path(f'M592 {cy} H624', LINE, 1)
        if box:
            a += rect(624, y, 488, 60, PAPER, 10, ACCENT if hot else LINE)
            a += text(644, y + 25, box[0], 14, ACCENT if hot else INK, 700) + text(644, y + 47, box[1], 13, INK if hot else DIM, 700 if hot else 400)
        else:
            a += text(644, cy + 5, plain, 14, DIM)
    a += text(0, 344, '今回の作業に合う大きさか：PMが1人で一度に終え、次の作業へ移れるか ／ 別の週の議事録にも使えるか', 14, INK)
    return svg('「課題を整理して」は、誰が何のために整理するかが書いていない。目的レベルには、業務の目的 ・ 作業の目的 ・ 細かすぎるプロンプトがある。AIには、作業の目的を伝える', a, 356)


def m1_task():
    """作業の目的と、意味・範囲・条件で決めて伝える内容を示す。下に、教材1で学ぶ3つを置く。"""
    heads = ['誰が何に使うか', '課題と呼ぶもの', '見る案件・時期', '各課題に入れる内容']
    rows = [('原因を知る', '「課題を整理して」', 0, '必要な内容が出力に入らない'),
            ('意味を決める', '作業の目的', 1, ''),
            ('意味を決める', '＋意味', 2, ''),
            ('範囲を決める', '＋範囲', 3, ''),
            ('条件を決める', '＋条件', 4, '')]
    a = text(0, 14, '動画', 13, DIM, 700) + text(130, 14, 'AIに伝える内容', 13, DIM, 700) + text(500, 14, '目的・意味・範囲・条件で決めて伝えること', 13, DIM, 700)
    cw = 148
    for k, h in enumerate(heads):
        a += text(500 + k * (cw + 4) + cw / 2, 38, h, 13, DIM, 700, 'middle')
    for i, (video, wrote, n, note_) in enumerate(rows):
        y = 48 + i * 52
        last = i == 4
        a += rect(0, y, W, 44, PAPER, 10, ACCENT if last else LINE)
        a += text(20, y + 28, video, 13, DIM, 700)
        a += text(130, y + 28, wrote, 16, INK, 700)
        for k in range(4):
            x = 500 + k * (cw + 4)
            on = k < n
            a += rect(x, y + 8, cw, 28, MARK if on else PANEL, 6)
            if on:
                a += text(x + cw / 2, y + 27, '決める', 13, INK, 700, 'middle')
        if note_:
            a += text(300, y + 28, note_, 13, DIM)
    a += text(0, 330, '決めすぎない：その週の項目まで名指しすると、細かすぎるプロンプトになり、その議事録にしか使えない', 14, DIM)
    a += text(0, 366, '教材1で学ぶこと', 13, DIM, 700)
    for i, t in enumerate(['出力が、目的からずれる原因を知る', '作業の目的を伝え、意味 ・ 範囲 ・ 条件を決めて伝える', '議事録ごとに変わる中身までは決めない']):
        x = [120, 420, 860][i]
        a += text(x, 366, t, 14, INK, 700)
    return svg('作業の目的を伝え、意味 ・ 範囲 ・ 条件を1つずつ決める。誰が何に使うか・課題と呼ぶもの・見る案件と時期・各課題に入れる内容を決める', a, 378)


def m1_map():
    a = ''
    books = [('原因を知る','2つの入力を比べる'),('意味を決める','何を課題と呼ぶか'),('範囲を決める','どの案件・期間か'),('条件を決める','各課題に何が必要か'),('具体と抽象を振り返る','共通と個別を見る')]
    for i,(name,sub) in enumerate(books):
        x=i*226
        a += rect(x,0,208,134,PAPER,10,LINE)
        a += text(x+20,30,str(i+1),16,DIM,700)
        a += text(x+20,70,name,17,INK,700)
        a += text(x+20,109,sub,16,DIM)
        if i<4:a+=arrow(x+208,67,x+226,67)
    a += rect(0,169,886,94,PAPER,12,LINE)
    a += text(24,202,'目的に合う内容を考える',20,INK,700)
    a += text(24,242,'個別の不足を見る → 利用目的を確かめる → 共通の定義・条件を決める',19,INK)
    a += rect(904,169,208,94,PAPER,12,ACCENT)
    a += text(924,202,'考え方を振り返る',18,ACCENT,700)
    a += text(924,242,'次の教材へ進む',17,INK)
    return svg('本編5本。原因を知り、意味・範囲・条件を決め、具体と抽象の考え方を振り返る',a,274)


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
    x0, x1 = 470, 1050            # 9/1 と 12/31 の位置
    def at(m, d):                  # 月日を横位置へ
        days = {9: 0, 10: 30, 11: 61, 12: 91}[m] + d - 1
        return round(x0 + (x1 - x0) * days / 121)
    a += text(410, 22, 'PMが見ているもの', 17, DIM)
    a += rect(410, 34, 702, 290, PAPER, 12, ACCENT)
    a += person(442, 70, .6, INK)
    a += text(468, 66, 'PMにとっての課題', 13, DIM)
    a += text(468, 88, '本番の切り替えまでに、進行が止まるかどうか', 17, ACCENT, 700)
    for m, lab in [(9, '9月'), (10, '10月'), (11, '11月'), (12, '12月')]:
        a += text(at(m, 1) + 6, 116, lab, 13, DIM)
        a += path(f'M{at(m, 1)} 104 V310', PANEL, 1.5)
    a += path(f'M{x1} 104 V310', ACCENT, 2)
    a += text(x1 - 8, 304, '12月末 本番切り替え', 13, ACCENT, 700, 'end')
    # 行の中心を、左の吹き出しの中心と同じ高さに置く
    rows = [('運用の準備と確認', (9, 30), (12, 20), '②', '9/30までに、経理が申請の締め日を決める', 128),
            ('連携の開発', (9, 16), (12, 10), '①', '9/16までに、開発が連携の仕様を決める', 240)]
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
    a += text(110, 276, 'PMだけが基準を知っている', 14, DIM, anchor='middle')
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
    a += text(622, 144, '整理の目的を言語化する', 15, INK)
    a += text(822, 144, '教材1', 13, DIM, 700, 'end')
    a += text(622, 182, '言葉の意味を言語化する', 15, INK)
    a += text(822, 182, '教材2', 13, DIM, 700, 'end')
    a += path('M622 210 H822', LINE, 1)
    a += text(622, 240, '仕組みにして改善する', 14, DIM)
    a += text(822, 240, '教材3', 13, DIM, 700, 'end')
    for y in RY:
        a += _ray(848, 165, 960, y, INK, 2)
    a += receivers(1000)
    for y in RY:
        a += check(1086, y, ACCENT, .7)
    a += band(318, '頭の中の基準を言葉にしてAIに伝え、出力を確認する')
    return svg('いまは基準がPMの頭の中にしかなく、補佐にも後任にもAIにも届かない。言葉にすれば届く', a, 384)


def i0_books():
    """土台は具体と抽象の考え方。ユースケース実践とドメイン駆動設計は、その上で教材1と教材2に使う。"""
    from visuals import cover, BOOKS
    tops = [(BOOKS[1], '教材1で使う', ['目的を大きさで分けて、', '何を実現したいかを書く']),
            (BOOKS[2], '教材2で使う', ['言葉の意味を定義し、', '違いも言葉にする'])]
    a = ''
    for k, ((isbn, title, author, pub), use, note) in enumerate(tops):
        x = k * 572
        a += cover(x + 20, 0, 112, 160, isbn)
        a += text(x + 156, 26, ''.join(title), 16, INK, 700)
        a += text(x + 156, 50, f'{author}（{pub}）', 13, DIM)
        a += rect(x + 156, 66, 372, 94, PAPER, 10, LINE)
        a += text(x + 176, 92, use, 13, DIM, 700)
        a += text(x + 176, 118, note, 14, INK, gap=22)
    isbn, title, author, pub = BOOKS[0]
    a += rect(0, 186, W, 200, PAPER, 12, ACCENT)
    a += cover(20, 206, 112, 160, isbn)
    a += text(156, 228, '土台', 14, ACCENT, 700)
    a += text(156, 258, ''.join(title), 18, INK, 700)
    a += text(156, 284, f'{author}（{pub}）', 13, DIM)
    a += text(156, 326, '共通の決まりと個別の内容を比べ、目的と手段を行き来する', 16, INK, 700)
    a += text(156, 354, '教材1 ・ 教材2 ・ 教材3は、どれもこの考え方をもとにしている', 14, DIM)
    return svg('土台はシステム開発と「具体と抽象」の考え方。ユースケース実践ガイドは教材1、ドメイン駆動設計をはじめようは教材2で、その上に使う', a, 396)


def i1_target():
    """目指すのは、目的レベルの両端のあいだにある、作業の目的で書いたプロンプト。上のプロンプトのよさ（どの議事録にも使える）と、下のプロンプトのよさ（出力が決まる）を、両方持てる。"""
    a = text(100, 18, 'プロンプト', 14, DIM) + text(632, 18, '使える議事録', 14, DIM) + text(880, 18, '出力', 14, DIM)
    a += text(0, 52, '抽象', 18, INK, 700) + text(0, 72, 'なぜ', 13, DIM)
    a += text(0, 268, '具体', 18, INK, 700) + text(0, 290, 'どのように', 13, DIM)
    a += path('M80 296 V30 m-6 6 l6 -6 6 6', LINE, 2)
    rows = [('作業の目的が書かれていないプロンプト', '「課題を整理して」', ('どの週の議事録にも使える', True), ('実行するたびに変わる', False), False),
            ('作業の目的で書いたプロンプト', '？　何を書けばこのプロンプトになるか', ('どの週の議事録にも使える', True), ('目的から外れない', True), True),
            ('細かすぎるプロンプト', '「承認の経路と申請の締め日を、課題として表に入れて」', ('この議事録にしか使えない', False), ('1つに決まる', True), False)]
    for i, (kind, instr, use, ret, hot) in enumerate(rows):
        y = 32 + i * 90
        cy = y + 36
        a += circle(80, cy, 7 if hot else 6, ACCENT if hot else PAPER, ACCENT if hot else LINE) + path(f'M87 {cy} H100', LINE, 1)
        box = rect(100, y, 496, 72, PAPER, 10, ACCENT if hot else LINE)
        a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 5"') if hot else box
        a += text(120, y + 28, kind, 13, ACCENT if hot else DIM, 700) + text(120, y + 54, instr, 16, ACCENT if hot else INK, 700)
        a += rect(612, y, 496, 72, PAPER, 10, LINE) + path(f'M860 {y + 14} V{y + 58}', LINE, 1)
        for x, (t, good) in ((612, use), (860, ret)):
            a += (check(x + 24, cy, ACCENT, .5) if good else cross(x + 24, cy, DIM, .6))
            a += text(x + 46, cy + 6, t, 15, INK if good else DIM, 700 if good else 400)
    return svg('目指すのは、目的レベルの両端のあいだにある、作業の目的で書いたプロンプトである。作業の目的が書かれていないプロンプトはどの議事録にも使えるが実行するたびに変わり、細かすぎるプロンプトは1つに決まるがこの議事録にしか使えない。作業の目的で書いたプロンプトは、どの議事録にも使え、目的から外れない', a, 304)
