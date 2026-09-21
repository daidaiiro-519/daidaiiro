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
      intro='社内AI人材育成｜非同期・個人制作型',
      body='<svg class="ws-title-art" viewBox="0 0 1112 130" role="img" aria-label="個々の点がつながり、一つの形になる"><g fill="none" stroke="#9aa18f" stroke-width="2"><circle cx="26" cy="65" r="12"/><circle cx="80" cy="34" r="12"/><circle cx="88" cy="99" r="12"/><path d="M143 65 H205 M265 35 L310 94 L355 35 Z"/><circle cx="265" cy="35" r="10" fill="#f4f1e8"/><circle cx="310" cy="94" r="10" fill="#f4f1e8"/><circle cx="355" cy="35" r="10" fill="#f4f1e8"/><path d="M397 65 H459"/></g><path d="M519 23 L603 65 L519 107 L477 65 Z" fill="#b64326"/></svg>',
      notes='下期AI活用ワークショップの企画をご説明します。開発者、PM・PLなどが、それぞれの仕事の中でAIを活用して新しい仕組みを生み出す力を育てる企画です。'),
 dict(label='企画の目的', title='身近な課題を、<br>AIを使った仕組みで解決できる人材を育成する。', cls='ws-cover',
      intro='開発者とPM・PL約15名が、2026年10月から2027年3月に取り組む。',
      body=diagram([('課題を選択する','自分の仕事を観察する'),('小さく作成する','AIで仕組みを用意する'),('改善する','他の人の反応を確認する')],hot=2),
      notes='身近な業務課題から、新しい仕組みをAIで生み出せる人材を育てます。課題の本質を捉え、それを解決する具体的な仕組みへ変えることが狙いです。エージェントやSkillsを使って形にし、実際に試した結果から改善します。'),
 dict(label='育てたい力', title='課題の本質を捉え、具体化して解決する力を<br>共通の土台にする。', cls='ws-abstraction',
      intro='開発者もPM・PLも、抽象と具体を往復する。',
      body='<svg class="ws-abstraction-fig" viewBox="0 0 1112 295" role="img" aria-label="抽象では課題の本質を捉え、具体ではAIで解決の仕組みを作る。試した結果から課題の捉え方を再確認する。"><rect x="0" y="40" width="382" height="177" rx="8" fill="#e9e5da"/><text x="28" y="78" font-size="18" fill="#59605a">抽象</text><text x="28" y="122" font-size="28" font-weight="700" fill="#252b29">課題の本質を捉える</text><text x="28" y="162" font-size="19" fill="#59605a">出来事から、重要な構造や関係を</text><text x="28" y="193" font-size="19" fill="#59605a">取り出す。</text><rect x="730" y="40" width="382" height="177" rx="8" fill="#e9e5da"/><text x="758" y="78" font-size="18" fill="#59605a">具体</text><text x="758" y="122" font-size="27" font-weight="700" fill="#252b29">AIで解決の仕組みを作る</text><text x="758" y="162" font-size="19" fill="#59605a">使う場面と条件に合わせて形にし、</text><text x="758" y="193" font-size="19" fill="#59605a">役に立つかを確認する。</text><text x="556" y="86" font-size="19" text-anchor="middle" fill="#b64326" font-weight="700">具体化する</text><path d="M411 109 H701 m -10 -10 l 10 10 -10 10" stroke="#b64326" stroke-width="3" fill="none"/><text x="556" y="169" font-size="18" text-anchor="middle" fill="#b64326" font-weight="700">結果から捉え直す</text><path d="M701 191 H411 m 10 -10 l -10 10 10 10" stroke="#b64326" stroke-width="3" fill="none"/></svg>',
      notes='共通の土台にしたいのは、課題の本質を抽象的に捉え、それを具体化して解決する力です。ここでいう抽象とは、目的に照らして、個別の出来事から重要な構造や関係を取り出すことです。何を残し何を省くかを明らかにするため、曖昧な表現へ言い換えることとは異なります。具体化では、使う場面や制約に合わせてAIで仕組みを作り、実際に役に立つかを確認します。最初に捉えた本質も仮説なので、試した結果に応じて捉え直します。この抽象と具体の往復を、開発者とPM・PLの共通の土台にします。'),
 dict(label='テーマの設計', title='幅のある領域から、取り組む課題を本人が選択する。',
      intro='運営は困りごとの領域を提示し、解決方法と成果物の形は参加者が選択する。',
      body=table(['開発者向けの領域','PM・PL向けの領域'],[
          ['開発プロセスの手間を削減する','提案準備と関係者調整の手間を削減する'],
          ['Webアプリの作成を効率化する','メンバーの状況把握を支援する'],
          ['新しい参加者の業務開始を支援する','相談と知識共有を促進する']])+note('テーマを絞る問い：誰が、どんな場面で困るか。何が変われば役に立ったと判定できるか。'),
      notes='職種ごとに完成品を指定するのではなく、取り組む領域を選べるようにします。たとえばPM・PLなら、資料を作ることだけでなく、判断材料を集める、認識のずれを見つける、相談の入口を作るといった仕事も対象です。これは用途のヒントであり、推奨する解法の一覧ではありません。参加者自身が経験した場面へ絞り込みます。'),
 dict(label='目指す成果',title='自分の知識と手順をAIの仕組みに組み込み、他の人へ提供する。',intro='知識・ノウハウ・判断の目安を言葉にし、他の人も使える形にする。',body=visuals.handoff()+note('目指す成果：他の人が業務の効率化や課題の改善に使用できる仕組み。提出物は、動く仕組み・使い方・試した記録の3点。'),notes='自分が持つ知識や仕事の進め方を言語化し、AIで動く仕組みに組み込みます。エージェントに仕事を進める役割を与え、Skillsに手順や判断の目安を持たせることで、その知識を他の人も利用できる形にします。本人と同じ熟練を前提にせず、仕組みを通じて業務の効率化や課題改善に役立ててもらうことが目的です。AIの出力を確認する必要はあり、技能全体を完全に置き換えるという意味ではありません。説明や初回の支援があっても構いません。他の人が使った結果と、どんな改善に役立ったかを確認します。',cls='ws-diagram'),
 dict(label='エージェントとSkills', title='エージェントに役割を与え、Skillsで仕事の進め方を渡す。', cls='ws-diagram',
      intro='人が目的と制約を決め、AIが道具を使って実行し、人が結果を確認する。',
      body=visuals.agents()+note('エージェントとSkillsはKiroの標準機能である。環境の確認状況は14枚目に記載する。'),
      notes='前のページで示した、自分のスキルをAIで仕組化して他の人にも届けるという成果を、エージェントとSkillsで実現します。この図は学習のために仕事の関係を整理したものです。カスタムエージェントには役割や使える道具、参照する情報などを設定できます。Skillsは必要な場面で参照する手順や参考資料のまとまりです。すべてを自律実行させる必要はなく、何を任せて何を人が確認するかを決めることを重視します。Kiro公式資料は付録に記載しています。'),
 dict(label='概念教育', title='教材で考える道具を渡し、操作の入口を支える。', cls='ws-diagram',
      intro='音声付きの短いスライド動画を、必要なときに繰り返し視聴できる教材にする。',
      body=visuals.learning()+note('操作の入口を支援し、解決方法は参加者自身が検討する。'),
      notes='主教材は概念中心の動画にし、操作の説明は独立した短い補助教材にします。操作の形式が不明なために着手できない状態は回避します。一方、業務課題の完成例を見せすぎると解法を固定するため、主教材には目的や分担の図を置き、用途のヒントは別途少量だけ公開します。教材はこれから制作する計画で、この提案デッキが受講者向け動画ではありません。'),
 dict(label='下期の進め方', title='半年間で、作成から他の人の利用へ進める。',
      intro='2026年10月〜2027年3月の実施案。月ごとの区切りで、非同期に進める。',
      body='<div class="ws-phases"><svg class="ws-phase-path" viewBox="0 0 1112 240" role="img" aria-label="10・11月の試作から、12・1月の再確認を経て、2・3月の他の人利用へ進む"><path d="M 355 107 h 21 m -6 -6 l 6 6 -6 6 M 736 107 h 21 m -6 -6 l 6 6 -6 6" fill="none" stroke="#747c6e" stroke-width="2"/></svg><div><p class="ws-date">10・11月</p><h3>学習と作成</h3><p>概念教材を視聴する<br>課題を選択し、小さく作成する</p></div><div><p class="ws-date">12・1月</p><h3>共有と修正</h3><p>12月に中間成果を共有する<br>他の人の反応で方向を再確認する</p></div><div class="ws-phase-hot"><p class="ws-date">2・3月</p><h3>他の人の利用</h3><p>他の人が試用し、改善する<br>3月に成果と学びを共有する</p></div></div>'+note('日程は提案。開始前の9月中に、社内Kiro環境での動作確認を完了する。主催者指定の「10月開始・下期」を、上記の期間として計画している。'),
      notes='毎週同じ時間に集まることは前提にしません。10月から翌3月までを提案期間とし、12月に中間共有、3月に最終共有を置きます。初期から試作に触れ、中間発表まで完成を待たないようにします。途中のテーマ変更も、試した結果に理由があれば認めます。休暇や繁忙期を踏まえ、実働20週を工数試算の仮定にしています。'),
 dict(label='サポート体制', title='学習コンテンツと、運営への相談で支援する。', cls='ws-diagram',
      intro='運営に知識が集まっている前提で、参加者へ必要な支援を提供する。',
      body=visuals.support()+note('定例相談会の時間帯は未定。定時後に開催する場合は任意参加とし、Teams投稿でも相談を受け付ける。'),
      notes='参加者同士で専門的なサポートを分担する前提は置きません。ブログや動画で学び、Teamsで主催者に相談できる体制にします。月1回のよろず相談会は開催案です。資料発表の準備を求めず、今の課題や相談したいことをざっくばらんに話す場にします。時間帯はまだ決まっていません。定時後になる場合は任意参加とし、相談内容の要点をTeamsに残します。参加できない人も投稿で同じ支援を受けられるようにします。'),
 dict(label='中間成果物発表', title='中間発表で、次に試すことを見つける。', cls='ws-sharing ws-diagram',
      intro='12月は、途中の成果と判断に迷っている点を共有する。',
      body=visuals.meeting()+note('発表用の動画制作は不要。開催日は調整し、必要に応じて会を分ける。'),
      notes='中間発表はTeamsのWeb会議で行います。参加者が画面共有で途中の成果物を動かし、困っている点を話します。発表用動画の制作は求めません。開催日は調整し、全員が一度に集まれない場合は発表枠を分けます。完成していなくても、動く部分や試した結果があれば共有できます。発表後に中間アンケートへ回答し、次に試すことを記録します。'),
 dict(label='最終成果物発表・交流会',title='気軽に集まり、作った仕組みを見せ合う。',intro='コミュニケーション施策として、軽食を囲むオフライン交流会を開く。',body=diagram([('見せる','短いデモで取り組みを紹介'),('話す・試す','軽食を囲んで気軽に交流'),('つなげる','使いたい人へ仕組みを共有')],hot=1)+note('その場で試せないものは後日試用する。人気投票・表彰は交流を楽しむ企画として検討する。'),notes='最終回は参加者が集まるオフラインの交流会兼成果物発表会にします。軽食を囲み、短いデモを互いに紹介したり、作った工夫や困ったことを話したりできる、堅苦しくない場を目指します。会場、飲食費、日程は今後調整します。その場で試せない成果物は後日の試用につなげます。交流と他の人利用のきっかけを作る場とし、人気投票は任意の企画案として残します。',cls=''),
 dict(label='効果測定の方法',title='主KPIは試用記録、補助指標は同じ人の設問で測定する。',intro='事前・中間・終了時に同じ設問で回答し、成果物の試用記録も集める。',body=visuals.evaluation()+note('設問の区分と、それぞれが答える問いは次の枚に示す。能力の自己評価と実際の利用経験は合算しない。'),notes='主KPIは、自分の知識をAIで動く仕組みにし、他の人が使用して役立ったと確認できた参加者の割合を提案します。開始時の参加者数を分母に固定し、未提出や未確認も明示します。個人目標の達成状況と、業務別のAI活用の変化を補助指標にします。アンケートは同じ人・同じ業務を比較し、回答人数を示します。能力の自己評価と実際の利用経験は合算しません。数値目標は事前調査と運営条件を見て開催前に決めます。',cls='ws-eval ws-diagram'),
 dict(label='付録｜共通アンケートの設問例', title='普段の言葉で、今できることを聞く。', cls='ws-survey-core ws-diagram',
      intro='AIに詳しくない参加者も、自分の経験から答えられる質問にする。',
      body=visuals.questionnaire()+note('共通7問から3問を抜粋。事前・中間・終了時に、同じ質問と選択肢で聞く。'),
      notes='共通項目は、仕事の中でできることを本人の言葉で答えられるようにしています。使いどころ、困りごとの整理、AIへの依頼、複数作業の依頼、使い方の説明、答えの確認、感想をもとにした改善の7問です。回答は0から4の5段階で、スライド右側の選択肢から、今の自分に近いものを選びます。試す機会がない場合も選べます。難しい概念の理解を問う試験にはせず、同じ質問で自己評価の変化を比べます。実際の力は成果物や試用記録も合わせて確認します。開始前に参加者に近い人に回答してもらい、意味が伝わるかを確認します。'),
 dict(label='付録｜PM・PLアンケートの設問',title='PM・PLの業務ごとに、AIの活用状況を聞く。',intro='質問：過去4週間に、次の業務でAIをどの程度使いましたか。',body=visuals.pm_survey()+note('選択肢は開発者も共通。実務で使った場合は、AIの結果を人が確認して利用した経験を答える。'),notes='提供された管理・リーダー業務の例をもとに、情報収集・集計、報告・予実、課題・担当・期限、品質・障害・リスク、改善提案、過去事例の再利用の6項目を聞きます。開発者は要求・仕様、設計、実装、レビュー、テスト、運用・保守について、同じ期間と選択肢で答えます。担当外と実施機会なしを分け、0点に置き換えません。業務ごとに質問は変えますが、利用実績を聞く回答形式は揃えます。',cls='ws-survey-pm ws-diagram'),
 dict(label='部長への依頼',title='運営工数と利用環境費用の承認を依頼する。',intro='参加者は期間中に各自で時間を確保し、課題と制作目標を決定して取り組む。',body=table(['承認を依頼する事項','内訳','提示の時期'],[['運営者の工数','教材24時間＋継続支援60時間＋発表10時間＋相談会9時間＝103時間','本日了承。効果測定分は別途積算'],['利用環境の費用','Kiroの契約・利用枠・追加枠','9月中に見積もり'],['交流会の会場・軽食費','人数と会場の確定後に算定','1月に見積もり']])+note('予算の承認は、運営担当・利用環境・日程が確定した時点で改めて依頼する。実働20週は、休暇と繁忙期を見込んだ試算条件である。'),notes='参加者全員に週何時間という一律の学習時間は設けず、期間中に各自が時間を見つけて進めます。開始時に、どの課題を、どんな仕組みで解決し、誰に使ってもらうかと達成を確認する方法を決めてもらいます。今回了承を得たいのは、教材制作や相談対応などの運営者の工数と、AI利用環境にかかる費用です。工数・契約・追加枠を見積もって提示します。会場と軽食の費用も別途確認します。',cls=''),
 dict(label='付録｜PM・PLのテーマ', title='PM・PL向けには、判断・調整・相談の負荷を題材にする。',
      intro='用途のヒント。参加者は、実際に経験した場面から自分の課題を選ぶ。',
      body=table(['テーマの領域','困りごとの例','参加者が確認すること'],[
          ['判断の準備','提案のたびに、必要な材料を探し直す','何が揃えば判断を始められるか'],
          ['関係者の調整','決まったことと保留事項が混ざってしまう','どこで認識がずれているか'],
          ['チームの相談','困りごとが、問題になってから分かる','どの場面なら相談しやすいか']])+note('メンバー個人を自動採点する題材より、仕事の状況と支援の必要性を共有する題材を推奨する。'),
      notes='PM・PLの仕事を、文書作成だけに狭めないための補足です。進捗把握には情報収集、解釈、支援判断が含まれます。コミュニケーション活性化も、発言数を増やすことに固定せず、必要な相手に相談しやすくなったかを確認します。これらは主催者向けの題材候補で、受講者には課題選択後に必要なヒントだけを渡します。'),
 dict(label='付録｜利用環境', title='Kiroを共通の実行環境にし、Claudeは補助に使う。',
      intro='Claudeを使えない参加者も、必須課題を完了できる構成にする。',
      body=table(['主環境：Kiro','補助環境：Claude'],[
          ['エージェントの設定・実行','利用可能な範囲での別視点からの検討'],['Skillsの作成・利用','文章や説明の別視点での確認'],['成果物の試行と改善','任意の補助作業']])+ '<p class="ws-source">開始前に確認：社内のIDE／CLIの版、利用枠、ファイル・コマンドの権限。<br>機能の根拠：<a href="https://kiro.dev/docs/skills/">Kiro Agent Skills</a> ／ <a href="https://kiro.dev/docs/cli/custom-agents/creating/">Creating custom agents</a>（2026-09-05確認）。<br>社内Kiro環境での動作確認は9月中に実施する。状態は14枚目に記載する。</p>',
      notes='Kiro公式ドキュメントではAgent Skillsとカスタムエージェントの作成方法が説明されています。ただし社内で導入済みの版と設定は未確認です。初回教材を収録する前に参加者と同じ環境で、作成、読み込み、実行、結果確認までを試します。教材は概念編と操作編に分け、機能更新時は操作編だけを差し替えられるようにします。')
]



