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
if layout.get('snapshot_manifest'):
    saved_manifest = A / layout['snapshot_manifest']
    for entry in json.loads(saved_manifest.read_text())['files']:
        saved = saved_manifest.parent / entry['path']
        assert hashlib.sha256(saved.read_bytes()).hexdigest() == entry['sha256'], '保存した変更前が変わっています: ' + entry['path']
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
cover = json.loads((A / 'cover.json').read_text()) if (A / 'cover.json').exists() else None
if cover:
    cover_section = ('<section class="slide ws-titlepage"><h1>' + html.escape(cover['heading'])
                     + '</h1><p class="ws-intro">' + html.escape(cover['lede']) + '</p><div class="body">'
                     + (A / 'cover.svg').read_text() + '</div></section>')
    (A / 'cover.html').write_text(deck([cover_section], ['表紙：はじめに']))
    (A / 'presentation.html').write_text(deck([cover_section] + [section(s, figure(s['source_file']), '確認用') for s in slides], ['表紙：はじめに'] + ['本文' + str(s['slide']) + '：' + s['heading'] for s in slides]))

(A / 'script.md').write_text('# 三教材共通のはじめに：確認用の原稿\n\n' + '\n\n'.join('## ' + str(s['slide']) + ' ' + s['heading'] + '\n\n' + s['narration'] for s in slides) + '\n')
if cover:
    script = (A / 'script.md').read_text()
    (A / 'script.md').write_text(script.replace('確認用の原稿\n\n', '確認用の原稿\n\n## 表紙：はじめに\n\n' + cover['narration'] + '\n\n', 1))
before_sections, before_labels, oldnotes = [], [], []
for n, source in enumerate(layout['before_sources'], 1):
    prior_root = A / layout['baseline']
    override = layout.get('before_overrides', {}).get(str(n))
    if override:
        prior_root = A / override
        source = Path(layout['sources'][n - 1]).name
    if source is None:
        description = layout.get('added_slide_descriptions', {}).get(str(n), {})
        missing = dict(slide=n, heading='新規追加：変更前のスライドはありません', lede=description.get('lede', '新しいスライドを追加しました。'))
        before_sections.append(section(missing, '<p>' + html.escape(description.get('body', '変更前の図と原稿はありません。')) + '</p>', '追加前'))
        before_labels.append(missing['heading'])
        oldnotes.append(None)
    else:
        prior = json.loads((prior_root / source).read_text())
        prior['slide'] = n
        before_sections.append(section(prior, figure(source, prior_root), '変更前'))
        before_labels.append(prior['heading'])
        oldnotes.append(notes(prior, n))
(A / 'before.html').write_text(deck(before_sections, before_labels))


