"""Semantic SVG diagrams for the workshop slides; no external assets."""
from html import escape

INK = '#252b29'
DIM = '#59605a'
LINE = '#a8b09f'
PANEL = '#e9e5da'
PAPER = '#fffaf2'
ACCENT = '#b64326'

def text(x, y, lines, size=22, color=INK, weight=400, anchor='start', gap=None):
    if isinstance(lines, str):
        lines = [lines]
    gap = gap or size*1.55
    return ''.join(f'<text x="{x}" y="{y+i*gap}" font-size="{size}" fill="{color}" font-weight="{weight}" text-anchor="{anchor}">{escape(s)}</text>' for i,s in enumerate(lines))

def rect(x,y,w,h,fill=PANEL,r=12,stroke='none'):
    return f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{r}" fill="{fill}" stroke="{stroke}" stroke-width="2"/>'

def path(d,color=LINE,width=2,fill='none'):
    return f'<path d="{d}" stroke="{color}" stroke-width="{width}" fill="{fill}" stroke-linecap="round" stroke-linejoin="round"/>'

def arrow(x1,y1,x2,y2,color=LINE):
    tip=f'M{x2-8} {y2-7} l8 7 -8 7' if x2>x1 else f'M{x2+8} {y2-7} l-8 7 8 7'
    return path(f'M{x1} {y1} H{x2} '+tip,color)

def circle(x,y,r,fill=PAPER,stroke=LINE):
    return f'<circle cx="{x}" cy="{y}" r="{r}" fill="{fill}" stroke="{stroke}" stroke-width="2"/>'

def person(x,y,scale=1,color=INK):
    return f'<g transform="translate({x} {y}) scale({scale})">'+circle(0,-14,12,'none',color).replace('stroke-width="2"','stroke-width="2.5"')+path('M-24 28 v-6 c0 -27 48 -27 48 0 v6',color,2.5)+'</g>'

def icon(x,y,kind,color=INK,scale=1):
    icons={
      'screen':'M0 0 H56 V36 H0 Z M28 36 V47 M13 48 H43',
      'doc':'M3 0 H33 L46 13 V55 H3 Z M33 0 V13 H46 M12 25 H35 M12 35 H35 M12 45 H28',
      'chat':'M0 0 H54 V33 H23 L10 44 V33 H0 Z M12 11 H42 M12 22 H31',
      'check':'M0 20 L13 33 L40 0',
      'book':'M0 3 Q14 -3 27 3 Q40 -3 54 3 V44 Q40 38 27 44 Q14 38 0 44 Z M27 3 V44',
      'search':'M35 17 a17 17 0 1 1 -34 0 a17 17 0 1 1 34 0 M30 30 L49 49',
      'calendar':'M0 7 H50 V48 H0 Z M0 19 H50 M13 0 V12 M37 0 V12 M10 29 H19 M28 29 H39 M10 39 H19',
      'branch':'M0 8 H20 V39 H49 M20 8 H49 M20 24 H49',
    }
    return f'<g transform="translate({x} {y}) scale({scale})">'+path(icons[kind],color,2.3)+'</g>'

def svg(label, body, height=330):
    return f'<svg class="ws-visual" viewBox="0 0 1112 {height}" role="img" aria-label="{escape(label)}">{body}</svg>'

def meeting():
    a=rect(6,45,316,194,PAPER,12,LINE)+rect(22,61,284,146,PANEL,5)
    a+=icon(129,86,'screen',scale=1.1)+text(164,180,'画面を見せる',26,weight=700,anchor='middle')
    a+=path('M164 239 V263 M116 264 H212',INK)+text(164,308,'TeamsのWeb会議',18,DIM,anchor='middle')
    a+=arrow(342,140,393,140)
    a+=rect(414,44,275,90)+text(437,80,['ここまでできた','ここで困っている'],22)
    a+=path('M437 134 l0 12 17 -12',LINE)+rect(452,164,250,74,PAPER,12,LINE)
    a+=text(477,196,['気になる点や','使ってみたい場面を話す'],19)
    a+=arrow(724,140,779,140)
    a+=circle(954,134,100,ACCENT,ACCENT)+icon(932,83,'check',PAPER,1.1)
    a+=text(954,160,['次に試すことを','決める'],25,PAPER,700,'middle',36)
    a+=text(954,278,'完成前でも共有する',19,DIM,anchor='middle')
    return svg('Teamsで画面を見せ、困りごとを話し、次に試すことを決める',a)

def support():
    # 上下の線は対称にする。上はポータルの中央（65）で頭の上（105）の40上、下はTeamsの中央（285）で名前の下（245）の40下
    a=person(95,147,1.6)+text(95,236,'参加者',25,weight=700,anchor='middle')
    a+=person(1013,147,1.6)+text(1013,236,'運営',25,weight=700,anchor='middle')
    a+=rect(336,12,442,107,PAPER,12,LINE)+icon(364,42,'search',scale=.8)
    a+=text(426,53,'ナレッジ共有ポータル',25,weight=700)+text(426,86,'教材動画と成果物で学ぶ',19,DIM)
    a+=path('M1013 100 V65 H778',INK,3)+path('M336 65 H95 V100 M88 92 L95 100 L102 92',INK,3)
    a+=rect(336,229,442,112,ACCENT)+icon(365,263,'chat',PAPER,.8)
    a+=text(428,270,'Teamsで相談する',25,PAPER,700)+text(428,306,'いつでも投稿 ＋ スポット相談会',17,PAPER)
    a+=path('M95 250 V285 H336',ACCENT,3)+path('M778 285 H1013 V250 M1006 258 L1013 250 L1020 258',ACCENT,3)
    return svg('参加者はナレッジ共有ポータルで学び、Teamsで運営に相談する',a,350)

def handoff():
    """教材を視聴して決めた目的を、AIの仕組みで達成する。他の人も使えれば、補助指標で確かめる。"""
    a = rect(0, 8, 300, 200, PAPER, 12, LINE)
    a += text(28, 50, '目的', 25, INK, 700)
    a += text(28, 96, ['教材を視聴して、', '目的と達成条件を', '書く'], 18, DIM, gap=28)
    a += arrow(308, 108, 352, 108)
    a += rect(360, 8, 400, 200, PAPER, 12, LINE)
    a += text(388, 50, 'AIの仕組み', 25, INK, 700)
    a += text(388, 100, 'Skill', 18, INK, 700) + text(452, 100, '目的 ・ 意味 ・ 範囲 ・ 条件を書く', 18, DIM)
    a += text(388, 150, 'Agent', 18, INK, 700) + text(452, 150, 'Skillを使って進める', 18, DIM)
    a += arrow(768, 108, 804, 108)
    a += rect(812, 8, 300, 200, ACCENT, 12)
    a += text(840, 50, '目的を達成する', 25, PAPER, 700)
    a += text(840, 96, ['終了時の結果を、', '達成条件と照合する', '（主KPI）'], 18, PAPER, gap=28)
    a += rect(0, 236, 1112, 56, PANEL, 12)
    a += text(28, 271, '他の人も使えれば、価値はさらに広がる', 19, INK, 700)
    a += text(1084, 271, '他の人に役立ったか（補助指標）', 17, DIM, anchor='end')
    return svg('教材を視聴して決めた目的と達成条件を、SkillとAgentの仕組みで達成し、終了時の結果と照合する。他の人も使えれば補助指標で確かめる', a, 300)

