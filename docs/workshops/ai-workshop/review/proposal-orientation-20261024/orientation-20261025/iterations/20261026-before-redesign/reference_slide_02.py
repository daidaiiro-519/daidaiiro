"""変更前i0_whyの配置と部品を維持し、文字とアイコンの種類を変更する。"""
from intro_visuals import *
from intro_visuals import _robot_at

def slide_02():
    """AIで作りやすくなった効果を認め、確認と修正を含めた仕事全体の改善を問う。"""
    steps = [('AIで変わったこと', 'robot', '作りやすく、速く', '資料・ツール・アプリなど', ['作れなかったものも試せる', '出力を得るまでの時間も短く']), ('確認・修正も含む仕事', 'doc', '内容と品質を確認する', '必要なら修正して、実務で使う', ['必要な内容や条件を満たすか', '結果を確かめ、必要なら直す']), ('仕事全体への効果', 'check', '工数・期間・品質は？', '確認・修正まで含めて考える', ['仕事を終えるまでの工数', '案件の進む速さ・必要な品質'])]
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
    a += text(cx, cy - 8, '皆さんの', 21, PAPER, 700, 'middle')
    a += text(cx, cy + 24, '仕事では？', 21, PAPER, 700, 'middle')
    return svg('作りやすさと速さから、確認と修正を含む仕事全体の工数・期間・品質へ視野を広げ、受講者の仕事ではどう変わったかを問う', a, 324)
