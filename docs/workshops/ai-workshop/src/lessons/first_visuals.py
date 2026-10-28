"""入力と出力例の不足を示す。教材の既存部品とwarm-paperの配色を使う。"""
from lesson_visuals import text, rect, path, circle, arrow, svg, INK, DIM, LINE, PAPER, ACCENT
from intro_visuals import _robot_at, _table_icon

W, H = 1112, 400

def _sheet(x, y, w, h):
    fold = 22
    a = path(f'M{x} {y} H{x+w-fold} L{x+w} {y+fold} V{y+h} H{x} Z', LINE, 2, PAPER)
    return a + path(f'M{x+w-fold} {y} V{y+fold} H{x+w}', LINE, 2)

def _flow(a, input_y, output_y):
    a += _robot_at(402, 155, INK) + text(426, 236, 'AI', 20, INK, 700, 'middle')
    a += path(f'M332 {min(input_y)} V320', LINE, 2)
    for y in input_y + [320]:
        a += path(f'M310 {y} H332', LINE, 2)
    a += circle(332, 182, 2.5, LINE, LINE) + arrow(332, 182, 404, 182)
    if len(output_y) == 1:
        a += arrow(448, 182, 544, 182)
    else:
        a += path(f'M448 182 H510 M510 {min(output_y)} V{max(output_y)}', LINE, 2)
        for y in output_y:
            a += circle(510, y, 2.5, LINE, LINE) + arrow(510, y, 544, y)
    return a

def _prompt(lines):
    a = rect(0, 268, 310, 112, PAPER, 12, ACCENT)
    a += text(155, 285, 'AIに伝えるプロンプト', 13, DIM, 700, 'middle')
    # 1行と3行でも、文の中央が同じ位置になるようにする。
    first_y = 335 - (len(lines) - 1) * 13
    return a + text(155, first_y, lines, 20 if len(lines) == 1 else 18,
                    ACCENT, 700, 'middle', gap=26)

def _table(x, y, w, h, heading, columns, rows):
    a = _sheet(x, y, w, h) + text(x+18, y+30, heading, 19, INK, 700)
    positions = [x+18, x+250] if len(columns) == 2 else [x+18, x+230, x+390]
    for px, label in zip(positions, columns):
        a += text(px, y+60, label, 15, DIM, 700)
    a += path(f'M{x+16} {y+70} H{x+w-16}', LINE, 1)
    for j, row in enumerate(rows):
        for px, label in zip(positions, row):
            a += text(px, y+96+j*28, label, 16, INK)
    return a



def ends():
    # 2つの例の間に問いを置く。軸は例だけにつなぎ、問いには目印を付けない。
    a = text(0, 22, '抽象', 20, INK, 700) + text(0, 45, '一般的な', 14, DIM) + text(0, 65, '作業', 14, DIM)
    a += text(0, 343, '具体', 20, INK, 700) + text(0, 366, '個別の指定', 14, DIM)
    a += path('M80 372 V0 m-7 8 l7 -8 7 8 M73 364 l7 8 7 -8', LINE, 2)
    for y, heading, sub, result, detail in [
        (0, '「課題を整理して」', '作業は伝えるが、目的と必要な内容がない', 'どの週にも使える文だが、', '欲しかった担当と期限が入らない'),
        (250, '承認の経路・経理と開発・9月16日を指定', 'その週の項目と値まで決める', '今週の不足は直せるが、', '翌週には指定を変える必要がある'),
    ]:
        a += circle(80, y+44, 7, PAPER, LINE) + path(f'M87 {y+44} H100', LINE, 1)
        a += rect(100, y, 560, 88, PAPER, 12, LINE)
        a += text(124, y+36, heading, 23 if y == 0 else 20, INK, 700)
        a += text(124, y+68, sub, 17, INK)
        a += rect(676, y, 436, 88, PAPER, 12, LINE)
        a += text(696, y+32, result, 19, INK, 700)
        a += text(696, y+66, detail, 18, INK)
    box = rect(100, 122, 560, 88, PAPER, 12, ACCENT)
    a += box.replace('stroke-width="2"', 'stroke-width="2.5" stroke-dasharray="7 5"')
    a += text(124, 158, '？ 課題を整理するのは、何のためか', 22, ACCENT, 700)
    a += text(124, 190, 'PMが課題管理表を何に使うかを確かめる', 17, INK)
    a += rect(676, 122, 436, 88, PAPER, 12, LINE)
    a += text(696, 155, '今回必要な内容', 15, DIM, 700)
    a += text(696, 188, '毎週、担当と期限を含む課題管理表', 18, INK, 700)
    a += text(100, 394, '一般的な作業と個別の指定の比較。目的の大きさは、実現したいことに合わせる。', 17, INK)
    return svg('上下の2つの入力例を抽象と具体で比較し、中央に目的の問いを置く。問いは軸につながない', a, H)

