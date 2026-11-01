"""3本目（範囲を決める）の試作の図。議事録の中に混ざっている話を、どこまで対象にするか。"""
from lesson_visuals import (text, rect, path, arrow, circle, person, icon, svg,
                            INK, DIM, LINE, PANEL, PAPER, ACCENT, W, cross, check, band)
from intro_visuals import _robot_at


def _sheet(x, y, w, h, stroke=LINE):
    """角を折った書類の形。矢印の付け根を判定できるよう、同じ大きさの見えない枠を重ねる。"""
    d = f'M{x} {y} H{x + w - 22} L{x + w} {y + 22} V{y + h} H{x} Z M{x + w - 22} {y} V{y + 22} H{x + w}'
    return path(d, stroke, 2, PAPER) + rect(x, y, w, h, 'none', 0, 'none')






def _scope_columns(a, bottom):
    """範囲を比較する枚では、対象の列と境界を同じ位置に置く。"""
    for i, (head, sub, items) in enumerate([
        ('今回の案件', '本番の切り替えまで', ['承認の経路／申請の締め日', '既存データの移行対象']),
        ('今回の案件', '次のフェーズ', ['交通費の自動取得', '海外出張の通貨の換算']),
        ('別案件', '作業の開始まで', ['要員の手配'])]):
        x = 248 + i * 288
        a += text(x + 18, 34, head, 19, INK, 700)
        a += text(x + 18, 63, sub, 16, INK)
        a += text(x + 18, 94, items, 14, DIM, gap=23)
        if i:
            a += path(f'M{x} 16 V{bottom}', LINE, 1).replace('stroke-linecap', 'stroke-dasharray="3 5" stroke-linecap')
    a += path('M16 136 H1096', LINE, 1)
    return a


def s3_ranges():
    a = rect(0, 0, 1112, 312, PAPER, 12, LINE)
    a = _scope_columns(a, 294)
    a += text(24, 196, '出力例に含む', 18, INK, 700)
    a += rect(260, 164, 840, 48, PAPER, 24, LINE)
    a += text(280, 195, 'この3つの対象の事項が含まれている', 18, INK, 700)
    a += text(24, 274, 'PMが必要とする', 18, INK, 700)
    a += rect(260, 242, 264, 48, ACCENT, 24, ACCENT)
    a += text(392, 273, '今回の対象はここ', 18, PAPER, 700, 'middle')
    a += text(566, 273, '次のフェーズと別案件は、今回の対象外', 18, DIM)
    a += rect(0, 332, 1112, 58, PAPER, 12, LINE)
    a += text(24, 368, '定義には合う：どの事項も、決定しないと各案件・フェーズの進行が止まる', 18, INK, 700)
    return svg('3つの対象の事項を含む出力例の帯と、PMが必要とする切り替えまでの事項の帯を比較する。どの事項も課題の定義には合う', a, 398)


def s3_after():
    a = rect(0, 0, 1112, 314, PAPER, 12, LINE)
    a = _scope_columns(a, 296)
    a += text(24, 186, '意味だけの例', 18, INK, 700)
    a += rect(260, 156, 840, 46, PAPER, 23, LINE)
    a += text(280, 186, '次のフェーズと別案件の事項も含む', 18, INK, 700)
    a += text(24, 271, '意味と範囲の例', 18, INK, 700)
    a += rect(260, 238, 840, 46, PAPER, 23, LINE).replace('stroke-width="2"', 'stroke-width="1.5" stroke-dasharray="5 5"')
    a += rect(260, 238, 264, 46, ACCENT, 23, ACCENT)
    a += text(392, 268, '今回の対象だけ', 18, PAPER, 700, 'middle')
    a += text(566, 268, 'この例では、対象外の事項を含まない', 18, DIM)
    a += rect(0, 334, 1112, 58, PAPER, 12, LINE)
    a += text(24, 369, '必要な課題に漏れはないか、対象外の課題が残っていないか、議事録と照合する。', 17, INK)
    return svg('同じ対象の列で説明用の前後の出力例を比較する。上は3つの対象を含み、下は切り替えまでの今回の案件だけを含む。破線は対象外となった部分。課題の漏れも確認する', a, 400)



