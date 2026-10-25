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
    from first_visuals import ask as draw
    return draw()


def symptoms():
    from first_visuals import symptoms as draw
    return draw()



def _offaxis_row(y, head, sub_, use, ret, size=16):
    """目的が書かれていないプロンプト。軸の外に置き、段が決まらないことを示す。"""
    from lesson_visuals import check, cross
    a = text(0, y + 44, '大きさが', 13, DIM, 700) + text(0, y + 62, '決まらない', 13, DIM, 700)
    box = rect(100, y, 560, 84, PAPER, 12, LINE)
    a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 5"')
    a += text(124, y + 36, head, 19, INK, 700) + text(124, y + 64, sub_, 15, DIM)
    a += rect(676, y, 436, 84, PAPER, 12, LINE) + path(f'M894 {y + 14} V{y + 70}', LINE, 1)
    for x, label, (ok, t) in ((676, '使える議事録', use), (894, '出力', ret)):
        a += text(x + 20, y + 28, label, 12, DIM, 700)
        a += (check(x + 30, y + 56, ACCENT, .6) if ok else cross(x + 30, y + 56, DIM, .7))
        a += text(x + 54, y + 63, t, size, INK if ok else DIM, 700 if ok else 400)
    return a


def _axis_rows(rows, top=110, step=128, size=17):
    from lesson_visuals import check, cross
    a = text(0, top + 18, '抽象', 20, INK, 700) + text(0, top + 40, 'なぜ', 14, DIM)
    a += text(0, 346, '具体', 20, INK, 700) + text(0, 368, 'どのように', 14, DIM)
    a += path(f'M80 372 V{top} m-7 8 l7 -8 7 8 M73 364 l7 8 7 -8', LINE, 2)
    for i, (head, sub_, use, ret, hot, dashed) in enumerate(rows):
        y = top + 12 + i * step
        cy = y + 44
        a += circle(80, cy, 9 if hot else 7, ACCENT if hot else PAPER, ACCENT if hot else LINE) + path(f'M89 {cy} H100', LINE, 1)
        box = rect(100, y, 560, 88, PAPER, 12, ACCENT if hot else LINE)
        a += box.replace('stroke-width="2"', 'stroke-width="2.5" stroke-dasharray="7 5"') if dashed else box
        a += text(124, y + 38, head, 19 if len(head) < 18 else 16, ACCENT if hot else INK, 700) + text(124, y + 68, sub_, 15, INK if hot else DIM, 700 if hot else 400)
        a += rect(676, y, 436, 88, PAPER, 12, LINE) + path(f'M894 {y + 14} V{y + 74}', LINE, 1)
        for x, label, (ok, t) in ((676, '使える議事録', use), (894, '出力', ret)):
            a += text(x + 20, y + 28, label, 12, DIM, 700)
            a += (check(x + 30, y + 58, ACCENT, .6) if ok else cross(x + 30, y + 58, DIM, .7))
            a += text(x + 54, y + 65, t, size, INK if ok else DIM, 700 if ok else 400)
    return a


def ends():
    from first_visuals import ends as draw
    return draw()


