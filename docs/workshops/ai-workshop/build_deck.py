"""Build the workshop proposal from the local slide-deck HTML template."""
from pathlib import Path
import html
import json
import re
import visuals

OUT = Path(__file__).resolve().parent
TEMPLATE = OUT / 'skills/ai-workshop-slide-deck/assets/deck-template.html'

def esc(s):
    return html.escape(s)

def diagram(nodes, *, caption='', hot=-1):
    count = len(nodes)
    width = 500 if count == 2 else 1040
    gap = 44
    box = (width - gap * (count - 1)) / count
    pieces = [f'<svg class="ws-flow" viewBox="0 0 {width} 172" role="img" aria-label="{esc(caption or "、次に".join(n[0] for n in nodes))}">']
    for i, (title, sub) in enumerate(nodes):
        x = i * (box + gap)
        fill = '#b64326' if i == hot else '#e9e5da'
        ink = '#fffaf2' if i == hot else '#252b29'
        dim = '#fff1e9' if i == hot else '#59605a'
        pieces.append(f'<rect x="{x}" y="18" width="{box}" height="126" rx="8" fill="{fill}"/>')
        pieces.append(f'<text x="{x+box/2}" y="69" text-anchor="middle" fill="{ink}" font-size="25" font-weight="700">{esc(title)}</text>')
        pieces.append(f'<text x="{x+box/2}" y="106" text-anchor="middle" fill="{dim}" font-size="17">{esc(sub)}</text>')
        if i < count - 1:
            end = x + box
            pieces.append(f'<path d="M {end+10} 81 H {end+32} m -7 -7 l 7 7 -7 7" stroke="#80867e" stroke-width="2" fill="none"/>')
    pieces.append('</svg>')
    return ''.join(pieces)

def table(headers, rows, cls=''):
    return '<table class="ws-table '+cls+'"><thead><tr>'+''.join('<th>'+x+'</th>' for x in headers)+'</tr></thead><tbody>'+''.join('<tr>'+''.join('<td>'+c+'</td>' for c in row)+'</tr>' for row in rows)+'</tbody></table>'

def note(s):
    return f'<p class="ws-note">{s}</p>'

def group_svg():
    parts=['<svg class="ws-groups" viewBox="0 0 570 285" role="img" aria-label="15人を、3人ずつ5組の相談グループにする提案">']
    for g, (x,y) in enumerate([(98,72),(283,72),(468,72),(190,217),(375,217)]):
        pts=[(x-32,y+19),(x+32,y+19),(x,y-32)]
        parts.append(f'<path d="M{pts[0][0]} {pts[0][1]} L{pts[1][0]} {pts[1][1]} L{pts[2][0]} {pts[2][1]} Z" fill="none" stroke="#b5b8ab" stroke-width="2"/>')
        for k,(px,py) in enumerate(pts):
            parts.append(f'<circle cx="{px}" cy="{py}" r="15" fill="{"#b64326" if g==2 and k==2 else "#f4f1e8"}" stroke="{"#b64326" if g==2 and k==2 else "#4f5952"}" stroke-width="2"/>')
    parts.append('</svg>')
    return ''.join(parts)

