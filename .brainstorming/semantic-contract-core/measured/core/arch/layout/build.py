"""確定するディレクトリ構成とアーキテクチャ、CODING.md の頁（arch/layout/index.html）。木は tree.py が正本。"""
import os, re, html, sys
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
sys.path.insert(0, H)
from tree import TREE, REVISIONS
src = open(os.path.join(C, 'arch.html')).read()
CSS = '''main>*{min-width:0;max-width:100%}
.cm h2{font-size:17px;margin:14px 0 4px}.cm{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:14px;display:flex;flex-direction:column;gap:8px}.cm p,.cm td{overflow-wrap:anywhere}
.ftree,.ftree ul{list-style:none;margin:0;padding:0}.ftree ul{margin-left:14px;padding-left:12px;border-left:1px solid var(--line)}
.ftree{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:10px 12px;overflow-x:auto}
.ftree summary,.ftree .row,.ftree details,.ftree li{border:0 !important;padding:0;margin:0;background:none;box-shadow:none;border-radius:0}
.ftree code{color:var(--ink);background:none;padding:0;border:0}
.ftree .row{display:grid;grid-template-columns:max(230px,calc(400px - var(--d,0)*25px)) minmax(0,1fr);gap:4px 16px;align-items:baseline;padding:3px 4px !important;list-style:none}
.ftree summary.row{cursor:pointer}.ftree summary::-webkit-details-marker{display:none}.ftree .row:hover{background:var(--bg)}
.ftree .nm{display:flex;flex-wrap:wrap;gap:4px 6px;align-items:baseline;min-width:0}.ftree .nm code{font-size:13px;white-space:nowrap}.ftree li.d>details>summary .nm code{font-weight:700}
.ftree .ic{width:1em;color:var(--sub);flex:none}.ftree details[open]>summary .ic{transform:rotate(90deg)}
.ftree .ds{font-size:12.5px;color:var(--sub);min-width:0}
.kb{font-size:10.5px;font-weight:700;border-radius:999px;padding:0 6px;border:1px solid var(--accent);color:var(--accent);white-space:nowrap}
.mk{font-size:10.5px;font-weight:700;border-radius:4px;padding:0 5px;margin-right:6px;white-space:nowrap}.mk-ok{color:var(--sub);border:1px solid var(--line)}.mk-rev{color:var(--hot);background:var(--hot-bg)}.mk-new{color:var(--accent);border:1px solid var(--accent)}
@media (max-width:640px){.ftree .nm code{white-space:normal;overflow-wrap:anywhere}.ftree .row{grid-template-columns:1fr}.ftree .ds{padding-left:1.4em}.ftree ul{margin-left:6px;padding-left:8px}}
.tb{display:flex;gap:8px;align-items:center;flex-wrap:wrap}.tb button{font:inherit;font-size:12.5px;padding:3px 12px;border-radius:999px;border:1px solid var(--line);background:var(--card);color:var(--ink);cursor:pointer}
.sm{margin:0;font-size:12px;color:var(--sub)}
pre.out{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:8px 10px;font-size:12px;overflow-x:auto;margin:0}'''
head = src[:src.index('</style>')] + CSS + '\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>ディレクトリ構成と CODING.md</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
def inline(t):
    t = html.escape(t, quote=False); t = re.sub(r'`([^`]+)`', r'<code>\1</code>', t)
    return re.sub(r'\*\*([^*]+)\*\*', r'<b>\1</b>', t)
def md(text):
    out, rows = [], []
    def flush():
        if rows: out.append(table([inline(c) for c in rows[0]], [[inline(c) for c in r] for r in rows[2:]])); rows.clear()
    for line in text.split('\n'):
        if line.startswith('|'): rows.append([c.strip() for c in line.strip().strip('|').split('|')]); continue
        flush()
        if line.startswith('# ') or line.startswith('## '): out.append(f'<h2>{inline(line.lstrip("#").strip())}</h2>')
        elif line.strip() in ('', '---'): continue
        else: out.append(f'<p>{inline(line)}</p>')
    flush(); return '\n'.join(out)
