"""企画書のポータルとサポート体制の図を受講者向けに使用する。"""
import visuals as V

def portal():
    figure = (V.portal().replace('教材動画</text>', '動画・解説記事</text>')
            .replace('参加者が作った仕組み', '参加者のSkillsと記録')
            .replace('目的に合う仕組み', '目的に合うSkills')
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
    footer = V.rect(0, 368, 1112, 62, V.PAPER, 12, V.LINE)
    footer += V.icon(20, 381, 'screen', V.INK, .6)
    footer += V.text(72, 393, '実践の参加者15名には、KiroのアカウントとAWS環境を発行', 19, V.INK, 700)
    footer += V.text(72, 417, '実践に使うAIツールと実行環境です。ポータルとは別に用意します', 15, V.DIM)
    return figure.replace('1112 350', '1112 438').replace('</svg>', footer + '</svg>')