# The slide order and one-sentence claims are fixed before layout construction.
SLIDES = [
 dict(label='下期AI活用ワークショップ', title='下期AI活用ワークショップ', cls='ws-titlepage',
      intro='社内AI人材育成｜各自が進める半年間の取り組み',
      body='<svg class="ws-title-art" viewBox="0 0 1112 130" role="img" aria-label="個々の点がつながり、一つの形になる"><g fill="none" stroke="#9aa18f" stroke-width="2"><circle cx="26" cy="65" r="12"/><circle cx="80" cy="34" r="12"/><circle cx="88" cy="99" r="12"/><path d="M143 65 H205 M265 35 L310 94 L355 35 Z"/><circle cx="265" cy="35" r="10" fill="#f4f1e8"/><circle cx="310" cy="94" r="10" fill="#f4f1e8"/><circle cx="355" cy="35" r="10" fill="#f4f1e8"/><path d="M397 65 H459"/></g><path d="M519 23 L603 65 L519 107 L477 65 Z" fill="#b64326"/></svg>',
      notes='下期AI活用ワークショップの企画をご説明します。開発者、PM・PLなどが、それぞれの仕事の中でAIを活用して新しい仕組みを生み出す力を育てる企画です。'),
 dict(label='企画の目的', title='企画の目的：AIで仕組みを作り、課題を解決する力', cls='ws-cover',
      intro='身近な課題に対して、AIで動く仕組みを自分で作り、解決できる人材を育成する。開発者とPM・PL約15名が、2026年10月から2027年3月に取り組む。',
      body=diagram([('課題を選択する','自分の仕事を観察する'),('小さく作成する','AIで仕組みを用意する'),('改善する','他の人の反応を確認する')],hot=2),
      notes='身近な業務課題から、新しい仕組みをAIで生み出せる人材を育てます。課題の本質を捉え、それを解決する具体的な仕組みへ変えることが狙いです。エージェントやSkillsを使って形にし、実際に試した結果から改善します。'),
 dict(label='育てたい力', title='育てたい力：抽象と具体の往復', cls='ws-abstraction',
      intro='課題の本質を捉え、使う場面に合わせて仕組みへ具体化する。開発者もPM・PLも、この往復を土台にする。',
      body='<svg class="ws-abstraction-fig" viewBox="0 0 1112 295" role="img" aria-label="抽象では課題の本質を捉え、具体ではAIで解決の仕組みを作る。試した結果から課題の捉え方を再確認する。"><rect x="0" y="40" width="382" height="177" rx="8" fill="#e9e5da"/><text x="28" y="78" font-size="18" fill="#59605a">抽象</text><text x="28" y="122" font-size="28" font-weight="700" fill="#252b29">課題の本質を捉える</text><text x="28" y="162" font-size="19" fill="#59605a">出来事から、重要な構造や関係を</text><text x="28" y="193" font-size="19" fill="#59605a">取り出す。</text><rect x="730" y="40" width="382" height="177" rx="8" fill="#e9e5da"/><text x="758" y="78" font-size="18" fill="#59605a">具体</text><text x="758" y="122" font-size="27" font-weight="700" fill="#252b29">AIで解決の仕組みを作る</text><text x="758" y="162" font-size="19" fill="#59605a">使う場面と条件に合わせて形にし、</text><text x="758" y="193" font-size="19" fill="#59605a">役に立つかを確認する。</text><text x="556" y="86" font-size="19" text-anchor="middle" fill="#b64326" font-weight="700">具体化する</text><path d="M411 109 H701 m -10 -10 l 10 10 -10 10" stroke="#b64326" stroke-width="3" fill="none"/><text x="556" y="169" font-size="18" text-anchor="middle" fill="#b64326" font-weight="700">結果から捉え直す</text><path d="M701 191 H411 m 10 -10 l -10 10 10 10" stroke="#b64326" stroke-width="3" fill="none"/></svg>',
      notes='共通の土台にしたいのは、課題の本質を抽象的に捉え、それを具体化して解決する力です。ここでいう抽象とは、目的に照らして、個別の出来事から重要な構造や関係を取り出すことです。何を残し何を省くかを明らかにするため、曖昧な表現へ言い換えることとは異なります。具体化では、使う場面や制約に合わせてAIで仕組みを作り、実際に役に立つかを確認します。最初に捉えた本質も仮説なので、試した結果に応じて捉え直します。この抽象と具体の往復を、開発者とPM・PLの共通の土台にします。'),
 dict(label='テーマの設計', title='テーマ：例は運営が示し、課題は本人が決める',
      intro='運営は困りごとの例を示す。何に取り組むか、どう解決するかは参加者が決める。',
      body=table(['開発者向けの例','PM・PL向けの例'],[
          ['開発プロセスの手間を削減する','提案準備と関係者調整の手間を削減する'],
          ['Webアプリの作成を効率化する','メンバーの状況把握を支援する'],
          ['新しい参加者の業務開始を支援する','相談と知識共有を促進する']])+note('テーマを絞る問い：誰が、どんな場面で困るか。何が変われば役に立ったと判定できるか。'),
      notes='職種ごとに完成品を指定するのではなく、取り組む対象を選べるようにします。たとえばPM・PLなら、資料を作ることだけでなく、判断材料を集める、認識のずれを見つける、相談の入口を作るといった仕事も対象です。これは用途のヒントであり、推奨する解法の一覧ではありません。参加者自身が経験した場面へ絞り込みます。'),
 dict(label='目指す成果',title='目指す成果：他の人が使える仕組み',intro='自分の知識と手順をAIの仕組みに組み込み、他の人へ提供する。',body=visuals.handoff()+note('目指す成果：他の人が業務の効率化や課題の改善に使用できる仕組み。提出物は、動く仕組み・使い方・試した記録の3点。'),notes='自分が持つ知識や仕事の進め方を言語化し、AIで動く仕組みに組み込みます。エージェントに仕事を進める役割を与え、Skillsに手順や判断の目安を持たせることで、その知識を他の人も利用できる形にします。本人と同じ熟練を前提にせず、仕組みを通じて業務の効率化や課題改善に役立ててもらうことが目的です。AIの出力を確認する必要はあり、技能全体を完全に置き換えるという意味ではありません。説明や初回の支援があっても構いません。他の人が使った結果と、どんな改善に役立ったかを確認します。',cls='ws-diagram'),
 dict(label='エージェントとSkills', title='手段：エージェントとSkills', cls='ws-diagram',
      intro='エージェントに役割を与え、Skillsで仕事の進め方を渡す。人が目的と制約を決め、結果を確認する。',
      body=visuals.agents()+note('エージェントとSkillsはKiroの標準機能である。環境の確認状況は{{枚:前提条件と未確定事項}}枚目に記載する。'),
      notes='前のページで示した、自分のスキルをAIで仕組化して他の人にも届けるという成果を、エージェントとSkillsで実現します。この図は学習のために仕事の関係を整理したものです。カスタムエージェントには役割や使える道具、参照する情報などを設定できます。Skillsは必要な場面で参照する手順や参考資料のまとまりです。すべてを自律実行させる必要はなく、何を任せて何を人が確認するかを決めることを重視します。Kiro公式資料は付録に記載しています。'),
 dict(label='教材', title='教材：考え方を教え、AIツールの使い方は最小限', cls='ws-diagram',
      intro='音声付きの短い動画で考え方を学ぶ。AIツールの使い方は、設定を用意して実行し、結果を確認するところまでを示す。',
      body=visuals.learning()+note('完成例は配らない。何をどう作るかは、参加者自身が考える。'),
      notes='教材は2種類に分けます。中心になるのは考え方の動画で、何を任せるか、何を伝えるか、どう確認するかを扱います。もう1つはAIツールの使い方で、設定を用意して実行し、結果を確認するところまでの最小限にとどめます。実行環境はKiroですが、教材が対象とするのは道具に共通する手順です。ツールの使い方が分からないために着手できない状態は避けますが、業務課題の完成例は配りません。完成例を見せると、参加者が自分で考える範囲が狭くなるためです。教材はこれから制作する計画で、この提案資料が受講者向けの動画ではありません。'),
 dict(label='下期の進め方', title='進め方：半年で作成から利用へ',
      intro='2026年10月から2027年3月まで、月ごとの区切りで、各自のペースで進める。',
      body='<div class="ws-phases"><svg class="ws-phase-path" viewBox="0 0 1112 240" role="img" aria-label="10・11月の試作から、12・1月の再確認を経て、2・3月の他の人利用へ進む"><path d="M 355 107 h 21 m -6 -6 l 6 6 -6 6 M 736 107 h 21 m -6 -6 l 6 6 -6 6" fill="none" stroke="#747c6e" stroke-width="2"/></svg><div><p class="ws-date">10・11月</p><h3>学習と作成</h3><p>概念教材を視聴する<br>課題を選択し、小さく作成する</p></div><div><p class="ws-date">12・1月</p><h3>共有と修正</h3><p>12月に中間成果を共有する<br>他の人の反応で方向を再確認する</p></div><div class="ws-phase-hot"><p class="ws-date">2・3月</p><h3>他の人の利用</h3><p>他の人が試用し、改善する<br>3月に成果と学びを共有する</p></div></div>'+note('日程は提案。開始前の9月中に、社内Kiro環境での動作確認を完了する。「10月開始・下期」という決定済みの条件を、上記の期間として計画している。'),
      notes='全員が同じ時間に集合することは前提にしません。10月から翌3月までを提案期間とし、12月に中間共有、3月に最終共有を置きます。初期から試作に触れ、中間発表まで完成を待たないようにします。途中のテーマ変更も、試した結果に理由があれば認めます。休暇や繁忙期を踏まえ、実働20週を工数試算の仮定にしています。'),
 dict(label='サポート体制', title='支援：学習コンテンツと相談', cls='ws-diagram',
      intro='運営に知識が集まっている前提で、学習コンテンツと相談の2つで支援する。',
      body=visuals.support()+note('定例相談会の時間帯は未定。定時後に開催する場合は任意参加とし、Teams投稿でも相談を受け付ける。'),
      notes='参加者同士で専門的なサポートを分担する前提は置きません。ブログや動画で学び、Teamsで運営に相談できる体制にします。月1回の定例相談会は開催案です。資料発表の準備を求めず、今の課題や相談したいことをざっくばらんに話す場にします。時間帯はまだ決まっていません。定時後になる場合は任意参加とし、相談内容の要点をTeamsに残します。参加できない人も投稿で同じ支援を受けられるようにします。'),
 dict(label='中間成果物発表', title='中間発表で、次に試すことを見つける。', cls='ws-sharing ws-diagram',
      intro='12月は、途中の成果と判断に迷っている点を共有する。',
      body=visuals.meeting()+note('発表用の動画制作は不要。開催日は調整し、必要に応じて会を分ける。'),
      notes='中間発表はTeamsのWeb会議で行います。参加者が画面共有で途中の成果物を動かし、困っている点を話します。発表用動画の制作は求めません。開催日は調整し、全員が一度に集まれない場合は発表枠を分けます。完成していなくても、動く部分や試した結果があれば共有できます。発表後に中間アンケートへ回答し、次に試すことを記録します。'),
 dict(label='最終成果物発表・交流会',title='気軽に集まり、作った仕組みを見せ合う。',intro='コミュニケーション施策として、軽食を囲むオフライン交流会を開く。',body=diagram([('見せる','短いデモで取り組みを紹介'),('話す・試す','軽食を囲んで気軽に交流'),('つなげる','使いたい人へ仕組みを共有')],hot=1)+note('その場で試せないものは後日試用する。人気投票・表彰は交流を楽しむ企画として検討する。'),notes='最終回は参加者が集まるオフラインの交流会兼成果物発表会にします。軽食を囲み、短いデモを互いに紹介したり、作った工夫や困ったことを話したりできる、堅苦しくない場を目指します。会場、飲食費、日程は今後調整します。その場で試せない成果物は後日の試用につなげます。交流と他の人利用のきっかけを作る場とし、人気投票は任意の企画案として残します。',cls=''),
 dict(label='効果測定の方法',title='効果測定：事前・中間・終了時の変化',intro='始める前・途中・終わった後の3回、同じ設問に答えてもらい、同じ人の中で変化を見る。',body=visuals.evaluation()+note('判定に使う指標と、その数字を取る設問の実物は、次の2枚に示す。能力の自己評価と実際の利用経験は合算しない。'),notes='{{枚:目指す成果}}枚目で示した目指す成果を、3つの指標で判定します。主KPIは、自分の知識をAIで動く仕組みにし、他の人が使用して役立ったと確認できた参加者の割合を提案します。開始時の参加者数を分母に固定し、未提出や未確認も明示します。個人目標の達成状況と、業務別のAI活用の変化を補助指標にします。アンケートは同じ人・同じ業務を比較し、回答人数を示します。能力の自己評価と実際の利用経験は合算しません。数値目標は事前調査と運営条件を見て開催前に決めます。',cls='ws-eval ws-diagram'),
 dict(label='共通の設問', title='共通の設問：今できることを聞く', cls='ws-survey-core ws-diagram',
      intro='普段の言葉で聞く。AIに詳しくない参加者も、自分の経験から答えられる質問にする。',
      body=visuals.questionnaire()+note('共通7問から3問を抜粋。事前・中間・終了時に、同じ質問と選択肢で聞く。'),
      notes='共通項目は、仕事の中でできることを本人の言葉で答えられるようにしています。使いどころ、困りごとの整理、AIへの依頼、複数作業の依頼、使い方の説明、答えの確認、感想をもとにした改善の7問です。回答は0から4の5段階で、スライド右側の選択肢から、今の自分に近いものを選びます。試す機会がない場合も選べます。難しい概念の理解を問う試験にはせず、同じ質問で自己評価の変化を比べます。実際の力は成果物や試用記録も合わせて確認します。開始前に参加者に近い人に回答してもらい、意味が伝わるかを確認します。'),
 dict(label='付録｜職種別の設問',title='職種別の設問：同じ選択肢で聞く',cls='ws-form',
 intro='職種ごとに担当業務を並べ、回答の選択肢は共通にする。質問：過去4週間に、次の業務でAIをどの程度使いましたか。',
 body=table(['PM・PLの業務 P1〜P6','開発者の業務 D1〜D6'],
   [[p, d] for p, d in zip(visuals.PM_TASKS, visuals.DEV_TASKS)])
  +note('0 未利用 ／ 1 試用のみ ／ 2 実務で1回 ／ 3 実務で複数回。「担当外」「機会なし」は0点へ変換しない。兼務者は両方へ回答する。'),
 notes='職種ごとの設問です。PM・PLには情報収集・集計、報告・予実、課題・担当・期限、品質・障害・リスク、改善提案、過去事例の再利用の6項目を聞きます。開発者には要求・仕様、設計、実装、レビュー、テスト、運用・保守の6項目を聞きます。選択肢は両方とも同じで、0から3の4段階です。担当していない業務と、担当しているが今回は機会が無かった場合は、0点へ変換せず別に扱います。兼務している人は両方へ回答します。'),
 dict(label='部長への依頼',title='本日の依頼：進め方の合意と、Kiro費用の許可',intro='決めていただくのは2件である。運営の工数は、社内の調整で確保する。',body=table(['依頼','内容'],[['企画の趣旨と進め方への合意','身近な課題をAIで解決する力を育てる。半年間、各自が時間を見つけて進める'],['Kiroの費用の許可','Kiro Pro は1人 $20／月。15名で月 $300、半年で $1,800。超過は $0.04／クレジット']])+note('金額は税別で、超過は既定で無効である。社内の契約枠と有効化の可否を9月中に確認する。交流会の会場・軽食費は1月に相談する。'),notes='本日お願いしたいのは2件です。1件目は、この企画の趣旨と進め方へのご合意です。身近な課題をAIで解決する力を育てること、半年間にわたって各自が時間を見つけて進めることの2点です。2件目は、Kiroの費用のご許可です。Kiro Proは1人あたり月額20ドルで、1,000クレジットが含まれます。15名では月300ドル、半年で1,800ドルです。金額は税別です。プランの範囲を超えた分は1クレジットあたり0.04ドルで請求されますが、超過は既定で無効になっており、Kiroのコンソールで有効化しない限り発生しません。社内の契約枠と、有効化の可否を9月中に確認します。運営の工数は社内の調整で確保するため、本日のご依頼には含めていません。交流会の会場と軽食の費用は、人数と会場が決まる1月に改めてご相談します。',cls=''),
 dict(label='付録｜PM・PLのテーマ', title='PM・PLの題材：判断・調整・相談',
      intro='用途のヒント。参加者は、実際に経験した場面から自分の課題を選ぶ。',
      body=table(['困りごとの種類','よくある場面','参加者が確認すること'],[
          ['判断の準備','提案のたびに、必要な材料を探し直す','何が揃えば判断を始められるか'],
          ['関係者の調整','決まったことと保留事項が混ざってしまう','どこで認識がずれているか'],
          ['チームの相談','困りごとが、問題になってから分かる','どの場面なら相談しやすいか']])+note('メンバー個人を自動採点する題材より、仕事の状況と支援の必要性を共有する題材を推奨する。'),
      notes='PM・PLの仕事を、文書作成だけに狭めないための補足です。進捗把握には情報収集、解釈、支援判断が含まれます。コミュニケーション活性化も、発言数を増やすことに固定せず、必要な相手に相談しやすくなったかを確認します。これらは運営が用意した題材の候補で、受講者には課題選択後に必要なヒントだけを渡します。'),
 dict(label='付録｜利用環境', title='利用環境：Kiroが主、Claudeは補助',
      intro='Kiroを共通の実行環境にする。Claudeを使えない参加者も、必須課題を完了できる構成にする。',
      body=table(['主環境：Kiro','補助環境：Claude'],[
          ['エージェントの設定・実行','利用可能な範囲での別視点からの検討'],['Skillsの作成・利用','文章や説明の別視点での確認'],['成果物の試行と改善','任意の補助作業']])+ '<p class="ws-source">開始前に確認：社内のIDE／CLIの版、利用枠、ファイル・コマンドの権限。<br>機能の根拠：<a href="https://kiro.dev/docs/skills/">Kiro Agent Skills</a> ／ <a href="https://kiro.dev/docs/cli/custom-agents/creating/">Creating custom agents</a>（2026-09-05確認）。料金：<a href="https://kiro.dev/pricing/">Kiro Pricing</a>（2026-09-21確認）。Kiro Pro は1人 $20／月で1,000クレジット、超過は $0.04／クレジット。<br>社内Kiro環境での動作確認は9月中に実施する。状態は{{枚:前提条件と未確定事項}}枚目に記載する。</p>',
      notes='Kiro公式ドキュメントではAgent Skillsとカスタムエージェントの作成方法が説明されています。ただし社内で導入済みの版と設定は未確認です。初回教材を収録する前に参加者と同じ環境で、作成、読み込み、実行、結果確認までを試します。教材は概念編と操作編に分け、機能更新時は操作編だけを差し替えられるようにします。')
]



