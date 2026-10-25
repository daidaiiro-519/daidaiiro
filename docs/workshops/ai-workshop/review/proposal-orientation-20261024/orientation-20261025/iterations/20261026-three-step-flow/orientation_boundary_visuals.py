"""既存の人物、教材、仕事の図で背景とまとめを示す。"""
from intro_visuals import *
from intro_visuals import _robot_at,i0_why

def opening():
    """ワークショップで行う学習・まとめ・実践と、結果から改善する戻りを示す。"""
    a=rect(0,0,1112,322,PAPER,12,ACCENT)
    a+=text(24,36,'このワークショップで行うこと',20,INK,700)
    a+=rect(24,62,320,172,PAPER,10,LINE)
    a+=icon(161,84,'book',INK,.85)
    a+=text(184,162,'考え方を学ぶ',21,INK,700,'middle')
    a+=text(184,197,['目的と要件の明確化','業務のモデル化と設計'],17,INK,anchor='middle',gap=26)
    a+=arrow(352,148,388,148)
    a+=rect(396,62,320,172,PAPER,10,LINE)
    a+=icon(540,80,'doc',INK,.7)
    a+=text(556,162,'Skillsにまとめる',21,INK,700,'middle')
    a+=text(556,197,['何を確かめ、どう進めるか','（判断の観点と手順）'],17,INK,anchor='middle',gap=26)
    a+=arrow(724,148,760,148)
    a+=rect(768,62,320,172,PAPER,10,LINE)
    a+=_robot_at(905,80,INK)
    a+=text(928,162,'自分の業務で試す',21,INK,700,'middle')
    a+=text(928,197,['AgentsがSkillsを使い','作業を進める'],17,INK,anchor='middle',gap=26)
    a+=path('M928 242 V266 H556 V243 M549 250 L556 243 L563 250',LINE,2)
    a+=person(616,289,.5,INK)
    a+=text(644,296,'人が結果を確かめ、Skillsと進め方を改善する',18,INK,700)
    return svg('ワークショップでは、目的と要件、業務のモデル化と設計の考え方を学び、判断の観点と手順をSkillsにまとめ、自分の業務でAgentsが使って作業を進める。人が結果を確かめてSkillsと進め方を改善する',a,338)

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
