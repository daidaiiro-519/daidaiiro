"""変更前i0_whyの配置と部品を維持し、文字とアイコンの種類を変更する。"""
from intro_visuals import *
from intro_visuals import _robot_at

def slide_02():
    """作業の目的と言葉の意味を言語化し、仕組みにして改善する。3つの方法を教材で1つずつ学び、目的をAIで達成する。"""
    steps = [('AIによる成果物の作成', 'robot', '成果物を作りやすい', '資料・ツール・アプリなど', ['作成しやすくなった', '出力を得るまでの時間も短く']), ('仕事で使うまで', 'doc', '内容と品質を確認する', '必要なら修正して、実務で使う', ['作成だけでなく、確認や修正', '利用までの作業を含めて見る']), ('仕事全体の効率・生産性', 'check', '全体の進み方を見る', '工数と案件の期間にも注目する', ['作業を終える工数は減ったか', '案件を進める速さは変わったか'])]
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
    a += text(cx, cy - 8, '仕事全体の', 21, PAPER, 700, 'middle')
    a += text(cx, cy + 24, '生産性は？', 21, PAPER, 700, 'middle')
    return svg('作成、確認と修正、仕事全体への効果を区別して問う', a, 324)
