"""本質のページ（index.html）を、essence.json から作る。"""
import json
from html import escape
from pathlib import Path

P = Path(__file__).resolve().parent
d = json.loads((P / 'essence.json').read_text())
style = (P / 'style.css.part').read_text()


def i(x):
    return f'<span class="id">{escape(x)}</span>'


out = [f'<title>{escape(d["title"])}</title>', style, '<main>',
       '<header style="display:grid;gap:6px"><p class="soft">Semantic Agent Schema ・ 本質</p>'
       f'<h1>{escape(d["title"])}</h1><p>{escape(d["purpose"])}</p>'
       '<p class="soft">このページは <code>essence.json</code>（スキーマは <code>essence.schema.json</code>）から作る。'
       '2枚のページは「本質 W1」のように id で引用する。</p></header>']

out.append('<section class="rule"><h2>利用者の言葉</h2><p class="soft">原文のまま。</p>')
for w in d['words']:
    body = '<br>'.join(escape(l) for l in w['lines'])
    about = f'<p class="soft">{escape(w["about"])}</p>' if 'about' in w else ''
    out.append(f'<div style="display:grid;gap:4px"><p>{i(w["id"])}<span class="soft">{escape(w["date"])}</span></p>{about}<blockquote>{body}</blockquote></div>')
out.append('</section>')

cm = d['concept_model']
rows = ''.join(f'<tr><td>{i(s["id"])}{escape(s["name"])}</td><td>{escape(s["what"])}</td><td>{escape(s["relation"])}</td></tr>' for s in cm['schemas'])
lis = ''.join(f'<li>{i(s["id"])}{escape(s["text"])}</li>' for s in cm['statements'])
out.append('<section><h2><span class="n">1</span>概念モデル：Abstract と Concrete</h2>'
           '<div class="wrap"><table class="stack"><tr><th></th><th>何か</th><th>Semantic Agent Schema との関係</th></tr>'
           f'{rows}</table></div><ul style="list-style:none;padding:0;display:grid;gap:4px">{lis}</ul></section>')

es = d['essence']
rows = ''.join(f'<tr><td>{i(s["id"])}{escape(s["layer"])}</td><td>{escape(s["how"])}</td></tr>' for s in es['layers'])
lis = ''.join(f'<li>{i(s["id"])}{escape(s["text"])}</li>' for s in es['values'])
out.append('<section><h2><span class="n">2</span>本質</h2>'
           f'<div class="wrap"><table><tr><th>検証</th><th>何で担保するか</th></tr>{rows}</table></div>'
           f'<h3>{escape(es["values_heading"])}</h3><ul style="list-style:none;padding:0;display:grid;gap:4px">{lis}</ul></section>')

for n, key, title in [(3, 'intake', '受け取り方'), (4, 'design', '設計の進め方')]:
    lis = ''.join(f'<li>{i(s["id"])}{escape(s["text"])}</li>' for s in d[key])
    out.append(f'<section><h2><span class="n">{n}</span>{title}</h2><ul style="list-style:none;padding:0;display:grid;gap:4px">{lis}</ul></section>')

rows = ''.join(f'<tr><td>{i(p["id"])}{escape(p["name"])}</td><td><code>{escape(p["path"])}</code></td><td>{escape(p["holds"])}</td></tr>' for p in d['pages'])
out.append('<section><h2><span class="n">5</span>ページと文書</h2>'
           f'<div class="wrap"><table class="stack"><tr><th></th><th>path</th><th>持つもの</th></tr>{rows}</table></div></section>')
out.append('</main>')
(P / 'index.html').write_text('\n'.join(out) + '\n')
