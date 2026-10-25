"""作成を、確認・修正と完了まで含む仕事全体の中に置く。"""
from intro_visuals import *
from intro_visuals import _robot_at


def slide_02():
    """作る速さと、仕事全体の工数・期間・品質への効果を区別する。"""
    a=rect(0,0,1112,258,PAPER,12,LINE)
    a+=text(24,34,'仕事全体：作成から、確認・修正を経て完了するまで',20,INK,700)
    a+=rect(24,58,310,176,PAPER,10,ACCENT)
    a+=text(179,89,'資料・ツール・アプリ',20,INK,700,'middle')
    a+=_robot_at(58,111,INK)
    a+=icon(143,117,'doc',INK,.7)+icon(228,117,'screen',INK,.7)
    a+=text(179,211,'作りやすく、速く',24,ACCENT,700,'middle')
    a+=arrow(342,146,394,146)
    a+=rect(402,58,306,176,PAPER,10,LINE)
    a+=text(555,89,'確認・修正',20,INK,700,'middle')
    a+=person(471,143,.8,INK)
    a+=icon(529,117,'doc',INK,.7)+icon(605,118,'search',INK,.7)
    a+=text(555,211,'内容や品質を確かめる',20,INK,700,'middle')
    a+=arrow(716,146,768,146)
    a+=rect(776,58,312,176,PAPER,10,LINE)
    a+=text(932,89,'仕事の完了',20,INK,700,'middle')
    a+=icon(906,116,'check',INK,1.3)
    a+=text(932,211,'必要な条件を満たす',20,INK,700,'middle')

    a+=rect(0,272,1112,80,PAPER,12,ACCENT)
    a+=text(556,305,'工数・案件の期間・必要な品質',25,ACCENT,700,'middle')
    a+=text(556,337,'仕事の完了までを含めて、AI活用の効果を考える',19,INK,anchor='middle')
    return svg('仕事全体の境界の中に作成、確認と修正、完了を置く。AIで速くなった作成の部分と、仕事全体への効果を区別して問う',a,368)
