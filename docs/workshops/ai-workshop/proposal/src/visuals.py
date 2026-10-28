"""Semantic SVG diagrams for the workshop slides; no external assets."""
from html import escape

from theme import T  # 配色の正本は slide-deck の warm-paper。ここに色を書かない

INK = T['ink']
DIM = T['dim']
LINE = T['diagram-line']  # 図の線は、本文の文字色とは別のキーから取る
PANEL = T['surface']
PAPER = T['paper']
ACCENT = T['accent']

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

def support_proposal():
    """企画書の支援の図。オリエンテーションが support() を読み込んで置き換えているため、support() は変えず、企画書用はここで組む（オリエンテーション13枚目と同じ形）。"""
    figure = (support().replace('教材動画と成果物で学ぶ', '教材動画・解説記事・事例を見る')
              .replace('Teamsで相談する', 'Teamsで質問・相談')
              .replace('いつでも投稿 ＋ スポット相談会', '投稿して質問 ＋ 必要に応じて相談会'))
    # 人物を背景へ直接置かず、枠で囲む。矢印は枠の上端と下端につなぐ
    frames = rect(20, 88, 150, 172, PAPER, 12, LINE) + rect(938, 88, 150, 172, PAPER, 12, LINE)
    figure = figure.replace('>', '>' + frames, 1)
    for before, after in [('d="M1013 100 V65 H778"', 'd="M1013 88 V65 H778"'),
                          ('d="M336 65 H95 V100 M88 92 L95 100 L102 92"', 'd="M336 65 H95 V88 M88 80 L95 88 L102 80"'),
                          ('d="M95 250 V285 H336"', 'd="M95 260 V285 H336"'),
                          ('d="M778 285 H1013 V250 M1006 258 L1013 250 L1020 258"', 'd="M778 285 H1013 V260 M1006 268 L1013 260 L1020 268"')]:
        figure = figure.replace(before, after)
    footer = rect(0, 356, 1112, 46, PAPER, 12, LINE)
    footer += icon(20, 367, 'screen', INK, .5)
    footer += text(64, 386, '参加者（15〜20名）には、KiroのアカウントとAWS環境を発行する（ポータルのアカウントとは別）', 17, INK, 700)
    # 相談には運営から回答が返るため、参加者側にも矢じりを付けて双方向にする
    footer += path('M88 268 L95 260 L102 268', ACCENT, 3)
    return figure.replace('1112 350', '1112 406').replace('</svg>', footer + '</svg>')


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
    a += person(712, 66, .45, ACCENT) + person(690, 66, .45, ACCENT) + person(668, 66, .45, ACCENT)
    a += text(22, 94, '目的と要件の明確化率', 20, INK, 700) + text(22, 118, '目的と要件を以前より明確にできると感じた人', 13, DIM)
    a += path('M374 68 V124', LINE, 1)
    a += text(398, 94, '改善の達成率', 20, INK, 700) + text(398, 118, '参加時に決めた改善を、1つ以上達成した人', 13, DIM)
    # 誰が使ったかで分ける
    a += path('M374 136 V160 M182 160 H566 M182 160 V172 M566 160 V172', LINE, 2)
    a += path('M175 165 L182 172 L189 165 M559 165 L566 172 L573 165', LINE, 2)
    a += text(386, 153, '作ったSkillsを誰が使ったかで分ける', 13, DIM)
    lanes = [(0, '補助 ・ 自分で使った人', '作ったSkillsを自分だけで使った',
              [('実務でAIを使う人の増加', '月に1回以上、実務でAIを使う人の増え方'), ('確認 ・ 修正の手間の変化', '手間が減った人の割合と、その記録')]),
             (384, '補助 ・ チームで使った人', '同じチームの人にも使ってもらった',
              [('チームでの改善の実施', '振り返りの場を1回以上開いた人の割合'), ('やり方のチームへの定着', 'Skillsの共有が進み、立ち上げが短くなった人の割合')])]
    for x, head, who, items in lanes:
        a += rect(x, 176, 364, 176, PAPER, 12, LINE)
        # 誰が使ったかを人物の数で示す
        a += person(x + 322, 210, .55) if x == 0 else person(x + 282, 210, .5) + person(x + 308, 210, .5) + person(x + 334, 210, .5)
        a += text(x + 22, 202, head, 15, INK, 700) + text(x + 22, 222, who, 13, DIM)
        for k, (name, judge) in enumerate(items):
            y = 260 + k * 50
            a += text(x + 22, y, name, 16, INK, 700) + text(x + 22, y + 20, judge, 13, DIM)
    # 基盤づくりの検証
    box = rect(780, 32, 332, 320, PAPER, 12, LINE)
    a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 6"')
    a += icon(1062, 46, 'search', DIM, .42)
    checks = [('チームでの改善の継続', '終了後も振り返りを続ける人の割合'), ('参加者以外での再利用', '業務で試した、参加しなかった希望者'),
              ('必要なナレッジの見つけやすさ', '探していたSkillsにたどり着けた割合'), ('ナレッジを出す人の広がり', 'ノウハウを載せた、参加しなかった希望者')]
    for k, (name, judge) in enumerate(checks):
        y = 66 + k * 62
        a += text(802, y, name, 16, INK, 700) + text(802, y + 20, judge, 13, DIM)
    a += path('M802 304 H1090', LINE, 1)
    a += text(802, 332, '次の期に何を広げるかの判断に使う', 13, DIM, 700)
    return svg('効果測定の全体像。ワークショップの効果は、参加者全員に共通の主KPI2つ（目的と要件の明確化率、改善の達成率）と、作ったSkillsを誰が使ったかで分けた補助（自分だけで使った人：実務でAIを使う人の増加と確認・修正の手間の変化、チームで使った人：振り返りとやり方の定着）で測る。基盤づくりは先行トライアルとして、チームでの継続 ・ 広がり ・ たどり着きやすさ ・ ナレッジを出した割合で検証する', a, 358)

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
    """三つの教材で身につけること。前の教材で考えたことを次の教材の前提にし、合わなければ前に戻って見直す。"""
    a = ''
    cards = [('教材1　目的と要件の明確化', '力①', ['何のために、どこまでを', '実現するかを明確にする'],
              ['作業の目的から、', '実現する範囲と満たす要件を考える。', '目的を広げすぎず、', '具体的な要件と作業に落とし込む']),
             ('教材2　業務のモデル化と設計', '力②', ['業務をモデルに表し、', '設計へつなげる'],
              ['業務で扱う対象 ・ 判断 ・ 関係と、', '守るべき条件をモデルに表す。', '業務用語の意味も、', 'モデル化のために確認する']),
             ('教材3　AI活用の実践と改善', '力③④', ['考え方をSkillsにまとめ、', 'Agentsで実践し、改善する'],
              ['判断の観点と確認の手順を', 'Skillsにまとめる。作業結果を', '要件や設計と照合し、', '人が確認して改善する'])]
    marks = ['doc', 'book', 'robot']
    for i, (lesson, ref, name, lines) in enumerate(cards):
        x = i * 384
        a += rect(x, 8, 344, 300, PAPER, 12, LINE)
        # 教材ごとの図：要件の書類、業務のモデル、AIの作業
        a += rect(x + 20, 24, 64, 64, PANEL, 10)
        if marks[i] == 'robot':
            a += f'<g transform="translate({x + 28} {34}) scale(.95)">' + _robot_at(0, 0, INK) + '</g>'
        elif marks[i] == 'book':
            a += circle(x + 36, 44, 7, PAPER, INK) + circle(x + 68, 44, 7, PAPER, INK) + circle(x + 52, 72, 7, PAPER, INK)
            a += path(f'M{x + 43} 44 H{x + 61} M{x + 40} 50 L{x + 48} 66 M{x + 64} 50 L{x + 56} 66', INK, 1.8)
        else:
            a += icon(x + 34, 30, 'doc', INK, .8)
        a += text(x + 98, 44, lesson.split('　')[0], 14, DIM, 700) + text(x + 98, 68, lesson.split('　')[1], 15, INK, 700)
        a += text(x + 320, 44, ref, 12, DIM, anchor='end')
        a += text(x + 24, 124, name, 20, INK, 700, gap=28)
        a += path(f'M{x+24} 168 H{x+320}', LINE, 1)
        a += text(x + 24, 192, '学ぶこと', 13, DIM, 700)
        for k, s_ in enumerate(lines):
            a += text(x + 24, 218 + k * 22, s_, 15, INK)
        if i < 2:
            a += arrow(x + 350, 158, x + 378, 158)
    a += rect(0, 324, 1112, 48, PAPER, 12, ACCENT)
    a += text(556, 354, '前の教材で考えたことを、次の教材の前提にする。合わなければ、前の教材に戻って見直す', 17, ACCENT, 700, 'middle')
    return svg('三つの教材で身につけること。教材1は目的と要件の明確化で、何のために、どこまでを実現するかを明確にする。教材2は業務のモデル化と設計で、業務をモデルに表し、設計へつなげる。教材3はAI活用の実践と改善で、考え方をSkillsにまとめ、Agentsで実践し、改善する。前の教材で考えたことを次の教材の前提にし、合わなければ前に戻って見直す', a, 376)

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
    """10〜12月は希望者が教材を見て、改善したい業務と何が変われば達成かを考える準備期間。12月に受け付けた先着15〜20名が、1月にSkillsを作り、2月に業務で使って確認し直し、3月に何が変わったかを共有する。アンケートは事前・参加時・終了時。"""
    cols = [('10〜12月（希望者）', '準備する', [['事前アンケートを取り、', 'アカウントを発行する'], ['教材1を公開し、', '11月から教材2・3を公開する'], ['改善したい業務と、何が', '変われば達成かを考える'], ['12月に参加を受け付け、', '先着15〜20名を決める']], False, True),
            ('1月', 'Skillsを作る', [['AWS環境とKiroを発行する'], ['目的と要件、業務の', 'モデルを整理する'], ['判断の観点と手順を', 'Skillsにまとめ、小さく試す']], False, False),
            ('2月', '業務で使って直す', [['Skillsを実際の業務で使う'], ['目的と要件に合うか、', '確認・修正の手間がどう', '変わったかを確認する'], ['合わない所は、Skillsの', '観点と手順を直す'], ['チームでは振り返りの場を開く']], False, False),
            ('3月', '成果を共有する', [['Skillsと記録を、', 'ポータルに載せる'], ['最終共有会で、何が', '変わったかを振り返る']], True, False)]
    a = ''
    for k, (month, head, items, hot, dash) in enumerate(cols):
        x = k * 286
        if k:
            a += arrow(x - 24, 160, x - 6, 160)
        box = rect(x, 8, 254, 304, PAPER, 12, ACCENT if hot else LINE)
        a += box.replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="6 6"') if dash else box
        a += text(x + 20, 38, month, 15, ACCENT if hot else DIM, 700) + text(x + 20, 70, head, 21, ACCENT if hot else INK, 700)
        y = 108
        for lines in items:
            a += path(f'M{x+22} {y-6} h8', ACCENT if hot else LINE, 3)
            a += text(x + 38, y, lines, 15, INK, gap=22)
            y += 22 * len(lines) + 12
    a += text(0, 342, 'アンケート', 15, DIM, 700)
    a += path('M96 336 H1112', LINE, 2)
    for x, label, anc in ((128, '事前（10月 ・ 動画の公開前）', 'start'), (560, '参加時（12月 ・ 教材の後）', 'end'), (1098, '終了時（3月）', 'end')):
        a += circle(x, 336, 6, PAPER, INK)
        a += text(x - 8 if anc == 'start' else x, 364, label, 14, DIM, anchor=anc)
    return svg('10〜12月は準備期間とし、事前アンケートで希望者にポータルのアカウントを発行し、教材を順に公開する。希望者は改善したい業務と何が変われば達成かを考え、12月に受け付けた先着15〜20名が参加する。1月にSkillsを作り、2月に業務で使って目的と要件に合うか、確認・修正の手間がどう変わったかを確認して直し、3月にSkillsと記録をポータルに載せ、最終共有会で何が変わったかを振り返る。アンケートは事前、参加時、終了時', a, 372)

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
    a += text(196, 272, ['前の教材を、次の教材の前提にする', '合わなければ、前に戻って直す'], 13, DIM, gap=19)
    a += path('M392 246 C 404 226, 404 168, 392 150 M386 160 L392 150 L400 157', LINE, 1.5)
    # 成果物：参加者が作った仕組み
    a += rect(428, 64, 240, 240, PAPER, 10, LINE) + icon(448, 76, 'doc', INK, .55)
    a += text(488, 106, '成果物', 19, INK, 700)
    a += text(448, 150, '参加者が作った仕組み', 17, INK)
    for k, t in enumerate(('目的', '使い方', '試した記録', '言葉の定義')):
        a += icon(452, 174 + k * 30, 'check', ACCENT, .4) + text(478, 188 + k * 30, t, 16, DIM)
    # カテゴリとタグで探す：強調。目的そのもので分類せず、目的にたどり着きやすい分類にする
    a += rect(680, 64, 244, 240, PAPER, 10, ACCENT) + icon(700, 80, 'search', ACCENT, .5)
    a += text(736, 105, 'カテゴリとタグで探す', 17, ACCENT, 700)
    a += rect(694, 132, 88, 34, PAPER, 17, LINE) + text(738, 155, '顧客対応', 13, INK, anchor='middle')
    a += rect(788, 132, 82, 34, PAPER, 17, LINE) + text(829, 155, '問い合わせ', 13, INK, anchor='middle')
    a += rect(876, 132, 42, 34, PAPER, 17, LINE) + text(897, 155, '集計', 13, INK, anchor='middle')
    a += path('M802 178 V198 M795 191 L802 198 L809 191', ACCENT, 2)
    a += rect(700, 206, 204, 44, ACCENT, 8) + text(802, 234, '目的に合う仕組み', 16, PAPER, 700, 'middle')
    a += text(700, 282, '自分の業務で試し、参考にする', 13, DIM)
    a += arrow(952, 140, 986, 140)
    for k, x in enumerate((1016, 1076)):
        a += person(x, 128, .8)
    a += text(1046, 196, '希望した人', 17, INK, 700, 'middle')
    a += text(1046, 228, ['探して', '社内で使う'], 14, DIM, anchor='middle', gap=22)
    return svg('教材は社内のナレッジ共有ポータルで提供する。参加者は教材動画で学び、作った仕組みを成果物として目的と一緒に載せる。希望した人は、カテゴリとタグから仕組みを探して使う。今期はトライアルとする', a, 330)


