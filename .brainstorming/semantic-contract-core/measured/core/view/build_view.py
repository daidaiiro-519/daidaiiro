"""核の全容を1枚に描く：ドメインモデル ・ ER 図 ・ DB スキーマ ・ Abstract（本体と一員）・ Concrete（ACDR）。正本のファイルから作る。"""
import html, json, os, re
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(H)
e = lambda s: html.escape(str(s))
m = json.load(open(os.path.join(C, 'model', 'model.json'))); ch = json.load(open(os.path.join(C, 'model', 'changes.json')))
touched = {}
for c in ch:
    p = c['path']
    if not p: continue
    key = p[1] if len(p) > 1 else p[0]
    if c['op'] == 'add': key = c.get('key') or (c['after'][0] if isinstance(c['after'], list) else c['after'][:20])
    touched.setdefault(str(key), []).append(c['id'])
mk = lambda k: (' <span class="chg">' + ' ・ '.join(touched[k]) + '</span>') if k in touched else ''
def T(cols, rows, cls=''):
    return f'<div class="tw"><table class="{cls}"><tr>' + ''.join(f'<th>{c}</th>' for c in cols) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{v}</td>' for v in r) + '</tr>' for r in rows) + '</table></div>'

# ---------- 1 ドメインモデル
dm = '<h2 id="dm">1　ドメインモデル</h2><p class="meta">正本 model/model.json（v2 に changes.py の' + str(len(ch)) + '件を適用）。<span class="chg">C01</span> のような印は、v2 から変えた行と変更の番号</p>'
dm += '<h3>業務領域</h3>' + T(['業務領域', '区分'], [[e(k), e(v)] for k, v in m['subdomains'].items()])
dm += '<h3>集約（9）</h3>' + T(['集約', '持ち主', 'ルート', '中身', '使う場面'], [[e(a['name']) + mk(a['name']), e(a['holder']), e(a['root']), e(a['members']), ' '.join(a['u'])] for a in m['aggregates']])
dm += '<h3>エンティティ</h3>' + T(['名前', '中身', '置き場所'], [[e(x[0]) + mk(x[0]), e(x[1]), e(x[2])] for x in m['entities']])
dm += f'<h3>値オブジェクト（{len(m["value_objects"])}）</h3>' + T(['名前', '中身'], [[e(x[0]) + mk(x[0]), e(x[1])] for x in m['value_objects']])
dm += f'<h3>ドメインサービス（{len(m["domain_services"])}）</h3>' + T(['名前', '中身', '業務領域', '使う場面'], [[e(x[0]) + mk(x[0]), e(x[1]), e(x[2]), ' '.join(x[3:])] for x in m['domain_services']])
dm += '<h3>ポート</h3>' + T(['駆動する側（入口）', '使う場面', 'アダプター'], [[e(x[0]) + mk(x[0]), ' '.join(x[1]), e(x[2])] for x in m['ports']['inbound']])
dm += T(['駆動される側（外に求める）', 'アダプター', '業務領域'], [[e(x[0]) + mk(x[0]), e(x[1]), e(x[2])] for x in m['ports']['outbound']])
dm += '<h3>状態</h3>' + T(['対象', '状態'], [[e(k), ' → '.join(map(e, v))] for k, v in m['states'].items()])
dm += f'<h3>不変条件（{len(m["invariants"])}）</h3>' + T(['id', '対象', '中身'], [[x[0] + mk(x[0]), e(x[1]), e(x[2])] for x in m['invariants']])
dm += '<h3>境界</h3>' + T(['もの', '持ち主', '変えてよい者'], [[e(x[0]) + mk(x[0]), e(x[1]), e(x[2])] for x in m['boundary']])
dm += '<h3>ボードにだけ在る直し（ACDR でボードへ適用する）</h3>' + T(['どこ', '変更前', '変更後'], [[e(b['where']), e(b['before']), e(b['after'])] for b in m['board_only']])

# ---------- 2 ER 図
er = '<h2 id="er">2　ER 図</h2><p class="meta">確定するファイル（共有する）と、手元の DB（確定しない）の関係。目的と確定の単位の参照は外</p><div class="fig">' + open(os.path.join(H, 'er.svg')).read() + '</div>'

# ---------- 3 DB スキーマ
ddl = open(os.path.join(C, 'adapters', 'sqlite.sql')).read()
def cols(table):
    body = re.search(rf'CREATE TABLE {table} \((.*?)\n\);', ddl, re.S).group(1)
    out, rest = [], []
    for ln in body.split('\n'):
        s = ln.strip().rstrip(',')
        m2 = re.match(r'^([a-z_]+)\s+(TEXT.*?)\s*(?:--\s*(.*))?$', s)
        if m2 and m2.group(1) not in ('PRIMARY', 'FOREIGN', 'CHECK'): out.append([f'<code>{m2.group(1)}</code>', e(m2.group(2).strip().rstrip(',')), e(m2.group(3) or '')])
        elif s and not s.startswith('--'): rest.append(e(s))
    return T(['列', '型と制約', '意味'], out) + ('<pre class="code">' + '\n'.join(rest) + '</pre>' if rest else '')