PURPOSE = {'参加者情報・利用頻度': '利用頻度 F1。実務でAIを使う頻度の変化を測定する。部長報告の「実務での利用」に使用する。', '共通アンケート①': '能力 C1〜C4。課題の選択と、AIへの依頼に関する変化を測定する。', '共通アンケート②': '能力 C5〜C7。説明・確認・改善に関する変化を測定する。', 'PM・PLの業務': '業務別 P1〜P6。どの管理業務で実務利用が生じたかを確認する。補助指標の取得元。', '開発者の業務': '業務別 D1〜D6。どの開発工程で実務利用が生じたかを確認する。補助指標の取得元。', '個人目標': '個人目標 G1〜G4。開始時に達成条件を設定し、終了時の照合に使用する。補助指標の取得元。', '困りごと・支援': '困りごと N1〜N4。必要な支援を把握し、教材と相談の調整に使用する。成果指標には含めない。', '終了時・作者': '終了時 R1〜R6。作ったものと、使ってもらった結果を記録する。主KPIの取得元。',  '成果物の登録': '成果物の登録 W1〜W5。作成した仕組みを1件ずつ登録する。試用フォームの選択肢になる。'}

def _polite(s):
    for a, b in (('測定する。','測定します。'),('確認する。','確認します。'),('使用する。','使用します。'),
                 ('記録する。','記録します。'),('含めない。','含めません。'),('選択肢になる。','選択肢になります。'),
                 ('登録する。','登録します。'),('取得元。','取得元です。'),('使う。','使います。')):
        s = s.replace(a, b)
    return s

