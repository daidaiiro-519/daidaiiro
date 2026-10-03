import json,html,sys,os,subprocess,itertools
HERE=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,HERE)
from data3 import D
from desc3 import DESC
HC=[0]
OUT='/home/daidaiiro/workspace/daidaiiro/.brainstorming-board/usecase-driven-concrete/sample'
DSB='/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/tool/target/release/design-svg'
for sub in ('decls','figures'): os.makedirs(f'{OUT}/{sub}',exist_ok=True)
for f in os.listdir(f'{OUT}/decls'): os.remove(f'{OUT}/decls/{f}')
for f in os.listdir(f'{OUT}/figures'): os.remove(f'{OUT}/figures/{f}')
for k,v in D.items(): json.dump(v,open(f'{OUT}/decls/{k}.json','w'),ensure_ascii=False,indent=1)
E=html.escape
THEME={"color.box-fill":"var(--card)","color.box-stroke":"var(--line-strong)","color.ink":"var(--ink)","color.ink-soft":"var(--muted)","color.ink-faint":"var(--edge)",
 "color.line":"var(--edge)","color.accent":"var(--accent)","color.accent-bg":"var(--accent-soft)","color.accent-fg":"var(--accent)","color.warn":"var(--warn)","color.warn-bg":"var(--warn-soft)",
 "role.失敗.color.box-stroke":"color.warn","role.失敗.color.text":"color.warn"}
def svg(name,args,decl):
  decl=dict(decl); decl['theme']={**THEME,**decl.get('theme',{})}
  p=f'{OUT}/figures/{name}.json'; json.dump(decl,open(p,'w'),ensure_ascii=False)
  r=subprocess.run([DSB]+args+[p,'--out',f'{OUT}/figures/{name}.svg'],capture_output=True,text=True)
  if r.returncode: print(name,r.stdout.strip(),r.stderr.strip())
  return f'<figure class="fig">{open(OUT+f"/figures/{name}.svg").read()}</figure>'