idx = [[f'<code>{a}</code>', e(b), e(c)] for a, b, c in re.findall(r'CREATE UNIQUE INDEX (\w+) ON (\w+ \([^)]*\)+)\s*WHERE ([^;]+);', ddl)]
trg = [[f'<code>{a}</code>', e(b.strip()), e(c)] for a, b, c in re.findall(r"CREATE TRIGGER (\w+) (BEFORE [^\n]+(?:\n  WHEN[^\n]+(?:\n    OR[^\n]+)*)?)\s*BEGIN.*?RAISE\(ABORT, '([^']+)'", ddl, re.S)]
db = ('<h2 id="db">3　DB スキーマ（手元の SQLite のアダプター）</h2><p class="meta">正本 adapters/sqlite.sql。行の形は Abstract の一員 session から書く。不変条件は接続の設定に依らずに断る</p>'
      + '<h3>sessions</h3>' + cols('sessions') + '<h3>session_changes</h3>' + cols('session_changes')
      + '<h3>一意の索引（I46）</h3>' + T(['名前', '対象', '条件（開いている行の範囲）'], idx)
      + '<h3>トリガー</h3>' + T(['名前', 'いつ', '断る理由'], trg))

# ---------- 4 ・ 5 スキーマ
def cons(s):
    if not isinstance(s, dict): return e(s)
    out = []
    for k in ('type', 'const', 'enum', 'pattern', 'minLength', 'maxLength', 'minItems', 'readOnly', 'format', '$ref', 'x-relationship', 'x-decided-by', 'x-derived-from', 'default'):
        if k in s: out.append(f'{k}: {json.dumps(s[k], ensure_ascii=False)}')
    if 'items' in s and isinstance(s['items'], dict): out.append('items: ' + (json.dumps(s['items'].get('$ref') or s['items'].get('type') or '…', ensure_ascii=False)))
    if 'properties' in s and s.get('type') == 'object': out.append('欄: ' + ' ・ '.join(s['properties']))
    return '<code>' + e('\n'.join(out)) + '</code>'
def prompt(p):
    if not p: return '', ''
    r = p.get('read', {}); w = p.get('write', [])
    rd = '<br>'.join(f'<b>{k}</b> {e(v)}' for k, v in r.items())
    wr = '<br>'.join(f'<span class="rid">{e(x["id"])}</span>（{x["by"]}{" ・ " + x["agent"] if x.get("agent") else ""}）{e(x["rule"])}' for x in w)
    return rd, wr
def fields(props, required=()):
    rows = []
    for k, v in props.items():
        if k.startswith('x-') and isinstance(v, dict) and '$ref' in v and len(v) == 1:
            rows.append([f'<code>{e(k)}</code>', cons(v), '（Abstract の x-）', '', '']); continue
        rd, wr = prompt(v.get('x-prompt') if isinstance(v, dict) else None)
        rows.append([f'<code>{e(k)}</code>' + (' <span class="req">必須</span>' if k in required else ''), cons(v), e(v.get('description', '')) if isinstance(v, dict) else '', rd, wr])
    return T(['欄', '型と制約', '説明', '読み方（read）', '書くときの決まり（write）'], rows, 'wide')
def conds(s):
    out = []
    for c in s.get('allOf', []) + ([{'if': s['if'], 'then': s['then']}] if 'if' in s else []):
        if 'if' in c: out.append([f'<code>{e(json.dumps(c["if"], ensure_ascii=False))}</code>', f'<code>{e(json.dumps(c["then"], ensure_ascii=False))}</code>'])
    return T(['もし（if）', 'なら（then）'], out) if out else ''
def head(s):
    keys = [k for k in ('$schema', '$id', 'title', 'description', 'x-instance-path', 'x-instance-label') if k in s]
    return T(['キー', '値'], [[f'<code>{k}</code>', e(s[k])] for k in keys])
def rawjson(s, name): return f'<details><summary>{e(name)} の JSON 全文</summary><pre class="code">{e(json.dumps(s, ensure_ascii=False, indent=2))}</pre></details>'