def form_slide(label, title, rows, intro=None, foot='', cls='ws-form'):
    purpose = PURPOSE.get(label, '')
    intro = intro or purpose or '回答用フォームのイメージ。枠内に記入し、各設問の指定に従って選ぶ。'
    return dict(label='付録｜'+label,title=title,intro=intro,cls=cls,body=table(['質問・項目','回答欄'],rows)+note(foot),notes='このページは「'+label+'」の回答イメージです。'+_polite(purpose)+'左に質問、右に回答欄を示しています。実際のフォームは別途作成し、参加者に近い人に試してもらってから確定します。')

SLIDES.extend([
 dict(label='主KPI',title='主KPI：他の人に役立つ仕組みを作れた人の割合',cls='ws-diagram',
 intro='分母は開始時の参加者数、分子は「使ってもらい、役立った」と報告できた人数。右がその設問である。',
 body=visuals.kpi_main()+note('デモの閲覧だけでは数えない。本人の報告を、運営がデモと成果物で確認する。未提出・未確認も分母に残し、内訳を示す。'),
 notes='主KPIは1つです。他の人に使ってもらい、役立ったと報告できた参加者の割合です。分母は開始時の参加者数15名で固定します。分子は、右に示した2つの設問から取ります。R3で誰に使ってもらったかを聞き、R4でその人の反応を聞きます。R4で「役立った」または「一部役立った」を選んだ人を、分子に数えます。人数単位で数え、1人が複数作った場合も1人として数えます。デモを閲覧しただけの相手は数えません。本人の報告は、運営がデモと成果物で確認します。未提出や未確認も分母から外さず、内訳として示します。目標値は開始時の参加者の6割を暫定とし、事前調査を踏まえて確定します。'),
 form_slide('参加者情報・利用頻度','回答フォーム：参加者情報と利用頻度',[
 ['回答者名（自動）','Formsが組織アカウントから記録する。入力欄は設けない'],
 ['担当する業務（複数選択可）','□PM・PL　□開発　□その他：＿＿＿'],
 ['過去4週間に実務でAIを使った日数','○0日　○1〜3日　○4〜7日　○8〜15日　○16日以上'],
 ['実務の機会がない場合','○実務の機会なし（上の日数の代わりに選択）']],foot='研修だけで使った日は除く。事前・中間・終了時はフォームを分け、回答者名で突合する。個別回答は運営が確認する。'),
 form_slide('共通アンケート①','回答フォーム：共通アンケート①',[[f'C{i+1}　'+q,'○0　○1　○2　○3　○4　○機会なし'] for i,q in enumerate(visuals.CORE_QUESTIONS[:4])],foot='0 まだやり方が分からない ／ 1 やり方は分かるが、まだできない ／ 2 手助けがあればできる<br>3 一人でできる ／ 4 状況に合わせてやり方を変えられる ／ 機会なし 試す機会がない'),
 form_slide('共通アンケート②','回答フォーム：共通アンケート②',[[f'C{i+5}　'+q,'○0　○1　○2　○3　○4　○機会なし'] for i,q in enumerate(visuals.CORE_QUESTIONS[4:])]+[['中間・終了時：具体例を一つ記入','できたこと・まだ難しいこと：＿＿＿＿']],foot='0 まだやり方が分からない ／ 1 やり方は分かるが、まだできない ／ 2 手助けがあればできる<br>3 一人でできる ／ 4 状況に合わせてやり方を変えられる ／ 機会なし 試す機会がない'),
 form_slide('PM・PLの業務','回答フォーム：PM・PLの業務',[[f'P{i+1}　'+q,'○0　○1　○2　○3　○担当外　○機会なし'] for i,q in enumerate(visuals.PM_TASKS)],foot='過去4週間について、各行で一つ選択。0 未利用 ／ 1 試用のみ ／ 2 実務で1回 ／ 3 実務で複数回。実務利用は結果を人が確認したもの。'),
 form_slide('開発者の業務','回答フォーム：開発者の業務',[[f'D{i+1}　'+q,'○0　○1　○2　○3　○担当外　○機会なし'] for i,q in enumerate(visuals.DEV_TASKS)],foot='過去4週間について、各行で一つ選択。0 未利用 ／ 1 試用のみ ／ 2 実務で1回 ／ 3 実務で複数回。兼務者は両方に回答。'),
 form_slide('個人目標','回答フォーム：個人目標',[
 ['G1　誰の、どんな課題ですか。','対象者・困っている場面：＿＿＿＿'],
 ['G2　どんな仕組みをAIで作りますか。','作るもの・使う知識や手順：＿＿＿＿'],
 ['G3　誰に使ってもらいますか。','利用者・試してもらう場面：＿＿＿＿'],
 ['G4　何が変われば達成ですか。','確認する結果・方法・期限：＿＿＿＿']],foot='開始時に記入し、中間で再確認する。変更前の目標と変更理由も残す。'),
 form_slide('困りごと・支援','回答フォーム：困りごと・支援',[
 ['N1　困っていること（最大3つ）','□使いどころ　□依頼方法　□結果の確認　□共有　□環境　□時間　□なし　□その他'],
 ['N2　AIを使いたい業務（最大2つ）','P／Dの項目：＿＿　□その他　□まだ思いつかない'],
 ['N3　どんな場面で困っていますか。','具体的な場面：＿＿＿＿'],
 ['N4　欲しい支援（最大2つ）','□概念動画　□操作説明　□事例　□ブログ　□Teams相談　□会話　□環境　□時間　□その他']],foot='回答は支援の調整に使う。具体的な案件名は記入不要。'),
 form_slide('終了時・作者','回答フォーム：終了時の振り返り',[
 ['R1　目標に対する結果はどうでしたか。','○達成　○一部達成　○未達　○未実施'],
 ['R2　何を作りましたか。','名称と、組み込んだ知識・手順（最大3件）：＿＿'],
 ['R3　誰に使ってもらいましたか。','○同僚・チーム　○他の参加者　○まだ　（相手と場面：＿＿）'],
 ['R4　使った人の反応はどうでしたか。','○役立った　○一部役立った　○役立たなかった　（反応と改善点：＿＿）'],
 ['R5　うまくいかなかった点は何ですか。','□導入　□入力準備　□業務との不一致　□確認の手間　□なし'],
 ['R6　今後必要なことは何ですか。','未解決の点・続けたいこと・必要な支援：＿＿']],foot='使ってもらった相手が回答する欄は設けない。本人の報告を、運営がデモと成果物で確認する。',cls='ws-form ws-tight'),
 dict(label='アンケートの取り方',title='アンケートの取り方：Microsoft Forms 3本',cls='ws-diagram',
 intro='回答するのは参加者だけである。事前・中間・終了時の3本に分け、同じ人の回答を突き合わせる。',
 body=visuals.forms_timeline()
  +note('回答者名の記入欄は設けない。「1人1回」は1つのフォームにつき1回のため、時点ごとにフォームを分ける。回答フォームのイメージは付録に置く。'),
 notes='アンケートの取り方です。Microsoft Formsで実装し、事前・中間・終了時の3本に分けます。回答するのは参加者だけです。作ったものを使ってもらった相手には配りません。相手に回答を依頼できないためです。回答者名はFormsが組織アカウントから記録するので、氏名や番号の記入欄は設けません。この回答者名で、3時点の回答を突き合わせます。「1人1回」の設定が制限するのは1つのフォームにつき1回の回答なので、時点ごとにフォームを分けます。回答フォームのイメージは付録に置いています。')
])