def evaluation():
    """効果測定の全体像。ワークショップの効果（共通の主KPIと、取り組みごとの補助）と、基盤づくりの検証（先行トライアル）を、同じアンケートとポータルの記録で測る。"""
    a = text(0, 18, 'ワークショップの効果', 15, DIM, 700) + text(780, 18, '基盤づくりの検証（先行トライアル）', 15, DIM, 700)
    # 共通の主KPI
    a += rect(0, 32, 748, 104, PAPER, 12, ACCENT)
    a += text(22, 58, '共通 ・ 参加者全員 ・ 主KPI', 14, ACCENT, 700)
    a += text(22, 94, '目的の明確化率', 20, INK, 700) + text(22, 118, 'AIで実現したい目的を、視聴前より明確に書けた人', 13, DIM)
    a += path('M374 68 V124', LINE, 1)
    a += text(398, 94, '目的達成率', 20, INK, 700) + text(398, 118, '参加申込で書いた目的を、AIで達成した人', 13, DIM)
    # 取り組みで分ける
    a += path('M374 136 V160 M182 160 H566 M182 160 V172 M566 160 V172', LINE, 2)
    a += path('M175 165 L182 172 L189 165 M559 165 L566 172 L573 165', LINE, 2)
    a += text(386, 153, '参加申込で選んだコースで分ける', 13, DIM)
    lanes = [(0, '補助 ・ 個人コース', '個人ではまだ目的を達成できていない人',
              [('実務でAIを使う人の増加', '担当業務で、実務にAIを使った人の増え方'), ('手戻りの減少', '手直しせずに使える出力が増えた人の割合')]),
             (384, '補助 ・ チームコース', '個人では既にできている人',
              [('チームでの改善の実施', '振り返りの場を1回以上開いた人の割合'), ('やり方のチームへの定着', '型の共有が進んだ人、立ち上げが短くなった人の割合')])]
    for x, head, who, items in lanes:
        a += rect(x, 176, 364, 176, PAPER, 12, LINE)
        a += text(x + 22, 202, head, 15, INK, 700) + text(x + 22, 222, who, 13, DIM)
        for k, (name, judge) in enumerate(items):
            y = 260 + k * 50
            a += text(x + 22, y, name, 16, INK, 700) + text(x + 22, y + 20, judge, 13, DIM)
    # 基盤づくりの検証
    box = rect(780, 32, 332, 320, PAPER, 12, LINE)
    a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 6"')
    checks = [('チームでの改善の継続', '終了後も振り返りを続ける人の割合'), ('参加者以外での再利用', '見つけた仕組みを業務で試した希望者'),
              ('必要なナレッジの見つけやすさ', '探していた仕組みにたどり着けた割合'), ('ナレッジを出す人の広がり', 'ノウハウを1件以上載せた希望者')]
    for k, (name, judge) in enumerate(checks):
        y = 66 + k * 62
        a += text(802, y, name, 16, INK, 700) + text(802, y + 20, judge, 13, DIM)
    a += path('M802 304 H1090', LINE, 1)
    a += text(802, 332, '次の期に何を広げるかの判断に使う', 13, DIM, 700)
    return svg('効果測定の全体像。ワークショップの効果は、参加者全員に共通の主KPI2つと、参加申込で選んだ取り組みごとの補助（個人コース：実務でAIを使う人の増加と手戻りの減少、チームコース：振り返りと品質の統一）で測る。基盤づくりは先行トライアルとして、チームでの継続 ・ 広がり ・ たどり着きやすさ ・ ナレッジを出した割合で検証する', a, 358)

CORE_QUESTIONS=[
 'AIを使えそうな仕事を思いつけますか。',
 '仕事で困っている理由を整理できますか。',
 'AIに、してほしいことを伝えられますか。',
 'AIに、いくつかの作業をまとめて頼めますか。',
 '作ったものの使い方を、他の人にも分かるように説明できますか。',
 'AIの答えが間違っていないか確認できますか。',
 '他の人からの感想をもとに、作ったものを直せますか。',
]
CORE_SCALE=['まだやり方が分からない','やり方は分かるが、まだできない','手助けがあればできる','一人でできる','状況に合わせてやり方を変えられる']
PM_TASKS=['複数資料から情報を集めて集計する','週次・月次報告や予実を可視化する','会議・チャットから課題・担当・期限を整理する','品質・障害・リスクの傾向を分析する','改善案・効果の見込み・提案資料を作る','過去の会議・成果物・トラブル事例を探して再利用する']
DEV_TASKS=['要求を整理して仕様をまとめる','設計案を作り、選択肢を比較する','コードを作成・修正する','コードや設計をレビューする','テスト項目・テストコードを作り、結果を確認する','ログや不具合を調べ、運用・保守に役立てる']
B_SCALE=['使っていない','試したが、実務には使っていない','実務で1回使った','実務で複数回使った']

def scale(x,y,values,width=448):
    a=text(x,y,'今の自分に近いものを選ぶ',21,weight=700)
    for i,t in enumerate(values):
        yy=y+20+i*44
        a+=rect(x,yy,width,39,PANEL,6)+circle(x+22,yy+19,11,PAPER,LINE)
        a+=text(x+22,yy+25,str(i),15,DIM,anchor='middle')+text(x+46,yy+26,t,18)
    return a

def questionnaire():
    a=text(0,25,'質問の例',19,DIM)
    qs=[['AIを使えそうな仕事を','思いつけますか。'],['AIに、してほしいことを','伝えられますか。'],['AIの答えが間違っていないか','確認できますか。']]
    for i,q in enumerate(qs):
        y=42+i*92
        a+=rect(0,y,574,80,PAPER,10,LINE)+text(24,y+32,q,23,gap=32)
    a+=scale(631,25,CORE_SCALE,481)
    a+=text(631,309,'「試す機会がない」も選べる',18,DIM)
    return svg('身近な言葉の3つの質問に、今の自分に近い5段階の答えを選ぶ',a)

