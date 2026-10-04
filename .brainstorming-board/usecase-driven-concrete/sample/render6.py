# 宣言を、共通の部品（タイル ・ カード ・ 札 ・ 表 ・ 図）とデザイントークンで描画する
import json,html,sys,os,subprocess,itertools,re
HERE=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,HERE)
from data6 import D,RETIRED
import gen6 as g
sys.path.insert(0,os.path.join(HERE,'design')); from parts import P
from desc6 import DESC
import concrete7
SPEC=concrete7.checks(json.load(open(os.path.join(HERE,'approved-record.json'),encoding='utf-8')))
CONDS=concrete7.conditions()
OUT='/home/daidaiiro/workspace/daidaiiro/.brainstorming-board/usecase-driven-concrete/sample'
DSB='/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/tool/target/release/design-svg'
for sub in ('decls','figures'):
  os.makedirs(f'{OUT}/{sub}',exist_ok=True)
  for f in os.listdir(f'{OUT}/{sub}'): os.remove(f'{OUT}/{sub}/{f}')
for k,v in D.items(): json.dump(v,open(f'{OUT}/decls/{k}.json','w'),ensure_ascii=False,indent=1)
json.dump({"conditions":CONDS,"retired":RETIRED},open(f'{OUT}/conditions.json','w'),ensure_ascii=False,indent=1)
E=html.escape; HC=[0]
THEME={"color.box-fill":"var(--paper)","color.box-stroke":"var(--line)","color.ink":"var(--ink)","color.ink-soft":"var(--muted)","color.ink-faint":"var(--muted)",
 "color.line":"var(--muted)","color.accent":"var(--accent)","color.accent-bg":"var(--accent-soft)","color.accent-fg":"var(--accent)","color.warn":"var(--warn)","color.warn-bg":"var(--warn-soft)"}
def svg(name,args,decl):
  decl=dict(decl); decl['theme']={**THEME,**decl.get('theme',{})}
  p=f'{OUT}/figures/{name}.json'; json.dump(decl,open(p,'w'),ensure_ascii=False)
  r=subprocess.run([DSB]+args+[p,'--out',f'{OUT}/figures/{name}.svg'],capture_output=True,text=True)
  if r.returncode: print(name,r.stdout.strip(),r.stderr.strip())
  return f'<figure class="fig">{open(OUT+f"/figures/{name}.svg").read()}</figure>'
