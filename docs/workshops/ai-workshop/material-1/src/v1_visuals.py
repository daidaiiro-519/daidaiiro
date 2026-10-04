"""1本目（原因を知る）の図。1枚に主役を1つ置き、図の中の文字は16px以上を基本にする。
文字は必ずカードや札の中に置き、背景の上にじかに書かない（軸の両端の目印だけは例外）。
原稿で言うことは図に書かず、見れば分かるものだけを置く。"""
from lesson_visuals import (text, rect, path, arrow, circle, icon, svg,
                            INK, DIM, LINE, PAPER, ACCENT, check, cross)
from intro_visuals import _robot_at, _table_icon


def _sheet(x, y, w, h, stroke=LINE):
    """角を折った書類の形。矢印の付け根を判定できるよう、同じ大きさの見えない枠を重ねる。"""
    d = f'M{x} {y} H{x + w - 22} L{x + w} {y + 22} V{y + h} H{x} Z M{x + w - 22} {y} V{y + 22} H{x + w}'
    return path(d, stroke, 2, PAPER) + rect(x, y, w, h, 'none', 0, 'none')


def _badge(x, y, t, w=76):
    return rect(x, y, w, 28, ACCENT, 14, ACCENT) + text(x + w / 2, y + 19, t, 14, PAPER, 700, 'middle')


def _axis(top, bottom, x=80):
    """目的レベルの軸。上が抽象（なぜ）、下が具体（どのように）。"""
    a = text(0, top + 22, '抽象', 20, INK, 700) + text(0, top + 44, 'なぜ', 14, DIM)
    a += text(0, bottom - 26, '具体', 20, INK, 700) + text(0, bottom - 4, 'どのように', 14, DIM)
    # 抽象と具体は行き来するので、上下の両方に矢じりを付ける
    a += path(f'M{x} {bottom} V{top} m-7 8 l7 -8 7 8 M{x - 7} {bottom - 8} l7 8 7 -8', LINE, 2)
    return a


def _down(x, y1, y2, color=LINE):
    return path(f'M{x} {y1} V{y2}', color, 2) + path(f'M{x - 7} {y2 - 8} l7 8 7 -8', color, 2)


def _ray(x1, y1, x2, y2, color=LINE):
    """(x1, y1) から (x2, y2) へ向かう線と、終点の矢じり。斜めでも向きに合わせて描く。"""
    dx, dy = x2 - x1, y2 - y1
    n = (dx * dx + dy * dy) ** .5
    ux, uy = dx / n, dy / n
    px, py = -uy, ux
    b1 = (x2 - 9 * ux + 6 * px, y2 - 9 * uy + 6 * py)
    b2 = (x2 - 9 * ux - 6 * px, y2 - 9 * uy - 6 * py)
    return (path(f'M{x1:.0f} {y1:.0f} L{x2:.0f} {y2:.0f}', color, 2)
            + path(f'M{b1[0]:.0f} {b1[1]:.0f} L{x2:.0f} {y2:.0f} L{b2[0]:.0f} {b2[1]:.0f}', color, 2))


def ask():
    """L1-S0　目的まで決まっている作業を、「課題を整理して」だけでAIに頼む。主役はプロンプトの1文。"""
    a = ''
    for i, (k, v) in enumerate([('誰が', 'PM'), ('何のため', '優先順位を付ける'), ('入力する', '定例の議事録'), ('出力する', '課題管理表')]):
        x = i * 282
        a += rect(x, 0, 266, 44, PAPER, 22, LINE)
        a += text(x + 22, 28, k, 14, DIM, 700) + text(x + 100, 28, v, 16, INK, 700)
    # 左：議事録
    a += _sheet(0, 76, 300, 264)
    a += text(20, 108, '定例の議事録', 18, INK, 700) + text(20, 132, '今週の1回分', 14, DIM)
    a += text(20, 168, ['承認の経路', '差し戻しの扱い', '申請の締め日', '旧システムの停止日', '既存データの移行'], 16, INK, gap=26)
    a += text(20, 316, '月次の締め ・ 試験環境 など', 14, DIM)
    a += arrow(300, 144, 356, 144)
    # 真ん中：プロンプト（主役）と、それを解釈するAI
    a += rect(364, 92, 316, 104, PAPER, 16, ACCENT)
    a += text(522, 140, '「課題を整理して」', 26, INK, 700, 'middle')
    a += text(522, 176, '何のためかは書いていない', 15, ACCENT, 700, 'middle')
    a += _down(522, 196, 232)
    a += rect(404, 236, 236, 104, PAPER, 12, LINE)
    a += _robot_at(426, 266, INK) + text(500, 286, 'AIがプロンプトを', 16, INK, 700) + text(500, 310, '解釈する', 16, INK, 700)
    a += arrow(640, 288, 712, 288)
    # 右：出力される課題管理表
    a += rect(720, 76, 392, 264, PAPER, 12, LINE)
    a += _table_icon(744, 100, INK) + text(788, 124, '課題管理表', 18, INK, 700)
    a += cross(756, 196, ACCENT, .7) + text(780, 204, '実行するたびに違う', 24, ACCENT, 700)
    a += text(744, 266, '同じ議事録で何度か実行すると、', 15, DIM) + text(744, 290, '毎回違う表が出力される', 15, DIM)
    return svg('目的まで決まっている作業を、「課題を整理して」だけでAIに頼むと、AIがプロンプトを解釈して、実行するたびに違う課題管理表を出力する', a, 344)