ab = json.load(open(os.path.join(C, 'schemas', 'abstract.json')))
x = '<h2 id="abs">4　Abstract スキーマ（本体）の全容</h2><p class="meta">正本 schemas/abstract.json（$schema は自分の $id。自分自身で検査して合格）。Concrete はこれに従う</p>' + head(ab)
x += '<h3>9つの x-（x-extensions）</h3>' + T(['x-', '説明', '値の形', '読み方', '書くときの決まり'], [[f'<code>{k}</code>', e(v['description']), cons(v['schema']), *prompt(v['prompt'])] for k, v in ab['x-extensions'].items()], 'wide')
x += '<h3>本体の任せる先（x-delegates）</h3>' + T(['名前', '立場', '目的'], [[f'<code>{k}</code>', e(v['role']), v['run']['kind']] for k, v in ab['x-delegates'].items()])
rd, wr = prompt(ab['x-prompt'])
x += '<h3>Concrete を書くときの決まり（根の x-prompt）</h3>' + T(['読み方', '書くときの決まり'], [[rd, wr]], 'wide')
x += '<h3>Concrete の根に書けるもの（properties）</h3>' + fields(ab['properties'], ab.get('required', [])) + '<h3>インスタンスを作るときの条件</h3>' + conds(ab)
x += '<h3>部品（$defs）</h3>'
for k, v in ab['$defs'].items():
    x += f'<h4><code>$defs/{e(k)}</code> <span class="meta">{e(v.get("description", ""))}</span></h4>'
    if isinstance(v, dict) and isinstance(v.get('properties'), dict) and k not in ('fields',): x += fields(v['properties'], v.get('required', [])) + conds(v)
    else: x += f'<pre class="code">{e(json.dumps(v, ensure_ascii=False, indent=1))}</pre>'
x += rawjson(ab, 'abstract.json')
mem = '<h2 id="mem">5　Abstract の一員</h2><p class="meta">本体に従うスキーマ。インスタンスのファイル（変更の記録 ・ 判定の記録 ・ 設定）と、手元の DB の行（session）と、出力の形（change-view）</p>'
for n in ['change', 'judgment', 'settings', 'session', 'change-view', 'annotations']:
    s = json.load(open(os.path.join(C, 'schemas', f'{n}.schema.json')))
    mem += f'<h3 id="m-{n}">{e(s["title"])}（{n}）</h3>' + head(s)
    if 'properties' in s: mem += fields(s['properties'], s.get('required', [])) + conds(s)
    for k, v in s.get('$defs', {}).items():
        mem += f'<h4><code>$defs/{e(k)}</code> <span class="meta">{e(v.get("description", ""))}</span></h4>'
        if isinstance(v.get('properties'), dict): mem += fields(v['properties'], v.get('required', []))
        if v.get('oneOf'): mem += T(['種類（oneOf）', '条件'], [[e(o.get('title', '')), f'<code>{e(json.dumps({k2: v2 for k2, v2 in o.items() if k2 != "title"}, ensure_ascii=False))}</code>'] for o in v['oneOf']])
        if v.get('x-prompt'): rd, wr = prompt(v['x-prompt']); mem += T(['読み方', '書くときの決まり'], [[rd, wr]], 'wide')
    mem += rawjson(s, f'{n}.schema.json')

cc = json.load(open(os.path.join(C, 'examples', 'acdr.concrete.json'))); ins = json.load(open(os.path.join(C, 'examples', 'acdr.instances.json')))
co = '<h2 id="con">6　Concrete スキーマ（ACDR）の全容</h2><p class="meta">正本 examples/acdr.concrete.json（本体に従う例）。ACDR は基盤の変更の記録を広げて見やすくする文脈</p>' + head(cc)
co += '<h3>任せる先（x-delegates）</h3>' + T(['名前', '立場', '目的', '起動', '答えの形'], [[f'<code>{k}</code>', e(v['role']), v['run']['kind'], e(' '.join(v['run'].get('argv', [])) or 'AI ツール'), cons(v.get('answer', {}))] for k, v in cc['x-delegates'].items()])
co += '<h3>欄（properties）</h3>' + fields(cc['properties'], cc.get('required', [])) + '<h3>状態ごとの条件</h3>' + conds(cc)
if cc.get('$defs'): co += '<h3>部品（$defs）</h3>' + ''.join(f'<h4><code>$defs/{e(k)}</code></h4>' + (fields(v['properties'], v.get('required', [])) if isinstance(v.get('properties'), dict) else f'<pre class="code">{e(json.dumps(v, ensure_ascii=False, indent=1))}</pre>') for k, v in cc['$defs'].items())
co += '<h3>インスタンスの例（accepted の姿）</h3><pre class="code">' + e(json.dumps(ins['I3'], ensure_ascii=False, indent=2)) + '</pre>' + rawjson(cc, 'acdr.concrete.json')

css = open(os.path.join(H, 'view.css')).read()
nav = '<nav>' + ''.join(f'<a href="#{i}">{t}</a>' for i, t in [('dm', 'ドメインモデル'), ('er', 'ER 図'), ('db', 'DB'), ('abs', 'Abstract'), ('mem', '一員'), ('con', 'Concrete')]) + '</nav>'
page = f'<title>基盤の核の全容</title><style>{css}</style><main><h1>基盤の核の全容</h1><p class="meta">measured/core/ の正本から view/build_view.py が作る。手で直さない</p>{nav}{dm}{er}{db}{x}{mem}{co}</main>'
open(os.path.join(H, 'core-view.html'), 'w').write(page)
print('ok', len(page))
