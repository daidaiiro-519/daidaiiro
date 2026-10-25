"""2本目（意味を決める）の図。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)
from intro_visuals import _robot_at

def _sheet(x, y, w, h, stroke=LINE):
    """角を折った書類の形。"""
    d = f'M{x} {y} H{x + w - 22} L{x + w} {y + 22} V{y + h} H{x} Z M{x + w - 22} {y} V{y + 22} H{x + w}'
    return path(d, stroke, 2, PAPER) + rect(x, y, w, h, "none", 0, "none")


def t2_intro():
    from bridge_visuals import intro
    return intro(
        '前の動画で確認したこと',
        [('目的', 'PMが優先順位を判断するために整理する'), ('必要な内容', '目的に合わせ、意味・範囲・条件を決める')],
        'この動画で詳しく見る内容', 1, 0, '図の出力は、考え方を説明するための例',
        [('出力例を確認する', 'PMが必要な内容と比べる'), ('使う目的を比べる', '課題が何を指すかを見る'),
         ('課題の意味を定義する', 'PMの利用目的に合わせる'), ('出力を確認する', '定義に合っているかを見る')],
        '目的に合わせて課題を定義し、具体的な出力に必要な事項があるか確認する',
        '前の動画で確認した内容をもとに、課題の意味を詳しく定義する動画の進め方',
        focus=[('使う目的', 'PMは何に使うか？'), ('課題の意味', '何を課題と呼ぶか？')])




def t2_sorting():
    """利用目的と課題の意味を対応させる。3回の実行結果ではない。"""
    people = [('精算の担当者', '経費の精算業務', '精算を滞りなく続ける', '精算を妨げる事項', '例：確認の時間が足りない', False),
              ('PM', '案件の進行管理', '案件を止めずに進める', '進行を止める未決定事項', '例：承認の経路が未決定', True),
              ('開発者', 'システムの開発', '予定どおりに実装する', '実装を妨げる事項', '例：試験環境が足りない', False)]
    a = ''
    for i, (who, biz, goal, meaning, example, hot) in enumerate(people):
        cx = 185+i*371;col = ACCENT if hot else LINE
        a += rect(cx-170, 0, 340, 356, PAPER, 14, col)
        a += circle(cx, 52, 34, PAPER, col) + person(cx, 58, .58, ACCENT if hot else INK)
        a += text(cx, 112, who, 18, INK, 700, 'middle') + text(cx, 134, biz, 14, DIM, anchor='middle')
        a += rect(cx-150, 156, 300, 56, PAPER, 28, col) + path(f'M{cx-10} 157 L{cx} 146 L{cx+10} 157', col, 2, PAPER)
        a += text(cx, 190, goal, 17, ACCENT if hot else INK, 700, 'middle')
        a += path(f'M{cx} 214 V242', col, 2) + path(f'M{cx-7} 235 l7 8 7 -8', col, 2)
        a += _sheet(cx-148, 248, 296, 66, col)
        a += text(cx-130, 276, meaning, 17, INK, 700) + text(cx-130, 300, example, 14, DIM)
    a += rect(486, 322, 140, 26, ACCENT, 13, ACCENT) + text(556, 340, '今回の利用目的', 13, PAPER, 700, 'middle')
    return svg('精算、進行管理、開発では課題が指すものが異なる。今回の作業ではPMが表を使う目的を確かめる', a, 360)


def t2_ambiguous():
    a = rect(0, 0, 440, 230, PAPER, 36, LINE)
    a += text(32, 42, 'PMの頭の中', 14, DIM, 700)
    for i, (k, v) in enumerate([('目的', '優先順位を判断したい'), ('PMの業務', '案件の進行管理'), ('必要な課題', '進行に関わる未決定事項')]):
        y = 86+i*40
        a += text(32, y, k, 14, DIM) + text(144, y, v, 17, INK, 700)
    a += text(32, 206, '当たり前だと思い、説明していなかった', 15, ACCENT, 700)
    a += circle(120, 246, 8, PAPER, LINE) + circle(100, 266, 5, PAPER, LINE)
    a += rect(0, 276, 200, 60, PAPER, 30, LINE)
    a += circle(36, 306, 18, PAPER, LINE) + person(36, 309, .34, INK) + text(66, 312, 'PM', 17, INK, 700)
    a += arrow(440, 116, 484, 116)
    a += rect(492, 56, 320, 96, PAPER, 12, LINE)
    a += text(512, 84, '入力したプロンプト', 14, DIM, 700) + text(512, 128, '「課題を整理して」', 24, INK, 700)
    box = rect(492, 168, 320, 52, PAPER, 10, LINE)
    a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 5"')
    a += cross(514, 194, DIM, .55) + text(536, 200, '目的と課題の意味が、未記載', 15, DIM)
    a += arrow(812, 116, 856, 116)
    a += rect(864, 40, 248, 152, PAPER, 14, ACCENT) + _robot_at(964, 62, INK)
    a += text(988, 142, 'AIに伝えたのは、', 16, ACCENT, 700, 'middle') + text(988, 168, '議事録とプロンプト', 16, ACCENT, 700, 'middle')
    return svg('PMが考えていた利用目的と課題の意味をプロンプトで説明していなかった', a, 340)


def t2_goal():
    """作業から利用目的を確認し、その目的に必要な意味を問う。"""
    a = rect(0, 0, 510, 136, PAPER, 12, LINE)
    a += text(24, 32, 'PMが表を使う目的', 16, DIM, 700)
    a += text(24, 86, '優先順位を判断する', 28, INK, 700)
    a += _sheet(0, 232, 510, 104)
    a += text(24, 264, 'AIに任せたい作業', 16, DIM, 700)
    a += text(24, 310, '議事録から課題を整理する', 25, INK, 700)
    a += path('M255 232 V136 M248 144 L255 136 L262 144', LINE, 2)
    a += text(280, 190, '何のために？', 18, INK, 700)
    a += arrow(510, 68, 602, 68)
    a += rect(602, 0, 510, 136, PAPER, 12, ACCENT)
    a += text(626, 32, '意味：何を課題と呼ぶか', 16, ACCENT, 700)
    a += text(626, 72, ['どの事項を課題として', '含めればよいか？'], 25, INK, 700, gap=34)
    a += rect(602, 190, 510, 146, PAPER, 12, LINE)
    a += text(618, 218, 'このあとの動画で詳しく決めること', 14, DIM, 700)
    for x, name, body in [(618, '範囲', '対象の案件と期間'), (864, '条件', '各課題に含める情報')]:
        a += rect(x, 234, 232, 86, PAPER, 10, LINE)
        a += text(x+16, 264, name, 19, DIM, 700) + text(x+16, 298, body, 16, INK)
    a += text(24, 385, '作業の利用目的を確かめ、その目的に合わせて、課題の意味を定義する。', 18, INK)
    return svg('左下の課題整理という作業から何のためかを問い、上のPMの利用目的を確認する。その目的に必要な課題の意味を右で考える', a, 404)


def t2_pick():
    """左右の業務別一覧へ、一つの共通定義を適用する。"""
    a = rect(0, 0, 1112, 382, PAPER, 12, ACCENT)
    a += text(24, 30, 'AIに伝える共通の定義', 15, DIM, 700)
    a += text(1088, 30, '✓ 課題に含める　× 含めない', 14, DIM, anchor='end')
    a += text(24, 66, '課題とは、決定しないと案件の進行が止まる事項です。', 23, INK, 700)
    a += path('M24 86 H1088', LINE, 1)
    a += text(556, 115, 'すべての事項に、同じ判断基準を使う', 17, ACCENT, 700, 'middle')
    a += circle(556, 133, 2.5, LINE, LINE)
    a += rect(12, 157, 532, 211, PAPER, 10, LINE)
    a += rect(568, 157, 532, 211, PAPER, 10, LINE)
    a += path('M556 133 H278 V157 M556 133 H834 V157 M271 149 l7 8 7 -8 M827 149 l7 8 7 -8', LINE, 2)
    groups = [('経理から出た事項', [('承認の経路', True), ('差し戻しの扱い', True), ('申請の締め日', True), ('月次の締めの進め方', False)]),
              ('開発から出た事項', [('既存データの移行対象', True), ('旧システムの停止日', True), ('試験環境の準備', False)])]
    for i, (heading, items) in enumerate(groups):
        x = 24 + i * 556
        a += text(x, 190, heading, 18, INK, 700)
        for j, (item, include) in enumerate(items):
            y = 230 + j * 40
            col = ACCENT if include else DIM
            a += text(x, y, '✓' if include else '×', 23, col, 700)
            a += text(x + 34, y, item, 17, INK, 700 if include else 400)
            a += text(x + 508, y, '未決定・進行が止まる' if include else '対応済み・進行に影響なし', 14, col, 700 if include else 400, 'end')
    a += text(24, 415, '判断の根拠は、議事録に記録された状態と、案件の進行への影響。', 17, INK)
    return svg('経理と開発の個別事項へ同じ課題の定義を適用する。未決定で進行を止める事項を含め、対応済みで影響のない事項を含めない', a, 430)




def t2_end():
    """具体の困りごとから利用目的と共通の定義へ戻り、各週の出力で検証する往復。"""
    from v1_visuals import _ray
    a = rect(0, 0, 510, 112, PAPER, 12, LINE)
    a += text(24, 30, 'PMが表を使う目的', 16, DIM, 700)
    a += text(24, 78, '優先順位を判断する', 27, INK, 700)
    a += rect(602, 0, 510, 112, PAPER, 12, ACCENT)
    a += text(626, 30, '抽象：各週に使う共通の定義', 16, ACCENT, 700)
    a += text(626, 67, ['課題＝決定しないと', '案件の進行が止まる事項'], 22, INK, 700, gap=29)
    a += rect(0, 224, 510, 112, PAPER, 12, LINE)
    a += text(24, 254, '具体：出力例の困りごと', 16, DIM, 700)
    a += text(24, 291, ['対応済みの事項も、', '課題に含まれていた'], 22, INK, 700, gap=29)
    a += rect(602, 224, 510, 112, PAPER, 12, LINE)
    a += text(626, 254, '具体：各週の出力を確認する', 16, DIM, 700)
    a += text(626, 291, ['必要な課題の漏れと、', '余分な事項を確認する'], 22, INK, 700, gap=29)
    a += _ray(255, 224, 255, 112)
    a += text(280, 171, '何に使うかを確かめる', 17, INK)
    a += _ray(510, 56, 602, 56)
    a += _ray(857, 112, 857, 224)
    a += text(880, 171, '具体例に適用', 16, INK)
    a += _ray(602, 280, 510, 280)
    a += text(556, 359, '不足があれば、目的と定義を見直す', 17, INK, anchor='middle')
    a += rect(0, 380, 1112, 44, PAPER, 12, LINE)
    a += text(24, 409, '次は範囲：対象の案件と期間を決める。', 18, INK)
    return svg('出力例の具体的な困りごとからPMの利用目的を確認し、共通の定義を決めて各週の出力へ適用する。不足があれば目的と定義を見直す', a, 430)


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

def t2_three_returns():
    """対応済みの事項が課題に含まれる問題を、2つの代表例で示す。"""
    a = _sheet(0, 16, 352, 300)
    a += text(20, 50, '今週の議事録（抜粋）', 20, INK, 700)
    a += text(20, 110, '承認の経路', 25, INK, 700)
    a += text(20, 149, '未決定・進行が止まる', 19, DIM)
    a += path('M20 183 H332', LINE, 1)
    a += text(20, 226, '試験環境の準備', 25, INK, 700)
    a += text(20, 265, '対応済み・進行に影響なし', 18, DIM)
    a += arrow(352, 124, 380, 124)
    a += rect(380, 58, 352, 114, PAPER, 12, ACCENT)
    a += text(556, 106, '「課題を整理して」', 26, INK, 700, 'middle')
    a += text(556, 146, '目的と課題の意味は未記載', 17, DIM, anchor='middle')
    a += path('M556 172 V214 M549 206 l7 8 7 -8', LINE, 2)
    a += rect(438, 214, 236, 102, PAPER, 12, LINE)
    a += _robot_at(458, 244, INK)
    a += text(524, 256, ['AIが議事録を', '整理する'], 17, INK, 700, gap=28)
    a += arrow(674, 265, 760, 265)
    a += rect(760, 16, 352, 300, PAPER, 12, LINE)
    a += text(780, 50, '出力例：課題列の抜粋', 20, INK, 700)
    a += text(780, 114, '承認の経路', 25, INK, 700)
    a += rect(776, 184, 320, 110, PAPER, 10, ACCENT)
    a += text(792, 226, '試験環境の準備', 25, ACCENT, 700)
    a += text(792, 266, '対応済みの事項も入っている', 17, ACCENT)
    a += text(24, 366, 'PMが必要なのは、決定しないと案件の進行が止まる事項。', 21, INK, 700)
    return svg('2つの代表例で、対応済みで進行に影響しない事項も課題に含まれる問題を示す。図は議事録と出力の抜粋', a, 386)


def t2_after():
    a = rect(0, 0, 1112, 296, PAPER, 12, LINE)
    a += text(24, 34, '確認の基準：決定しないと案件の進行が止まる事項', 20, INK, 700)
    a += text(1088, 34, '● 含む　○ 含まない', 14, DIM, anchor='end')
    a += text(24, 82, '議事録の事項（抜粋）', 16, DIM, 700)
    a += text(640, 82, '意味を伝える前の例', 19, INK, 700, 'middle')
    a += text(944, 82, '目的と意味を伝えた例', 19, ACCENT, 700, 'middle')
    a += path('M24 100 H1088 M792 55 V278', LINE, 1)
    a += text(24, 154, '承認の経路', 25, INK, 700)
    a += text(24, 184, '未決定・進行が止まる', 17, DIM)
    a += circle(640, 152, 10, DIM, DIM) + circle(944, 152, 10, DIM, DIM)
    a += rect(14, 206, 1084, 76, PAPER, 10, ACCENT)
    a += text(24, 239, '試験環境の準備', 25, ACCENT, 700)
    a += text(24, 270, '対応済み・進行に影響なし', 17, DIM)
    a += circle(640, 240, 10, ACCENT, ACCENT) + circle(944, 240, 10, PAPER, LINE)
    a += rect(0, 326, 1112, 58, PAPER, 12, LINE)
    a += text(24, 362, '余分な事項の除外と、必要な課題の漏れを確認する。', 22, INK, 700)
    return svg('3枚目と同じ2つの代表例で、定義を伝える前と後の出力例を比較する。除外だけでなく必要な課題の漏れも確認する', a, 392)