def pm_survey():
    a=text(0,25,'普段の仕事',19,DIM)
    labels=[['複数資料から','情報収集・集計'],['週次・月次報告','予実の可視化'],['課題・担当・期限の','整理'],['品質・障害・リスクの','傾向分析'],['改善案・効果の見込み','提案資料の作成'],['過去の会議・成果物','トラブル事例の再利用']]
    icons=['search','doc','branch','calendar','chat','book']
    for i,(lines,kind) in enumerate(zip(labels,icons)):
        x=(i%2)*292;y=42+(i//2)*90
        a+=rect(x,y,278,78,PAPER,9,LINE)+icon(x+15,y+19,kind,scale=.55)
        a+=text(x+56,y+31,lines,18,gap=28)
    a+=scale(631,25,B_SCALE,481)
    a+=text(631,258,['過去4週間について回答する','担当外・実施機会なしは別に選ぶ'],18,DIM)
    return svg('PMとPLの6つの業務について、過去4週間のAI活用を同じ選択肢で答える',a)

def abilities():
    """目的をAIで達成するために身につける3つと、それぞれでできるようになること。土台は、目的に合った細かさで決めること。"""
    a = ''
    cards = [('教材1　目的の把握', '目的を捉える', ['AIに頼む前に、何のためか、', '何が変われば達成かを指示に書ける']),
             ('教材2　言葉の定義', '言葉をそろえる', ['AIに渡す言葉の意味をチームで', '定め、誰が頼んでも同じに解釈させる']),
             ('教材3　仕組みの構築', '仕組みにする', ['決めたことをSkillにし、AIが', '毎回同じ手順で目的を達成できる'])]
    for i, (lesson, name, can) in enumerate(cards):
        x = i * 384
        a += rect(x, 8, 344, 200, PAPER, 12, LINE)
        a += text(x + 24, 40, lesson, 14, DIM)
        a += text(x + 24, 80, name, 26, INK, 700)
        a += path(f'M{x+24} 98 H{x+320}', LINE, 1)
        a += text(x + 24, 126, 'AIへの指示で、できるようになること', 13, DIM, 700)
        a += text(x + 24, 156, can, 17, INK, gap=26)
        if i < 2:
            a += arrow(x + 350, 108, x + 378, 108)
    a += rect(0, 228, 1112, 52, PAPER, 12, ACCENT)
    a += text(556, 260, '土台は、目的に合った細かさでAIに指示すること。粗すぎると毎回変わり、細かすぎるとその1回にしか使えない', 17, ACCENT, 700, 'middle')
    return svg('目的をAIで達成するために、目的を捉える ・ 言葉をそろえる ・ 仕組みにするの3つを身につけ、それぞれでできるようになることを示す。土台は、目的に合った細かさで決めること', a, 286)

def agents():
    """エージェントは業務の目的を達成し、Skillsは作業ごとの道具である。"""
    a=person(95,98,1.5)+text(95,185,'人',23,weight=700,anchor='middle')+text(95,222,['業務の目的を','伝える'],18,DIM,anchor='middle')
    a+=text(95,290,['例：案件を','遅延なく進める'],15,DIM,anchor='middle')
    a+=arrow(169,113,311,113)
    a+=rect(335,9,448,316,PAPER,14,LINE)+text(559,50,'エージェント',27,weight=700,anchor='middle')
    a+=text(559,88,'Skillsを使い、業務の目的を達成する',19,DIM,anchor='middle')
    for i,name in enumerate(['課題を整理する','進捗報告を作る','遅れを検知する']):
        x=358+i*138
        # 強調しない箱は地を塗らず線だけにする（灰色の地は文字が沈む）
        a+=rect(x,118,126,146,ACCENT,10) if i==0 else rect(x,118,126,146,PAPER,10,LINE)
        c=PAPER if i==0 else INK
        a+=text(x+63,152,'Skill',18,c,700,'middle')
        a+=text(x+63,196,'作業：',15,c,anchor='middle')
        a+=text(x+63,224,name,15,c,anchor='middle')
    a+=text(559,300,'各Skillの中に、目的 ・ 意味 ・ 範囲 ・ 条件を書く',16,INK,anchor='middle')
    a+=arrow(808,113,925,113)+icon(972,69,'doc',scale=1.25)
    a+=text(1004,187,'結果',23,weight=700,anchor='middle')+text(1004,222,['人が確認し、','次の指示を出す'],18,DIM,anchor='middle')
    return svg('人が業務の目的を伝え、エージェントが作業ごとのSkillsを使って目的を達成し、結果を人が確認して次の指示を出す。各Skillの中に目的・意味・範囲・条件を書く',a,340)

def quarter():
    """10〜12月は希望者が教材を視聴して目的を考える準備期間。1月に目的を書けた先着15名が参加し、1月に作り、2月にチームで使い、3月に共有する。アンケートは事前・参加申込・終了時。"""
    cols = [('10〜12月（希望者）', '準備する', [['10月2週目に教材1を公開し、', '事前アンケートを取る'], ['教材2 ・ 3を、12月までに', '順次公開する'], ['教材を視聴して、実現したい', '目的を考える']], False, True),
            ('1月', '目的を持って作る', [['参加申込で目的を書く'], ['目的を書けた人を、', '先着15名で受け付ける'], ['AWS環境とKiroを発行し、', '小さく作る']], False, False),
            ('2月', 'チームで使う', [['自分のチームで使ってもらう'], ['振り返りの場で、うまく', 'いかない所を持ち寄る'], ['同じ考え方と言葉で直す']], False, False),
            ('3月', '成果を共有する', [['成果物をポータルに載せる'], ['最終共有会で見せ合う']], True, False)]
    a = ''
    for k, (month, head, items, hot, dash) in enumerate(cols):
        x = k * 286
        if k:
            a += arrow(x - 24, 150, x - 6, 150)
        box = rect(x, 8, 254, 290, PAPER, 12, ACCENT if hot else LINE)
        a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 6"') if dash else box
        a += text(x + 20, 42, month, 15, ACCENT if hot else DIM, 700) + text(x + 20, 78, head, 21, ACCENT if hot else INK, 700)
        y = 122
        for lines in items:
            a += path(f'M{x+22} {y-6} h8', ACCENT if hot else LINE, 3)
            a += text(x + 38, y, lines, 15, INK, gap=23)
            y += 23 * len(lines) + 20
    a += text(0, 346, 'アンケート', 15, DIM, 700)
    a += path('M96 340 H1112', LINE, 2)
    for x, label, anc in ((128, '事前（10月 ・ 教材の前）', 'start'), (300, '参加申込（1月 ・ 教材の後）', 'start'), (1098, '終了時（3月）', 'end')):
        a += circle(x, 340, 6, PAPER, INK)
        a += text(x - 8 if anc == 'start' else x, 370, label, 14, DIM, anchor=anc)
    return svg('10〜12月は希望者が教材を視聴して目的を考える準備期間とし、1月に目的を書けた先着15名が参加して小さく作り、2月にチームで使い、3月に成果物をポータルに載せて見せ合う。アンケートは事前、参加申込、終了時', a, 380)

def portal():
    """教材は社内のナレッジ共有ポータルで提供する。教材動画と参加者の成果物を集め、カテゴリとタグで目的に合う仕組みにたどり着けるようにする。"""
    a = person(62, 128, 1.1) + text(62, 196, '参加者', 20, INK, 700, 'middle')
    a += text(62, 228, ['動画で学び、', '成果物を載せる'], 14, DIM, anchor='middle', gap=22)
    a += arrow(112, 140, 146, 140)
    a += rect(156, 8, 788, 316, PAPER, 14, LINE)
    a += text(180, 44, 'ナレッジ共有ポータル（社内 ・ 今期はトライアル）', 20, INK, 700)
    # 教材動画：教材1〜3を学ぶ順に並べる
    a += rect(176, 64, 240, 240, PAPER, 10, LINE) + icon(196, 82, 'screen', INK, .6)
    a += text(242, 106, '教材動画', 19, INK, 700)
    for k, (no, name) in enumerate((('教材1', '目的の把握'), ('教材2', '言葉の定義'), ('教材3', '仕組みの構築'))):
        y = 150 + k * 48
        a += text(196, y, no, 15, DIM, 700) + text(252, y, name, 17, INK)
        if k < 2:
            a += path(f'M206 {y+10} V{y+28}', LINE, 2)
    a += text(196, 290, '前の教材を、次の教材の前提にする', 13, DIM)
    # 成果物：参加者が作った仕組み
    a += rect(428, 64, 240, 240, PAPER, 10, LINE) + icon(448, 76, 'doc', INK, .55)
    a += text(488, 106, '成果物', 19, INK, 700)
    a += text(448, 150, '参加者が作った仕組み', 17, INK)
    for k, t in enumerate(('目的', '使い方', '試した記録')):
        a += icon(452, 178 + k * 32, 'check', ACCENT, .4) + text(478, 192 + k * 32, t, 16, DIM)
    a += text(448, 290, '目的と一緒に載せる', 13, DIM)
    # カテゴリとタグで探す：強調。目的そのもので分類せず、目的にたどり着きやすい分類にする
    a += rect(680, 64, 244, 240, PAPER, 10, ACCENT) + icon(700, 80, 'search', ACCENT, .5)
    a += text(736, 105, 'カテゴリとタグで探す', 17, ACCENT, 700)
    a += rect(694, 132, 100, 34, PAPER, 17, LINE) + text(744, 155, '会議 ・ 報告', 14, INK, anchor='middle')
    a += rect(802, 132, 60, 34, PAPER, 17, LINE) + text(832, 155, '議事録', 14, INK, anchor='middle')
    a += rect(870, 132, 44, 34, PAPER, 17, LINE) + text(892, 155, '課題', 14, INK, anchor='middle')
    a += path('M802 178 V198 M795 191 L802 198 L809 191', ACCENT, 2)
    a += rect(700, 206, 204, 44, ACCENT, 8) + text(802, 234, '目的に合う仕組み', 16, PAPER, 700, 'middle')
    a += text(700, 290, '自分の業務で試し、参考にする', 13, DIM)
    a += arrow(952, 140, 986, 140)
    for k, x in enumerate((1016, 1076)):
        a += person(x, 128, .8)
    a += text(1046, 196, '希望した人', 17, INK, 700, 'middle')
    a += text(1046, 228, ['探して', '社内で使う'], 14, DIM, anchor='middle', gap=22)
    return svg('教材は社内のナレッジ共有ポータルで提供する。参加者は教材動画で学び、作った仕組みを成果物として目的と一緒に載せる。希望した人は、カテゴリとタグから仕組みを探して使う。今期はトライアルとする', a, 330)


def promotion():
    """いまの案件と、この企画のあとの案件を対比する。"""
    a = rect(0, 8, 520, 278, PAPER, 12, LINE) + text(28, 50, 'いまの案件', 25, weight=700)
    for i, s in enumerate(['AIを使う人が限られる', '使い方が個人の中に閉じる', '試した結果が案件に残らない']):
        y = 104 + i * 58
        a += circle(40, y - 7, 5, LINE, LINE) + text(64, y, s, 20, DIM)
    a += arrow(546, 147, 610, 147, ACCENT)
    a += rect(636, 8, 476, 278, PAPER, 12, ACCENT)
    a += text(664, 50, 'この企画のあと', 25, ACCENT, 700)
    for i, s in enumerate(['案件ごとにAIで作った仕組みが動く', '同僚がその仕組みを使う', '使った記録が次の案件へ渡る']):
        y = 104 + i * 58
        a += circle(676, y - 7, 5, ACCENT, ACCENT) + text(700, y, s, 20)
    return svg('いまの案件と、この企画のあとの案件の違い', a, 300)


def same_loop():
    """AIツールはただの道具である。大事なのは、ツールの使い方ではなく、目的をAIで達成できるかである。"""
    a = rect(0, 8, 520, 210, PAPER, 12, LINE)
    a += text(28, 50, 'AIツール（ただの道具）', 22, DIM, 700)
    a += text(28, 96, 'Copilot ・ ChatGPT ・ Claude', 20, INK, 700)
    a += text(28, 130, '開発では Kiro ・ Claude Code ・ GitHub Copilot ・ Codex', 16, INK)
    a += text(28, 172, ['操作に詳しくても、', '目的が曖昧なら成果は出ない'], 17, DIM, gap=26)
    a += arrow(546, 113, 610, 113)
    a += rect(636, 8, 476, 210, PAPER, 12, ACCENT)
    a += text(664, 50, '目的をAIで達成する', 22, ACCENT, 700)
    a += text(664, 96, ['自分やチームで', '実現したい目的を、AIで', '仕組みにして達成する'], 20, INK, 700, gap=30)
    a += text(664, 196, 'どのツールを使っても発揮できる', 17, DIM)
    for i in range(15):
        a += person(24 + i * 33, 272, .5)
    a += text(546, 272, '参加した人が、できるようになることを目指す', 21, INK, 700)
    a += text(546, 302, '準備期間は希望者、ワークショップは先着15名', 17, DIM)
    return svg('AIツールはただの道具で、操作に詳しくても目的が曖昧なら成果は出ない。大事なのは、自分やチームで実現したい目的を、AIで達成できるかである', a, 312)


def kpi_main():
    """主KPIの2つを、分子と分母と取り方で並べる。どちらもアンケートの回答で数える。"""
    panels = [('主KPI', '目的の明確化率 ── AIで実現したい目的を明確に書けたか',
               ['目的の記述で、書けた観点が', '参加申込で増えた人'], ['事前アンケートと参加申込の', '両方に回答した人'],
               ['同じ設問を、事前アンケート（視聴前）と参加申込（視聴後）で聞く', '観点：何を実現するか ・ 誰のためか ・ 何が変われば達成か', 'AIが同じ判定基準で判定し、運営が抜き取りで確認する']),
              ('主KPI', '目的達成率 ── 書いた目的をAIで達成できたか',
               ['参加申込で書いた目的を、', 'AIで「達成できた」と答えた参加者'], ['ワークショップに参加した人', '（先着15名。開始時点で固定）'],
               ['終了時アンケートで、本人が回答する', '回答しなかった人も、分母から外さない'])]
    a = ''
    for k, (kind, head, num, den, how) in enumerate(panels):
        x = k * 572
        cx = x + 270
        a += rect(x, 8, 540, 396, PAPER, 12, ACCENT)
        a += text(x + 26, 44, kind, 15, ACCENT, 700) + text(x + 26, 82, head, 18, ACCENT, 700)
        a += text(x + 26, 132, '分子', 14, DIM, 700) + text(cx + 20, 132, num, 17, INK, 700, 'middle', gap=26)
        a += path(f'M{x+90} 176 H{x+490}', INK, 2)
        a += text(x + 26, 212, '分母', 14, DIM, 700) + text(cx + 20, 212, den, 17, INK, 700, 'middle', gap=26)
        a += path(f'M{x+26} 272 H{x+514}', LINE, 1)
        a += text(x + 26, 306, '取り方', 14, DIM, 700)
        a += text(x + 26, 336, how, 14, DIM, gap=24)
    return svg('主KPIは2つ。目的の明確化率は、事前アンケートと参加申込の両方に回答した人のうち、目的の記述で書けた観点が増えた人の割合。目的達成率は、ワークショップに参加した人のうち、書いた目的をAIで達成できたと終了時アンケートで答えた人の割合', a, 412)

def kpi_sub():
    """補助の指標4つを、それぞれの設問と選択肢と並べる。"""
    cards = [('実務でAIを使う人の増加', '過去4週間に、次の業務でAIをどの程度使いましたか', '0 未利用 ／ 1 試用のみ ／ 2 実務で1回 ／ 3 実務で複数回', '個人コースで、2 ・ 3の人数の変化'),
             ('チームでの改善の実施', 'チームで使う期間に、振り返りの場を何回開きましたか', '0回 ／ 1回 ／ 2回以上（話し合った問題と直したこと）', 'チームコースで、1回以上開いた割合'),
             ('手戻りの減少', '手直しせずに使える結果は、どのくらい返ってきましたか', 'ほとんど無い ／ 3回に1回 ／ 半分 ／ たいてい ／ ほぼ毎回', '個人コースで、上がった割合'),
             ('やり方のチームへの定着', '指示の型やSkillをチームで共有しているか。新規参画者の立ち上げ期間', '共有の4つの状態 ／ 数日以内 〜 1か月より長い', 'チームコースで、上がった ・ 短くなった割合')]
    a = ''
    for k, (name, q, choice, look) in enumerate(cards):
        x = (k % 2) * 572
        y = 8 + (k // 2) * 166
        a += rect(x, y, 540, 152, PAPER, 12, LINE)
        a += text(x + 24, y + 34, name, 18, INK, 700)
        a += text(x + 24, y + 68, q, 15, INK)
        a += text(x + 24, y + 98, choice, 14, DIM)
        a += text(x + 24, y + 132, look, 15, ACCENT, 700)
    return svg('補助の指標4つ。業務別の活用、振り返りの実施、個人の変化、チームの変化を、それぞれの設問と選択肢で取る', a, 334)

def keep_loop():
    """AI活用を促進する基盤づくり。なぜ必要か（左）と、運営チーム → メンバー → ナレッジ共有ポータル → 運営チームと回る運用の流れ（右）。"""
    import math
    a = text(0, 16, 'なぜ必要か', 14, DIM, 700) + text(480, 16, '運用の流れ', 14, DIM, 700)
    whys = [('calendar', 'AIの進化は速い', '新しい機能や使い方が次々に出る'),
            ('branch', '仕組みを知るほど、目的の幅が広がる', 'AIで達成できる目的が増える'),
            ('search', 'ただ、自分から追わないと分からない', '追う人だけが、新しい使い方を知る')]
    for k, (ic, h1, h2) in enumerate(whys):
        y = 32 + k * 114
        hot = k == 2
        a += rect(0, y, 404, 100, PAPER, 12, ACCENT if hot else LINE)
        a += icon(20, y + 26, ic, ACCENT if hot else INK, .8)
        a += text(82, y + 42, h1, 17, ACCENT if hot else INK, 700)
        a += text(82, y + 72, h2, 14, DIM)
    a += text(442, 198, 'だから', 14, DIM, 700, 'middle') + arrow(418, 212, 466, 212)
    # 輪：楕円の上に3つの箱を置く。弧の両端は、楕円が箱を出る角度を計算して決める
    cx, cy, rx, ry = 796, 212, 216, 128
    bw, bh = 232, 92
    nodes = [(-90, '運営チーム', ['運用ルールと基盤を整備し、', '触れる機会を提供する']),
             (30, 'メンバー（参加者を含む）', ['機会とナレッジを活用し、', '新しい事例を創出する']),
             (150, 'ナレッジ共有ポータル', ['事例とノウハウを集約し、', '次の活用に再利用する'])]
    P = lambda t: (cx + rx * math.cos(math.radians(t)), cy + ry * math.sin(math.radians(t)))
    def leave(ang, step):
        x0, y0 = P(ang); t = ang
        while True:
            t += step; x, y = P(t)
            if abs(x - x0) > bw / 2 + 10 or abs(y - y0) > bh / 2 + 10: return t
    labels = [('機会を提供する', 'start', 14, 0), ('事例を登録する', 'middle', 0, 28), ('ナレッジで改善する', 'end', -14, 0)]
    for k in range(3):
        a0 = leave(nodes[k][0], .5); nxt = nodes[(k + 1) % 3][0]
        a1 = leave(nxt if nxt > nodes[k][0] else nxt + 360, -.5)
        p0, p1 = P(a0), P(a1)
        a += path(f'M{p0[0]:.1f} {p0[1]:.1f} A{rx} {ry} 0 0 1 {p1[0]:.1f} {p1[1]:.1f}', ACCENT, 2.5)
        t = math.radians(a1)
        dx, dy = -rx * math.sin(t), ry * math.cos(t); n = math.hypot(dx, dy); dx, dy = dx / n, dy / n
        nx, ny = -dy, dx
        b1 = (p1[0] - 12 * dx + 6 * nx, p1[1] - 12 * dy + 6 * ny); b2 = (p1[0] - 12 * dx - 6 * nx, p1[1] - 12 * dy - 6 * ny)
        a += path(f'M{b1[0]:.1f} {b1[1]:.1f} L{p1[0]:.1f} {p1[1]:.1f} L{b2[0]:.1f} {b2[1]:.1f}', ACCENT, 2.5)
        m = P((a0 + a1) / 2); lab, anc, ox, oy = labels[k]
        a += text(m[0] + ox, m[1] + oy, lab, 14, ACCENT, 700, anc)
    for ang, head, body in nodes:
        x, y = P(ang)
        a += rect(x - bw / 2, y - bh / 2, bw, bh, PAPER, 12, LINE)
        a += text(x, y - 12, head, 17, INK, 700, 'middle')
        a += text(x, y + 14, body, 14, DIM, anchor='middle', gap=20)
    a += text(cx, cy - 4, '継続的なAI活用が、', 16, INK, 700, 'middle') + text(cx, cy + 22, '組織に定着する', 16, INK, 700, 'middle')
    return svg('AIの進化は速く、仕組みを知るほど目的の幅が広がるが、自分から追わないと分からない。だから、運営チームが機会を提供し、メンバーが事例を創出してポータルに登録し、集約したナレッジで運営チームが改善する流れを回し、継続的なAI活用を組織に定着させる', a, 372)

def ops_cards():
    """運営チームの役割。運用の流れの中で基盤と機会の提供を担当し、今期に先行トライアルで行う3つ。"""
    cards = [('screen', '環境と運用ルールを整える', ['AWS環境と開発環境、目的を限定した', 'ナレッジ共有ポータルを構築し、', '載せる項目と運用ルールを決める']),
             ('branch', '浸透の運用を設計する', ['ナレッジが集まり、使われ続ける', '運用を設計し、参加者と', '希望者で試す']),
             ('chat', 'トレンドに触れる機会を作る', ['AIアイデアソンやワークショップを', '定期的に開き、チームでの推進の', 'プラクティスを整える'])]
    a = ''
    for k, (ic, head, body) in enumerate(cards):
        x = k * 380
        a += rect(x, 8, 352, 250, PAPER, 12, LINE)
        a += icon(x + 26, 32, ic, INK, .8)
        a += text(x + 26, 120, head, 21, INK, 700)
        a += path(f'M{x + 26} 140 H{x + 326}', LINE, 1)
        a += text(x + 26, 176, body, 16, DIM, gap=27)
    a += rect(0, 282, 1112, 56, PAPER, 12, ACCENT)
    a += text(556, 317, '運営チームは人を選んだ約3名で構成し、方向を統一する。ナレッジを出すのは、メンバー全員である', 17, ACCENT, 700, 'middle')
    return svg('運営チームは、環境と運用ルールを整え、浸透の運用を設計し、トレンドに触れる機会を作る。運営は約3名で担当し、ナレッジを出すのは全員である', a, 346)

def portal_design():
    """ポータルの設計。汎用のサービスでは、置き方も探し方も人に委ねられてばらばらになる。このポータルは目的を限定し、載せる項目とカテゴリ ・ タグを決めて、迷わず載せて探せるようにする。"""
    def dashed(d):
        return path(d, LINE, 2).replace('stroke-linecap', 'stroke-dasharray="5 5" stroke-linecap')
    a = rect(0, 8, 470, 300, PAPER, 12, LINE)
    a += text(22, 40, '汎用のサービス', 18, INK, 700) + text(22, 62, 'GitHub ・ Confluence など', 13, DIM)
    docs = [(236, 84, -12), (330, 104, 9), (404, 78, -6), (262, 164, 14), (372, 176, -10), (300, 226, 6), (410, 236, 12), (222, 232, -8)]
    for x, y, r in docs:
        a += f'<g transform="rotate({r} {x + 12} {y + 15})">' + icon(x, y, 'doc', DIM, .5) + '</g>'
    for k, (py, d, qx, qy) in enumerate([(122, 'M92 116 Q170 60 232 92', 214, 76), (186, 'M92 180 Q240 250 366 190', 378, 214), (250, 'M92 244 Q150 290 218 246', 196, 276)]):
        a += person(62, py, .7) + dashed(d)
        a += text(qx, qy, '?', 20, DIM, 700)
    a += text(22, 292, '使い方が人に委ねられ、置き方も探し方もばらばら', 14, DIM)
    # 目的を限定する
    a += text(556, 146, '目的を限定する', 14, ACCENT, 700, 'middle') + arrow(486, 160, 626, 160, ACCENT)
    # このポータル
    a += rect(642, 8, 470, 300, PAPER, 12, ACCENT)
    a += text(664, 40, 'このポータル', 18, ACCENT, 700) + text(664, 62, 'ナレッジ共有ポータル', 13, DIM)
    a += rect(664, 84, 188, 124, PAPER, 8, LINE)
    a += text(680, 106, '載せる項目は決まっている', 12, DIM, 700)
    for k, lab in enumerate(['目的', '使い方', '試した記録']):
        y = 134 + k * 26
        a += text(680, y, lab, 13, INK, 700) + path(f'M760 {y + 3} H836', LINE, 1.5)
    a += rect(872, 84, 218, 34, PAPER, 17, LINE) + icon(884, 92, 'search', INK, .34) + text(912, 106, '議事録', 13, INK)
    cats = [('会議 ・ 報告', ['議事録', '週次報告']), ('開発', ['レビュー', 'テスト']), ('調査', ['比較', '要約'])]
    for k, (cat, tags) in enumerate(cats):
        x = 872 + k * 75
        a += rect(x, 142, 68, 96, PAPER, 8, LINE)
        a += text(x + 34, 226, cat, 11, INK, 700, 'middle')
        for m, tag in enumerate(tags):
            hot = (k == 0 and m == 0)
            a += rect(x + 6, 152 + m * 30, 56, 22, PAPER, 11, ACCENT if hot else LINE)
            a += text(x + 34, 167 + m * 30, tag, 11, ACCENT if hot else DIM, 700 if hot else 400, 'middle')
    a += path('M906 118 V148', ACCENT, 2) + path('M899 141 L906 148 L913 141', ACCENT, 2)
    a += text(664, 236, ['載せる項目が決まっていて、', 'カテゴリとタグで迷わず探せる'], 14, INK, 700, gap=22)
    a += text(664, 292, 'シンプルで直感的にし、共有のハードルを下げる', 14, DIM)
    a += rect(0, 324, 1112, 48, PAPER, 12, LINE)
    a += text(556, 354, '教材で教える「目的に合った高さを決める」考え方を、ポータルにも適用する', 17, INK, 700, 'middle')
    return svg('汎用のサービスでは、使い方が人に委ねられ、置き方も探し方もばらばらになる。目的を限定したこのポータルでは、載せる項目が決まっていて、カテゴリとタグで迷わず探せる', a, 378)

def closing():
    """まとめ。ワークショップで、目的をAIで達成できる人を増やし、基盤づくりで、そのやり方が部内で続く形を作る。2つを合わせて、目的をAIで達成するやり方を部内に定着させる。"""
    a = ''
    a += rect(0, 8, 330, 250, PAPER, 12, LINE)
    for k in range(3):
        a += person(120 + k * 45, 70, .7)
    a += text(165, 146, 'ワークショップ', 24, INK, 700, 'middle')
    a += text(165, 190, ['目的をAIで達成できる', '人を増やす'], 17, DIM, anchor='middle', gap=26)
    a += text(365, 146, '＋', 40, DIM, 700, 'middle')
    a += rect(400, 8, 330, 250, PAPER, 12, LINE)
    a += icon(538, 44, 'search', INK, .9)
    a += text(565, 146, 'AI活用を促進する', 22, INK, 700, 'middle') + text(565, 176, '基盤づくり', 22, INK, 700, 'middle')
    a += text(565, 214, ['そのやり方が、部内で', '続く形を作る'], 17, DIM, anchor='middle', gap=26)
    a += arrow(740, 133, 772, 133, ACCENT)
    a += rect(782, 8, 330, 250, PAPER, 12, ACCENT)
    a += text(947, 60, '目指す先', 16, ACCENT, 700, 'middle')
    a += text(947, 118, ['目的をAIで達成する', 'やり方が、部内で', '個人に頼らず続く'], 24, ACCENT, 700, 'middle', gap=36)
    a += text(0, 300, '最初の一歩：10月2週目に、希望者へ教材1を公開し、事前アンケートを取る', 17, INK, 700)
    return svg('ワークショップで目的をAIで達成できる人を増やし、基盤づくりでそのやり方が部内で続く形を作る。2つを合わせて、目的をAIで達成するやり方が部内で個人に頼らず続く状態を目指す。最初の一歩は、10月2週目に希望者へ教材1を公開し、事前アンケートを取ること', a, 312)

def part2_open():
    """目指す姿。基盤を今期に先行トライアルで試し、次の期に部内へ展開する。定着すると、機会とナレッジが部内で回り、目的をAIで達成するやり方が個人に頼らず続く。"""
    stages = [(8, '今期（ワークショップと並行）', '先行トライアル', ['ポータルとワークショップ ・', 'アイデアソンを、参加者と', '希望者で試す']),
              (200, '次の期', '部内へ展開', ['試行の結果をもとに、ポータルを', '部内に開き、ワークショップと', 'アイデアソンを定期的に開く'])]
    a = ''
    for y, when, head, body in stages:
        a += rect(0, y, 360, 160, PAPER, 12, LINE)
        a += text(22, y + 30, when, 13, DIM, 700) + text(22, y + 62, head, 20, INK, 700)
        a += text(22, y + 96, body, 14, INK, gap=22)
    a += path('M180 168 V192 M173 185 L180 192 L187 185', LINE, 2)
    a += arrow(372, 184, 412, 184)
    a += rect(420, 8, 692, 352, PAPER, 12, ACCENT)
    a += text(448, 42, '基盤が部内に定着すると', 15, ACCENT, 700)
    rows = [('calendar', '新しいAIの使い方に、全員が触れている', '定期的なワークショップとアイデアソンで、機会が途切れない'),
            ('search', '必要なナレッジを、探して使える', 'ナレッジ共有ポータルに、事例とノウハウが集まっている'),
            ('branch', '事例が載り続け、やり方が改善される', '使った人の事例をもとに、運営が基盤を改善する')]
    for k, (ic, head, sub) in enumerate(rows):
        y = 64 + k * 76
        a += icon(448, y + 6, ic, INK, .7)
        a += text(510, y + 24, head, 18, INK, 700) + text(510, y + 50, sub, 14, DIM)
    a += path('M448 300 H1084', LINE, 1)
    a += text(448, 338, '個人が追わなくても、目的をAIで達成するやり方が部内で続く', 19, ACCENT, 700)
    return svg('基盤を今期に先行トライアルで試し、次の期に部内へ展開する。部内に定着すると、全員が新しい使い方に触れ、必要なナレッジを探して使え、事例が載り続けて改善される。その結果、個人が追わなくても、目的をAIで達成するやり方が部内で続く', a, 368)
def kpi_sub_view():
    """補助の指標4つを、参加申込で選んだ取り組み（個人／チーム）ごとに2つずつ並べる。設問は付録に置く。"""
    lanes = [(0, '個人コース', '個人ではまだ目的を達成できていない人',
              [('実務でAIを使う人の増加', '担当業務で、実務にAIを使った人の増え方', '実務でAIを使う範囲が広がったか'),
               ('手戻りの減少', '手直しせずに使える出力が、事前より増えた人の割合', '毎回同じ品質が得られるようになったか')]),
             (572, 'チームコース', '個人では既にできている人',
              [('チームでの改善の実施', 'チームで使う期間に、振り返りの場を1回以上開いた人の割合', 'チームで使い、直す流れを実行できたか'),
               ('やり方のチームへの定着', '型の共有が進んだ人、立ち上げが短くなった人の割合', 'チームで品質がそろい、新しい人もすぐ使えるか')])]
    a = ''
    for x, head, who, cards in lanes:
        a += text(x, 20, head, 18, INK, 700) + text(x + 18 * len(head) + 14, 20, who, 14, DIM)
        for k, (name, what, judge) in enumerate(cards):
            y = 36 + k * 150
            a += rect(x, y, 540, 138, PAPER, 12, LINE)
            a += text(x + 24, y + 36, name, 19, INK, 700)
            a += text(x + 24, y + 72, what, 15, INK)
            a += text(x + 24, y + 112, '判定 ・ ' + judge, 15, DIM)
    return svg('補助の指標4つを、参加申込で選んだ取り組みごとに分ける。個人コースは実務でAIを使う人の増加と手戻りの減少、チームコースはチームでの改善の実施とやり方のチームへの定着', a, 326)

def kpi_part2():
    """AI活用を促進する基盤づくりの指標4つを、それぞれで何を判定するかと並べる。"""
    cards = [('チームでの改善の継続', '終了後も振り返りの場を続けると決めた参加者の割合', '義務ではなく、自然に続く形になったか', True),
             ('参加者以外での再利用', '希望者のうち、見つけた仕組みを自分の業務で試した人の割合', '部内に根付く方向へ、今期どこまで近づいたか', False),
             ('必要なナレッジの見つけやすさ', '探していた仕組みやノウハウにたどり着けた割合', 'ポータルを、次の期に部内へ広げる価値が在るか', False),
             ('ナレッジを出す人の広がり', 'ポータルに、自分のノウハウや使い方を1件以上載せた希望者の割合', 'ナレッジを出すのは全員、という形がどこまで起きたか', False)]
    a = ''
    for k, (name, what, judge, hot) in enumerate(cards):
        x = (k % 2) * 572
        y = 8 + (k // 2) * 166
        a += rect(x, y, 540, 152, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 24, y + 38, name, 20, ACCENT if hot else INK, 700)
        a += text(x + 24, y + 76, what, 15, INK)
        a += text(x + 24, y + 122, '判定 ・ ' + judge, 15, DIM)
    return svg('AI活用を促進する基盤づくりの指標4つ。チームでの継続、参加者以外への広がり、たどり着きやすさ、ナレッジを出した希望者の割合と、それぞれで判定すること', a, 334)

def down(x, y1, y2, color=LINE):
    return path(f'M{x} {y1} V{y2} m-7 -7 l7 7 7 -7', color)


def forms_timeline():
    """3本のフォームを時期の順に並べ、回答者名で突き合わせることを示す。"""
    cards = [
        ('教材の視聴前 ・ 希望者', '事前', ['利用頻度 ・ 能力 ・ 業務別', '実現したい目的 ・ 達成条件', '個人とチームの状態 ・ 困りごと']),
        ('教材の視聴後 ・ ワークショップの開始前', '参加申込', ['実現したい目的 ・ 達成条件', '（事前と同じ設問）', '参加の希望']),
        ('ワークショップの終わり ・ 参加者', '終了時', ['同じ設問で変化を見る', '目的の結果 ・ 振り返り', 'ポータル']),
    ]
    a = ''
    for i, (when, name, items) in enumerate(cards):
        x = i * 384
        cx = x + 172
        a += text(cx, 26, when, 17, DIM, 400, 'middle')
        a += rect(x, 40, 344, 210, PAPER, 12, LINE)
        a += text(cx, 84, name, 25, INK, 700, 'middle')
        for j, s_ in enumerate(items):
            y = 124 + j * 34
            a += circle(x + 32, y - 6, 4, LINE, LINE)
            a += text(x + 52, y, s_, 17, DIM)
        if i < 2:
            a += arrow(x + 346, 145, x + 382, 145)
    a += path('M172 250 V288 M556 250 V288 M940 250 V288 M172 288 H940', ACCENT, 2.5)
    a += text(556, 320, '回答者名（Formsが自動で記録）で、同じ人の回答を突き合わせる', 20, ACCENT, 700, 'middle')
    return svg('事前（教材の視聴前）・ 参加申込（視聴後）・ 終了時の3本のフォームを、回答者名で突き合わせる', a, 332)

def final_share():
    """成果の共有は3月の最終共有会の1回。成果物をポータルに載せ、共有会で見せ合う。"""
    cards = [('成果物をポータルに載せる', 'doc', ['目的 ・ 使い方 ・ 試した記録を', '添えて載せる'], False),
             ('最終共有会', 'chat', ['軽食を囲むオフラインの交流会で、', '短いデモと会話で見せ合う'], True)]
    a = ''
    for k, (head, ic, sub, hot) in enumerate(cards):
        x = k * 592
        a += rect(x, 8, 520, 220, PAPER, 12, ACCENT if hot else LINE)
        a += icon(x + 32, 36, ic, ACCENT if hot else INK, .7)
        a += text(x + 32, 126, head, 24, ACCENT if hot else INK, 700)
        a += text(x + 32, 172, sub, 18, DIM, gap=30)
    a += arrow(532, 118, 580, 118)
    return svg('成果物をポータルに載せ、最終共有会で短いデモと会話で見せ合う', a, 236)


def background():
    """個人のAI活用は進んだが、チームのやり方として浸透せず、チームや組織での活用はなかなか進まない、という現状。"""
    items = [('一人でも作れるようになった', ['資料やアプリ、ツールを', 'AIで一人で作れる']),
             ('活用のレベルは人しだい', ['スキルの高い人は自分用の', '仕組みを作り、効率を上げる']),
             ('チームには浸透しにくい', ['一部の人の効率は上がっても、', 'チームではなかなか進まない'])]
    a = text(0, 24, 'いま起きていること', 18, DIM)
    for i, (head, sub) in enumerate(items):
        x = i * 388
        a += rect(x, 40, 336, 150, PAPER, 12, LINE)
        a += text(x + 28, 86, head, 21, INK, 700)
        a += text(x + 28, 128, sub, 17, DIM, gap=28)
        if i < 2:
            a += arrow(x + 344, 115, x + 380, 115)
    a += rect(0, 216, 1112, 62, PANEL, 12)
    a += text(556, 255, '問われているのは、個人の効率ではない。チームや組織の目的を、AIで達成できるかである。', 22, ACCENT, 700, 'middle')
    return svg('資料やアプリ、ツールを一人でも作れるようになり、個人の活用は進んだが、チームのやり方として浸透せず、チームや組織での活用はなかなか進まない', a, 290)


def leverage():
    """3つを身につけると、仕事の進め方がどう変わるか。いまと、身につけたあとを、個人とチームの2つの層で、人と資料の図で並べる。"""
    # いまと身につけたあとは、枠で分ける。地は塗らない（灰色の地は文字と線が沈む）。あいだは流れの矢印と別の形（太い山形）で、比較であることを示す
    a = rect(76, 8, 440, 344, PAPER, 12, LINE) + rect(588, 8, 524, 344, PAPER, 12, ACCENT)
    a += text(96, 38, 'いま', 17, DIM, 700) + text(608, 38, '身につけると', 17, ACCENT, 700)
    a += path('M534 146 L566 180 L534 214', ACCENT, 6)
    a += text(0, 118, '個人', 21, INK, 700) + text(0, 276, 'チーム', 21, INK, 700)
    # 個人：いまは頼み方が毎回変わり、返りの品質がばらばら。身につけると、仕組みで毎回同じ品質
    L, R = 0, 0
    a += person(150, 110, .8) + text(150, 166, '頼み直しを繰り返す', 14, DIM, anchor='middle')
    a += arrow(186, 104, 222, 104)
    for x, y, sc in ((236, 90, .42), (290, 76, .62), (358, 86, .5)):
        a += icon(x, y, 'doc', DIM, sc)
    a += text(354, 166, '目的に合うかは毎回ばらばら', 14, DIM, anchor='middle')
    a += person(664, 110, .8) + text(664, 166, '同じ仕組みで頼む', 14, DIM, anchor='middle')
    a += arrow(700, 104, 728, 104)
    a += rect(736, 82, 104, 44, ACCENT, 8) + text(788, 111, '仕組み', 18, PAPER, 700, 'middle')
    a += arrow(848, 104, 878, 104)
    for x in (888, 944, 1000):
        a += icon(x, 80, 'doc', INK, .5) + icon(x + 22, 114, 'check', ACCENT, .45)
    a += text(960, 166, '手戻りが減り、目的に時間を使える', 15, ACCENT, 700, 'middle')
    a += path('M96 192 H496', LINE, 1) + path('M608 192 H1092', LINE, 1)
    # チーム：いまは頼み方を本人だけが知り、新メンバーは一から。身につけると、同じ仕組みで品質がそろう
    a += person(164, 270, .8) + icon(192, 240, 'doc', INK, .42)
    a += text(166, 326, 'やり方は本人だけが知る', 14, DIM, anchor='middle')
    a += path('M256 236 V304', LINE, 2).replace('stroke-linecap', 'stroke-dasharray="4 6" stroke-linecap')
    for x in (314, 370, 426):
        a += person(x, 270, .7, DIM)
    a += text(372, 326, '他の人 ・ 新メンバーは一から覚える', 14, DIM, anchor='middle')
    a += rect(624, 248, 104, 44, ACCENT, 8) + text(676, 277, '仕組み', 18, PAPER, 700, 'middle')
    a += arrow(736, 270, 772, 270)
    for x in (820, 900, 980):
        a += person(x, 270, .7)
    a += text(980, 311, '新メンバー', 13, DIM, anchor='middle')
    a += text(860, 336, 'やり方が残り、誰が担当しても同じ成果が出る', 15, ACCENT, 700, 'middle')
    return svg('いまと身につけたあとを枠で分けて並べる。いまは頼み方が毎回違い返りの品質がばらばらで、頼み方は本人だけが知る。3つを身につけると、仕組みで毎回同じ品質が得られ、チームで品質がそろい新メンバーもすぐ使える', a, 356)


def structure():
    """企画は2つの取り組みで構成する。ワークショップは参加者が目的をAIで達成できるようになる場、AI活用を促進する基盤づくりは運営が組織に根付かせる取り組み。"""
    cols = [('ワークショップ', '参加者（希望者から先着15名）', ['個人とチームで、', '目的をAIで達成する'], '使う側', True),
            ('AI活用を促進する基盤づくり', '運営チーム（人を選んだ約3名）', ['目的をAIで達成することが、', '組織に根付き自然に続く'], '作る側', False)]
    a = ''
    for i, (name, who, goal, side, hot) in enumerate(cols):
        x = i * 572
        a += rect(x, 8, 540, 220, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 28, 50, name, 25, ACCENT if hot else INK, 700)
        a += text(x + 512, 50, side, 16, DIM, anchor='end')
        a += text(x + 28, 96, '主体', 15, DIM, 700) + text(x + 84, 96, who, 18, INK)
        a += text(x + 28, 144, '目標', 15, DIM, 700) + text(x + 84, 144, goal, 19, INK, 700, gap=30)
    a += rect(0, 252, 1112, 56, PAPER, 12, LINE)
    a += text(556, 287, '運営チームが環境と運用を整え、メンバー全員がナレッジを出す。両者は、目的をAIで達成するやり方が定着した組織を目指す', 18, INK, 700, 'middle')
    return svg('企画は2つの取り組みで構成する。ワークショップでは、参加者が個人とチームで目的をAIで達成する。AI活用を促進する基盤づくりでは、運営が組織に根付かせる', a, 320)


def layers():
    """組織の姿へ至る3層と、ワークショップが担当する範囲。"""
    rows = [('1', '個人', '共通の言葉と、実務での成功体験', 'ワークショップで実施', False),
            ('2', 'チーム', 'チームで使い、振り返って直す習慣と、各チームの推進役', 'ワークショップで実施', False),
            ('3', '組織', 'ナレッジが集まり使われる環境と運用、継続的な活用支援', '先行トライアル', True)]
    a = text(0, 22, '目指す組織へ至る3つの層', 18, DIM)
    a += text(1112, 22, '目指す組織：目的に対してAIを正しく使い、目的を達成するやり方が定着した集団', 16, ACCENT, 700, 'end')
    for i, (no, layer, need, scope, hot) in enumerate(rows):
        y = 40 + i * 92
        a += rect(0, y, 1112, 80, PAPER, 12, ACCENT if hot else LINE)
        a += text(28, y + 49, no, 22, DIM, 700)
        a += text(64, y + 49, layer, 24, INK, 700)
        a += text(190, y + 49, need, 19, INK)
        a += rect(846, y + 18, 242, 44, ACCENT, 22) if hot else rect(846, y + 18, 242, 44, PAPER, 22, LINE)
        a += text(967, y + 47, scope, 17, PAPER if hot else INK, 700, 'middle')
    return svg('組織の姿へ至る3層。個人とチームの層はワークショップで実施し、組織の層は今期に先行トライアルを行う', a, 320)


def vision():
    """ワークショップの先に、組織として目指す姿を3つの時期で示す。"""
    cols = [('2027年3月', '本企画の終了', ['参加者が、自分の目的を', 'AIで達成している'], ['ポータルにナレッジが集まり始める', '浸透の運用を小さく試した'], False),
            ('次の期', '部内へ展開', ['試行の結果をもとに、', '部内へ広げる'], ['推進役が各チームで実践する', 'アイデアソンで活用を続ける'], False),
            ('その先', '目指す組織', ['目的に対してAIを正しく使い、', '目的を達成するやり方が', '定着した集団'], ['顧客の業務でも発揮できる'], True)]
    a = ''
    for i, (when, head, main, sub, hot) in enumerate(cols):
        x = i * 386
        a += rect(x, 8, 340, 300, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 26, 46, when, 18, ACCENT if hot else DIM, 700)
        a += text(x + 26, 88, head, 25, INK, 700)
        a += path(f'M{x + 26} 108 H{x + 314}')
        a += text(x + 26, 146, main, 18, INK, 700 if hot else 400, gap=28)
        a += text(x + 26, 146 + 28 * len(main) + 34, sub, 17, DIM, gap=28)
        if i < 2:
            a += arrow(x + 348, 158, x + 378, 158)
    return svg('2027年3月に参加者が目的を達成し、次の期に試行の結果をもとに部内へ広げ、その先に目的をAIで達成するやり方が定着した組織を目指す', a, 320)
