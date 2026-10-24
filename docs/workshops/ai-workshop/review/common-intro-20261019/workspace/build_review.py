from pathlib import Path
import hashlib
import html
import json
import re
import markdown
from review_ui import enhance
from review_navigation import build_navigation
from review_deck import load_deck
from review_structure import add_structure

A = Path(__file__).resolve().parent
V = A.parent / 'viewer'
slides, layout = load_deck(A)
for approval in sorted(A.glob('approved-slide-*-hashes.json')):
    for name, expected in json.loads(approval.read_text()).items():
        assert hashlib.sha256((A / name).read_bytes()).hexdigest() == expected, '承認済みの枚が変わっています: ' + name
base = (A / 'baseline-after.html').read_text()
parts = re.findall(r'<section\b.*?</section>', base, re.S)


def figure(source, root=A):
    return (root / source.replace('.json', '.svg')).read_text().replace('<svg ', '<svg class="ws-visual" ', 1)


def section(slide, svg, version):
    return ('<section class="slide ws-diagram"><div class="kicker"><span class="ws-mark" aria-hidden="true"></span>'
            '<span class="ws-kicker-label">3教材共通 はじめに ／ 内容' + str(slide['slide']) + '枚目（' + version + '）</span></div>'
            '<h2>' + html.escape(slide['heading']) + '</h2><p class="ws-intro">' + html.escape(slide['lede']) + '</p>'
            '<div class="body">' + svg + '</div></section>')


def deck(sections, labels):
    result = base.replace(parts[0], ''.join(sections), 1)
    for old in parts[1:]:
        result = result.replace(old, '', 1)
    return re.sub(r'const LABELS = .*?;', lambda _: 'const LABELS = ' + json.dumps(labels, ensure_ascii=False) + ';', result)


def notes(slide, number):
    return dict(number=number, title=slide['heading'], body='<h4>見出し</h4><p>' + html.escape(slide['heading']) + '</p><h4>導入文</h4><p>'
                + html.escape(slide['lede']) + '</p><h4>原稿</h4>' + markdown.markdown(slide['narration']))


after = deck([section(s, figure(s['source_file']), '確認用') for s in slides], [s['heading'] for s in slides])
(A / 'after.html').write_text(after)
(A / 'script.md').write_text('# 三教材共通のはじめに：確認用の原稿\n\n' + '\n\n'.join('## ' + str(s['slide']) + ' ' + s['heading'] + '\n\n' + s['narration'] for s in slides) + '\n')
before_sections, before_labels, oldnotes = [], [], []
for n, source in enumerate(layout['before_sources'], 1):
    if source is None:
        missing = dict(slide=n, heading='新規追加：変更前のスライドはありません', lede='この位置に、想定される展開を説明するスライドを追加しました。')
        before_sections.append(section(missing, '<p>以前の教材紹介は、変更後の6〜8枚目へ移動しています。</p>', '追加前'))
        before_labels.append(missing['heading'])
        oldnotes.append(None)
    else:
        prior = json.loads((A / layout['baseline'] / source).read_text())
        prior['slide'] = n
        before_sections.append(section(prior, figure(source, A / layout['baseline']), '変更前'))
        before_labels.append(prior['heading'])
        oldnotes.append(notes(prior, n))
(A / 'before.html').write_text(deck(before_sections, before_labels))