def _simple_flow(a, lines):
    a += rect(364, 44, 316, 108, PAPER, 12, ACCENT)
    a += text(522, 107 - (len(lines)-1)*15, lines, 25 if len(lines)==1 else 20, INK, 700, 'middle', gap=30)
    a += arrow(300, 96, 364, 96)
    a += path('M522 152 V206 M515 198 l7 8 7 -8', LINE, 2)
    a += rect(404, 206, 236, 108, PAPER, 12, LINE)
    a += _robot_at(426, 236, INK)
    a += text(500, 263, 'AIが議事録を', 16, INK, 700)
    a += text(500, 289, '整理する', 16, INK, 700)
    a += arrow(640, 260, 720, 260)
    return a


def ask():
    a = _sheet(0, 16, 300, 300) + rect(0, 16, 300, 300, 'none', 0, 'none')
    a += text(20, 52, '今週の議事録（抜粋）', 20, INK, 700)
    a += text(20, 96, '承認の経路', 25, INK, 700)
    a += text(20, 143, '担当', 17, DIM, 700) + text(86, 143, '経理と開発', 22, INK, 700)
    a += text(20, 184, '期限', 17, DIM, 700) + text(86, 184, '9月16日', 22, INK, 700)
    a += path('M20 219 H280', LINE, 1)
    a += text(20, 255, ['議事録には、', '担当と期限も書かれている'], 17, DIM, gap=27)
    a = _simple_flow(a, ['「課題を整理して」'])
    a += rect(720, 16, 392, 300, PAPER, 12, LINE)
    a += text(744, 52, '課題管理表の出力例（抜粋）', 19, INK, 700)
    a += text(744, 91, '課題', 16, DIM, 700) + text(944, 91, '概要', 16, DIM, 700)
    a += path('M744 104 H1088', LINE, 1)
    a += text(744, 140, '承認の経路', 21, INK) + text(944, 140, '未確定', 21, INK)
    a += path('M744 170 H1088', LINE, 1)
    for x, label in [(744, '担当'), (922, '期限')]:
        a += rect(x, 192, 166, 98, PAPER, 10, ACCENT).replace('stroke-width="2"','stroke-width="1.5" stroke-dasharray="5 5"')
        a += text(x + 83, 227, label, 23, INK, 700, 'middle')
        a += text(x + 83, 269, '出力にない', 21, ACCENT, 700, 'middle')
    return svg('一つの代表例で、議事録にあった担当と期限が出力には含まれないことを示す。破線の枠はPMが欲しかった内容', a, 332)


def symptoms():
    a = _sheet(0, 16, 300, 140) + rect(0, 16, 300, 140, 'none', 0, 'none')
    a += text(20, 48, '今週の議事録', 20, INK, 700)
    a += text(20, 88, '承認の経路', 23, INK, 700)
    a += text(20, 128, '経理と開発／9月16日', 20, INK)
    a += _sheet(0, 176, 300, 140) + rect(0, 176, 300, 140, 'none', 0, 'none')
    a += text(20, 208, '翌週の議事録', 20, INK, 700)
    a += text(20, 250, ['課題・担当・期限が', '変わる'], 23, INK, 700, gap=32)
    a = _simple_flow(a, ['承認の経路は、経理と開発が', '9月16日までに決める、', 'と表に入れて'])
    a += rect(720, 16, 392, 300, PAPER, 12, LINE)
    a += text(744, 52, '今週の出力例（抜粋）', 20, INK, 700)
    for x, h, t in [(744, '課題', '承認の経路'), (870, '担当', '経理と開発'), (1000, '期限', '9月16日')]:
        a += text(x, 91, h, 16, DIM, 700)
        a += text(x, 140, t, 18, INK, 700)
    a += path('M744 104 H1088', LINE, 1)
    a += rect(744, 192, 344, 98, PAPER, 10, ACCENT)
    a += text(760, 226, '翌週にも使うには、', 20, INK)
    a += text(760, 269, '指定を変える必要がある', 24, ACCENT, 700)
    return svg('3枚目と同じ配置で、今週の具体的な指定は不足を補うが、翌週は項目と担当と期限が変わり、指定の修正が必要と示す', a, 332)
