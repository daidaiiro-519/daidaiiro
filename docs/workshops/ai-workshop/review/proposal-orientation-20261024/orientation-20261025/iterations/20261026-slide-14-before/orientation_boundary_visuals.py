"""既存の人物、教材、仕事の図で背景とまとめを示す。"""
from intro_visuals import *
from intro_visuals import _robot_at,i0_why

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
    a+=text(776,158,['判断の観点と手順をSkillsに','Agentsで実行し、結果を確かめる','Skillsと進め方を改善する'],18,INK,gap=28)
    a+=band(296,'目指す姿：自分の仕事に合ったSkillsを作り、Agentsで作業を進められる',54)
    return svg('AIで実現できることが増えた背景から、三つの教材で学び、自分の仕事で判断の観点と手順をSkillsにしてAgentsで実践する。自分の仕事に合ったSkillsを作り、Agentsで作業を進められる姿を目指す',a,350)

def closing():
    a=i0_why().replace('>目的を</text>','>目的の達成に</text>').replace('>AIで達成する</text>','>AIを使う</text>')
    mapping={
      '教材1　目的の把握':'目的の把握',
      '教材2　言葉の定義':'言葉の定義',
      '教材3　仕組みの構築':'仕組みの構築',
      '例：優先順位を付けるために、':'自分の仕事で、',
      '課題を整理する':'何を実現したいかを考える',
      '例：「課題」が経理 ・ 開発 ・ PMで':'必要な内容や条件が、',
      '何を指すかを言葉にする':'何を指すかを説明する',
      '例：使いながらずれを直し、':'実務で確認し、',
      '課題管理表を確認して改善する':'条件や手順を改善する',
    }
    for before,after in mapping.items():a=a.replace(before,after)
    a=a.replace('viewBox="0 0 1112 324"','viewBox="0 0 1112 430"')
    footer=rect(0,344,1112,70,PAPER,12,LINE)
    footer+=text(22,373,'次の動画「はじめに」',18,INK,700)
    footer+=text(22,400,'共通の題材と、教材1〜3の学習の流れ',18,INK)
    return a.replace('</svg>',footer+'</svg>')