figure=lambda n,d: svg(n,['figure'],d)
chart=lambda n,k,d: svg(n,['chart',k],d)
KIND={"domain":"ドメイン（プロダクト）","glossary":"用語集","other_requirements":"その他の要求","subdomain":"サブドメイン","context":"区切られた文脈","use_case":"ユースケース","aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
ORDER=["domain","glossary","other_requirements","use_case","subdomain","context","aggregate","domain_service"]
SIDE={"domain":"要求の側","glossary":"要求の側","other_requirements":"要求の側","use_case":"要求の側"}
CAT={"中核":"t-core","補完":"t-supporting","一般":"t-generic"}
def dname(i):
  d=D.get(i)
  if not d: return None
  n=d['header']['name']; return g.word(n)
def idt(i): return P('id',id=E(i))
def ref(i):
  n=dname(i); return P('ref-missing',id=E(i)) if n is None else P('ref',id=E(i),name=E(n))
def helpbtn(t):
  if t not in DESC: return ''
  HC[0]+=1; i=f'h{HC[0]}'
  return P('help',hid=i,title=E(t),text=E(DESC[t]))
def lab(t): return f'{E(t)}{helpbtn(t)}'
def pill(t,tone=''): return P('pill',tone=tone,text=E(t))
def tbl(cols,rows):
  # テスト条件の列は表に出さず、隣の欄の後ろに付ける（宣言の欄を表示したときだけ見える）
  if cols and cols[0]=='テスト条件':
    cols=cols[1:]; rows=[[r[1]+' '+r[0]]+list(r[2:]) for r in rows]
  elif cols and cols[-1]=='テスト条件':
    cols=cols[:-1]; rows=[[r[0]+' '+r[-1]]+list(r[1:-1]) for r in rows]
  lab_=[re.sub(r'<[^>]+>','',re.sub(r'<div class="pop".*?</div>','',c)).replace('?','').strip() for c in cols]
  return P('table',head=''.join(P('th',text=c) for c in cols),body=''.join(P('tr',cells=''.join(P('td',label=E(lab_[i]),text=c) for i,c in enumerate(r))) for r in rows))
def ftbl(cols,rows,groups=None,label='種類',unit='件'):
  """rows は (値, 行) の並び。表の上に絞り込みを置く。値の札は複数選べ、何も選ばなければ全部を見せる"""
  ks=list(dict.fromkeys(k for k,_ in rows))
  if len(ks)<2: return tbl(cols,[r for _,r in rows])
  n=lambda k:sum(1 for kk,_ in rows if kk==k)
  btn=lambda k:P('ftable-button',key=E(k),n=n(k))
  if groups:
    grp=[(gname,[k for k in gk if k in ks]) for gname,gk in groups]+[('その他',[k for k in ks if not any(k in gk for _,gk in groups)])]
  else: grp=[('',ks)]
  rowsh=''.join(P('ftable-group',name=P('ftable-group-name',name=E(gname)) if gname else '',buttons=''.join(btn(k) for k in gk)) for gname,gk in grp if gk)
  head_=P('ftable-head',label=E(label),unit=E(unit),count=f'{len(rows)}{E(unit)}')
  lab_=[re.sub(r'<[^>]+>','',c) for c in cols]
  body=''.join(P('ftr',key=E(k),cells=''.join(P('td',label=E(lab_[i]),text=c) for i,c in enumerate(r))) for k,r in rows)
  return P('ftable',label=E(label),head=head_,groups=rowsh,cols=''.join(P('th',text=c) for c in cols),body=body)
def block(title,body): return P('block',title=E(title),help=helpbtn(title),body=body)
import dev7
dev7.use(CONDS,block=lambda t,b: block(t,b),table=lambda c,r: tbl(c,r),pill=lambda t,tone='': pill(t,tone))
tchip,tblock=dev7.tchip,dev7.tblock
def head(d,badges='',lead=''):
  side=SIDE.get(d['kind'],'設計の側')
  return P('page-head',side=pill(side),kind=KIND[d["kind"]],badges=badges,title=E(dname(d["id"])),id=idt(d["id"]),lead=P('lead',text=E(lead)) if lead else '')
def raw(d): return P('json',label=lab("宣言の JSON"),json=E(json.dumps(d,ensure_ascii=False,indent=1)))
def p_drift():
  import collections
  b='<header class="ph"><p class="kind">テスト</p><h1>テスト条件と宣言どうしの検査</h1></header><p class="lead">宣言から道具が出すもの。テストの合否はここに無い（テストの実行器が判定し、保存しない）。</p>'
  scnt=collections.Counter(x['status'] for x in SPEC)
  b+='<h2 class="sec">宣言どうし</h2><p class="stat">'+' ・ '.join(f'{k} <b>{scnt.get(k,0)}</b>' for k in ['ずれ','欠け','確かめ直し','レビュー','合格'])+'</p>'
  CAT={'structure':'構造で検査','change':'上流の変更','review':'人のレビュー'}
  SST={'合格':'','ずれ':'t-warn','欠け':'t-warn','確かめ直し':'','レビュー':''}
  def at(i):
    if ':' in i: a,r=i.split(':',1); return ref(a)+' → '+at(r)
    if i.startswith('TERM-'): return f'「{E(g.word(i))}」'
    return ref(i.split('.')[0])+(f' <span class="no">{E(".".join(i.split(".")[1:]))}</span>' if '.' in i else '') if i and i.split('.')[0] in D else E(i)
  ck=lambda x:'上流の変更' if x['category']=='change' else x['check']
  srow=[(ck(x),[pill(x['status'],SST[x['status']]),E(CAT[x['category']]),E(x['check']),at(x['from']),'、'.join(at(t) for t in x['to'].split('・')) if x['to'] else '',E(x['text'])]) for x in sorted(SPEC,key=lambda x:(x['status']=='合格',list(CAT).index(x['category'])))]
  b+=block('検査ごとの結果',ftbl(['状態','分け方','検査','参照元','参照先','内容'],srow,[(v,list(dict.fromkeys(ck(x) for x in SPEC if CAT[x['category']]==v))) for v in CAT.values()],'検査','件'))
  b+='<h2 class="sec">宣言とテスト</h2>'
  KN={'use_case':'ユースケース','aggregate':'集約','value_object':'値オブジェクト','domain_service':'ドメインサービス','other_requirements':'その他の要求'}
  b+=block('テスト条件',ftbl(['ID','宣言','対象','確かめること','求めるレベル','ハッシュ値'],[(KN.get(c['kind'],c['kind']),[tchip(c['id']),ref(c['decl']),E(c['label']),E(c['checks']),pill(c['required_level']),f'<span class="no">{c["hash"]}</span>']) for c in CONDS],None,'宣言の種類','件'))
  b+=block('突き合わせ',tbl(['何を','どこで見られるか'],[
    ['テストは走ったときに、上の ID ・ ハッシュ値 ・ レベルを記録ファイルへ1行ずつ追記する（記録の契約）。道具は宣言と記録だけを照らし、欠け ・ 余り ・ 古い ・ レベル違いを出す','<a class="ref" href="https://claude.ai/artifact/MGX9MpSk6MtnCF8QpWqDdh" target="_blank" rel="noopener">テスト条件 ID の突き合わせ</a>'],
    ['Python と Go のテストを実際に実行し、9手のコミットごとに CI を流した記録','<a class="ref" href="https://claude.ai/artifact/HwxXEHAKFcGig7f7UamAxY" target="_blank" rel="noopener">実行の記録</a>']]))
  return b
# ── 書き方（スキーマ）。各欄の description と x-prompt.write を、そのまま並べる
SCH={os.path.basename(f)[:-12]:json.load(open(f,encoding='utf-8')) for f in __import__('glob').glob(os.path.join(HERE,'schema','*.schema.json'))}
def sch_rows(props):
  return [[f'<b>{E(v.get("description",""))}</b>',f'<code>{E(k)}</code>',f'<span class="txt">{E(v.get("x-prompt",{}).get("write",""))}</span>'] for k,v in props.items() if k not in ('$schema',)]
def p_sch(k):
  sc=SCH[k]; b=f'<header class="ph"><p class="kind">{pill("書き方")} スキーマ</p><h1>{E(sc["title"])}{idt(k+".schema.json")}</h1></header><p class="lead">{E(sc["description"])}</p>'
  if k=='common':
    return b+block('共通の形',tbl(['項目','名前','書くこと'],sch_rows(sc['$defs'])))
  if 'properties' not in sc:
    return b+block('形',f'<p class="txt">キーは「参照元→参照先」、値は承認した時点の参照先のハッシュ値（16進8桁）。置き場所：<code>{E(sc["x-generates"])}</code></p>')
  b+=block('欄と書くこと',(f'<p class="txt">置き場所：<code>{E(sc["x-generates"])}</code></p>' if sc.get('x-generates') else '')+tbl(['項目','キー','書くこと'],sch_rows(sc['properties'])))
  for n,d in sc.get('$defs',{}).items():
    if n=='sub_step': continue
    b+=f'<section class="blk"><h2>{E(d["description"])}</h2><p class="txt">{E(d["x-prompt"]["write"])}</p>{tbl(["項目","キー","書くこと"],sch_rows(d.get("properties",{})))}</section>'
  return b
import compose7, fig7
fig7.use(figure=figure,chart=chart)
PARTS={"head":head,"block":block,"table":tbl,"ftable":ftbl,"raw":raw,"ref":ref,"help":helpbtn,"tones":{"category":CAT},"fig":fig7.FIGS,"dev":{k:getattr(dev7,k) for k in ("tchip","tblock","impossible","bounds","violation_state","violation_via","reject_example","accept_rows")}}
R={k:(lambda d: compose7.page(d,PARTS)) for k in compose7.TPL}
SCH_ORDER=[k for k in ORDER+['value_object']]+['common','trace','approved','migration']
nav_sch='<div class="ng"><span class="nk">書き方（スキーマ）</span>'+''.join(f'<a href="#SCH-{k}" data-id="SCH-{k}">{E(SCH[k]["title"])}</a>' for k in SCH_ORDER)+'</div>'
nav='<div class="ng"><span class="nk">テスト</span><a href="#DRIFT" data-id="DRIFT">テスト条件と検査</a><a href="https://claude.ai/artifact/MGX9MpSk6MtnCF8QpWqDdh" target="_blank" rel="noopener">突き合わせ ↗</a><a href="https://claude.ai/artifact/HwxXEHAKFcGig7f7UamAxY" target="_blank" rel="noopener">実行の記録 ↗</a></div>'+''.join(f'<div class="ng"><span class="nk">{KIND[k]}</span>'+''.join(f'<a href="#{i}" data-id="{i}">{E(dname(i))}</a>' for i,v in D.items() if v['kind']==k)+'</div>' for k in ORDER)+nav_sch
pages=''.join(f'<article class="page" id="{i}">{R[v["kind"]](v)}</article>' for i,v in D.items())+f'<article class="page" id="DRIFT">{p_drift()}</article>'+''.join(f'<article class="page" id="SCH-{k}">{p_sch(k)}</article>' for k in SCH_ORDER)
css=open(os.path.join(HERE,'tokens.css')).read()+open(os.path.join(HERE,'design','tokens.css')).read()+open(os.path.join(HERE,'sample4.css')).read()
page=f'''<title>モバイルオーダーの宣言</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;600;700&family=JetBrains+Mono:wght@400&display=swap">
<style>{css}</style>
<div class="shell"><aside class="side"><p class="brand">{E(D["DOM-1"]["header"]["name"])}</p><details class="navd" open><summary>目次を開く ・ 閉じる</summary><nav class="nav">{nav}</nav></details></aside>
<main class="main">{pages}</main></div>
<script>
const show=()=>{{const id=(location.hash||'#DOM-1').slice(1);const hit=[...document.querySelectorAll('.page')].some(p=>p.id===id);const cur=hit?id:'DOM-1';document.querySelectorAll('.page').forEach(p=>p.hidden=p.id!==cur);document.querySelectorAll('.nav a').forEach(a=>a.classList.toggle('on',a.dataset.id===cur));window.scrollTo(0,0)}};
addEventListener('hashchange',show);show();
const keep=()=>{{const a=document.querySelector('.nav a.on');if(a&&innerWidth>760)a.scrollIntoView({{block:'nearest'}})}};addEventListener('hashchange',keep);keep();
document.querySelectorAll('.fwrap').forEach(w=>{{const g=w.querySelector('.filt'),c=w.querySelector('.fc'),x=w.querySelector('.fclear'),rows=[...w.querySelectorAll('tbody tr')];const apply=()=>{{const on=[...g.querySelectorAll('button[data-f][aria-pressed="true"]')].map(b=>b.dataset.f);let n=0;rows.forEach(r=>{{const v=!on.length||on.includes(r.dataset.k);r.hidden=!v;if(v)n++}});const u=c.dataset.unit;c.textContent=on.length?`${{rows.length}}${{u}}のうち ${{n}}${{u}}`:`${{rows.length}}${{u}}`;x.hidden=!on.length}};g.addEventListener('click',e=>{{const b=e.target.closest('button[data-f]');if(!b)return;b.setAttribute('aria-pressed',b.getAttribute('aria-pressed')!=='true');apply()}});x.addEventListener('click',()=>{{g.querySelectorAll('button[data-f]').forEach(b=>b.setAttribute('aria-pressed','false'));apply()}})}});
document.querySelectorAll('.tg').forEach(b=>b.addEventListener('click',()=>{{const o=b.getAttribute('aria-expanded')!=='true';b.setAttribute('aria-expanded',o);document.querySelectorAll('tr[data-grp="'+b.getAttribute('aria-controls')+'"]').forEach(r=>r.hidden=!o)}}));
if(innerWidth<=760){{const n=document.querySelector('.navd');if(n)n.open=false;document.querySelectorAll('.nav a').forEach(a=>a.addEventListener('click',()=>{{n.open=false}}))}}
const go=n=>{{document.querySelectorAll('.simstep').forEach(s=>s.hidden=s.dataset.no!=n);document.querySelectorAll('.stepper .sbtn').forEach(b=>b.classList.toggle('on',b.dataset.go==n));const st=document.querySelector('.stepper');if(st&&window.scrollY>st.offsetTop)st.scrollIntoView();}};
document.querySelectorAll('.sbtn').forEach(b=>b.addEventListener('click',()=>{{go(b.dataset.go);document.querySelector('.stepper').scrollIntoView({{behavior:'smooth'}})}}));go('0');
</script>'''
open(f'{OUT}/viewer.html','w').write(page); print(len(page))