def fit():
    """PMが確かめた目的と共通条件を、各議事録に適用する。"""
    CX, CY, R = 556, 188, 140
    a = ''
    for side, x0, head, rows in [
        (0, 0, '1回分ずつの議事録', ['今週の議事録', '翌週の議事録', '別の案件の議事録']),
        (1, 812, '課題管理表の例', [
            ('今週：承認の経路', '担当：経理と開発', '期限：9月16日'),
            ('翌週：連携の仕様', '担当：開発', '期限：9月23日'),
            ('別の案件：移行の手順', '担当：開発', '期限：10月7日'),
        ]),
    ]:
        a += rect(x0, 0, 300, 356, PAPER, 12, LINE)
        a += rect(x0+16, 14, 56, 26, INK, 13, INK) + text(x0+44, 32, '具体', 13, PAPER, 700, 'middle')
        a += text(x0+84, 33, head, 15, INK, 700)
        for i, row in enumerate(rows):
            y = 56+i*98
            cy = y+34
            a += rect(x0+16, y, 268, 68, PAPER, 10, LINE)
            if side == 0:
                a += icon(x0+32, y+14, 'doc', DIM, .6) + text(x0+76, cy+6, row, 16, INK, 700)
                d = ((CX-x0-284)**2+(CY-cy)**2)**.5
                # 関係線は適用先を示す。入力から目的を導く矢印にしない。
                a += path(f'M{x0+284} {cy} L{CX-(CX-x0-284)*R/d:.0f} {CY-(CY-cy)*R/d:.0f}', LINE, 2)
            else:
                a += text(x0+32, y+20, row[0], 16, INK, 700)
                a += text(x0+32, y+42, row[1], 14, INK)
                a += text(x0+32, y+61, row[2], 14, INK)
                d = ((x0+16-CX)**2+(cy-CY)**2)**.5
                a += _ray(CX+(x0+16-CX)*R/d, CY+(cy-CY)*R/d, x0+16, cy)
    a += text(376, 24, '各議事録に適用', 14, DIM, 700, 'middle')
    a += text(748, 24, '出力を確認', 14, DIM, 700, 'middle')
    a += circle(CX, CY, R, PAPER, ACCENT)
    a += rect(CX-56, 68, 112, 26, ACCENT, 13, ACCENT) + text(CX, 86, 'PMの目的', 13, PAPER, 700, 'middle')
    a += text(CX, 117, '何のために整理するか', 16, INK, 700, 'middle')
    a += text(CX, 148, ['優先順位を判断するために、', '課題を整理する。'], 18, ACCENT, 700, 'middle', gap=27)
    a += path('M450 196 H662', LINE, 1)
    a += rect(CX-75, 207, 150, 26, ACCENT, 13, ACCENT) + text(CX, 225, '抽象：共通条件', 13, PAPER, 700, 'middle')
    a += text(CX, 254, ['各課題に、', '担当と期限を含める。'], 20, ACCENT, 700, 'middle', gap=26)
    a += text(CX, 300, '値は議事録から', 14, INK, anchor='middle')
    a += text(556, 388, '目的はPMが確かめる。共通条件を各週に適用する。別案件では対象の案件と期間を変更する。', 16, INK, anchor='middle')
    return svg('PMの利用目的を確かめて共通条件を決め、各議事録に適用する。共通条件は個別の担当者と日付に対して抽象的である', a, 400)




def order():
    """L1-S5　作業の目的を伝え、意味 ・ 範囲 ・ 条件の順に決める。目的の大きさは変わらず、決めることが増える。作業の目的に合わせて、何を課題とし、どの話を見て、各課題に何を入れるかを決める。"""
    rows = [('目的', '「PMが、優先順位を付けるために、課題を整理する」', 'PMが優先順位の判断に使う'),
            ('意味', '「課題とは、決定しないと案件の進行が止まる事項」', '進行に関わる課題を含める'),
            ('範囲', '「対象は、今回の案件で12月末の切り替えまでに決定が必要な事項」', '対象の案件と期間を決める'),
            ('条件', '「各課題に、決定の担当者と期限を含める」', '全課題に担当と期限を入れる')]
    a = text(0, 18, '決めること', 13, DIM, 700)
    for i, (name, quote, why) in enumerate(rows):
        y = 10 + i * 76
        cy = y + 31
        last = i == len(rows) - 1
        a += rect(100, y, 680, 62, PAPER, 10, ACCENT if last else LINE)
        a += circle(126, cy, 15, ACCENT, ACCENT) + text(126, cy + 6, str(i + 1), 16, PAPER, 700, 'middle')
        a += text(154, cy + 7, name, 18 if i else 16, ACCENT, 700) + text(238, cy + 7, quote, 16 if i != 2 else 15, INK, 700)
        a += rect(800, y, 312, 62, PAPER, 10, LINE)
        if i == 0:
            a += text(822, cy - 10, '誰が何に使うか', 14, DIM, 700) + text(822, cy + 18, why, 16, INK, 700)
        else:
            a += text(822, cy + 7, why, 16, INK, 700)
    box = rect(100, 322, 1012, 44, PAPER, 10, LINE)
    a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 5"')
    a += text(124, 350, '目的に合わせて必要な内容を決める。担当者名や日付は、その週の議事録に合わせる。', 16, DIM, 700)
    return svg('目的と、その目的に合わせて決める意味・範囲・条件を示す。抽象度が順に下がる階層ではない', a, 376)


