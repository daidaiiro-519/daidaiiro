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
    """4つの作業から、課題を整理するを選ぶ。作業まで絞ると、渡すものと返すものが決まり、AIに頼める。"""
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
        a += text(24, y, k, 15, DIM, 700)
        a += text(140, y, v, 16, INK)
    # 右：AIへの頼み方の対比
    a += text(560, 96, 'AIに頼むとき', 15, DIM)
    a += rect(560, 108, 552, 98, PANEL, 12)
    a += text(580, 134, '業務のまま', 14, DIM, 700)
    a += text(580, 174, '「本番まで案件を止めずに進めて」', 15, INK)
    a += arrow(820, 168, 850, 168)
    a += _robot_at(866, 144, DIM)
    a += arrow(928, 168, 958, 168)
    a += text(1010, 176, '？', 26, DIM, 700, 'middle')
    a += text(1010, 198, '返すものが決まらない', 13, DIM, anchor='middle')
    a += rect(560, 216, 552, 134, PAPER, 12, LINE)
    a += text(580, 244, '作業にすると', 14, INK, 700)
    a += icon(584, 258, 'doc', INK, .6)
    a += text(620, 284, '議事録', 15, INK)
    a += arrow(676, 278, 706, 278)
    a += _robot_at(722, 254, INK)
    a += arrow(784, 278, 814, 278)
    a += _table_icon(826, 262)
    a += text(866, 284, '課題管理表', 15, INK)
    a += check(592, 330, ACCENT, .6)
    a += text(608, 336, '返ってきた表で、そのまま優先順位を付けられるかで判断できる', 14, INK)
    return svg('4つの作業から課題を整理するを選ぶ。作業まで絞ると、渡すものと返すものが決まり、AIに頼める', a, 360)


def i0_next2():
    """教材1の課題管理表は、手間と影響が空いている。各業務の言葉で読むと埋まり、期限と優先順位に裏付けが付く。"""
    a = text(0, 20, '教材1で作る課題管理表', 15, DIM)
    cols = [('課題', 0), ('決める担当', 196), ('期限', 316), ('手間', 412), ('影響', 562)]
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
        a += text(214, cy + 6, who, 16, INK)
        a += text(334, cy + 6, due, 16, INK)
        for x in (412, 562):
            a += rect(x + 12, cy - 16, 126, 32, PAPER, 6, ACCENT)
            a += text(x + 75, cy + 7, '？', 17, ACCENT, 700, 'middle')
        a += rect(772, cy - 29, 340, 58, PANEL, 10)
        a += person(802, cy + 2, .55, INK)
        a += text(832, cy - 5, f'{who}にとっての課題', 13, DIM, 700)
        a += text(832, cy + 17, mean, 16, INK, 700)
        a += path(f'M766 {cy} H720 M728 {cy - 7} L720 {cy} L728 {cy + 7}', LINE, 2)
    a += text(772, 62, 'その業務の言葉で読む', 15, DIM)
    a += text(0, 226, '期限は、定例で各業務が言った日付を写しただけで、裏付けが無い', 14, DIM)
    a += text(772, 226, '分かると、手間と影響が埋まる', 14, DIM)
    a += band(252, '業務ごとに言葉を揃えると、期限と優先順位に裏付けが付く')
    return svg('教材1の課題管理表は手間と影響が空いている。各業務の言葉で読むと埋まり、期限と優先順位に裏付けが付く', a, 318)


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
    a += band(300, '誰が使っても、同じ基準で課題を整理できる')
    return svg('言葉にしたことを道具に置いておく。誰が使っても同じ基準で動き、最後は人が確認して承認する', a, 366)


def i0_journey():
    """扱う範囲は入れ子で広がる。教材1を教材2が包み、それを教材3が包む。"""
    a = text(0, 20, '扱う範囲　前の教材で決めたことを、次の教材で前提にする', 15, DIM)
    def body(x, y, q, out, hot=False):
        o = text(x, y, '扱う問い', 13, DIM)
        o += text(x, y + 24, q, 15, INK, gap=22)
        yy = y + 24 + 22 * len(q) + 12
        o += text(x, yy, '身につくもの', 13, DIM)
        o += text(x, yy + 24, out, 15, ACCENT if hot else INK, 700, gap=22)
        return o
    # 教材3：いちばん外
    a += rect(0, 32, W, 336, PAPER, 12, LINE)
    a += text(24, 62, '教材3　仕組みの構築', 17, INK, 700)
    a += text(212, 62, 'チームで使う仕組み', 14, DIM)
    a += body(808, 190, ['揃えた言葉を、', '誰が使っても同じに動かせるか'], ['人が確かめて承認できる', '仕組みにする'])
    # 教材2：中
    a += rect(20, 80, 760, 272, PANEL, 12)
    a += text(40, 108, '教材2　業務の整理', 17, INK, 700)
    a += text(212, 108, '案件に関わる業務全体', 14, DIM)
    a += body(444, 190, ['各業務の言う課題は、', '何を指しているか'], ['業務ごとに、言葉が', '指すものを揃える'])
    # 教材1：いちばん内。ここから始めるので際立たせる
    a += rect(40, 126, 384, 210, PAPER, 12, ACCENT)
    a += text(60, 154, '教材1　課題の把握', 17, ACCENT, 700)
    a += text(232, 154, 'PMの業務', 14, DIM)
    a += body(60, 190, ['PMは課題を、', 'どう捉えているか'], ['課題の捉え方を、', '意味・範囲・条件で書く'], True)
    return svg('扱う範囲は入れ子で広がる。教材1を教材2が包み、それを教材3が包む', a, 376)


