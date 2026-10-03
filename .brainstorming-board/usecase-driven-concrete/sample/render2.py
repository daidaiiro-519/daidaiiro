import json,html,sys,os,subprocess,tempfile
HERE=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,HERE)
from sample_data import D
OUT='/home/daidaiiro/workspace/daidaiiro/.brainstorming-board/usecase-driven-concrete/sample'
DS_BIN='/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/bin/design-svg'
os.makedirs(OUT+'/figures',exist_ok=True)
for k,v in D.items(): json.dump(v,open(f'{OUT}/decls/{k}.json','w'),ensure_ascii=False,indent=1)
E=html.escape
THEME={"color.box-fill":"var(--card)","color.box-stroke":"var(--line-strong)","color.ink":"var(--ink)","color.ink-soft":"var(--muted)","color.ink-faint":"var(--muted)",
 "color.line":"var(--edge)","color.accent":"var(--accent)","color.accent-bg":"var(--accent-soft)","color.accent-fg":"var(--accent)","color.warn":"var(--warn)","color.warn-bg":"var(--warn-soft)",
 "role.失敗.color.box-stroke":"color.warn","role.失敗.color.text":"color.warn"}
def fig(name,decl):
  decl=dict(decl); decl['theme']=THEME
  p=f'{OUT}/figures/{name}.json'; json.dump(decl,open(p,'w'),ensure_ascii=False)
  r=subprocess.run([DS_BIN,'figure',p,'--out',f'{OUT}/figures/{name}.svg'],capture_output=True,text=True)
  if r.returncode: print(name,r.stdout,r.stderr)
  return f'<div class="fig">{open(OUT+f"/figures/{name}.svg").read()}</div>'
