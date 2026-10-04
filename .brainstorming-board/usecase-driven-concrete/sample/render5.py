# 宣言を、共通の部品（タイル ・ カード ・ 札 ・ 表 ・ 図）とデザイントークンで描画する
import json,html,sys,os,subprocess,itertools
HERE=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,HERE)
from data5 import D,RETIRED
import gen5 as g
from desc3 import DESC
import drift5 as drift, spec5 as spec_drift
SPEC=spec_drift.checks()
CONDS=drift.conditions()
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
KIND={"domain":"ドメイン（プロダクト）","subdomain":"サブドメイン","context":"区切られた文脈","use_case":"ユースケース","aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
ORDER=["domain","subdomain","context","use_case","aggregate","value_object","domain_service"]
CAT={"中核":"t-core","補完":"t-supporting","一般":"t-generic"}
def ctx_of(d): return d['header'].get('context') or (d['header'].get('scope') or {}).get('context')
def term(ctx,t): return g.word(t)
def dname(i):
  d=D.get(i)
  if not d: return None
  n=d['header']['name']; return g.word(n)
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
  return f'<span class="tc">{E(cid)}</span>'
def tblock(decl):
  cs=[c for c in CONDS if c['decl']==decl]
  if not cs: return ''
  return block('テスト条件',tbl(['ID','対象','確かめること','求めるレベル','ハッシュ値'],[[tchip(c['id']),E(c['label']),E(c['checks']),pill(c['required_level']),f'<span class="no">{c["hash"]}</span>'] for c in cs]))
def head(d,badges='',lead=''):
  return f'<header class="ph"><p class="kind">{KIND[d["kind"]]}{badges}</p><h1>{E(dname(d["id"]))}{idt(d["id"])}</h1></header>'+(f'<p class="lead">{E(lead)}</p>' if lead else '')
def yn(v): return '<span class="yes">はい</span>' if v else '<span class="no-ans">いいえ</span>'
def gen(t): return f'{E(t)}'
def raw(d): return f'<details class="fold"><summary>{lab("宣言の JSON")}</summary><div class="fbody"><pre class="code">{E(json.dumps(d,ensure_ascii=False,indent=1))}</pre></div></details>'
def sv(x): return E(g.show(x))
def ex_rows(decl,cmd,exs):
  """例を、前の状態 ・ 引数 ・ 後の状態 ・ 業務イベントの表にする"""
  rows=[]
  for o in exs:
    bf='、'.join(f'{E(g.qname(k,decl,cmd))}＝{sv(v)}' for k,v in o['before'].items())
    ag='、'.join(f'{E(g.word(g.find(cmd["args"],k)["name"]))}＝{sv(v)}' for k,v in o.get('args',{}).items()) or '―'
    af=g.after(cmd,o)
    at='、'.join(f'{E(g.qname(k,decl,cmd))}＝{sv(v)}' for k,v in af.items())
    ev=' '.join(pill(g.word(e['name']),'t-accent') for e in cmd['emits']) or '―'
    rows.append([tchip(f'{decl}.{cmd["id"]}.{o["id"]}'),bf,ag,at,ev])
  return rows
# ── ドメイン
def p_domain(d):
  h=d['header']; vp=d['value_proposition']
  uc={}
  for k,v in D.items():
    for sc in v.get('contributes_to',[]): uc.setdefault(sc,[]).append(k)
  b=head(d)+tiles([('課題',E(h['problem']))])
  vals=[card(E(v['name'])+(pill('★ 競合との違い','t-accent') if v['differentiator'] else ''),f'<span class="txt">{E(v["text"])}</span>'+''.join(f'<span class="ln"><span class="k">担う</span>{ref(s)} {pill(D[s]["classification"]["category"],CAT[D[s]["classification"]["category"]])}</span>' for s in d['subdomains'] if v['id'] in D[s]['serves_values']),'left-accent' if v['differentiator'] else 'left-neutral') for v in vp['values']]
  vals+=[card('競合 ・ '+E(c['name']),f'<span class="txt">{gen(g.competitor_text(c))}</span>','left-neutral') for c in vp['competitors']]
  b+=block('提供価値',cards(vals))
  def scrow(x):
    src='業務イベント' if x['source']=='event' else 'システムの外'
    return [f'<b>{E(x["name"])}</b>',gen(g.sc_text(x)),pill(src),pill(g.thr(x['threshold'])),pill(g.thr(x['ratio'])) if x.get('ratio') else '―',E(x['window']),' '.join(ref(u) for u in uc.get(x['id'],[])) or '<span class="missing">なし</span>']
  b+=block('達成の基準',tbl(['基準','組んだ文','測る対象','閾値','割合','期間','寄与するユースケース'],[scrow(x) for x in d['success_criteria']])+'<p class="txt">運用で測る。テスト条件にはしない。</p>')
  b+=block('範囲',cards([card('作るもの（In）',''.join(item(x) for x in g.scope_in(d))+'<span class="ln"><span class="k">組んだ文</span>サブドメインから組む</span>','left-accent'),card('作らないもの（Out）',''.join(item(x) for x in d['scope']['out']),'left-neutral')]))
  sh={x['id']:x['who'] for x in d['stakeholders']}
  nm=lambda x: E(sh.get(x) or next((y['name'] for y in d['design_scopes'] if y['id']==x),x))
  b+=block('設計スコープ',tbl(['高さ','名前','内側','外側'],[[pill(x['level']),f'<b>{E(x["name"])}</b>','、'.join([nm(y) for y in x.get('inside',[])]+[ref(c) for c in x.get('contexts',[])]),'、'.join(nm(y) for y in x['outside'])] for x in d['design_scopes']])+'<p class="txt">ユースケースは、システムの高さのスコープを対象に書く。アプリ ・ 画面 ・ サービスへの分け方はスコープの内側のことで、実装の定義が持つ。</p>')
  b+=block('サブドメイン',cards([card(ref(s)+pill(D[s]['classification']['category'],CAT[D[s]['classification']['category']]),f'<span class="txt">{E(D[s]["header"]["description"])}</span>') for s in d['subdomains']]))
  b+=block('利害関係者',tbl(['利害関係者','関心'],[[f'<b>{E(x["who"])}</b>',f'<span class="txt">{E(x["interest"])}</span>'] for x in d['stakeholders']]))
  return b+raw(d)
def p_sd(d):
  c=d['classification']; bl=d['business_logic']; dc=g.derived_category(c)
  b=head(d,' '+pill(c['category'],CAT[c['category']]),d['header']['description'])
  b+=block('カテゴリー',tiles([('カテゴリー（宣言）',pill(c['category'],CAT[c['category']])),('判断基準の答えから導いたカテゴリー',pill(dc,CAT[dc])),('競合との違いになるか',yn(c['competitive_advantage'])),('外部のサービスや製品があるか',yn(c['external_available'])),('自分たちで作るほうが簡単で費用も少ないか',yn(c['cheaper_to_build']))]+([('調達',E(c['sourcing']))] if c.get('sourcing') else [])))
  b+=block('業務ロジックの性質',tiles([('経緯を追う必要があるか',yn(bl['needs_tracking'])),('業務ルールが複雑か',yn(bl['complex_rules'])),('データの構造が複雑か',yn(bl['complex_data']))])+(f'<p class="ln"><span class="k">根拠の条件</span>{" ".join(pill(x) for x in bl["rule_conditions"])}</p>' if bl['rule_conditions'] else ''))
  if d['serves_values']: b+=block('担う提供価値',cards([card(E(v['name']),f'<span class="txt">{E(v["text"])}</span>','left-accent') for v in D['DOM-1']['value_proposition']['values'] if v['id'] in d['serves_values']]))
  return b+raw(d)
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
    b+=block('コンテキストマップ',f+tbl(['相手','連携のパターン','この文脈の側','使う場合'],[[ref(r['with']),pill(r['pattern'],'t-accent'),pill(r['direction']),E(r['case'])] for r in rel]))
  else:
    b+=block('対応するサブドメイン',' '.join(ref(s)+pill(D[s]['classification']['category'],CAT[D[s]['classification']['category']]) for s in h['subdomains']))
  if d.get('boundary'): b+=block('物理的な境界',tiles([('配置の単位',E(d['boundary']['kind'])),('所有するチーム',E(d['boundary']['owner']))]))
  if d.get('business_rules'): b+=block('業務ルール',tbl(['ID','組んだ文'],[[f'<span class="no">{E(x["id"])}</span>',gen(g.cond(x['condition']))] for x in d['business_rules']]))
  if d.get('quality'):
    b+=block('品質の要求',tbl(['ID','組んだ文','対象','閾値','割合','条件','非機能要求グレード','測り方'],[[f'<span class="no">{E(q["id"])}</span>',gen(g.qr_text(q)),E(g.cmd_label(q['target'])),pill(g.thr(q['threshold'])),pill(g.thr(q['ratio'])),E(q['condition']['period']+' ・ '+str(q['condition']['load']['value'])+q['condition']['load']['unit']),E(f'{q["grade"]["item"]} ・ レベル{q["grade"]["level"]}'),pill(q['method'])] for q in d['quality']]))
  ts=d['ubiquitous_language']['terms']
  if ts:
    kinds=list(dict.fromkeys(x['kind'] for x in ts))
    b+=block('ユビキタス言語',cards([card(E(k),''.join(item(x['word'],x['definition'],(f'<span class="ln"><span class="k">使わない語</span>{" ".join(f"<span class=avoid>{E(a)}</span>" for a in x["avoid"])}</span>' if x['avoid'] else '')) for x in ts if x['kind']==k)) for k in kinds]))
  mem=[k for k,v in D.items() if ctx_of(v)==d['id'] and v['kind']!='context']
  if mem: b+=block('この文脈の宣言',tbl(['種類','名前'],[[KIND[D[m]['kind']],ref(m)] for m in mem]))
  return b+raw(d)
def p_agg(d):
  h=d['header']; st=d['structure']; k=d['id']
  ents={e['id']:e for e in st['entities']}
  b=head(d,f' ・ {ref(h["context"])}')+tiles([('守る一貫性',gen(g.agg_consistency(d)))])
  rows=[]
  for s in st['state']:
    t=s['type']; ty=ref(t) if t in D else (E(g.word(ents[t]['name'])) if t in ents else E(t))
    rows.append([f'<b>{E(g.word(s["name"]))}</b>',ty,pill(g.mul_text(s['multiplicity'])),tchip(f'{k}.{s["id"]}.MAX') if s['multiplicity']['max'] not in (None,1) else ''])
    if t in ents:
      rows+=[[f'<span class="txt">└ {E(g.word(x["name"]))}</span>',ref(x['type']) if x['type'] in D else E(x['type']),pill('1つ'),''] for x in ents[t]['state']]
  b+=block('構造',tbl(['状態','型','個数','テスト条件'],rows))
  if d['invariants']:
    ir=[]
    for i in d['invariants']:
      ve=g.violation_example(d,i)
      ir.append([tchip(f'{k}.{i["id"]}'),gen(g.cond(i['condition'],k)),'、'.join(f'{E(g.qname(a,k))}＝{sv(v)}' for a,v in ve['before'].items()),' ・ '.join(E(g.word([c for c in d['commands'] if c['id']==x][0]['name'])) for x in ve['via']) or '<span class="missing">なし</span>'])
    b+=block('不変条件',tbl(['テスト条件','組んだ文','違反する状態（道具が組む）','至る操作'],ir))
  for c in d['commands']:
    cname=g.word(c['name'])
    args=' '.join(f'{E(g.word(x["name"]))}（{ref(x["type"])}）' for x in c['args']) or 'なし'
    evs=' '.join(pill(g.word(e['name']),'t-accent')+'<span class="txt">'+' ・ '.join(E(g.field_name(d,f)) for f in e['fields'])+'</span>' for e in c['emits'])
    pr=[]
    for p in c['preconditions']:
      ex=g.reject_example(d,c,p)
      exs='、'.join(f'{E(g.qname(a,k,c))}＝{sv(v)}' for a,v in list(ex['before'].items())+list(ex.get('args',{}).items())) if ex else '<span class="missing">組めない</span>'
      pr.append([tchip(f'{k}.{c["id"]}.{p["id"]}'),gen(g.cond(p['condition'],k,c)),pill(g.word(p['reject'])),exs+(' '+pill('手で書いた例') if ex and ex.get('manual') else ' '+pill('道具が組む'))])
    pre=[p for p in c['preconditions'] if p['condition']['op']=='eq']; post=[p for p in c['postconditions'] if p['condition']['op']=='eq']
    nodes=[];edges=[];seen=set()
    for a in pre:
      for z in post:
        if a['condition']['target']==z['condition']['target'] and str(a['condition']['value']).startswith('TERM-') and str(z['condition']['value']).startswith('TERM-'):
          s1=g.word(a['condition']['value']); s2=g.word(z['condition']['value'])
          for x in (s1,s2):
            if x not in seen: seen.add(x); nodes.append({"id":x,"label":x})
          edges.append({"from":s1,"to":s2,"label":cname})
          for ev in c['emits']: nodes.append({"id":ev['id'],"label":g.word(ev['name']),"role":"muted"}); edges.append({"from":s2,"to":ev['id'],"label":"業務イベント","dashed":True})
    fig=figure('agg-'+k+'-'+c['id'],{"direction":"LR","nodes":nodes,"edges":edges}) if nodes else ''
    sec=f'<section class="blk"><h2>コマンド「{E(cname)}」{helpbtn("コマンド")}</h2>'+fig+tiles([('引数',args)]+([('業務イベント（項目は from から組む）',evs)] if evs else []))
    if pr: sec+=f'<h3 class="sub">事前条件と拒否の例</h3>'+tbl(['テスト条件','組んだ文','拒否の理由','拒否の例'],pr)
    sec+=f'<h3 class="sub">事後条件</h3>'+tbl(['ID','組んだ文'],[[f'<span class="no">{E(p["id"])}</span>',gen(g.cond(p['condition'],k,c))] for p in c['postconditions']])
    if c.get('accept_examples'): sec+=f'<h3 class="sub">受け付ける例{helpbtn("例")}</h3>'+tbl(['テスト条件','前の状態','引数','後の状態（道具が導く）','業務イベント'],ex_rows(k,c,c['accept_examples']))
    b+=sec+'</section>'
  b+=tblock(k)
  return b+raw(d)
def p_vo(d):
  ctx=d['header']['context']; k=d['id']
  gdef=[x['definition'] for x in D[ctx]['ubiquitous_language']['terms'] if x['id']==d['header']['name']]
  b=head(d,f' ・ {ref(ctx)}',gdef[0] if gdef else '')
  def shape(c):
    if c.get('values'): return '値の一覧：'+' ・ '.join(g.word(v) for v in c['values'])
    parts=[]
    if c.get('digits'): parts.append(f'{c["digits"]}桁')
    if c.get('precision'): parts.append(c['precision'])
    return ' ・ '.join(parts) or '―'
  b+=block('構成する値',tbl(['名前','値の種類','桁 ・ 精度 ・ 値','単位'],[[f'<b>{E(c["name"])}</b>',pill(c['kind']),E(shape(c)),E(c.get('unit','―'))] for c in d['components']]))
  inv=[[tchip(f'{k}.{i["id"]}'),gen(g.cond(i['condition'],k)),pill(str(g.impossible(i['condition']))),' ・ '.join(f'{x}（{kk}）' for x,kk in g.bounds(i['condition']))] for c in d['components'] for i in c['invariants']]
  if inv: b+=block('不変条件',tbl(['テスト条件','組んだ文','作れない値（道具が組む）',f'境界値（道具が導く）'],inv))
  if d['operations']:
    b+=block('操作',tbl(['テスト条件','操作','引数','結果','例'],[[tchip(f'{k}.{o["id"]}.{x["id"]}'),f'<b>{E(g.word(o["name"]))}</b>',' ・ '.join(ref(a) for a in o['args']),ref(o['result']),E(f'{x["self"]} を {x["args"][0]} で「{g.word(o["name"])}」→ {x["result"]}')] for o in d['operations'] for x in o['accept_examples']]))
  b+=tblock(k)
  return b+raw(d)
def p_ds(d):
  h=d['header']; ctx=h['context']; k=d['id']
  b=head(d,' '+pill(h['reason'],'t-accent')+f' ・ {ref(ctx)}')+tiles([('置く理由（組んだ文）',gen(g.ds_reason(d))),('参照する集約（変更しない）',' ・ '.join(ref(x) for x in d['reads']))])
  for o in d['operations']:
    on=g.word(o['name'])
    nodes=[{"id":"op","label":on,"role":"focus"},{"id":"out","label":dname(o['output'])}]; at={"op":["b","r1"],"out":["c","r1"]}; edges=[{"from":"op","to":"out"}]
    for i,x in enumerate(o['inputs']): nodes.append({"id":f"in{i}","label":g.input_name(x)}); at[f"in{i}"]=["a",f"r{i}"]; edges.append({"from":f"in{i}","to":"op"})
    f=figure('ds-'+k+'-'+o['id'],{"layout":"grid","direction":"LR","nodes":nodes,"edges":edges,"grid":{"cols":["a","b","c"],"rows":sorted({v[1] for v in at.values()}),"at":at}})
    b+=f'<section class="blk"><h2>操作「{E(on)}」{helpbtn("操作")}</h2>{f}'
    b+='<h3 class="sub">入力</h3>'+tbl(['ID','組んだ名前','元にする状態','型'],[[f'<span class="no">{E(x["id"])}</span>',gen(g.input_name(x)),f'<span class="no">{E(x["from"]["target"])}</span>'+(' '+pill('合計') if x['from'].get('agg')=='sum' else ''),ref(x['type'])] for x in o['inputs']])
    b+='<h3 class="sub">事後条件</h3>'+tbl(['テスト条件','組んだ文'],[[tchip(f'{k}.{o["id"]}.{p["id"]}'),gen(g.cond(p['condition'],k))] for p in o['postconditions']])+'</section>'
  b+=tblock(k)
  return b+raw(d)
def p_uc(d):
  h=d['header']; sc=d['scenario']; ctx=h['scope'].get('context'); k=d['id']; nums=g.number(d)
  role={h['primary_actor']:'primary','システム':'system'}; role.update({a:'supporting' for a in sc['supporting_actors']})
  RL={'primary':'主','supporting':'支援','system':''}
  act=lambda a: f'<span class="act {role.get(a,"system")}">'+(f'<span class="ar">{RL[role.get(a,"system")]}</span>' if RL[role.get(a,'system')] else '')+f'{E(a)}</span>'
  dom={x['id']:x for x in D['DOM-1']['stakeholders']}
  sh={x['id']:x for x in d['stakeholders']}; gu=d['guarantees']
  who=lambda i: dom[sh[i]['who'].split('.')[-1]]['who']
  whos=lambda ids: pills([who(i) for i in ids])
  def pref(rr):
    x,decl,cmd=g.item(rr); a=rr.split('.')[0]
    return f'{ref(a)}「{E(g.word(cmd["name"]))}」の{"事前条件" if x["id"].startswith("PRE") else "事後条件"}<span class="no">{E(x["id"])}</span>'
  def links(s):
    o=''
    if s.get('reply'): o+=f'<span class="ln"><span class="k">戻りメッセージ</span>{E(g.reply_text(s))}</span>'
    if s.get('invokes'):
      a=s['invokes'].split('.')[0]; o+=f'<span class="ln"><span class="k">invokes</span>{ref(a)} <span class="no">{E(s["invokes"])}</span></span>'
    if s.get('checks'): o+=f'<span class="ln"><span class="k">checks</span>'+' '.join(f'<span class="no">{E(r)}</span>' for r in s['checks'])+'</span>'
    if s.get('keeps'): o+=f'<span class="ln"><span class="k">守る最低保証</span>{pills([m["name"] for m in gu["minimal"] if m["id"] in s["keeps"]],"t-minimal")}</span>'
    if s.get('quality'): o+=f'<span class="ln"><span class="k">品質の要求</span>'+'、'.join(E(g.qr_text(g.item(q)[0])) for q in s['quality'])+' '+''.join(tchip(f'{k}.{s["id"]}.{q.split(".")[-1]}') for q in s['quality'])+'</span>'
    return o
  parts=[h['primary_actor'],'システム']+sc['supporting_actors']; msgs=[]; groups=[]
  def add(s):
    frm=s['actor']; to=s.get('to') or frm
    msgs.append({"from":frm,"to":to,"label":f"{nums[s['id']]} {g.step_short(s)}"})
    if s.get('reply'): msgs.append({"from":to,"to":frm,"label":g.reply_text(s),"kind":"return"})
  for s in sc['steps']: add(s)
  TH={"font.size-small":14,"font.size":15,"chart.exchange-col-w":220,"chart.pad":6,"chart.exchange-row-h":40}
  seqm=chart('uc-'+k+'-main','exchange',{"participants":parts,"steps":list(msgs),"groups":[],"theme":TH})
  msgs.clear()
  for s in sc['steps']:
    add(s)
    for x in s['extensions']:
      st0=len(msgs)
      for t in x['steps']: add(t)
      groups.append({"label":'break' if x['ending']=='失敗' else 'opt',"cases":[{"name":f"{nums[x['id']]} → {g.ending_text(x,nums)}","span":[st0,len(msgs)-1]}]})
  seq=chart('uc-'+k,'exchange',{"participants":parts,"steps":msgs,"groups":groups,"theme":{"font.size-small":14,"font.size":15,"chart.exchange-col-w":220,"chart.pad":6,"chart.exchange-row-h":40}})
  scs=[x for x in D['DOM-1']['success_criteria'] if x['id'] in d['contributes_to']]
  trig=[s for s in sc['steps'] if s['id']==h['trigger_step']][0]
  sys_=[x for x in D['DOM-1']['design_scopes'] if 'DOM-1.'+x['id']==h['scope']['system']][0]
  b=head(d,' '+pill(h['level'])+f' ・ スコープ <a class="ref" href="#DOM-1">{E(sys_["name"])}</a> ・ 文脈 {ref(ctx)}')
  b+=tiles([('主アクター',act(h['primary_actor'])),('支援アクター',''.join(act(a) for a in sc['supporting_actors'])),('トリガー',gen(g.step_text(trig))),('寄与する達成の基準',' '.join(f'<a class="ref" href="#DOM-1">{E(x["name"])}</a>' for x in scs))])
  sline=lambda x: f'<span class="ln"><span class="k">成り立たせる事後条件</span>{"、".join(pref(r) for r in x.get("established_by",[])) or "<span class=missing>なし</span>"}</span>'
  sgc=card(lab('成功時保証'),''.join(item(x['name'],g.sg_text(x),f'<span class="ln">{whos(x["satisfies"])}</span>'+sline(x)) for x in gu['success']),'top-success')
  mgc=card(lab('最低保証'),''.join(item(x['name'],g.mg_text(x),f'<span class="ln">{whos(x["protects"])}</span>') for x in gu['minimal']),'top-minimal')
  b+='<div style="height:.75rem"></div>'+cards([sgc,mgc])
  b+=block('事前条件',tbl(['組んだ文','成り立たせるユースケース','これで起こらない拒否'],[[gen(g.pre_text(x)),ref(x['established_by']),'、'.join(pref(r) for r in x.get('ensures',[])) or '―'] for x in d['preconditions']]))
  def who_to(t): return act(t['actor'])
  def body(t):
    s=g.step_text(t); p=t['actor']+'は、'
    return s[len(p):] if s.startswith(p) else s
  def meta(t,extra=''):
    return f'<div class="meta"><span class="no">{E(t["id"])} ・ {E(t["kind"])}</span>{links(t)}{extra}</div>'
  def line(t,no,cls='step'):
    rep=f'<span class="reply">← {E(g.reply_text(t))}</span>' if t.get('reply') else ''
    return f'<li class="{cls}"><span class="sn">{E(no)}</span><div class="sb"><div class="sh">{who_to(t)}<b>{gen(body(t))}</b>{rep}</div>{meta(t)}</div></li>'
  fl=''
  for s in sc['steps']:
    fl+=line(s,nums[s['id']])
    for x in s['extensions']:
      fail=x['ending']=='失敗'
      sub=''.join(line(t,nums[t['id']],'sub') for t in x['steps'])
      end=(f'<li class="end fail"><span class="sn">✕</span><div class="sb"><b>失敗で終わる</b><span class="note">最低保証が成り立つ</span></div></li>' if fail
           else f'<li class="end back"><span class="sn">↩</span><div class="sb"><b>手順{E(nums[x["ending"]])}へ戻る</b></div></li>')
      xm=f'<div class="meta"><span class="no">{E(x["id"])} ・ {E(x["condition_kind"])}</span>'+(f'<span class="ln"><span class="k">扱う拒否</span>{"、".join(pref(r) for r in x["handles"])}</span>' if x.get('handles') else '')+(f'<span class="ln"><span class="k">fails</span>{" ".join(E(r) for r in x["fails"])}</span>' if x.get('fails') else '')+f'<span class="ln">{tchip(k+"."+x["id"])}</span></div>'
      fl+=f'<li class="ext"><div class="xh"><span class="xl">{E(nums[x["id"]])}</span><b>{gen(g.ext_text(x))}</b></div>{xm}<ol class="xs">{sub}{end}</ol></li>'
  fl+=f'<li class="end ok"><span class="sn">✓</span><div class="sb"><b>成功で終わる</b><span class="note">成功時保証が成り立つ</span><div class="meta">{tchip(k+".M")}</div></div></li>'
  b+=f'<section class="blk"><h2>主成功シナリオと拡張{helpbtn("主成功シナリオ")}</h2><label class="mt"><input type="checkbox" id="mt-{k}" class="mtog"> 宣言の欄を表示（ID ・ invokes ・ テスト条件）</label><ol class="flow">{fl}</ol></section>'
  b+=f'<details class="fold near"><summary>主成功シナリオのシーケンス図{helpbtn("シーケンス図")}</summary><div class="fbody">{seqm}</div></details>'
  b+=f'<details class="fold near"><summary>拡張を含むシーケンス図</summary><div class="fbody">{seq}</div></details>'
  srows=[[f'<b>{E(who(i))}</b>',f'<span class="txt">{E(x["interest"])}</span>','、'.join(nums[s['id']] for s in sc['steps'] if i in s.get('serves',[])) or '<span class="missing">なし</span>',pills([m['name'] for m in gu['success'] if i in m['satisfies']],'t-success')+pills([m['name'] for m in gu['minimal'] if i in m['protects']],'t-minimal')] for i,x in sh.items()]
  b+=block('利害関係者と利益',tbl(['利害関係者','利益','守る手順','守る保証'],srows))
  vr=[[nums[s['id']],E(v['varies']),pills(v['values'])] for s in sc['steps'] for v in s.get('variations',[])]
  if vr: b+=block('技術およびデータのバリエーション',tbl(['手順','違い','値'],vr))
  if d.get('open_issues'): b+=block('未決定事項',tbl(['未決定事項'],[[E(x)] for x in d['open_issues']]))
  b+=tblock(k)
  return b+raw(d)
def p_drift():
  import collections
  b='<header class="ph"><p class="kind">テスト</p><h1>テスト条件と宣言どうしの検査</h1></header><p class="lead">宣言から道具が出すもの。テストの合否はここに無い（テストの実行器が判定し、保存しない）。</p>'
  scnt=collections.Counter(x['status'] for x in SPEC)
  b+='<h2 class="sec">宣言どうし</h2>'+tiles([(k,f'<b style="font-size:1.3rem">{scnt.get(k,0)}</b>') for k in ['ずれ','対応の欠け','確かめ直し','レビュー','合格']])
  CAT={'structure':'構造で検査','link':'対応の欄で検査','change':'上流の変更','review':'人のレビュー'}
  SST={'合格':'','ずれ':'t-warn','対応の欠け':'t-warn','確かめ直し':'','レビュー':''}
  def at(i):
    if i.startswith('TERM-'): return f'「{E(g.word(i))}」'
    return ref(i.split('.')[0])+(f' <span class="no">{E(".".join(i.split(".")[1:]))}</span>' if '.' in i else '') if i and i.split('.')[0] in D else E(i)
  srow=[[pill(x['status'],SST[x['status']]),E(CAT[x['category']]),E(x['check']),at(x['from']),'、'.join(at(t) for t in x['to'].split('・')) if x['to'] else '',E(x['text'])] for x in sorted(SPEC,key=lambda x:(x['status']=='合格',list(CAT).index(x['category'])))]
  b+=block('宣言どうしの検査',tbl(['状態','分け方','検査','参照元','参照先','内容'],srow))
  b+='<h2 class="sec">宣言とテスト</h2>'
  b+=block('テスト条件',tbl(['ID','宣言','対象','確かめること','求めるレベル','ハッシュ値'],[[tchip(c['id']),ref(c['decl']),E(c['label']),E(c['checks']),pill(c['required_level']),f'<span class="no">{c["hash"]}</span>'] for c in CONDS]))
  b+=block('突き合わせ',tbl(['何を','どこで見られるか'],[
    ['テストは走ったときに、上の ID ・ ハッシュ値 ・ レベルを記録ファイルへ1行ずつ追記する（記録の契約）。道具は宣言と記録だけを照らし、欠け ・ 余り ・ 古い ・ レベル違いを出す','<a class="ref" href="https://claude.ai/artifact/MGX9MpSk6MtnCF8QpWqDdh" target="_blank" rel="noopener">テスト条件 ID の突き合わせ</a>'],
    ['Python と Go のテストを実際に実行し、9手のコミットごとに CI を流した記録','<a class="ref" href="https://claude.ai/artifact/HwxXEHAKFcGig7f7UamAxY" target="_blank" rel="noopener">実行の記録</a>']]))
  return b
R={"domain":p_domain,"subdomain":p_sd,"context":p_bc,"aggregate":p_agg,"value_object":p_vo,"domain_service":p_ds,"use_case":p_uc}
nav='<div class="ng"><span class="nk">テスト</span><a href="#DRIFT" data-id="DRIFT">テスト条件と検査</a><a href="https://claude.ai/artifact/MGX9MpSk6MtnCF8QpWqDdh" target="_blank" rel="noopener">突き合わせ ↗</a><a href="https://claude.ai/artifact/HwxXEHAKFcGig7f7UamAxY" target="_blank" rel="noopener">実行の記録 ↗</a></div>'+''.join(f'<div class="ng"><span class="nk">{KIND[k]}</span>'+''.join(f'<a href="#{i}" data-id="{i}">{E(dname(i))}</a>' for i,v in D.items() if v['kind']==k)+'</div>' for k in ORDER)
pages=''.join(f'<article class="page" id="{i}">{R[v["kind"]](v)}</article>' for i,v in D.items())+f'<article class="page" id="DRIFT">{p_drift()}</article>'
css=open(os.path.join(HERE,'tokens.css')).read()+open(os.path.join(HERE,'sample4.css')).read()
page=f'''<title>モバイルオーダーの宣言</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;600;700&family=JetBrains+Mono:wght@400&display=swap">
<style>{css}</style>
<div class="shell"><aside class="side"><p class="brand">{E(D["DOM-1"]["header"]["name"])}</p><nav class="nav">{nav}</nav></aside>
<main class="main">{pages}</main></div>
<script>
const show=()=>{{const id=(location.hash||'#DOM-1').slice(1);const hit=[...document.querySelectorAll('.page')].some(p=>p.id===id);const cur=hit?id:'DOM-1';document.querySelectorAll('.page').forEach(p=>p.hidden=p.id!==cur);document.querySelectorAll('.nav a').forEach(a=>a.classList.toggle('on',a.dataset.id===cur));window.scrollTo(0,0)}};
addEventListener('hashchange',show);show();
const go=n=>{{document.querySelectorAll('.simstep').forEach(s=>s.hidden=s.dataset.no!=n);document.querySelectorAll('.stepper .sbtn').forEach(b=>b.classList.toggle('on',b.dataset.go==n));const st=document.querySelector('.stepper');if(st&&window.scrollY>st.offsetTop)st.scrollIntoView();}};
document.querySelectorAll('.sbtn').forEach(b=>b.addEventListener('click',()=>{{go(b.dataset.go);document.querySelector('.stepper').scrollIntoView({{behavior:'smooth'}})}}));go('0');
</script>'''
open(f'{OUT}/viewer.html','w').write(page); print(len(page))