def _bubble_r(x, y, w, h, tail_y):
    """右に尾を持つ吹き出し。右側の人の発言を示す。"""
    return (rect(x, y, w, h, PAPER, 12, LINE)
            + path(f'M{x + w - 1} {tail_y - 8} L{x + w + 14} {tail_y} L{x + w - 1} {tail_y + 8}', LINE, 2, PAPER))


def i0_base():
    """人は聞き返して意味を確認できるが、AIは聞き返さずに解釈して実行する。だから言葉の意味を先に決めておく。"""
    # 左：人どうし
    a = rect(0, 0, 540, 276, PANEL, 12)
    a += text(24, 34, '人どうし', 16, INK, 700)
    a += person(44, 96, .6, INK)
    a += text(44, 136, 'PM', 13, DIM, 700, 'middle')
    a += _bubble(86, 70, 290, 44, 92)
    a += text(104, 98, '「その課題って、どういう意味？」', 15, INK)
    a += person(496, 172, .6, INK)
    a += text(496, 212, '経理', 13, DIM, 700, 'middle')
    a += _bubble_r(150, 146, 300, 44, 168)
    a += text(168, 174, '「月末の支払が回らない、ということ」', 15, INK)
    a += check(36, 248, INK, .6)
    a += text(56, 254, '聞き返して、意味を確認できる', 16, INK, 700)
    # 右：AI。頼むたびに違う表が返る
    a += rect(572, 0, 540, 276, PAPER, 12, ACCENT)
    a += text(596, 34, 'AI', 16, ACCENT, 700)
    a += person(616, 96, .6, INK)
    a += text(616, 136, 'PM', 13, DIM, 700, 'middle')
    a += _bubble(658, 70, 190, 44, 92)
    a += text(676, 98, '「課題を整理して」', 15, INK)
    a += arrow(856, 92, 896, 92)
    a += _robot_at(914, 66, INK)
    a += text(976, 70, '？', 20, DIM, 700)
    # 原因を知る動画の3回と同じ ── 件数ではなく、並べ方が毎回違う
    for i, n in enumerate(['業務の流れの順', '期限の順', '機能ごと']):
        x = 612 + i * 164
        a += _table_icon(x, 166)
        a += text(x + 38, 180, f'{i + 1}回目', 12, DIM)
        a += text(x + 38, 200, n, 14, INK, 700)
    a += path('M944 124 V144 H627 V160 M621 154 l6 8 6 -8', LINE, 2)
    a += text(596, 254, '聞き返さずに、自分なりに解釈して実行する', 16, ACCENT, 700)
    a += band(298, 'だから、頼む側が言葉の意味を先に決めておく')
    a += text(W / 2, 384, '揃えるのは全社で1つの意味ではなく、それぞれの業務の中での意味', 14, DIM, anchor='middle')
    return svg('人は聞き返して意味を確認できるが、AIは聞き返さずに解釈して実行する。だから言葉の意味を先に決めておく', a, 394)


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
    a += text(1108, 22, '個人のノウハウを、チームで共有できる', 16, ACCENT, 700, 'end')
    base = 296
    tops = [226, 176, 126]
    a += path(f'M352 {base} V{tops[0]} H604 V{tops[1]} H856 V{tops[2]} H1108 V{base} Z', LINE, 1.5, PANEL)
    a += path(f'M604 {tops[0]} V{base} M856 {tops[1]} V{base}', LINE, 1)
    # 3つの段階は、表紙の3つの教材と同じ名前・同じ印にする
    steps = [('教材1　課題の把握', '課題の本質を捉える'),
             ('教材2　業務の整理', '業務ごとに言葉を揃える'),
             ('教材3　仕組みの構築', '揃えた言葉で仕組みにする')]
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
    a += band(318, 'この3つの段階を上がれる力が、課題をAIで解決する力である')
    return svg('いまはノウハウが個人の手元に留まっている。3つの段階を上がると、チームで共有できる', a, 386)
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
    """作業は固定し、指示の書き方だけを動画ごとに変えて、返ってくる表を比べる。"""
    a = text(0, 20, '変えないもの　作業', 15, DIM)
    a += rect(0, 32, 400, 256, PAPER, 12, LINE)
    a += text(24, 68, '課題を整理する', 20, INK, 700)
    a += path('M24 84 H376', LINE, 1)
    for i, (k, v) in enumerate([('誰が', 'PM'), ('何のために', ['どの課題から決めるか、', '優先順位を付ける']),
                                ('渡すもの', '定例の議事録'), ('返すもの', '課題管理表')]):
        y = [114, 150, 212, 248][i]
        a += text(24, y, k, 14, DIM, 700)
        a += text(128, y, v, 15, INK, gap=24)
    a += text(0, 314, '6本の動画とも、この4つは変えない', 14, DIM)
    # 右：動画ごとに、指示へ1つずつ書き足す
    a += text(440, 20, '変えるもの　指示の書き方', 15, ACCENT, 700)
    # 結果は、本編の各動画で返ってきたものを言葉で示す。段が進むごとに、揃うものが1つずつ増える
    rows = [('原因を知る', [], '並べ方が毎回違う'), ('意味を決める', ['意味'], '並べるものが揃う'),
            ('範囲を決める', ['意味', '範囲'], '見る範囲が揃う'),
            ('条件を決める', ['意味', '範囲', '条件'], '1件ずつの形が揃う')]
    for i, (video, added, result) in enumerate(rows):
        y = 36 + i * 64
        a += text(440, y + 30, video, 14, DIM)
        a += rect(556, y, 156, 48, PAPER, 8, LINE)
        a += text(634, y + 30, '「課題を整理して」', 14, INK, anchor='middle')
        for j, k in enumerate(added):
            x = 720 + j * 70
            a += rect(x, y + 6, 62, 36, PAPER, 8, ACCENT)
            a += text(x + 31, y + 30, '＋' + k, 14, ACCENT, 700, 'middle')
        a += arrow(936, y + 24, 964, y + 24)
        a += text(972, y + 30, result, 14, INK if i else DIM, 700 if i else 400)
    a += text(972, 20, '3回頼んだ結果', 15, DIM)
    a += text(440, 314, '段ごとに議事録を渡して3回ずつ頼み、返ってきた表を比べる', 14, DIM)
    return svg('作業は固定し、指示の書き方だけを動画ごとに変えて、返ってくる表を比べる', a, 324)