def publish():
    (V / 'common-intro-review').mkdir(parents=True, exist_ok=True)
    for filename in ['before.html', 'after.html', 'script.md']:
        (V / 'common-intro-review' / filename).write_bytes((A / filename).read_bytes())
    def version(side):
        return hashlib.sha256((A / (side + '.html')).read_bytes()).hexdigest()[:12]
    text = (A / 'baseline-design-review.html').read_text()
    match = re.search(r'(<script type="application/json" id="data">)(.*?)(</script>)', text, re.S)
    data = json.loads(match[2])
    lead = next(d for d in data if d['id'] == 'lead')
    newnotes = [notes(s, s['slide']) for s in slides]
    lead.update(name=f'3教材共通 · はじめに（1〜{len(slides)}枚目）',
                before='common-intro-review/before.html?v=' + version('before'), after='common-intro-review/after.html?v=' + version('after'),
                notes=newnotes, beforeNotes=oldnotes, afterNotes=newnotes, reasons=layout['reasons'], addedSlides=layout['added_slides'])
    structure_path = A / 'slide-structure.json'
    if structure_path.exists():
        structure = json.loads(structure_path.read_text())
        lead['structureAppliedSlides'] = structure.get('applied_slides', [])
        lead['structureApprovedSlides'] = structure.get('approved_slides', [])
    text = text[:match.start(2)] + json.dumps(data, ensure_ascii=False).replace('</', '<\\/') + text[match.end(2):]
    start = text.index('<article id="lead-review-notice">')
    end = text.index('</article>', start)
    notice = text[start:end]
    notice = re.sub(r'<h2>.*?</h2>', '<h2>共通の「はじめに」：要件と設計を軸に8枚の構成を改訂中</h2>', notice, count=1, flags=re.S)
    notice = re.sub(r'<p>.*?</p>', '<p>1〜4枚目は架空案件で起きた出来事、5枚目はその設計のまま変更を続けた場合に想定される展開です。教材1〜3の紹介は6〜8枚目へ移動しています。新しい5枚目には変更前のスライドと原稿はありません。動画・音声と正本は更新していません。</p>', notice, count=1, flags=re.S)
    notice = re.sub(r'<section><h3>表紙の次：共通の題材を説明</h3>.*?</section>', '', notice, flags=re.S)
    notice = notice.replace('更新した動画・音声の再生ページ', '現行の動画・音声（今回の修正は未反映）')
    plan = '# 現在の8枚構成\n\n' + '\n\n'.join('## ' + str(s['slide']) + ' ' + s['heading'] + '\n\n' + s['lede'] for s in slides)
    notice += '<details id="seven-slide-plan"><summary>現在の8枚構成を開く</summary>' + markdown.markdown(plan) + '</details>'
    text = add_structure(enhance(text[:start] + notice + text[end:]), A)
    (V / 'design-review.html').write_text(text)
    (A / 'comparison.html').write_text(text)
    build_navigation(A, V)
    history = json.loads((A / layout['baseline'] / 'log-course.json').read_text())
    # Historical slide numbers refer to the preceding seven-slide layout.
    log = [dict(entry, layout='以前の7枚構成') for entry in history]
    log.extend([dict(slide=5, group='story', change='事例の事実と想定される展開を分け、条件や処理の分散から確認範囲が増える可能性を説明するスライドを追加。'),
                dict(slide=5, group='figure', change='担当者による追加、ルールの分散、照合対象の増加を人物・コード・同じ大きさで重ねた資料で図示。'),
                dict(slide=5, group='japanese', change='起こり得る一例として説明。AI使用や類似コードだけを悪い設計の根拠にせず、途中の見直しで防げると明記。'),
                dict(slide=6, group='story', change='以前の教材1の紹介を移動。新しい前枚の想定例を受けた導入へ変更。'),
                dict(slide=1, group='story', change='案件を紹介する1枚目に、業務ルールの継承・構成変更・刷新後の仕様設計資料の更新を追加。要件と設計からAIを活用するチームと品質・確認修正を含む時間短縮の目標を示す。'),
                dict(slide=1, group='figure', change='人物とチームを維持し、右側に引き継ぐ業務ルールと見直す構成・更新する資料を線画で表示。チームのAI活用目標は下段に別枠で示す。warm-paperを指定して解決。'),
                dict(slide=6, group='story', change='主体・対象が不明な設計実装の目的という表現を修正。申請者の目的の大きさ、目的を満たす要件、AIへ任せる開発作業を区別。'),
                dict(slide=6, group='figure', change='申請者の目的、具体的な要件、開発者の目的とAI作業を別の所属へ配置。人物・申請書・証憑・承認の維持と再確認・コード・レビューのアイコンで対象と動作を視覚化。'),
                dict(slide=2, group='story', change='改訂構成の見出し・導入文・原稿を反映。刷新で構成と仕様・設計資料も更新する設定を追加。業務を知る人が差分・AIの説明・テスト結果を確認し、要件と確認結果の対応を追う必要があった進め方と、次の結合テストへの接続を示す。'),
                dict(slide=2, group='figure', change='資料・開発者・AI・重ねた成果物・テスト・人のレビュー・承認を既存の線画で示し、重複する説明文を短縮。warm-paperを指定して描画。'),
                dict(slide=3, group='story', change='レビュー通過後に結合テストで承認段階の取り違えが発覚した流れを改訂構成へ反映。AIと人が要件と確認結果の対応を見逃した過程、修正と再確認、残った設計を見る次の枚へ接続。'),
                dict(slide=3, group='figure', change='本来と今回の実装の左右比較を維持。同じ申請変更を人物・ペン・鍵・金額で対応させ、取り込み前の未確認ケースと再作業の順序を線画で追加。')])
    (A / 'log-course.json').write_text(json.dumps(log, ensure_ascii=False, indent=2) + '\n')


if __name__ == '__main__':
    import sys
    if '--publish' in sys.argv:
        publish()