def portal_proposal():
    """企画書のポータルの図。オリエンテーションが portal() を読み込んで置き換えているため、portal() は変えず、企画書用はここで組む。
    教材名と参加者が載せる項目をオリエンテーション11枚目と同じにし、人物は枠で囲む。"""
    figure = (portal().replace('参加者が作った仕組み', '参加者のSkillsと記録')
              .replace('目的に合う仕組み', '業務に合うSkills')
              .replace('>成果物</text>', '>参加者の実践</text>', 1)
              .replace('合わなければ、前に戻って直す', '合わなければ、前の教材に戻る'))
    for before, after in [('目的の把握', '目的と要件の明確化'), ('言葉の定義', '業務のモデル化と設計'), ('仕組みの構築', 'AI活用の実践と改善')]:
        figure = figure.replace(f'font-size="17" fill="{INK}" font-weight="400" text-anchor="start">{before}</text>',
                                f'font-size="14" fill="{INK}" font-weight="400" text-anchor="start">{after}</text>', 1)
    figure = figure.replace('<text x="252" ', '<text x="246" ')
    figure = figure.replace('d="M392 246 C 404 226, 404 168, 392 150 M386 160 L392 150 L400 157"', 'd="M398 246 C 410 226, 410 168, 398 150 M392 160 L398 150 L406 157"')
    for before, after in [('>目的</text>', '>目的と要件</text>'), ('>使い方</text>', '>判断の観点と手順</text>'), ('>試した記録</text>', '>使い方</text>'), ('>言葉の定義</text>', '>試した結果と改善</text>')]:
        figure = figure.replace(before, after, 1)
    for old, new_ in [(rect(694, 132, 88, 34, PAPER, 17, LINE), rect(694, 132, 72, 34, PAPER, 17, LINE)),
                      (rect(788, 132, 82, 34, PAPER, 17, LINE), rect(772, 132, 72, 34, PAPER, 17, LINE)),
                      (rect(876, 132, 42, 34, PAPER, 17, LINE), rect(850, 132, 72, 34, PAPER, 17, LINE)),
                      (text(738, 155, '顧客対応', 13, INK, anchor='middle'), text(730, 155, '開発', 13, INK, anchor='middle')),
                      (text(829, 155, '問い合わせ', 13, INK, anchor='middle'), text(808, 155, 'レビュー', 13, INK, anchor='middle')),
                      (text(897, 155, '集計', 13, INK, anchor='middle'), text(886, 155, 'テスト', 13, INK, anchor='middle'))]:
        figure = figure.replace(old, new_)
    # ポータルの外枠は、中の箱と同じ地にしない
    figure = figure.replace(rect(156, 8, 788, 316, PAPER, 14, LINE), rect(156, 8, 788, 316, PANEL, 14, LINE), 1)
    figure = figure.replace('>成果物を載せる</text>', '>Skillsを載せる</text>', 1)
    # 人物と説明を一つの対象として枠で囲む
    frames = rect(0, 60, 114, 240, PAPER, 12, LINE) + rect(988, 60, 124, 240, PAPER, 12, LINE)
    figure = figure.replace('>', '>' + frames, 1)
    return figure.replace('aria-label="教材は社内のナレッジ共有ポータルで提供する。参加者は教材動画で学び、作った仕組みを成果物として目的と一緒に載せる。希望した人は、カテゴリとタグから仕組みを探して使う。今期はトライアルとする"',
                          'aria-label="教材は社内のナレッジ共有ポータルで提供する。参加者は教材動画で学び、作ったSkillsを、目的と要件・判断の観点と手順・使い方・試した結果と改善と一緒に載せる。希望した人は、カテゴリとタグから業務に合うSkillsを探して使う。今期はトライアルとする"')


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
    """AIツールは手段である。成果の差を生むのは、目的と要件を明確にし、業務をモデルに表して設計へつなぎ、判断の観点と手順にまとめて、AIの作業と人の確認で使えるかである。"""
    a = rect(0, 8, 520, 240, PAPER, 12, LINE)
    a += text(28, 50, 'AIツールは手段', 22, DIM, 700)
    a += text(28, 100, 'Copilot ・ ChatGPT ・ Claude', 20, INK, 700)
    a += text(28, 136, '開発では Kiro ・ Claude Code ・ GitHub Copilot ・ Codex', 16, INK)
    a += text(28, 200, ['操作に詳しくても、', '成果には差が出る'], 17, DIM, gap=26)
    a += arrow(546, 128, 610, 128)
    a += rect(636, 8, 476, 240, PAPER, 12, ACCENT)
    a += text(664, 50, '身につけたい力', 22, ACCENT, 700)
    a += text(664, 94, ['目的と要件を明確にし、', '業務をモデルに表して設計へつなぎ、', '判断の観点と手順にまとめて、', 'AIの作業と人の確認で使う'], 19, INK, 700, gap=29)
    a += text(664, 230, 'どのツールを使っても発揮できる', 16, DIM)
    return svg('AIツールは手段で、操作に詳しくても成果には差が出る。身につけたいのは、目的と要件を明確にし、業務をモデルに表して設計へつなぎ、判断の観点と手順にまとめて、AIの作業と人の確認で使う力で、どのツールでも発揮できる', a, 256)


def example_case():
    """共通の「はじめに」の架空案件で起きた3つのことを左に、それぞれを防ぐ身につけたい力を右に、行をそろえて対応させる。"""
    a = text(0, 18, 'この案件で起きたこと（架空の事例）', 15, DIM, 700) + text(600, 18, '目的に掲げた4つの力との対応', 15, ACCENT, 700)
    rows = [('要件の解釈がずれた', '「承認後は変更不可」を、経理確認後と理解した',
             '① 目的と要件を明確にする', 'どの承認段階を指すかを確認し、テストすべき状態を決める'),
            ('動作を直しても、設計の問題が残った', '上長の承認と経理の確認を、一つの状態で扱った',
             '② 業務をモデルに表し、設計へつなぐ', '二つの判断を分けてモデル化し、変更の影響を把握しやすくする'),
            ('テストとAIレビューも同じ解釈で進んだ', '上長承認後・経理確認前の状態のテストが抜けた',
             '③④ Skillsで照合し、人が確認する', '要件とテストの対応を、AIと人のレビューで照合する')]
    for k, (lh, ls, rh, rs) in enumerate(rows):
        y = 32 + k * 84
        a += rect(0, y, 520, 72, PAPER, 12, LINE)
        a += text(22, y + 30, lh, 18, INK, 700) + text(22, y + 56, ls, 15, DIM)
        a += arrow(530, y + 36, 588, y + 36)
        a += rect(600, y, 512, 72, PAPER, 12, ACCENT)
        a += text(622, y + 30, rh, 18, INK, 700) + text(622, y + 56, rs, 15, DIM)
    a += rect(0, 290, 520, 50, PANEL, 12)
    a += text(260, 321, 'その結果、修正 ・ 再テスト ・ 再レビューが必要になった', 15, INK, 700, 'middle')
    a += rect(600, 290, 512, 50, PANEL, 12)
    a += text(856, 321, '必ず防げるわけではないが、見落としを前の工程で発見しやすくなる', 15, INK, 700, 'middle')
    return svg('経費精算システム刷新の架空の案件で、要件の解釈がずれた、動作を直しても設計の問題が残った、テストとAIレビューも同じ解釈で進んだ、という3つが起き、修正・再テスト・再レビューが必要になった。それぞれ、目的と要件を明確にする、業務をモデルに表し設計へつなぐ、Skillsで照合し人が確かめる、という力で、見落としを前の工程で見つけやすくする', a, 346)


