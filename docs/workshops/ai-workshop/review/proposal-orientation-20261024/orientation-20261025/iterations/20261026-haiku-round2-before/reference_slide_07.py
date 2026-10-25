"""変更前の教材三つの並びと、06のSkill・Agentの関係をつなぐ。"""
from intro_visuals import *
from intro_visuals import _robot_at

def slide_07():
    a=''
    steps=[('教材1　目的と要件の明確化','search','何を実現し、何を満たすか','目的から、範囲と要件を考える'),('教材2　業務のモデル化と設計','book','業務をモデルに表し、設計へ','対象・判断・関係と守る条件'),('教材3　AI活用の実践と改善','robot','考え方をAIで実践する','Skillsにまとめ、使いながら改善')]
    for k,(tag,ic,head,sub) in enumerate(steps):
        x=k*380
        a+=rect(x,0,352,184,PAPER,12,LINE)
        a+=_robot_at(x+22,20,INK) if ic=='robot' else icon(x+22,22,ic,INK,.8)
        a+=text(x+88,49,tag,16,INK,700)
        a+=text(x+22,105,head,18,INK,700)
        a+=text(x+22,145,sub,16,DIM)
    # 教材1・2の下にSkills、教材3の下にAgentを置き、列ごとに真下へつなぐ。
    for x in (176,556,936):
        a+=path(f'M{x} 184 V236 M{x-6} 228 L{x} 236 L{x+6} 228',LINE,2)
    a+=rect(0,238,732,108,PAPER,12,LINE)+icon(24,256,'doc',INK,.7)
    a+=text(84,272,'Skills',22,INK,700)
    a+=text(84,302,'教材1・2の考え方を、判断の観点と確認の手順にまとめる',17,INK)
    a+=text(84,330,'まとめ方は教材3で学ぶ',15,DIM)
    a+=arrow(734,292,758,292)
    a+=rect(760,238,352,108,PAPER,12,ACCENT)+_robot_at(782,258,INK)
    a+=text(852,272,'Agent',22,INK,700)
    a+=text(852,302,'Skillsを使って作業・照合',17,INK)
    a+=band(370,'三つの教材を、共通の題材で順に学ぶ',54)
    return svg('教材1で目的と要件、教材2で業務のモデルと設計を考え、その考え方を教材3で判断の観点と確認の手順としてSkillsにまとめ、Agentが使って作業と照合を行う関係',a,430)
