"""i0_sceneの人物の発言と大きな仕事の図を土台にする。"""
from intro_visuals import *
from intro_visuals import _bubble,_ray

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
    a=text(0,22,'AIによる作成',17,DIM)
    for y,who,msg in [(102,'PM・PL',['計画書や説明資料など','いろいろな案がすぐできた']),(219,'開発者',['設計書・コード・テストも','短時間で作れるようになった'])]:
        a+=person(24,y+18,.7,INK)+text(24,y+72,who,14,DIM,700,'middle')
        a+=_bubble(66,y-18,292,70,y+17)+text(87,y+9,msg,17,INK,gap=26)
        a+=arrow(368,y+17,404,y+17)
    a+=text(410,22,'仕事で使えるかを、人が確かめる',17,DIM)
    a+=rect(410,34,702,260,PAPER,12,LINE)
    a+=person(442,62,.55,INK)+text(475,64,'元の情報や仕様と照合する',19,INK,700)
    for y,left,outputs,condition in [(82,'元の情報',['計画書','説明資料','報告資料'],'必要な情報があり、事実と合うか'),(177,'要件・仕様',['設計書','コード','テスト'],'要件・仕様を満たし、品質に問題はないか')]:
        a+=doc(436,y,left)
        for x,label in zip([724,850,976],outputs):
            a+=doc(x,y,label,output=True,w=112)
        a+=icon(642,y+15,'search',INK,.65)
        a+=path(f'M632 {y+37} H620 M695 {y+37} H710',LINE,2)
        a+=text(436,y+87,condition,17,INK)
    a+=text(0,323,['作れる成果物の種類と量が増え、','確認する対象も増えることがある'],15,DIM,gap=26)
    a+=path('M561 294 V312 M555 304 L561 312 L567 304',LINE,2)
    a+=rect(436,312,250,44,PAPER,10,LINE)+text(561,340,'不足があれば修正',17,INK,700,'middle')
    a+=arrow(697,334,730,334)
    a+=rect(742,312,238,44,PAPER,10,LINE)+text(861,340,'修正した内容を再確認',17,INK,700,'middle')
    a+=path('M980 334 H1092 V294 M1086 302 L1092 294 L1098 302',LINE,2)
    a+=band(378,'作成の速さは、確認・修正を含む時間にもつながっていますか？',48)
    return svg('PMと開発者の作成が速くなった実感を、人が元の情報や要件・仕様と、計画書・説明資料・報告資料・設計書・コード・テストの案を照合し修正と再確認をする仕事へつなげる',a,430)