def publish():
    (V / 'common-intro-review').mkdir(parents=True, exist_ok=True)
    for filename in ['before.html', 'after.html', 'script.md', 'cover.html', 'presentation.html']:
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
                readerSlideOffset=1 if cover else 0, notes=newnotes, beforeNotes=oldnotes, afterNotes=newnotes, reasons=layout['reasons'], addedSlides=layout['added_slides'])
    structure_path = A / 'slide-structure.json'
    if structure_path.exists():
        structure = json.loads(structure_path.read_text())
        lead['structureAppliedSlides'] = structure.get('applied_slides', [])
        lead['structureApprovedSlides'] = structure.get('approved_slides', [])
    repo = A.parents[5]
    orientation_track = json.loads((repo / 'docs/workshops/ai-workshop/review/shared/tracks/proposal-orientation.json').read_text())
    if orientation_track.get('orientation_review_input'):
        override = json.loads((repo / orientation_track['orientation_review_input']).read_text())
        next(item for item in data if item['id'] == 'orientation').update(override)
    text = text[:match.start(2)] + json.dumps(data, ensure_ascii=False).replace('</', '<\\/') + text[match.end(2):]
    start = text.index('<article id="lead-review-notice">')
    end = text.index('</article>', start)
    notice = text[start:end]
    notice = re.sub(r'<h2>.*?</h2>', '<h2>共通の「はじめに」：要件と設計を軸に8枚の構成を改訂中</h2>', notice, count=1, flags=re.S)
    notice = re.sub(r'<p>.*?</p>', '<p>1〜4枚目は架空案件で起きた出来事、5枚目はその設計のまま変更を続けた場合に想定される展開です。教材1〜3の紹介は6〜8枚目へ移動しています。5〜8枚目の変更前は、今回の見直し前の作業案です。動画・音声と正本は更新していません。</p>', notice, count=1, flags=re.S)
    notice = re.sub(r'<section><h3>表紙の次：共通の題材を説明</h3>.*?</section>', '', notice, flags=re.S)
    notice = notice.replace('更新した動画・音声の再生ページ', '現行の動画・音声（今回の修正は未反映）')
    progress = json.loads((A / 'progress.json').read_text())
    new_story = A / 'whole-project-story-20261025'
    if layout.get('story_revision'):
        archived_public = V / 'whole-project-before-20261025'
        archived_public.mkdir(exist_ok=True)
        for name in ['presentation.html', 'cover.html', 'after.html', 'script.md']:
            (archived_public / name).write_bytes((new_story / 'before' / name).read_bytes())
        notice = re.sub(r'<h2>.*?</h2>', '<h2>開発全体の生産性・品質を軸に改訂：本文' + '・'.join(map(str, layout.get('story_applied_slides', [1]))) + 'を反映</h2>', notice, count=1, flags=re.S)
        notice = re.sub(r'<p>.*?</p>', '<p>変更前は、今回の改訂直前の表紙と本文8枚を保存した版です。変更後は本文1から一枚ずつ確認しています。本文6は直前のPM・PL入力案を変更前に保存し、三つの問題と教材の学びを対応させる内容へ再構成しました。本文5は末尾の予告だけを変更しています。教材1〜3の紹介は本文7〜9へ移動し、その内容の見直しはこれからです。正本・音声・動画は未変更です。</p>', notice, count=1, flags=re.S)
        notice += '<details id="whole-project-story-plan"><summary>今回の改訂構成と反映状況</summary>' + markdown.markdown((new_story / 'plan.md').read_text(), extensions=['tables']) + '<p><a href="whole-project-before-20261025/presentation.html#1" target="_blank">保存した旧版を表紙から見る</a> ／ <a href="whole-project-before-20261025/script.md">旧版の原稿</a></p></details>'
    if progress.get('source_updated'):
        notice = re.sub(r'<p>.*?</p>', '<p>全10枚の図と原稿を利用者が確認し、ACDR 0205で正本へ正式反映しました。音声・動画は改訂前です。変更前後の比較と旧版の保存は引き続き利用できます。</p>', notice, count=1, flags=re.S)
    elif not progress.get('unrevised_slides'):
        notice = notice.replace('教材1〜3の紹介は本文7〜9へ移動し、その内容の見直しはこれからです。', '教材1〜3の紹介も最新の内容へ反映しています。')
    if progress.get('audio_video_updated'):
        notice = notice.replace('音声・動画は改訂前です。', '音声・動画も最新の全10枚へ更新しました。')
        notice = notice.replace('現行の動画・音声（今回の修正は未反映）', '最新の動画・音声（全10枚）')
    if cover:
        notice += ('<details id="intro-cover" open><summary>追加した表紙と原稿</summary><p>新規追加のため変更前はありません。本文1〜8の比較番号は維持しています。通し閲覧は表紙と本文の全9枚です。</p>'
                   + '<iframe src="common-intro-review/cover.html#1" title="はじめにの表紙" style="width:100%;aspect-ratio:16/9;border:0"></iframe>'
                   + '<h3>表紙の原稿</h3>' + markdown.markdown(cover['narration'])
                   + '<p><a href="materials.html?slide=1#lead">表紙から図と原稿を通して見る</a></p></details>')
    if layout.get('story_revision'):
        notice = notice.replace('新規追加のため変更前はありません。本文1〜8の比較番号は維持しています。通し閲覧は表紙と本文の全9枚です。', '表紙は保存した旧版から変更していません。現在は本文' + str(len(slides)) + '枚、表紙を含む全' + str(len(slides)+1) + '枚です。新本文6を追加し、旧教材紹介6〜8を7〜9へ移動しました。')
    if orientation_track.get('orientation_review_plan'):
        orientation_plan = repo / orientation_track['orientation_review_plan']
        notice += '<details id="orientation-plan" open><summary>オリエンテーションの構成見直し案（図と原稿は未修正）</summary>' + markdown.markdown(orientation_plan.read_text(), extensions=['tables']) + '</details>'
    plan = '# 現在の本文' + str(len(slides)) + '枚構成\n\n' + '\n\n'.join('## ' + str(s['slide']) + ' ' + s['heading'] + '\n\n' + s['lede'] for s in slides)
    notice += '<details id="seven-slide-plan"><summary>現在の本文' + str(len(slides)) + '枚構成を開く</summary>' + markdown.markdown(plan) + '</details>'
    final_review = A / 'final-sequence-review-20261025'
    if (final_review / 'summary.md').exists():
        public_review = V / 'final-sequence-review'
        public_review.mkdir(exist_ok=True)
        for report in final_review.glob('*.md'):
            (public_review / report.name).write_text(report.read_text())
        notice += '<details id="final-sequence-review"><summary>以前の8枚案の通し確認結果を開く</summary>' + markdown.markdown((final_review / 'summary.md').read_text(), extensions=['tables']) + '</details>'
    roles_feedback = A / 'roles-luna-review-20261025' / 'feedback.md'
    if roles_feedback.exists():
        roles_public = V / 'roles-luna-review'
        roles_public.mkdir(exist_ok=True)
        (roles_public / 'feedback.md').write_text(roles_feedback.read_text())
        notice += ('<details id="roles-luna-feedback"><summary>受講者Lunaのフィードバック：各役割のAI活用</summary>'
                   + '<p>改訂前の表紙と本文8枚に対する受講者役の感想と提案です。今回の改訂の参考にしていますが、現在の変更後を評価したものではありません。</p>'
                   + markdown.markdown(roles_feedback.read_text(), extensions=['tables']) + '</details>')
    text = add_structure(enhance(text[:start] + notice + text[end:]), A)
    if progress.get('source_updated'):
        repo = A.parents[5]
        official = V / 'official/common-intro'
        official.mkdir(parents=True, exist_ok=True)
        source_out = repo / 'docs/workshops/ai-workshop/material-1/out'
        (official / 'slides.html').write_bytes((source_out / 'slides/lesson-01-lead.html').read_bytes())
        (official / 'script.md').write_bytes((source_out / 'scripts/lesson-01-lead-script.md').read_bytes())
        (V / 'decisions').mkdir(exist_ok=True)
        (V / 'decisions/0205.html').write_bytes((repo / '.acdr/0205-三教材共通導入の正式版改訂/index.html').read_bytes())
        if progress.get('audio_video_updated'):
            (V / 'decisions/0206.html').write_bytes((repo / '.acdr/0206-共通導入の音声動画更新/index.html').read_bytes())
    (V / 'design-review.html').write_text(text)
    (A / 'comparison.html').write_text(text)
    build_navigation(A, V)
    history = json.loads((A / layout.get('history_baseline', layout['baseline']) / 'log-course.json').read_text())
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
                dict(slide=3, group='figure', change='本来と今回の実装の左右比較を維持。同じ申請変更を人物・ペン・鍵・金額で対応させ、取り込み前の未確認ケースと再作業の順序を線画で追加。'),
                dict(slide=4, group='story', change='不具合の動作は修正したが、業務判断の有効性を表せない設計が残り、差し戻しと再提出を個別に追加して確認が複雑になった経緯を反映。'),
                dict(slide=4, group='figure', change='修正した動作と残った設計を上段で区別し、人物・支出書類・証憑で意味の違いを表示。再提出の要件と三つの条件追加、レビュー時に照合する対象の増加を線画で接続。')])
    additions = A / 'log-course-additions.json'
    if additions.exists():
        log.extend(json.loads(additions.read_text()))
    (A / 'log-course.json').write_text(json.dumps(log, ensure_ascii=False, indent=2) + '\n')


if __name__ == '__main__':
    import sys
    if '--publish' in sys.argv:
        publish()