PURPOSE = {'参加者情報・利用頻度': '利用頻度 F1。実務でAIを使う頻度の変化を測定する。部長報告の「実務での利用」に使用する。', '共通アンケート①': '能力 C1〜C4。課題の選択と、AIへの依頼に関する変化を測定する。', '共通アンケート②': '能力 C5〜C7。説明・確認・改善に関する変化を測定する。', 'PM・PLの業務': '業務別 P1〜P6。どの管理業務で実務利用が生じたかを確認する。補助指標の取得元。', '開発者の業務': '業務別 D1〜D6。どの開発工程で実務利用が生じたかを確認する。補助指標の取得元。', '個人目標': '個人目標 G1〜G4。開始時に達成条件を設定し、終了時の照合に使用する。補助指標の取得元。', '困りごと・支援': '困りごと N1〜N4。必要な支援を把握し、教材と相談の調整に使用する。成果指標には含めない。', '終了時・作者': '個人目標 R1〜R5。達成条件に対する結果と根拠を記録する。補助指標の取得元。', '利用者の試用記録': '試用記録 U1〜U6。他の人の利用結果と、うまくいかなかった点を記録する。主KPIの取得元。', '成果物の登録': '成果物の登録 W1〜W5。作成した仕組みを1件ずつ登録する。試用フォームの選択肢になる。'}

