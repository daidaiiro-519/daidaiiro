"""既存の人物、教材、仕事の図で背景とまとめを示す。"""
from intro_visuals import *
from intro_visuals import _robot_at,i0_why

def opening():
    """背景、学習と実践、その先に目指す仕事の改善を接続する。"""
    a=rect(0,16,252,310,PAPER,12,LINE)
    a+=text(20,48,'AIで広がった可能性',18,INK,700)
    a+=_robot_at(28,80,INK)
    a+=icon(112,87,'doc',INK,.72)+icon(181,87,'screen',INK,.72)
    a+=text(126,201,['資料やツールを','作りやすく、試しやすく'],20,INK,700,'middle',gap=32)
    a+=text(126,292,'難しかったことも実現できる',16,DIM,anchor='middle')
    a+=arrow(260,171,290,171)

    a+=rect(298,16,526,310,PAPER,12,ACCENT)
    a+=text(322,48,'このワークショップ',20,INK,700)
    a+=rect(318,76,214,178,PAPER,10,LINE)
    a+=icon(397,98,'book',INK,.85)
    a+=text(425,176,'考え方を学ぶ',21,INK,700,'middle')
    a+=text(425,211,['目的と要件','業務の理解と設計'],17,INK,anchor='middle',gap=27)
    a+=arrow(540,166,574,166)
    a+=rect(582,76,222,178,PAPER,10,LINE)
    a+=_robot_at(670,96,INK)
    a+=text(693,176,'自分の業務で試す',21,INK,700,'middle')
    a+=text(693,211,['Agents・Skillsを使い','結果を確かめる'],17,INK,anchor='middle',gap=27)
    a+=text(561,294,'学んだことを使い、結果から改善する',19,INK,700,'middle')
    a+=arrow(832,171,862,171)

    a+=rect(870,16,242,310,PAPER,12,LINE)
    a+=text(890,48,'目指す仕事の改善',18,INK,700)
    a+=icon(976.6,92,'check',INK,.72)
    a+=text(991,154,'必要な品質を満たす',21,INK,700,'middle')
    a+=icon(974.75,194,'calendar',INK,.65)
    a+=text(991,260,['確認・修正も含め','効率よく進める'],20,INK,700,'middle',gap=30)
    return svg('AIでできることが増えた背景から、ワークショップで考え方を学び自分の業務で試すことを通じ、品質と仕事全体の効率の改善を目指す',a,346)

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