# 提案：旧10・旧11の統合、前提条件の新設、付録の索引、本編の並び替え
SHARING = dict(label='共有の2回', title='共有：12月と3月の2回', cls='ws-diagram',
  intro='目的を分けて開催する。12月は途中経過の共有、3月は成果物の公開と交流。',
  body=visuals.sharing_two()
   +note('発表用の動画制作は不要。1人5分＋会話3分で15名なら約120分を見込み、必要に応じて会を分割する。人気投票・表彰は交流の企画として検討する。'),
  notes='共有の機会は12月と3月の2回です。12月はTeamsのWeb会議で、画面共有により途中の成果物を動かし、判断に迷っている点を話します。発表用動画の制作は求めません。1人5分と会話3分で、15名の場合は約120分を見込みます。3月はオフラインの交流会兼成果物発表会です。軽食を囲んで短いデモを紹介し、希望者へ仕組みを共有します。その場で試せないものは、後日1週間の試用期間へつなげます。人気投票と表彰は、交流を楽しむ企画として検討します。')

PREMISE = dict(label='前提条件と未確定事項', title='前提条件：決定済みと未確定', cls='ws-form',
  intro='開催条件は決定済みで、それ以外は本日の了承後に確定する。',
  body=table(['項目','状態','確定の時期'],[
    ['開催条件（10月開始・下期・Kiro主・約15名・Teams）','決定済みの条件','確定済み'],
    ['社内Kiro環境の動作','未検証','9月中に確認する'],
    ['概念教材6本','構成案のみ、未制作','10月中に1本目を用意する'],
    ['運営担当と工数','未確保','合意の後に社内で調整する'],
    ['利用環境と交流会の費用','Kiro Proは1人 $20／月','9月・1月に確認'],
    ['主KPIの目標値','未設定','事前調査の後に設定する']])
   +note('ヘッダに「企画案」と記載した枚は、今回の提案である。上表の条件は、開催前から決まっているものである。'),
  notes='本日の説明で、決まっている条件と、まだ決まっていない事項を区別します。開催条件は、開催前から決まっているものです。社内Kiro環境の動作確認は9月中に実施します。概念教材は構成案までで、1本目は10月中に用意します。運営担当と工数、利用環境と交流会の費用、主KPIの目標値は、本日の了承のあとに確定します。')

APPENDIX_INDEX = dict(label='付録｜案内', title='付録の構成', cls='ws-form',
  intro='題材・環境・設問・回答フォームの4つで構成する。指標と主要な設問は本編にあり、付録は全文と体裁を示す。',
  body=table(['付録','内容'],[
    ['{{枚:付録｜PM・PLのテーマ}}〜{{枚:付録｜開発者のテーマ}}','職種ごとの題材の候補'],
    ['{{枚:付録｜利用環境}}','利用環境（Kiro・Claude）'],
    ['{{枚:付録｜設問の構成}}・{{枚:付録｜職種別の設問}}','設問の区分と、職種別の設問の全文'],
    ['{{枚:付録｜参加者情報・利用頻度}}〜{{枚:付録｜終了時・作者}}','回答フォームのイメージ']])
   +note('付録は質疑への応答用である。設問の正本はアンケート・効果測定案が保持する。'),
  notes='ここから付録です。付録は4つに分かれます。職種ごとの題材の候補、利用環境、設問の区分と職種別の設問の全文、回答フォームのイメージです。同じ体裁のフォームが続くため、必要な設問群から参照してください。')


KPI_SUB = dict(label='補助の指標', title='補助の指標：業務別の活用と、個人目標', cls='ws-diagram',
  intro='主KPIだけでは、作るところまで到達しなかった人の変化が残らない。補助の2つで、そこを拾う。',
  body=visuals.kpi_sub()
   +note('0〜3と能力の0〜4は合算しない。「担当外」「機会なし」は0点へ変換せず、別に扱う。'),
  notes='補助の指標は2つです。1つ目は業務別のAI活用です。過去4週間に、担当する業務でAIをどの程度使ったかを、0から3の4段階で聞きます。0が未利用、1が試用のみ、2が実務で1回、3が実務で複数回です。実務利用にあたる2と3を選んだ人数が、事前から終了時にかけてどう変化したかを見ます。2つ目は個人目標の達成です。開始時にG4で達成条件を書いてもらい、終了時にR1でその結果を聞きます。開始時の条件と終了時の結果を照合します。この2つがあるため、作るところまで到達しなかった人の変化も記録に残ります。担当していない業務と、担当しているが機会が無かった場合は、0点へ変換せず別に扱います。')