def _polite(s):
    for a, b in (('測定する。','測定します。'),('確認する。','確認します。'),('使用する。','使用します。'),
                 ('記録する。','記録します。'),('含めない。','含めません。'),('選択肢になる。','選択肢になります。'),
                 ('登録する。','登録します。'),('取得元。','取得元です。'),('使う。','使います。')):
        s = s.replace(a, b)
    return s

def form_slide(label, title, rows, intro=None, foot=''):
    purpose = PURPOSE.get(label, '')
    intro = intro or purpose or '回答用フォームのイメージ。枠内に記入し、各設問の指定に従って選ぶ。'
    return dict(label='付録｜'+label,title=title,intro=intro,cls='ws-form',body=table(['質問・項目','回答欄'],rows)+note(foot),notes='このページは「'+label+'」の回答イメージです。'+_polite(purpose)+'左に質問、右に回答欄を示しています。実際のフォームは別途作成し、参加者に近い人に試してもらってから確定します。')

SLIDES.extend([
 dict(label='成果の判定',title='3枚目の成果を、3つの指標で判定する。',intro='指標ごとに、どの設問群から数字を取るかを決めておく。',cls='ws-form',body=table(['指標','数え方・確認方法','取得元の設問区分'],[
 ['主KPI：他の人に役立つ仕組み','他の人が使って役立った仕組みを作成した参加者数／開始時の参加者数','試用記録 U1〜U6'],
 ['補助：業務別のAI活用','同じ人・同じ業務で、実務利用あり（2・3）の人数の変化','業務別 P1〜P6／D1〜D6'],
 ['補助：個人目標の達成','開始時に決めた達成条件と、終了時の結果・根拠を照合','個人目標 G1〜G4・R1〜R5']])+note('人数と分母を併記。配布・試用・実務利用を区別し、人気投票は成果指標に含めない。一人が複数作っても一人として数える。役立たなかった試用も件数で報告する。目標値は開始時の参加者の6割を暫定値とし、事前調査を踏まえて確定する。'),notes='主KPIは人数単位で重複を除きます。他の人の利用記録に用途・結果・役立った点が残ることを条件にし、デモの閲覧だけでは数えません。試用と実務利用の内訳を分けます。未提出・未確認は分母から外さず内訳を示します。共同制作では本人が担当した知識や手順の具体化を確認します。業務活用は前後とも担当・実施機会があり回答した人のみで比較し、職種の総合点や順位を作りません。'),
 form_slide('参加者情報・利用頻度','担当業務と、直近4週間のAI利用状況を確認する。',[
 ['回答者名（自動）','Formsが組織アカウントから記録する。入力欄は設けない'],
 ['担当する業務（複数選択可）','□PM・PL　□開発　□その他：＿＿＿'],
 ['過去4週間に実務でAIを使った日数','○0日　○1〜3日　○4〜7日　○8〜15日　○16日以上'],
 ['実務の機会がない場合','○実務の機会なし（上の日数の代わりに選択）']],foot='研修だけで使った日は除く。事前・中間・終了時はフォームを分け、回答者名で突合する。個別回答は運営が確認する。'),
 form_slide('共通アンケート①','課題を選び、AIへ仕事を伝える力を聞く。',[[f'C{i+1}　'+q,'○0　○1　○2　○3　○4　○機会なし'] for i,q in enumerate(visuals.CORE_QUESTIONS[:4])],foot='0 まだやり方が分からない ／ 1 やり方は分かるが、まだできない ／ 2 手助けがあればできる<br>3 一人でできる ／ 4 状況に合わせてやり方を変えられる ／ 機会なし 試す機会がない'),
 form_slide('共通アンケート②','説明・確認・改善の力を聞く。',[[f'C{i+5}　'+q,'○0　○1　○2　○3　○4　○機会なし'] for i,q in enumerate(visuals.CORE_QUESTIONS[4:])]+[['中間・終了時：具体例を一つ記入','できたこと・まだ難しいこと：＿＿＿＿']],foot='0 まだやり方が分からない ／ 1 やり方は分かるが、まだできない ／ 2 手助けがあればできる<br>3 一人でできる ／ 4 状況に合わせてやり方を変えられる ／ 機会なし 試す機会がない'),
 form_slide('PM・PLの業務','管理業務でのAI活用を回答する。',[[f'P{i+1}　'+q,'○0　○1　○2　○3　○担当外　○機会なし'] for i,q in enumerate(visuals.PM_TASKS)],foot='過去4週間について、各行で一つ選択。0 未利用 ／ 1 試用のみ ／ 2 実務で1回 ／ 3 実務で複数回。実務利用は結果を人が確認したもの。'),
 form_slide('開発者の業務','要求整理から運用・保守まで、AI活用を回答する。',[[f'D{i+1}　'+q,'○0　○1　○2　○3　○担当外　○機会なし'] for i,q in enumerate(visuals.DEV_TASKS)],foot='過去4週間について、各行で一つ選択。0 未利用 ／ 1 試用のみ ／ 2 実務で1回 ／ 3 実務で複数回。兼務者は両方に回答。'),
 form_slide('個人目標','何を作り、誰のどんな課題を改善するかを決める。',[
 ['G1　誰の、どんな課題ですか。','対象者・困っている場面：＿＿＿＿'],
 ['G2　どんな仕組みをAIで作りますか。','作るもの・使う知識や手順：＿＿＿＿'],
 ['G3　誰に使ってもらいますか。','利用者・試してもらう場面：＿＿＿＿'],
 ['G4　何が変われば達成ですか。','確認する結果・方法・期限：＿＿＿＿']],foot='開始時に記入し、中間で再確認する。変更前の目標と変更理由も残す。'),
 form_slide('困りごと・支援','進めるうえで必要な支援を聞く。',[
 ['N1　困っていること（最大3つ）','□使いどころ　□依頼方法　□結果の確認　□共有　□環境　□時間　□なし　□その他'],
 ['N2　AIを使いたい業務（最大2つ）','P／Dの項目：＿＿　□その他　□まだ思いつかない'],
 ['N3　どんな場面で困っていますか。','具体的な場面：＿＿＿＿'],
 ['N4　欲しい支援（最大2つ）','□概念動画　□操作説明　□事例　□ブログ　□Teams相談　□会話　□環境　□時間　□その他']],foot='回答は支援の調整に使う。具体的な案件名は記入不要。'),
 form_slide('終了時・作者','個人目標に対する結果を確認する。',[
 ['R1　目標に対する結果はどうでしたか。','○達成　○一部達成　○未達　○未実施'],
 ['R2　期間中にいくつ作りましたか。','○1件　○2件　○3件以上　○未提出'],
 ['R3　最も使われたものは、何が改善しましたか。','結果・根拠：＿＿　○実測　○体感　○未確認'],
 ['R4　うまくいかなかったものはありますか。','○ある（何が難しかったか：＿＿）　○ない　○未確認'],
 ['R5　今後必要なことは何ですか。','未解決の点・続けたいこと・必要な支援：＿＿']],foot='成果物そのものは「成果物の登録」で1件ずつ登録する。効果がなかった結果も記入する。'),
 form_slide('利用者の試用記録','使った人に、役立った結果とうまくいかなかった点を確認する。',[
 ['U1　どの成果物を使いましたか。','一覧から選択（作者名｜成果物名）'],
 ['U2　何のために使いましたか。','用途・場面：＿＿'],
 ['U3　どこまで使いましたか。','○デモ閲覧のみ　○試用　○実務で利用'],
 ['U4　目的に役立ちましたか。','○役立った　○一部役立った　○役立たなかった　○未確認'],
 ['U5　うまくいかなかった点（複数可）','□導入　□入力準備　□業務との不一致　□確認の手間　□なし'],
 ['U6　結果と改善点','具体的な結果・受けた支援・改善点：＿＿']],foot='回答者名と日時はFormsが記録する。成果物1件につき1回回答する。動作しなかった場合はU6へ記入する。'),
 form_slide('成果物の登録','作成したものを、1件ずつ登録する。',[
 ['W1　成果物の名称','＿＿＿＿（試用フォームの選択肢になる）'],
 ['W2　誰の、どんな課題を改善しますか。','対象者・困っている場面：＿＿'],
 ['W3　組み込んだ知識・手順・判断の目安','＿＿＿＿'],
 ['W4　利用条件・実行方法・期待する結果・限界','＿＿＿＿'],
 ['W5　試してもらう相手','○決まっている（誰：＿＿）　○未定']],foot='1件につき1回。複数作った場合は件数分だけ回答する。運営が一覧を試用フォームの選択肢へ登録する。'),
 dict(label='付録｜フォームの構成',title='アンケートは5本のフォームに分ける。',cls='ws-form',
 intro='Microsoft Formsで実装する。回答者名はFormsが記録し、手入力のIDは設けない。',
 body=table(['フォーム','時期','1人あたりの回答'],[
   ['事前','10月','1回'],
   ['中間','12月','1回'],
   ['成果物の登録','随時','作った件数だけ'],
   ['終了時（作者）','3月','1回'],
   ['試用記録（利用者）','3月の試用期間','試した成果物の数だけ']])
  +note('「1人1回」の設定は1つのフォームにつき1回のため、時点ごとにフォームを分ける。能力C・業務別P／Dは、リッカートのグリッドで各1問にまとめる。'),
 notes='アンケートはMicrosoft Formsで実装します。フォームは5本に分けます。回答者名はFormsが記録するため、参加者IDや利用者IDの手入力欄は設けません。「1人1回」の設定が制限するのは、1つのフォームにつき1回の回答です。事前・中間・終了時を1本のフォームで実施すると、2回目以降の回答を受け付けません。成果物の登録は、作った件数だけ回答してもらい、その一覧が試用フォームの選択肢になります。試用記録は、試した成果物の数だけ回答できます。')
])