KB = {'agg':'集約','ent':'エンティティ','fac':'ファクトリ','svc':'ドメインサービス','val':'値','port':'ポート','uc':'使う場面','ad':'アダプター'}
MK = {'ok':('承認済み','mk-ok'),'rev':('見直し','mk-rev'),'new':('新しく決める','mk-new')}
def render(node, depth=0):
    n, d, kids, o, k = node
    badge = f'<span class="kb">{KB[k]}</span>' if k else ''
    label = f'<span class="nm"><span class="ic">{"▸" if kids is not None else "·"}</span><code>{html.escape(n)}</code>{badge}</span><span class="ds"><span class="mk {MK[o][1]}">{MK[o][0]}</span>{d}</span>'
    if kids is None: return f'<li class="f"><div class="row" style="--d:{depth}">{label}</div></li>'
    return f'<li class="d"><details{" open" if depth < 3 else ""}><summary class="row" style="--d:{depth}">{label}</summary><ul>' + ''.join(render(c, depth+1) for c in kids) + '</ul></details></li>'
def count(node, acc):
    n, d, kids, o, k = node; acc[o] = acc.get(o, 0) + 1
    for c in kids or []: count(c, acc)
    return acc
cnt = count(TREE, {})
TREE_HTML = ('<div class="tb"><button type="button" id="openall">すべて開く</button><button type="button" id="closeall">すべて閉じる</button>'
  f'<span class="sm">▸ フォルダ（押すと開閉）。行の印：<span class="mk mk-ok">承認済み</span>{cnt.get("ok",0)} ／ <span class="mk mk-rev">見直し</span>{cnt.get("rev",0)} ／ <span class="mk mk-new">新しく決める</span>{cnt.get("new",0)}</span></div>'
  '<ul class="ftree">' + render(TREE) + '</ul>'
  '<script>document.getElementById("openall").onclick=()=>document.querySelectorAll(".ftree details").forEach(d=>d.open=true);document.getElementById("closeall").onclick=()=>document.querySelectorAll(".ftree details").forEach((d,i)=>d.open=i===0);</script>')
