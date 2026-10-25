"""アーキテクチャのページ（arch.html）の5節：ディレクトリ構成を作る。木の正本は arch/layout/tree.py で、ここは表示だけを持つ。"""
import json, os
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(H)
m = json.load(open(os.path.join(C, 'model', 'model.json')))
L = []
def line(path, d='', cls=''): L.append((path, d, cls))
import sys
sys.path.insert(0, os.path.join(C, 'arch', 'layout'))
from tree import TREE   # 木の正本（arch/layout/tree.py）。ここに木を書かない
CLS = {'agg': 'ag', 'ent': 'ag', 'fac': 'fc', 'svc': 'sv', 'uc': 'sv', 'port': 'pt'}
got_a, got_s, svc_rows_l = [], [], []
def walk(node, depth, trail):
    n, d, kids, o, k = node
    is_crate = kids is not None and any(c[0] == 'Cargo.toml' for c in kids) and depth > 0
    cls = 'cr' if is_crate else CLS.get(k, '')
    line('  ' * depth + n, d, cls)
    if k == 'agg': got_a.append(d.split('。')[0])
    if k == 'svc':
        en = d.split('（')[0]; kana = d.split('（')[1].rstrip('）') if '（' in d else ''
        got_s.append(en); svc_rows_l.append((en, kana, '/'.join(trail + [n])))
    for c in kids or []: walk(c, depth + 1, trail + [n.rstrip('/')])
walk(TREE, 0, [])
aggs = [a['name'] for a in m['aggregates']]
assert sorted(got_a) == sorted(aggs), (set(aggs) ^ set(got_a))
assert sorted(got_s) == sorted(d[0] for d in m['domain_services']), (set(got_s) ^ set(d[0] for d in m['domain_services']))
rows = ''.join(f'<span class="{c}">{p}{"  <i>" + d + "</i>" if d else ""}</span>' for p, d, c in L)
svc_rows = ''.join(f'<tr><td>{n}</td><td>{k}</td><td>{p_.replace("semantic-agent-schema/", "")}</td></tr>' for n, k, p_ in svc_rows_l)
sec = f'''<h2>5　ディレクトリ構成（crate と、その中のモジュール）</h2>
<p class="meta">コードは crates/ にまとめ、その下のフォルダで層を見せる（利用者が案 C を選んだ）。domain の各 crate の中は values/（値オブジェクト）・ aggregates/（集約）・ services/（ドメインサービス）に分け、エンティティかファクトリを持つ集約は aggregates/&lt;集約&gt;/ に entities/ ・ factories/ を置く（欄はその集約のフォルダの中にだけ見せる）。ポートは application の ports/ に置く（arch/ports の承認）。集約は model.json の9つと、作るときに突き合わせて全部が1回ずつ現れることを確かめている。ドメインサービスは決めた4つで、ほかの処理は集約の操作とファクトリ3つが持つ</p>
<p class="meta lg"><span class="cr">crate</span> <span class="ag">集約</span> <span class="fc">ファクトリ</span> <span class="sv">ドメインサービス ・ 使う場面</span> <span class="pt">ポート</span></p>
<pre class="v">{rows}</pre>
<h3>ドメインサービス4つの置き場所</h3>
<div class="tw"><table><tr><th>ドメインサービス</th><th>カタカナ</th><th>ファイル</th></tr>{svc_rows}</table></div>
<div class="tw"><table><tr><th>決まり</th><th>中身</th></tr>
<tr><td>根</td><td>コード（crates/）と、契約 ・ 場面 ・ 規則（schemas/ ・ test-suite/ ・ CODING.md ・ architecture.json）を分ける</td></tr>
<tr><td>crate ＝ フォルダ</td><td>crates/domain/ ・ crates/outbound/ ・ crates/inbound/ の下のフォルダと crates/application/ が、crate 1つずつ。フォルダの並びが依存の向きの層。inbound ・ outbound の名前は Q5 #9</td></tr>
<tr><td>domain の crate の中</td><td>values/（値オブジェクト1つに1ファイル）・ aggregates/（集約1つに1ファイル。集約の操作もここ）・ services/（ドメインサービス1つに1ファイル）。エンティティかファクトリを持つ集約は aggregates/&lt;集約&gt;/ に entities/ ・ factories/ を置き、集約とエンティティの欄を pub(in crate::aggregates::&lt;集約&gt;) にする。2つ以上の集約が持つエンティティだけ aggregates/entities/。ポートを持たない。値オブジェクトは、その語を持つ業務領域の crate に置く</td></tr>
<tr><td>application の中</td><td>use_cases/（入口のコマンドの組ごとに1ファイル。読む → 集約の操作 ・ ドメインサービス → 書く）・ ports/（駆動される側のポート1つに1ファイル）</td></tr>
<tr><td>入口</td><td>inbound/cli の crate は lib と実行ファイル2つ（src/main.rs の CLI と src/bin/mcp.rs の MCP）。起動時の結合は compose.rs の1か所</td></tr>\n<tr><td>識別子</td><td>crate ・ ファイル ・ 型の名前は英語（ASCII）。業務領域の語との対応は、型の doc コメントの1行目（日本語の名前）に持つ</td></tr>
</table></div>
'''
p = os.path.join(C, 'arch.html'); s = open(p).read()
i = s.index('<h2>5　ディレクトリ構成'); j = s.index('<h2>6　決定的にする検査</h2>')
s = s[:i] + sec + s[j:]
if 'pre.v .ag' not in s:
    s = s.replace('</style>', 'pre.v .ag{background:var(--hot-bg);color:var(--hot)}pre.v .sv{background:var(--code);color:var(--accent)}pre.v .pt{color:var(--bad)}pre.v .fc{background:var(--accent-bg);color:var(--ink)}.lg .fc{background:var(--accent-bg);color:var(--ink)}.lg span{border-radius:6px;padding:1px 8px;margin-right:6px;font-size:12px}.lg .cr{background:var(--accent-bg);color:var(--accent)}.lg .ag{background:var(--hot-bg);color:var(--hot)}.lg .sv{background:var(--code);color:var(--accent)}.lg .pt{color:var(--bad)}\n</style>', 1)
open(p, 'w').write(s)
print('ok', len(got_a), len(got_s))