KIND={"domain":"ドメイン","subdomain":"サブドメイン","context":"区切られた文脈","use_case":"ユースケース","aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
ORDER=["domain","subdomain","context","use_case","aggregate","value_object","domain_service"]
def term(ctx,t):
  for x in D.get(ctx,{}).get('glossary',[]):
    if x['id']==t: return x['word']
def dname(i):
  d=D.get(i)
  if not d: return None
  return term(d.get('context',''),d['name']) or d['name']
def idt(i): return f'<span class="id">{E(i)}</span>'
def ref(i):
  n=dname(i)
  if n is None: return f'<span class="missing">{E(i)}（まだ無い）</span>'
  return f'<a class="ref" href="#{E(i)}">{E(n)}</a>'
def table(cols,rows,cls=''):
  return f'<div class="tw"><table class="{cls}"><thead><tr>'+''.join(f'<th>{c}</th>' for c in cols)+'</tr></thead><tbody>'+''.join('<tr>'+''.join(f'<td>{c}</td>' for c in r)+'</tr>' for r in rows)+'</tbody></table></div>'
def sec(title,body): return f'<section class="blk"><h2>{E(title)}</h2>{body}</section>'
def fold(label,n,body): return f'<details class="fold"><summary>{E(label)}<span class="count">{n}</span></summary><div class="fbody">{body}</div></details>'
def facts(pairs): return '<dl class="facts">'+''.join(f'<div><dt>{E(k)}</dt><dd>{v}</dd></div>' for k,v in pairs)+'</dl>'
def head(d,sub=''):
  return f'<header class="ph"><span class="kind k-{d["kind"]}">{KIND[d["kind"]]}</span><h1>{E(dname(d["id"]))}{idt(d["id"])}</h1>{sub}</header>'
OPS={"eq":"は{v}","ne":"は{v}ではない","ge":"は{v}以上","le":"は{v}以下","not_empty":"は空でない"}
def cond(d,c,names):
  def val(v):
    if isinstance(v,str) and v.startswith('TERM-'): return term(d.get('context',d['id']),v) or v
    return names.get(v,str(v)) if isinstance(v,str) else str(v)
  def one(c,cl=False):
    t=names.get(c['target'],c['target'])+('の件数' if c.get('agg')=='count' else '')
    v=c.get('value',''); op=OPS[c['op']]
    if isinstance(v,str) and v in names and c['op']=='eq': op='は{v}と同じ'
    if cl: op='が'+op[1:]
    return t+op.format(v=val(v))
  s=one(c)
  return (one(c['if'],True)+'なら、'+s) if c.get('if') else s
# ── ドメイン
def p_domain(d):
  nodes=[{"id":"p","label":"課題"}]
  n=len(d['values']); rows=[]
  for i in range(1,n+1): rows+= [f"r{i}"]+([f"m{i}"] if i<n else [])
  at={"p":["a",f"m{(n+1)//2}" if n>1 else "r1"]}; edges=[]
  for i,v in enumerate(d['values'],1):
    nodes.append({"id":v['id'],"label":v['name']+(" ★" if v['differentiator'] else ""),"role":"focus" if v['differentiator'] else None})
    at[v['id']]=["b",f"r{i}"]; edges.append({"from":"p","to":v['id']})
  for i,s in enumerate(d['subdomains'],1):
    sd=D[s]; nodes.append({"id":s,"label":f"{sd['name']}（{sd['category']}）","role":"muted"})
    tgt=[v for v in sd['serves_values']]
    at[s]=["c",at[tgt[0]][1] if tgt else f"r{len(d['values'])}"]
    for v in tgt: edges.append({"from":s,"to":v,"label":"担う"})
  for n in nodes:
    if n.get('role') is None: n.pop('role',None)
  f=fig('domain',{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","b","g","c"],"rows":rows,"at":at,"elbow":[["p",v['id'],"vertical"] for v in d['values']]}})
  uc_by_sc={}
  for k,v in D.items():
    for sc in v.get('contributes_to',[]): uc_by_sc.setdefault(sc,[]).append(k)
  b=head(d)+sec('課題と提供価値',f'<p class="lead">{E(d["problem"])}</p>'+f+'<p class="cap">★ は競合との違いになる提供価値。右はそれを担うサブドメイン</p>')
  b+=sec('達成の基準',table(['基準','状態','寄与するユースケース'],[[f'<b>{E(x["name"])}</b>',E(x['text']),' '.join(ref(u) for u in uc_by_sc.get(x['id'],[])) or '<span class="missing">まだ無い</span>'] for x in d['success_criteria']]))
  b+=sec('範囲',table(['作るもの','作らないもの'],[[E(a),E(c)] for a,c in __import__('itertools').zip_longest(d['scope']['in'],d['scope']['out'],fillvalue='')],'two'))
  b+=fold('利害関係者と利益',len(d['stakeholders']),table(['だれ','利益'],[[E(x['who']),E(x['interest'])] for x in d['stakeholders']]))
  b+=fold('品質の要求',len(d['quality']),table(['要求','内容'],[[E(x['name']),E(x['text'])] for x in d['quality']]))
  b+=fold('競合',len(d['competitors']),table(['競合','違い'],[[E(x['name']),E(x['difference'])] for x in d['competitors']]))
  return b
def yn(v): return '<span class="yes">はい</span>' if v else '<span class="no">いいえ</span>'
def p_sd(d):
  bl=d['business_logic']
  b=head(d,f'<span class="badge cat-{d["category"]}">{E(d["category"])}</span>')
  b+=facts([('説明',E(d['description'])),('担う提供価値',' '.join(E(x2['name']) for x2 in D['DOM-1']['values'] if x2['id'] in d['serves_values']) or 'なし'),('カテゴリーの理由',E(d['category_reason']['answer']))]+([('調達',E(d['sourcing']))] if d.get('sourcing') else []))
  b+=sec('業務ロジックの性質',table(['質問','答え'],[['経緯を追う必要があるか',yn(bl['needs_tracking'])],['業務ルールが複雑か',yn(bl['complex_rules'])],['データの構造が複雑か',yn(bl['complex_data'])]]))
  if bl['rule_conditions']: b+=fold('業務ルールの根拠の条件',len(bl['rule_conditions']),' '.join(f'<code>{E(x)}</code>' for x in bl['rule_conditions']))
  return b
def p_bc(d):
  nodes=[{"id":d['id'],"label":d['name'],"role":"focus"}]; edges=[]; at={d['id']:["a","r1"]}
  for i,s in enumerate(d['subdomains']):
    nodes.append({"id":s,"label":f"{D[s]['name']}（{D[s]['category']}）","role":"muted"}); at[s]=["a","r2"]; edges.append({"from":d['id'],"to":s,"label":"対応"})
  for r in d['relations']:
    o=D[r['with']]; nodes.append({"id":o['id'],"label":o['name']}); at[o['id']]=["b","r1"]; edges.append({"from":d['id'],"to":o['id'],"label":r['pattern']})
    for s in o['subdomains']:
      if s not in at: nodes.append({"id":s,"label":f"{D[s]['name']}（{D[s]['category']}）","role":"muted"}); at[s]=["b","r2"]; edges.append({"from":o['id'],"to":s,"label":"対応"})
  f=fig('context-'+d['id'],{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","b"],"rows":["r1","r2"],"at":at}})
  b=head(d)+f'<p class="lead">{E(d["purpose"])}</p>'+sec('文脈の関係',f)
  if d['glossary']:
    b+=sec('用語集',table(['語','種類','意味'],[[f'<b>{E(x["word"])}</b>',f'<span class="tag">{E(x["kind"])}</span>',E(x['definition'])] for x in d['glossary']]))
    av=[x for x in d['glossary'] if x['avoid']]
    b+=fold('使わない語',len(av),table(['使う語','使わない語'],[[E(x['word']),E('、'.join(x['avoid']))] for x in av]))
  mem=[k for k,v in D.items() if v.get('context')==d['id'] or (v.get('scope') or {}).get('context')==d['id']]
  b+=fold('この文脈の宣言',len(mem),table(['種類','名前'],[[KIND[D[m]['kind']],ref(m)] for m in mem]))
  return b
def tname(d,t): return term(d['context'],t) or t
def p_agg(d):
  names={s['id']:tname(d,s['name']) for s in d['state']}
  for c in d['commands']:
    for a in c['args']: names[a['id']]='指定された'+tname(d,a['name'])
  ents={e['id']:tname(d,e['name']) for e in d['entities']}
  def ty(t): return ref(t) if t in D else E(ents.get(t,t))
  MUL={"1":"1つ","0..1":"なくてもよい"}
  def mul(m): return MUL.get(m, m.replace('..','〜')+'件' if '..' in m else m)
  b=head(d)+f'<p class="lead">{E(d["description"])}</p>'
  # 状態遷移の図（コマンドの事前・事後条件から導く）
  trans=[]
  for c in d['commands']:
    pre=[p for p in c['pre'] if p['op']=='eq' and str(p.get('value','')).startswith('TERM-')]
    post=[p for p in c['post'] if p['op']=='eq' and str(p.get('value','')).startswith('TERM-')]
    for a in pre:
      for z in post:
        if a['target']==z['target']: trans.append((tname(d,a['value']),tname(d,z['value']),tname(d,c['name']),[tname(d,e2['name']) for e2 in d['events'] if e2['id'] in c['emits']]))
  if trans:
    nodes=[];edges=[];seen=set()
    for a,z,cmd,evs in trans:
      for s in (a,z):
        if s not in seen: seen.add(s); nodes.append({"id":s,"label":s})
      edges.append({"from":a,"to":z,"label":cmd})
      for e2 in evs: nodes.append({"id":e2,"label":"業務イベント："+e2,"role":"muted"}); edges.append({"from":z,"to":e2,"label":"出す","dashed":True})
    b+=sec('状態の移り変わり',fig('agg-'+d['id'],{"direction":"LR","nodes":nodes,"edges":edges}))
  b+=sec('状態',table(['名前','型','個数'],[[f'<b>{E(names[s["id"]])}</b>',ty(s['type']),E(mul(s['multiplicity']))] for s in d['state']]))
  rules=[[E(cond(d,c,names)),'<span class="tag">不変条件</span>','—'] for c in d['invariants']]
  for c in d['commands']:
    rules+=[[E(cond(d,p,names)),f'<span class="tag">{E(tname(d,c["name"]))}の前</span>',E(tname(d,p['reject'])) if p.get('reject') else '—'] for p in c['pre']]
    rules+=[[E(cond(d,p,names)),f'<span class="tag">{E(tname(d,c["name"]))}の後</span>','—'] for p in c['post']]
  b+=sec('決まり',table(['条件','いつ','破ったときの理由'],rules))
  for c in d['commands']:
    b+=fold(f'「{tname(d,c["name"])}」の例',len(c['accepts'])+len(c['rejects']),table(['前の状態','結果'],[[E(x['before']+' に '+x['args']),'受け付ける：'+E(x['after'])] for x in c['accepts']]+[[E(x['before']),'拒否：'+E(tname(d,x['reason']))] for x in c['rejects']]))
  b+=fold('不変条件に違反する例',len(d['invariants']),table(['状態','結果'],[[E(c['violation']),'拒否'] for c in d['invariants']]))
  for e2 in d['events']:
    b+=fold(f'「{tname(d,e2["name"])}」の項目',len(e2['fields']),table(['項目','元'],[[E(f['name']),E(names.get(f['from'],f['from']))] for f in e2['fields']]))
  return b
def p_vo(d):
  names={c['id']:c['name'] for c in d['components']}
  gdef=[x['definition'] for x in D[d['context']]['glossary'] if x['id']==d['name']]
  b=head(d)+(f'<p class="lead">{E(gdef[0])}</p>' if gdef else '')
  b+=sec('構成する値',table(['名前','種類','桁 ・ 値','単位'],[[f'<b>{E(c["name"])}</b>',E(c['kind']),E(c['digits']),E(c['unit'])] for c in d['components']]))
  if d['invariants']: b+=sec('決まり',table(['条件','作れない値の例'],[[E(cond(d,c,names)),E(c['violation'])] for c in d['invariants']]))
  if d['operations']: b+=sec('操作',table(['名前','内容'],[[f'<b>{E(o["name"])}</b>',E(o['text'])] for o in d['operations']]))
  return b
def p_ds(d):
  nodes=[];edges=[];at={}
  for o in d['operations']:
    nodes.append({"id":o['id'],"label":tname(d,d['name']),"role":"focus"}); at[o['id']]=["b","r1"]
    for i,x in enumerate(o['inputs']):
      nodes.append({"id":f"in{i}","label":x.split('（')[0]}); at[f"in{i}"]=["a",f"r{i}"]; edges.append({"from":f"in{i}","to":o['id']})
    nodes.append({"id":"out","label":dname(o['output'])}); at["out"]=["c","r1"]; edges.append({"from":o['id'],"to":"out"})
  rows=sorted({v[1] for v in at.values()})
  f=fig('ds-'+d['id'],{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","b","c"],"rows":rows,"at":at}})
  b=head(d,f'<span class="badge">{E(d["reason"])}</span>')+sec('入力と出力',f)
  b+=facts([('参照する集約（変更しない）',' ・ '.join(ref(x) for x in d['reads']))])
  b+=sec('決まり',table(['条件','いつ'],[[E(cond(d,p,{})),'<span class="tag">計算の後</span>'] for o in d['operations'] for p in o['post']]))
  b+=fold('置く理由',1,f'<p>{E(d["reason_text"])}</p>')
  return b
def p_uc(d):
  sh=d['stakeholders']; lanes=[d['primary_actor']['role'],'システム']
  nodes=[{"id":f"h{i}","label":l,"role":"muted"} for i,l in enumerate(lanes)]
  at={f"h{i}":[f"c{i}","h"] for i in range(len(lanes))}; edges=[]; elbow=[]
  cols=[f"c{i}" for i in range(len(lanes))]+["x"]
  for i,s in enumerate(d['main_success_scenario'],1):
    lane=lanes.index(s['actor']) if s['actor'] in lanes else 1
    extra=f"（{s['to']}）" if s.get('to') and s['to'] not in lanes else ''
    nodes.append({"id":f"s{i}","label":f"{i}. {s['name']}{extra}"}); at[f"s{i}"]=[f"c{lane}",f"r{i}"]
    if i>1:
      edges.append({"from":f"s{i-1}","to":f"s{i}"})
      if at[f"s{i-1}"][0]!=at[f"s{i}"][0]: elbow.append([f"s{i-1}",f"s{i}","vertical"])
    if s.get('to') in lanes and lanes.index(s['to'])!=lane:
      pass
  for x in d['extensions']:
    step=int(x['label'][0]); fail=x['ending']=='失敗する'
    nodes.append({"id":"e"+x['label'],"label":f"{x['label']} {x['name']} → {'失敗' if fail else '戻る'}","role":"失敗" if fail else "plain"})
    at["e"+x['label']]=["x",f"r{step}"]; edges.append({"from":f"s{step}","to":"e"+x['label'],"dashed":True})
  f=fig('uc-'+d['id'],{"layout":"grid","direction":"TB","nodes":nodes,"edges":edges,"grid":{"cols":cols,"rows":["h"]+[f"r{i}" for i in range(1,len(d['main_success_scenario'])+1)],"at":at,"elbow":elbow}})
  sc=d['scope']
  b=head(d,f'<span class="badge">{E(d["level"])}</span>')
  b+=facts([('スコープ',ref(sc.get('context') or sc.get('domain'))),('主アクター',E(d['primary_actor']['role'])),('トリガー',E(d['trigger']['event'])),('事前条件','<br>'.join(f'{E(x["text"])} <span class="sub">← {ref(x["established_by"])}</span>' for x in d['preconditions']) or 'なし')])
  b+=sec('流れ',f+'<p class="cap">左が主アクター、中央がシステムのレーン。（ ）は相手の支援アクター。右の列は拡張で、点線の手順で起きる</p>')
  hdr=['保証']+[E(x['who']) for x in sh]
  rows=[]
  for kind,key,ref_key in (('最低保証','minimal_guarantees','protects'),('成功時保証','success_guarantees','satisfies')):
    for g in d[key]:
      rows.append([f'<span class="tag {"t-min" if kind=="最低保証" else "t-ok"}">{kind}</span> {E(g["text"])}']+['<span class="dot">●</span>' if x['id'] in g[ref_key] else '' for x in sh])
  b+=sec('保証と、守る相手',table(hdr,rows,'matrix'))
  b+=fold('利害関係者と利益',len(sh),table(['だれ','利益'],[[E(x['who']),E(x['interest'])] for x in sh]))
  b+=fold('手順の文',len(d['main_success_scenario']),'<ol class="plain">'+''.join(f'<li>{E(s["text"])}</li>' for s in d['main_success_scenario'])+'</ol>')
  b+=fold('拡張の処理',len(d['extensions']),table(['拡張','条件','処理','終わり'],[[x['label'],E(x['condition']),'<br>'.join(E(s) for s in x['steps']),E(x['ending'])] for x in d['extensions']]))
  aggs=list(dict.fromkeys(s['aggregate'] for s in d['main_success_scenario'] if s.get('aggregate')))
  b+=fold('変更する集約（手順から導く）',len(aggs),' ・ '.join(ref(a) for a in aggs))
  b+=fold('寄与する達成の基準',len(d['contributes_to']),' ・ '.join(E(x['name']) for x in D['DOM-1']['success_criteria'] if x['id'] in d['contributes_to']))
  b+=fold('技術およびデータのバリエーション',len(d['variations']),table(['手順','違い','値'],[[str(x['step']),E(x['varies']),E('、'.join(x['values']))] for x in d['variations']]))
  lk=d['links']; b+=fold('関連情報',len(lk.get('quality',[]))+len(lk.get('open_issues',[])),table(['種類','内容'],[['品質の要求',E(q['name'])] for q in D['DOM-1']['quality'] if q['id'] in lk.get('quality',[])]+[['未決定',E(x)] for x in lk.get('open_issues',[])]))
  return b
R={"domain":p_domain,"subdomain":p_sd,"context":p_bc,"aggregate":p_agg,"value_object":p_vo,"domain_service":p_ds,"use_case":p_uc}
nav=''
for k in ORDER:
  items=[i for i,v in D.items() if v['kind']==k]
  nav+=f'<div class="ng"><span class="nk">{KIND[k]}</span>'+''.join(f'<a href="#{i}" data-id="{i}">{E(dname(i))}</a>' for i in items)+'</div>'
pages=''.join(f'<article class="page" id="{i}">{R[v["kind"]](v)}</article>' for i,v in D.items())
css=open(os.path.join(HERE,'sample2.css')).read()
page=f'''<title>来店前注文の宣言</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;600;700&family=JetBrains+Mono:wght@400&display=swap">
<style>{css}</style>
<div class="shell"><aside class="side"><p class="brand">来店前注文</p><nav class="nav">{nav}</nav></aside>
<main class="main">{pages}</main></div>
<script>
const show=()=>{{const id=(location.hash||'#DOM-1').slice(1);let hit=[...document.querySelectorAll('.page')].some(p=>p.id===id);const cur=hit?id:'DOM-1';document.querySelectorAll('.page').forEach(p=>p.hidden=p.id!==cur);document.querySelectorAll('.nav a').forEach(a=>a.classList.toggle('on',a.dataset.id===cur));window.scrollTo(0,0)}};
addEventListener('hashchange',show);show();
</script>'''
open(f'{OUT}/viewer.html','w').write(page)
print(len(page))
