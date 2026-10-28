"""既存の人物、教材、仕事の図で背景とまとめを示す。"""
from intro_visuals import *
from intro_visuals import _robot_at

def opening():
    """開催の背景から、ワークショップの位置づけと目指す姿へ進む。"""
    a=text(0,23,'開催の背景',17,DIM,700)
    a+=rect(0,38,352,240,PAPER,12,LINE)
    a+=_robot_at(35,58,INK)
    a+=icon(143,62,'doc',INK,.8)+icon(232,62,'screen',INK,.8)
    a+=text(25,148,['AIで、これまで難しかった','ことも実現できるように'],21,INK,700,gap=30)
    a+=path('M25 194 H327',LINE,1)
    a+=text(25,226,['この力を、自分の仕事の','改善に活かしたい'],18,INK,gap=30)
    a+=arrow(362,158,394,158)
    a+=text(404,23,'ワークショップの位置づけ',17,DIM,700)
    a+=rect(404,38,708,240,PAPER,12,ACCENT)
    a+=rect(420,60,278,196,PAPER,10,LINE)+rect(756,60,338,196,PAPER,10,LINE)
    a+=icon(437,82,'book',INK,.75)+text(501,108,'教材で学ぶ',20,INK,700)
    a+=text(438,158,['目的と要件の明確化','業務のモデル化と設計','AI活用の実践と改善'],18,INK,gap=28)
    a+=arrow(698,158,756,158)
    a+=_robot_at(777,80,INK)+text(845,108,'自分の仕事で実践する',20,INK,700)
    a+=text(776,158,['判断の観点と手順をSkillsにする','Agentで実行し、結果を確かめる','Skillsと作業の進め方を改善する'],18,INK,gap=28)
    a+=band(296,'目指す姿：自分の仕事に合ったSkillsを作り、Agentで作業を進められる',54)
    return svg('AIで実現できることが増えた背景から、三つの教材で学び、自分の仕事で判断の観点と手順をSkillsにしてAgentで実践する。自分の仕事に合ったSkillsを作り、Agentで作業を進められる姿を目指す',a,350)

def closing():
    """三つの教材から、表紙と同じ到達点へつなぎ、次の動画を案内する。"""
    steps=[('教材1　目的と要件の明確化','search','何を実現し、何を満たすか','目的から、範囲と要件を考える',['作業の目的に合わせて、','満たす要件を明確にする']),
           ('教材2　業務のモデル化と設計','book','業務をモデルに表し、設計へ','対象・判断・関係と守る条件',['業務のルールと設計の','対応を確かめる']),
           ('教材3　AI活用の実践と改善','robot','考え方をAIで実践する','Skillsにまとめ、Agentで使う',['結果を確かめ、','Skillsと作業の進め方を改善する'])]
    a=''
    cw,gap=280,36
    for k,(tag,ic,name,what,ex) in enumerate(steps):
        x=k*(cw+gap)
        a+=rect(x,0,cw,284,PAPER,12,LINE)
        a+=text(x+22,32,tag,14,DIM,700)
        a+=_robot_at(x+22,52,INK) if ic=='robot' else icon(x+22,52,ic,INK,.9)
        a+=text(x+22,140,name,18,INK,700)
        a+=text(x+22,170,what,15,INK)
        a+=path(f'M{x+22} 196 H{x+cw-22}',LINE,1)
        a+=text(x+22,226,ex,14,DIM,gap=24)
        a+=arrow(x+cw+6,142,x+cw+gap-6,142)
    cx,cy,r=1112-84,142,84
    a+=circle(cx,cy,r,ACCENT,ACCENT)
    a+=text(cx,cy-24,['仕事全体の','生産性と品質を','高める'],18,PAPER,700,'middle',gap=28)
    a+=rect(0,304,1112,50,PAPER,12,LINE)
    a+=text(22,336,'次の動画「はじめに」',18,INK,700)+text(220,336,'共通の題材と、教材1〜3の学習の流れ',17,INK)
    return svg('三つの教材で目的と要件、業務のモデル化と設計、AIの作業での実践と改善を学び、仕事全体の生産性と品質を高める。次の動画「はじめに」で共通の題材を紹介する',a,360)
