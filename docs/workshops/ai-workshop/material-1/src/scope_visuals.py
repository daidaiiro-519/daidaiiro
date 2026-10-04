"""3本目（範囲を決める）の試作の図。議事録の中に混ざっている話を、どこまで対象にするか。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)
from intro_visuals import _robot_at


def _sheet(x, y, w, h, stroke=LINE):
    """角を折った書類の形。矢印の付け根を判定できるよう、同じ大きさの見えない枠を重ねる。"""
    d = f'M{x} {y} H{x + w - 22} L{x + w} {y + 22} V{y + h} H{x} Z M{x + w - 22} {y} V{y + 22} H{x + w}'
    return path(d, stroke, 2, PAPER) + rect(x, y, w, h, 'none', 0, 'none')


MIX = [('今回の案件で、切り替えまでに決めること', '承認の経路／申請の締め日／旧システムの停止日'),
       ('次のフェーズで決めること', '交通費の自動取得／海外出張の通貨の換算'),
       ('定例で出た別案件の相談', '別案件の要員の手配'),
       ('全社の連絡事項', '研修の案内／勤怠の締め')]


def s3_mixed():
    """翌週の定例の議事録には、議題のほかに3種類の話が出ている。意味を追記したプロンプトで3回実行すると、抽出する件数が3倍近く違う。"""
    # 左：翌週の定例の議事録。上から、議題 ・ 次のフェーズの相談 ・ 別案件の相談 ・ 連絡事項
    a = _sheet(0, 0, 480, 330)
    a += text(20, 30, '定例の議事録　翌週の1回分', 15, INK, 700)
    a += path('M20 44 H440', LINE, 1)
    secs = [('議題', ['承認の経路', '申請の締め日', '既存データの移行 ほか'], True),
            ('次のフェーズの相談', ['交通費の自動取得', '海外出張の通貨の換算'], False),
            ('ついでに出た別案件の相談', ['要員の手配'], False),
            ('連絡事項', ['研修の案内', '勤怠の締め'], False)]
    y = 72
    for head, items, hot in secs:
        a += text(20, y, head, 13, ACCENT if hot else DIM, 700)
        if hot:
            a += circle(26, y + 22, 4, ACCENT, ACCENT)
        a += text(36 if hot else 20, y + 26, ' ・ '.join(items), 13, INK if hot else DIM, 700 if hot else 400)
        y += 64
    a += arrow(480, 160, 508, 160)
    a += rect(508, 100, 168, 120, PAPER, 14, LINE)
    a += _robot_at(568, 112, INK)
    a += text(592, 180, '同じプロンプトで3回', 16, INK, 700, 'middle') + text(592, 202, '意味は書いてある', 13, DIM, anchor='middle')
    a += arrow(676, 160, 720, 160)
    # 右：抽出した件数
    a += rect(720, 20, 392, 290, PAPER, 12, LINE)
    a += text(744, 48, '抽出した件数', 14, DIM, 700)
    X0, U = 800, 18
    for t in (0, 5, 10, 15):
        a += path(f'M{X0 + t * U} 66 V244', LINE, 1).replace('stroke-linecap', 'stroke-dasharray="2 4" stroke-linecap')
        a += text(X0 + t * U, 266, str(t), 12, DIM, anchor='middle')
    for i, (no, n) in enumerate([('1回目', 5), ('2回目', 9), ('3回目', 14)]):
        y = 82 + i * 56
        hot = i == 2
        a += text(744, y + 20, no, 14, DIM)
        a += rect(X0, y, n * U, 28, PAPER, 6, ACCENT if hot else LINE)
        a += text(X0 + n * U + 10, y + 21, f'{n}件', 16, ACCENT if hot else INK, 700)
    a += text(744, 294, '少ない回と多い回で、3倍近く違う', 13, ACCENT, 700)
    return svg('翌週の定例の議事録には、議題のほかに3種類の話が出ている。意味を追記したプロンプトで3回実行すると、抽出する件数が5件・9件・14件と3倍近く違う', a, 334)


def s3_ranges():
    """回ごとに、議事録のどこまでを対象にしたかが違う。列を話の種類、行を回にして、抽出した広さを帯で示す。意味は効いている。"""
    X, CW = 200, 228
    a = rect(0, 0, 1112, 268, PAPER, 12, LINE)
    for c, (h, eg) in enumerate([('今回の案件、切り替えまで', '12月末までに決めること'), ('次のフェーズ', '切り替えの後の話'),
                                 ('別案件の相談', 'ほかの案件の話'), ('全社の連絡', '案件と関係ない話')]):
        x = X + c * CW
        a += text(x + 16, 34, h, 16, INK, 700) + text(x + 16, 56, eg, 13, DIM)
        if c:
            a += path(f'M{x} 16 V252', LINE, 1).replace('stroke-linecap', 'stroke-dasharray="3 5" stroke-linecap')
    a += path('M16 72 H1096', LINE, 1.5)
    for r, (no, n, upto) in enumerate([('1回目', '5件', 1), ('2回目', '9件', 2), ('3回目', '14件', 4)]):
        y = 92 + r * 56
        hot = r == 2
        a += text(20, y + 26, no, 16, INK, 700) + text(100, y + 26, n, 18, ACCENT if hot else INK, 700)
        a += rect(X + 10, y, upto * CW - 20, 38, PAPER, 19, ACCENT if hot else INK)
        a += text(X + 30, y + 25, 'ここまで抽出した', 14, ACCENT if hot else INK, 700)
    a += rect(0, 288, 1112, 56, PAPER, 12, LINE)
    a += check(30, 316, ACCENT, .55) + text(54, 323, '意味は効いている。月次の締めと試験環境の話は、どの回にも入っていない', 16, INK, 700)
    return svg('回ごとに、議事録のどこまでを対象にしたかが違う。1回目は切り替えまでの話、2回目は次のフェーズまで、3回目は別案件と全社の連絡まで抽出した。意味は効いている', a, 348)


def s3_two_roles():
    """意味と範囲は別のもの。翌週の議事録に出た話を、意味（進行が止まるか）と範囲（今回の案件の切り替えまでか）で分けた2行2列の表。枠を重ねない。"""
    X, CW, HY, RH = 230, 441, 70, 124
    a = rect(0, 0, 1112, HY + 2 * RH, PAPER, 12, LINE)
    a += path(f'M16 {HY} H1096 M16 {HY + RH} H1096', LINE, 1.5)
    a += path(f'M{X} 16 V{HY + 2 * RH - 16} M{X + CW} 16 V{HY + 2 * RH - 16}', LINE, 1.5)
    for c, (h, sub) in enumerate([('範囲の内', '今回の案件、12月末の切り替えまで'), ('範囲の外', '次のフェーズ ・ 別案件 ・ 全社')]):
        x = X + c * CW
        a += text(x + 20, 32, h, 17, INK, 700) + text(x + 20, 54, sub, 13, DIM)
    for r, (h, sub) in enumerate([('意味に当てはまる', '決めないと進行が止まる'), ('意味に当てはまらない', 'その業務の中で解決できる')]):
        y = HY + r * RH
        a += text(20, y + 56, h, 16, INK, 700) + text(20, y + 80, sub, 13, DIM)
    cells = [(0, 0, '抽出する', ['承認の経路 ・ 申請の締め日', '既存データの移行 ほか'], True),
             (1, 0, '範囲で落とす', ['交通費の自動取得 ・ 通貨の換算', '別案件の要員の手配'], False),
             (0, 1, '意味で落とす', ['月次の締めの進め方', '試験環境の準備'], False),
             (1, 1, 'どちらでも落ちる', ['研修の案内 ・ 勤怠の締め'], False)]
    for c, r, verdict, items, hot in cells:
        x, y = X + c * CW, HY + r * RH
        w = 24 + len(verdict) * 15
        a += rect(x + 20, y + 18, w, 28, ACCENT if hot else PAPER, 14, ACCENT if hot else INK)
        a += text(x + 20 + w / 2, y + 37, verdict, 14, PAPER if hot else INK, 700, 'middle')
        a += text(x + 20, y + 74, items, 16, ACCENT if hot else DIM, 700 if hot else 400, gap=24)
    return svg('翌週の議事録に出た話を、意味（進行が止まるか）と範囲（今回の案件の切り替えまでか）で分ける。次のフェーズや別案件の話は意味では落とせず、範囲で落とす。その業務の中で解決できる話は意味で落ちる', a, HY + 2 * RH + 4)


def s3_draw_line():
    """範囲を、時期（本番の切り替えの前後）と案件（今回 ・ 別 ・ 全社）の地図で示す。PMが見たいのは、今回の案件で切り替えまでの枠の中。その枠を1文にして追記する。"""
    CUT = 660
    a = rect(170, 0, 942, 32, PAPER, 8, LINE)
    a += text(186, 22, '12月末の本番の切り替えまで', 14, INK, 700) + text(CUT + 16, 22, '切り替えの後', 14, INK, 700)
    lanes = [('今回の案件', 40, 100), ('別の案件', 150, 60), ('全社', 220, 60)]
    for name, y, h in lanes:
        a += rect(0, y, 1112, h, PAPER, 10, LINE)
        a += text(18, y + h / 2 + 6, name, 16, INK, 700)
    # 範囲：今回の案件で、切り替えまで
    # 範囲：今回の案件 × 本番の切り替えまで。範囲は境目であって、中に入る課題ではない
    a += rect(170, 48, 478, 84, PAPER, 10, ACCENT)
    a += rect(186, 60, 56, 24, ACCENT, 12, ACCENT) + text(214, 77, '範囲', 13, PAPER, 700, 'middle')
    a += text(254, 78, '今回の案件 × 12月末の切り替えまで', 17, ACCENT, 700)
    a += text(186, 106, '中に入る課題は、週ごとに変わる', 13, DIM, 700)
    a += text(186, 125, '例：翌週は 承認の経路 ・ 申請の締め日 ・ 既存データの移行', 13, DIM)
    for j, t in enumerate(['交通費の自動取得', '海外出張の通貨の換算']):
        a += rect(CUT + 20 + j * 214, 73, 200, 34, PAPER, 17, LINE) + text(CUT + 120 + j * 214, 95, t, 14, DIM, anchor='middle')
    a += rect(184, 163, 140, 34, PAPER, 17, LINE) + text(254, 185, '要員の手配', 14, DIM, anchor='middle')
    a += text(344, 185, 'ここでの「課題」は、別の担当者の課題', 14, ACCENT, 700)
    for i, t in enumerate(['研修の案内', '勤怠の締め']):
        a += rect(184 + i * 152, 233, 140, 34, PAPER, 17, LINE) + text(254 + i * 152, 255, t, 14, DIM, anchor='middle')
    a += path(f'M{CUT} 32 V280', ACCENT, 2).replace('stroke-linecap', 'stroke-dasharray="5 5" stroke-linecap')
    # 下：追記する1文
    a += rect(0, 298, 1112, 84, PAPER, 12, ACCENT)
    a += text(24, 322, 'プロンプトに追記する範囲', 13, DIM, 700)
    a += text(24, 350, '見るのは、今回の案件で、12月末の本番の切り替えまでに決めないと間に合わないものです。', 18, INK, 700)
    a += text(24, 372, '次のフェーズの話と、定例で出た別案件の話、全社の連絡事項は含めません。', 15, INK)
    return svg('範囲を、時期と案件の地図で示す。PMが見たいのは、今回の案件で本番の切り替えまでの枠の中。別案件での「課題」は、別の担当者にとっての課題である。この枠を1文にして追記する', a, 386)


def s3_after():
    """範囲を追記する前と後を、L3-S2 と同じ表で比べる。前に抽出した広さを破線、後を強調色の帯で示す。後は3回とも切り替えまでの話だけ。"""
    X, CW = 200, 228
    a = rect(0, 0, 1112, 268, PAPER, 12, LINE)
    for c, h in enumerate(['今回の案件、切り替えまで', '次のフェーズ', '別案件の相談', '全社の連絡']):
        x = X + c * CW
        a += text(x + 16, 40, h, 16, INK, 700)
        if c:
            a += path(f'M{x} 16 V252', LINE, 1).replace('stroke-linecap', 'stroke-dasharray="3 5" stroke-linecap')
    a += path('M16 64 H1096', LINE, 1.5)
    for r, (no, before, n, upto) in enumerate([('1回目', '5件', '5件', 1), ('2回目', '9件', '6件', 2), ('3回目', '14件', '5件', 4)]):
        y = 84 + r * 58
        a += text(20, y + 26, no, 16, INK, 700) + text(84, y + 26, before, 14, DIM) + text(126, y + 26, '→', 14, DIM) + text(150, y + 26, n, 18, ACCENT, 700)
        ghost = rect(X + 10, y, upto * CW - 20, 40, PAPER, 20, LINE)
        a += ghost.replace('stroke-width="2"', 'stroke-width="1.5" stroke-dasharray="5 5"')
        a += rect(X + 10, y + 5, CW - 20, 30, ACCENT, 15, ACCENT)
        a += text(X + 28, y + 26, 'ここまで', 14, PAPER, 700)
    a += rect(0, 288, 1112, 56, PAPER, 12, LINE)
    a += text(24, 322, '破線は、範囲を追記する前に抽出していた広さ。6件の回は、1件を2行に分けている', 15, INK)
    return svg('範囲を追記する前は回ごとに抽出する広さが違ったが、追記したあとは3回とも切り替えまでの話だけを抽出する。件数は5件・6件・5件', a, 348)