def kpi_main():
    """主KPIの2つを、人の並びで割合を示し、いつ・何を聞くか・判定のルールを3行で添える。どちらもアンケートの回答で数える。"""
    panels = [('目的と要件の明確化率', '以前より明確にできると感じたか',
               '「とてもそう思う」「そう思う」と答えた人', '参加時アンケートに回答した人',
               [('いつ', ['参加時アンケート（教材の視聴後）']),
                ('聞くこと', ['目的と要件を明確にできるように', 'なったと感じるか（5段階）']),
                ('ルール', ['書いた内容の良し悪しは、', '運営が判定しない'])]),
              ('改善の達成率', '決めた改善を達成できたか',
               '決めた改善を1つ以上達成した人', '参加した人（15〜20名。開始時に固定）',
               [('いつ', ['終了時アンケート（根拠と記録を添える）']),
                ('聞くこと', ['「何が変われば達成か」（工数 ・ 期間 ・', '品質）に照らして、達成したか']),
                ('ルール', ['回答しなかった人も分母に含める。', '達成した改善の数は別に添える'])])]
    a = ''
    for k, (name, q, num, den, rows) in enumerate(panels):
        x = k * 572
        a += rect(x, 0, 540, 420, PAPER, 12, ACCENT)
        a += text(x + 26, 38, name, 22, ACCENT, 700) + text(x + 26, 64, q, 15, DIM)
        # 割合を人の並びで示す：色の付いた人が分子、全員が分母
        a += text(x + 26, 100, '分子', 13, ACCENT, 700) + text(x + 70, 100, num, 15, ACCENT, 700)
        for m in range(10):
            a += person(x + 52 + m * 48, 140, .55, ACCENT if m < 6 else LINE)
        a += path(f'M{x + 26} 170 H{x + 514}', LINE, 1.5)
        a += text(x + 26, 194, '分母', 13, DIM, 700) + text(x + 70, 194, den, 15, INK, 700) + text(x + 514, 194, '人数は例', 11, DIM, anchor='end')
        # いつ・何を聞くか・ルール
        y = 212
        for label, lines in rows:
            h = 22 * len(lines) + 18
            a += path(f'M{x + 26} {y} H{x + 514}', LINE, 1)
            a += text(x + 26, y + 26, label, 13, DIM, 700) + text(x + 110, y + 26, lines, 15, INK, gap=22)
            y += h + 8
    return svg('主KPIは2つ。目的と要件の明確化率は、参加時アンケートに回答した人のうち、教材を見る前と比べて目的と要件を明確にできるようになったと「とてもそう思う」「そう思う」と答えた人の割合。改善の達成率は、ワークショップに参加した人のうち、参加時に決めた改善を、何が変われば達成かに照らして達成したと、終了時アンケートで根拠と記録を添えて答えた人の割合', a, 424)

def kpi_sub():
    """補助の指標4つを、それぞれの設問と選択肢と並べる。"""
    cards = [('実務でAIを使う人の増加', '過去4週間に、実務でAIをどのくらい使いましたか', '使っていない ／ 試しただけ ／ 月に1〜3回 ／ 週に1回以上 ／ ほぼ毎日', '自分だけで使った人で、月に1回以上の人数の変化'),
             ('チームでの改善の実施', '作った仕組みについて、振り返りの場を何回開きましたか', '0回 ／ 1回 ／ 2回以上（話し合った問題と直したこと）', 'チームで使った人で、1回以上開いた割合'),
             ('手戻りの減少', 'AIが出した結果に、不満やストレスを感じたことはありましたか', 'ほぼ毎回 ／ たびたび ／ ときどき ／ ほとんど感じなかった', '自分だけで使った人で、ほぼ毎回とたびたびの人の減り方'),
             ('やり方のチームへの定着', '指示の型やSkillをチームで共有しているか。新規参画者の立ち上げ期間', '共有の4つの状態 ／ 数日以内 〜 1か月より長い', 'チームで使った人で、上がった ・ 短くなった割合')]
    a = text(0, 18, '自分だけで使った人', 16, INK, 700) + text(572, 18, 'チームで使った人', 16, INK, 700)
    for k, (name, q, choice, look) in enumerate(cards):
        x = (k % 2) * 572
        y = 34 + (k // 2) * 160
        a += rect(x, y, 540, 152, PAPER, 12, LINE)
        a += text(x + 24, y + 34, name, 18, INK, 700)
        a += text(x + 24, y + 68, q, 15, INK)
        a += text(x + 24, y + 98, choice, 14, DIM)
        a += text(x + 24, y + 132, look, 15, ACCENT, 700)
    return svg('補助の指標4つ。左が自分だけで使った人、右がチームで使った人で、それぞれの設問と選択肢で取る', a, 348)

def keep_loop():
    """AI活用を促進する基盤づくり。なぜ必要か（左）と、運営チーム → メンバー → ナレッジ共有ポータル → 運営チームと回る運用の流れ（右）。"""
    import math
    a = text(0, 16, 'なぜ必要か', 14, DIM, 700) + text(480, 16, '運用の流れ', 14, DIM, 700)
    whys = [('branch', 'ほかの人も探して使えるように', '試した結果を添えて、活用事例とSkillsを集める'),
            ('calendar', 'AIの進化は速い', '新しい機能や使い方が次々に出る'),
            ('screen', '使い方を知るほど、改善の幅が広がる', 'AIで改善できる業務が増える'),
            ('search', 'ただ、自分から追わないと分からない', '追う人だけが、新しい使い方を知る')]
    for k, (ic, h1, h2) in enumerate(whys):
        y = 30 + k * 86
        hot = k == 0
        a += rect(0, y, 404, 76, PAPER, 12, ACCENT if hot else LINE)
        a += icon(20, y + 18, ic, ACCENT if hot else INK, .7)
        a += text(82, y + 32, h1, 17, ACCENT if hot else INK, 700)
        a += text(82, y + 58, h2, 14, DIM)
    a += text(442, 198, 'だから', 14, DIM, 700, 'middle') + arrow(418, 212, 466, 212)
    # 輪：楕円の上に3つの箱を置く。弧の両端は、楕円が箱を出る角度を計算して決める
    cx, cy, rx, ry = 796, 212, 216, 128
    bw, bh = 232, 92
    nodes = [(-90, '運営チーム', ['運用ルールと基盤を整え、', '触れる機会を作る']),
             (30, 'メンバー（参加者を含む）', ['機会とナレッジを使って、', '新しい事例を作る']),
             (150, 'ナレッジ共有ポータル', ['事例 ・ ノウハウ ・ Skillsを', '集め、次の活用に使う'])]
    P = lambda t: (cx + rx * math.cos(math.radians(t)), cy + ry * math.sin(math.radians(t)))
    def leave(ang, step):
        x0, y0 = P(ang); t = ang
        while True:
            t += step; x, y = P(t)
            if abs(x - x0) > bw / 2 + 10 or abs(y - y0) > bh / 2 + 10: return t
    labels = [('機会を提供する', 'start', 14, 0), ('事例とSkillsを登録し、更新する', 'middle', 0, 28), ('ナレッジで改善する', 'end', -14, 0)]
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
    return svg('作った仕組みをほかの人も探せるよう、目的と言葉の意味を添えて仕組みを集める。AIの進化は速く、仕組みを知るほど目的の幅が広がるが、自分から追わないと分からない。だから、運営チームが機会を提供し、メンバーが事例を作ってポータルに登録し、言葉の定義も更新し、集めたナレッジで運営チームが改善する流れを回し、継続的なAI活用を組織に定着させる', a, 380)

def ops_cards():
    """運営チームの役割。運営は環境を整え、活用を促す取り組みと情報を発信する。利用者はそれを受けて試し、事例とノウハウを共有する。共有を受けて運営が次の発信をし、往復で循環させる。"""
    cards = [('screen', '環境と運用ルールを整える', ['AWS環境と開発環境、目的を限定した', 'ナレッジ共有ポータルを構築し、', '載せる項目と運用ルールを決める']),
             ('branch', 'AI活用を促す取り組みを設計する', ['事例やノウハウが集まり、', '使われ続ける運用を設計し、', '参加者と希望者で試す']),
             ('chat', '情報と機会を発信する', ['新しい使い方や事例を発信し、', 'アイデアソンやワークショップを', '定期的に開く'])]
    a = ''
    for k, (ic, head, body) in enumerate(cards):
        x = k * 380
        a += rect(x, 0, 352, 236, PAPER, 12, LINE)
        a += icon(x + 26, 22, ic, INK, .8)
        a += text(x + 26, 108, head, 20, INK, 700)
        a += path(f'M{x + 26} 126 H{x + 326}', LINE, 1)
        a += text(x + 26, 160, body, 16, DIM, gap=26)
    # 運営と利用者の往復：どちらも受け身にならず、発信と共有で循環する
    a += rect(0, 258, 440, 104, PAPER, 12, LINE)
    a += person(38, 316, .55) + person(64, 316, .55) + person(90, 316, .55)
    a += text(128, 296, '運営チーム（約3名）', 14, DIM, 700) + text(128, 328, ['環境を整え、取り組みと', '情報を発信する'], 17, INK, 700, gap=24)
    a += rect(672, 258, 440, 104, PAPER, 12, ACCENT)
    a += person(712, 316, .6, ACCENT) + person(744, 316, .6, ACCENT)
    a += text(784, 296, '利用者（参加者を含むメンバー）', 14, ACCENT, 700) + text(784, 328, ['受け取って試し、事例と', 'ノウハウを共有する'], 17, ACCENT, 700, gap=24)
    a += path('M450 290 H662 m-8 -7 l8 7 -8 7', ACCENT, 2.5) + text(556, 280, '発信する', 14, ACCENT, 700, 'middle')
    a += path('M662 332 H450 m8 -7 l-8 7 8 7', ACCENT, 2.5) + text(556, 356, '共有する', 14, ACCENT, 700, 'middle')
    return svg('運営チームは、環境と運用ルールを整え、活用を促す取り組みを設計し、情報と機会を発信する。利用者はそれを受け取って試し、事例とノウハウを共有する。運営は共有を受けて次の発信をし、運営と利用者が往復して循環する', a, 366)


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
    for k, lab in enumerate(['目的と要件', '判断の観点と手順', '使い方', '試した結果']):
        y = 128 + k * 21
        a += text(680, y, lab, 12, INK, 700) + path(f'M792 {y + 3} H836', LINE, 1.5)
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
    a += text(556, 354, '教材1と同じく、何のためのポータルかを先に決め、載せる項目と探し方を決めておく', 17, INK, 700, 'middle')
    return svg('汎用のサービスでは、使い方が人に委ねられ、置き方も探し方もばらばらになる。目的を限定したこのポータルでは、載せる項目が決まっていて、カテゴリとタグで迷わず探せる', a, 378)

def closing():
    """まとめ。ワークショップで、目的をAIで達成できる人を増やし、基盤づくりで、そのやり方が部内で続く形を作る。2つを合わせて、目的をAIで達成するやり方を、人が替わっても続く形で部内に定着させる。"""
    a = ''
    a += rect(0, 8, 330, 250, PAPER, 12, LINE)
    for k in range(3):
        a += person(120 + k * 45, 70, .7)
    a += text(165, 146, 'ワークショップ', 24, INK, 700, 'middle')
    a += text(165, 190, ['AIを使って、自分の業務を', '改善できる人を増やす'], 17, DIM, anchor='middle', gap=26)
    a += text(365, 146, '＋', 40, DIM, 700, 'middle')
    a += rect(400, 8, 330, 250, PAPER, 12, LINE)
    a += icon(538, 44, 'search', INK, .9)
    a += text(565, 146, 'AI活用を促進する', 22, INK, 700, 'middle') + text(565, 176, '基盤づくり', 22, INK, 700, 'middle')
    a += text(565, 214, ['発信と共有で、AI活用が', '部内で循環する形を作る'], 17, DIM, anchor='middle', gap=26)
    a += arrow(740, 133, 772, 133, ACCENT)
    a += rect(782, 8, 330, 250, PAPER, 12, ACCENT)
    a += text(947, 60, '目指す先', 16, ACCENT, 700, 'middle')
    a += text(947, 104, ['AIの活用を、', '仕事全体の生産性と', '品質の改善につなげる'], 22, ACCENT, 700, 'middle', gap=36)
    a += text(0, 300, 'うまくいかなかった改善も、理由と試した記録がポータルに残り、次の期の判断に使える', 16, DIM)
    a += text(0, 336, '次の一歩：本日ご了承いただければ、今月中に希望者へ事前アンケートを取り、ポータルのアカウントを発行する', 17, INK, 700)
    return svg('ワークショップでAIを使って自分の業務を改善できる人を増やし、基盤づくりで発信と共有によりAI活用が部内で循環する形を作る。2つを合わせて、AIの活用を、仕事全体の生産性と品質の改善につなげる。次の一歩は、了承を得たら今月中に希望者へ事前アンケートを取り、ポータルのアカウントを発行すること', a, 348)

def part2_open():
    """目指す姿。基盤を今期に先行トライアルで試し、次の期に部内へ展開する。定着すると、機会とナレッジが部内で回り、目的をAIで達成するやり方が、人が替わっても続く。"""
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
            ('search', '必要な事例やナレッジを、探して使える', 'ナレッジ共有ポータルに、事例 ・ ノウハウ ・ Skillsが集まっている'),
            ('branch', '事例やSkillsが使われ続け、改善される', '使った人が、試した結果をもとに事例とSkillsを更新し続ける')]
    for k, (ic, head, sub) in enumerate(rows):
        y = 64 + k * 76
        a += icon(448, y + 6, ic, INK, .7)
        a += text(510, y + 24, head, 18, INK, 700) + text(510, y + 50, sub, 14, DIM)
    a += path('M448 300 H1084', LINE, 1)
    a += text(448, 338, '担当者が替わっても、AIを使った改善のやり方が部内で続く', 19, ACCENT, 700)
    return svg('基盤を今期に先行トライアルで試し、次の期に部内へ展開する。部内に定着すると、全員が新しい使い方に触れ、必要な事例やナレッジを探して使え、事例やSkillsが使われ続けて改善される。その結果、担当者が替わっても、AIを使った改善のやり方が部内で続く', a, 368)