coding = md(open(os.path.join(H, 'CODING.md')).read())
MEASURE = [
 ['crate の直下の factories/ から、欄を private のまま集約を作る（承認済みの形）', 'コンパイルが通らない：<code>error[E0451]: field … is private</code>'],
 ['同じ形で欄を pub(crate) にしたあと、集約の外から欄を書き換える', 'コンパイルが通る（書き換えを止められない）'],
 ['values/ ・ aggregates/（集約のフォルダ aggregates/&lt;集約&gt;/ に entities/ ・ factories/）・ services/ に分け、欄を pub(in crate::aggregates::&lt;集約&gt;) にする', 'コンパイルが通る（ファクトリは欄で集約を作れる）'],
 ['同じ形で、別の集約（glossary）から ConcreteSchema の欄を書き換える', 'コンパイルが通らない：<code>error[E0616]: field `inner` of struct `ConcreteSchema` is private</code>'],
 ['同じ形で、services/ から集約の欄を書き換える', 'コンパイルが通らない：E0616'],
 ['同じ形で、別の集約からエンティティ rule を作る', 'コンパイルが通らない：<code>error[E0624]: associated function `new` is private</code>'],
 ['（前の案）aggregates/entities/ ・ aggregates/factories/ をまとめて置く形で、別の集約から欄を書き換える', 'コンパイルが通ってしまった。そのため集約ごとのフォルダにした（利用者「①でお願いします」）'],
 ['この木から作った雛形（16 crate）で <code>cargo check --workspace --all-targets</code>（build.warnings = "deny"）', '通った'],
 ['16 crate を1つずつ <code>cargo check -p</code>', '全部通った'],
 ['<code>cargo build</code>（cli の lib ＋ 実行ファイル2つ）', 'semantic-agent-schema と semantic-agent-schema-mcp ができた'],
 ['<code>architecture-lint check</code>（4節の辺の表 ＋ session → declaration）', '違反なし。ただし雛形の依存と規則は同じ表から作ったので、一致するのは当然。検出できることの裏付けは次の行'],
 ['わざと session → application の辺を足す', '<code>not-in-allowed semantic-agent-schema-session -&gt; semantic-agent-schema-application</code> で止まった'],
 ['雛形の中の <code>mod.rs</code>', '0件'],
]
NOT_MEASURED = ['ポートの trait を outbound が実装すること、application が domain を使うこと（雛形は空の型だけ）', '外の crate（jsonschema ・ rusqlite ・ syn）', 'clippy.toml の disallowed と allow-*-in-tests、rustfmt', '追跡の検査']
QUOTES = [
 ['private の項目は、そのモジュールと子孫から触れられる', 'If an item is private, it may be accessed by the current module and its descendants.', 'The Rust Reference, Visibility and privacy:47（commit 3989fe91）'],
 ['子のモジュールは親の項目に触れられる', 'This module could access any items of the parent module through the second case', '同:55'],
 ['pub(in path) は、指定した path の中にだけ見せる', 'pub(in path) makes an item visible within the provided path.', 'The Rust Reference, Visibility and privacy:123'],
 ['集約を作る式は、全部の欄が見えている必要がある', 'all of the fields of the struct must be visible, even those not explicitly named.', 'The Rust Reference, Struct expressions:105'],
 ['欄は既定で private', 'struct fields follow the general rule of everything being private by default unless annotated with pub.', 'The Rust Programming Language 7.3:283'],
 ['foo.rs の形が勧められている', 'It is encouraged to use the new naming convention as it is more consistent, and avoids having many files named mod.rs within a project.', 'The Rust Reference, Modules:71'],
 ['foo.rs と foo/mod.rs を両方は置けない', 'It is not allowed to have both util.rs and util/mod.rs.', '同:68'],
]
REV_ROWS = [[a, b, c, d] for a, b, c, d in REVISIONS]
body = f'''<main>
<h1>ディレクトリ構成と CODING.md</h1>
<p class="ans">承認済みの構成（ACDR 0201 の arch.html とボード）を土台にした。domain の crate の中は values/ ・ aggregates/ ・ services/ のフォルダで分ける。エンティティかファクトリを持つ集約は aggregates/&lt;集約&gt;/ に entities/ ・ factories/ を置き、欄をその集約のフォルダの中にだけ見せる（別の集約からも書き換えられないことをコンパイラで確かめた）。この木から作った雛形で cargo check と architecture-lint が通ることを実測した</p>
<h2>1　ディレクトリ構成</h2>
{TREE_HTML}
<h2>2　承認済みから変えたもの</h2>
{table(['もの', '承認済みの形（変更前）', '確定する形（変更後）', 'なぜ'], REV_ROWS)}
<p class="meta">承認を受けたら、arch.html の5節 ・ 図 ・ 4節の辺の表を、この形に直す（ACDR）</p>
<h2>3　実測</h2>
{table(['確かめたこと', '結果'], MEASURE)}
<p class="meta">雛形は作業用のフォルダに tree.py から機械的に作った（arch/layout/skeleton.py）。中身は空の型だけ。確かめていないもの：{' ／ '.join(NOT_MEASURED)}</p>
<h2>4　原典（Rust の公式文書の最新版、原文と照合）</h2>
{table(['主張', '原文', '出どころ'], QUOTES)}
<h2>5　まだ決めていないこと（このあとの作業で決める）</h2>
{table(['こと', 'いつ決めるか'], [
 ['「新しく決める」の印の名前（rule.rs ・ delegate.rs ・ procedure ・ query ・ change-summary ・ settings の .schema.json ・ views の名前 ・ commands の分け方など）', '雛形を作るときに、この名前で作る。違和感があれば言ってほしい'],
 ['model.json の states に、状態を導く決定表の列が無い', '単体テストを書く前。ボードの model.json の変更として足す'],
 ['システムテストのコマンドの名前と引数', 'protocol（ボードの Q4）を決めるとき'],
])}
<h2>6　CODING.md（全文）</h2>
<div class="cm">{coding}</div>
<p class="meta">根拠：Rust の原典（Cargo Book ・ The Rust Programming Language ・ The Rust Reference ・ Rust API Guidelines ・ Clippy ・ rustfmt。sources/rust/ に原文）・ ddd-advisor ・ qa-advisor（arch/layout/reviews/）・ 実測（3節）</p>
</main>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
