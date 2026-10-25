"""既存の仕事別の枠と文書の部品で、自分の仕事への応用を示す。"""
from intro_visuals import *

def small_doc(x,y,label):
    w,h=142,74
    a=path(f'M{x} {y} H{x+w-14} L{x+w} {y+14} V{y+h} H{x} Z M{x+w-14} {y} V{y+14} H{x+w}',LINE,2,PAPER)
    a+=text(x+13,y+29,label,17,INK,700)
    a+=path(f'M{x+13} {y+44} H{x+w-18} M{x+13} {y+58} H{x+w-40}',LINE,1)
    return a

def slide_08():
    a=''
    for x,title in [(0,'資料作成で使う'),(582,'設計・開発で使う')]:
        a+=rect(x,0,530,306,PAPER,12,LINE)
        a+=person(x+34,40,.6,INK)+text(x+74,43,title,21,INK,700)
    # 資料は判断に使う情報と根拠を確認できる姿を示す。
    a+=path('M24 70 H488 L506 88 V237 H24 Z M488 70 V88 H506',LINE,2,PAPER)
    a+=text(42,103,'用途に必要な内容と根拠',19,INK,700)
    a+=path('M42 117 H485',LINE,1)
    for yy,label,value in [(145,'用途','誰が、何の判断に使うか'),(184,'根拠','元の情報との対応'),(220,'詳細','必要な説明と確認先')]:
        a+=text(42,yy,label,15,DIM,700)+text(125,yy,value,17,INK)
    a+=check(34,272,LINE,.55)+text(56,278,'必要な内容を伝え、根拠とともに確認できる',17,INK,700)
    # 設計・コード・テストを別々に増やすだけでなく、要件との対応を確かめる。
    a+=rect(680,70,336,42,PAPER,8,LINE)+text(848,98,'目的・対象範囲・要件と仕様',17,INK,700,'middle')
    a+=path('M848 112 V130 H666 M848 130 H1030 M666 130 V148 M848 130 V148 M1030 130 V148',LINE,2)
    for x,label in [(598,'設計書'),(775,'コード'),(952,'テスト')]:
        a+=small_doc(x,148,label)
    a+=text(848,250,'要件・仕様との対応を確認する',16,DIM,anchor='middle')
    a+=check(616,272,LINE,.55)+text(638,278,'対象と品質の条件を伝え、出力を確かめる',17,INK,700)
    a+=path('M265 306 V330 H848 V306 M556 330 V352 M550 344 L556 352 L562 344',LINE,2)
    a+=band(358,'条件と手順を記録し、繰り返す作業で再利用・改善する',54)
    return svg('資料作成では用途に必要な内容と根拠を確認し、設計・開発では設計書・コード・テストと要件仕様の対応を確認する。両方で条件と手順を記録し再利用と改善に活かす',a,420)