def figure(name,decl): return svg(name,['figure'],decl)
def chart(name,kind,data): return svg(name,['chart',kind],data)
KIND={"domain":"ドメイン","subdomain":"サブドメイン","context":"区切られた文脈","use_case":"ユースケース","aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
ORDER=["domain","subdomain","context","use_case","aggregate","value_object","domain_service"]
def ctx_of(d): return d['header'].get('context') or (d['header'].get('scope') or {}).get('context')
def term(ctx,t):
  for x in D.get(ctx,{}).get('ubiquitous_language',{}).get('terms',[]):
    if x['id']==t: return x['word']
def dname(i):
  d=D.get(i)
  if not d: return None
  n=d['header']['name']; return term(ctx_of(d) or '',n) or n
def idt(i): return f'<span class="id">{E(i)}</span>'
def ref(i):
  n=dname(i)
  return f'<span class="missing">{E(i)}（未作成）</span>' if n is None else f'<a class="ref" href="#{E(i)}">{E(n)}</a>'
def tbl(cols,rows,cls=''):
  return f'<div class="tw"><table class="{cls}"><thead><tr>'+''.join(f'<th>{c}</th>' for c in cols)+'</tr></thead><tbody>'+''.join('<tr>'+''.join(f'<td>{c}</td>' for c in r)+'</tr>' for r in rows)+'</tbody></table></div>'
def helpbtn(t):
  if t not in DESC: return ''
  HC[0]+=1; i=f'h{HC[0]}'
  return f'<button class="help" popovertarget="{i}" aria-label="{E(t)}の説明">?</button><div class="pop" id="{i}" popover><b>{E(t)}</b><p>{E(DESC[t])}</p></div>'
def label(t): return f'{E(t)}{helpbtn(t)}'
def block(title,body,cls=''):
  base=title.split('（')[0]
  return f'<section class="blk {cls}"><h2>{E(title)}{helpbtn(base)}</h2>{body}</section>'
def fold(label,n,body): return f'<details class="fold"><summary>{E(label)}<span class="count">{n}</span></summary><div class="fbody">{body}</div></details>'
def props(pairs): return '<dl class="props">'+''.join(f'<dt>{E(k)}</dt><dd>{v}</dd>' for k,v in pairs)+'</dl>'
def head(d,badges=''):
  return f'<header class="ph"><p class="kind">{KIND[d["kind"]]}{badges}</p><h1>{E(dname(d["id"]))}{idt(d["id"])}</h1></header>'
def badge(t,cls=''): return f' <span class="badge {cls}">{E(t)}</span>'
OPS={"eq":"は{v}","ne":"は{v}ではない","ge":"は{v}以上","le":"は{v}以下","not_empty":"は空でない"}
def cond(ctx,c,names):
  def val(v):
    if isinstance(v,str) and v.startswith('TERM-'): return term(ctx,v) or v
    return names.get(v,str(v)) if isinstance(v,str) else str(v)
  def one(c,cl=False):
    t=names.get(c['target'],c['target'])+('の件数' if c.get('agg')=='count' else '')
    v=c.get('value',''); op=OPS[c['op']]
    if isinstance(v,str) and v in names and c['op']=='eq': op='は{v}と同じ'
    if cl: op='が'+op[1:]
    return t+op.format(v=val(v))
  s=one(c); return (one(c['if'],True)+'なら、'+s) if c.get('if') else s
def yn(v): return '<span class="yes">はい</span>' if v else '<span class="no">いいえ</span>'
# ── ドメイン
def p_domain(d):
  h=d['header']; vp=d['value_proposition']
  n=len(vp['values']); rows=[]
  for i in range(1,n+1): rows+=[f"r{i}"]+([f"m{i}"] if i<n else [])
  nodes=[{"id":"p","label":"課題"}]; at={"p":["a",f"m{(n+1)//2}" if n>1 else "r1"]}; edges=[]
  for i,v in enumerate(vp['values'],1):
    nd={"id":v['id'],"label":v['name']+(" ★" if v['differentiator'] else "")}
    if v['differentiator']: nd['role']='focus'
    nodes.append(nd); at[v['id']]=["b",f"r{i}"]; edges.append({"from":"p","to":v['id']})
  for s in d['subdomains']:
    sd=D[s]; tg=sd['serves_values']; nodes.append({"id":s,"label":f"{sd['header']['name']}（{sd['classification']['category']}）","role":"muted"})
    at[s]=["c",at[tg[0]][1] if tg else f"r{n}"]
    for v in tg: edges.append({"from":s,"to":v,"label":"担う"})
  f=figure('domain',{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","b","g","c"],"rows":rows,"at":at,"elbow":[["p",v['id'],"vertical"] for v in vp['values']]}})
  b=head(d)+f'<p class="lead">{E(h["problem"])}</p>'
  vt=tbl(['提供価値','内容','競合との違い'],[[f'<b>{E(v["name"])}</b>',E(v['text']),'★' if v['differentiator'] else ''] for v in vp['values']])
  ct=tbl(['競合','違い'],[[E(c['name']),E(c['difference'])] for c in vp['competitors']])
  b+=block('提供価値',f+vt+ct)
  uc={}
  for k,v in D.items():
    for sc in v.get('contributes_to',[]): uc.setdefault(sc,[]).append(k)
  b+=block('達成の基準',tbl(['基準','観測する状態','寄与するユースケース'],[[f'<b>{E(x["name"])}</b>',E(x['text']),' '.join(ref(u) for u in uc.get(x['id'],[])) or '<span class="missing">なし</span>'] for x in d['success_criteria']]))
  b+=block('範囲',tbl(['In','Out'],[[E(a),E(c)] for a,c in itertools.zip_longest(d['scope']['in'],d['scope']['out'],fillvalue='')]))
  b+=fold('利害関係者と利益',len(d['stakeholders']),tbl(['利害関係者','利益'],[[E(x['who']),E(x['interest'])] for x in d['stakeholders']]))
  b+=fold('品質の要求',len(d['quality']),tbl(['要求','内容'],[[E(x['name']),E(x['text'])] for x in d['quality']]))
  return b
def p_sd(d):
  c=d['classification']; bl=d['business_logic']
  b=head(d,badge(c['category'],'cat-'+c['category']))+f'<p class="lead">{E(d["header"]["description"])}</p>'
  b+=block('カテゴリー',props([('カテゴリー',E(c['category'])),('競合との違いになるか',yn(c['competitive_advantage'])),('理由',E(c['answer']))]+([('調達',E(c['sourcing']))] if c.get('sourcing') else [])))
  b+=block('業務ロジックの性質',tbl(['質問','答え'],[['経緯を追う必要があるか',yn(bl['needs_tracking'])],['業務ルールが複雑か',yn(bl['complex_rules'])],['データの構造が複雑か',yn(bl['complex_data'])]])+(f'<p class="note">根拠の条件：'+' ・ '.join(f'<code>{E(x)}</code>' for x in bl['rule_conditions'])+'</p>' if bl['rule_conditions'] else ''))
  if d['serves_values']: b+=block('担う提供価値',' ・ '.join(E(v['name']) for v in D['DOM-1']['value_proposition']['values'] if v['id'] in d['serves_values']))
  return b
def p_bc(d):
  h=d['header']; nodes=[{"id":d['id'],"label":h['name'],"role":"focus"}]; edges=[]; at={d['id']:["a","r1"]}
  for s in h['subdomains']:
    nodes.append({"id":s,"label":f"{D[s]['header']['name']}（{D[s]['classification']['category']}）","role":"muted"}); at[s]=["a","r2"]; edges.append({"from":d['id'],"to":s,"label":"対応"})
  for r in d['context_map']['relations']:
    o=D[r['with']]; nodes.append({"id":o['id'],"label":o['header']['name']}); at[o['id']]=["b","r1"]; edges.append({"from":d['id'],"to":o['id'],"label":r['pattern']})
    for s in o['header']['subdomains']:
      if s not in at: nodes.append({"id":s,"label":f"{D[s]['header']['name']}（{D[s]['classification']['category']}）","role":"muted"}); at[s]=["b","r2"]; edges.append({"from":o['id'],"to":s,"label":"対応"})
  b=head(d)+f'<p class="lead">{E(h["purpose"])}</p>'
  if d['context_map']['relations']:
    f=figure('context-'+d['id'],{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","g","b"],"rows":["r1","r2"],"at":at}})
    b+=block('コンテキストマップ',f+tbl(['相手','連携のパターン','理由'],[[ref(r['with']),E(r['pattern']),E(r['reason'])] for r in d['context_map']['relations']]))
  ts=d['ubiquitous_language']['terms']
  if ts: b+=block('ユビキタス言語',tbl(['語','種類','定義','使わない語'],[[f'<b>{E(x["word"])}</b>',f'<span class="tag">{E(x["kind"])}</span>',E(x['definition']),E('、'.join(x['avoid']))] for x in ts]))
  mem=[k for k,v in D.items() if ctx_of(v)==d['id'] and v['kind']!='context']
  b+=fold('この文脈の宣言',len(mem),tbl(['種類','名前'],[[KIND[D[m]['kind']],ref(m)] for m in mem]))
  return b
MUL={"1":"1つ","0..1":"0か1つ"}
def mul(m): return MUL.get(m,m.replace('..','〜')+'件' if '..' in m else m)
def p_agg(d):
  h=d['header']; ctx=h['context']; st=d['structure']
  names={s['id']:term(ctx,s['name']) or s['name'] for s in st['state']}
  for c in d['commands']:
    for a in c['args']: names[a['id']]='指定された'+term(ctx,a['name'])
  ents={e['id']:term(ctx,e['name']) for e in st['entities']}
  def ty(t): return ref(t) if t in D else E(ents.get(t,t))
  b=head(d,f' ・ {ref(ctx)}')+f'<p class="lead">{E(h["description"])}</p>'
  rows=[]
  for s in st['state']:
    rows.append([f'<b>{E(names[s["id"]])}</b>',ty(s['type']),E(mul(s['multiplicity']))])
    for e in st['entities']:
      if e['id']==s['type']:
        rows+=[[f'<span class="indent">└ {E(x["name"])}</span>',ref(x['type']) if x['type'] in D else E(x['type']),'1つ'] for x in e['state']]
  b+=block('構造',tbl(['状態','型','個数'],rows))
  b+=block('不変条件',tbl(['条件','違反する例'],[[E(cond(ctx,i['condition'],names)),E(i['violation'])] for i in d['invariants']]))
  for c in d['commands']:
    pre=[p for p in c['preconditions'] if p['condition']['op']=='eq']; post=[p for p in c['postconditions'] if p['condition']['op']=='eq']
    nodes=[];edges=[];seen=set()
    for a in pre:
      for z in post:
        if a['condition']['target']==z['condition']['target']:
          s1=term(ctx,a['condition']['value']); s2=term(ctx,z['condition']['value'])
          for s in (s1,s2):
            if s not in seen: seen.add(s); nodes.append({"id":s,"label":s})
          edges.append({"from":s1,"to":s2,"label":term(ctx,c['name'])})
          for ev in c['emits']: nodes.append({"id":ev['id'],"label":term(ctx,ev['name']),"role":"muted"}); edges.append({"from":s2,"to":ev['id'],"label":"業務イベント","dashed":True})
    f=figure('agg-'+d['id']+'-'+c['id'],{"direction":"LR","nodes":nodes,"edges":edges}) if nodes else ''
    crow=[['引数',' ・ '.join(f'{E(term(ctx,x["name"]))}（{ref(x["type"])}）' for x in c['args']),'']]
    crow+=[['事前条件',E(cond(ctx,p['condition'],names)),E(term(ctx,p['reject']))] for p in c['preconditions']]
    crow+=[['事後条件',E(cond(ctx,p['condition'],names)),''] for p in c['postconditions']]
    crow+=[['業務イベント',f'<b>{E(term(ctx,e["name"]))}</b>（項目：{E("、".join(x["name"] for x in e["fields"]))}）',''] for e in c['emits']]
    card=tbl(['種類','内容','拒否の理由'],crow)
    ex=fold('例',1+len(c['preconditions']),tbl(['例','結果'],[[E(c['accept_example'].split(' → ')[0]),'受け付ける：'+E(c['accept_example'].split(' → ')[1])]]+[[E(p['example']),'拒否：'+E(term(ctx,p['reject']))] for p in c['preconditions']]))
    b+=block(f'コマンド「{term(ctx,c["name"])}」',f+card+ex)
  return b
def p_vo(d):
  ctx=d['header']['context']
  gdef=[x['definition'] for x in D[ctx]['ubiquitous_language']['terms'] if x['id']==d['header']['name']]
  b=head(d,f' ・ {ref(ctx)}')+(f'<p class="lead">{E(gdef[0])}</p>' if gdef else '')
  rows=[[f'<b>{E(c["name"])}</b>',E(c['kind']),E(c['digits']),E(c['unit'])] for c in d['components']]
  irows=[[E(c['name']),E(cond(ctx,i['condition'],{c['id']:c['name']})),E(i['impossible'])] for c in d['components'] for i in c['invariants']]
  b+=block('構成する値',tbl(['名前','値の種類','桁 ・ 値','単位'],rows))
  if irows: b+=block('不変条件',tbl(['構成する値','条件','作れない値の例'],irows))
  if d['operations']: b+=block('操作',tbl(['操作','結果'],[[f'<b>{E(o["name"])}</b>',E(o['text'])] for o in d['operations']]))
  return b
def p_ds(d):
  h=d['header']; ctx=h['context']
  b=head(d,badge(h['reason'])+f' ・ {ref(ctx)}')+props([('参照する集約（変更しない）',' ・ '.join(ref(x) for x in d['reads'])),('置く理由',E(h['reason_text']))])
  for o in d['operations']:
    nodes=[{"id":"op","label":o['name'],"role":"focus"},{"id":"out","label":dname(o['output'])}]; at={"op":["b","r1"],"out":["c","r1"]}; edges=[{"from":"op","to":"out"}]
    for i,x in enumerate(o['inputs']):
      nodes.append({"id":f"in{i}","label":x['name']}); at[f"in{i}"]=["a",f"r{i}"]; edges.append({"from":f"in{i}","to":"op"})
    rws=sorted({v[1] for v in at.values()})
    f=figure('ds-'+d['id']+'-'+o['id'],{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","b","c"],"rows":rws,"at":at}})
    crow=[['入力',f'{E(x["name"])}（{ref(x["type"])}）'] for x in o['inputs']]+[['出力',ref(o['output'])]]+[['事後条件',E(cond(ctx,p['condition'],{}))] for p in o['postconditions']]
    card=tbl(['種類','内容'],crow)
    b+=block(f'操作「{o["name"]}」',f+card)
  return b
ACT={}
def actor(a):
  if a not in ACT: ACT[a]=len(ACT)%4
  return f'<span class="act a{ACT[a]}">{E(a)}</span>'
def chips(names,cls=''): return ''.join(f'<span class="pill {cls}">{E(n)}</span>' for n in names)
def p_uc(d):
  ACT.clear()
  h=d['header']; sc=d['scenario']; ctx=h['scope'].get('context')
  for a in [h['primary_actor'],'システム']+sc['supporting_actors']: actor(a)
  sh={x['id']:x for x in d['stakeholders']}; g=d['guarantees']
  def whos(ids): return chips([sh[i]['who'] for i in ids])
  def cmd(r):
    a,c=r.split('.'); ag=D[a]; ca=ag['header']['context']
    cm=[x for x in ag['commands'] if x['id']==c][0]
    return f'{ref(a)}「{E(term(ca,cm["name"]))}」'
  def qual(ids): return '、'.join(E(q['text']) for q in D['DOM-1']['quality'] if q['id'] in ids)
  def links(s):
    out=[]
    if s.get('invokes'): out.append(f'<span class="ln"><span class="k">コマンド</span>{cmd(s["invokes"])}</span>')
    if s.get('quality'): out.append(f'<span class="ln"><span class="k">品質の要求</span>{qual(s["quality"])}</span>')
    return ''.join(out)
  parts=[h['primary_actor'],'システム']+sc['supporting_actors']; msgs=[]; groups=[]
  def add(s):
    frm=s['actor']; to=s.get('to') or frm
    msgs.append({"from":frm,"to":to,"label":f"{s['no']} {s['name']}"})
    if s.get('reply'): msgs.append({"from":to,"to":frm,"label":s['reply'],"kind":"return"})
  for s in sc['steps']:
    add(s)
    for x in s['extensions']:
      st=len(msgs)
      for t in x['steps']: add(t)
      groups.append({"label":'break' if x['ending']=='失敗' else 'opt',"cases":[{"name":f"{x['label']} {x['name']} → {x['ending']}","span":[st,len(msgs)-1]}]})
  seq=chart('uc-'+d['id'],'exchange',{"participants":parts,"steps":msgs,"groups":groups,"theme":{"font.size-small":14,"font.size":15,"chart.exchange-col-w":220,"chart.pad":6,"chart.exchange-row-h":40}})
  scs=[x for x in D['DOM-1']['success_criteria'] if x['id'] in d['contributes_to']]
  b=head(d,badge(h['level'])+f' ・ スコープ {ref(ctx)}')
  tiles=[('主アクター',actor(h['primary_actor'])),('支援アクター',''.join(actor(a) for a in sc['supporting_actors'])),('トリガー',E(h['trigger'])),('寄与する達成の基準',' '.join(f'<a class="ref" href="#DOM-1">{E(x["name"])}</a>' for x in scs))]
  b+='<div class="tiles">'+''.join(f'<div class="tile"><div class="tl">{label(k)}</div><div class="tv">{v}</div></div>' for k,v in tiles)+'</div>'
  def gcard(kind,items,key,cls):
    return f'<div class="gcard {cls}"><h3>{label(kind)}</h3>'+''.join(f'<div class="gi"><b>{E(x["name"])}</b><span class="gt">{E(x["text"])}</span><span class="gw">{whos(x[key])}</span></div>' for x in items)+'</div>'
  b+='<div class="gpair">'+gcard('成功時保証',g['success'],'satisfies','ok')+gcard('最低保証',g['minimal'],'protects','ng')+'</div>'
  b+=block('事前条件',tbl(['事前条件','成り立たせるユースケース'],[[E(x['text']),ref(x['established_by'])] for x in d['preconditions']]))
  rows=''
  for s in sc['steps']:
    rows+=f'<tr><td class="num"><span>{s["no"]}</span></td><td>{actor(s["actor"])}</td><td><b>{E(s["name"])}</b><span class="txt">{E(s["text"])}</span>'+(f'<span class="ln"><span class="k">戻りメッセージ</span>{E(s["reply"])}</span>' if s.get('reply') else '')+links(s)+f'</td><td>{whos(s.get("serves",[]))}</td><td class="exl">'+''.join(f'<span class="pill {"bad" if x["ending"]=="失敗" else "alt"}">{E(x["label"])}</span>' for x in s['extensions'])+'</td></tr>'
  rows+='<tr class="end"><td></td><td></td><td colspan="3">→ 成功時保証が成り立つ</td></tr>'
  b+=block('主成功シナリオ','<div class="tw"><table class="mss"><thead><tr><th>#</th><th>アクター</th><th>手順</th><th>守る利害関係者</th><th>拡張</th></tr></thead><tbody>'+rows+'</tbody></table></div>')
  ex=''
  for s in sc['steps']:
    for x in s['extensions']:
      fail=x['ending']=='失敗'
      held=chips([m['name'] for m in g['minimal'] if m['id'] in x.get('guarantees_hold',[])],'bad') if fail else ''
      steps=''.join(f'<div class="hs"><span class="no">{E(t["no"])}</span>{actor(t["actor"])}<span><b>{E(t["name"])}</b><span class="txt">{E(t["text"])}</span>{links(t)}</span></div>' for t in x['steps'])
      ex+=f'<div class="xcard {"fail" if fail else ""}"><div class="xh"><span class="xl">{E(x["label"])}</span><b>{E(x["name"])}</b><span class="pill {"bad" if fail else "alt"}">{E("失敗" if fail else x["ending"])}</span></div><p class="xc">{E(x["condition"].rstrip("："))}<span class="kind">{E(x["condition_kind"])}</span></p>{steps}'+(f'<div class="xg"><span class="k">成り立つ最低保証</span>{held}</div>' if fail else '')+'</div>'
  b+=block('拡張','<div class="xgrid">'+ex+'</div>')
  srows=[]
  for k,x in sh.items():
    st='、'.join(str(s['no']) for s in sc['steps'] if k in s.get('serves',[]))
    gs=chips([m['name'] for m in g['success'] if k in m['satisfies']],'ok')+chips([m['name'] for m in g['minimal'] if k in m['protects']],'bad')
    srows.append([f'<b>{E(x["who"])}</b>',f'<span class="txt">{E(x["interest"])}</span>',st or '<span class="missing">なし</span>',gs or '<span class="missing">なし</span>'])
  b+=block('利害関係者と利益',tbl(['利害関係者','利益','守る手順','守る保証'],srows))
  vr=[[str(s['no']),E(v['varies']),'、'.join(E(x) for x in v['values'])] for s in sc['steps'] for v in s.get('variations',[])]
  if vr: b+=block('技術およびデータのバリエーション',tbl(['手順','違い','値'],vr))
  if d['links'].get('open_issues'): b+=block('未決定事項',tbl(['未決定事項'],[[E(x)] for x in d['links']['open_issues']]))
  b+=f'<details class="fold"><summary>シーケンス図{helpbtn("シーケンス図")}</summary><div class="fbody">{seq}</div></details>'
  return b
R={"domain":p_domain,"subdomain":p_sd,"context":p_bc,"aggregate":p_agg,"value_object":p_vo,"domain_service":p_ds,"use_case":p_uc}
nav=''.join(f'<div class="ng"><span class="nk">{KIND[k]}</span>'+''.join(f'<a href="#{i}" data-id="{i}">{E(dname(i))}</a>' for i,v in D.items() if v['kind']==k)+'</div>' for k in ORDER)
pages=''.join(f'<article class="page" id="{i}">{R[v["kind"]](v)}</article>' for i,v in D.items())
css=open(os.path.join(HERE,'sample3.css')).read()
page=f'''<title>来店前注文の宣言</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;600;700&family=JetBrains+Mono:wght@400&display=swap">
<style>{css}</style>
<div class="shell"><aside class="side"><p class="brand">来店前注文</p><nav class="nav">{nav}</nav></aside>
<main class="main">{pages}</main></div>
<script>
const show=()=>{{const id=(location.hash||'#DOM-1').slice(1);const hit=[...document.querySelectorAll('.page')].some(p=>p.id===id);const cur=hit?id:'DOM-1';document.querySelectorAll('.page').forEach(p=>p.hidden=p.id!==cur);document.querySelectorAll('.nav a').forEach(a=>a.classList.toggle('on',a.dataset.id===cur));window.scrollTo(0,0)}};
addEventListener('hashchange',show);show();
</script>'''
open(f'{OUT}/viewer.html','w').write(page); print(len(page))