QUESTIONS = dict(label='付録｜設問の構成', title='設問の構成：6つの区分', cls='ws-form ws-q4',
  intro='本編で示した指標を、設問の区分の側から一覧にしたものである。問数と尺度を確認するために置く。',
  body=table(['設問の区分','答える問い','尺度','使い道'],[
    ['利用頻度 F1（1問）','実務での利用頻度が変わったか','日数の6区分','実務での利用'],
    ['能力 C1〜C7（7問）','自力でできる範囲が広がったか','0〜4','自力でできる範囲'],
    ['業務別 P・D（各6問）','どの業務で実務利用が生じたか','0〜3＋担当外','業務別の活用'],
    ['個人目標 G（4問）','達成条件を満たしたか','記述','目標の達成'],
    ['終了時 R（6問）','作ったものが他の人に役立ったか','選択と記述','主KPI'],
    ['困りごと N（4問）','どの支援が要るか','選択','運営の改善']])
   +note('0〜4と0〜3は合算しない。「担当外」「機会なし」は0点へ変換しない。回答フォームは付録{{枚:付録｜参加者情報・利用頻度}}〜{{枚:付録｜終了時・作者}}に置く。'),
  notes='付録です。設問は6つの群に分かれます。利用頻度は実務でAIを使う頻度の変化、能力は自力でできる範囲の変化を見ます。業務別は、どの業務で実務利用が生じたかを職種ごとに聞きます。個人目標は開始時の達成条件と終了時の結果を照合します。試用記録は、作者以外が使って役立ったかを聞くもので、主KPIの取得元です。困りごとは支援の調整に使い、成果指標には含めません。回答フォームのイメージは付録に置いています。')


DEV_THEME = dict(label='付録｜開発者のテーマ', title='開発者の題材：手間と着手の重さ',
  intro='用途のヒント。参加者は、実際に経験した場面から自分の課題を選ぶ。',
  body=table(['困りごとの種類','よくある場面','参加者が確認すること'],[
    ['開発プロセス','毎回の確認や引き継ぎで同じ手間が生まれる','どの手間が繰り返すか'],
    ['Webアプリの着手','試し始めるまでに時間がかかる','何が揃えば着手できるか'],
    ['オンボーディング','新しい参加者が判断の根拠を探し回る','どの情報が見つからないか']])
   +note('完成例や推奨構成は指定しない。既存のSkillの改良も認める。'),
  notes='開発者向けの題材の候補です。開発プロセスでは、毎回の確認や引き継ぎで同じ手間が生まれる場面を扱います。Webアプリの着手では、アイデアがあっても試し始めるまでに時間がかかる場面を扱います。オンボーディングでは、新しく参加した人が判断の根拠を探し回る場面を扱います。完成例や推奨構成は指定しません。これらは運営が用意した候補で、受講者には課題選択後に必要なヒントだけを渡します。')


# 期待する効果と、終了時の判断
IMPACT = dict(label='期待する効果', title='期待する効果：各案件でAIの活用が進む', cls='ws-diagram',
  intro='参加者が自分の案件でAIを使い、同僚がその仕組みを使う。案件の進め方が変わる。',
  body=visuals.promotion()
   +note('15名が、それぞれの案件で使える仕組みを1つ以上持つ。使った記録は、次の案件の材料になる。'),
  notes='この企画で期待しているのは、それぞれの案件でAIの活用が進むことです。いまは、AIを使う人が案件の中で限られ、使い方も個人の中に閉じています。試した結果が案件に残らないため、次の人が同じところからやり直します。この企画のあとは、案件ごとにAIで作った仕組みが動き、同僚がその仕組みを使えます。使った記録も残るため、次の案件へ渡せます。15名がそれぞれの案件で、使える仕組みを1つ以上持つ状態を目指します。')

NEXT_STEP = dict(label='次の期の扱い', title='次の期の扱い：3月の結果で決まる3つ', cls='ws-diagram',
  intro='3月の結果を見て、次の期の扱いを決める。3つのどれになるかで、次にすることが変わる。',
  body=visuals.next_period()
   +note('どの結果でも記録は残す。うまくいかなかった理由は、次の設計の材料になる。'),
  notes='3月の結果を見て、次の期の扱いを決めます。結果によって、次にすることが変わります。広げる、発想を鍛える、支援を補うの3つです。次の3枚で、それぞれの中身をご説明します。他の人に使われた仕組みが複数出た場合は、共有リポジトリへ集約し、次の期は対象を広げます。作れたけれども使われなかった場合は、定期的なAIアイデアソンを組み込みます。作るところまで到達しなかった場合は、教材・時間・相談の不足を特定し、条件を変えます。どの結果でも記録は残します。うまくいかなかった理由は、次の設計の材料になるためです。')

HORIZON = dict(label='広げる', title='広げる：ナレッジを個人で閉じさせない', cls='ws-diagram',
  intro='他の人に使われた仕組みが複数出た場合である。各自に貯まったナレッジを、部署を越えて使える形にする。',
  body=visuals.horizon()
   +note('置き場所と名前の付け方を決め、使い方の記録を添える。置き場は目的ではなく、力を持つ人が部署を越えて増えることが目的である。'),
  notes='1つ目、他の人に使われた仕組みが複数出た場合です。この場合の展望をご説明します。AI人材が育つと、それぞれにAIのナレッジが貯まります。SkillsやAgents、業務で使う小さなToolです。しかし、そのままでは個人の中に留まり、他の人は同じものを一から作り直します。そこで、共有リポジトリを1か所に用意して集約することを提案します。第1案は、部署または本部の共有GitHubです。本部との調整が難航する場合は、部内のAWS CodeCommitを共有リポジトリとします。どちらの場合も、置き場所と名前の付け方を決め、使い方の記録を添えます。そうすれば、他部署の案件でも顧客の案件でも取り出して使えます。ここで重要なのは、置き場そのものではありません。使い方の記録が添えてあれば、取り出した人は、誰がどの課題をどう解決したかを読み取れます。つまり、使った人が次の作り手になります。先ほど申し上げた、課題をAIで解決する力を持つ人が、部署を越えて増えるということです。置き場と運用の決め方は、この企画の範囲外です。次の期に、全社でのAI活用の共有方法として、別途ご提案します。')

REBUILD = dict(label='発想を鍛える', title='発想を鍛える：定期的なAIアイデアソンを組み込む', cls='ws-diagram',
  intro='作れたが、使われなかった場合である。AIの使い方は、0から1を作る発想力で決まる。その発想を出す場を定期的に置く。',
  body=visuals.branch_rebuild()
   +note('作る競技ではなく、課題と使い道を出し合う場にする。開催の間隔は隔月1回を案とし、次の期の設計で確定する。'),
  notes='2つ目、作れたけれども使われなかった場合です。この場合、原因は作る力ではありません。課題を見つける発想の側にあります。AIの使い方は、0から1を作る発想力で決まるためです。左が、1回だけ作る形で起きることです。作ること自体が目的になり、課題は各自が抱え、使ってもらう相手が決まりません。ハッカソンのように作る競技にしても、ここは変わりません。次の期は、定期的なAIアイデアソンを組み込みます。困りごとを持ち寄り、AIでどう解くかを出し合い、その場で使ってもらう相手と場面を決めます。小さく作り、次の回に結果を持ち寄ります。1回で終了させず、定期的に実施することが重要です。開催の間隔は隔月1回を案とし、次の期の設計で確定します。')