def goal_line():
    from lesson_visuals import W
    """L1-S3B　作業の目的を、誰が ・ 何のために ・ 何をするかに分ける。目的を伝えないと、AIは回ごとに誰かの目的を選ぶ。"""
    a = _sheet(0, 0, W, 166)
    a += text(24, 34, 'AIに伝える作業の目的', 14, DIM, 700)
    parts = [('誰が', ['PMが、']), ('何のために', ['どれから決めるか', '優先順位を付けるために、']), ('何をする', ['課題を整理する'])]
    for k, (label, body) in enumerate(parts):
        x = 24 + k * 360
        a += rect(x, 50, 344, 100, PAPER, 10, ACCENT)
        a += text(x + 20, 76, label, 13, ACCENT, 700) + text(x + 20, 106, body, 18, INK, 700, gap=26)
    a += text(0, 196, '目的を伝えなかったときの、3回の表', 15, DIM, 700)
    who = [('1回目　精算の担当者の表', '業務の流れの順'), ('2回目　PMの表', '決定の期限の順'), ('3回目　開発者の表', '機能ごと')]
    for k, (t, s_) in enumerate(who):
        x = k * 380
        a += rect(x, 210, 352, 72, PAPER, 10, LINE) + _table_icon(x + 18, 226, DIM)
        a += text(x + 64, 238, t, 16, INK, 700) + text(x + 64, 264, s_ + 'に並ぶ', 14, DIM)
    a += rect(0, 304, W, 52, PAPER, 12, LINE)
    a += text(W / 2, 337, '誰のための表かを、AIがその場で決めていた。目的を伝えれば、毎回PMの表になる', 18, INK, 700, 'middle')
    return svg('作業の目的を、誰が ・ 何のために ・ 何をするかに分けて書く。目的を伝えないと、誰の目的かをAIがその場で決める', a, 360)
def three():
    """意味・範囲・条件を一度ずつ示し、決める対象を例で区別する。"""
    a = ''
    for i, (name, question) in enumerate([('意味', '何を課題と呼ぶか'), ('範囲', 'どの案件・時期が対象か'), ('条件', '各課題に何を含めるか')]):
        x = i * 380
        a += rect(x, 16, 352, 286, PAPER, 12, LINE)
        a += text(x + 176, 64, name, 34, ACCENT, 700, 'middle')
        a += text(x + 176, 106, question, 21, INK, 700, 'middle')
        a += path(f'M{x + 20} 129 H{x + 332}', LINE, 1)
    a += check(28, 181, ACCENT, .7)
    a += text(52, 178, ['進行を止める', '未決定の事項'], 22, INK, 700, gap=30)
    a += cross(28, 259, DIM, .7)
    a += text(52, 266, '対応済み・影響なし', 18, DIM)
    a += rect(400, 152, 312, 72, PAPER, 10, ACCENT)
    a += text(420, 183, '今回の案件', 22, INK, 700)
    a += text(420, 210, '本番の切り替えまで', 18, INK)
    a += text(420, 266, '別案件や次のフェーズは対象外', 16, DIM)
    a += text(782, 170, '担当', 18, DIM, 700) + text(958, 170, '期限', 18, DIM, 700)
    for x in [780, 950]:
        a += rect(x, 186, 140, 58, PAPER, 8, LINE)
        a += text(x + 70, 223, '議事録から', 18, INK, 700, 'middle')
    a += text(780, 277, '担当と期限を、どの課題にも含める', 15, DIM)
    a += rect(0, 326, 1112, 54, PAPER, 10, LINE)
    a += text(24, 360, '毎回同じにしたいことを伝え、課題名・名前・日付は議事録に合わせる。', 20, INK, 700)
    return svg('意味は課題の定義、範囲は対象の案件と時期、条件は担当と期限など必要な内容と分ける。個別の項目名や値を固定しない', a, 388)