# 提案：旧10・旧11の統合、前提条件の新設、付録の索引、本編の並び替え
SHARING = dict(label='共有の2回', title='共有は12月と3月の2回、目的を分けて開催する。', cls='ws-form',
  intro='12月は途中経過の共有、3月は成果物の公開と交流。',
  body=table(['12月　中間共有（TeamsのWeb会議）','3月　最終共有会（オフライン交流会）'],[
    ['画面共有で途中の成果物を実演する','短いデモで取り組みを紹介する'],
    ['判断に迷う点を提示し、助言を受ける','軽食を囲んで交流し、希望者へ仕組みを共有する'],
    ['発表後に中間アンケートへ回答する','後日1週間の試用期間を設ける']])
   +note('発表用の動画制作は不要。1人5分＋会話3分で15名なら約120分を見込み、必要に応じて会を分割する。人気投票・表彰は交流の企画として検討する。'),
  notes='共有の機会は12月と3月の2回です。12月はTeamsのWeb会議で、画面共有により途中の成果物を動かし、判断に迷っている点を話します。発表用動画の制作は求めません。1人5分と会話3分で、15名の場合は約120分を見込みます。3月はオフラインの交流会兼成果物発表会です。軽食を囲んで短いデモを紹介し、希望者へ仕組みを共有します。その場で試せないものは、後日1週間の試用期間へつなげます。人気投票と表彰は、交流を楽しむ企画として検討します。')