STOP = dict(label='支援を補う', title='支援を補う：つまずいた場所を特定し、条件を変える', cls='ws-diagram',
  intro='作るところまで到達しなかった場合である。どこで止まったかを切り分け、条件を変えて再実施する。',
  body=visuals.branch_stop()
   +note('切り分けには、困りごとの設問（どの支援が要るか）と、中間の回答を使う。改善の見込みが無い場合に限り停止し、その理由も記録に残す。'),
  notes='3つ目、作るところまで到達しなかった場合です。ここで決めるのは、まず改善の方法です。どこで止まったかを3つで切り分けます。教材が届かなかったのか、時間が取れなかったのか、相談が届かなかったのかです。切り分けには、困りごとの設問と中間アンケートの回答を使います。改善できる場合は、条件を変えて次の期に再実施します。たとえば教材の順序を変える、時間の確保を上長と調整する、相談の場を業務時間内に置くといった変更です。改善できる見込みが無い場合は、次の期で止めます。その場合も、止めた理由は記録に残します。次に全社でAIの活用を進めるときの材料になるためです。')

ESSENCE = dict(label='この企画の本質', title='本質：顧客先でも価値を生み続けられる人材を増やす', cls='ws-diagram',
  intro='案件が終わると、作った仕組みはその案件に残る。持ち運べるのは、課題をAIで解決する力である。',
  body=visuals.same_loop()
   +note('価値は、作った仕組みの数ではない。どの案件でも課題を解決できる人が増えることである。'),
  notes='ここがこの企画の本質です。案件でAIの仕組みを作ると、その仕組みは案件の中に残ります。しかし案件が終われば、そこまでです。持ち運べるのは、課題の本質を捉えてAIで解決する力のほうです。この力は人に残るため、自社の次の案件でも、顧客の案件でも同じように使えます。だからこの企画が増やそうとしているのは、成果物の数ではなく、その力を持つ人です。下期の15名が、その最初の一歩です。')

ORDER = [0, 1, 4, 2, ESSENCE, 3, 5, 6, 7, 8, SHARING, IMPACT, 11, 17, KPI_SUB, 12, 26, NEXT_STEP, HORIZON, REBUILD, STOP, PREMISE, 14,
         APPENDIX_INDEX, 15, DEV_THEME, 16, QUESTIONS, 13,
         18, 19, 20, 21, 22, 23, 24, 25]
SLIDES = [SLIDES[i] if isinstance(i, int) else i for i in ORDER]

def _resolve(text):
    """{{枚:ラベル}} を、そのラベルの枚番号へ置き換える。並べ替えても壊れない。"""
    import re as _re
    def sub(m):
        label = m.group(1)
        for i, s in enumerate(SLIDES, 1):
            if s['label'] == label:
                return str(i)
        raise KeyError('枚が無い: ' + label)
    return _re.sub(r'\{\{枚:([^}]+)\}\}', sub, text)

for _s in SLIDES:
    for _k in ('title', 'intro', 'body', 'notes'):
        if _k in _s:
            _s[_k] = _resolve(_s[_k])

CSS = '''
:root{--ground:#f4f1e8;--surface:#e9e5da;--surface-2:#e1dfd3;--paper:#fffaf2;--ink:#252b29;--dim:#59605a;--faint:#686f65;--line:#d5d6c9;--accent:#b64326;--accent-dim:#873f2b;--serif:'Noto Sans JP','Yu Gothic',sans-serif;--sans:'Noto Sans JP','Hiragino Sans','Yu Gothic',sans-serif;--mono:var(--sans)}
body{font-feature-settings:normal;background:#deded3}
.stage{background:var(--ground)}
.slide{padding:84px 84px 78px}
.kicker{font-family:var(--sans);font-size:14px;letter-spacing:.06em;color:var(--dim);margin-bottom:20px;gap:12px}
.kicker::after{display:none}
.ws-mark{display:inline-block;width:12px;height:12px;border:2px solid #6d766a;border-radius:50%;position:relative}
.ws-mark::after{content:'';position:absolute;left:16px;top:3px;width:13px;height:2px;background:#b7bcae}
.ws-kicker-label{padding-left:17px}
h1,h2{font-family:var(--sans);font-weight:700;text-wrap:initial;letter-spacing:-.03em}
h1{font-size:51px;line-height:1.43} h2{font-size:36px;line-height:1.46;margin-bottom:16px}
.ws-intro{font-size:20px;line-height:1.7;max-width:none;color:var(--dim)}
.body{justify-content:flex-start;padding-top:35px;gap:16px;min-height:0}
.ws-cover .body{padding-top:29px}
.ws-cover .ws-intro{font-size:19px;margin-top:14px}
.ws-cover h2{font-size:47px;line-height:1.43}
.ws-titlepage{padding-top:215px}
.ws-titlepage .kicker{display:none}
.ws-titlepage h1{font-size:61px;line-height:1.4}
.ws-titlepage .ws-intro{font-size:21px;margin-top:19px}
.ws-titlepage .body{padding-top:48px}
.ws-title-art{width:100%;height:130px;display:block;flex:none}
.ws-abstraction .body{padding-top:20px}
.ws-abstraction-fig{width:100%;height:auto;display:block;flex:none;font-family:var(--sans)}
.ws-flow{width:100%;height:auto;max-height:184px;display:block;flex:none;font-family:var(--sans)}
.ws-note{font-size:16px;line-height:1.8;max-width:none;color:var(--dim);margin-top:8px}
.ws-table{font-family:var(--sans);width:100%;font-size:20px;border-collapse:collapse;table-layout:fixed}
.ws-table th{font-family:var(--sans);font-size:15px;color:var(--dim);font-weight:500;letter-spacing:0;text-transform:none;padding:0 16px 15px;border-bottom:1px solid #b8bdb0;line-height:1.5}
.ws-table td{padding:19px 16px;color:var(--ink);border-bottom:1px solid var(--line);font-size:19px;line-height:1.65}
.ws-table td:first-child{font-weight:500}.ws-table b{color:var(--accent)}
.ws-split{display:grid;grid-template-columns:1.06fr 1fr;gap:54px;align-items:start}
.ws-split .ws-flow{height:165px;max-height:none;margin-top:24px}
.ws-split .ws-table td{font-size:18px;padding:16px 10px}.ws-split .ws-table th{padding-left:10px;padding-right:10px}
.ws-skill-link{display:flex;justify-content:center;align-items:center;gap:20px;font-size:20px;padding:8px 0}
.ws-pill{display:inline-block;padding:10px 24px;border:1px solid #939d8b;border-radius:30px;font-size:17px;color:var(--dim)}
.ws-eyebrow{font-size:16px;margin-bottom:18px;color:var(--dim)}
.ws-copy{font-size:22px;line-height:1.8;max-width:none;color:var(--ink);margin-top:14px}
.ws-phases{display:grid;grid-template-columns:repeat(3,1fr);gap:30px;position:relative;padding-top:16px}
.ws-phases>div{position:relative;padding:29px 25px;background:#e9e5da;border-radius:8px}
.ws-phase-path{position:absolute;left:0;top:16px;width:100%;height:240px;pointer-events:none}
.ws-phases>.ws-phase-hot{background:var(--accent);color:#fffaf2}
.ws-date{font-size:18px;color:var(--dim);margin-bottom:21px}
.ws-phases h3{font-size:23px;margin-bottom:16px}.ws-phases p:not(.ws-date){font-size:18px;line-height:1.9;max-width:none}
.ws-phase-hot p{color:#fff2e7}.ws-phase-hot h3{color:#fffaf2}
.ws-groups{width:100%;height:258px;display:block}
.ws-plain-stack{display:flex;flex-direction:column;gap:37px;padding-top:19px}
.ws-plain-stack h3{font-size:25px;margin-bottom:10px}.ws-plain-stack p{font-size:20px;line-height:1.85}
.ws-hours{display:flex;align-items:baseline;gap:18px;padding-top:3px}.ws-hours span{font-size:76px;line-height:1.3;color:var(--accent);font-weight:700;letter-spacing:-.05em}.ws-hours small{font-size:22px}
.ws-source{font-size:14px;line-height:1.8;color:var(--dim);max-width:none;margin-top:16px}
.ws-source a{color:inherit;text-decoration:underline;text-underline-offset:3px;position:relative;z-index:6}
.ws-form .body{padding-top:18px;gap:10px}.ws-form .ws-table td{font-size:17px;padding:10px 12px;line-height:1.55}.ws-form .ws-table th{padding-bottom:10px}.ws-form .ws-note{font-size:14px;line-height:1.6}.ws-form .ws-table th:first-child{width:48%}
.ws-q4 .ws-table th:first-child{width:26%}
.ws-tight .ws-table td{font-size:16px;padding:8px 12px;line-height:1.5}.ws-q4 .ws-table td{font-size:16px;padding:9px 10px}
.ws-eval .ws-table td{padding-top:13px;padding-bottom:13px}
.ws-eval .ws-table th:first-child{width:25%}
.ws-survey-core .ws-table th:first-child,.ws-survey-pm .ws-table th:first-child{width:18%}
.ws-survey-core .ws-table td{font-size:18px;padding:12px;line-height:1.65}
.ws-survey-pm .body{padding-top:20px;gap:10px}
.ws-survey-pm .ws-table td{font-size:18px;padding:6px 12px;line-height:1.6}
.ws-survey-core .ws-note,.ws-survey-pm .ws-note{font-size:15px;line-height:1.8}
.ws-sharing .ws-table th:first-child{width:30%}
.ws-visual{width:100%;height:auto;display:block;flex:none;font-family:var(--sans)}
.ws-diagram .body{padding-top:18px;gap:8px}
.ws-diagram .ws-note{font-size:15px;line-height:1.6;margin-top:0}
.chrome{font-family:var(--sans);font-size:12px;letter-spacing:0;height:48px;padding:0 42px;color:#6c7468}
.hint{opacity:1}.progress{height:2px;background:#adb4a3}
.ws-nav{background:none;border:0;color:inherit;font:inherit;cursor:pointer;pointer-events:auto;padding:7px 15px}
.navzone{width:6%;opacity:0}
@media print{
 @page{size:1280px 720px;margin:0}
 html,body{height:auto;overflow:visible;background:var(--ground)}
 .deck{position:static;overflow:visible}.stage{position:static;width:1280px;height:auto;transform:none!important}
 .slide{position:relative;display:flex!important;height:720px;break-after:page;animation:none!important}
 .chrome,.progress,.navzone{display:none}
}
'''

