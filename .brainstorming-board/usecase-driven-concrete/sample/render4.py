# 宣言を、共通の部品（タイル ・ カード ・ 札 ・ 表 ・ 図）とデザイントークンで描画する
import json,html,sys,os,subprocess,itertools
HERE=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,HERE)
from data3 import D
from desc3 import DESC
import drift, spec_drift, sim
SIM=sim.simulate()
SPEC=spec_drift.checks()
CONDS=drift.conditions(); REPORT=drift.report(CONDS); JUDGE,EXTRA=drift.judge(CONDS,REPORT)
STONE={'合格':'t-success','不合格':'t-fail','欠け':'t-fail','古い':'t-fail','レベル違い':'t-fail','未実行':'t-caution','免除':''}
OUT='/home/daidaiiro/workspace/daidaiiro/.brainstorming-board/usecase-driven-concrete/sample'
DSB='/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/tool/target/release/design-svg'
for sub in ('decls','figures'):
  os.makedirs(f'{OUT}/{sub}',exist_ok=True)
  for f in os.listdir(f'{OUT}/{sub}'): os.remove(f'{OUT}/{sub}/{f}')
for k,v in D.items(): json.dump(v,open(f'{OUT}/decls/{k}.json','w'),ensure_ascii=False,indent=1)
json.dump({"conditions":CONDS,"retired":drift.RETIRED},open(f'{OUT}/conditions.json','w'),ensure_ascii=False,indent=1)
json.dump(REPORT,open(f'{OUT}/report.json','w'),ensure_ascii=False,indent=1)
E=html.escape; HC=[0]
THEME={"color.box-fill":"var(--figure-box)","color.box-stroke":"var(--figure-box-stroke)","color.ink":"var(--figure-ink)","color.ink-soft":"var(--figure-soft)","color.ink-faint":"var(--figure-edge)",
 "color.line":"var(--figure-edge)","color.accent":"var(--figure-accent)","color.accent-bg":"var(--figure-accent-bg)","color.accent-fg":"var(--figure-accent)","color.warn":"var(--figure-warn)","color.warn-bg":"var(--figure-warn-bg)"}
def svg(name,args,decl):
  decl=dict(decl); decl['theme']={**THEME,**decl.get('theme',{})}
  p=f'{OUT}/figures/{name}.json'; json.dump(decl,open(p,'w'),ensure_ascii=False)
  r=subprocess.run([DSB]+args+[p,'--out',f'{OUT}/figures/{name}.svg'],capture_output=True,text=True)
  if r.returncode: print(name,r.stdout.strip(),r.stderr.strip())
  return f'<figure class="fig">{open(OUT+f"/figures/{name}.svg").read()}</figure>'
