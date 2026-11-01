"""変更前i0_flowの時間軸と期間の部品を維持する。"""
from intro_visuals import i0_flow, icon, INK

def slide_10():
    a=i0_flow()
    a=a.replace(icon(22,270,'chat',INK,.55),icon(22,270,'check',INK,.55))
    for before,after in {
        '分からないことは、Teamsで相談できる':'何が変われば達成かも、先に決めておく',
        '使って直す':'実務で使い、改善する',
        '目的を達成する仕組みを、':'自分の仕事で使う仕組みを、',
    }.items():
        a=a.replace(before,after)
    return a
