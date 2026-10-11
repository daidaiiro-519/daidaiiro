"""最終動画。共通の定義・条件と個別の内容の関係を描く。配色は既存の部品から読む。"""
from lesson_visuals import text,rect,path,arrow,svg,person,circle,INK,DIM,LINE,PAPER,PANEL,ACCENT,W

def cover(no):
    if no in (0,1):
        from derived_visuals import t_cover
        return t_cover(no)
    a=''
    for i,name in enumerate(['原因を知る','意味を決める','範囲を決める','条件を決める','具体と抽象を振り返る']):
        x=i*226;on=i+2==no
        a+=rect(x,24,208,96,PAPER,10,ACCENT if on else LINE)
        a+=text(x+104,57,str(i+1),15,DIM,anchor='middle')
        a+=text(x+104,94,name,17,ACCENT if on else INK,700,'middle')
        if i<4:a+=arrow(x+208,72,x+226,72)
    return svg('本編5本の順番と現在の位置',a,130)

def opening():
    from bridge_visuals import intro
    return intro('ここまでに決めたこと',[('目的','PMが優先順位を判断する'),('内容','意味・範囲・条件を伝える')], 'この動画で振り返ること',(0,1,2,3),[], '毎週使う基準と、その週に変わる内容を分けて考える。', [('変化','最初と現在を比べる'),('関係','共通と個別を見る'),('考え方','目的に合う内容を考える'),('次の教材','チームで意味を確かめる')], '自分の目的を言葉にすることから、チームで意味を確かめることへ。','具体と抽象を振り返り次の教材につなぐ',focus=[('共通の基準','何を共通に決めたか'),('チームとの共有','同じ内容を理解できるか')])

def changed():
    a=rect(0,0,356,320,PAPER,12,LINE)+text(24,34,'はじめのプロンプト',17,DIM,700)
    a+=text(24,89,'「課題を整理して」',27,INK,700)
    a+=text(24,163,['作業の名前は伝えている','PMが何に使うかは伝えていない','必要な内容も伝えていない'],18,INK,gap=42)
    a+=arrow(356,160,412,160)
    a+=rect(412,0,700,320,PAPER,12,ACCENT)+text(436,34,'いまAIに伝える内容',17,ACCENT,700)
    for i,(tag,body) in enumerate([('目的','PMが優先順位を判断するため'),('意味','決定しないと進行が止まる事項'),('範囲','今回の案件、本番の切り替えまで'),('条件','担当・期限、未記載の扱い、期限順')]):
        y=77+i*55;a+=rect(436,y-22,76,34,PAPER,8,LINE)+text(474,y+2,tag,18,INK,700,'middle')+text(536,y+2,body,21,INK,700)
    a+=text(0,362,'決めたのは、毎回同じにしたいこと。各週の課題名や日付は、議事録に合わせる。',20,INK)
    return svg('作業名だけの入力と、目的・意味・範囲・条件を伝える入力を対比する',a,382)

def _level_axis(x, top, bottom):
    a=path(f'M{x} {bottom} V{top} m-7 10 l7 -10 7 10 M{x-7} {bottom-10} l7 10 7 -10',LINE,2)
    a+=text(x-20,top+22,'抽象',22,INK,700,'end')
    a+=text(x-20,bottom-5,'具体',22,INK,700,'end')
    return a


def meaning():
    a=path('M350 12 H910 L1112 372 H128 Z',LINE,2,PANEL)
    a+=_level_axis(92,12,372)
    a+=rect(350,36,560,118,PAPER,12,ACCENT)
    a+=text(630,67,'共通に使う課題の定義',18,ACCENT,700,'middle')
    a+=text(630,107,['決定しないと、案件の進行が','止まる事項'],23,INK,700,'middle',gap=32)
    a+=path('M630 154 V188 M336 234 V188 H912 V234 M624 188 V234',LINE,2)
    for x in [336,624,912]:a+=path(f'M{x-6} 224 l6 10 6 -10',LINE,2)
    for x,title,who in [(204,'承認の経路','経理・開発'),(492,'申請の締め日','経理'),(780,'既存データの移行対象','開発')]:
        a+=rect(x,234,264,110,PAPER,12,LINE)
        a+=text(x+20,265,who,16,DIM,700)+text(x+20,301,title,21,INK,700)+text(x+20,328,'未決定だと進行が止まる',16,INK)
    a+=text(128,412,'一つの定義が、業務の異なる多くの課題に当てはまる。',22,INK)
    return svg('上の共通定義から下の三つの個別課題へ広がるピラミッド。左の全長の縦軸は上が抽象、下が具体',a,432)