FLAT_CSS = '''
<style>
/* 閲覧用：全枚を縦に並べる。印刷用の指定を画面へ当てている */
html,body{height:auto;overflow:visible;background:#deded3}
.deck{position:static;overflow:visible}
.stage{position:static;width:1280px;height:auto;transform:none!important;margin:0 auto}
.slide{position:relative;display:flex!important;height:720px;animation:none!important;border-bottom:1px solid #c9c9bd}
.chrome,.progress,.navzone{display:none}
</style>'''

def build():
    template = TEMPLATE.read_text()
    prefix = template.split('<!-- 01 -->')[0]
    prefix = re.sub(r'<link[^>]+>\s*', '', prefix)
    prefix = prefix.replace('<title>デッキの題名</title>', '<!doctype html>\n<html lang="ja"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>下期AI活用ワークショップ</title>')
    prefix = prefix.replace('<div class="navzone prev"', '<style>'+CSS+'</style></head><body>\n<div class="navzone prev"', 1)
    slides=[]
    for i, s in enumerate(SLIDES,1):
        heading='h1' if i==1 else 'h2'
        slides.append(f'<section class="slide {s.get("cls", "")}" aria-label="{i}: {esc(re.sub("<[^>]*>", "", s["title"]))}">\n'
            f'<div class="kicker"><span class="ws-mark" aria-hidden="true"></span><span class="ws-kicker-label">{esc(s["label"])}　／　企画案</span></div>\n'
            f'<{heading}>{s["title"]}</{heading}>\n<p class="ws-intro">{esc(s["intro"])}</p>\n<div class="body">{s["body"]}</div>\n</section>')
    suffix=template[template.index('<div class="chrome">'):]
    labels=json.dumps([s['label'] for s in SLIDES],ensure_ascii=False)
    suffix=re.sub(r'const LABELS = .*?;',f'const LABELS = {labels};',suffix)
    suffix=suffix.replace('<span class="hint">← → でめくる</span>','<span class="hint"><button class="ws-nav" id="ws-prev" aria-label="前のスライド">←</button>矢印キーでめくる<button class="ws-nav" id="ws-next" aria-label="次のスライド">→</button></span>')
    suffix=suffix.replace("const prev = ()=> show(i-1);", "const prev = ()=> show(i-1);\n  document.getElementById('ws-prev').addEventListener('click', prev);\n  document.getElementById('ws-next').addEventListener('click', next);\n  addEventListener('hashchange',()=>{const n=parseInt(location.hash.slice(1),10)-1;if(Number.isFinite(n)&&n!==i)show(n);});")
    suffix += '\n</body></html>\n'
    (OUT/'director-deck.html').write_text(prefix+'\n'.join(slides)+suffix)
    _apx = sum(1 for s in SLIDES if s['label'].startswith('付録｜'))
    notes=[f'# 部長向け発表原稿\n\n全{len(SLIDES)}枚。表紙1枚、本編{len(SLIDES)-1-_apx}枚、付録{_apx}枚。決定済みの条件と、今回の運営提案を分けて説明します。\n']
    for i,s in enumerate(SLIDES,1):
        notes.append(f'## {i:02d}　{re.sub("<[^>]*>", "", s["title"])}\n\n{s["notes"]}\n')
    (OUT/'speaker-notes.md').write_text('\n'.join(notes))
    # 閲覧用：全枚を縦に並べた1ページ版。めくる操作を外し、見出しと重複する説明を外す
    flat = re.sub(r'<script\b.*?</script>', '', prefix+'\n'.join(slides)+suffix, flags=re.S)
    flat = re.sub(r'(<section class="slide[^"]*") aria-label="[^"]*"', r'\1 aria-label="スライド"', flat)
    flat = re.sub(r'(<svg class="ws-[^"]*"[^>]*?) aria-label="[^"]*"', r'\1', flat)
    flat = flat.replace('</head>', FLAT_CSS + '</head>')
    (OUT/'director-deck-all.html').write_text(flat)
    print(f'Built {len(SLIDES)} slides: {OUT / "director-deck.html"} / {OUT / "director-deck-all.html"}')

if __name__=='__main__':
    build()
