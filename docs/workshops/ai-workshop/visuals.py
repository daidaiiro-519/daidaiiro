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
    return f'<g transform="translate({x} {y}) scale({scale})">'+circle(0,-14,12,PAPER,color)+path('M-24 28 v-6 c0 -27 48 -27 48 0 v6',color,2.5)+'</g>'

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
    a=person(95,147,1.6)+text(95,236,'参加者',25,weight=700,anchor='middle')
    a+=person(1013,147,1.6)+text(1013,236,'運営',25,weight=700,anchor='middle')
    a+=rect(336,12,442,107,PAPER,12,LINE)+icon(362,39,'book',scale=.8)
    a+=text(426,53,'ブログ・動画',25,weight=700)+text(426,86,'必要なときに、自分で学ぶ',19,DIM)
    a+=path('M969 120 V65 H778',INK,3)+path('M336 65 H137 V110 M130 102 L137 110 L144 102',INK,3)
    a+=rect(336,182,442,112,ACCENT)+icon(365,216,'chat',PAPER,.8)
    a+=text(428,223,'Teamsで相談する',25,PAPER,700)+text(428,259,'投稿 ＋ 月1回の定例相談会（案）',17,PAPER)
    a+=path('M137 187 V260 H336',ACCENT,3)+path('M778 260 H969 V191 M962 199 L969 191 L976 199',ACCENT,3)
    return svg('運営のブログや動画から学び、Teamsで運営に相談する',a)

def handoff():
    a=person(105,134,1.5)+text(105,220,'自分',23,weight=700,anchor='middle')
    a+=arrow(169,137,359,137)
    a+=rect(385,20,349,251,PAPER,12,LINE)+icon(409,43,'doc',scale=.7)
    a+=text(465,71,'AIで動く仕組み',24,weight=700)
    for y,t in [(123,'知識・ノウハウ'),(175,'仕事の手順'),(227,'判断の目安')]:
        a+=circle(421,y-7,5,INK,INK)+text(445,y,t,22)
    a+=arrow(756,137,919,137)
    a+=circle(1002,127,66,ACCENT,ACCENT)+person(1002,123,1.25,PAPER)
    a+=text(1002,227,'他の人も使う',23,weight=700,anchor='middle')
    a+=text(557,316,'自分の知識やノウハウを、他の人の仕事にも役立てる',25,ACCENT,700,'middle')
    return svg('自分の知識や手順をAIで動く仕組みにし、他の人も業務改善に使う',a)

def evaluation():
    a=rect(0,8,540,278,PAPER,12,LINE)+text(28,47,'同じ人の回答を比較する',25,weight=700)
    for i,t in enumerate(['事前','中間','終了']):
        x=98+i*165
        if i<2:a+=path(f'M{x+30} 97 H{x+135}')
        a+=circle(x,97,24,PANEL,LINE)+text(x,104,str(i+1),20,anchor='middle')
        a+=text(x,147,t,21,anchor='middle')
    a+=text(28,186,['実務で使った日数 ／ できることを5段階で回答'],17,DIM)
    a+=rect(28,202,484,44,ACCENT,8)
    a+=text(46,232,'向上・同じ・低下の人数',24,PAPER,700)
    a+=text(28,272,'項目ごとに集計する',17,DIM)
    a+=rect(574,8,538,278,PAPER,12,LINE)+text(602,47,'実際にできたことを確認する',25,weight=700)
    a+=icon(620,84,'doc',scale=.9)+arrow(696,112,741,112)+icon(782,86,'check',scale=1.05)
    a+=text(874,105,['成果物','使ってもらった結果'],20)
    a+=text(602,192,'新しくできたことの実例',21,weight=700)
    a+=text(602,238,['誰に使ってもらい、どんな反応だったかを','本人が書く'],20,DIM)
    return svg('3時点のアンケートの人数比較と、成果物および本人が書いた結果による確認を組み合わせる',a,310)

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