def symptoms():
    """L1-S1　困りごと2つを、同じ幅の2枚に並べる。左は3回の表の違い、右は細かく決めたプロンプトが来週は合わないこと。"""
    a = rect(0, 0, 540, 330, PAPER, 12, LINE)
    a += _badge(20, 20, '困りごと1') + text(112, 41, '実行するたびに、違う表が出力される', 19, INK, 700)
    a += text(20, 82, '「課題を整理して」で、同じ議事録を3回実行する', 14, DIM)
    X0, CW = 150, 124
    for k in range(3):
        a += text(X0 + k * CW + CW / 2, 120, f'{k + 1}回目', 15, DIM, 700, 'middle')
    a += path('M20 132 H520', LINE, 1.5)
    rows = [('並び順', ['業務の流れの順', '期限の順', '機能ごと']),
            ('各項目の内容', ['影響を受ける担当', '誰が決めるか', '依存する機能']),
            ('混じった話', ['月次の締め', 'なし', '試験環境'])]
    for r, (head, cells) in enumerate(rows):
        y = 170 + r * 52
        a += text(20, y, head, 14, DIM, 700)
        for k, c in enumerate(cells):
            hot = r == 2 and c != 'なし'
            a += text(X0 + k * CW + CW / 2, y, c, 15, ACCENT if hot else INK, 700, 'middle')
        if r < 2:
            a += path(f'M20 {y + 22} H520', LINE, 1)
    # 右：細かく決めたプロンプト
    a += rect(572, 0, 540, 330, PAPER, 12, LINE)
    a += _badge(592, 20, '困りごと2') + text(684, 41, 'その1回にしか使えない', 19, INK, 700)
    a += rect(592, 66, 500, 84, PAPER, 10, LINE)
    a += text(612, 100, '「1行目に承認の経路、', 18, INK, 700) + text(612, 130, '　2行目に申請の締め日」', 18, INK, 700)
    for i, (wk, res, ok) in enumerate([('今週の議事録', 'そのとおりに出力される', True), ('来週の議事録', '承認の話が無く、合わない', False)]):
        y = 190 + i * 50
        a += (check(606, y, ACCENT, .6) if ok else cross(606, y, ACCENT, .7))
        a += text(632, y + 7, wk, 15, DIM, 700) + text(752, y + 7, res, 17, INK, 700)
    a += text(592, 300, '毎週、プロンプトを書き直すことになる', 16, ACCENT, 700)
    return svg('困りごと1は、同じ議事録で3回実行すると、並び順・各項目の内容・混じった話が毎回違うこと。困りごと2は、細かく決めたプロンプトが来週の議事録には合わないこと', a, 334)


