from pathlib import Path
import hashlib
import html
import json
import re
import markdown

A = Path(__file__).resolve().parent
V = Path('/tmp/ai-workshop-viewer-20261017')
slides = [json.loads(p.read_text()) for p in sorted(A.glob('slide-[0-9][0-9].json'))]
for approval in sorted(A.glob('approved-slide-*-hashes.json')):
    for name, expected in json.loads(approval.read_text()).items():
        assert hashlib.sha256((A / name).read_bytes()).hexdigest() == expected, '承認済みの枚が変わっています: ' + name
base = (A / 'baseline-after.html').read_text()
parts = re.findall(r'<section\b.*?</section>', base, re.S)
after = base
labels = [html.unescape(re.search(r'<h2>(.*?)</h2>', x, re.S)[1]) for x in parts]
for s in slides:
    n = s['slide']
    fig = (A / f'slide-{n:02}.svg').read_text().replace('<svg ', '<svg class="ws-visual" ', 1)
    label = '最初の内容スライド' if n == 1 else f'内容{n}枚目'
    section = ('<section class="slide ws-diagram"><div class="kicker">'
           '<span class="ws-mark" aria-hidden="true"></span>'
           '<span class="ws-kicker-label">3教材共通 はじめに ／ ' + label + '（確認用）</span></div>'
           '<h2>' + html.escape(s['heading']) + '</h2><p class="ws-intro">'
           + html.escape(s['lede']) + '</p><div class="body">' + fig + '</div></section>')
    after = after.replace(parts[n - 1], section, 1)
    labels[n - 1] = s['heading']
after = re.sub(r'const LABELS = .*?;', lambda _: 'const LABELS = ' + json.dumps(labels, ensure_ascii=False) + ';', after)
(A / 'after.html').write_text(after)
(A / 'script.md').write_text('# 三教材共通のはじめに：確認用の原稿\n\n' + '\n\n'.join('## ' + str(x['slide']) + ' ' + x['heading'] + '\n\n' + x['narration'] for x in slides) + '\n')
prior_deck = (A / 'before-slide-02-bubble' / 'slide-02.json')
prior_slide = json.loads(prior_deck.read_text())
# The saved deck predates the second slide; use its approved shell for this revision's baseline.
prior_svg = (A / 'before-slide-02-bubble' / 'slide-02.svg').read_text().replace('<svg ', '<svg class="ws-visual" ', 1)
prior_section = ('<section class="slide ws-diagram"><div class="kicker"><span class="ws-mark" aria-hidden="true"></span>'
                 '<span class="ws-kicker-label">3教材共通 はじめに ／ 内容2枚目（変更前）</span></div>'
                 '<h2>' + html.escape(prior_slide['heading']) + '</h2><p class="ws-intro">'
                 + html.escape(prior_slide['lede']) + '</p><div class="body">' + prior_svg + '</div></section>')
before = base.replace(parts[1], prior_section, 1)
review_prior = json.loads((A / 'before-slide-03-review-cause' / 'slide-03.json').read_text())
review_prior_svg = (A / 'before-slide-03-review-cause' / 'slide-03.svg').read_text().replace('<svg ', '<svg class="ws-visual" ', 1)
review_prior_section = ('<section class="slide ws-diagram"><div class="kicker"><span class="ws-mark" aria-hidden="true"></span>'
                        '<span class="ws-kicker-label">3教材共通 はじめに ／ 内容3枚目（変更前）</span></div>'
                        '<h2>' + html.escape(review_prior['heading']) + '</h2><p class="ws-intro">'
                        + html.escape(review_prior['lede']) + '</p><div class="body">' + review_prior_svg + '</div></section>')
before = before.replace(parts[2], review_prior_section, 1)
for n in [3,8,9,10]:
    prior = json.loads((A / 'full-review' / 'before' / f'slide-{n:02}.json').read_text())
    svg = (A / 'full-review' / 'before' / f'slide-{n:02}.svg').read_text().replace('<svg ', '<svg class="ws-visual" ', 1)
    current = re.findall(r'<section\b.*?</section>', before, re.S)[n-1]
    replacement = '<section class="slide ws-diagram"><h2>' + html.escape(prior['heading']) + '</h2><p class="ws-intro">' + html.escape(prior['lede']) + '</p><div class="body">' + svg + '</div></section>'
    before = before.replace(current, replacement, 1)