def s3_two_roles():
    """意味と範囲は別のもの。翌週の議事録に出た話を、意味（進行が止まるか）と範囲（今回の案件の切り替えまでか）で分けた2行2列の表。枠を重ねない。"""
    X, CW, HY, RH = 230, 441, 70, 124
    a = rect(0, 0, 1112, HY + 2 * RH, PAPER, 12, LINE)
    a += rect(X + 12, HY + 10, CW - 24, RH - 20, PANEL, 10, ACCENT)
    a += text(20, 40, '意味 × 範囲', 18, INK, 700)
    a += path(f'M16 {HY} H1096 M16 {HY + RH} H1096', LINE, 1.5)
    a += path(f'M{X} 16 V{HY + 2 * RH - 16} M{X + CW} 16 V{HY + 2 * RH - 16}', LINE, 1.5)
    for c, (h, sub) in enumerate([('範囲の内', '今回の案件、12月末の切り替えまで'), ('範囲の外', '次のフェーズ ・ 別案件 ・ 全社')]):
        x = X + c * CW
        a += text(x + 20, 32, h, 17, INK, 700) + text(x + 20, 54, sub, 13, DIM)
    for r, (h, sub) in enumerate([('定義に合う', '決定しないと進行が止まる'), ('定義に合わない', '対応済み・進行に影響なし')]):
        y = HY + r * RH
        a += text(20, y + 56, h, 16, INK, 700) + text(20, y + 80, sub, 13, DIM)
    cells = [(0, 0, '抽出する', ['承認の経路 ・ 申請の締め日', '既存データの移行対象'], True),
             (1, 0, '対象外', ['交通費の自動取得 ・ 通貨の換算', '別案件の要員の手配'], False),
             (0, 1, '定義に合わない', ['月次の締めの進め方', '試験環境の準備'], False),
             (1, 1, 'どちらにも合わない', ['研修の案内 ・ 勤怠の締め'], False)]
    for c, r, verdict, items, hot in cells:
        x, y = X + c * CW, HY + r * RH
        w = 24 + len(verdict) * 15
        a += rect(x + 20, y + 18, w, 28, ACCENT if hot else PAPER, 14, ACCENT if hot else INK)
        a += text(x + 20 + w / 2, y + 37, verdict, 14, PAPER if hot else INK, 700, 'middle')
        a += text(x + 20, y + 74, items, 16, ACCENT if hot else DIM, 700 if hot else 400, gap=24)
    return svg('翌週の議事録に出た話を、意味（進行が止まるか）と範囲（今回の案件の切り替えまでか）で分ける。次のフェーズや別案件の事項は定義に合っても対象外。対応済みで進行に影響しない事項は定義に合わない', a, HY + 2 * RH + 4)





def s3_intro():
    from bridge_visuals import intro
    return intro('前の動画で決めた意味',
        [('意味', '課題とは、決定しないと案件の進行が止まる事項'), ('範囲', '対象の案件と時期は、まだ決まらない')],
        'この動画で決める範囲', 2, 2, '図の出力は、説明用の例',
        [('出力例を確認する', '対象外の事項を見る'), ('意味と範囲を比べる', '定義と対象を区別する'),
         ('目的に合う対象を決める', '案件と時期を決める'), ('出力を確認する', '対象外の事項と漏れを見る')],
        '利用目的に合わせて範囲を決め、各週の具体的な出力で確かめる',
        '前の定義と今回の対象案件と時期を比べ、4つの手順を示す',
        focus=[('どの案件か', '今回の案件？　別案件？'), ('いつまでか', '切り替えまで？　その後？')])


