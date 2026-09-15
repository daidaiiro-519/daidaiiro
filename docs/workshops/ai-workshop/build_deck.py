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
      intro='社内AI人材育成',
      body='<svg class="ws-title-art" viewBox="0 0 1112 130" role="img" aria-label="個々の点がつながり、一つの形になる"><g fill="none" stroke="#9aa18f" stroke-width="2"><circle cx="26" cy="65" r="12"/><circle cx="80" cy="34" r="12"/><circle cx="88" cy="99" r="12"/><path d="M143 65 H205 M265 35 L310 94 L355 35 Z"/><circle cx="265" cy="35" r="10" fill="#f4f1e8"/><circle cx="310" cy="94" r="10" fill="#f4f1e8"/><circle cx="355" cy="35" r="10" fill="#f4f1e8"/><path d="M397 65 H459"/></g><path d="M519 23 L603 65 L519 107 L477 65 Z" fill="#b64326"/></svg>',
      notes='下期AI活用ワークショップの企画をご説明します。開発者、PM・PLなどが、それぞれの仕事の中でAIを活用して新しい仕組みを生み出す力を育てる企画です。'),
 dict(label='企画の目的', title='身近な課題から、<br>新しい仕組みをAIで生み出す力を育てる。', cls='ws-cover',
      intro='社内AI人材育成ワークショップ｜下期実施案',
      body=diagram([('課題を選ぶ','自分の仕事を観察する'),('小さく作る','AIと仕組みを形にする'),('試して直す','他者の反応から学ぶ')],hot=2),
      notes='身近な業務課題から、新しい仕組みをAIで生み出せる人材を育てます。課題の本質を捉え、それを解決する具体的な仕組みへ変えることが狙いです。エージェントやSkillsを使って形にし、実際に試した結果から改善します。'),
 dict(label='育てたい力', title='課題の本質を捉え、具体化して解決する力を<br>共通の土台にする。', cls='ws-abstraction',
      intro='開発者もPM・PLも、抽象と具体を往復する。',
      body='<svg class="ws-abstraction-fig" viewBox="0 0 1112 295" role="img" aria-label="抽象では課題の本質を捉え、具体ではAIで解決の仕組みを作る。試した結果から課題の捉え方を見直す。"><rect x="0" y="40" width="382" height="177" rx="8" fill="#e9e5da"/><text x="28" y="78" font-size="18" fill="#59605a">抽象</text><text x="28" y="122" font-size="28" font-weight="700" fill="#252b29">課題の本質を捉える</text><text x="28" y="162" font-size="19" fill="#59605a">出来事から、重要な構造や関係を</text><text x="28" y="193" font-size="19" fill="#59605a">取り出す。</text><rect x="730" y="40" width="382" height="177" rx="8" fill="#b64326"/><text x="758" y="78" font-size="18" fill="#fff1e9">具体</text><text x="758" y="122" font-size="27" font-weight="700" fill="#fffaf2">AIで解決の仕組みを作る</text><text x="758" y="162" font-size="19" fill="#fff1e9">使う場面と条件に合わせて形にし、</text><text x="758" y="193" font-size="19" fill="#fff1e9">役に立つかを確かめる。</text><text x="556" y="86" font-size="19" text-anchor="middle" fill="#252b29">具体化する</text><path d="M411 109 H701 m -10 -10 l 10 10 -10 10" stroke="#747c6e" stroke-width="2" fill="none"/><text x="556" y="169" font-size="18" text-anchor="middle" fill="#59605a">結果から捉え直す</text><path d="M701 191 H411 m 10 -10 l -10 10 10 10" stroke="#747c6e" stroke-width="2" fill="none"/></svg>',
      notes='共通の土台にしたいのは、課題の本質を抽象的に捉え、それを具体化して解決する力です。ここでいう抽象とは、目的に照らして、個別の出来事から重要な構造や関係を取り出すことです。何を残し何を省くかを明らかにするため、曖昧な表現へ言い換えることとは異なります。具体化では、使う場面や制約に合わせてAIで仕組みを作り、実際に役に立つかを確かめます。最初に捉えた本質も仮説なので、試した結果に応じて捉え直します。この抽象と具体の往復を、開発者とPM・PLの共通の土台にします。'),
 dict(label='テーマの設計', title='幅のあるテーマから、取り組む課題を本人が決める。',
      intro='主催者は困りごとの領域を示し、解決方法と成果物の形は参加者が選ぶ。',
      body=table(['開発者向けの領域','PM・PL向けの領域'],[
          ['開発プロセスの手間を減らす','提案準備や関係者調整の手間を減らす'],
          ['Webアプリを作り始めやすくする','メンバーの状況を把握しやすくする'],
          ['新しい参加者が仕事を始めやすくする','相談や知識共有が生まれやすくする']])+note('テーマを絞る問い：誰が、どんな場面で困るか。何が変われば役に立ったと言えるか。'),
      notes='職種ごとに完成品を指定するのではなく、取り組む領域を選べるようにします。たとえばPM・PLなら、資料を作ることだけでなく、判断材料を集める、認識のずれを見つける、相談の入口を作るといった仕事も対象です。これは用途のヒントであり、推奨する解法の一覧ではありません。参加者自身が経験した場面へ絞り込みます。'),
 dict(label='目指す成果',title='自分のスキルをAIで仕組化し、他の人にも届ける。',intro='知識・ノウハウ・判断の目安を言葉にし、他の人も使える形にする。',body=visuals.handoff()+note('目指す成果：業務の効率化や課題の改善に、他の人もその仕組みを使える。'),notes='自分が持つ知識や仕事の進め方を言語化し、AIで動く仕組みに組み込みます。エージェントに仕事を進める役割を与え、Skillsに手順や判断の目安を持たせることで、その知識を他の人も利用できる形にします。本人と同じ熟練を前提にせず、仕組みを通じて業務の効率化や課題改善に役立ててもらうことが目的です。AIの出力を確認する必要はあり、技能全体を完全に置き換えるという意味ではありません。説明や初回の支援があっても構いません。他者が使った結果と、どんな改善に役立ったかを確かめます。',cls='ws-diagram'),
 dict(label='エージェントとSkills', title='エージェントに役割を与え、Skillsで仕事の進め方を渡す。', cls='ws-diagram',
      intro='人が目的と制約を決め、AIが道具を使って実行し、人が結果を確かめる。',
      body=visuals.agents()+note('Kiroの機能と利用環境は、開始前に確認する。'),
      notes='前のページで示した、自分のスキルをAIで仕組化して他の人にも届けるという成果を、エージェントとSkillsで実現します。この図は学習のために仕事の関係を整理したものです。カスタムエージェントには役割や使える道具、参照する情報などを設定できます。Skillsは必要な場面で参照する手順や参考資料のまとまりです。すべてを自律実行させる必要はなく、何を任せて何を人が確認するかを決めることを重視します。Kiro公式資料は付録に記載しています。'),
 dict(label='概念教育', title='教材で考える道具を渡し、操作の入口を支える。', cls='ws-diagram',
      intro='音声付きの短いスライド動画を、必要なときに見返せる教材にする。',
      body=visuals.learning()+note('操作の入口を支え、解決方法は参加者自身が考える。'),
      notes='主教材は概念中心の動画にし、操作の説明は独立した短い補助教材にします。形式を知らないために始められない状態は防ぎます。一方、業務課題の完成例を見せすぎると解法を固定するため、主教材には目的や分担の図を置き、用途のヒントは別途少量だけ公開します。教材はこれから制作する計画で、この提案デッキが受講者向け動画ではありません。'),
 dict(label='下期の進め方', title='半年間で、試作から他者の利用へ進める。',
      intro='2026年10月〜2027年3月の実施案。月ごとの区切りで、非同期に進める。',
      body='<div class="ws-phases"><svg class="ws-phase-path" viewBox="0 0 1112 240" role="img" aria-label="10・11月の試作から、12・1月の見直しを経て、2・3月の他者利用へ進む"><path d="M 355 107 h 21 m -6 -6 l 6 6 -6 6 M 736 107 h 21 m -6 -6 l 6 6 -6 6" fill="none" stroke="#747c6e" stroke-width="2"/></svg><div><p class="ws-date">10・11月</p><h3>学び、試作する</h3><p>概念教材を視聴する<br>課題を選び、小さく動かす</p></div><div class="ws-phase-hot"><p class="ws-date">12・1月</p><h3>見せて、絞り直す</h3><p>12月に中間成果を共有する<br>他者の反応で方向を見直す</p></div><div><p class="ws-date">2・3月</p><h3>使ってもらい、残す</h3><p>別の人が試して改善する<br>3月に成果と学びを共有する</p></div></div>'+note('日程は提案。主催者指定の「10月開始・下期」を、上記の期間として計画している。'),
      notes='毎週同じ時間に集まることは前提にしません。10月から翌3月までを提案期間とし、12月に中間共有、3月に最終共有を置きます。初期から試作に触れ、中間発表まで完成を待たないようにします。途中のテーマ変更も、試した結果に理由があれば認めます。休暇や繁忙期を踏まえ、実働20週を工数試算の仮定にしています。'),
 dict(label='サポート体制', title='学習コンテンツと、主催者への相談で支える。', cls='ws-diagram',
      intro='主催者に知見が集まっている前提で、参加者が必要な支援に届くようにする。',
      body=visuals.support()+note('相談会の時間帯は未定。定時後の場合は任意参加とし、Teams投稿でも相談できる。'),
      notes='参加者同士で専門的なサポートを担う前提は置きません。ブログや動画で学び、Teamsで主催者に相談できる体制にします。月1回のよろず相談会は開催案です。資料発表の準備を求めず、今の課題や相談したいことをざっくばらんに話す場にします。時間帯はまだ決まっていません。定時後になる場合は任意参加とし、相談内容の要点をTeamsに残します。参加できない人も投稿で同じ支援を受けられるようにします。'),
 dict(label='中間成果物発表', title='中間発表で、次に試すことを見つける。', cls='ws-sharing ws-diagram',
      intro='12月は、途中の成果と判断に迷っている点を共有する。',
      body=visuals.meeting()+note('発表用の動画制作は不要。開催日は調整し、必要に応じて会を分ける。'),
      notes='中間発表はTeamsのWeb会議で行います。参加者が画面共有で途中の成果物を動かし、困っている点を話します。発表用動画の制作は求めません。開催日は調整し、全員が一度に集まれない場合は発表枠を分けます。完成していなくても、動く部分や試した結果があれば共有できます。発表後に中間アンケートへ回答し、次に試すことを記録します。'),
 dict(label='最終成果物発表・交流会',title='気軽に集まり、作った仕組みを見せ合う。',intro='コミュニケーション施策として、軽食を囲むオフライン交流会を開く。',body=diagram([('見せる','短いデモで取り組みを紹介'),('話す・試す','軽食を囲んで気軽に交流'),('つなげる','使いたい人へ仕組みを共有')],hot=1)+note('その場で試せないものは後日試用する。人気投票・表彰は交流を楽しむ企画として検討する。'),notes='最終回は参加者が集まるオフラインの交流会兼成果物発表会にします。軽食を囲み、短いデモを見たり、作った工夫や困ったことを話したりできる、堅苦しくない場を目指します。会場、飲食費、日程は今後調整します。その場で試せない成果物は後日の試用につなげます。交流と他者利用のきっかけを作る場とし、人気投票は任意の企画案として残します。',cls=''),
 dict(label='効果測定の方法',title='他者に役立つ仕組みと、業務での活用の変化を確かめる。',intro='事前・中間・終了時に同じ設問で回答し、成果物の試用記録も集める。',body=visuals.evaluation()+note('主KPI案：他者に役立つ仕組みを作れた参加者数／開始時の参加者数。詳細は付録へ。'),notes='主KPIは、自分の知識をAIで動く仕組みにし、他者が使用して役立ったと確認できた参加者の割合を提案します。開始時の参加者数を分母に固定し、未提出や未確認も明示します。個人目標の達成状況と、業務別のAI活用の変化を補助指標にします。アンケートは同じ人・同じ業務を比較し、回答人数を示します。能力の自己評価と実際の利用経験は合算しません。数値目標は事前調査と運営条件を見て開催前に決めます。',cls='ws-eval ws-diagram'),
 dict(label='共通アンケートの設問例', title='普段の言葉で、今できることを聞く。', cls='ws-survey-core ws-diagram',
      intro='AIに詳しくない参加者も、自分の経験から答えられる質問にする。',
      body=visuals.questionnaire()+note('共通7問から3問を抜粋。事前・中間・終了時に、同じ質問と選択肢で聞く。'),
      notes='共通項目は、仕事の中でできることを本人の言葉で答えられるようにしています。使いどころ、困りごとの整理、AIへの依頼、複数作業の依頼、使い方の説明、答えの確認、感想をもとにした改善の7問です。回答は0から4の5段階で、スライド右側の選択肢から、今の自分に近いものを選びます。試す機会がない場合も選べます。難しい概念の理解を問う試験にはせず、同じ質問で自己評価の変化を比べます。実際の力は成果物や試用記録も合わせて確認します。開始前に参加者に近い人に回答してもらい、意味が伝わるかを確かめます。'),
 dict(label='PM・PLアンケートの設問',title='PM・PLの業務ごとに、AIの活用状況を聞く。',intro='質問：過去4週間に、次の業務でAIをどの程度使いましたか。',body=visuals.pm_survey()+note('選択肢は開発者も共通。実務で使った場合は、AIの結果を人が確認して利用した経験を答える。'),notes='提供された管理・リーダー業務の例をもとに、情報収集・集計、報告・予実、課題・担当・期限、品質・障害・リスク、改善提案、過去事例の再利用の6項目を聞きます。開発者は要求・仕様、設計、実装、レビュー、テスト、運用・保守について、同じ期間と選択肢で答えます。担当外と実施機会なしを分け、0点に置き換えません。業務ごとに質問は変えますが、利用実績を聞く回答形式は揃えます。',cls='ws-survey-pm ws-diagram'),
 dict(label='部長への依頼',title='運営者の工数と、利用環境の費用について了承を得る。',intro='参加者は期間中に各自で時間を見つけ、自分の課題と制作目標を決めて取り組む。',body=table(['了承を得たいこと','見積もりに含めるもの'],[['運営者の工数','教材制作・相談対応・進行・効果測定'],['利用環境の費用','Kiro等の利用枠・契約・追加費用']])+note('金額・工数は見積もり後に提示する。交流会の会場・軽食費も別途確認する。'),notes='参加者全員に週何時間という一律の学習時間は設けず、期間中に各自が時間を見つけて進めます。開始時に、どの課題を、どんな仕組みで解決し、誰に使ってもらうかと達成を確かめる方法を決めてもらいます。今回了承を得たいのは、教材制作や相談対応などの運営者の工数と、AI利用環境にかかる費用です。工数・契約・追加枠を見積もって提示します。会場と軽食の費用も別途確認します。',cls=''),
 dict(label='付録｜PM・PLのテーマ', title='PM・PL向けには、判断・調整・相談の負荷を題材にする。',
      intro='用途のヒント。参加者は、実際に経験した場面から自分の課題を選ぶ。',
      body=table(['テーマの領域','困りごとの例','参加者が確かめること'],[
          ['判断の準備','提案のたびに、必要な材料を探し直す','何が揃えば判断を始められるか'],
          ['関係者の調整','決まったことと保留事項が混ざってしまう','どこで認識がずれているか'],
          ['チームの相談','困りごとが、問題になってから分かる','どの場面なら相談しやすいか']])+note('メンバー個人を自動採点する題材より、仕事の状況と支援の必要性を共有する題材を勧める。'),
      notes='PM・PLの仕事を、文書作成だけに狭めないための補足です。進捗把握には情報収集、解釈、支援判断が含まれます。コミュニケーション活性化も、発言数を増やすことに固定せず、必要な相手に相談しやすくなったかを確かめます。これらは主催者向けの題材候補で、受講者には課題選択後に必要なヒントだけを渡します。'),
 dict(label='付録｜利用環境', title='Kiroを共通の実行環境にし、Claudeは補助に使う。',
      intro='Claudeを使えない参加者も、必須課題を完了できる構成にする。',
      body=table(['主環境：Kiro','補助環境：Claude'],[
          ['エージェントの設定・実行','利用可能な範囲での壁打ち'],['Skillsの作成・利用','文章や説明の別視点での確認'],['成果物の試行と改善','任意の補助作業']])+ '<p class="ws-source">開始前に確認：社内のIDE／CLIの版、利用枠、ファイル・コマンドの権限。<br>機能の根拠：<a href="https://kiro.dev/docs/skills/">Kiro Agent Skills</a> ／ <a href="https://kiro.dev/docs/cli/custom-agents/creating/">Creating custom agents</a>（2026-09-05確認）。<br>本デッキ作成時点で、社内Kiro環境での動作検証は未実施。</p>',
      notes='Kiro公式ドキュメントではAgent Skillsとカスタムエージェントの作成方法が説明されています。ただし社内で導入済みの版と設定は未確認です。初回教材を収録する前に参加者と同じ環境で、作成、読み込み、実行、結果確認までを試します。教材は概念編と操作編に分け、機能更新時は操作編だけを差し替えられるようにします。')
]



