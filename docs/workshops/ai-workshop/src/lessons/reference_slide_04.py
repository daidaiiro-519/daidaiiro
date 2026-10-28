"""i0_sceneの人物の発言と大きな仕事の図を土台にする。"""
from intro_visuals import *
from intro_visuals import _bubble,_ray,_robot_at

def doc(x,y,label,output=False,w=184):
    h=65 if not output else 55
    a=''
    if output:
        for layer in range(3,0,-1):
            xx,yy=x+layer*4,y-layer*3
            a+=path(f'M{xx} {yy} H{xx+w-12} L{xx+w} {yy+12} V{yy+h} H{xx} Z M{xx+w-12} {yy} V{yy+12} H{xx+w}',LINE,2,PAPER)
    a+=path(f'M{x} {y} H{x+w-12} L{x+w} {y+12} V{y+h} H{x} Z M{x+w-12} {y} V{y+12} H{x+w}',LINE,2,PAPER)
    a+=text(x+12,y+25,label,14 if output else 16,INK,700)
    a+=path(f'M{x+12} {y+37} H{x+w-16} M{x+12} {y+47} H{x+w-34}',LINE,1)
    return a

def slide_04():
    a=text(0,22,'AIで作った成果物',17,DIM)
    a+=rect(0,34,358,260,PAPER,12,LINE)
    a+=_robot_at(26,92,INK)
    a+=doc(112,96,'資料',output=True,w=96)+doc(234,96,'コード',output=True,w=96)
    a+=text(179,204,['成果物が増えると','照合する対象も','増えることがある'],18,INK,700,'middle',gap=28)
    a+=arrow(368,164,402,164)
    a+=text(410,22,'仕事で使えるかを、人が確かめる',17,DIM)
    a+=rect(410,34,702,260,PAPER,12,LINE)
    a+=person(442,62,.55,INK)+text(475,64,'元の情報や仕様、成果物どうしを照合する',19,INK,700)
    for y,left,outputs,condition in [(82,'元の情報',['計画書','説明資料','報告資料'],'必要な情報が入っていて、事実と合うか'),(177,'要件・仕様',['設計書','コード','テスト'],'要件・仕様を満たし、品質に問題はないか')]:
        a+=doc(436,y,left)
        for x,label in zip([724,850,976],outputs):
            a+=doc(x,y,label,output=True,w=112)
        a+=icon(642,y+15,'search',INK,.65)
        a+=path(f'M632 {y+37} H620 M695 {y+37} H710',LINE,2)
        a+=text(436,y+87,condition,17,INK)
    a+=path('M561 294 V312 M555 304 L561 312 L567 304',LINE,2)
    a+=rect(436,312,250,44,PAPER,10,LINE)+text(561,340,'不足があれば修正',17,INK,700,'middle')
    a+=arrow(697,334,730,334)
    a+=rect(742,312,238,44,PAPER,10,LINE)+text(861,340,'修正した内容を再確認',17,INK,700,'middle')
    a+=path('M980 334 H1092 V294 M1086 302 L1092 294 L1098 302',LINE,2)
    a+=band(378,'作成の時間が短くなっても、確認と修正の時間が同じように減るとは限らない',48)
    return svg('AIで作った成果物が増えると照合する対象も増えることを示し、人が元の情報や要件・仕様、成果物どうしを照合し、修正と再確認をする流れへつなげる',a,430)
