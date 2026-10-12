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
    a=text(0,22,'案件ごとの入力',17,DIM,700)
    a+=rect(0,68,238,170,PAPER,12,LINE)
    a+=icon(18,91,'doc',INK,.6)+text(67,116,'今回の情報',19,INK,700)
    a+=path('M18 136 H218',LINE,1)
    a+=text(18,165,['案件・対象・変更点','記録や要件・仕様など'],16,INK,gap=29)
    a+=arrow(248,151,310,151)
    a+=rect(320,68,312,170,PAPER,12,ACCENT)
    a+=_robot_at(342,114,INK)
    a+=text(413,111,'Agent',24,INK,700)
    a+=text(413,153,['Skillを使って','作業を進める'],18,INK,gap=28)
    a+=arrow(642,151,684,151)
    a+=text(694,22,'仕事に必要な成果物',17,DIM,700)
    a+=rect(694,56,246,202,PAPER,12,LINE)
    for i,label in enumerate(['計画書','説明資料','報告資料','設計書','コード','テスト']):
        a+=output_doc(704+(i%2)*118,80+(i//2)*56,label)
    a+=arrow(950,151,968,151)
    a+=rect(978,68,134,170,PAPER,12,LINE)
    a+=person(1045,126,.8,INK)+text(1045,181,'人が確認',17,INK,700,'middle')
    a+=text(1045,214,'条件に照らして使う',13,DIM,anchor='middle')
    # SkillとAgentを別の要素として境界を示し、使用と改善の線を分ける。
    a+=rect(320,262,312,94,PAPER,12,LINE)
    a+=icon(338,280,'doc',INK,.55)+text(387,290,'Skill',23,INK,700)
    a+=text(387,320,['目的・必要な内容','確認条件・手順'],16,INK,gap=25)
    a+=path('M476 262 V238 M470 246 L476 238 L482 246',LINE,2)
    a+=text(491,255,'記録した内容を使う',13,DIM)
    a+=path('M1045 238 V326 H632 M640 320 L632 326 L640 332',LINE,2)
    a+=text(1019,304,'合わない条件・手順を改善',16,INK,anchor='end')
    a+=band(370,'決めた条件と手順を、同じ種類の作業で再利用・改善する',54)
    return svg('案件ごとの情報をAgentへ入力し、AgentがSkillの目的・条件・手順を使い必要な成果物を作成する。人が確認して使い、合わない条件と手順を改善する',a,430)