def kpi_sub_view():
    """補助の指標4つを、作ったSkillsを誰が使ったかで分けて2つずつ並べる。各指標に、変化の向き（開始前と終了後の棒）か実施の印を添える。設問は付録に置く。"""
    lanes = [(0, '自分だけで使った人', 'one',
              [('実務でAIを使う人の増加', '月に1回以上、実務でAIを使う人の数', '実務でAIを使う人が広がったか', 'up'),
               ('確認 ・ 修正の手間の変化', '手間が減った人の割合と、その記録', '作成後の確認と手戻りが減ったか', 'down')]),
             (572, 'チームで使った人', 'team',
              [('チームでの改善の実施', '振り返りの場を1回以上開いた人の割合', 'チームで使い、直す流れを実行できたか', 'check'),
               ('やり方のチームへの定着', 'Skillsの共有が進み、立ち上げが短くなった人の割合', '担当者が替わっても、同じ観点で確認できるか', 'check')])]
    a = ''
    for x, head, who, cards in lanes:
        a += (person(x + 14, 20, .5) if who == 'one' else person(x + 14, 20, .5) + person(x + 36, 20, .5) + person(x + 58, 20, .5))
        a += text(x + (40 if who == 'one' else 84), 26, head, 18, INK, 700)
        for k, (name, what, judge, mark) in enumerate(cards):
            y = 46 + k * 142
            a += rect(x, y, 540, 130, PAPER, 12, LINE)
            a += text(x + 24, y + 36, name, 19, INK, 700)
            a += text(x + 24, y + 68, what, 15, INK)
            a += text(x + 24, y + 106, '判定 ・ ' + judge, 15, DIM)
            # 右端に、変化の向きか実施の印を置く
            bx = x + 440
            if mark in ('up', 'down'):
                h1, h2 = (30, 62) if mark == 'up' else (62, 30)
                a += rect(bx, y + 92 - h1, 30, h1, LINE, 3) + rect(bx + 44, y + 92 - h2, 30, h2, ACCENT, 3)
                a += path(f'M{bx - 6} {y + 92} H{bx + 80}', LINE, 1.5)
                a += text(bx + 15, y + 112, '開始前', 11, DIM, anchor='middle') + text(bx + 59, y + 112, '終了後', 11, ACCENT, 700, anchor='middle')
            else:
                a += circle(bx + 37, y + 62, 28, PAPER, ACCENT) + icon(bx + 24, y + 50, 'check', ACCENT, .65)
    return svg('補助の指標4つを、作ったSkillsを誰が使ったかで分ける。自分だけで使った人は、実務でAIを使う人の増加と、確認・修正の手間の変化を、開始前と終了後で比べる。チームで使った人は、チームでの改善の実施と、やり方のチームへの定着を、実施した人の割合で見る', a, 336)

def kpi_part2():
    """基盤づくりの指標4つを、運営の発信を受けた利用者の循環（受け取って試す→共有する→探して使う→続ける）の順に並べ、各段で何を判定するかを添える。最後は運営の次の発信へ戻る。"""
    cards = [('person', '受け取って試す', '参加者以外での再利用', ['参加しなかった希望者のうち、', '見つけた事例やSkillsを', '業務で試した人の割合'], ['部内に広がる方向へ', '近づいたか']),
             ('chat', '共有する', 'ナレッジを出す人の広がり', ['参加しなかった希望者のうち、', '事例やノウハウを1件以上', '載せた人の割合'], ['利用者からも、受け身では', 'ない発信が起きたか']),
             ('search', '探して使う', 'ナレッジの見つけやすさ', ['探していた事例や', 'ノウハウにたどり着けた', '割合'], ['次の期に部内へ広げる', '価値があるか']),
             ('calendar', '続ける', 'チームでの改善の継続', ['終了後も振り返りの場を', '続けると決めた', '参加者の割合'], ['義務ではなく、自然に', '続く形になったか'])]
    w, gap = 260, 24
    a = ''
    for k, (ic, step, name, what, judge) in enumerate(cards):
        x = k * (w + gap)
        a += rect(x, 0, w, 300, PAPER, 12, ACCENT if k == 0 else LINE)
        if ic == 'person':
            a += person(x + 38, 44, .6, ACCENT)
        else:
            a += icon(x + 20, 22, ic, ACCENT, .6)
        a += text(x + 70, 46, step, 16, ACCENT, 700)
        a += text(x + 20, 92, name, 17, INK, 700)
        a += text(x + 20, 126, what, 14, INK, gap=22)
        a += path(f'M{x + 20} 206 H{x + w - 20}', LINE, 1)
        a += text(x + 20, 232, '判定', 12, DIM, 700) + text(x + 20, 256, judge, 14, DIM, gap=21)
        if k < 3:
            a += arrow(x + w + 3, 46, x + w + gap - 3, 46, ACCENT)
    # 続けた結果を受けて、運営が次の発信と基盤の改善へつなげる
    a += path('M1102 300 V328 H10 V308 m-7 7 l7 -7 7 7', ACCENT, 2)
    a += text(556, 352, '運営は結果を受けて、次の発信と基盤の改善につなげる', 15, ACCENT, 700, 'middle')
    return svg('基盤づくりの指標4つを、運営の発信を受けた利用者の循環の順に並べる。受け取って試す（参加者以外での再利用）、共有する（ナレッジを出す人の広がり）、探して使う（必要なナレッジの見つけやすさ）、続ける（チームでの改善の継続）。運営は結果を受けて、次の発信と基盤の改善につなげる', a, 362)

def down(x, y1, y2, color=LINE):
    return path(f'M{x} {y1} V{y2} m-7 -7 l7 7 7 -7', color)


def forms_timeline():
    """4本のフォームを時期の順に並べ、回答者名で突き合わせることを示す。"""
    cards = [
        ('10月 ・ 希望者 ・ 約3分', '事前', ['いまのAIの使い方', '確認 ・ 修正の手間', 'チームでの共有']),
        ('12月 ・ 希望者 ・ 約5分', '参加時', ['参加の意思', '改善したいこと（最大3件）', '明確化の実感']),
        ('3月 ・ 参加者 ・ 約5〜8分', '参加者の終了時', ['改善ごとの結果と根拠', '手間の変化 ・ いまの使い方', 'チームでの振り返り']),
        ('3月 ・ 参加しなかった人 ・ 約1分', '希望者の終了時', ['試したか', 'たどり着けたか', '何を探していたか']),
    ]
    a = ''
    for i, (when, name, items) in enumerate(cards):
        x = i * 284
        cx = x + 130
        a += text(cx, 26, when, 15, DIM, 400, 'middle')
        a += rect(x, 40, 260, 210, PAPER, 12, LINE)
        a += text(cx, 84, name, 22, INK, 700, 'middle')
        for j, s_ in enumerate(items):
            y = 124 + j * 34
            a += circle(x + 28, y - 6, 4, LINE, LINE)
            a += text(x + 44, y, s_, 16, DIM)
        if i < 3:
            a += arrow(x + 262, 145, x + 282, 145)
    a += path('M130 250 V288 M414 250 V288 M698 250 V288 M982 250 V288 M130 288 H982', ACCENT, 2.5)
    a += text(556, 320, '回答者名（Formsの Record name で記録）で、同じ人の回答を突き合わせる', 20, ACCENT, 700, 'middle')
    return svg('事前（動画の公開前）・ 参加時（教材の視聴後）・ 参加者の終了時 ・ 参加しなかった希望者の終了時の4本のフォームを、回答者名で突き合わせる', a, 332)


