"""変更前の教材三つの並びと、06のSkill・Agentの関係をつなぐ。"""
from intro_visuals import *
from intro_visuals import _robot_at

def slide_07():
    a=''
    steps=[('教材1　目的の把握','search','何のために、何をどこまで作るか','作業の目的と必要な内容を決める'),('教材2　言葉の定義','book','言葉が何を指すかを明確にする','必要な内容や条件を定義する'),('教材3　仕組みの構築','robot','AIで実行し、改善する','Agents・Skillsの使い方を学ぶ')]
    for k,(tag,ic,head,sub) in enumerate(steps):
        x=k*380
        a+=rect(x,0,352,184,PAPER,12,LINE)
        a+=_robot_at(x+22,20,INK) if ic=='robot' else icon(x+22,22,ic,INK,.8)
        a+=text(x+88,49,tag,16,INK,700)
        a+=text(x+22,105,head,18,INK,700)
        a+=text(x+22,145,sub,16,DIM)
    # 教材1・2はSkillに記録する前提、教材3はそれを使う実行と改善へ。
    a+=path('M176 184 V220 H556 V184 M408 220 V244 M402 236 L408 244 L414 236',LINE,2)
    a+=path('M936 184 V244 M930 236 L936 244 L942 236',LINE,2)
    a+=rect(192,244,436,88,PAPER,12,LINE)+icon(210,263,'doc',INK,.65)
    a+=text(267,276,'Skillに記録する前提',20,INK,700)
    a+=text(267,306,'目的・内容・言葉の意味・確認条件',16,INK)
    a+=arrow(638,288,776,288)+text(707,269,'使う',14,DIM,anchor='middle')
    a+=rect(786,244,326,88,PAPER,12,ACCENT)+_robot_at(804,260,INK)
    a+=text(872,278,'Agent',23,INK,700)+text(872,309,'Skillを使って作成・改善',16,INK)
    a+=band(370,'目的と言葉を明確にし、AIで使える仕組みにする',54)
    return svg('教材1と教材2で考える目的・内容・言葉の意味・確認条件をSkillに記録し、教材3でAgentが使う仕組みと改善を学ぶ関係',a,430)