def ends():
    """L1-S2　2つのプロンプトは抽象と具体の両端にある。目指すのは、そのあいだの作業の目的。"""
    a = _axis(0, 372)
    rows = [('「課題を整理して」', '何のための整理かが書かれていない', (True, 'どの週にも使える'), (False, '毎回変わる'), False),
            ('？　作業に合った目的レベル', '優先順位を付けるために、課題を整理する', (True, 'どの週にも使える'), (True, '目的から外れない'), True),
            ('「1行目に承認の経路、2行目に申請の締め日」', '具体に寄りすぎている', (False, 'この議事録だけ'), (True, '1つに決まる'), False)]
    for i, (head, sub, use, ret, hot) in enumerate(rows):
        y = 10 + i * 124
        cy = y + 48
        a += circle(80, cy, 9 if hot else 7, ACCENT if hot else PAPER, ACCENT if hot else LINE) + path(f'M89 {cy} H100', LINE, 1)
        box = rect(100, y, 560, 96, PAPER, 12, ACCENT if hot else LINE)
        a += box.replace('stroke-width="2"', 'stroke-width="2.5" stroke-dasharray="7 5"') if hot else box
        a += text(124, y + 42, head, 19 if len(head) < 18 else 17, ACCENT if hot else INK, 700) + text(124, y + 72, sub, 15, DIM)
        a += rect(676, y, 436, 96, PAPER, 12, LINE) + path(f'M894 {y + 16} V{y + 80}', LINE, 1)
        for x, label, (ok, t) in ((676, '使える議事録', use), (894, '出力', ret)):
            a += text(x + 20, y + 30, label, 12, DIM, 700)
            a += (check(x + 30, y + 62, ACCENT, .6) if ok else cross(x + 30, y + 62, DIM, .7))
            a += text(x + 54, y + 69, t, 17, INK if ok else DIM, 700 if ok else 400)
    return svg('2つのプロンプトは抽象と具体の両端にある。上は使えるが出力が決まらず、下は決まるがその議事録にしか使えない。目指すのは、そのあいだの作業に合った目的レベル', a, 376)


def fit():
    """L1-S3　1つずつ違う議事録（具体）から、どの議事録にも共通する目的（抽象）を通って、目的から外れない表が返る。左右を同じ形にそろえる。"""
    CX, CY, R = 556, 188, 120
    a = ''
    for side, x0, head, rows in [(0, 0, '1つずつ違う議事録', ['今週の議事録', '来週の議事録', '別の案件の議事録']),
                                 (1, 812, '出力される課題管理表', ['今週の表　5件', '来週の表　6件', '別の案件の表　9件'])]:
        a += rect(x0, 0, 300, 356, PAPER, 12, LINE)
        a += rect(x0 + 16, 14, 56, 26, INK, 13, INK) + text(x0 + 44, 32, '具体', 13, PAPER, 700, 'middle')
        a += text(x0 + 84, 33, head, 15, INK, 700)
        for i, name in enumerate(rows):
            y = 56 + i * 98
            cy = y + 34
            a += rect(x0 + 16, y, 268, 68, PAPER, 10, LINE)
            if side == 0:
                a += icon(x0 + 32, y + 14, 'doc', DIM, .6) + text(x0 + 76, cy + 6, name, 16, INK, 700)
                d = ((CX - x0 - 284) ** 2 + (CY - cy) ** 2) ** .5
                a += _ray(x0 + 284, cy, CX - (CX - x0 - 284) * R / d, CY - (CY - cy) * R / d)
            else:
                a += text(x0 + 32, cy - 4, name, 16, INK, 700)
                a += check(x0 + 40, cy + 18, ACCENT, .45) + text(x0 + 58, cy + 24, '優先順位を付けるのに使える', 13, INK)
                d = ((x0 + 16 - CX) ** 2 + (cy - CY) ** 2) ** .5
                a += _ray(CX + (x0 + 16 - CX) * R / d, CY + (cy - CY) * R / d, x0 + 16, cy)
    # 真ん中：どの議事録にも共通する目的（主役）
    a += circle(CX, CY, R, PAPER, ACCENT)
    a += rect(CX - 28, CY - 86, 56, 26, ACCENT, 13, ACCENT) + text(CX, CY - 68, '抽象', 13, PAPER, 700, 'middle')
    a += text(CX, CY - 34, 'どの議事録にも共通する目的', 13, DIM, 700, 'middle')
    a += text(CX, CY + 10, '優先順位を', 28, ACCENT, 700, 'middle') + text(CX, CY + 48, '付ける', 28, ACCENT, 700, 'middle')
    return svg('1つずつ違う議事録（具体）に共通する目的（抽象）は、優先順位を付けること。目的レベルが明確なプロンプトなら、出力される表は中身も件数も違っても、どれも目的から外れない', a, 360)


