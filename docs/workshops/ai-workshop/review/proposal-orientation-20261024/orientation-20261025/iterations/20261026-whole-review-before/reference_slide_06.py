"""既存の入力・Agent・Skill・成果物・人の確認の関係図を使う。"""
from intro_visuals import *
from intro_visuals import _robot_at,_ray

def output_doc(x,y,label):
    w,h=108,44
    a=''
    for k in range(2,-1,-1):
        xx,yy=x+k*3,y-k*3
        a+=path(f'M{xx} {yy} H{xx+w-10} L{xx+w} {yy+10} V{yy+h} H{xx} Z M{xx+w-10} {yy} V{yy+10} H{xx+w}',LINE,2,PAPER)
    a+=text(x+10,y+24,label,13,INK,700)
    a+=path(f'M{x+10} {y+33} H{x+w-16}',LINE,1)
    return a

def slide_06():
    a=text(0,22,'作業に必要な情報',17,DIM,700)
    a+=rect(0,68,238,170,PAPER,12,LINE)
    a+=icon(18,91,'doc',INK,.6)+text(67,116,'その作業の情報',19,INK,700)
    a+=path('M18 136 H218',LINE,1)
    a+=text(18,165,['資料の元になる情報','要件・仕様・変更点'],16,INK,gap=29)
    a+=arrow(248,151,310,151)
    a+=rect(320,68,312,170,PAPER,12,ACCENT)
    a+=_robot_at(342,114,INK)
    a+=text(413,111,'Agent',24,INK,700)
    a+=text(413,153,['Skillsを使って作業し、','結果の照合も行う'],18,INK,gap=28)
    a+=arrow(642,151,684,151)
    a+=text(694,22,'仕事に必要な成果物',17,DIM,700)
    a+=rect(694,56,246,202,PAPER,12,LINE)
    for i,label in enumerate(['計画書','説明資料','報告資料','設計書','コード','テスト']):
        a+=output_doc(704+(i%2)*118,80+(i//2)*56,label)
    a+=arrow(950,151,968,151)
    a+=rect(978,68,134,170,PAPER,12,LINE)
    a+=person(1045,126,.8,INK)+text(1045,181,'人が確認',17,INK,700,'middle')
    a+=text(1045,214,'根拠と結果を確認',13,DIM,anchor='middle')
    # SkillとAgentを別の要素として境界を示し、使用と改善の線を分ける。
    a+=rect(320,262,312,94,PAPER,12,LINE)
    a+=icon(338,280,'doc',INK,.55)+text(387,290,'Skills',23,INK,700)
    a+=text(387,320,['参照する情報・判断の観点','確認と照合の手順'],16,INK,gap=25)
    a+=path('M476 262 V238 M470 246 L476 238 L482 246',LINE,2)
    a+=text(491,255,'観点と手順を使う',13,DIM)
    a+=path('M1045 238 V326 H632 M640 320 L632 326 L640 332',LINE,2)
    a+=rect(716,276,322,40,PAPER,12,LINE)
    a+=text(877,302,'成果物を修正し、Skillsも改善',16,INK,anchor='middle')
    a+=band(370,'同じ種類の作業でSkillsを繰り返し使い、結果から改善する',54)
    return svg('作業に必要な情報をAgentへ入力し、AgentがSkillsの判断の観点と手順を使って成果物を作り、結果を照合する。人が根拠と結果を確認し、Skillsの観点と手順を改善する',a,430)