def values():
    a=path('M280 12 H960 L1112 388 H128 Z',LINE,2,PANEL)
    a+=_level_axis(92,12,388)
    a+=rect(300,36,640,112,PAPER,12,ACCENT)
    a+=text(324,66,'各課題に共通して使う条件',18,ACCENT,700)
    a+=text(324,99,['担当と期限を含める。期限の早い順に並べる。','記録がない項目は「記載なし」と示す。'],21,INK,700,gap=31)
    a+=path('M620 148 V177 M422 210 V177 H862 V210',LINE,2)
    for x in [422,862]:a+=path(f'M{x-6} 200 l6 10 6 -10',LINE,2)
    for x,head,rows in [(216,'今週の内容',[('承認の経路','経理・開発','9月30日'),('旧システムの停止日','PM','10月7日')]),(656,'翌週の内容',[('差し戻しの扱い','経理','10月14日'),('移行対象の追加','開発','記載なし')])]:
        a+=rect(x,210,412,156,PAPER,12,LINE)+text(x+20,239,head,18,INK,700)
        for dx,t in [(20,'課題'),(209,'担当'),(310,'期限')]:a+=text(x+dx,272,t,15,DIM,700)
        a+=path(f'M{x+20} 282 H{x+392}',LINE,1)
        for j,row in enumerate(rows):
            for dx,t in zip([20,209,310],row):a+=text(x+dx,312+j*35,t,16,INK,700 if dx==20 else 400)
    a+=text(128,424,'同じ条件を各週へ当てはめる。課題名・担当・期限は、議事録に合わせて変わる。',20,INK)
    return svg('上の共通条件から下の二週の具体的な表へ広がるピラミッド。条件は共通で、個別の値は変わる',a,444)


def thinking():
    a=_level_axis(92,16,408)
    # 読む順を1→2→3とし、具体→抽象→具体を同じ図で追えるようにする。
    a+=rect(266,16,636,152,PAPER,12,ACCENT)
    a+=circle(294,47,13,ACCENT,ACCENT)+text(294,52,'2',15,PAPER,700,'middle')
    a+=text(320,53,'各課題に共通する条件を考える',21,INK,700)
    a+=text(290,103,'担当と期限を含める',30,INK,700)
    a+=text(290,145,'PMが優先順位を判断するには、担当と期限が必要',19,DIM)
    a+=rect(144,258,410,150,PAPER,12,LINE)
    a+=circle(172,287,13,ACCENT,ACCENT)+text(172,292,'1',15,PAPER,700,'middle')
    a+=text(198,294,'今週の出力で不足に気づく',19,INK,700)
    a+=rect(696,258,416,150,PAPER,12,LINE)
    a+=circle(724,287,13,ACCENT,ACCENT)+text(724,292,'3',15,PAPER,700,'middle')
    a+=text(750,294,'翌週の表で条件を確かめる',19,INK,700)
    for x,rows in [(144,('承認の経路','空欄','空欄')),(696,('差し戻しの扱い','経理','10月14日'))]:
        for dx,t in [(24,'課題'),(218,'担当'),(308,'期限')]:a+=text(x+dx,331,t,16,DIM,700)
        a+=path(f'M{x+24} 342 H{x+386}',LINE,1)
        if x==144:
            for dx in [210,300]:
                box=rect(x+dx,355,82,32,PAPER,6,ACCENT)
                a+=box.replace('stroke-width="2"','stroke-width="2" stroke-dasharray="5 4"')
        for dx,t in zip([24,218,308],rows):a+=text(x+dx,379,t,18 if dx<300 else 16,ACCENT if x==144 and dx>24 else INK,700 if dx==24 else 400)
    a+=path('M344 258 V168 m-7 11 l7 -11 7 11',ACCENT,3)
    a+=path('M824 168 V258 m-7 -11 l7 11 7 -11',ACCENT,3)
    a+=text(367,218,'抽象化',23,ACCENT,700)
    a+=text(847,218,'具体化',23,ACCENT,700)
    return svg('1今週の具体的な出力の不足から上へ抽象化し2利用目的に必要な共通条件を考え、下へ具体化して3翌週の具体例で確かめる。担当期限の空欄と記載が見える三つの枠',a,422)


