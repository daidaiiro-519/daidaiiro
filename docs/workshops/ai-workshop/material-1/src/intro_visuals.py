"""はじめに（全教材に共通する前置き）の図。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)


def i0_case():
    """1つの案件に、3つの業務が関わっている。"""
    a = text(0, 24, '題材の案件', 18, DIM)
    a += rect(356, 44, 400, 84, ACCENT, 12, ACCENT)
    a += text(556, 82, '経費精算システムを入れ替える', 21, PAPER, 700, 'middle')
    a += text(556, 110, '本番の切り替えは3か月後', 15, PAPER, anchor='middle')
    roles = [('経費の精算業務', '経理が、日々の精算と支払を担当している'),
             ('案件の進行管理', 'PMが、案件を止めずに本番まで進める'),
             ('システムの開発', '開発が、画面と連携を作っている')]
    for i, (name, what) in enumerate(roles):
        x = i * 384
        a += path(f'M556 128 V152 H{x + 172} V176', LINE)
        a += path(f'M{x + 172 - 6} 170 l6 8 6 -8', LINE, 2)
        a += rect(x, 182, 344, 96, PAPER, 12, LINE)
        a += person(x + 44, 222, .62, DIM)
        a += text(x + 80, 226, name, 19, INK, 700)
        a += text(x + 24, 262, what, 16, INK)
    # 3つの業務の担当者が定例に集まり、書記が議事録にまとめる。流れを線でたどれるようにする
    # 定例と議事録の組を、図の中央に置く
    for i in range(3):
        cx = i * 384 + 172
        a += path(f'M{cx} 278 V300', LINE)
    a += path('M172 300 H940 M370 300 V316 M364 310 l6 8 6 -8', LINE, 2)
    a += rect(170, 322, 400, 72, PANEL, 12)
    a += icon(194, 336, 'chat', INK, .8)
    a += text(254, 352, '週に1回の定例', 18, INK, 700)
    a += text(254, 378, '3つの業務の担当者が集まる', 15, INK)
    a += arrow(580, 358, 626, 358)
    a += rect(634, 322, 308, 72, PAPER, 12, LINE)
    a += icon(656, 336, 'doc', INK, .8)
    a += text(712, 352, '議事録', 18, INK, 700)
    a += text(712, 378, '書記が、決定と未決をまとめる', 15, INK)
    return svg('1つの案件に3つの業務が関わり、担当者が定例に集まって、書記が議事録にまとめる', a, 402)


def i0_usecases():
    """業務は作業でできていて、前の作業の結果を次の作業が受け取る。経理にも開発にも、それぞれの作業がある。"""
    a = text(0, 22, 'PMの業務', 17, DIM)
    a += rect(0, 34, W, 190, PAPER, 12, ACCENT)
    a += text(24, 68, '案件の進行管理', 20, INK, 700)
    a += text(186, 68, '目的　本番の切り替えまで、案件を止めずに進める', 15, DIM)
    tasks = ['課題を整理する', 'スケジュールを引き直す', '工数を見積もる', '要員を調整する']
    passed = ['課題管理表', '引き直したスケジュール', '見積もった工数']
    for i, name in enumerate(tasks):
        x = 24 + i * 284
        a += rect(x, 92, 212, 60, PANEL, 10)
        a += text(x + 106, 128, name, 17, INK, 700, 'middle')
        if i < 3:
            gx = x + 212 + 36          # 作業のあいだの中心
            a += arrow(x + 218, 122, x + 278, 122)
            # 次の作業へ渡るもの
            a += path(f'M{gx} 130 V170', LINE, 1.5)
            w = 176
            a += rect(gx - w / 2, 172, w, 30, PAPER, 15, LINE)
            a += text(gx, 193, passed[i], 14, INK, anchor='middle')
    a += text(0, 258, 'ほかの業務も、同じように作業でできている', 17, DIM)
    others = [('経費の精算業務', '経理', ['申請を受け付ける', '内容を確認する', '支払を処理する']),
              ('システムの開発', '開発', ['仕様を決める', '画面を作る', '連携を試験する'])]
    for k, (name, who, ts) in enumerate(others):
        x0 = k * 572
        a += rect(x0, 270, 540, 102, PAPER, 12, LINE)
        a += text(x0 + 20, 300, name, 16, INK, 700)
        a += text(x0 + 20 + len(name) * 16 + 12, 300, who, 14, DIM)
        for j, t in enumerate(ts):
            x = x0 + 20 + j * 172
            a += rect(x, 318, 148, 36, PANEL, 8)
            a += text(x + 74, 341, t, 14, INK, anchor='middle')
            if j < 2:
                a += arrow(x + 150, 336, x + 170, 336)
    return svg('業務は作業でできていて、前の作業の結果を次の作業が受け取る。経理にも開発にも作業がある', a, 380)


def _table_icon(x, y, color=INK):
    """表の形。課題管理表を示す。"""
    a = rect(x, y, 30, 32, PAPER, 3, color)
    a += path(f'M{x} {y + 10} H{x + 30} M{x} {y + 21} H{x + 30} M{x + 11} {y + 10} V{y + 32}', color, 1.5)
    return a


def i0_usecase():
    """4つの作業から、課題を整理するを選ぶ。作業まで絞ると、目的が指示に書ける大きさになり、AIに頼める。"""
    # 上：前の枚の作業の並びから、1つを選ぶ
    a = text(0, 20, 'PMの業務の作業', 15, DIM)
    for i, name in enumerate(['課題を整理する', 'スケジュールを引き直す', '工数を見積もる', '要員を調整する']):
        x = i * 236
        hot = (i == 0)
        a += rect(x, 32, 200, 36, ACCENT if hot else PANEL, 8)
        a += text(x + 100, 56, name, 15, PAPER if hot else DIM, 700 if hot else 400, 'middle')
        if i < 3:
            a += arrow(x + 204, 50, x + 232, 50)
    a += path('M100 68 V100', ACCENT, 2)
    # 左：選んだ作業の4つの項目
    a += rect(0, 100, 520, 250, PAPER, 12, ACCENT)
    a += text(24, 140, '課題を整理する', 22, INK, 700)
    a += path('M24 158 H496', LINE, 1)
    for i, (k, v) in enumerate([('誰が', 'PM'), ('何のために', 'どの課題から決めるか、優先順位を付ける'),
                                ('渡すもの', '定例の議事録'), ('返すもの', '課題管理表')]):
        y = 200 + i * 40
        hot = (k == '何のために')   # 目的の行。教材1が扱う中心である
        a += text(24, y, k, 15, ACCENT if hot else DIM, 700)
        a += text(140, y, v, 16, INK, 700 if hot else 400)
    # 右：AIへの頼み方の対比。上下の枠は同じ高さ ・ 同じ地にし、入力 ・ AI ・ 出力 ・ 結果を同じ位置に置く
    a += text(560, 96, 'AIに頼むとき', 15, DIM)
    for k, (title, bad) in enumerate([('業務の目的のまま', True), ('作業の目的にすると', False)]):
        y = 108 + k * 126
        a += rect(560, y, 552, 116, PAPER, 12, LINE)
        a += text(580, y + 28, title, 14, INK, 700)
        if bad:
            a += text(580, y + 66, '「本番まで案件を止めずに進めて」', 15, INK)
            a += arrow(820, y + 60, 850, y + 60)
        else:
            a += icon(584, y + 40, 'doc', INK, .6)
            a += text(620, y + 66, '議事録', 15, INK)
            a += arrow(676, y + 60, 850, y + 60)
        a += _robot_at(866, y + 36, INK)
        a += arrow(928, y + 60, 958, y + 60)
        if bad:
            a += text(990, y + 70, '？', 26, DIM, 700, 'middle')
            a += cross(592, y + 96, DIM, .6)
            a += text(608, y + 102, '返すものが決まらない', 14, DIM)
        else:
            a += _table_icon(970, y + 44)
            a += text(1010, y + 66, '課題管理表', 15, INK)
            a += check(592, y + 96, ACCENT, .6)
            a += text(608, y + 102, '返ってきた表で、そのまま優先順位を付けられるかで判断できる', 14, INK)
    return svg('4つの作業から課題を整理するを選ぶ。作業まで絞ると、目的が指示に書ける大きさになり、AIに頼める', a, 360)


def i0_next2():
    """教材1の課題管理表は、工数と遅延時の影響が空いている。各業務の言葉で読むと埋まり、期限と優先順位に裏付けが付く。"""
    a = text(0, 20, '教材1でAIが返す課題管理表', 15, DIM)
    cols = [('課題', 0), ('決める担当', 176), ('期限', 290), ('工数', 380), ('遅延時の影響', 540)]
    a += rect(0, 32, 712, 168, PAPER, 10, LINE)
    a += rect(0, 32, 712, 40, PANEL, 10)
    for name, x in cols:
        a += text(x + 18, 58, name, 14, DIM, 700)
    # 行の中心を、右のカードの中心と同じ高さに置く
    rows = [('連携の仕様', '開発', '9/16', '実装が進まない要因'), ('申請の締め日', '経理', '9/30', '日々の精算が止まる要因')]
    for i, (k, who, due, mean) in enumerate(rows):
        y = 72 + i * 64
        cy = y + 32
        if i:
            a += path(f'M0 {y} H712', LINE, 1)
        a += text(18, cy + 6, k, 16, INK, 700)
        a += text(194, cy + 6, who, 16, INK)
        a += text(308, cy + 6, due, 16, INK)
        for x, w in ((380, 136), (540, 144)):
            a += rect(x + 12, cy - 16, w, 32, PAPER, 6, ACCENT)
            a += text(x + 12 + w / 2, cy + 7, '？', 17, ACCENT, 700, 'middle')
        a += rect(772, cy - 29, 340, 58, PANEL, 10)
        a += person(802, cy + 2, .55, INK)
        a += text(832, cy - 5, f'{who}にとっての課題', 13, DIM, 700)
        a += text(832, cy + 17, mean, 16, INK, 700)
        a += path(f'M766 {cy} H720 M728 {cy - 7} L720 {cy} L728 {cy + 7}', LINE, 2)
    a += text(772, 62, 'その業務の言葉で、AIが読む', 15, DIM)
    a += text(0, 226, '期限は、定例で各業務が言った日付を写しただけで、裏付けが無い', 14, DIM)
    a += text(772, 226, '分かると、2つの列が埋まる', 14, DIM)
    a += band(252, '言葉が何を指すかを指示に書くと、期限と優先順位に裏付けが付く')
    return svg('教材1の課題管理表は工数と遅延時の影響が空いている。各業務の言葉で読むと埋まり、期限と優先順位に裏付けが付く', a, 318)


def i0_next3():
    """言葉にしたことを、道具に置いておく。誰が使っても同じ基準で動き、最後は人が確認して承認する。"""
    a = text(0, 20, '教材1・2で言葉にしたもの', 15, DIM)
    a += text(262, 20, '置いておく先', 15, DIM)
    rows = [('意味', '何を並べるか', 'Skill', '決めた意味を、呼ぶたびに同じに読み込ませる'),
            ('範囲', 'どこから拾うか', 'MCP', '見に行く情報源を、決めた範囲に限定する'),
            ('条件', '必ず入れるもの', 'Hook', '必ず入っているかを、機械で確認する'),
            ('目的ごとの依頼', '目的が2つなら分ける', 'Agent', '分けた目的を、別々の担当に割り当てる')]
    for i, (k, sub, tool, what) in enumerate(rows):
        y = 32 + i * 62
        a += rect(0, y, 222, 52, PANEL, 8)
        a += text(16, y + 23, k, 15, INK, 700)
        a += text(16, y + 43, sub, 13, DIM)
        a += arrow(226, y + 26, 256, y + 26)
        a += rect(262, y, 440, 52, PAPER, 8, LINE)
        a += text(282, y + 32, tool, 17, INK, 700)
        a += text(356, y + 32, what, 14, INK)
    # 右：仕組みの流れを上から下へ。最後は人が確認して承認する
    a += text(760, 20, '仕組みの流れ', 15, DIM)
    a += rect(760, 32, 352, 242, PAPER, 12, ACCENT)
    a += icon(788, 48, 'doc', INK, .5)
    a += text(846, 68, '議事録', 15, INK)
    a += path('M800 80 V90 M795 85 l5 5 5 -5', LINE, 2)
    a += _robot_at(776, 94, INK)
    a += text(846, 126, '仕組み', 15, INK, 700)
    a += text(902, 126, 'Skill ・ MCP ・ Hook ・ Agent', 13, DIM)
    a += path('M800 150 V160 M795 155 l5 5 5 -5', LINE, 2)
    a += _table_icon(785, 164)
    a += text(846, 186, '課題管理表', 15, INK)
    a += path('M800 200 V212 M795 207 l5 5 5 -5', LINE, 2)
    a += person(800, 240, .55, INK)
    a += text(846, 246, 'PMが確認して承認する', 15, ACCENT, 700)
    a += check(1040, 240, ACCENT, .6)
    a += band(300, '誰が使っても、目的から外れずに課題を整理できる')
    return svg('言葉にしたことを道具に置いておく。誰が使っても同じ基準で動き、最後は人が確認して承認する', a, 366)


def i0_journey():
    """2枚目と同じ3段の階段。段を1つ上がるごとに、AIでできることが1つ増える。"""
    a = text(0, 22, 'この教材を終えると、AIでできること', 16, DIM)
    base, SW = 336, 370
    tops = [236, 166, 96]
    a += path(f'M0 {base} V{tops[0]} H{SW} V{tops[1]} H{2 * SW} V{tops[2]} H{3 * SW} V{base} Z', LINE, 1.5, PANEL)
    a += path(f'M{SW} {tops[0]} V{base} M{2 * SW} {tops[1]} V{base}', LINE, 1)
    # 段の名前と印は、2枚目の階段と同じにする
    steps = [('教材1　目的の把握', ['議事録が替わっても、', '同じ指示で使える']),
             ('教材2　言葉の定義', ['＋ 言葉の意味が違っても、', '　 正しく読み分ける']),
             ('教材3　仕組みの構築', ['＋ 誰が使っても、', '　 同じように動く'])]
    for k, (title, can) in enumerate(steps):
        x0 = k * SW
        top = tops[k]
        cx = x0 + SW / 2
        a += text(x0 + 22, top + 32, title, 17, INK, 700)
        a += text(x0 + 22, top + 62, can, 15, INK, gap=24)
        if k == 0:
            a += icon(cx - 25, top - 58, 'search', INK, 1)
        elif k == 1:
            a += icon(cx - 27, top - 58, 'book', INK, 1)
        else:
            a += _robot_at(cx - 24, top - 60, INK)
    return svg('2枚目と同じ3段の階段。段を1つ上がるごとに、AIでできることが1つ増える', a, base + 8)


def _bubble_r(x, y, w, h, tail_y):
    """右に尾を持つ吹き出し。右側の人の発言を示す。"""
    return (rect(x, y, w, h, PAPER, 12, LINE)
            + path(f'M{x + w - 1} {tail_y - 8} L{x + w + 14} {tail_y} L{x + w - 1} {tail_y + 8}', LINE, 2, PAPER))


def i0_base():
    """目的を言語化し、AIで仕組みを作り、目的を達成する。開始時に書いた達成条件で、終了時に照合する。"""
    T = 40   # 段の名前の行の下から、カードを置く
    a = text(0, 20, '1　目的を言語化する', 16, INK, 700)
    a += text(340, 20, '2　AIで仕組みを作る', 16, INK, 700)
    a += text(812, 20, '3　目的を達成する', 16, INK, 700)
    # 1：開始時に記入する用紙。空欄の線は、受講者が書き込む欄である
    a += rect(0, T, 300, 254, PANEL, 12)
    a += text(24, T + 30, '開始時に記入する', 14, DIM, 700)
    a += rect(20, T + 46, 260, 190, PAPER, 8, LINE)
    a += text(40, T + 76, '実現したい目的', 13, INK, 700)
    a += rect(40, T + 88, 220, 10, LINE, 5)
    a += text(40, T + 130, '達成条件', 13, INK, 700)
    for k, name in enumerate(['結果', '確認の方法', '期限']):
        y = T + 158 + k * 26
        a += text(40, y, name, 13, DIM)
        a += rect(130, y - 9, 130, 8, LINE, 4)
    a += arrow(308, T + 127, 332, T + 127)
    # 2：3つの教材。例は課題の整理だが、作るのは選んだ目的のための仕組み
    for k, (name, what) in enumerate([('教材1　目的の把握', '目的を捉え、指示に書く'),
                                      ('教材2　言葉の定義', '言葉が何を指すかを定める'),
                                      ('教材3　仕組みの構築', '目的から外れない仕組みにする')]):
        y = T + k * 96
        a += rect(340, y, 432, 62, PAPER, 10, LINE)
        a += text(360, y + 37, name, 16, INK, 700)
        a += text(548, y + 37, what, 13, DIM)
    a += arrow(780, T + 127, 804, T + 127)
    # 3：終了時に照合する。開始時の達成条件と実際の結果を並べ、判定する
    a += rect(812, T, 300, 254, PAPER, 12, ACCENT)
    a += text(836, T + 30, '終了時に照合する', 14, DIM, 700)
    a += text(836, T + 74, '達成条件', 13, INK, 700)
    a += rect(836, T + 86, 252, 10, LINE, 5)
    a += text(836, T + 130, '実際の結果', 13, INK, 700)
    a += rect(836, T + 142, 252, 10, LINE, 5)
    a += text(836, T + 186, '判定', 13, INK, 700)
    a += text(836, T + 212, '達成 ／ 一部達成 ／ 未達 ／ 未実施', 13, DIM)
    # 開始時の達成条件を、そのまま終了時の照合に使う
    a += path(f'M150 {T + 254} V{T + 274} H962 V{T + 260}', LINE, 2)
    a += path(f'M956 {T + 266} L962 {T + 258} L968 {T + 266}', LINE, 2)
    a += text(556, T + 296, '開始時に書いた達成条件で、終了時に照合する', 13, DIM, anchor='middle')
    a += band(T + 314, '見るのは、何を作ったかではなく、目的をどれだけ達成できたか')
    return svg('目的を言語化し、AIで仕組みを作り、目的を達成する。開始時に書いた達成条件で、終了時に照合する', a, T + 378)


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
    """いまは、ノウハウが個人の手元に留まっている。3つの段階を上がると、チームで共有できる。"""
    a = text(0, 22, 'いま', 17, DIM)
    a += rect(0, 38, 300, 258, PAPER, 12, LINE)
    a += text(22, 78, ['AIを使うノウハウが、', '個人の手元に留まっている'], 17, INK, 700, gap=26)
    for k, s_ in enumerate(['指示の書き方が人によって違う', '返ってくるものも人によって違う', 'うまくいった指示が共有されない']):
        y = 164 + k * 42
        a += cross(28, y - 6, DIM, .55)
        a += text(52, y, s_, 15, DIM)
    a += arrow(310, 262, 342, 262, LINE)
    a += text(352, 22, 'このワークショップで上がる3つの段階', 17, DIM)
    a += text(1108, 22, '誰が頼んでも、目的をAIで達成できる', 16, ACCENT, 700, 'end')
    base = 296
    tops = [226, 176, 126]
    a += path(f'M352 {base} V{tops[0]} H604 V{tops[1]} H856 V{tops[2]} H1108 V{base} Z', LINE, 1.5, PANEL)
    a += path(f'M604 {tops[0]} V{base} M856 {tops[1]} V{base}', LINE, 1)
    # 3つの段階は、表紙の3つの教材と同じ名前・同じ印にする
    steps = [('教材1　目的の把握', '目的を捉え、指示に書く'),
             ('教材2　言葉の定義', '言葉が何を指すかを定める'),
             ('教材3　仕組みの構築', '目的から外れない仕組みにする')]
    for k, (title, sub) in enumerate(steps):
        x0 = 352 + k * 252
        top = tops[k]
        cx = x0 + 126
        a += text(x0 + 20, top + 30, title, 17, INK, 700)
        a += text(x0 + 20, top + 54, sub, 14, DIM)
        if k == 0:
            a += icon(cx - 25, top - 58, 'search', INK, 1)
        elif k == 1:
            a += icon(cx - 27, top - 58, 'book', INK, 1)
        else:
            a += _robot_at(cx - 24, top - 60, INK)
    a += path('M372 150 L1060 40 M1049 48 L1060 40 L1047 36', ACCENT, 2)
    a += band(318, '3つの段階を上がると、目的をAIで達成する力が身につく')
    return svg('いまはノウハウが個人の手元に留まっている。3つの段階を上がると、チームで共有できる', a, 386)
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
    a += text(622, 144, '整理の目的', 15, INK)
    a += text(822, 144, '教材1', 13, DIM, 700, 'end')
    a += text(622, 182, '言葉が指すもの', 15, INK)
    a += text(822, 182, '教材2', 13, DIM, 700, 'end')
    a += path('M622 210 H822', LINE, 1)
    a += text(622, 240, '誰でも使える形にする', 14, DIM)
    a += text(822, 240, '教材3', 13, DIM, 700, 'end')
    for y in RY:
        a += _ray(848, 165, 960, y, INK, 2)
    a += receivers(1000)
    for y in RY:
        a += check(1086, y, ACCENT, .7)
    a += band(318, '頭の中の基準を言葉にすれば、ほかの人にもAIにも渡せる')
    return svg('いまは基準がPMの頭の中にしかなく、補佐にも後任にもAIにも届かない。言葉にすれば届く', a, 384)