def agents():
    a=person(95,98,1.5)+text(95,185,'人',23,weight=700,anchor='middle')+text(95,222,['してほしいことを','伝える'],18,DIM,anchor='middle')
    a+=arrow(169,113,311,113)
    a+=rect(335,9,448,278,PAPER,14,LINE)+text(559,52,'エージェント',27,weight=700,anchor='middle')
    a+=text(559,94,'道具を使って仕事を進める',20,DIM,anchor='middle')
    a+=icon(413,124,'search',scale=.6)+icon(533,124,'doc',scale=.6)+icon(653,124,'screen',scale=.6)
    a+=rect(358,192,402,73,ACCENT,10)+text(379,224,'Skills',20,PAPER,700)+text(379,249,'必要なときに読む手順・判断の目安',17,PAPER)
    a+=arrow(808,113,925,113)+icon(972,69,'doc',scale=1.25)
    a+=text(1004,187,'結果',23,weight=700,anchor='middle')+text(1004,222,['人が確認し、','次の指示を出す'],18,DIM,anchor='middle')
    return svg('人が依頼し、Skillsを参照するエージェントが道具を使い、結果を人が確認する',a,310)

def learning():
    a=rect(0,12,526,277,PAPER,12,LINE)+text(30,54,'考え方を学ぶ',27,weight=700)
    a+=rect(30,82,181,137,PANEL,8)+path('M101 116 l0 65 53 -32 Z',ACCENT,1,ACCENT)
    a+=text(245,117,['何を任せるか','何を伝えるか','どう確認するか'],23,gap=43)
    a+=text(30,264,'短い動画で、繰り返し学ぶ',19,DIM)
    a+=rect(572,12,540,277,PAPER,12,LINE)+text(602,54,'AIツールを試す',27,weight=700)
    for i,t in enumerate(['作る','動かす','確認する']):
        x=659+i*176
        a+=circle(x,139,44,PANEL,LINE)+text(x,147,t,20,weight=700,anchor='middle')
        if i<2:a+=arrow(x+55,139,x+122,139)
    a+=text(602,228,['最小限の手順から始め、','自分の課題へつなげる。'],20,DIM)
    return svg('考え方の動画で学び、AIツールで作る・動かす・確認するまでを試す',a,310)


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


def horizon():
    """個人に留まるナレッジを共有リポジトリへ集め、使った人が次の作り手になる。"""
    a = text(0, 22, 'いまは個人の中に留まる', 18, DIM)
    for i, name in enumerate(['Skills', 'Agents', 'Tool']):
        y = 44 + i * 76
        a += person(24, y + 34, .75)
        a += rect(54, y, 186, 62, PAPER, 10, LINE) + text(147, y + 39, name, 20, weight=700, anchor='middle')
    a += arrow(256, 155, 316, 155)
    a += rect(338, 54, 372, 202, PANEL, 12)
    a += text(524, 92, '共有リポジトリを1か所', 22, INK, 700, 'middle')
    a += rect(360, 112, 328, 60, PAPER, 8, LINE)
    a += text(524, 138, '部署または本部の共有GitHub', 19, INK, 700, 'middle')
    a += text(524, 162, '第1案', 15, DIM, 400, 'middle')
    a += rect(360, 182, 328, 60, PAPER, 8, LINE)
    a += text(524, 208, '部内のAWS CodeCommit', 19, INK, 700, 'middle')
    a += text(524, 232, '本部との調整が難航する場合', 15, DIM, 400, 'middle')
    a += arrow(726, 155, 786, 155)
    a += text(808, 22, '誰でも取り出して使える', 18, DIM)
    a += rect(808, 48, 304, 82, PAPER, 10, LINE) + text(834, 96, '他部署の案件', 23, weight=700)
    a += rect(808, 176, 304, 82, PAPER, 10, LINE) + text(834, 224, '顧客の案件', 23, weight=700)
    a += path('M960 272 V300 H147 V278 M140 285 L147 278 L154 285', ACCENT, 2.5)
    a += text(552, 330, '使った人が、次の作り手になる', 21, ACCENT, 700, 'middle')
    return svg('個人に留まるナレッジを共有リポジトリへ集め、他部署と顧客の案件で取り出し、使った人が次の作り手になる', a, 338)