(A / 'before.html').write_text(before)

def notes(heading, lede, narration):
    return '<h4>見出し</h4><p>' + html.escape(heading) + '</p><h4>導入文</h4><p>' + html.escape(lede) + '</p><h4>原稿</h4>' + markdown.markdown(narration)

def publish():
    (V / 'common-intro-review' / 'script.md').write_text((A / 'script.md').read_text())
    for side in ['before', 'after']:
        (V / 'common-intro-review' / (side + '.html')).write_text((A / (side + '.html')).read_text())
    def version(side):
        return hashlib.sha256((A / (side + '.html')).read_bytes()).hexdigest()[:12]
    text = (A / 'baseline-design-review.html').read_text()
    m = re.search(r'(<script type="application/json" id="data">)(.*?)(</script>)', text, re.S)
    data = json.loads(m[2])
    d = next(x for x in data if x['id'] == 'lead')
    oldnotes = json.loads(json.dumps(d['afterNotes']))
    oldnotes[1] = dict(number=2, title=prior_slide['heading'], body=notes(prior_slide['heading'], prior_slide['lede'], prior_slide['narration']))
    oldnotes[2] = dict(number=3, title=review_prior['heading'], body=notes(review_prior['heading'], review_prior['lede'], review_prior['narration']))
    for n in [3,8,9,10]:
        prior = json.loads((A / 'full-review' / 'before' / f'slide-{n:02}.json').read_text())
        oldnotes[n-1] = dict(number=n, title=prior['heading'], body=notes(prior['heading'], prior['lede'], prior['narration']))
    newnotes = json.loads(json.dumps(oldnotes))
    for s in slides:
        newnotes[s['slide'] - 1] = dict(number=s['slide'], title=s['heading'], body=notes(s['heading'], s['lede'], s['narration']))
    d.update(before='common-intro-review/before.html?v=' + version('before'), after='common-intro-review/after.html?v=' + version('after'),
             notes=newnotes, beforeNotes=oldnotes, afterNotes=newnotes,
             reasons={'1': '新しく参画した開発者を主役にした案件説明へ修正。業務担当者と既存コードを図に加え、本人も資料を確認して作業する前提を原稿に明記しました。原因の説明はこの枚では行いません。',
                      '2': '開発者を左端に配置し、刷新前の機能と刷新後の実装対象、および確認で着目している動作を同じ吹き出し内に配置しました。原稿の図の案内も修正。変更前は直前の図と原稿です。',
                      '3': 'AIで成果物が増える一方、条件と実装・テストの対応が把握しにくく、人のレビューがAIの指摘確認に偏った過程へ変更。この案件のPR承認基準と、レビュー承認後もマージ前であることを明示しました。変更前は直前の図と原稿です。',
                      '4': 'レビューで発見する旧設定を変更。マージ前の結合テストで申請・承認・再変更を試し、不具合が発覚する場面を追加。期待する動作と実際の金額を対比しました。',
                      '5': '資料の不在を原因にせず、資料を確認しても条件を見落とした事実から、目的・言葉の意味・AIと人の確認方法を振り返る三つの問いへ変更。',
                      '6': '承認後の変更制御を直す具体的な対応から、この案件で業務として実現したい目的へ戻り、その目的から必要な作業を考える図へ修正。参照漏れと原因を固定する説明を外しました。',
                      '7': '教材1で確かめた業務の目的を受け、既存の条件を表す言葉の意味と範囲を具体化する学びを紹介。仕様が存在し、資料も確認したという設定を維持しています。',
                      '8': '共通の条件と役割別のSkillを使う既存図を維持し、各出力に条件と確認結果の対応を含める学びへ変更。仕様を参照していなかったという旧原因は説明しません。',
                      '9': '別の機能に必要な条件を確かめ、AIが示す根拠を仕様・コード・実際のテスト結果と照合。確認の負担も見て参照情報やSkillを改善する流れへ更新。',
                      '10': '新しい開発者も資料を確認し、AIと人のレビューで見逃して結合テストで発覚した設定に合わせてまとめを修正。必要な確認を維持しながら品質とチームの判断につなげる到達点を示します。'})
    text = text[:m.start(2)] + json.dumps(data, ensure_ascii=False).replace('</', '<\\/') + text[m.end(2):]
    start = text.index('<article id="lead-review-notice">')
    end = text.index('</article>', start)
    # Keep the existing cover and its links; replace only the explanatory notice.
    notice = text[start:end]
    notice = re.sub(r'<h2>.*?</h2>', '<h2>三教材共通の「はじめに」：全編の確認と修正</h2>', notice, count=1, flags=re.S)
    notice = re.sub(r'<p>.*?</p>', '<p>表紙と全10枚の図・原稿を確認しました。今回は内容3・8・9・10枚目を修正しています。この4枚の変更前には、直前の版を表示します。表紙とオリエンテーション、動画・音声は今回更新していません。</p>', notice, count=1, flags=re.S)
    notice = re.sub(r'<section><h3>表紙の次：共通の題材を説明</h3>.*?</section>', '', notice, flags=re.S)
    notice = notice.replace('更新した動画・音声の再生ページ', '現行の動画・音声（今回の修正は未反映）')
    text = text[:start] + notice + text[end:]
    (V / 'design-review.html').write_text(text)
    (A / 'comparison.html').write_text(text)
    log = [dict(slide=1, group='story', change='資料とコードを確認する新規参画者を含む案件の前提を説明し、原因を先に断定しない原稿へ変更。'),
           dict(slide=1, group='figure', change='既存の人物・資料の部品を使用し、業務担当者と既存コードを追加。新しく加わった開発者は枠線だけを強調し、背景はpaper色に変更。'),
           dict(slide=2, group='story', change='案件の前提を受け、既存の申請変更機能を刷新する担当作業を説明。原因の説明は後の枚へ移動。'),
           dict(slide=2, group='figure', change='既存の申請画面と、刷新後に実装する画面・APIを対比。下書きの金額修正と保存を同じ部品で表現。'),
           dict(slide=2, group='japanese', change='新しく参画した開発者を主語とし、次の実装・単体テスト・レビューの流れへ接続。'),
           dict(slide=2, group='figure', change='左右に同じ画面を置いた図を改め、稼働中の機能と新しく実装する画面・APIを区別。画面とAPIの間隔を確保し、ボタンが枠線に接触しない配置へ変更。'),
           dict(slide=2, group='heading', change='引き継ぐ機能と作り直す部分を見出しに明示。原稿の左右の案内も修正。'),
           dict(slide=2, group='story', change='新規参画者が資料も確認したうえで、金額の変更と保存に着目していることを図と原稿へ追加。後続の結合テストで不具合が発覚する設定に接続。'),
           dict(slide=2, group='layout', change='人物を左端へ移し、刷新前後の内容と確認の着目点を同じ吹き出しに配置。原稿の案内も図に対応。'),
           dict(slide=3, group='story', change='実装、作成した単体テストの通過、AIレビュー、人のPRレビュー、結合テストへの移行を示す。発見場面は次の枚。'),
           dict(slide=3, group='figure', change='コードとテストの図、AIと人の確認、結合テストを三つの工程として図示。確認した下書きの金額変更と保存を明記。'),
           dict(slide=3, group='japanese', change='全条件の確認済みを意味しないよう、作成した単体テストはすべて通ったと表現。PRはナレーションでプルリクエストと説明。'),
           dict(slide=3, group='story', change='AIで短時間に増えたコード・テスト・説明を確認する中で、AIの指摘確認に偏り、業務条件と実装・テストの照合が不十分なままレビュー承認した設定を反映。'),
           dict(slide=3, group='figure', change='承認基準、増えた成果物、指摘なしのAIレビュー結果に着目する人、確認不足、マージ前の結合テストを図示。'),
           dict(slide=3, group='japanese', change='レビュー承認とマージを区別。この案件では結合テスト通過もマージ条件とする前提を明記。'),
           dict(slide=4, group='story', change='レビュー承認後、マージ前の結合テストで不具合が発覚。承認済み状態のケースが単体テストに含まれていなかったことも説明。'),
           dict(slide=4, group='figure', change='同じ申請の作成・承認・再変更を順に示し、期待する12,000円維持と実際の18,000円への変更を対比。'),
           dict(slide=4, group='japanese', change='単体テストでもケースがあれば検出可能だったと明記。マージせず修正・再テスト・再レビューへ進む。'),
           dict(slide=5, group='story', change='仕様とコードが存在し確認もあった事実を受け、作業の目的・言葉の理解・AIと人の確認方法へ視点を移す。'),
           dict(slide=5, group='figure', change='既存の条件がある仕様書とコード、AIを使う開発者とレビュー担当者、振り返る三つの問いを対応させる。'),
           dict(slide=6, group='story', change='今回の修正と別機能にも使う進め方を区別。業務で実現したい目的を確かめ、その目的から必要な作業を考える教材1へ接続。'),
           dict(slide=6, group='figure', change='既存の上下の図と往復する矢印を維持。上に業務ルールに沿って経費を精算できる目的、下に今回の修正と他の機能での作業を配置。'),
           dict(slide=6, group='japanese', change='目的は大きいほどよいという説明を避け、実現したいことに合う目的の大きさを考えると明記。参照漏れという固定原因を削除。'),
           dict(slide=7, group='story', change='教材1の業務の目的を受け、条件を表す言葉の意味と適用範囲を確かめ、各担当者とAIが判断の根拠に使う教材2へ接続。'),
           dict(slide=7, group='figure', change='既存の人物・操作制限・申請データの図を維持。短い説明と具体的な条件、確認する範囲の関係を示す。'),
           dict(slide=7, group='japanese', change='短い説明を新たな原因と断定せず、既存の仕様とコードを照合して意味を確かめる例として説明。'),
           dict(slide=8, group='story', change='共通の業務条件を役割ごとのAIの手順で使い、PR承認基準に照らして確認できる根拠も出す学びへ接続。'),
           dict(slide=8, group='figure', change='共通の参照情報から各AgentとSkillへつながる既存図を維持。条件とコードの対応、照合した条件、条件ごとの実行結果を出力として明記。'),
           dict(slide=8, group='japanese', change='関連仕様を参照していなかったという旧原因を削除。人が条件と根拠を確認し、参照情報と手順を改善する流れを説明。'),
           dict(slide=9, group='story', change='別の機能の条件を確認して実行し、根拠の正しさと人が確認する負担を見て改善する流れへ更新。'),
           dict(slide=9, group='figure', change='既存の循環図を維持。条件と実装・テストを照合し、参照情報の意味・範囲やSkillの照合手順・出力を見直す内容へ変更。'),
           dict(slide=9, group='japanese', change='AIの確認済みという説明だけで判断せず、仕様・コード・実際のテスト結果と照合すると明記。試す範囲は目的に応じて考える。'),
           dict(slide=10, group='story', change='資料も確認もあるが、AIで増えた成果物と指摘確認への偏りにより条件を見逃し、結合テストで発覚したという事例を振り返る。'),
           dict(slide=10, group='figure', change='既存の三教材と担当業務・チームの確認・仕事全体の改善の図を維持。業務担当者も確認する関係を示し、必要な確認を省くと読める表現を変更。'),
           dict(slide=10, group='japanese', change='資料を参照していなかったという旧原因を削除。PM・PL・Agent・Skillはナレーションで正式名称または発音する表記を使用。')]
    (A / 'log-course.json').write_text(json.dumps(log, ensure_ascii=False, indent=2) + '\n')

if __name__ == '__main__':
    import sys
    if '--publish' in sys.argv:
        publish()