def three():
    """L1-S4　見えた違いに1つずつ対応させて、書くことを3つ決める。主役は3つの語。"""
    cols = [('並ぶ課題の種類が違う', '月次の締め ・ 試験環境が混じる', '意味', '課題が何を指すか'),
            ('抽出する量が違う', '翌週の議事録に使うと分かる', '範囲', 'どこまでを見るか'),
            ('並び順と各項目の書き方が違う', '期限の順か、誰が決めるか', '条件', '何を入れ、何を落とし、どう並べるか')]
    a = ''
    for i, (diff, eg, name, what) in enumerate(cols):
        x = i * 380
        cx = x + 176
        a += rect(x, 0, 352, 96, PAPER, 10, LINE)
        a += text(x + 20, 26, '見えた違い', 12, DIM, 700)
        a += text(x + 20, 56, diff, 17, INK, 700) + text(x + 20, 82, eg, 14, DIM)
        a += _down(cx, 96, 128)
        a += circle(cx, 184, 54, PAPER, ACCENT) + text(cx, 198, name, 34, ACCENT, 700, 'middle')
        a += _down(cx, 238, 262)
        a += rect(x, 266, 352, 50, PAPER, 10, LINE)
        a += text(cx, 298, what + 'を書く', 17, INK, 700, 'middle')
    a += rect(0, 336, 540, 52, PAPER, 10, ACCENT)
    a += text(20, 368, '書かないと、AIが回ごとに決めるので、出力がばらつく', 15, ACCENT, 700)
    a += rect(572, 336, 540, 52, PAPER, 10, LINE)
    a += text(592, 368, '全部は書かない。どう並べるかまで書くと、具体に寄りすぎる', 15, INK)
    return svg('出力の違い（種類 ・ 量 ・ 並び順と各項目の書き方）に1つずつ対応させて、意味 ・ 範囲 ・ 条件の3つを書く。書かないとAIが決めてばらつき、全部を書くと具体に寄りすぎる', a, 392)


def order():
    """L1-S5　意味 ・ 範囲 ・ 条件の順に追記すると、プロンプトは作業の目的へ下りていく。前の1つが決まっていないと、次の1つを決められない。"""
    rows = [(None, '「課題を整理して」', 'ここから始める'),
            ('意味', '「課題とは、決定しないと案件の進行が止まるもの」', '何を抽出するかが決まる'),
            ('範囲', '「見るのは、今回の案件で12月末の切り替えまでに決めるもの」', '意味が通じるところまでを決める'),
            ('条件', '「どの項目にも、誰が決めるかと、いつまでかを入れる」', '範囲の中の各項目に入れる')]
    a = _axis(0, 372)
    for i, (name, quote, why) in enumerate(rows):
        y = 10 + i * 76
        cy = y + 31
        last = i == len(rows) - 1
        a += circle(80, cy, 8 if last else 6, ACCENT if last else PAPER, ACCENT if last else LINE) + path(f'M88 {cy} H100', LINE, 1)
        a += rect(100, y, 680, 62, PAPER, 10, ACCENT if last else LINE)
        if name:
            a += circle(126, cy, 15, ACCENT, ACCENT) + text(126, cy + 6, str(i), 16, PAPER, 700, 'middle')
            a += text(154, cy + 7, '＋' + name, 18, ACCENT, 700) + text(222, cy + 7, quote, 16, INK, 700)
        else:
            a += text(124, cy + 8, quote, 18, INK, 700)
        a += rect(800, y, 312, 62, PAPER, 10, LINE)
        if i == 0:
            a += text(822, cy - 6, 'この順である理由', 11, DIM, 700) + text(822, cy + 14, why, 16, INK, 700)
        else:
            a += text(822, cy + 7, why, 16, INK, 700)
    a += text(64, 279, '作業の目的', 13, ACCENT, 700, 'end')
    box = rect(100, 322, 1012, 44, PAPER, 10, LINE)
    a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 5"')
    a += circle(80, 344, 6, PAPER, LINE) + path('M86 344 H100', LINE, 1)
    a += text(124, 350, '具体に寄りすぎない', 16, DIM, 700)
    return svg('意味・範囲・条件の順に追記すると、プロンプトの目的レベルが作業の目的に明確になる。前の1つが決まっていないと次の1つを決められない。具体に寄りすぎない', a, 376)
