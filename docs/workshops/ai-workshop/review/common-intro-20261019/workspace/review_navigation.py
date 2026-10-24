"""Generate the review hub and routes without changing teaching-slide content."""
import hashlib
import html
import json
import re
import markdown


def build_navigation(here, viewer):
    repo = here.parents[5]
    shared = repo / 'docs/workshops/ai-workshop/review/shared'
    progress = json.loads((here / 'seven-slide-revision-20261019/progress.json').read_text())
    other = json.loads((shared / 'tracks/proposal-orientation.json').read_text())
    awaiting = progress.get('awaiting_user_review')
    number = awaiting if isinstance(awaiting, int) else 1
    review = f'design-review.html?deck=lead&slide={number}#lead'
    reader = f'materials.html?slide={number}#lead'
    pending = [str(n) for n in (6, 7) if n not in progress.get('story_reconsideration_applied_slides', [])]
    pending_note = '・'.join(pending) + '枚目は再構成方針への修正前です。' if pending else ''
    theme = (repo / '.claude/skills/slide-deck/references/themes/warm-paper.css').read_text()
    css = theme + '\n' + (here / 'review-navigation.css').read_text()
    (viewer / 'review-navigation.css').write_text(css)

    def page(title, content, script=''):
        return ('<!doctype html><html lang="ja"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">'
                '<title>' + html.escape(title) + '</title><link rel="stylesheet" href="review-navigation.css"><header>'
                '<p class="eyebrow">AIワークショップ · 制作と閲覧</p><h1>' + html.escape(title) + '</h1>'
                '<nav class="site-nav" aria-label="メインの作業"><a href="./">メイン画面</a><a href="' + review + '">変更を確認</a>'
                '<a href="' + reader + '">資料を通して見る</a><a href="workflow.html">制作方針・進捗</a><a href="navigation.html">画面遷移図</a></nav></header>'
                '<main>' + content + '</main><footer>スライド・原稿の確認状態 ／ 動画・音声の更新状態を個別に表示</footer>' + script + '</html>')

    status = f'{awaiting}枚目を確認待ち' if isinstance(awaiting, int) else '構成を再確認中'
    home = f'''<p>何を確認するかを選んでください。修正の比較、通しの閲覧、制作方針と進捗を分けています。</p>
<div class="task-grid" id="main-tasks">
<section class="card"><p class="task-index">01 · REVIEW</p><h2>変更を確認する</h2><p>図・原稿の変更前後と、その変更理由を確認します。</p><span class="status">はじめに：{html.escape(status)}</span><div class="actions"><a class="primary" id="resume-review" href="{review}">確認中のスライドを開く</a></div></section>
<section class="card"><p class="task-index">02 · READ</p><h2>資料を通して見る</h2><p>スライドと原稿を一枚ずつ読み、動画・音声も確認できます。</p><p class="muted">はじめにの動画・音声は変更前です。</p><div class="actions"><a class="primary" href="{reader}">スライドと原稿を見る</a></div></section>
<section class="card"><p class="task-index">03 · WORK</p><h2>制作方針・進捗を見る</h2><p>共通方針、担当ごとの確認待ちと残作業、制作手順を確認します。</p><div class="actions"><a class="primary" href="workflow.html">方針と担当を確認する</a></div></section>
</div>
<h2 class="section-title">資料から選ぶ</h2><div class="material-list">
<section class="card"><h3>企画書</h3><p>部長向けの現行資料。新しい共通方針での修正は別担当が進めます。</p><div class="actions"><a href="materials.html#brief">要点版を見る</a><a href="materials.html#detail">詳細版を見る</a><a href="design-review.html?deck=brief&slide=1#slide-comparison">変更を確認</a></div></section>
<section class="card"><h3>オリエンテーション</h3><p>受講者向けの現行資料。通しの閲覧と、保存された変更前後の比較を選べます。</p><div class="actions"><a href="materials.html#orientation">スライドと原稿</a><a href="materials.html?mode=video#orientation">動画・音声</a><a href="design-review.html?deck=orientation&slide=1#slide-comparison">変更を確認</a></div></section>
<section class="card"><h3>共通の「はじめに」</h3><p>三教材に入る前の題材説明。{html.escape(status)}。{pending_note}</p><div class="actions"><a href="{review}">修正を確認</a><a href="materials.html#lead">最初から読む</a></div></section>
</div><details><summary>参考資料と画面の見方</summary><div class="actions"><a href="navigation.html">画面遷移図</a><a href="asset-gallery/">過去の画像・SVG一覧</a><a href="design-review.html?deck=lead&slide={number}#reference-materials">過去の案・デザインの参考</a><a href="handoff.html">技術的な引き継ぎ</a></div></details>'''
    redirect = '<script>function legacyRoute(){if(["brief","detail","orientation","lead"].includes(location.hash.slice(1)))location.replace("materials.html"+location.search+location.hash);}addEventListener("hashchange",legacyRoute);legacyRoute();</script>'
    (viewer / 'index.html').write_text(page('AIワークショップの確認・制作', home, redirect))

    # Preserve the former reader as its own page, with current draft notes for the common intro.
    material = (here / 'materials-source.html').read_text()
    match = re.search(r'(<script type="application/json" id="data">)(.*?)(</script>)', material, re.S)
    decks = json.loads(match[2])
    lead = next(deck for deck in decks if deck['id'] == 'lead')
    slides = [json.loads((here / f'slide-{n:02}.json').read_text()) for n in range(1, 8)]
    version = hashlib.sha256((here / 'after.html').read_bytes()).hexdigest()[:12]
    lead.update(slides='common-intro-review/after.html?v=' + version, script='common-intro-review/script.md', draft=True,
                summary='確認用の7枚構成です。' + status + '。' + pending_note + '動画・音声は変更前の版です。',
                notes=[dict(number=s['slide'], title=s['heading'], body=markdown.markdown(s['narration'])) for s in slides])
    lead.pop('pdf', None)
    material = material[:match.start(2)] + json.dumps(decks, ensure_ascii=False).replace('</', '<\\/') + material[match.end(2):]
    material = material.replace('<body>', '<body><nav class="reader-nav" aria-label="制作画面への移動"><a href="./">← メイン画面</a><a id="compare-current" href="' + review + '">この資料の変更を確認</a><a href="workflow.html">制作方針・進捗</a><a href="navigation.html">画面遷移図</a></nav>', 1)
    material = re.sub(r'<p><a href="handoff.html">.*?</a></p>', '', material, count=1)
    material = re.sub(r'<p><a href="asset-gallery/"[^>]*>.*?</a></p>', '', material, count=1)
    material = re.sub(r'<p><a href="design-review.html#lead">.*?</a></p>', '', material, count=1)
    material = material.replace('</style>', '.reader-nav{display:flex;gap:10px;flex-wrap:wrap;max-width:1540px;margin:16px auto;padding:0 32px}.reader-nav a{padding:8px 13px;background:var(--paper);border:1px solid var(--line);border-radius:9px} </style>', 1)
    material = material.replace('<h1>企画書・オリエンテーション・教材</h1>', '<h1>資料を通して見る</h1>')
    material = material.replace("mode='slides'", "mode=new URLSearchParams(location.search).get('mode')==='video'?'video':'slides'")
    material = material.replace("el('slide-select').value=number;", "el('slide-select').value=number;el('compare-current').href='design-review.html?deck='+encodeURIComponent(active.id)+'&slide='+number+'#'+encodeURIComponent(active.id);")
    material = material.replace("const note=active.notes[number-1];", "const note=active.notes[number-1];const stateUrl=new URL(location.href);stateUrl.searchParams.set('slide',String(number));stateUrl.hash=active.id;history.replaceState(null,'',stateUrl);")
    material = material.replace("function setMode(m){mode=m;", "function setMode(m){mode=m;const modeUrl=new URL(location.href);modeUrl.searchParams.set('mode',m);history.replaceState(null,'',modeUrl);")
    marker = "choose(location.hash.slice(1)||'brief');"
    assert marker in material
    material = material.replace(marker, "const requestedSlide=Number(new URLSearchParams(location.search).get('slide'));" + marker + "if(Number.isInteger(requestedSlide)&&requestedSlide>0)setNumber(requestedSlide);")
    (viewer / 'materials.html').write_text(material)

    mapping = { (shared / 'direction.md').resolve(): '#direction', (shared / 'README.md').resolve(): '#operation',
                (shared / 'tracks').resolve(): '#tracks', (shared / 'changes').resolve(): '#changes',
                (shared / 'start-proposal-orientation.md').resolve(): '#session-start',
                (shared / 'viewer-guide.md').resolve(): '#viewer-guide',
                (repo / '.claude/skills/slide-deck/references/ai-workshop-production.md').resolve(): '#production' }
    def doc(path):
        source = path.read_text()
        def link(m):
            label, target = m[1], m[2]
            if '://' in target or target.startswith('#'):
                return m[0]
            resolved = (path.parent / target).resolve()
            if resolved in mapping:
                return '[' + label + '](' + mapping[resolved] + ')'
            return label + '（参照ファイル：`' + target + '`）'
        source = re.sub(r'\[([^\]]+)\]\(([^)]+)\)', link, source)
        rendered = markdown.markdown(source, extensions=['tables', 'fenced_code'])
        rendered = re.sub(r'<(/?)h([1-5])>', lambda m: '<' + m[1] + 'h' + str(int(m[2]) + 1) + '>', rendered)
        return '<div class="doc-content">' + rendered + '</div>'
    other_status = {'not_started': '開始待ち', 'in_progress': '作業中', 'awaiting_review': '確認待ち', 'complete': '完了'}.get(other.get('status'), other.get('status', '未記録'))
    workflow = '<p>共有ファイルから生成した制作情報である。方針への同意と、各スライドの確認状態は別に扱う。</p><article id="tracks"><h2>担当別の進捗</h2><table><thead><tr><th>担当</th><th>現在の状態</th><th>次の確認先</th></tr></thead><tbody>'
    workflow += '<tr><td>共通の「はじめに」</td><td>' + html.escape(status) + '</td><td><a href="' + review + '">確認中のスライドへ</a></td></tr>'
    workflow += '<tr><td>企画書・オリエンテーション</td><td>' + html.escape(other_status) + '</td><td>' + html.escape(other.get('next', '未記録')) + '</td></tr></tbody></table><p class="source-note">' + html.escape(progress.get('note', '')) + '</p></article>'
    workflow += '<article id="direction">' + doc(shared / 'direction.md') + '</article>'
    workflow += '<details id="viewer-guide"><summary>共同制作で使う画面・生成元・更新方法</summary>' + doc(shared / 'viewer-guide.md') + '</details>'
    workflow += '<details id="production"><summary>共通の制作手順と原稿・図の注意点</summary>' + doc(repo / '.claude/skills/slide-deck/references/ai-workshop-production.md') + '</details>'
    workflow += '<details id="operation"><summary>共同制作の進め方</summary>' + doc(shared / 'README.md') + '</details>'
    workflow += '<details id="changes"><summary>方針と共同作業の変更記録</summary>' + ''.join(doc(p) for p in sorted((shared / 'changes').glob('*.md'), reverse=True)) + '</details>'
    workflow += '<details id="session-start"><summary>別セッションの開始用プロンプト</summary>' + '<pre>' + html.escape((shared / 'start-proposal-orientation.md').read_text()) + '</pre>' + '</details>'
    open_anchor = '<script>function reveal(){let n=document.getElementById(location.hash.slice(1));while(n){if(n.tagName==="DETAILS")n.open=true;n=n.parentElement;}}addEventListener("hashchange",reveal);reveal();</script>'
    (viewer / 'workflow.html').write_text(page('制作方針と進捗', workflow, open_anchor))

    svg = (here / 'navigation.svg').read_text().replace('slide=5', 'slide=' + str(number))
    diagram = '<p>太い枠がメイン画面、上段の三つが遷移先である。下段は各画面内での切替・確認内容を示す。</p><article class="diagram"><p class="diagram-hint muted">小さい画面では、図を横にスクロールして全体を確認できる。</p>' + svg + '</article>'
    diagram += '<article><h2>確認の順序</h2><ol class="flow-list"><li>メイン画面で、修正の比較・通しの閲覧・制作情報のどれを見るか選ぶ。</li><li>比較画面では資料と枚を選び、図・原稿・変更理由を同じ画面で確認する。</li><li>前後の流れを読みたくなったら「この資料を通して見る」へ進む。選択した資料と枚を維持する。</li><li>資料閲覧画面から「この資料の変更を確認」で同じ枚の比較へ戻る。</li><li>別の作業へ移るときは「メイン画面」へ戻る。</li></ol><p>過去の画像・SVGはメイン画面の参考資料から開ける。スライドの拡大と、PDF・原稿のファイル表示は別タブで開く。</p></article>'
    diagram += '<details><summary>Mermaidの画面遷移図を表示</summary><pre>' + html.escape((here / 'navigation.mmd').read_text()) + '</pre></details>'
    (viewer / 'navigation.html').write_text(page('画面遷移図と操作の流れ', diagram))