PREMISE = dict(label='前提条件と未確定事項', title='開催条件は与件、それ以外は本日の了承後に確定する。', cls='ws-form',
  intro='主催者が指定した条件と、今回の提案で未確定の事項を区別する。',
  body=table(['項目','状態','確定の時期'],[
    ['開催条件（10月開始・下期・Kiro主・約15名・Teams）','主催者指定の与件','確定済み'],
    ['社内Kiro環境の動作','未検証','9月中に確認する'],
    ['概念教材6本','構成案のみ、未制作','10月中に1本目を用意する'],
    ['運営担当と工数','未確保','本日の了承後に確定する'],
    ['利用環境と交流会の費用','未算定','9月・1月に見積もる'],
    ['主KPIの目標値','未設定','事前調査の後に設定する']])
   +note('ヘッダに「企画案」と記載した枚は、今回の提案である。上表の与件は、主催者が指定した条件である。'),
  notes='本日の説明で、決まっている条件と、まだ決まっていない事項を区別します。開催条件は主催者から指定された与件です。社内Kiro環境の動作確認は9月中に実施します。概念教材は構成案までで、1本目は10月中に用意します。運営担当と工数、利用環境と交流会の費用、主KPIの目標値は、本日の了承のあとに確定します。')

APPENDIX_INDEX = dict(label='付録｜案内', title='付録は、題材・環境・設問の区分で構成する。', cls='ws-form',
  intro='必要な箇所から参照する。',
  body=table(['付録','内容'],[
    ['17〜18','PM・PL向けの題材、利用環境（Kiro・Claude）'],
    ['19〜20','設問の考え方（共通・PM・PL）'],
    ['21〜31','回答フォーム。13枚目の区分の順に並び、最後に成果物の登録とフォームの構成を置く']])
   +note('回答フォームは同じ体裁が続く。必要な設問群から参照する。'),
  notes='ここから付録です。付録は3つに分かれます。16枚目と17枚目は題材と利用環境、18枚目と19枚目は設問の考え方、20枚目以降は回答フォームのイメージです。同じ体裁のフォームが続くため、必要な設問群から参照してください。')


