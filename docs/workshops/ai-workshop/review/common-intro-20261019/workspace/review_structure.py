"""Show the proposed slide structure alongside the current comparison."""
import html
import json
import markdown


def add_structure(text, root):
    path = root / 'slide-structure.json'
    if not path.exists():
        return text
    plan = json.loads(path.read_text())
    esc = html.escape
    cards = []
    for slide in plan['slides']:
        n = slide['number']
        before = slide['before']
        fields = ''.join('<p><strong>' + label + '</strong> ' + esc(slide[key]) + '</p>'
                         for label, key in [('この枚の主張：', 'claim'), ('受講者に残す問い：', 'question'),
                                            ('図で表す関係：', 'visual'), ('引き継ぐ図の要素：', 'keep'),
                                            ('次の枚への接続：', 'next')])
        cards.append(
            f'<article class="structure-card" id="structure-slide-{n}"><p class="eyebrow">内容{n}枚目</p>'
            '<p class="muted">' + esc(slide['status']) + '</p>'
            '<div class="structure-pair"><section><h3>改訂前の構成</h3><h4>' + esc(before['heading']) + '</h4><p>' + esc(before['lede']) + '</p></section>'
            '<section><h3>改訂する構成</h3><h4>' + esc(slide['heading']) + '</h4><p>' + esc(slide['lede']) + '</p></section></div>'
            + fields + '<details><summary>変更前と改訂案の原稿を読む</summary><div class="structure-pair"><section><h3>改訂前の原稿</h3>'
            + markdown.markdown(before['narration']) + '</section><section><h3>改訂原稿案</h3>'
            + markdown.markdown(slide['narration']) + '</section></div></details>'
            f'<p><a href="?deck=lead&amp;slide={n}#lead">図と原稿をスライド比較で確認 →</a></p></article>')
    nav = '<nav class="task-nav" aria-label="構成案の枚を選ぶ">' + ''.join(
        f'<a href="#structure-slide-{n}">{n}枚目</a>' for n in range(1, len(cards) + 1)) + '</nav>'
    applied = '、'.join(str(n) for n in plan.get('applied_slides', []))
    panel = ('<details id="intro-structure" class="reference-group"><summary>要件と設計を軸に改訂する8枚の構成</summary>'
             '<p>' + esc(plan['purpose']) + '</p>'
             '<p class="muted">ここで確認するのは、8枚の見出し・導入文・原稿案と図の修正方針です。'
             + applied + '枚目は図と原稿へ反映済みです。ほかの枚は構成案で、図とスライド本体を一枚ずつ修正・確認します。</p>'
             + nav + ''.join(cards) + '</details>')
    text = text.replace('<div class="review-controls">', panel + '<div class="review-controls">', 1)
    text = text.replace('<nav class="task-nav" aria-label="この比較で確認する内容">',
                        '<nav class="task-nav" aria-label="この比較で確認する内容">'
                        '<a href="?deck=lead&amp;structure=1#lead">改訂する8枚の構成</a>', 1)
    style = '''<style>
    #intro-structure{margin:0 0 24px} .structure-card{border:1px solid var(--diagram-line);border-radius:12px;padding:20px;margin:20px 0;background:var(--paper);scroll-margin-top:24px}
    .structure-card h4{font-size:19px;margin:8px 0;line-height:1.6}.structure-card p{overflow-wrap:anywhere}
    .structure-pair{display:grid;grid-template-columns:1fr 1fr;gap:16px}.structure-pair>section{border:1px solid var(--line);border-radius:9px;padding:16px;min-width:0;background:var(--paper)}
    .structure-pair h3{font-size:14px;color:var(--dim);margin:0 0 10px}.structure-card details{margin-top:16px}.structure-card summary{padding:8px 0;cursor:pointer;font-weight:700}
    @media(max-width:850px){.structure-pair{grid-template-columns:1fr}.structure-card{padding:12px}.structure-pair>section{padding:12px}}
    </style>'''
    script = '''<script>
    if(new URLSearchParams(location.search).get('structure')==='1'){
      const panel=document.getElementById('intro-structure');panel.open=true;
      requestAnimationFrame(()=>panel.scrollIntoView({block:'start',behavior:'instant'}));
    }
    </script>'''
    return text.replace('</html>', style + script + '</html>', 1)
