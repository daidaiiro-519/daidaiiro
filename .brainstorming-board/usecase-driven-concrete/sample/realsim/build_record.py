# 実行の記録（sim.json ・ 記録ファイル ・ 管理しているファイルの一覧）から、1枚の頁を組む
import json,html,os,re
H=os.path.dirname(os.path.abspath(__file__)); E=html.escape
L=json.load(open(f'{H}/sim.json')); R=f'{H}/repo'
css=open(f'{H}/../tokens.css').read()+open(f'{H}/../sample4.css').read()
def pill(t,c=''): return f'<span class="pill {c}">{E(t)}</span>'
def tbl(cols,rows): return '<div class="tw"><table><thead><tr>'+''.join(f'<th>{c}</th>' for c in cols)+'</tr></thead><tbody>'+''.join('<tr>'+''.join(f'<td>{c}</td>' for c in r)+'</tr>' for r in rows)+'</tbody></table></div>'
def blk(t,b): return f'<section class="blk"><h2>{E(t)}</h2>{b}</section>'
def code(s): return f'<pre class="code">{E(s)}</pre>'
def part(ci,head):
  m=re.search(re.escape(head)+r'\n(.*?)(?=\n\$ |\nCI：)',ci,re.S); return m.group(1).strip() if m else ''
def verdict(ci,head,okpat):
  t=part(ci,head); return pill('通った','t-success') if re.search(okpat,t) else pill('落ちた','t-fail')
contract=re.search(r'CONTRACT="""(.*?)"""',open(f'{R}/tool/concrete.py').read(),re.S).group(1)
b='<header class="ph"><p class="kind">シミュレーション</p><h1>実行の記録</h1></header><p class="lead">実際のリポジトリで、Python（集約などの component テスト）と Go（ユースケースの system テスト）の2つの言語のテストを流し、道具 concrete と突き合わせた記録。</p>'
b+=blk('仕組み',tbl(['','道具（concrete）','テストの実行器（pytest ・ go test）'],[
 ['読むもの','宣言（spec/）と、記録ファイル（build/trace.jsonl）だけ','テストのソース'],
 ['読まないもの','テストのソース、テストの合否','宣言'],
 ['判定すること','欠け ・ 余り ・ 古い ・ レベル違い（仕様とテストの対応）','テストが通るか'],
 ['言語に依存するか','しない','する（その言語のもの）'],
 ['結果の置き場所','標準出力だけ。保存しない','標準出力だけ。保存しない']])+'<p class="txt">CI は、両方が通ったときだけ通る。2つをつなぐのは、テストが走ったときに1行ずつ追記する記録ファイルだけである。</p>')
b+=blk('記録の契約（concrete contract の出力）',code(contract.strip()))
py=open(f'{R}/tests/trace.py').read(); go=open(f'{R}/systemtest/trace_test.go').read()
b+=blk('補助の関数（契約どおりに、各言語の標準ライブラリだけで書いたもの）',f'<div class="cards"><div class="card"><h3>Python（tests/trace.py）</h3>{code(py)}</div><div class="card"><h3>Go（systemtest/trace_test.go）</h3>{code(go)}</div></div>')
pt=open(f'{R}/tests/test_domain.py').read(); gt=open(f'{R}/systemtest/uc1_test.go').read()
p1=pt[pt.index('def test_reserve_rejects_short'):pt.index('def test_reserve():')]
g1=gt[gt.index('func TestSlotShort'):gt.index('func TestPaymentDeclined')]
b+=blk('テストの側の書き方（テストの先頭で、確かめるテスト条件の ID とハッシュ値を記録する）',f'<div class="cards"><div class="card"><h3>Python ・ component</h3>{code(p1)}</div><div class="card"><h3>Go ・ system</h3>{code(g1)}</div></div>')
tr=open(f'{H}/trace-last.jsonl').read().splitlines()
b+=blk(f'記録ファイルの実物（最後の手で、2つの言語のテストが合わせて {len(tr)} 行を書いた。抜粋）',code('\n'.join(tr[:4]+['…']+tr[-3:]))+'<p class="txt">書いたのは pytest と go test で、道具は書いていない。1行が「このテストは、このテスト条件を、このハッシュ値の版で、このレベルで確かめた」を表す。合否は書かない。</p>')
tracked=open(f'{H}/tracked.txt').read().split()
grp={}
for f in tracked: grp.setdefault(f.split('/')[0] if '/' in f else '（根）',[]).append(f)
st=open(f'{H}/status.txt').read()
b+=blk('リポジトリに残るもの ／ 残らないもの',tbl(['場所','中身','git で管理するか'],[
 ['spec/decls/','宣言（17件）','する'],
 ['spec/confirmed.json','宣言どうしの対応を承認した時点のハッシュ値（承認者が concrete approve で書く）','する'],
 ['src/ ・ tests/ ・ systemtest/','実装と、2つの言語のテスト（テスト条件の ID とハッシュ値はテストのソースの中にある）','する'],
 ['tool/ ・ ci.sh','道具と、CI の手順','する'],
 ['build/trace.jsonl','記録ファイル（テストを流すたびに作り直す）','しない（.gitignore）'],
 ['テストの合否','実行器の標準出力','しない（どこにも残さない）']])+f'<p class="txt">最後の手のあとの <code>git status --ignored</code>：</p>'+code(st))