def s3_mixed():
    a = _sheet(0, 0, 440, 334)
    a += text(20, 35, '定例の議事録', 23, INK, 700)
    a += text(20, 65, '翌週の1回分', 16, DIM)
    for y, head, sub in [(112, '今回の案件の話', '本番の切り替えまで'),
                          (198, '次のフェーズの話', '切り替えの後'),
                          (284, '別案件の相談', '他の案件の作業開始まで')]:
        a += text(20, y, head, 25, INK, 700)
        a += text(20, y + 29, sub, 17, DIM)
    a += arrow(440, 162, 476, 162)
    a += rect(476, 96, 160, 132, PAPER, 12, LINE)
    a += _robot_at(528, 106, INK)
    a += text(556, 187, '意味は伝えた', 16, INK, 700, 'middle')
    a += text(556, 214, '範囲は未記載', 15, DIM, anchor='middle')
    a += arrow(636, 162, 672, 162)
    a += rect(672, 0, 440, 334, PAPER, 12, LINE)
    a += text(692, 35, '出力例（抜粋）', 23, INK, 700)
    a += rect(688, 66, 408, 106, PAPER, 10, LINE)
    a += text(704, 97, '今回の表に必要', 17, DIM, 700)
    a += text(704, 140, '承認の経路 など', 27, INK, 700)
    a += rect(688, 190, 408, 128, PAPER, 10, ACCENT)
    a += text(704, 221, '今回の対象外も含む', 17, ACCENT, 700)
    a += text(704, 264, '交通費の自動取得', 25, ACCENT, 700)
    a += text(704, 301, '別案件の要員の手配', 25, ACCENT, 700)
    a += text(24, 376, '必要なのは、今回の案件の本番の切り替えまでの課題。', 23, INK, 700)
    return svg('意味は伝えても範囲を伝えていないと、対象外の課題が出力に含まれる例。議事録の話の種類と代表的な出力だけを示す', a, 392)



def s3_draw_line():
    a = text(24, 26, 'PMの目的：本番の切り替えに向けて、課題の優先順位を判断する', 20, INK, 700)
    CUT = 660
    a += rect(170, 48, 942, 32, PAPER, 8, LINE)
    a += text(18, 71, '案件 × 時期', 17, DIM, 700)
    a += text(186, 70, '12月末の本番の切り替えまで', 15, INK, 700)
    a += text(CUT + 16, 70, '切り替えの後', 15, DIM, 700)
    for name, y, h in [('今回の案件', 90, 98), ('別案件', 200, 50), ('全社', 262, 50)]:
        a += rect(0, y, 1112, h, PAPER, 10, LINE)
        a += text(18, y + h / 2 + 6, name, 17, INK, 700)
    a += rect(170, 98, 478, 82, PAPER, 10, ACCENT)
    a += rect(186, 110, 56, 24, ACCENT, 12, ACCENT)
    a += text(214, 127, '範囲', 13, PAPER, 700, 'middle')
    a += text(254, 128, '今回の案件 × 本番の切り替えまで', 18, ACCENT, 700)
    a += text(186, 152, 'この範囲を使い、中の課題は週ごとに変わる', 15, DIM)
    a += text(186, 174, '例：承認の経路／申請の締め日／既存データの移行対象', 13, DIM)
    for j, t in enumerate(['交通費の自動取得', '海外出張の通貨の換算']):
        a += rect(CUT + 20 + j * 214, 125, 200, 34, PAPER, 17, LINE)
        a += text(CUT + 120 + j * 214, 147, t, 14, DIM, anchor='middle')
    a += rect(184, 208, 140, 34, PAPER, 17, LINE)
    a += text(254, 230, '要員の手配', 14, DIM, anchor='middle')
    a += text(344, 231, '案件が対象外', 17, DIM)
    for i, t in enumerate(['研修の案内', '勤怠の締め']):
        a += rect(184 + i * 152, 270, 140, 34, PAPER, 17, LINE)
        a += text(254 + i * 152, 292, t, 14, DIM, anchor='middle')
    a += path(f'M{CUT} 80 V312', ACCENT, 2).replace('stroke-linecap','stroke-dasharray="5 5" stroke-linecap')
    a += rect(0, 330, 1112, 76, PAPER, 12, ACCENT)
    a += text(24, 353, 'AIに伝える範囲', 13, DIM, 700)
    a += text(24, 378, '今回の案件で、12月末の本番の切り替えまでに決定が必要な事項を対象とします。', 18, INK, 700)
    a += text(24, 400, '次のフェーズの話と、定例で出た別案件の話、全社の連絡事項は含めません。', 15, INK)
    return svg('上のPMの目的に合わせ、案件と時期の交点を範囲として示す。枠の中の課題は週ごとに変わる。下はAIに伝える範囲', a, 416)