def final_share():
    """3月の共有を、載せる→見せ合う→確認するの3段で示す。確認は、誰のためかに書いた人が、受付で決めた基準と、確認・修正の手間の変化を見る。"""
    a = ''
    cols = [(0, 336, '1　載せる', False), (388, 336, '2　見せ合う', True), (776, 336, '3　確認する', False)]
    for x, w, head, hot in cols:
        a += rect(x, 0, w, 312, PAPER, 12, ACCENT if hot else LINE)
        a += text(x + 22, 34, head, 15, ACCENT if hot else DIM, 700)
    a += arrow(344, 156, 380, 156) + arrow(732, 156, 768, 156)
    # 1 載せる：Skillsと記録
    a += icon(22, 52, 'doc', INK, .8)
    a += text(76, 74, ['Skillsと記録を', 'ポータルに載せる'], 18, INK, 700, gap=26)
    for k, item in enumerate(['目的と要件', '判断の観点と手順', '使い方', '試した結果と改善']):
        y = 150 + k * 34
        a += icon(26, y - 14, 'check', ACCENT, .4) + text(54, y, item, 16, INK)
    a += text(22, 296, '会のあとも、ほかの人が探せる', 13, DIM)
    # 2 見せ合う：最終共有会
    a += text(410, 74, '最終共有会', 21, ACCENT, 700)
    a += rect(486, 92, 140, 84, PAPER, 6, ACCENT) + icon(530, 104, 'screen', ACCENT, .8) + text(556, 168, 'デモ', 13, ACCENT, 700, 'middle')
    for k, px in enumerate([466, 526, 586, 646]):
        a += person(px, 214, .62, INK)
    a += path('M426 246 H686', LINE, 2)
    a += text(410, 278, ['軽食を囲む交流会で、', '短いデモと会話で見せ合う'], 14, DIM, gap=20)
    # 3 確認する：何が変わったか
    a += person(806, 74, .6) + text(836, 70, ['誰のためかに書いた人', '（自分 ・ チームの人 ・ 上長など）'], 13, INK, 700, gap=19)
    a += text(798, 128, '受付で決めた「何が変われば達成か」', 13, DIM, 700)
    rows = ['工数', '期間', '品質', '確認 ・ 修正の手間']
    for k, r in enumerate(rows):
        y = 148 + k * 34
        a += rect(798, y, 292, 28, PAPER, 6, LINE)
        a += text(812, y + 19, r, 14, INK, 700)
        a += text(964, y + 19, '開始時', 12, DIM) + path(f'M1006 {y + 14} H1026 m-6 -5 l6 5 -6 5', LINE, 1.5) + text(1032, y + 19, '終了時', 12, INK, 700)
    a += text(798, 296, '何が変わったかを、基準と照らして確認する', 13, DIM)
    return svg('3月の共有は3段で進める。1 Skillsと記録（目的と要件、判断の観点と手順、使い方、試した結果と改善）をポータルに載せる。2 最終共有会で、軽食を囲み、短いデモと会話で見せ合う。3 誰のためかに書いた人（自分・チームの人・上長など）が、受付で決めた何が変われば達成かに照らして、工数・期間・品質・確認と修正の手間が開始時から終了時にどう変わったかを確認する', a, 316)

def steps_figure(entry_label, entry, steps, aside, band, label):
    """教材の進め方：入口の問い→番号付きの手順（左から右）→補足の一行→下の帯。"""
    a = rect(0, 0, 1112, 84, PANEL, 12)
    a += text(24, 32, entry_label, 14, DIM, 700)
    a += text(24, 64, entry, 19, INK, 700)
    a += path('M556 84 V112 M549 105 L556 112 L563 105', LINE, 2)
    n = len(steps); gap = 28; w = (1112 - gap * (n - 1)) / n
    for k, lines in enumerate(steps):
        x = k * (w + gap)
        a += rect(x, 120, w, 150, PAPER, 12, LINE)
        a += circle(x + 32, 152, 16, ACCENT, ACCENT) + text(x + 32, 158, str(k + 1), 16, PAPER, 700, 'middle')
        a += text(x + 20, 202, lines, 16, INK, 700, gap=24)
        if k < n - 1:
            a += arrow(x + w + 4, 195, x + w + gap - 4, 195)
    a += text(0, 298, aside, 14, DIM)
    a += rect(0, 314, 1112, 52, PAPER, 12, ACCENT)
    a += text(556, 347, band, 18, ACCENT, 700, 'middle')
    return svg(label, a, 372)


def lesson_figure(fact, question, start, tiers, aside, goal, label):
    """教材で理解すること：左に入口（よくある状況と問い）、右に4段（すること・なぜ必要か）、下に補足の一行と帯。"""
    a = rect(0, 24, 268, 304, PANEL, 12)
    a += text(20, 54, '入口：よくある状況', 13, DIM, 700)
    a += text(20, 84, fact, 15, INK, gap=23)
    a += path('M20 136 H248', LINE, 1)
    a += text(20, 168, question, 18, INK, 700, gap=28)
    a += text(20, 286, start, 14, ACCENT, 700, gap=21)
    a += path('M268 176 H290 V53 H318 m-8 -7 l8 7 -8 7', LINE, 2)
    a += text(442, 14, 'すること', 12, DIM, 700) + text(746, 14, 'なぜ必要か', 12, DIM, 700)
    y = 24
    for k, (ic, head, sub, why) in enumerate(tiers):
        h = 66
        x = 326
        a += rect(x, y, 786, h, PAPER, 12, LINE)
        a += circle(x + 26, y + h / 2, 14, ACCENT, ACCENT) + text(x + 26, y + h / 2 + 5, str(k + 1), 14, PAPER, 700, 'middle')
        if ic == 'person':
            a += person(x + 76, y + h / 2 + 2, .62)
        elif ic == 'robot':
            a += f'<g transform="translate({x + 60} {y + h / 2 - 19}) scale(.78)">' + _robot_at(0, 0, INK) + '</g>'
        else:
            a += icon(x + 58, y + h / 2 - 18, ic, INK, .66)
        a += text(x + 116, y + h / 2 - 3, head, 17, INK, 700) + text(x + 116, y + h / 2 + 18, sub, 13, DIM)
        a += path(f'M{x + 404} {y + 12} V{y + h - 12}', LINE, 1)
        a += text(x + 420, y + h / 2 - 6, why, 15, INK, gap=23)
        if k < len(tiers) - 1:
            a += path(f'M{x + 26} {y + h + 2} V{y + h + 12} m-6 -6 l6 6 6 -6', LINE, 2)
        y += h + 14
    a += text(0, 352, aside, 14, DIM)
    a += rect(0, 366, 1112, 50, PAPER, 12, ACCENT)
    # 他の枚の帯と同じく中央にそろえる。見出しの語は小さく、到達点と1行で読む
    a += (f'<text x="556" y="397" text-anchor="middle" fill="{ACCENT}" font-weight="700">'
          f'<tspan font-size="14">終えると　</tspan><tspan font-size="18">{escape(goal)}</tspan></text>')
    return svg(label, a, 420)


def lesson1_steps():
    """教材1で理解すること。割り当てられた作業だけを見る状態から、使う人の目的に戻り、範囲・要件・AIに任せる作業へと具体に戻る。"""
    return lesson_figure(['割り当てられた作業は、', 'AIで速く終わる。'], ['では、その作業は', '業務で何を', '満たすべきか'], ['まず、使う人の', '目的に戻る'],
        [('person', '使う人の目的に戻る', '何を達成したいのか', ['作業の内容だけでは、', '何を満たすべきかが決まらない']),
         ('branch', '実現する範囲を決める', '今回どこまでを扱うか', ['目的は広げればよいわけではない。', '範囲に合う大きさにする']),
         ('doc', '守る条件を要件にする', '仕様と業務を知る人に確認する', ['仕様書があっても、今回必要な', '条件が理解されているとは限らない']),
         ('robot', 'AIに任せる作業を決める', '確認する結果もあわせて決める', ['AIに任せる範囲と、', '人が確認する結果を分ける'])],
        '具体と抽象を往復し、目的に戻ってから、具体的な要件と作業へ下りていく',
        '作業を任されたとき、何を満たせば完了かを説明できる',
        '教材1は、割り当てられた作業はAIで速く終わるが、その作業が業務で何を満たすべきかは確認されていない、という状況から入る。使う人の目的に戻る、実現する範囲を決める、守る条件を要件にする、AIに任せる作業と確認する結果を決める、の順に、目的から具体的な要件と作業へ下りていく')


def lesson2_steps():
    """教材2で理解すること。要件は明確になったが、設計で業務の意味を保てるかは別の問題。業務の判断と関係をつかみ、モデルに表し、設計へ反映し、変更のたびに見直す。"""
    return lesson_figure(['要件は明確になった。', '実装もAIで速く進む。'], ['では、変更を重ねても', '業務の意味を保てる', '設計になっているか'], ['まず、業務の判断と', '関係をつかむ'],
        [('search', '業務の対象と判断をつかむ', '何に対する、誰の判断か', ['同じ手続きの中でも、', '判断の対象が異なることがある']),
         ('branch', '関係と守る条件をモデルに表す', '何を変えたら、どの判断が有効か', ['モデルがないと、', '条件が各処理に散らばる']),
         ('doc', '設計へ反映する', '状態 ・ 処理 ・ 責務 ・ 境界', ['設計から、業務の判断と', '守る条件を読み取れるようにする']),
         ('check', '変更のたびにモデルを見直す', '新しい要件をモデルで表せるか', ['見直さずに条件を足し続けると、', '確認の範囲が広がる'])],
        '業務の言葉の意味は、このモデルと設計を考えるために確認する',
        '新しい要件が来たとき、今の設計で扱えるかを判断できる',
        '教材2は、要件は明確になり実装もAIで速く進むが、変更を重ねても業務の意味を保てる設計かは別の問題、という状況から入る。業務の対象と判断をつかむ、関係と守る条件をモデルに表す、設計へ反映する、変更のたびにモデルを見直す、の順に進める')