rows=[]
for x in L:
  ci=x['ci']
  rows.append([f'<button class="sbtn" data-go="{x["no"]}">{x["no"]}</button>',E(x['who']),f'<span class="txt">{E(x["what"])}</span>',
    verdict(ci,'$ concrete check',r'^宣言どうし：合格 \d+$'),verdict(ci,'$ pytest -q tests',r'passed') if 'failed' not in part(ci,'$ pytest -q tests') else pill('落ちた','t-fail'),
    verdict(ci,'$ (cd systemtest && go test -count=1 ./...)',r'^ok'),verdict(ci,'$ concrete match build/trace.jsonl',r'^突き合わせ：[^・]*・ 記録 \d+ 行$'),pill('通った','t-success') if x['ci_ok'] else pill('落ちた','t-fail')])
b+=blk('手の一覧（手ごとに git でコミットし、CI を流した）',tbl(['手','誰が','何をしたか','concrete check','pytest','go test','concrete match','CI'],rows))
b+='<div class="stepper">'+''.join(f'<button class="sbtn" data-go="{x["no"]}">{x["no"]}</button>' for x in L)+'</div>'
for x in L:
  d=f'<section class="simstep" data-no="{x["no"]}">'
  d+=f'<p class="lead"><b>手{x["no"]}　{E(x["who"])}</b>　{E(x["what"])}　<span class="no">commit {E(x["commit"])}</span></p>'
  d+=blk('変更したファイル（git diff --stat）',code(x['stat'].strip() or '（最初のコミット）'))
  if x['diff']:
    df=x['diff']; df=df if len(df)<6000 else df[:6000]+'\n…（以下略）'
    d+=f'<details class="fold"><summary>差分（git diff）</summary><div class="fbody">{code(df)}</div></details>'
  d+=blk('CI の出力（./ci.sh。合否と検知は標準出力にだけ出る）',code(re.sub(r'\n\.+ +\[100%\]','',x['ci']).strip()))
  d+='<div class="snav">'+(f'<button class="sbtn" data-go="{x["no"]-1}">← 前の手</button>' if x['no']>0 else '<span></span>')+(f'<button class="sbtn" data-go="{x["no"]+1}">次の手 →</button>' if x['no']<len(L)-1 else '')+'</div></section>'
  b+=d
b+=blk('実行して分かったこと',tbl(['分かったこと','どうしたか'],[
 ['手4：業務の変更で条件が変わっても、テストは Python も Go もすべて通った。実行器だけでは気づけない','道具が「古い」を2件出して CI を止めた。ハッシュ値が、仕様の版とテストを結んでいる'],
 ['手7：最低保証を守る手順を消すと、道具は「対応の欠け」、Go のテストは最低保証の破れで落ちた','2つの判定は独立に落ち、どちらか片方だけでも CI は止まる'],
 ['Go のテストは、前回と同じ入力だと結果をキャッシュから返し、テストを実行しない。記録が書かれず、道具は system の5件を「欠け」と出した','契約に「キャッシュを切って流す」を足し、CI は go test -count=1 で流す'],
 ['シミュレーションの途中で、AI 役のスクリプトが Python のテストのハッシュ値を書き換え損ねた','道具がその条件を「古い」として出し続けた。直してから流し直した']]))
page=f'''<!doctype html><html lang="ja"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>実行の記録</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;600;700&family=JetBrains+Mono:wght@400&display=swap">
<style>{css} .main{{max-width:1100px;margin:0 auto}} body{{background:var(--surface-page)}}</style></head><body><main class="main" style="padding:1.5rem 16px">{b}</main>
<script>
const go=n=>{{document.querySelectorAll('.simstep').forEach(s=>s.hidden=s.dataset.no!=n);document.querySelectorAll('.stepper .sbtn').forEach(b=>b.classList.toggle('on',b.dataset.go==n));}};
document.querySelectorAll('.sbtn').forEach(b=>b.addEventListener('click',()=>{{go(b.dataset.go);document.querySelector('.stepper').scrollIntoView({{behavior:'smooth'}})}}));go('0');
</script></body></html>'''
open(f'{H}/record.html','w').write(page); print(len(page))
