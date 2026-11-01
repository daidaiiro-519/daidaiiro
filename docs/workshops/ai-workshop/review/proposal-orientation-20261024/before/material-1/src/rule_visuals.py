"""4本目。必要な内容、情報がないときの扱い、並び順を目的から決める。"""
from lesson_visuals import text, rect, path, arrow, circle, svg, INK, DIM, LINE, PANEL, PAPER, ACCENT, check, cross
from v1_visuals import _sheet, _down


def s4_missing():
    a = rect(0, 0, 1112, 318, PAPER, 12, LINE)
    a += text(24, 33, '説明用の出力例（抜粋）', 20, INK, 700)
    for x, t in [(24, '課題'), (520, '担当'), (820, '期限')]:a += text(x, 77, t, 18, DIM, 700)
    a += path('M24 92 H1088', LINE, 1)
    for y, item, who, when in [(130, '旧システムの停止日', 'PM', '10月7日'), (202, '承認の経路', '—', '9月30日'), (274, '申請の締め日', '経理', '—')]:
        a += text(24, y, item, 25, INK, 700)
        a += text(520, y, who, 24, INK)
        a += text(820, y, when, 24, INK)
    for x,y in [(504,166),(804,238)]:
        a += rect(x, y, 210, 46, PAPER, 8, ACCENT).replace('stroke-width="2"','stroke-width="1.5" stroke-dasharray="5 5"')
        a += text(x+105, y+32, '空欄', 22, ACCENT, 700, 'middle')
    a += _down(1080, 116, 198, ACCENT)
    a += text(1060, 166, '逆順', 18, ACCENT, 700, 'end')
    a += text(24, 308, '担当と期限が空欄の箇所があり、期限の順にも並んでいない。', 17, DIM)
    return svg('一つの出力例で担当と期限の空欄、期限の並び順の違いを示す。3回の実行比較ではない', a, 326)


def s4_unwritten():
    a = rect(0, 0, 540, 310, PAPER, 12, LINE)
    a += text(24, 34, 'PMが判断に必要な内容', 21, INK, 700)
    for i,(h,reason) in enumerate([('担当', '誰に確認するか'), ('期限', 'いつまでに決めるか'), ('情報の有無', '何が未確認なのか'), ('期限の順', 'どれから確認するか')]):
        y = 88+i*58
        a += text(24, y, h, 22, INK, 700)
        a += text(198, y, reason, 21, DIM)
        if i<3:a += path(f'M24 {y+20} H516', LINE, 1)
    a += arrow(540, 155, 572, 155)
    a += _sheet(572, 0, 540, 310)
    a += text(596, 34, 'AIに伝えた内容', 21, INK, 700)
    for y,tag,body in [(86,'意味','課題の定義'), (142,'範囲','対象の案件と時期')]:
        a += rect(596,y-25,102,36,PAPER,8,LINE)
        a += text(647,y,tag,18,INK,700,'middle')
        a += text(722,y,body,22,INK)
    a += rect(596, 188, 492, 86, PAPER, 10, ACCENT).replace('stroke-width="2"','stroke-width="1.5" stroke-dasharray="5 5"')
    a += text(620, 221, '条件は、まだ伝えていない', 24, ACCENT, 700)
    a += text(620, 253, '必要な情報・ないときの扱い・並び順', 17, DIM)
    a += text(24, 361, '伝えていない条件は、AIがその場で決める。', 24, INK, 700)
    return svg('PMに必要な担当、期限、情報の有無、確認順と、まだ伝えていない条件を左右で対比する', a, 382)


def s4_layers():
    a = rect(0, 0, 352, 136, PAPER, 12, LINE)
    a += text(24, 32, '意味・範囲', 20, INK, 700)
    a += text(24, 72, 'どの課題を対象にするか', 22, INK, 700)
    a += text(24, 111, '定義と対象の案件・時期', 18, DIM)
    a += arrow(352, 68, 396, 68)
    a += rect(0, 160, 352, 188, PAPER, 12, ACCENT)
    a += text(24, 192, '条件', 22, ACCENT, 700)
    for i,(tag,body) in enumerate([('含める','担当と期限'),('ないとき','記載なしと示す'),('並べる','期限の早い順')]):
        y=224+i*44
        a += rect(24,y-20,84,30,ACCENT,8,ACCENT)
        a += text(66,y,tag,15,PAPER,700,'middle')
        a += text(126,y,body,21,INK,700)
    a += arrow(352, 254, 396, 254, ACCENT)
    a += rect(396, 0, 716, 348, PAPER, 12, LINE)
    a += text(420, 32, '条件に合わせた出力例（抜粋）', 21, INK, 700)
    for x,t in [(420,'課題'),(684,'担当'),(876,'期限')]:a += text(x,76,t,18,ACCENT if x>420 else DIM,700)
    a += path('M420 92 H1088',LINE,1)
    a += rect(864,236,130,38,PAPER,8,ACCENT).replace('stroke-width="2"','stroke-width="1.5" stroke-dasharray="4 4"')
    for i,(item,who,when) in enumerate([('承認の経路','経理と開発','9月30日'),('旧システムの停止日','PM','10月7日'),('申請の締め日','経理','記載なし')]):
        y=134+i*65
        a += text(420,y,item,21,INK,700)
        a += text(684,y,who,21,ACCENT,700)
        a += text(876,y,when,21,ACCENT,700)
    a += _down(1076,117,204,ACCENT)
    a += text(1060,169,'期限順',15,ACCENT,700,'end')
    a += text(420,314,'記録がない期限は、後に置く',18,DIM)
    return svg('意味と範囲は対象の課題を決め、条件は担当と期限、情報がないときの表示、並び順を決める。条件から表の対応箇所へつなぐ',a,356)