def background():
    """AIで作成は速くなったが、作った分の確認と修正が残り、確認の観点は人によって異なる。作成の工程が短くなっても、確認と修正が増えれば案件全体は同じようには短くならない（例のイメージ）。"""
    a = text(0, 18, 'いま起きていること', 15, DIM, 700)
    # 3つの出来事を、アイコンと短い文で示す
    cards = [('作成が速くなった', ['資料・設計書・コード・テストを', 'AIで短時間に作成できる']),
             ('確認と修正は残る', ['要件や元の情報と照合し、', '修正して再確認する']),
             ('観点は人によって異なる', ['確認の観点が共有されていないと、', '見落としが手戻りにつながる'])]
    for i, (head, sub) in enumerate(cards):
        x = i * 388
        a += rect(x, 30, 336, 128, PAPER, 12, LINE)
        if i == 0:
            a += f'<g transform="translate({x + 22} 46) scale(.62)">' + _robot_at(0, 0, INK) + '</g>'
            for k in range(3):
                a += icon(x + 62 + k * 8, 50 - k * 4, 'doc', INK, .38)
        elif i == 1:
            a += icon(x + 30, 48, 'doc', INK, .5) + icon(x + 52, 62, 'search', INK, .38)
        else:
            a += person(x + 36, 66, .5) + person(x + 72, 66, .5)
            a += icon(x + 24, 84, 'check', ACCENT, .3) + text(x + 62, 96, '?', 15, ACCENT, 700)
        a += text(x + 118, 64, head, 18, INK, 700)
        a += text(x + 22, 120, sub, 15, DIM, gap=23)
        if i < 2:
            a += arrow(x + 344, 94, x + 380, 94)
    # 例のイメージ：作成の工程は短くなっても、案件全体は同じようには短くならない
    a += text(0, 196, 'その結果（例のイメージ）', 13, DIM, 700)
    rows = [('以前', [('作成', 300, PANEL), ('確認 ・ 修正', 260, PANEL)]),
            ('AIを使う', [('作成', 110, ACCENT), ('確認 ・ 修正 ・ 手戻り', 430, PANEL)])]
    for k, (name, segs) in enumerate(rows):
        y = 208 + k * 40
        a += text(0, y + 22, name, 14, INK, 700)
        x = 96
        for lab, w, col in segs:
            a += rect(x, y, w, 30, col, 4, LINE if col == PANEL else 'none')
            a += text(x + 10, y + 20, lab, 12, PAPER if col == ACCENT else INK, 700)
            x += w + 4
    a += text(760, 230, ['作成の工程は短くなっても、', '確認と修正が増えれば、案件全体の', '工数や期間は同じようには短くならない'], 14, INK, gap=21)
    a += rect(0, 300, 1112, 54, PANEL, 12)
    a += text(556, 334, 'AIの活用は、確認・修正まで含めた仕事全体の生産性と品質の改善につながっているか', 21, ACCENT, 700, 'middle')
    return svg('AIで資料・設計書・コード・テストを速く作成できるようになったが、作った分の照合・修正・再確認は残り、確認の観点は人によって異なる。例のイメージでは、作成の工程は短くなっても、確認・修正・手戻りが増えれば、案件全体の工数や期間は同じようには短くならない。問いは、AIの活用が仕事全体の生産性と品質の改善につながっているか', a, 360)

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



def lesson3_steps():
    """教材3で理解すること。教材1・2の考え方を判断の観点と手順としてSkillsにまとめ、一つのAgentが工程に応じて使い、レビューで元の根拠と照合し、人が確認して改善する。"""
    return lesson_figure(['要件も設計も考えた。', 'AIで作業も速く進む。'], ['では、AIの作業と', 'レビューは、その', '考え方に沿っているか'], ['判断の観点と手順を', 'Skillsにまとめる'],
        [('doc', '判断の観点と手順をSkillsにする', '何を参照し、どう判断し、何を確認するか', ['AIは、伝えられた解釈のまま', '作業を進める']),
         ('robot', '工程に応じてSkillsを使う', 'Agentが要件定義 ・ 設計 ・ 実装 ・ テストで', ['どの工程でも、同じ目的と', '要件に沿って判断させる']),
         ('search', 'レビューで元の根拠と照合する', '要件 ・ 業務モデル ・ 設計 ・ テスト結果', ['作業時の解釈のまま確認すると、', '同じ誤解を見逃す']),
         ('person', '人が確認し、Skillsを改善する', '判断の根拠と結果を確認する', ['Skillsを使うだけで、', '品質が保証されるわけではない'])],
        '一つのAgentが作業を進めながら、工程ごとに必要なSkillsを使う。工程ごとに別のAgentは置かない',
        '自分の業務に合ったSkillsを作り、AIの作業とレビューに使える',
        '教材3は、要件も設計も考え、AIで作業も速く進むが、AIの作業とレビューがその考え方に沿っているか、という状況から入る。判断の観点と手順をSkillsにする、Agentが工程に応じてSkillsを使う、レビューで元の根拠と照合する、人が確認しSkillsを改善する、の順に進める')

def structure():
    """企画は2つの施策で構成する。ワークショップは参加者が自分の業務を改善できるようになる場。基盤づくりは、その結果を部内で共有し使い続けられるようにする場。ワークショップの実践が基盤に載り、基盤から部内へ広がる。"""
    a = ''
    # 左：ワークショップ（参加者）
    a += rect(0, 0, 420, 236, PAPER, 12, ACCENT)
    a += text(24, 34, '施策1', 14, ACCENT, 700) + text(24, 66, 'ワークショップ', 24, ACCENT, 700)
    for k in range(4):
        a += person(52 + k * 34, 128, .55, ACCENT)
    a += text(200, 122, '参加者', 15, DIM, 700) + text(200, 146, '先着15〜20名', 15, INK, 700)
    a += path('M24 168 H396', LINE, 1)
    a += text(24, 194, '目標', 13, DIM, 700) + text(72, 194, ['AIを使って、自分の業務を', '改善できるようになる'], 17, INK, 700, gap=25)
    # 右：基盤づくり（運営と部内）
    a += rect(692, 0, 420, 236, PAPER, 12, LINE)
    a += text(716, 34, '施策2', 14, DIM, 700) + text(716, 66, 'AI活用を促進する基盤づくり', 22, INK, 700)
    a += person(744, 128, .55) + person(774, 128, .55) + person(804, 128, .55)
    a += text(848, 122, '運営チーム', 15, DIM, 700) + text(848, 146, '人選した約3名', 15, INK, 700)
    a += path('M716 168 H1088', LINE, 1)
    a += text(716, 194, '目標', 13, DIM, 700) + text(764, 194, ['AIを使った改善のやり方が、', '部内で共有され、使い続けられる'], 17, INK, 700, gap=25)
    # 中央：2つの施策のつながり
    a += arrow(432, 76, 680, 76, ACCENT) + text(556, 62, '実践した事例とSkillsを載せる', 13, ACCENT, 700, 'middle')
    a += path('M680 160 H432 m8 -7 l-8 7 8 7', LINE, 2) + text(556, 148, '事例と機会を発信する', 13, DIM, 700, 'middle')
    a += icon(530, 92, 'search', DIM, .5) + text(570, 118, 'ポータル', 13, DIM, 700)
    # 下：部内へ
    a += path('M902 236 V262', LINE, 2) + path('M210 236 V262', ACCENT, 2)
    a += rect(0, 262, 1112, 58, PANEL, 12)
    for k in range(9):
        a += person(40 + k * 22, 300, .38, DIM)
    a += text(260, 297, '部内のメンバー', 14, DIM, 700)
    a += text(400, 297, '両者で、仕事全体の改善が部内で続く状態を目指す', 19, INK, 700)
    return svg('企画は2つの施策で構成する。ワークショップでは、先着15〜20名の参加者がAIを使って自分の業務を改善できるようになる。AI活用を促進する基盤づくりでは、人選した約3名の運営チームが、改善のやり方を部内で共有し使い続けられるようにする。ワークショップで実践した事例とSkillsをポータルに載せ、基盤から事例と機会を発信する。両者で、仕事全体の改善が部内で続く状態を目指す', a, 324)


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


def agenda(cards, hot=None):
    """説明の順序を示す。2つの施策は並べて1つの枠に入れ、背景から施策へ、施策からお願いしたいことへ矢印でつなぐ。
    cards は (添え字, 見出し, 項目, 枚の範囲)。枚の範囲は {{枚:ラベル}} で書き、並びごとの枚番号に置き換わる。"""
    def em(s):  # 文字の幅（全角を1とする）
        return sum(.56 if ord(c) < 128 else 1 for c in s)
    gap, pad = 28, 12
    need = [max(em(c[1]) * 21, max(em(s_) * 17 + 22 for s_ in c[2])) + 44 for c in cards]
    room = 1112 - gap * 2 - pad * 3
    ws = [x + (room - sum(need)) / 4 for x in need]
    top = 42
    h = 44 + 40 * max(len(c[2]) for c in cards) + 40
    xs = [0, ws[0] + gap + pad]
    xs.append(xs[1] + ws[1] + pad)
    xs.append(xs[2] + ws[2] + pad + gap)
    gx, gw = xs[1] - pad, ws[1] + ws[2] + pad * 3
    a = rect(gx, 0, gw, top + h + pad, PANEL, 14, LINE)
    a += text(gx + pad, 28, '2つの施策', 15, DIM, 700)
    for i, ((tag, head, items, span), x, w) in enumerate(zip(cards, xs, ws)):
        on = (i == hot)
        a += rect(x, top, w, h, PAPER, 12, ACCENT if on else LINE)
        if tag:
            a += text(x + 20, top + 34, tag, 14, ACCENT if on else DIM, 700)
        a += text(x + w - 16, top + 34, span, 13, DIM, anchor='end')
        a += text(x + 20, top + 72, head, 21, ACCENT if on else INK, 700)
        for j, s_ in enumerate(items):
            y = top + 116 + j * 38
            a += circle(x + 26, y - 6, 4, LINE, LINE) + text(x + 40, y, s_, 16, DIM)
    mid = top + h // 2
    a += arrow(ws[0] + 5, mid, gx - 5, mid) + arrow(gx + gw + 5, mid, xs[3] - 5, mid)
    return svg('説明の順序。' + cards[0][1] + '、2つの施策（' + cards[1][1] + '、' + cards[2][1] + '）、' + cards[3][1], a, top + h + pad + 4)