def purpose():
    a=_level_axis(92,12,344)
    rows=[(24,'毎週の課題整理','課題の定義・範囲・条件を共通にする','課題名・担当・期限は各週の議事録に合わせる'),
          (224,'今週の1件の確認','「承認の経路」の担当と期限を確認する','必要な個別の課題を指定する')]
    for y,head,body,tail in rows:
        a+=circle(92,y+58,9,ACCENT,ACCENT)
        a+=rect(164,y,948,116,PAPER,12,ACCENT)
        a+=text(188,y+28,head,18,DIM,700)+text(188,y+66,body,26,INK,700)+text(188,y+98,tail,20,DIM)
    a+=text(188,188,'どちらも、利用目的に合う決め方',22,ACCENT,700)
    a+=rect(0,374,1112,52,PAPER,12,LINE)
    a+=text(556,408,'抽象であるほどよいわけではない。必要な具体性は、使う目的で決まる。',23,INK,700,'middle')
    return svg('抽象具体の全長の縦軸の上に毎週の共通条件、下に一件の具体的指定を置き両方を同じ強調で示す。適した位置は目的で異なる',a,434)


def next_course():
    # 表の見かけの不足ではなく、業務の言葉が指す内容の未確認を示す。
    a=rect(0,0,1112,76,PAPER,12,LINE)
    for x,label,body in [(24,'課題','承認の経路'),(510,'担当','経理・開発'),(902,'期限','9月30日')]:
        a+=text(x,25,label,15,DIM,700)+text(x,59,body,26,INK,700)
    a+=path('M556 76 V94 M266 114 V94 H846 V114',LINE,2)
    for x in [266,846]:a+=path(f'M{x-6} 104 l6 10 6 -10',LINE,2)
    for x,who,lines in [(0,'経理が確認したいこと',['誰が経費を','承認するか']), (580,'開発が確認したいこと',['承認する人を、システムで','どう判定するか'])]:
        a+=rect(x,114,532,128,PAPER,12,LINE)
        a+=text(x+24,143,who,17,DIM,700)+text(x+24,181,lines,24,INK,700,gap=33)
    a+=path('M266 242 V251 H846 V242 M556 251 V270 m-6 -10 l6 10 6 -10',LINE,2)
    a+=rect(0,270,1112,62,PAPER,12,ACCENT)
    a+=text(24,309,'PM',22,ACCENT,700)+text(112,309,'何が未決定で、どの作業に影響する？',26,INK,700)
    a+=path('M556 332 V352 m-6 -10 l6 10 6 -10',LINE,2)
    a+=rect(0,352,1112,72,PAPER,12,LINE)
    a+=text(24,382,'教材2　言葉の定義',21,ACCENT,700)
    a+=text(304,382,'言葉が指す内容と、決める内容をチームで確かめる',22,INK,700)
    a+=text(304,410,'合意した内容と必要な記録を、AIにも伝える',18,DIM)
    return svg('課題名・担当・期限だけでは未決定の内容が分からない。経理と開発が確認したい内容からPMの判断に必要な問いを示し、言葉が指す内容をチームで確かめる教材2へつなぐ',a,432)