def same_loop():
    """案件に残るのは仕組み、人に残るのは力。その力を持つ人を増やす。"""
    a = rect(0, 8, 520, 190, PAPER, 12, LINE)
    a += text(28, 50, '案件に残るもの', 22, DIM, 700)
    a += rect(28, 72, 464, 62, PANEL, 10)
    a += text(56, 111, 'AIで作った仕組み', 23, INK, 700)
    a += text(28, 173, 'その案件の中で使われる', 19, DIM)
    a += arrow(546, 103, 610, 103)
    a += rect(636, 8, 476, 190, PAPER, 12, ACCENT)
    a += text(664, 50, '人に残るもの', 22, ACCENT, 700)
    a += text(664, 111, '課題をAIで解決する力', 26, INK, 700)
    a += text(664, 173, '自社の次の案件でも、顧客の案件でも使える', 19, DIM)
    for i in range(15):
        a += person(24 + i * 33, 262, .5)
    a += text(546, 271, 'この力を持つ人を、下期に15名育成する', 21, INK, 700)
    return svg('作った仕組みはその案件に残り、課題をAIで解決する力は人に残る。その力を持つ人を15名育成する', a, 300)


def kpi_main():
    """主KPIの分子と分母を、それを取る設問の実物と並べる。"""
    a = text(0, 24, '数え方', 18, DIM)
    a += rect(0, 40, 470, 104, PAPER, 12, LINE)
    a += text(24, 80, ['他の人に使ってもらい、', '役立ったと報告できた参加者数'], 21, INK, 700, gap=34)
    a += path('M0 166 H470', ACCENT, 3)
    a += rect(0, 188, 470, 74, PANEL, 12)
    a += text(24, 233, '開始時の参加者数（15名）', 21, INK, 700)
    a += text(0, 296, '目標値は6割を暫定とし、事前調査を見て確定する', 17, DIM)
    a += text(560, 24, 'この数字を取る設問（終了時のフォーム）', 18, DIM)
    a += rect(560, 40, 552, 104, PAPER, 10, LINE)
    a += text(584, 78, 'R3　誰に使ってもらいましたか。', 20, INK, 700)
    a += text(584, 116, '○ 同僚・チーム　○ 他の参加者　○ まだ', 19, DIM)
    a += rect(560, 160, 552, 104, PAPER, 10, LINE)
    a += text(584, 198, 'R4　使った人の反応はどうでしたか。', 20, INK, 700)
    a += text(584, 236, '○ 役立った　○ 一部役立った　○ 役立たなかった', 19, DIM)
    a += text(560, 296, '「役立った」「一部役立った」を選んだ人を、分子に数える', 17, ACCENT, 700)
    return svg('主KPIの分子と分母、およびその数字を取る終了時の設問と選択肢', a, 310)


def kpi_sub():
    """補助の指標2つを、それぞれの設問の実物と並べる。"""
    a = rect(0, 8, 540, 276, PAPER, 12, LINE)
    a += text(28, 50, '業務別のAI活用', 24, INK, 700)
    a += text(28, 90, '過去4週間に、次の業務でAIを', 19, DIM)
    a += text(28, 118, 'どの程度使いましたか。', 19, DIM)
    for i, s in enumerate(['0　未利用', '1　試用のみ', '2　実務で1回', '3　実務で複数回']):
        y = 142 + i * 34
        a += rect(28, y, 484, 28, PANEL, 6) + text(44, y + 21, s, 17, INK)
    a += text(28, 308, '実務利用（2・3）の人数の変化を見る', 17, ACCENT, 700)
    a += rect(572, 8, 540, 276, PAPER, 12, LINE)
    a += text(600, 50, '個人目標の達成', 24, INK, 700)
    a += text(600, 92, '開始時', 17, DIM)
    a += rect(600, 106, 484, 62, PANEL, 8)
    a += text(618, 130, 'G4　何が変われば達成ですか。', 19, INK, 700)
    a += text(618, 156, '確認する結果・方法・期限：＿＿＿', 17, DIM)
    a += text(600, 200, '終了時', 17, DIM)
    a += rect(600, 214, 484, 62, PANEL, 8)
    a += text(618, 238, 'R1　目標に対する結果はどうでしたか。', 19, INK, 700)
    a += text(618, 264, '○ 達成　○ 一部達成　○ 未達　○ 未実施', 17, DIM)
    a += text(600, 308, '開始時の条件と、終了時の結果を照合する', 17, DIM, 700)
    return svg('業務別の活用は0から3の4段階で聞き、個人目標は開始時の達成条件と終了時の結果を照合する', a, 320)