def s3_end():
    a = rect(0, 0, 1112, 86, PAPER, 12, ACCENT)
    a += text(24, 35, '定義に合い、今回の対象に入る事項を抽出する', 25, INK, 700)
    a += text(24, 68, '共通の基準を使い、必要な課題と漏れを各週の出力で確認する', 18, DIM)
    a += _sheet(0, 108, 620, 228)
    a += text(20, 138, '今回の作業で決めた内容', 15, DIM, 700)
    rows = [('目的', 'PMが優先順位を判断する', False), ('意味', '決定しないと案件の進行が止まる事項', False),
            ('範囲', '今回の案件、12月末の本番の切り替えまで', True)]
    for i, (tag, body, hot) in enumerate(rows):
        y = 162 + i * 36
        a += rect(20, y, 104, 28, ACCENT if hot else PAPER, 8, ACCENT if hot else LINE)
        a += text(72, y + 20, tag, 15, PAPER if hot else INK, 700, 'middle')
        a += text(144, y + 21, body, 17, INK, 700 if hot else 400)
    a += rect(20, 270, 104, 28, PAPER, 8, LINE).replace('stroke-width="2"', 'stroke-width="1.5" stroke-dasharray="4 4"')
    a += text(72, 290, '条件', 15, DIM, 700, 'middle')
    a += text(144, 291, '次に、担当者・期限などを決める', 17, DIM)
    a += rect(652, 108, 460, 228, PAPER, 12, LINE)
    a += text(676, 138, '課題管理表の出力例（抜粋）', 15, DIM, 700)
    a += text(676, 170, '対象の課題はあるが、必要な情報が足りない', 17, INK, 700)
    a += text(676, 198, '課題', 15, DIM, 700)
    a += text(951, 198, '担当者', 15, DIM, 700, 'middle')
    a += text(1045, 198, '期限', 15, DIM, 700, 'middle')
    a += path('M676 210 H1088', LINE, 1)
    for i, t in enumerate(['承認の経路', '申請の締め日']):
        y = 238 + i * 36
        a += text(676, y, t, 18, INK)
        for x in [917, 1011]:
            a += rect(x, y - 21, 68, 28, PAPER, 6, ACCENT).replace('stroke-width="2"','stroke-width="1.5" stroke-dasharray="4 4"')
            a += text(x + 34, y - 1, '未記載', 14, ACCENT, 700, 'middle')
    a += text(676, 313, '誰が、いつまでに決めるかも必要', 18, INK, 700)
    a += rect(0, 354, 1112, 48, PAPER, 24, LINE)
    a += text(24, 385, '次の動画　条件を決める', 19, INK, 700)
    a += text(420, 385, '各課題に含める内容などを、目的に合わせて決める', 18, DIM)
    return svg('左は決めた内容と次の条件。右の説明用の出力例では対象の課題はあるが、担当者と期限が未記載で、次の条件が必要と分かる', a, 410)
