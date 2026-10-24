"""ACDR の Concrete を、キーの種類（JSON Schema の標準 ・ Abstract の x- ・ 欄の名前）で色分けして全文を描く。"""
import html, json, os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(H)
c = json.load(open(os.path.join(C, 'examples', 'acdr.concrete.json')))
X9 = set(json.load(open(os.path.join(C, 'schemas', 'abstract.json')))['x-extensions'])
MAPS = {'properties', '$defs', 'x-delegates', 'x-extensions', 'patternProperties'}
cnt = {'std': 0, 'x': 0, 'field': 0}
def key(k, parent):
    if parent in ('properties',): cnt['field'] += 1; return f'<span class="k field">"{html.escape(k)}"</span>'
    if parent in ('$defs', 'x-delegates'): return f'<span class="k name">"{html.escape(k)}"</span>'
    if k.startswith('x-'): cnt['x'] += 1; return f'<span class="k x">"{html.escape(k)}"</span>'
    if parent in ('x-prompt', 'read', 'write-item', 'run', 'delegate', 'mark'): return f'<span class="k inx">"{html.escape(k)}"</span>'
    cnt['std'] += 1; return f'<span class="k std">"{html.escape(k)}"</span>'
def render(v, ind=0, parent=None, inx=False):
    sp = '  ' * ind
    if isinstance(v, dict):
        if not v: return '{}'
        items = []
        for k, x in v.items():
            ctx = k if k in MAPS or k in ('x-prompt', 'read', 'run') else None
            if parent == 'x-delegates': ctx2 = 'delegate'
            else: ctx2 = None
            kp = parent if parent in ('properties', '$defs', 'x-delegates') else ('x-prompt' if inx else (parent if parent in ('read', 'run', 'delegate', 'write-item') else None))
            sub_inx = inx or k.startswith('x-')
            nxt = 'write-item' if k == 'write' else (k if k in MAPS or k in ('read', 'run') else (ctx2 or (None if not sub_inx else 'x-prompt')))
            items.append(f'{sp}  {key(k, kp)}: {render(x, ind + 1, nxt, sub_inx)}')
        return '{\n' + ',\n'.join(items) + f'\n{sp}}}'
    if isinstance(v, list):
        if not v: return '[]'
        return '[\n' + ',\n'.join(f'{sp}  ' + render(x, ind + 1, parent if parent == 'write-item' else None, inx) for x in v) + f'\n{sp}]'
    return '<span class="v">' + html.escape(json.dumps(v, ensure_ascii=False)) + '</span>'
body = render(c)
xs = [k for k in c if k.startswith('x-')]
fields = list(c['properties'])
css = open(os.path.join(H, 'view.css')).read() + '''
pre.json{font:12px/1.55 "JetBrains Mono",ui-monospace,monospace;background:var(--card);border:1px solid var(--line);border-radius:10px;padding:10px 12px;overflow-x:auto;white-space:pre;max-height:none}
.k.std{color:var(--sub)}.k.x{background:var(--hot-bg);color:var(--hot);font-weight:700;border-radius:4px}.k.field{background:var(--accent-bg);color:var(--accent);font-weight:700;border-radius:4px}.k.name{color:var(--accent);font-weight:700}.k.inx{color:var(--hot)}.v{color:var(--ink)}
.lg span{border-radius:6px;padding:1px 8px;font-size:12.5px;margin-right:6px}
'''
role = '''<div class="tw"><table><tr><th>種類</th><th>何を書くか</th><th>この Concrete での例</th></tr>
<tr><td><span class="k field">欄</span>（properties の中）</td><td>インスタンスが持つデータの形。拡張したい欄はここに足す。形は JSON Schema の標準のキーワード（type ・ pattern ・ items ・ required ・ if/then など）で書く</td><td>''' + ' ・ '.join(f'<code>{f}</code>' for f in fields) + '''</td></tr>
<tr><td><span class="k x">x-</span>（Abstract の9つ）</td><td>JSON Schema では書けない意味だけ：読み方と書くときの決まり（x-prompt）・ 任せる先（x-delegates）・ 置き場所（x-instance-path）・ 名前の型（x-instance-label）・ 人が決める欄（x-decided-by）・ ほかを指す（x-relationship）など</td><td>''' + ' ・ '.join(f'<code>{k}</code>' for k in xs) + '''、欄の中の <code>x-prompt</code> ・ <code>x-decided-by</code> ・ <code>x-relationship</code></td></tr>
<tr><td>Concrete が足す x-（x-extensions で定義）</td><td>9つで足りない新しい意味のキーワードを必要とするときだけ。名前 ・ 値の形 ・ 説明 ・ 書くときの決まりを x-extensions に書いてから使う</td><td>いまは無い。acdr Skill のスキーマにある <code>x-view</code>（描画で隠す）を移すなら、ここで定義する</td></tr>
</table></div>'''
page = f'''<title>ACDR の Concrete スキーマ</title><style>{css}</style><main>
<h1>ACDR の Concrete スキーマ（全文）</h1>
<p class="meta">正本 examples/acdr.concrete.json。docs と acdr-inspect を戻し、docs の形をいまの acdr Skill に合わせた版（本体の検査と58の場面は合格）</p>
<div class="ans">欄を拡張するのは x- ではなく、JSON Schema の properties。x- は、JSON Schema で書けない意味（読み方 ・ 決まり ・ 任せる先 ・ 置き場所など）だけを持つ。</div>
<h2>キーの種類と役割</h2>{role}
<h2>全文（色：キーの種類）</h2>
<p class="lg"><span class="k field">欄の名前</span><span class="k x">Abstract の x-</span><span class="k inx">x- の中の項目</span><span class="k std">JSON Schema の標準のキーワード</span><span class="k name">任せる先 ・ 部品の名前</span></p>
<pre class="json">{body}</pre>
</main>'''
open(os.path.join(H, 'concrete-view.html'), 'w').write(page)
print('ok', cnt)