def s4_effect():
    a = _sheet(0,0,620,344)
    a += text(24,34,'AIに伝える共通の条件',21,INK,700)
    for i,(tag,lines) in enumerate([('含める',['各課題に、担当と期限を含める']),('ないとき',['議事録にない担当・期限は','「記載なし」と表示する']),('並べる',['期限の早い順に並べる','記録がない項目は後に置く'])]):
        y=[90,178,280][i]
        a += rect(24,y-24,100,36,ACCENT,8,ACCENT)
        a += text(74,y,tag,18,PAPER,700,'middle')
        a += text(148,y,lines,22,INK,700,gap=30)
    a += rect(652,0,460,344,PAPER,12,LINE)
    a += text(676,34,'情報がある場合／ない場合',21,INK,700)
    a += text(676,82,'記録あり：承認の経路',17,DIM,700)
    a += text(676,123,'経理と開発／9月30日',25,INK,700)
    a += text(676,157,'議事録の情報を反映する',18,DIM)
    a += path('M676 182 H1088',LINE,1)
    a += text(676,217,'記録なし：申請の締め日の期限',17,DIM,700)
    a += rect(676,239,412,56,PAPER,10,ACCENT)
    a += text(700,275,'期限：記載なし',25,ACCENT,700)
    a += text(676,326,'範囲内の課題でも、情報があるとは限らない',17,DIM)
    return svg('共通の3つの条件と、記録された担当と期限を使う例、記録されていない期限を記載なしと示す例を対比する',a,352)


def s4_after():
    a = rect(0,0,1112,312,PAPER,12,LINE)
    a += text(24,32,'説明用の前後の出力例を比べる',20,INK,700)
    a += text(648,72,'条件を伝える前の例',19,INK,700,'middle')
    a += text(944,72,'条件を伝えた例',19,ACCENT,700,'middle')
    a += path('M24 90 H1088 M800 48 V294',LINE,1)
    rows=[('担当欄に記載がある',False),('期限の有無が分かる',False),('議事録にない情報を作っていない',True),('期限の早い順に並ぶ',False)]
    for i,(t,ok) in enumerate(rows):
        y=125+i*53
        a += text(24,y+7,t,22,INK,700)
        a += check(648,y,INK,.8) if ok else cross(648,y,DIM,.7)
        a += check(944,y,ACCENT,.9)
        if i<3:a += path(f'M24 {y+25} H1088',LINE,1)
    a += rect(0,336,1112,54,PAPER,12,LINE)
    a += text(24,370,'実際の出力でも、担当・期限を議事録と照合し、並び順を確認する。',20,INK,700)
    return svg('未実行の回数比較をせず、一つずつの説明用の前後の例を比べる。担当、期限の有無、情報を作らないこと、期限の順を確認する',a,398)


def s4_end():
    a = rect(0,0,1112,78,PAPER,12,ACCENT)
    a += text(24,33,'条件は、各週の事項に使う共通の決まり',26,INK,700)
    a += text(24,65,'担当者名や日付は固定せず、議事録の情報を使う',18,DIM)
    a += _sheet(0,102,620,222)
    a += text(24,132,'ここまでに決めた内容',17,DIM,700)
    for i,(tag,body) in enumerate([('目的','PMが優先順位を判断する'),('意味','進行を止める未決定の事項'),('範囲','今回の案件、本番の切り替えまで'),('条件','担当・期限／未記載の扱い／期限順')]):
        y=167+i*43
        a += rect(24,y-23,100,32,ACCENT if i==3 else PAPER,8,ACCENT if i==3 else LINE)
        a += text(74,y,tag,17,PAPER if i==3 else INK,700,'middle')
        a += text(148,y,body,21,INK,700 if i==3 else 400)
    a += rect(652,102,460,222,PAPER,12,LINE)
    a += text(676,132,'出力で確かめる',17,DIM,700)
    a += text(676,168,['議事録の情報が反映されているか','ない情報を明示しているか','期限の順に並んでいるか'],20,INK,700,gap=32)
    a += path('M676 245 H1088',LINE,1)
    a += text(676,280,'共通の決まりと個別の内容',24,ACCENT,700)
    a += text(676,312,'具体と抽象の関係を振り返る',18,DIM)
    a += rect(0,346,1112,48,PAPER,24,LINE)
    a += text(24,377,'次の動画　具体と抽象を振り返る',20,INK,700)
    a += text(476,377,'ここまでの考え方から、次の教材へ進む',18,DIM)
    return svg('目的、意味、範囲、条件を平らな4行でまとめ、出力で確認する内容と、最終動画で具体と抽象を振り返る予告を示す',a,402)