def index_cards(groups, cols=3):
    """付録の区分を、区分名と中身の並びでカードにする。"""
    a = ''
    w = (1112 - 24 * (cols - 1)) // cols
    rows = (len(groups) + cols - 1) // cols
    ih = max(len(g[1]) for g in groups)
    h = 70 + ih * 34
    for k, (head, items) in enumerate(groups):
        x = (k % cols) * (w + 24); y = (k // cols) * (h + 20)
        a += rect(x, y, w, h, PAPER, 12, LINE)
        a += text(x + 24, y + 42, head, 21, INK, 700)
        for j, s_ in enumerate(items):
            yy = y + 84 + j * 34
            a += circle(x + 30, yy - 6, 4, LINE, LINE) + text(x + 46, yy, s_, 16, DIM)
    return svg('付録の区分。' + '、'.join(g[0] for g in groups), a, rows * (h + 20) - 16)


def tools_env():
    """Kiroだけ参加者へ発行し、ClaudeとCopilotは社内の環境をそのまま使うことを、2枚のカードで示す。下に、開始前の確認を並べる。"""
    a = rect(0, 0, 544, 216, PAPER, 12, ACCENT)
    a += icon(26, 30, 'screen', ACCENT, 0.9) + text(94, 60, 'Kiro', 28, ACCENT, 700)
    a += text(26, 118, '参加者15〜20名に発行する', 16, ACCENT, 700)
    a += text(26, 152, ['アカウントとAWS環境', 'AgentとSkillsの作成 ・ 実行に使う'], 17, INK, gap=30)
    a += rect(568, 0, 544, 216, PAPER, 12, LINE)
    a += icon(594, 30, 'chat', INK, 0.9) + text(662, 60, 'Claude', 28, INK, 700)
    a += icon(840, 26, 'doc', INK, 0.9) + text(896, 60, 'Copilot', 28, INK, 700)
    a += text(594, 118, '社内の環境をそのまま使う', 16, DIM, 700)
    a += text(594, 152, ['どのツールでも、', '学んだ考え方を使える'], 17, INK, gap=30)
    a += text(0, 268, '開始前（12月まで）に確認する', 16, DIM, 700)
    for x, c in [(250, '社内のIDE／CLIの版'), (470, '利用枠'), (590, 'ファイル ・ コマンドの権限'), (850, '社内Kiro環境での動作')]:
        a += icon(x, 252, 'check', ACCENT, 0.45) + text(x + 26, 268, c, 16, INK)
    return svg('利用環境。Kiroは参加者15〜20名にアカウントとAWS環境を発行し、ClaudeとCopilotは社内の環境をそのまま使う。開始前に、版 ・ 利用枠 ・ 権限 ・ 動作を確認する', a, 284)



# ── 回答フォームのワイヤーフレーム ──
# 色と書体は決めず、灰色の線と面で描く。強調の色は、横に添える理由の注記だけに使う。
def _wf_lines(s, n):
    """設問文を、全角 n 字ぶんの幅で折り返す。半角は0.55字と数え、句読点と閉じ括弧は行頭に置かない。"""
    out, cur, w = [], '', 0.0
    for ch in s:
        cw = 0.55 if ord(ch) < 128 else 1.0
        if w + cw > n and cur and ch not in '、。）」』,.)':
            out.append(cur); cur, w = '', 0.0
        cur += ch; w += cw
    if cur:
        out.append(cur)
    return out


def wf_phone(x, y, form, section, widgets, w=340, h=372):
    """スマホ幅のFormsの画面。widgets は ('p'|'radio'|'check'|'input'|'area'|'shown'|'button'|'gap', ...)。"""
    a = rect(x, y, w, h, PAPER, 18, LINE)
    a += f'<path d="M{x} {y + 46} V{y + 18} Q{x} {y} {x + 18} {y} H{x + w - 18} Q{x + w} {y} {x + w} {y + 18} V{y + 46} Z" fill="{PANEL}"/>'
    a += text(x + 18, y + 20, form, 11, DIM, 700) + text(x + 18, y + 38, section, 14, INK, 700)
    cy = y + 66
    ix, iw = x + 18, w - 36
    per = iw / 12.5
    for wd in widgets:
        kind = wd[0]
        if kind == 'gap':
            cy += wd[1]
        elif kind == 'p':
            for line in wd[1]:
                a += text(ix, cy, line, 11, DIM); cy += 17
            cy += 6
        elif kind in ('radio', 'check'):
            for line in _wf_lines(wd[1], per):
                a += text(ix, cy, line, 12, INK, 700); cy += 17
            for opt in wd[2]:
                if kind == 'radio':
                    a += circle(ix + 6, cy - 4, 5.5, PAPER, DIM)
                else:
                    a += rect(ix + 1, cy - 9, 11, 11, PAPER, 2, DIM)
                a += text(ix + 20, cy, opt, 11, INK); cy += 16
            cy += 8
        elif kind in ('input', 'area'):
            for line in _wf_lines(wd[1], per):
                a += text(ix, cy, line, 12, INK, 700); cy += 17
            bh = 26 if kind == 'input' else 40
            a += rect(ix, cy - 8, iw, bh, PAPER, 4, LINE)
            a += text(ix + 8, cy + 8, wd[2], 10, DIM)
            cy += bh + 10
        elif kind == 'shown':
            bh = 18 + 16 * len(wd[2])
            a += rect(ix, cy - 10, iw, bh, PANEL, 6)
            a += text(ix + 10, cy + 4, wd[1], 10, DIM, 700)
            for k, line in enumerate(wd[2]):
                a += text(ix + 10, cy + 20 + 16 * k, line, 11, INK)
            cy += bh + 8
        elif kind == 'button':
            a += rect(x + w - 104, y + h - 40, 86, 28, PANEL, 6, LINE) + text(x + w - 61, y + h - 21, wd[1], 12, INK, 700, 'middle')
    return a


def wf_note(x, y, w, lines):
    """ワイヤーフレームの下に、その形にした理由を添える。"""
    a = ''
    for k, line in enumerate(lines):
        a += text(x, y + k * 19, line, 12, ACCENT if k == 0 else DIM, 700 if k == 0 else 400)
    return a


def wf_slide(phones, height=416):
    """phones は (form, section, widgets, notes, arrow_after)。3つまで横に並べる。"""
    a = ''
    gap = (1112 - 3 * 340) / 2
    for i, (form, section, widgets, notes, link) in enumerate(phones):
        x = i * (340 + gap)
        a += wf_phone(x, 0, form, section, widgets)
        a += wf_note(x + 4, 392, 336, notes)
        if link:
            a += arrow(x + 344, 186, x + 340 + gap - 4, 186, ACCENT)
            a += text(x + 340 + gap / 2, 175, link, 11, ACCENT, 700, 'middle')
    return svg('回答フォームの画面のワイヤーフレーム。' + '。'.join(p[1] + '：' + p[3][0] for p in phones), a, height)


def survey_compare(rows):
    """設問の見直しの前後を、フォームごとに並べる。"""
    a = text(0, 18, 'フォーム', 13, DIM, 700) + text(280, 18, '問いの数', 13, DIM, 700) + text(580, 18, '必須の自由記述', 13, DIM, 700) + text(840, 18, '所要時間', 13, DIM, 700)
    for k, (name, q0, q1, t0, t1, m0, m1) in enumerate(rows):
        y = 32 + k * 58
        a += rect(0, y, 1112, 50, PAPER, 8, LINE)
        a += text(16, y + 31, name, 15, INK, 700)
        for cx, before, after in ((280, q0, q1), (580, t0, t1), (840, m0, m1)):
            a += text(cx, y + 31, before, 13, DIM) + path(f'M{cx + 104} {y + 26} H{cx + 126} m-6 -5 l6 5 -6 5', LINE, 1.5) + text(cx + 134, y + 31, after, 15, ACCENT, 700)
    return svg('設問の見直しの前後。' + '、'.join(r[0] for r in rows), a, 32 + len(rows) * 58)


def question_matrix(rows, forms):
    """指標 ・ 用途ごとに、どのフォームで聞くかを点で示す。"""
    lx, cw = 560, 138
    a = ''
    for j, (name, who) in enumerate(forms):
        cx = lx + j * cw + cw // 2
        a += text(cx, 22, name, 16, INK, 700, 'middle') + text(cx, 44, who, 13, DIM, 400, 'middle')
    a += path(f'M0 60 H1112', LINE, 1)
    rh = 54 if len(rows) <= 6 else 48  # 行が多いときは詰めて、枠に収める
    for i, (name, qs, marks, hot) in enumerate(rows):
        y = 60 + i * rh
        a += text(0, y + rh // 2 - 3, name, 17, ACCENT if hot else INK, 700) + text(0, y + rh // 2 + 17, qs, 13, DIM)
        for j in range(len(forms)):
            cx = lx + j * cw + cw // 2
            if j in marks:
                a += circle(cx, y + rh // 2, 10, ACCENT if hot else INK, ACCENT if hot else INK)
            else:
                a += circle(cx, y + rh // 2, 3, LINE, LINE)
        a += path(f'M0 {y + rh} H1112', LINE, 1)
    return svg('指標と用途ごとに、どのフォームで聞くか。' + '、'.join(r[0] for r in rows), a, 60 + len(rows) * rh + 4)


def _robot_at(x, y, color):
    """AIを指す図形。絶対座標で描く（オリエンテーションの intro_visuals._robot_at と同じ形）。"""
    a = path(f'M{x + 24} {y + 12} V{y + 3}', color, 2.3)
    a += circle(x + 24, y + 2, 3, 'none', color)
    a += rect(x + 2, y + 12, 44, 32, 'none', 9, color)
    a += circle(x + 16, y + 27, 3.5, color, color)
    a += circle(x + 32, y + 27, 3.5, color, color)
    a += path(f'M{x + 17} {y + 36} H{x + 31}', color, 2.3)
    a += path(f'M{x + 2} {y + 26} H{x - 6} M{x + 46} {y + 26} H{x + 54}', color, 2.3)
    a += path(f'M{x + 13} {y + 44} V{y + 52} H{x + 35} V{y + 44}', color, 2.3)
    return a


def ws_goal():
    """参加者は自分の仕事で改善したいことと、何が変われば改善かを決める。身につけたい4つの力で進め、終えたときに、決めた改善ができたかを確認する。受付では4つの観点で書く。"""
    # 人物は地に直接置かず、枠で囲む
    a = rect(0, 0, 96, 248, PAPER, 12, LINE)
    a += person(48, 96, 1.0) + text(48, 162, '参加者', 17, INK, 700, 'middle')
    a += text(48, 188, ['先着', '15〜20名'], 13, DIM, anchor='middle', gap=19)
    a += arrow(100, 124, 112, 124)
    # 改善したいことを決める
    a += rect(118, 0, 266, 248, PAPER, 12, LINE)
    a += text(132, 32, '改善したい業務', 18, INK, 700)
    a += text(132, 56, '業務は問わない ・ いくつでも決めてよい', 13, DIM)
    a += rect(132, 70, 232, 66, PAPER, 8, LINE)
    a += text(148, 95, '改善したいこと', 16, INK, 700) + text(148, 120, '例：レビューと修正の手間', 13, DIM)
    a += path('M248 136 V146 M241 139 L248 146 L255 139', LINE, 2)
    a += rect(132, 150, 232, 90, PAPER, 8, ACCENT)
    a += text(148, 174, '何が変われば達成か', 16, ACCENT, 700)
    a += text(148, 198, ['工数 ・ 期間 ・ 品質のうち、', '何がどう変われば', '改善と言えるか'], 13, INK, gap=18)
    a += arrow(392, 124, 424, 124)
    # 身につけたい力：4つを順に使う
    a += rect(432, 0, 420, 248, PAPER, 12, LINE)
    a += text(452, 32, '身につける力', 18, INK, 700)
    steps = ['目的と要件を明確にする', '業務をモデルに表し、設計へつなぐ', '判断の観点と手順をSkillsにまとめる', 'Agentsで作業し、人が確認して改善する']
    for k, s_ in enumerate(steps):
        y = 52 + k * 40
        a += circle(466, y + 14, 12, PAPER, LINE) + text(466, y + 19, str(k + 1), 13, INK, 700, 'middle')
        a += text(488, y + 20, s_, 16, INK, 700)
    a += path('M452 212 H832', LINE, 1)
    a += text(452, 234, '自分だけで使うか、チームでも使うかは目的に合わせて決める', 13, DIM)
    a += arrow(860, 124, 884, 124)
    # 終えたとき
    a += rect(892, 0, 220, 248, PAPER, 12, LINE)
    a += text(912, 32, '終えたとき', 15, DIM, 700)
    a += icon(982, 50, 'check', INK, 1.1)
    a += text(1002, 120, ['決めた改善が', 'できたかを、', '記録で検証している'], 16, INK, 700, 'middle', 25)
    a += text(1002, 208, ['次の業務でも、同じ考え方で', 'Skillsを作成・改善できる'], 13, DIM, anchor='middle', gap=19)
    # 参加の受付で書く4つ
    a += rect(0, 258, 1112, 64, PANEL, 12)
    a += text(24, 284, '参加の受付で書く4つ', 15, INK, 700)
    a += text(196, 284, '何を実現するか ・ 誰のためか ・ 何が変われば達成か ・ 達成に何が必要か', 17, INK, 700)
    a += text(196, 309, '何が変われば達成かは、工数 ・ 期間 ・ 品質のうち何がどう変わるかで書き、終了時の検証の基準にする', 15, DIM)
    return svg('参加者は業務を問わず、自分の仕事で改善したいことと、工数・期間・品質のうち何が変われば改善かを決める。目的と要件を明確にする、業務をモデルに表し設計へつなぐ、判断の観点と手順をSkillsにまとめる、Agentsで作業し人が確認して改善する、の4つの力で進め、終えたときに決めた改善ができたかを確認・修正の手間も含めて検証する。参加の受付では、何を実現するか・誰のためか・何が変われば達成か・達成に何が必要かの4つを書く', a, 322)


def impact_rows():
    """三つの教材ごとに、残るものと、期待する業務の変化を並べ、下に、目指すこと（必要な品質で、仕事全体の工数と期間を短くする）を置く。"""
    a = text(0, 20, '身につけること（三つの教材）', 15, DIM, 700) + text(404, 20, '業務に残るもの', 15, DIM, 700)
    a += text(790, 20, '期待する業務の変化', 15, DIM, 700)
    rows = [('教材1', '目的と要件の明確化', 'doc', ['作業ごとの目的 ・ 要件と、', '達成の基準'], ['要件の解釈のずれに、', '作成前やレビューで気づきやすい'], '事例では：要件の解釈のずれ'),
            ('教材2', '業務のモデル化と設計', 'book', ['業務のモデルと、', '設計の理由'], ['変更の影響を把握しやすく、', '確認の範囲を絞り込みやすい'], '事例では：残った設計の問題'),
            ('教材3', 'AI活用の実践と改善', 'robot', ['判断の観点と手順を', 'まとめたSkills'], ['AIと人が同じ観点で確認でき、', '確認の質が人に左右されにくい'], '事例では：同じ解釈のテストとレビュー')]
    for k, (no, name, ic, keep, eff, case) in enumerate(rows):
        y = 34 + k * 86
        a += rect(0, y, 330, 74, PAPER, 12, LINE)
        a += rect(404, y, 310, 74, PAPER, 12, LINE)
        a += rect(790, y, 322, 74, PAPER, 12, LINE)
        a += text(22, y + 30, no, 14, DIM, 700) + text(22, y + 58, name, 20, INK, 700)
        a += arrow(340, y + 37, 386, y + 37)
        a += _robot_at(412, y + 11, INK) if ic == 'robot' else icon(414, y + 11, ic, INK, .9 if ic == 'book' else .95)
        a += text(484, y + 30, keep, 16, INK, gap=26)
        a += arrow(730, y + 37, 776, y + 37)
        a += circle(812, y + 37, 13, ACCENT, ACCENT) + icon(803, y + 30, 'check', PAPER, .45)
        a += text(838, y + 22, case, 11, DIM, 700) + text(838, y + 44, eff, 15, INK, 700, gap=21)
    a += rect(0, 296, 1112, 70, PAPER, 12, ACCENT)
    a += text(24, 324, '目指すこと', 15, ACCENT, 700) + text(24, 352, '仕事全体の改善', 20, ACCENT, 700)
    a += path('M262 312 V350', LINE, 1.5)
    a += text(288, 324, '必要な品質を満たしながら、手戻りと確認 ・ 修正の手間を減らし、案件全体の工数と期間を短縮する', 16, INK, 700)
    a += text(288, 352, 'Skillsが残るため、担当者が替わっても、同じ観点で確認できる', 16, INK, 700)
    return svg('三つの教材ごとに、残るものと期待する業務の変化を並べる。目的と要件の明確化からは目的と要件と達成の基準が残り、解釈のずれに早く気づく。業務のモデル化と設計からは業務のモデルと設計の理由が残り、変更の影響を読み取りやすくなる。AI活用の実践と改善からはSkillsが残り、AIと人が同じ観点で確かめる。目指すのは、必要な品質を満たしながら手戻りと確認・修正の手間を減らし、案件全体の工数と期間を短くすること', a, 372)


def mechanism():
    """作ったものは、業務ごとのAgentと、その下の作業ごとのSkills（判断の観点と確認の手順）で残る。参照する情報（目的・要件・業務のルール）は業務ごとにまとめ、Skillsが参照する。"""
    cols = [('PM ・ PL', '案件の状況を把握し、判断する', ['完了の根拠を要件と照合する', '残作業と日程の影響を整理する', '判断に必要な報告を作る']),
            ('開発', '要件どおりに、品質よく作る', ['要件と設計の対応を確認する', 'コードを設計と照合する', '必要な状態のテストを作る']),
            ('事務 ・ 管理', '申請と問い合わせを処理する', ['申請内容を規程と照合する', '根拠を添えて回答を作る', '定型の書類を作る'])]
    a = ''
    for k, (role, goal, skills) in enumerate(cols):
        x = k * 380
        hot = k == 0
        a += rect(x, 0, 352, 84, PAPER, 12, ACCENT if hot else INK)
        a += _robot_at(x + 20, 16, ACCENT if hot else INK)
        a += text(x + 88, 36, role + 'のAgent', 17, ACCENT if hot else INK, 700)
        a += text(x + 88, 62, '業務：' + goal, 14, DIM)
        a += path(f'M{x + 44} 84 V{102 + 2 * 42 + 17}', LINE, 2)
        for m, sk in enumerate(skills):
            y = 102 + m * 42
            a += path(f'M{x + 44} {y + 17} H{x + 70}', LINE, 2)
            a += rect(x + 70, y, 282, 34, PAPER, 8, LINE)
            a += text(x + 86, y + 23, 'Skill', 13, DIM, 700) + text(x + 130, y + 23, sk, 15, INK)
        # 参照する情報は業務ごとにまとめ、各Skillが参照する
        dash = rect(x + 70, 236, 282, 34, PAPER, 8, ACCENT).replace('stroke-width="2"', 'stroke-width="2" stroke-dasharray="5 4"')
        a += path(f'M{x + 336} 220 V236', ACCENT, 1.5)
        a += dash + text(x + 86, 259, '参照する情報：目的 ・ 要件 ・ ルール', 14, ACCENT, 700)
    a += text(0, 298, 'Skillsには判断の観点と確認の手順を書き、参照する情報は業務ごとにまとめる。使って見つかったずれは、Skillsの改善につなげる', 14, DIM)
    # 誰が使うかで変わる使い方
    uses = [('自分で使うSkills', person(34, 350, .7), '自分で使い、結果を確認して直す'),
            ('チームで使うSkills', person(588, 350, .6) + person(614, 350, .6) + person(640, 350, .6), 'チームで使い、確認の観点を一緒に直す')]
    for k, (name, fig, t) in enumerate(uses):
        x = k * 572
        a += rect(x, 312, 540, 72, PAPER, 12, LINE) + fig
        a += text(x + (88 if k == 0 else 102), 340, name, 14, DIM, 700) + text(x + (88 if k == 0 else 102), 368, t, 17, INK, 700)
    return svg('業務ごとにAgentがあり、その下に作業ごとのSkillsが並ぶ。Skillsには判断の観点と確認の手順を書き、目的・要件・業務のルールなど参照する情報は業務ごとにまとめる。自分で使うSkillsは自分で使って結果を確認して直し、チームで使うSkillsはチームで使って確認の観点を一緒に直す', a, 392)


# ワークショップの土台にした3冊。表紙は docs/workshops/ai-workshop/books/<ISBN>.jpg に置く
BOOKS = [('9784297157906', ['システム開発と', '「具体と抽象」'], '細谷功', '技術評論社'),
         ('9784798195490', ['ユースケース実践ガイド', '［復刻版］'], 'アリスター・コーバーン', '翔泳社'),
         ('9784814400737', ['ドメイン駆動設計を', 'はじめよう'], 'Vlad Khononov', 'オライリー・ジャパン')]


def cover(x, y, w, h, isbn):
    """書籍の表紙。ファイルを data URI にして埋め込み、1枚で完結させる。"""
    import base64, pathlib
    f = pathlib.Path(__file__).resolve().parents[2] / 'books' / f'{isbn}.jpg'
    b = base64.b64encode(f.read_bytes()).decode()
    return (f'<image x="{x}" y="{y}" width="{w}" height="{h}" preserveAspectRatio="xMidYMid meet" href="data:image/jpeg;base64,{b}"/>'
            + f'<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="none" stroke="{LINE}" stroke-width="1"/>')


def book_basis():
    """具体と抽象の考え方を土台の段に置き、それを補う2冊をその上の段に置く。どの本の考え方を、どの教材に取り入れたか。"""
    concrete, uc, ddd = BOOKS
    a = text(0, 14, 'その上の段：この考え方を補う2冊', 14, DIM, 700)
    upper = [(uc, ['目的レベルと範囲を決め、', '要件にする'], '教材1　目的と要件の明確化'),
             (ddd, ['業務をモデルに表し、', '設計と実装につなげる'], '教材2　業務のモデル化と設計')]
    for k, ((isbn, title, author, pub), idea, lesson) in enumerate(upper):
        x = k * 572; y = 26
        a += rect(x, y, 540, 168, PAPER, 12, LINE)
        a += cover(x + 18, y + 14, 98, 140, isbn)
        a += text(x + 134, y + 36, title, 17, INK, 700, gap=25)
        a += text(x + 134, y + 102, author, 13, DIM) + text(x + 134, y + 124, pub, 13, DIM)
        a += text(x + 340, y + 34, '教材に取り入れた考え方', 12, DIM, 700)
        a += text(x + 340, y + 62, idea, 15, INK, gap=23)
        a += rect(x + 330, y + 114, 196, 34, PAPER, 17, LINE) + text(x + 428, y + 136, lesson, 13, INK, 700, 'middle')
        a += path(f'M{x + 270} {y + 168} V{y + 196}', LINE, 2)
    # 土台の段
    y0 = 222
    isbn, title, author, pub = concrete
    a += rect(0, y0, 1112, 136, PAPER, 12, ACCENT)
    a += cover(18, y0 + 14, 76, 108, isbn)
    a += text(112, y0 + 34, '土台の段', 13, ACCENT, 700)
    a += text(112, y0 + 64, ''.join(title), 17, INK, 700)
    a += text(112, y0 + 92, author + ' ・ ' + pub, 13, DIM)
    a += path(f'M440 {y0 + 22} V{y0 + 114}', LINE, 1)
    a += text(468, y0 + 34, '土台にする考え方', 13, ACCENT, 700)
    a += text(468, y0 + 66, ['具体と抽象を往復し、', '目的と要件の適切な大きさを見極める'], 19, INK, 700, gap=28)
    a += text(468, y0 + 120, '教材1 ・ 教材2 ・ 教材3のすべてで使う', 14, DIM)
    return svg('土台の段は、システム開発と「具体と抽象」の、具体と抽象を往復して目的と要件の適切な大きさを見極める考え方で、すべての教材で使う。その上に、目的レベルと範囲を決めて要件にするユースケース実践ガイド（教材1）と、業務をモデルに表して設計と実装につなげるドメイン駆動設計をはじめよう（教材2）を置く', a, 364)
