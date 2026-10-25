"""既存の人物、教材、仕事の図で背景とまとめを示す。"""
from intro_visuals import *
from intro_visuals import _robot_at,i0_why

def opening():
    a=text(0,23,'開催の背景',17,DIM,700)
    a+=rect(0,38,352,286,PAPER,12,LINE)
    a+=_robot_at(35,65,INK)
    a+=icon(143,69,'doc',INK,.8)+icon(232,69,'screen',INK,.8)
    a+=text(25,161,['AIで、これまで難しかった','ことも実現できるように'],21,INK,700,gap=30)
    a+=path('M25 212 H327',LINE,1)
    a+=text(25,244,['この力を、仕事で実現したい','ことに合わせて使いたい'],18,INK,gap=30)
    a+=arrow(362,181,394,181)
    a+=text(404,23,'ワークショップの位置づけ',17,DIM,700)
    a+=rect(404,38,708,286,PAPER,12,ACCENT)
    a+=text(428,75,'考え方を学び、自分の仕事で実践する',23,INK,700)
    a+=rect(420,100,278,154,PAPER,10,LINE)+rect(756,100,338,154,PAPER,10,LINE)
    a+=icon(437,117,'book',INK,.75)+text(501,143,'考え方を学ぶ',20,INK,700)
    a+=text(438,193,['何を実現したいか','そのために何が必要か'],18,INK,gap=30)
    a+=arrow(698,177,756,177)
    a+=_robot_at(777,116,INK)+text(845,143,'AIで実践する',20,INK,700)
    a+=text(776,193,['Agents・Skillsで実行し','結果を確かめる'],18,INK,gap=30)
    a+=text(428,294,'部内のAI活用を進めるための、学習と実践の場',18,DIM)
    a+=band(350,'目指す姿：自分の仕事の目的を、AIで達成できる',54)
    return svg('AIで実現できることが増えた背景から、目的と必要な条件を考え、Agents・Skillsで実行して結果を確かめる学習と実践へ進み、仕事の目的をAIで達成できる姿を目指す',a,418)

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
