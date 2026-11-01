"""変更前i0_digの左右対比で、確認する人と成果物の関係を示す。"""
from intro_visuals import *
from intro_visuals import _ray

def sheets(x,y,label,h):
    w=96
    a=''
    for k in range(2,-1,-1):
        xx,yy=x+k*4,y-k*3
        a+=path(f'M{xx} {yy} H{xx+w-12} L{xx+w} {yy+12} V{yy+h} H{xx} Z M{xx+w-12} {yy} V{yy+12} H{xx+w}',LINE,2,PAPER)
    a+=text(x+9,y+26,label,13,INK,700)
    for yy in range(40,h-8,12):
        a+=path(f'M{x+9} {y+yy} H{x+w-(18 if yy%24 else 32)}',LINE,1)
    return a

def slide_05():
    a=text(0,22,'大量の成果物から、必要な内容を探す',17,DIM)
    a+=text(582,22,'確認・判断の目的に合わせて作る',17,DIM)
    for x in [0,582]:a+=rect(x,34,530,314,PAPER,12,LINE)
    a+=rect(20,58,158,70,PAPER,32,ACCENT)
    a+=text(99,86,['確認したい内容は','どこにある？'],17,INK,700,'middle',gap=26)
    a+=circle(79,143,6,PAPER,LINE)+circle(77,161,4,PAPER,LINE)
    a+=person(76,208,.9,INK)+text(76,271,'確認する人',15,INK,700,'middle')
    a+=text(105,185,'内容を探す',13,DIM)
    a+=_ray(164,207,190,120,LINE,2)+_ray(164,207,190,252,LINE,2)
    for x,y,label,h in [(198,72,'計画書',90),(306,86,'説明資料',104),(414,72,'報告資料',72),(198,208,'設計書',100),(306,212,'コード',112),(414,208,'テスト',84)]:
        a+=sheets(x,y,label,h)
    a+=text(20,330,'形式や詳しさが違い、読む量も多い',16,DIM)
    a+=text(606,62,'何を・どこまで・どう示すかを決める',17,INK,700)
    # 右は単なるテンプレートではなく、承認に使う資料の例。
    x,y,w,h=606,78,482,218
    a+=path(f'M{x} {y} H{x+w-22} L{x+w} {y+22} V{y+h} H{x} Z M{x+w-22} {y} V{y+22} H{x+w}',LINE,2,PAPER)
    a+=text(624,105,'例：承認に使う資料',14,DIM)
    a+=text(624,136,'判断に必要な内容と、その根拠',20,INK,700)
    a+=path('M624 149 H1066',LINE,1)
    for yy,label,value in [(178,'結論','何を決めるか・何が変わるか'),(218,'根拠','元の情報や条件との対応'),(258,'確認事項','満たす条件・未確認の点')]:
        a+=text(624,yy,label,15,DIM,700)+text(722,yy,value,17,INK)
    a+=path('M1066 270 V313 H968',LINE,2)
    a+=rect(814,305,154,31,PAPER,8,LINE)+text(891,326,'必要な詳細・確認先',14,INK,700,'middle')
    a+=text(606,329,'詳しい内容も確認できる',15,DIM)
    a+=arrow(540,191,574,191)
    a+=band(370,'量や形式だけでなく、確認・判断の目的に合わせて作る',54)
    return svg('大量で形式や詳しさの違う成果物から情報を探す状態と、内容・範囲・詳しさ・示し方を決めて判断と詳細の確認ができる資料の例を比較する',a,430)