def down(x, y1, y2, color=LINE):
    return path(f'M{x} {y1} V{y2} m-7 -7 l7 7 7 -7', color)


def branch_rebuild():
    """1回だけ作る形から、定期的に課題を出し合う形へ。"""
    left = ['作ることが目的になる', '課題は各自で抱える', '使ってもらう相手が決まらない']
    right = ['困りごとを持ち寄り、AIでどう解くかを出し合う', 'その場で、使ってもらう相手と場面を決める', '小さく作り、次の回に結果を持ち寄る']
    a = rect(0, 8, 440, 286, PAPER, 12, LINE)
    a += text(28, 50, '1回だけ作る形', 21, DIM, 700)
    for i, s in enumerate(left):
        y = 74 + i * 74
        a += rect(28, y, 384, 52, PANEL, 8) + text(50, y + 34, s, 19, INK)
        if i < 2:
            a += down(220, y + 54, y + 70)
    a += arrow(468, 150, 544, 150, ACCENT)
    a += rect(572, 8, 540, 286, PAPER, 12, ACCENT)
    a += text(600, 50, '次は、定期的なAIアイデアソン', 21, ACCENT, 700)
    for i, s in enumerate(right):
        y = 74 + i * 74
        a += rect(600, y, 484, 52, PANEL, 8) + text(622, y + 34, s, 18, INK)
        if i < 2:
            a += down(842, y + 54, y + 70)
    return svg('1回だけ作る形から、困りごとを持ち寄って使う相手を決める定期的なアイデアソンへ替える', a, 300)


def branch_stop():
    """3つの原因を1つの判定へ集め、改善できるかで次を決める。"""
    a = text(0, 24, 'どこで止まったかを、3つで切り分ける', 18, DIM)
    for i, s in enumerate(['教材が届かなかった', '時間が取れなかった', '相談が届かなかった']):
        x = i * 388
        a += rect(x, 40, 336, 66, PAPER, 10, LINE) + text(x + 168, 80, s, 20, INK, 700, 'middle')
        a += path(f'M{x + 168} 106 V126')
    a += path('M168 126 H944')
    a += down(556, 126, 144)
    a += rect(396, 146, 320, 54, PANEL, 27) + text(556, 181, '改善できるか', 21, INK, 700, 'middle')
    a += path('M556 200 V220 M270 220 H842')
    a += down(270, 220, 240) + down(842, 220, 240)
    a += rect(0, 242, 540, 88, PANEL, 12)
    a += text(28, 280, '改善できる', 21, INK, 700)
    a += text(28, 312, '条件を変えて、次の期に再実施する', 18, DIM)
    a += rect(572, 242, 540, 88, ACCENT, 12)
    a += text(600, 280, '改善できない', 21, PAPER, 700)
    a += text(600, 312, '次の期で止める。止めた理由を記録に残す', 18, PAPER)
    return svg('教材・時間・相談のどこで止まったかを切り分け、改善できるかの判定で再実施と停止に分かれる', a, 340)