def m1_map():
    """教材1のはじめに ── 6本ごとに、何が分かるか。"""
    a = text(0, 24, '動画ごとに分かること', 18, DIM)
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
    a += band(320, '前の動画で分かったことを、次の動画で前提にする')
    return svg('6本それぞれで分かること', a, 388)


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


def i0_dig():
    """いまは基準がPMの頭の中にしかなく、補佐にも後任にもAIにも渡せない。言葉にすれば渡せる。"""
    def receivers(x, ok):
        out = ''
        for i, (who, y) in enumerate([('補佐', 86), ('後任', 160), ('AI', 234)]):
            if who == 'AI':
                out += _robot_at(x - 24, y - 30, INK)
            else:
                out += person(x, y, .62, INK)
            out += text(x + 40, y + 6, who, 15, INK, 700)
        return out
    # 左：いま
    a = text(0, 22, 'いま', 17, DIM)
    a += rect(0, 34, 530, 262, PANEL, 12)
    a += rect(28, 58, 196, 64, PAPER, 32, ACCENT)
    a += text(126, 84, '判断の基準', 17, ACCENT, 700, 'middle')
    a += text(126, 106, 'PMの頭の中にしかない', 13, DIM, anchor='middle')
    a += circle(118, 136, 6, PAPER, ACCENT) + circle(112, 154, 4, PAPER, ACCENT)
    a += person(110, 196, .9, INK)
    a += text(110, 246, 'PM', 15, INK, 700, 'middle')
    a += text(110, 272, '毎回その場で考え直す', 14, DIM, anchor='middle')
    for y in (80, 154, 228):
        a += _dash(f'M170 190 L376 {y}', LINE)
        a += cross(273, round(190 + (y - 190) * 103 / 206), DIM, .8)
    a += receivers(420, False)
    a += arrow(540, 165, 574, 165)
    # 右：言葉にすると
    a += text(582, 22, '基準を言葉にすると', 17, DIM)
    a += rect(582, 34, 530, 262, PAPER, 12, LINE)
    a += rect(606, 62, 250, 206, PAPER, 8, INK)
    a += text(626, 94, '言葉にした基準', 16, INK, 700)
    a += path('M626 108 H836', LINE, 1)
    a += text(626, 142, '課題の捉え方', 15, INK)
    a += text(836, 142, '教材1', 13, DIM, 700, 'end')
    a += text(626, 180, '業務ごとの言葉の読み方', 15, INK)
    a += text(836, 180, '教材2', 13, DIM, 700, 'end')
    a += path('M626 210 H836', LINE, 1)
    a += text(626, 240, '誰でも使える形にする', 14, DIM)
    a += text(836, 240, '教材3', 13, DIM, 700, 'end')
    for y in (80, 154, 228):
        a += path(f'M864 165 L956 {y}', INK, 2)
    a += receivers(1000, True)
    for y in (86, 160, 234):
        a += check(1088, y - 2, ACCENT, .7)
    a += band(318, '頭の中の基準を言葉にすれば、ほかの人にもAIにも渡せる')
    return svg('いまは基準がPMの頭の中にしかなく、補佐にも後任にもAIにも渡せない。言葉にすれば渡せる', a, 384)
