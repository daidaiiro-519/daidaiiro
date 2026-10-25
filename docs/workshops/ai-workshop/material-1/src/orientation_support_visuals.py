"""企画書のポータルとサポート体制の図を受講者向けに使用する。"""
import visuals as V

def portal():
    figure = (V.portal().replace('教材動画</text>', '動画・解説記事</text>')
            .replace('参加者が作った仕組み', '参加者のSkillsと記録')
            .replace('目的に合う仕組み', '目的に合うSkills')
            .replace('合わなければ、前に戻って直す', '分からなければ、前の教材に戻る')
            .replace('>成果物</text>', '>参加者の実践</text>', 1)
            .replace('ナレッジ共有ポータル（社内 ・ 今期はトライアル）',
                     'ナレッジ共有ポータル（社内の教材・事例の閲覧場所）'))
    # 教材名を共通の「はじめに」の名称にする。長くなるため文字を一段小さくする。
    for before, after in [('目的の把握', '目的と要件の明確化'), ('言葉の定義', '業務のモデル化と設計'), ('仕組みの構築', 'AI活用の実践と改善')]:
        figure = figure.replace(f'font-size="17" fill="#252b29" font-weight="400" text-anchor="start">{before}</text>',
                                f'font-size="14" fill="#252b29" font-weight="400" text-anchor="start">{after}</text>', 1)
    figure = figure.replace('<text x="252" ', '<text x="246" ')
    # 戻る矢印を、長くなった教材名と重ならない位置へ移す。
    figure = figure.replace('d="M392 246 C 404 226, 404 168, 392 150 M386 160 L392 150 L400 157"', 'd="M398 246 C 410 226, 410 168, 398 150 M392 160 L398 150 L406 157"')
    # 参加者が載せる項目を、今回の教材の内容に合わせる。
    for before, after in [('>目的</text>', '>目的と要件</text>'), ('>使い方</text>', '>判断の観点と手順</text>'), ('>試した記録</text>', '>使い方</text>'), ('>言葉の定義</text>', '>試した結果と改善</text>')]:
        figure = figure.replace(before, after, 1)
    # 人物と説明を一つの対象として囲み、矢印の接続先を示す。
    frames = V.rect(0, 60, 114, 240, V.PAPER, 12, V.LINE)
    frames += V.rect(988, 60, 124, 240, V.PAPER, 12, V.LINE)
    figure = figure.replace('>', '>' + frames, 1)
    # 同じ役割のタグの幅と間隔を合わせる。
    for old, new in [
        (V.rect(694, 132, 88, 34, V.PAPER, 17, V.LINE), V.rect(694, 132, 72, 34, V.PAPER, 17, V.LINE)),
        (V.rect(788, 132, 82, 34, V.PAPER, 17, V.LINE), V.rect(772, 132, 72, 34, V.PAPER, 17, V.LINE)),
        (V.rect(876, 132, 42, 34, V.PAPER, 17, V.LINE), V.rect(850, 132, 72, 34, V.PAPER, 17, V.LINE)),
        (V.text(738, 155, '顧客対応', 13, V.INK, anchor='middle'), V.text(730, 155, '顧客対応', 13, V.INK, anchor='middle')),
        (V.text(829, 155, '問い合わせ', 13, V.INK, anchor='middle'), V.text(808, 155, '問い合わせ', 13, V.INK, anchor='middle')),
        (V.text(897, 155, '集計', 13, V.INK, anchor='middle'), V.text(886, 155, '集計', 13, V.INK, anchor='middle')),
    ]:
        figure = figure.replace(old, new)
    return figure

def support():
    figure = (V.support().replace('教材動画と成果物で学ぶ', '教材動画・解説記事・事例を見る')
              .replace('Teamsで相談する', 'Teamsで質問・相談')
              .replace('いつでも投稿 ＋ スポット相談会', '投稿して質問 ＋ 必要に応じて相談会'))
    # 人物を背景へ直接置かず、11枚目と同じく枠で囲む。矢印は枠の上端と下端につながる。
    frames = V.rect(20, 88, 150, 172, V.PAPER, 12, V.LINE) + V.rect(938, 88, 150, 172, V.PAPER, 12, V.LINE)
    figure = figure.replace('>', '>' + frames, 1)
    for before, after in [('d="M1013 100 V65 H778"', 'd="M1013 88 V65 H778"'),
                          ('d="M336 65 H95 V100 M88 92 L95 100 L102 92"', 'd="M336 65 H95 V88 M88 80 L95 88 L102 80"'),
                          ('d="M95 250 V285 H336"', 'd="M95 260 V285 H336"'),
                          ('d="M778 285 H1013 V250 M1006 258 L1013 250 L1020 258"', 'd="M778 285 H1013 V260 M1006 268 L1013 260 L1020 268"')]:
        figure = figure.replace(before, after)
    # 下段は1行にし、ページのフッターと重ならない高さに収める。
    footer = V.rect(0, 356, 1112, 46, V.PAPER, 12, V.LINE)
    footer += V.icon(20, 367, 'screen', V.INK, .5)
    footer += V.text(64, 386, 'ワークショップの参加者（15〜20名）には、KiroのアカウントとAWS環境を発行（ポータルのアカウントとは別）', 17, V.INK, 700)
    # 相談は運営から答えが返るため、参加者側にも矢じりを付けて双方向にする。
    footer += V.path('M88 268 L95 260 L102 268', V.ACCENT, 3)
    return figure.replace('1112 350', '1112 406').replace('</svg>', footer + '</svg>')