def forms_timeline():
    """3時点のフォームを並べ、回答者名で突き合わせることを示す。"""
    cards = [
        ('10月', '事前', ['利用頻度・能力', '業務別の活用', '困りごと', '個人目標（G）']),
        ('12月', '中間', ['同じ設問', '途中の変化']),
        ('3月', '終了時', ['同じ設問', '作ったもの', '使ってもらった結果（R）']),
    ]
    a = ''
    for i, (when, name, items) in enumerate(cards):
        x = i * 384
        cx = x + 172
        a += text(cx, 26, when, 19, DIM, 400, 'middle')
        a += rect(x, 40, 344, 210, PAPER, 12, LINE)
        a += text(cx, 84, name, 25, INK, 700, 'middle')
        for j, s in enumerate(items):
            y = 122 + j * 32
            a += circle(x + 32, y - 6, 4, LINE, LINE)
            a += text(x + 52, y, s, 18, DIM)
        if i < 2:
            a += arrow(x + 346, 145, x + 382, 145)
    a += path('M172 250 V288 M556 250 V288 M940 250 V288 M172 288 H940', ACCENT, 2.5)
    a += text(556, 320, '回答者名（Formsが自動で記録）で、同じ人の回答を突き合わせる', 20, ACCENT, 700, 'middle')
    return svg('事前10月・中間12月・終了時3月の3本のフォームを、回答者名で突き合わせる', a, 332)


def sharing_two():
    """12月の中間共有と、3月の最終共有会。目的が違うことを示す。"""
    a = rect(0, 8, 540, 282, PAPER, 12, LINE)
    a += text(28, 48, '12月', 18, DIM)
    a += text(28, 88, '中間共有', 26, INK, 700)
    a += text(28, 120, 'TeamsのWeb会議｜途中経過を共有する', 18, DIM)
    a += icon(452, 52, 'screen', INK, .9)
    for j, s in enumerate(['画面共有で、途中の成果物を実演する', '判断に迷う点を提示し、助言を受ける', '発表後に、中間アンケートへ回答する']):
        y = 174 + j * 40
        a += circle(32, y - 6, 4, LINE, LINE) + text(54, y, s, 19, INK)
    a += arrow(566, 150, 606, 150)
    a += rect(632, 8, 480, 282, PAPER, 12, ACCENT)
    a += text(660, 48, '3月', 18, DIM)
    a += text(660, 88, '最終共有会', 26, ACCENT, 700)
    a += text(660, 120, 'オフライン交流会｜成果物を公開する', 18, DIM)
    a += icon(1030, 54, 'chat', INK, .9)
    for j, s in enumerate(['短いデモで、取り組みを紹介する', '軽食を囲んで交流する', '希望者へ共有し、後日の試用へつなぐ']):
        y = 174 + j * 40
        a += circle(664, y - 6, 4, ACCENT, ACCENT) + text(686, y, s, 19, INK)
    return svg('12月はTeamsのWeb会議で途中経過を共有し、3月はオフライン交流会で成果物を公開する', a, 300)


def next_period():
    """3月の結果から、次にすることが3つに分かれる。"""
    cards = [
        (['他の人に使われた仕組みが', '複数出た'], '広げる', ['共有リポジトリへ集約し、', '次の期は対象を広げる'], True),
        (['作れたが、', '使われなかった'], '発想を鍛える', ['定期的なAIアイデアソンを', '組み込む'], False),
        (['作るところまで', '到達しなかった'], '支援を補う', ['教材・時間・相談の不足を', '特定し、条件を変える'], False),
    ]
    a = rect(396, 8, 320, 54, PANEL, 27) + text(556, 44, '3月の結果', 22, INK, 700, 'middle')
    a += path('M556 62 V88 M172 88 H940')
    for i, (head, verb, detail, hot) in enumerate(cards):
        x = i * 384
        a += down(x + 172, 88, 114)
        a += rect(x, 118, 344, 184, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 24, 156, head, 17, DIM, gap=24)
        a += path(f'M{x + 24} 200 H{x + 320}')
        a += text(x + 24, 238, verb, 24, ACCENT if hot else INK, 700)
        a += text(x + 24, 272, detail, 16, DIM, gap=22)
    a += text(556, 328, '次の3枚で、それぞれの中身を示す', 17, DIM, 400, 'middle')
    return svg('3月の結果から、広げる・発想を鍛える・支援を補うの3つに分かれる', a, 336)