def form_slide(label, title, rows, intro='回答用フォームのイメージ。枠内に記入し、各設問の指定に従って選ぶ。', foot=''):
    return dict(label='付録｜'+label,title=title,intro=intro,cls='ws-form',body=table(['質問・項目','回答欄'],rows)+note(foot),notes='このページは「'+label+'」の回答イメージです。左に質問、右に回答欄を示しています。選択できる数と回答期間を確認して記入してもらいます。ページ下部には回答や集計の注意点を示しています。実際のフォームは別途作成し、参加者に近い人に試してもらってから確定します。')

SLIDES.extend([
 dict(label='付録｜KPI案',title='制作・業務活用・個人目標の3つで成果を報告する。',intro='指標と集計方法の提案。達成人数の目標値は、開始前に設定する。',cls='ws-form',body=table(['指標','数え方・確かめ方'],[
 ['主KPI：他者に役立つ仕組み','他者が使って役立った仕組みを作れた参加者数／開始時の参加者数'],
 ['補助：業務別のAI活用','同じ人・同じ業務で、実務利用あり（2・3）の人数の変化'],
 ['補助：個人目標の達成','開始時に決めた達成条件と、終了時の結果・根拠を照合']])+note('人数と分母を併記。配布・試用・実務利用を区別し、人気投票は成果指標に含めない。'),notes='主KPIは人数単位で重複を除きます。他者の利用記録に用途・結果・役立った点が残ることを条件にし、デモを見ただけでは数えません。試用と実務利用の内訳を分けます。未提出・未確認は分母から外さず内訳を示します。共同制作では本人が担当した知識や手順の具体化を確認します。業務活用は前後とも担当・実施機会があり回答した人のみで比較し、職種の総合点や順位を作りません。'),
 form_slide('参加者情報・利用頻度','担当業務と、直近4週間のAI利用状況を確認する。',[
 ['参加者ID・回答時点','ID：＿＿＿　○事前　○中間　○終了'],
 ['担当する業務（複数選択可）','□PM・PL　□開発　□その他：＿＿＿'],
 ['過去4週間に実務でAIを使った日数','○0日　○1〜3日　○4〜7日　○8〜15日　○16日以上'],
 ['実務の機会がない場合','○実務の機会なし（上の日数の代わりに選択）']],foot='研修だけで使った日は除く。3回とも同じIDで回答。個別回答は運営が確認する。'),
 form_slide('共通アンケート①','課題を選び、AIへ仕事を伝える力を聞く。',[[f'C{i+1}　'+q,'○0　○1　○2　○3　○4　○機会なし'] for i,q in enumerate(visuals.CORE_QUESTIONS[:4])],foot='0 まだやり方が分からない ／ 1 やり方は分かるが、まだできない ／ 2 手助けがあればできる<br>3 一人でできる ／ 4 状況に合わせてやり方を変えられる ／ 機会なし 試す機会がない'),
 form_slide('共通アンケート②','説明・確認・改善の力を聞く。',[[f'C{i+5}　'+q,'○0　○1　○2　○3　○4　○機会なし'] for i,q in enumerate(visuals.CORE_QUESTIONS[4:])]+[['中間・終了時：具体例を一つ記入','できたこと・まだ難しいこと：＿＿＿＿']],foot='0 まだやり方が分からない ／ 1 やり方は分かるが、まだできない ／ 2 手助けがあればできる<br>3 一人でできる ／ 4 状況に合わせてやり方を変えられる ／ 機会なし 試す機会がない'),
 form_slide('PM・PLの業務','管理・リーダー業務のAI活用を回答する。',[[f'P{i+1}　'+q,'○0　○1　○2　○3　○担当外　○機会なし'] for i,q in enumerate(visuals.PM_TASKS)],intro='過去4週間に、次の業務でAIをどの程度使いましたか。各行で一つ選択。',foot='0 未利用 ／ 1 試用のみ ／ 2 実務で1回 ／ 3 実務で複数回。実務利用は結果を人が確認したもの。'),
 form_slide('開発者の業務','要求整理から運用・保守まで、AI活用を回答する。',[[f'D{i+1}　'+q,'○0　○1　○2　○3　○担当外　○機会なし'] for i,q in enumerate(visuals.DEV_TASKS)],intro='過去4週間に、次の業務でAIをどの程度使いましたか。各行で一つ選択。',foot='0 未利用 ／ 1 試用のみ ／ 2 実務で1回 ／ 3 実務で複数回。兼務者は両方の業務表に回答。'),
 form_slide('個人目標','何を作り、誰のどんな課題を改善するかを決める。',[
 ['G1　誰の、どんな課題ですか。','対象者・困っている場面：＿＿＿＿'],
 ['G2　どんな仕組みをAIで作りますか。','作るもの・使う知識や手順：＿＿＿＿'],
 ['G3　誰に使ってもらいますか。','利用者・試してもらう場面：＿＿＿＿'],
 ['G4　何が変われば達成ですか。','確認する結果・方法・期限：＿＿＿＿']],foot='開始時に記入し、中間で見直す。変更前の目標と変更理由も残す。'),
 form_slide('困りごと・支援','進めるうえで必要な支援を聞く。',[
 ['N1　困っていること（最大3つ）','□使いどころ　□依頼方法　□結果の確認　□共有　□環境　□時間　□なし　□その他'],
 ['N2　AIを使いたい業務（最大2つ）','P／Dの項目：＿＿　□その他　□まだ思いつかない'],
 ['N3　どんな場面で困っていますか。','具体的な場面：＿＿＿＿'],
 ['N4　欲しい支援（最大2つ）','□概念動画　□操作説明　□事例　□ブログ　□Teams相談　□会話　□環境　□時間　□その他']],foot='回答は支援の調整に使う。具体的な案件名は記入不要。'),
 form_slide('終了時・作者','個人目標に対して、作った結果を振り返る。',[
 ['R1　目標に対する結果はどうでしたか。','○達成　○一部達成　○未達　○未実施'],
 ['R2　何を作り、どの知識を組み込みましたか。','成果物ID・場所・具体化した知識や手順：＿＿'],
 ['R3　何が改善しましたか。','結果・根拠：＿＿　○実測　○体感　○未確認'],
 ['R4　今後必要なことは何ですか。','未解決の点・続けたいこと・必要な支援：＿＿']],foot='効果がなかった結果も記入。時間短縮を示す場合は、比較条件・確認修正時間も添える。'),
 form_slide('利用者の試用記録','使った人に、役立った結果と改善点を聞く。',[
 ['U1　何を、何のために使いましたか。','成果物ID・利用者ID・日付・用途：＿＿'],
 ['U2　どこまで使いましたか。','○デモ閲覧のみ　○試用　○実務で利用'],
 ['U3　目的に役立ちましたか。','○役立った　○一部役立った　○役立たなかった　○未確認'],
 ['U4　何が変わり、何が難しかったですか。','具体的な結果・受けた支援・改善点：＿＿']],foot='主KPIは作者以外の利用と「役立った」の具体的な記録で確認。閲覧だけでは数えない。')
])

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