figure=lambda n,d: svg(n,['figure'],d)
chart=lambda n,k,d: svg(n,['chart',k],d)
KIND={"domain":"ドメイン","subdomain":"サブドメイン","context":"区切られた文脈","use_case":"ユースケース","aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
ORDER=["domain","subdomain","context","use_case","aggregate","value_object","domain_service"]
CAT={"中核":"t-core","補完":"t-supporting","一般":"t-generic"}
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
  n=dname(i); return f'<span class="missing">{E(i)}（未作成）</span>' if n is None else f'<a class="ref" href="#{E(i)}">{E(n)}</a>'
def helpbtn(t):
  if t not in DESC: return ''
  HC[0]+=1; i=f'h{HC[0]}'
  return f'<button class="help" popovertarget="{i}" aria-label="{E(t)}の説明">?</button><div class="pop" id="{i}" popover><b>{E(t)}</b><p>{E(DESC[t])}</p></div>'
def lab(t): return f'{E(t)}{helpbtn(t)}'
def pill(t,tone=''): return f'<span class="pill {tone}">{E(t)}</span>'
def pills(ts,tone=''): return ''.join(pill(t,tone) for t in ts)
def tbl(cols,rows):
  return '<div class="tw"><table><thead><tr>'+''.join(f'<th>{c}</th>' for c in cols)+'</tr></thead><tbody>'+''.join('<tr>'+''.join(f'<td>{c}</td>' for c in r)+'</tr>' for r in rows)+'</tbody></table></div>'
def block(title,body): return f'<section class="blk"><h2>{E(title)}{helpbtn(title)}</h2>{body}</section>'
def tiles(pairs): return '<div class="tiles">'+''.join(f'<div class="tile"><div class="tl">{lab(k)}</div><div class="tv">{v}</div></div>' for k,v in pairs)+'</div>'
def card(title,body,cls=''): return f'<div class="card {cls}"><h3>{title}</h3>{body}</div>'
def cards(xs): return '<div class="cards">'+''.join(xs)+'</div>'
def item(name,text='',extra=''): return f'<div class="item"><b>{E(name)}</b>'+(f'<span class="txt">{E(text)}</span>' if text else '')+extra+'</div>'
def tchip(cid):
  st=JUDGE.get(cid,'')
  return f'<span class="tc {STONE.get(st,"")}" title="{E(st)}">{E(cid)}</span>'
def tblock(decl):
  cs=[c for c in CONDS if c['decl']==decl]
  if not cs: return ''
  return block('テスト条件',tbl(['ID','対象','確かめること','求めるレベル','ハッシュ値','状態'],[[tchip(c['id']),E(c['label']),E(c['checks']),pill(c['required_level']),f'<span class="no">{c["hash"]}</span>',pill(JUDGE[c['id']],STONE[JUDGE[c['id']]])] for c in cs]))
def head(d,badges='',lead=''):
  return f'<header class="ph"><p class="kind">{KIND[d["kind"]]}{badges}</p><h1>{E(dname(d["id"]))}{idt(d["id"])}</h1></header>'+(f'<p class="lead">{E(lead)}</p>' if lead else '')
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
def yn(v): return '<span class="yes">はい</span>' if v else '<span class="no-ans">いいえ</span>'
# ── ドメイン
def p_domain(d):
  h=d['header']; vp=d['value_proposition']
  uc={}
  for k,v in D.items():
    for sc in v.get('contributes_to',[]): uc.setdefault(sc,[]).append(k)
  b=head(d)+tiles([('課題',E(h['problem']))])
  vals=[card(E(v['name'])+(pill('★ 競合との違い','t-accent') if v['differentiator'] else ''),f'<span class="txt">{E(v["text"])}</span>'+''.join(f'<span class="ln"><span class="k">担う</span>{ref(s)} {pill(D[s]["classification"]["category"],CAT[D[s]["classification"]["category"]])}</span>' for s in d['subdomains'] if v['id'] in D[s]['serves_values']),'left-accent' if v['differentiator'] else 'left-neutral') for v in vp['values']]
  vals+=[card('競合 ・ '+E(c['name']),f'<span class="txt">{E(c["difference"])}</span>','left-neutral') for c in vp['competitors']]
  b+=block('提供価値',cards(vals))
  b+=block('達成の基準',cards([card(E(x['name']),f'<span class="txt">{E(x["text"])}</span><span class="ln"><span class="k">寄与するユースケース</span>{" ".join(ref(u) for u in uc.get(x["id"],[])) or "<span class=missing>なし</span>"}</span>') for x in d['success_criteria']]))
  b+=block('範囲',cards([card('作るもの（In）',''.join(item(x) for x in d['scope']['in']),'left-accent'),card('作らないもの（Out）',''.join(item(x) for x in d['scope']['out']),'left-neutral')]))
  b+=block('サブドメイン',cards([card(ref(s)+pill(D[s]['classification']['category'],CAT[D[s]['classification']['category']]),f'<span class="txt">{E(D[s]["header"]["description"])}</span>') for s in d['subdomains']]))
  b+=block('利害関係者',tbl(['利害関係者','関心'],[[f'<b>{E(x["who"])}</b>',f'<span class="txt">{E(x["interest"])}</span>'] for x in d['stakeholders']]))
  b+=block('品質の要求',tbl(['要求','内容'],[[f'<b>{E(x["name"])}</b>',f'<span class="txt">{E(x["text"])}</span>'] for x in d['quality']]))
  return b
def p_sd(d):
  c=d['classification']; bl=d['business_logic']
  b=head(d,' '+pill(c['category'],CAT[c['category']]),d['header']['description'])
  b+=block('カテゴリー',tiles([('カテゴリー',pill(c['category'],CAT[c['category']])),('競合との違いになるか',yn(c['competitive_advantage'])),('理由',E(c['answer']))]+([('調達',E(c['sourcing']))] if c.get('sourcing') else [])))
  b+=block('業務ロジックの性質',tiles([('経緯を追う必要があるか',yn(bl['needs_tracking'])),('業務ルールが複雑か',yn(bl['complex_rules'])),('データの構造が複雑か',yn(bl['complex_data']))])+(f'<p class="ln"><span class="k">根拠の条件</span>{" ".join(pill(x) for x in bl["rule_conditions"])}</p>' if bl['rule_conditions'] else ''))
  if d['serves_values']: b+=block('担う提供価値',cards([card(E(v['name']),f'<span class="txt">{E(v["text"])}</span>','left-accent') for v in D['DOM-1']['value_proposition']['values'] if v['id'] in d['serves_values']]))
  return b
def p_bc(d):
  h=d['header']; b=head(d,'',h['purpose'])
  rel=d['context_map']['relations']
  if rel:
    nodes=[{"id":d['id'],"label":h['name'],"role":"focus"}]; edges=[]; at={d['id']:["a","r1"]}
    for s in h['subdomains']: nodes.append({"id":s,"label":f"{D[s]['header']['name']}（{D[s]['classification']['category']}）","role":"muted"}); at[s]=["a","r2"]; edges.append({"from":d['id'],"to":s,"label":"対応"})
    for r in rel:
      o=D[r['with']]; nodes.append({"id":o['id'],"label":o['header']['name']}); at[o['id']]=["b","r1"]; edges.append({"from":d['id'],"to":o['id'],"label":r['pattern']})
      for s in o['header']['subdomains']:
        if s not in at: nodes.append({"id":s,"label":f"{D[s]['header']['name']}（{D[s]['classification']['category']}）","role":"muted"}); at[s]=["b","r2"]; edges.append({"from":o['id'],"to":s,"label":"対応"})
    f=figure('context-'+d['id'],{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","g","b"],"rows":["r1","r2"],"at":at}})
    b+=block('コンテキストマップ',f+cards([card(ref(r['with'])+pill(r['pattern'],'t-accent'),f'<span class="txt">{E(r["reason"])}</span>') for r in rel]))
  else:
    b+=block('対応するサブドメイン',' '.join(ref(s)+pill(D[s]['classification']['category'],CAT[D[s]['classification']['category']]) for s in h['subdomains']))
  ts=d['ubiquitous_language']['terms']
  if ts:
    kinds=list(dict.fromkeys(x['kind'] for x in ts))
    b+=block('ユビキタス言語',cards([card(E(k),''.join(item(x['word'],x['definition'],(f'<span class="ln"><span class="k">使わない語</span>{" ".join(f"<span class=avoid>{E(a)}</span>" for a in x["avoid"])}</span>' if x['avoid'] else '')) for x in ts if x['kind']==k)) for k in kinds]))
  mem=[k for k,v in D.items() if ctx_of(v)==d['id'] and v['kind']!='context']
  if mem: b+=block('この文脈の宣言',tbl(['種類','名前'],[[KIND[D[m]['kind']],ref(m)] for m in mem]))
  return b
MUL={"1":"1つ","0..1":"0か1つ"}
def mul(m): return MUL.get(m,m.replace('..','〜')+'件' if '..' in m else m)
def p_agg(d):
  h=d['header']; ctx=h['context']; st=d['structure']
  names={s['id']:term(ctx,s['name']) or s['name'] for s in st['state']}
  for c in d['commands']:
    for a in c['args']: names[a['id']]='指定された'+term(ctx,a['name'])
  ents={e['id']:e for e in st['entities']}
  b=head(d,f' ・ {ref(ctx)}')+tiles([('守る一貫性',E(h['description']))])
  rows=[]
  for s in st['state']:
    t=s['type']; ty=ref(t) if t in D else E(term(ctx,ents[t]['name']) if t in ents else t)
    rows.append([f'<b>{E(names[s["id"]])}</b>',ty,pill(mul(s['multiplicity']))])
    if t in ents:
      rows+=[[f'<span class="txt">└ {E(x["name"])}</span>',ref(x['type']) if x['type'] in D else E(x['type']),pill('1つ')] for x in ents[t]['state']]
  b+=block('構造',tbl(['状態','型','個数'],rows))
  b+=block('不変条件',cards([card(E(cond(ctx,i['condition'],names))+tchip(d['id']+'.'+i['id']),f'<span class="ln"><span class="k">違反する例</span>{pill(i["violation"],"t-fail")}</span>','left-accent') for i in d['invariants']]))
  for c in d['commands']:
    cname=term(ctx,c['name'])
    pre=[p for p in c['preconditions'] if p['condition']['op']=='eq']; post=[p for p in c['postconditions'] if p['condition']['op']=='eq']
    nodes=[];edges=[];seen=set()
    for a in pre:
      for z in post:
        if a['condition']['target']==z['condition']['target']:
          s1=term(ctx,a['condition']['value']); s2=term(ctx,z['condition']['value'])
          for x in (s1,s2):
            if x not in seen: seen.add(x); nodes.append({"id":x,"label":x})
          edges.append({"from":s1,"to":s2,"label":cname})
          for ev in c['emits']: nodes.append({"id":ev['id'],"label":term(ctx,ev['name']),"role":"muted"}); edges.append({"from":s2,"to":ev['id'],"label":"業務イベント","dashed":True})
    f=figure('agg-'+d['id']+'-'+c['id'],{"direction":"LR","nodes":nodes,"edges":edges}) if nodes else ''
    precs=card('事前条件',''.join(item(cond(ctx,p['condition'],names),'',f'<span class="ln"><span class="k">拒否の理由</span>{pill(term(ctx,p["reject"]),"t-fail")}{tchip(d["id"]+"."+c["id"]+"."+p["id"])}</span>') for p in c['preconditions']) or '<span class="txt">なし</span>','left-neutral')
    posts=card('事後条件',''.join(item(cond(ctx,p['condition'],names)) for p in c['postconditions'])+''.join(f'<span class="ln"><span class="k">受け付ける例</span>{E(o["text"])} {tchip(d["id"]+"."+c["id"]+"."+o["id"])}</span>' for o in c.get('accept_examples',[])),'left-accent')
    args=' '.join(f'{E(term(ctx,x["name"]))}（{ref(x["type"])}）' for x in c['args'])
    evs=' '.join(pill(term(ctx,e['name']),'t-accent') for e in c['emits'])
    b+=f'<section class="blk"><h2>コマンド「{E(cname)}」{helpbtn("コマンド")}</h2>{f}'+tiles([('引数',args)]+([('業務イベント',evs)] if evs else []))+'<div style="height:.6rem"></div>'+cards([precs,posts])+'</section>'
  b+=tblock(d['id'])
  return b
def p_vo(d):
  ctx=d['header']['context']
  gdef=[x['definition'] for x in D[ctx]['ubiquitous_language']['terms'] if x['id']==d['header']['name']]
  b=head(d,f' ・ {ref(ctx)}',gdef[0] if gdef else '')
  b+=block('構成する値',tbl(['名前','値の種類','桁 ・ 値','単位'],[[f'<b>{E(c["name"])}</b>',pill(c['kind']),E(c['digits']),E(c['unit'])] for c in d['components']]))
  inv=[card(E(cond(ctx,i['condition'],{c['id']:c['name']})),f'<span class="ln"><span class="k">作れない値</span>{pill(i["impossible"],"t-fail")}</span>','left-accent') for c in d['components'] for i in c['invariants']]
  if inv: b+=block('不変条件',cards(inv))
  if d['operations']: b+=block('操作',cards([card(E(o['name']),f'<span class="txt">{E(o["text"])}</span>') for o in d['operations']]))
  b+=tblock(d['id'])
  return b
def p_ds(d):
  h=d['header']; ctx=h['context']
  b=head(d,' '+pill(h['reason'],'t-accent')+f' ・ {ref(ctx)}')+tiles([('置く理由',E(h['reason_text'])),('参照する集約（変更しない）',' ・ '.join(ref(x) for x in d['reads']))])
  for o in d['operations']:
    nodes=[{"id":"op","label":o['name'],"role":"focus"},{"id":"out","label":dname(o['output'])}]; at={"op":["b","r1"],"out":["c","r1"]}; edges=[{"from":"op","to":"out"}]
    for i,x in enumerate(o['inputs']): nodes.append({"id":f"in{i}","label":x['name']}); at[f"in{i}"]=["a",f"r{i}"]; edges.append({"from":f"in{i}","to":"op"})
    f=figure('ds-'+d['id']+'-'+o['id'],{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","b","c"],"rows":sorted({v[1] for v in at.values()}),"at":at}})
    b+=f'<section class="blk"><h2>操作「{E(o["name"])}」{helpbtn("操作")}</h2>{f}'+cards([card('入力と出力',''.join(item(x['name'],'',f'<span class="ln">{ref(x["type"])}</span>') for x in o['inputs'])+f'<div class="item"><span class="k">出力</span>{ref(o["output"])}</div>','left-neutral'),card('事後条件',''.join(item(cond(ctx,p['condition'],{}),'',f'<span class="ln">{tchip(d["id"]+"."+o["id"]+"."+p["id"])}</span>') for p in o['postconditions']),'left-accent')])+'</section>'
  b+=tblock(d['id'])
  return b
def p_uc(d):
  h=d['header']; sc=d['scenario']; ctx=h['scope'].get('context')
  role={h['primary_actor']:'primary','システム':'system'}; role.update({a:'supporting' for a in sc['supporting_actors']})
  act=lambda a: f'<span class="act {role.get(a,"system")}">{E(a)}</span>'
  sh={x['id']:x for x in d['stakeholders']}; g=d['guarantees']
  whos=lambda ids: pills([sh[i]['who'] for i in ids])
  def cmd(r):
    a,c=r.split('.'); ag=D[a]; ca=ag['header']['context']; cm=[x for x in ag['commands'] if x['id']==c][0]
    return f'{ref(a)}「{E(term(ca,cm["name"]))}」'
  qual=lambda ids: '、'.join(E(q['text']) for q in D['DOM-1']['quality'] if q['id'] in ids)
  def links(s):
    o=''
    if s.get('reply'): o+=f'<span class="ln"><span class="k">戻りメッセージ</span>{E(s["reply"])}</span>'
    if s.get('invokes'): o+=f'<span class="ln"><span class="k">コマンド</span>{cmd(s["invokes"])}</span>'
    if s.get('keeps'): o+=f'<span class="ln"><span class="k">守る最低保証</span>{pills([m["name"] for m in g["minimal"] if m["id"] in s["keeps"]],"t-minimal")}</span>'
    if s.get('quality'): o+=f'<span class="ln"><span class="k">品質の要求</span>{qual(s["quality"])} '+''.join(tchip(f'{d["id"]}.{s["id"]}.{q}') for q in s['quality'])+'</span>'
    return o
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
  b=head(d,' '+pill(h['level'])+f' ・ スコープ {ref(ctx)}')
  b+=tiles([('主アクター',act(h['primary_actor'])),('支援アクター',''.join(act(a) for a in sc['supporting_actors'])),('トリガー',E(h['trigger'])),('寄与する達成の基準',' '.join(f'<a class="ref" href="#DOM-1">{E(x["name"])}</a>' for x in scs))])
  def pref(rr):
    a,c,p=rr.split('.'); cm=[x for x in D[a]['commands'] if x['id']==c][0]; ca=D[a]['header']['context']
    x=[y for y in cm['preconditions']+cm['postconditions'] if y['id']==p][0]
    return f'{ref(a)}「{E(term(ca,cm["name"]))}」の{"事前条件" if p.startswith("PRE") else "事後条件"}<span class="no">{E(p)}</span>'
  sline=lambda x: f'<span class="ln"><span class="k">成り立たせる事後条件</span>{"、".join(pref(r) for r in x.get("established_by",[])) or "<span class=missing>なし</span>"}</span>'
  gc=lambda title,items,key,cls: card(lab(title),''.join(item(x['name'],x['text'],f'<span class="ln">{whos(x[key])}</span>'+(sline(x) if key=='satisfies' else '')) for x in items),cls)
  b+='<div style="height:.75rem"></div>'+cards([gc('成功時保証',g['success'],'satisfies','top-success'),gc('最低保証',g['minimal'],'protects','top-minimal')])
  b+=block('事前条件',tbl(['事前条件','成り立たせるユースケース','これで起こらない拒否'],[[E(x['text']),ref(x['established_by']),'、'.join(pref(r) for r in x.get('ensures',[]))] for x in d['preconditions']]))
  rows=''.join(f'<tr><td class="num"><span>{s["no"]}</span></td><td>{act(s["actor"])}</td><td><b>{E(s["name"])}</b><span class="txt">{E(s["text"])}</span>{links(s)}</td><td>{whos(s.get("serves",[]))}</td><td>'+''.join(pill(x['label'],'t-fail' if x['ending']=='失敗' else 't-return') for x in s['extensions'])+'</td></tr>' for s in sc['steps'])
  rows+=f'<tr class="end"><td></td><td></td><td colspan="3">→ 成功時保証が成り立つ {tchip(d["id"]+".M")}</td></tr>'
  b+=block('主成功シナリオ','<div class="tw"><table><thead><tr><th>#</th><th>アクター</th><th>手順</th><th>守る利害関係者</th><th>拡張</th></tr></thead><tbody>'+rows+'</tbody></table></div>')
  ex=[]
  for s in sc['steps']:
    for x in s['extensions']:
      fail=x['ending']=='失敗'
      body=f'<span class="txt">{E(x["condition"].rstrip("："))} ・ {E(x["condition_kind"])}</span>'+''.join(f'<div class="hs"><span class="no">{E(t["no"])}</span>{act(t["actor"])}<span><b>{E(t["name"])}</b><span class="txt">{E(t["text"])}</span>{links(t)}</span></div>' for t in x['steps'])
      if x.get('handles'): body=f'<span class="ln"><span class="k">扱う拒否</span>{"、".join(pref(r) for r in x["handles"])}</span>'+body
      ex.append(card(f'<span class="no">{E(x["label"])}</span>{E(x["name"])}'+pill('失敗' if fail else x['ending'],'t-fail' if fail else 't-return')+tchip(d['id']+'.'+x['id']),body,'left-fail' if fail else 'left-return'))
  b+=block('拡張',cards(ex))
  srows=[[f'<b>{E(x["who"])}</b>',f'<span class="txt">{E(x["interest"])}</span>','、'.join(str(s['no']) for s in sc['steps'] if k in s.get('serves',[])) or '<span class="missing">なし</span>',pills([m['name'] for m in g['success'] if k in m['satisfies']],'t-success')+pills([m['name'] for m in g['minimal'] if k in m['protects']],'t-minimal')] for k,x in sh.items()]
  b+=block('利害関係者と利益',tbl(['利害関係者','利益','守る手順','守る保証'],srows))
  vr=[[str(s['no']),E(v['varies']),pills(v['values'])] for s in sc['steps'] for v in s.get('variations',[])]
  if vr: b+=block('技術およびデータのバリエーション',tbl(['手順','違い','値'],vr))
  if d['links'].get('open_issues'): b+=block('未決定事項',tbl(['未決定事項'],[[E(x)] for x in d['links']['open_issues']]))
  b+=tblock(d['id'])
  b+=f'<details class="fold"><summary>シーケンス図{helpbtn("シーケンス図")}</summary><div class="fbody">{seq}</div></details>'
  return b
def p_drift():
  import collections
  cnt=collections.Counter(JUDGE.values())
  order=['合格','不合格','欠け','古い','未実行','レベル違い']
  ok_all=all(v in('合格','免除') for v in JUDGE.values()) and not EXTRA and all(x['status']=='合格' for x in SPEC)
  b='<header class="ph"><p class="kind">テスト</p><h1>突き合わせ</h1></header>'+f'<p class="lead">報告：{E(REPORT["producer"])} ・ 宣言の版 {E(REPORT["spec_revision"])}</p>'
  b+=tiles([('終了基準',pill('満たしている','t-success') if ok_all else pill('満たしていない','t-fail')),('条件','宣言どうしのずれが0件で、テスト条件がすべて合格し、余りが0件')])
  scnt=collections.Counter(x['status'] for x in SPEC)
  b+='<h2 class="sec">宣言どうし</h2>'+tiles([(k,f'<b style="font-size:1.3rem">{scnt.get(k,0)}</b>') for k in ['ずれ','対応の欠け','確かめ直し','レビュー','合格']])
  CAT={'structure':'構造で検査','link':'対応の欄で検査','change':'上流の変更','review':'人のレビュー'}
  SST={'合格':'t-success','ずれ':'t-fail','対応の欠け':'t-fail','確かめ直し':'t-caution','レビュー':'t-caution'}
  def at(i):
    if i.startswith('TERM-'): return f'「{E(term("BC-1",i))}」'
    return ref(i.split('.')[0])+(f' <span class="no">{E(".".join(i.split(".")[1:]))}</span>' if '.' in i else '') if i and i.split('.')[0] in D else E(i)
  srow=[[pill(x['status'],SST[x['status']]),E(CAT[x['category']]),E(x['check']),at(x['from']),'、'.join(at(t) for t in x['to'].split('・')) if x['to'] else '',E(x['text'])] for x in sorted(SPEC,key=lambda x:(x['status']=='合格',list(CAT).index(x['category'])))]
  b+=block('宣言どうしの検査',tbl(['状態','分け方','検査','参照元','参照先','内容'],srow))
  b+='<h2 class="sec">宣言とテスト実装</h2>'+tiles([(k,f'<b style="font-size:1.3rem">{cnt.get(k,0)}</b>') for k in order]+[('余り',f'<b style="font-size:1.3rem">{len(EXTRA)}</b>')])
  b+=block('ドリフトの種類','<p class="txt">状態ごとの意味は ❓ にある。合格以外はすべて、終了基準を満たさない。</p>')
  rows=[[tchip(c['id']),ref(c['decl']),E(c['label']),E(c['checks']),pill(c['required_level']),pill(JUDGE[c['id']],STONE[JUDGE[c['id']]])] for c in sorted(CONDS,key=lambda c:(order.index(JUDGE[c['id']]) if JUDGE[c['id']] in order else 9)*-1 if False else order.index(JUDGE[c['id']]) if JUDGE[c['id']] in order else 9,reverse=False)]
  rows=sorted(rows,key=lambda r:0 if '合格' in r[5] and 't-success' in r[5] else -1)
  b+=block('テスト条件',tbl(['ID','宣言','対象','確かめること','求めるレベル','状態'],rows))
  if EXTRA: b+=block('余り（条件に無い報告）',tbl(['報告の条件 ID','テスト','理由'],[[f'<span class="tc t-fail">{E(x["condition"])}</span>',f'<span class="no">{E(x["test"])}</span>','廃止した条件' if x['condition'] in drift.RETIRED else '不明な ID'] for x in EXTRA]))
  b+=f'<details class="fold"><summary>報告の JSON（テスト実装が出したもの）</summary><div class="fbody"><pre class="code">{E(json.dumps(REPORT,ensure_ascii=False,indent=1))}</pre></div></details>'
  return b
def p_sim():
  SST={'合格':'t-success','ずれ':'t-fail','対応の欠け':'t-fail','確かめ直し':'t-caution','レビュー':'t-caution'}
  def ok(x): return not [s for s in x['spec'] if s['status']!='合格'] and all(v=='合格' for v in x['judge'].values()) and not x['extra']
  def short(v):
    t=json.dumps(v,ensure_ascii=False)
    return E(t if len(t)<=90 else t[:88]+'…')
  def nbad(x): return len([s for s in x['spec'] if s['status']!='合格'])
  def tbad(x): return len([v for v in x['judge'].values() if v!='合格'])
  b='<header class="ph"><p class="kind">テスト</p><h1>シミュレーション</h1></header><p class="lead">承認済みの宣言を JSON Patch で1手ずつ書き換え、そのたびに道具の2つの検査を流した結果。</p>'
  rows=[[f'<button class="sbtn" data-go="{x["no"]}">{x["no"]}</button>',E(x['who']),f'<span class="txt">{E(x["what"])}</span>',pill(str(nbad(x)),'t-fail' if nbad(x) else 't-success'),pill(str(tbad(x)),'t-fail' if tbad(x) else 't-success'),pill('満たす','t-success') if ok(x) else pill('満たさない','t-fail')] for x in SIM]
  b+=block('手の一覧',tbl(['手','誰が','何をしたか','宣言どうし','テスト','終了基準'],rows))
  b+='<div class="stepper">'+''.join(f'<button class="sbtn" data-go="{x["no"]}">{x["no"]}</button>' for x in SIM)+'</div>'
  for x in SIM:
    d=f'<section class="simstep" data-no="{x["no"]}">'
    d+=tiles([('手',f'<b style="font-size:1.3rem">{x["no"]}</b>'),('誰が',E(x['who']) or '―'),('終了基準',pill('満たす','t-success') if ok(x) else pill('満たさない','t-fail'))])
    d+=f'<p class="lead" style="margin-top:.75rem">{E(x["what"])}</p>'
    if x['ops']: d+=block('変更（JSON Patch）',tbl(['操作','場所（JSON Pointer）','値'],[[pill(o['op']),f'<span class="no">{E(o["path"])}</span>',short(o.get('value',o.get('removed','')))] for o in x['ops']]))
    else: d+=block('変更（JSON Patch）','<p class="txt">宣言は変えていない</p>')
    sb=[s for s in x['spec'] if s['status']!='合格']
    d+=block('道具の検知：宣言どうし',tbl(['状態','検査','参照元','参照先','内容'],[[pill(s['status'],SST[s['status']]),E(s['check']),f'<span class="no">{E(s["from"])}</span>',f'<span class="no">{E(s["to"])}</span>',E(s['text'])] for s in sb]) if sb else pill('検知なし','t-success'))
    cm={c['id']:c for c in x['conds']}
    tb=[(k,v) for k,v in x['judge'].items() if v!='合格']
    d+=block('道具の検知：宣言とテスト実装',tbl(['テスト条件','確かめること','報告のハッシュ値','今のハッシュ値','状態'],[[f'<span class="tc {STONE[v]}">{E(k)}</span>',E(cm[k]['checks']),f'<span class="no">{x["rep"][k]["hash"] if k in x["rep"] else "―"}</span>',f'<span class="no">{cm[k]["hash"]}</span>',pill(v,STONE[v])] for k,v in tb]) if tb else pill('検知なし','t-success'))
    d+='<div class="snav">'+(f'<button class="sbtn" data-go="{x["no"]-1}">← 前の手</button>' if x['no']>0 else '<span></span>')+(f'<button class="sbtn" data-go="{x["no"]+1}">次の手 →</button>' if x['no']<len(SIM)-1 else '')+'</div>'
    b+=d+'</section>'
  return b
R={"domain":p_domain,"subdomain":p_sd,"context":p_bc,"aggregate":p_agg,"value_object":p_vo,"domain_service":p_ds,"use_case":p_uc}
nav='<div class="ng"><span class="nk">テスト</span><a href="#DRIFT" data-id="DRIFT">突き合わせ</a><a href="#SIM" data-id="SIM">シミュレーション</a></div>'+''.join(f'<div class="ng"><span class="nk">{KIND[k]}</span>'+''.join(f'<a href="#{i}" data-id="{i}">{E(dname(i))}</a>' for i,v in D.items() if v['kind']==k)+'</div>' for k in ORDER)
pages=''.join(f'<article class="page" id="{i}">{R[v["kind"]](v)}</article>' for i,v in D.items())+f'<article class="page" id="DRIFT">{p_drift()}</article>'+f'<article class="page" id="SIM">{p_sim()}</article>'
css=open(os.path.join(HERE,'tokens.css')).read()+open(os.path.join(HERE,'sample4.css')).read()
page=f'''<title>来店前注文の宣言</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;600;700&family=JetBrains+Mono:wght@400&display=swap">
<style>{css}</style>
<div class="shell"><aside class="side"><p class="brand">来店前注文</p><nav class="nav">{nav}</nav></aside>
<main class="main">{pages}</main></div>
<script>
const show=()=>{{const id=(location.hash||'#DOM-1').slice(1);const hit=[...document.querySelectorAll('.page')].some(p=>p.id===id);const cur=hit?id:'DOM-1';document.querySelectorAll('.page').forEach(p=>p.hidden=p.id!==cur);document.querySelectorAll('.nav a').forEach(a=>a.classList.toggle('on',a.dataset.id===cur));window.scrollTo(0,0)}};
addEventListener('hashchange',show);show();
const go=n=>{{document.querySelectorAll('.simstep').forEach(s=>s.hidden=s.dataset.no!=n);document.querySelectorAll('.stepper .sbtn').forEach(b=>b.classList.toggle('on',b.dataset.go==n));const st=document.querySelector('.stepper');if(st&&window.scrollY>st.offsetTop)st.scrollIntoView();}};
document.querySelectorAll('.sbtn').forEach(b=>b.addEventListener('click',()=>{{go(b.dataset.go);document.querySelector('.stepper').scrollIntoView({{behavior:'smooth'}})}}));go('0');
</script>'''
open(f'{OUT}/viewer.html','w').write(page); print(len(page))
