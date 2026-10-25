"""i0_digの左右比較と、04の複数の成果物の表現を使う。"""
from intro_visuals import *
from intro_visuals import _robot_at,_ray

def paper(x,y,label,stack=False):
    w,h=98,65
    a=''
    for layer in range(3 if stack else 0,-1,-1):
        xx,yy=x+layer*4,y-layer*3
        a+=path(f'M{xx} {yy} H{xx+w-12} L{xx+w} {yy+12} V{yy+h} H{xx} Z M{xx+w-12} {yy} V{yy+12} H{xx+w}',LINE,2,PAPER)
    a+=text(x+12,y+27,label,14,INK,700)
    a+=path(f'M{x+12} {y+42} H{x+w-12} M{x+12} {y+53} H{x+w-28}',LINE,1)
    return a

def slide_03():
    a=''
    for x,title,is_ai in [(0,'以前の進め方の例',False),(582,'AIを使う進め方の例',True)]:
        a+=text(x,22,title,17,DIM,700)
        a+=rect(x,34,530,262,PAPER,12,LINE)
        if is_ai:
            a+=_robot_at(x+56,132,INK)
            a+=text(x+82,224,'人がAIを使う',17,INK,700,'middle')
        else:
            a+=person(x+82,163,1,INK)
            a+=text(x+82,224,'人が作成',17,INK,700,'middle')
        a+=_ray(x+120,164,x+188,110,LINE,2)
        a+=_ray(x+120,164,x+188,220,LINE,2)
        for y,labels in [(78,['計画書','説明資料','報告資料']),(190,['設計書','コード','テスト'])]:
            for px,label in zip([196,304,412],labels):
                a+=paper(x+px,y,label,stack=is_ai)
        a+=text(x+82,254,'短時間で作成' if is_ai else '自分で整理・記述',14,DIM,anchor='middle')
    a+=arrow(540,164,574,164)
    a+=band(318,'同じ時間で、作れる案や資料の数が増える',54)
    return svg('人が作成していた資料・設計書・コード・テストなどの一部をAIが担う例を対比し、AIを使う側の成果物を重ねて、作れる数の増加を示す',a,382)