QUESTIONS = dict(label='設問の構成', title='設問を6つの区分に分け、区分ごとの目的を明示する。', cls='ws-form',
  intro='同じ設問を事前・中間・終了時に回答してもらう。区分ごとに使い道が異なる。',
  body=table(['設問の区分','答える問い','使い道'],[
    ['利用頻度 F1（1問）','実務でAIを使う頻度が変わったか','部長報告：実務での利用'],
    ['能力 C1〜C7（7問）','自力でできる範囲が広がったか','部長報告：自力でできる範囲'],
    ['業務別 P・D（各6問）','どの業務で実務利用が生じたか','補助指標：業務別の活用'],
    ['個人目標 G・R（9問）','開始時の達成条件を満たしたか','補助指標：目標の達成'],
    ['試用記録 U（6問）','他の人が使って役立ったか','主KPI'],
    ['困りごと N（4問）','どの支援が要るか','運営の改善。指標にしない']])
   +note('回答フォームのイメージは付録21〜31に置く。成果物は「成果物の登録」フォームで1件ずつ登録する。'),
  notes='設問は6つの群に分かれます。利用頻度は実務でAIを使う頻度の変化、能力は自力でできる範囲の変化を見ます。業務別は、どの業務で実務利用が生じたかを職種ごとに聞きます。個人目標は開始時の達成条件と終了時の結果を照合します。試用記録は、作者以外が使って役立ったかを聞くもので、主KPIの取得元です。困りごとは支援の調整に使い、成果指標には含めません。回答フォームのイメージは付録に置いています。')

ORDER = [0, 1, 4, 2, 3, 5, 6, 7, 8, SHARING, 17, 11, QUESTIONS, PREMISE, 14, APPENDIX_INDEX, 15, 16, 12, 13,
         18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28]
SLIDES = [SLIDES[i] if isinstance(i, int) else i for i in ORDER]

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
    notes=[f'# 部長向け発表原稿\n\n全{len(SLIDES)}枚。表紙1枚、本編14枚、付録{len(SLIDES)-15}枚。主催者の条件と、今回の運営提案を分けて説明します。\n']
    for i,s in enumerate(SLIDES,1):
        notes.append(f'## {i:02d}　{re.sub("<[^>]*>", "", s["title"])}\n\n{s["notes"]}\n')
    (OUT/'speaker-notes.md').write_text('\n'.join(notes))
    print(f'Built {len(SLIDES)} slides: {OUT / "director-deck.html"}')

if __name__=='__main__':
    build()
