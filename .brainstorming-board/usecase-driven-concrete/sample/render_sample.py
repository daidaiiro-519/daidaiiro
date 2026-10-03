import json,html,sys,os
sys.path.insert(0,os.path.dirname(__file__))
from sample_data import D
OUT='/home/daidaiiro/workspace/daidaiiro/.brainstorming-board/usecase-driven-concrete/sample'
for k,v in D.items(): json.dump(v,open(f'{OUT}/decls/{k}.json','w'),ensure_ascii=False,indent=1)
E=html.escape
KIND={"domain":"ドメイン","subdomain":"サブドメイン","context":"区切られた文脈","use_case":"ユースケース","aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
ORDER=["domain","subdomain","context","use_case","aggregate","value_object","domain_service"]
def term(ctx,t):
  g=D.get(ctx,{}).get('glossary',[])
  for x in g:
    if x['id']==t: return x['word']
  return None
def dname(i):
  d=D.get(i)
  if not d: return None
  n=d['name']; w=term(d.get('context',''),n)
  return w or n
def idtag(i): return f'<span class="id">{E(i)}</span>'
def ref(i):
  n=dname(i)
  if n is None: return f'<span class="missing">{E(i)}（まだ無い）</span>'
  return f'<a class="ref" href="#{E(i)}">{E(n)}</a>{idtag(i)}'
def domref(i):
  dom=D['DOM-1']
  for key in ('values','success_criteria','stakeholders','quality'):
    for x in dom.get(key,[]):
      if x['id']==i: return f'<a class="ref" href="#DOM-1">{E(x.get("name") or x.get("who"))}</a>{idtag(i)}'
  return ref(i)
def row(label,body,note=None):
  n=f'<span class="note">{E(note)}</span>' if note else ''
  return f'<div class="row"><div class="label">{E(label)}{n}</div><div class="val">{body}</div></div>'
def fold(label,count,body):
  return f'<details class="fold"><summary>{E(label)}<span class="count">{count}</span></summary><div class="fbody">{body}</div></details>'
def ul(items): return '<ul>'+''.join(f'<li>{x}</li>' for x in items)+'</ul>'
OPS={"eq":"は{v}","ne":"は{v}ではない","ge":"は{v}以上","le":"は{v}以下","not_empty":"は空でない"}
def cond(d,c,names):
  def nm(t):
    if t in names: return names[t]
    return t
  def val(v):
    if isinstance(v,str) and v.startswith('TERM-'): return term(d.get('context',d['id']),v) or v
    if isinstance(v,str) and v in names: return names[v]
    return str(v)
  def one(c,cond_clause=False):
    t=nm(c['target'])+('の件数' if c.get('agg')=='count' else '')
    v=c.get('value','')
    isref=isinstance(v,str) and v in names
    op=OPS[c['op']]
    if isref and c['op']=='eq': op='は{v}と同じ'
    if cond_clause: op='が'+op[1:]
    return t+op.format(v=val(v))
  s=one(c)
  if c.get('if'): s=one(c['if'],True)+'なら、'+s
  return s
def head(d,extra=''):
  k=KIND[d['kind']]; ctx=d.get('context')
  eb=f'{k}'+(f' ・ {ref(ctx)}' if ctx else '')+extra
  return f'<p class="eyebrow">{eb}</p><h1>{E(dname(d["id"]))}{idtag(d["id"])}</h1>'
def p_domain(d):
  vals=sorted(d['values'],key=lambda x:not x['differentiator'])
  v=ul([f'{"<span class=chip>競合との違い</span> " if x["differentiator"] else ""}<b>{E(x["name"])}</b>{idtag(x["id"])}<br><span class="sub">{E(x["text"])}</span>' for x in vals])
  sc=f'<div class="scope"><div><span class="mini">作るもの</span>{ul([E(x) for x in d["scope"]["in"]])}</div><div><span class="mini">作らないもの</span>{ul([E(x) for x in d["scope"]["out"]])}</div></div>'
  sds=ul([f'{ref(s)} <span class="chip">{E(D[s]["category"])}</span>' for s in d['subdomains']])
  su=ul([f'<b>{E(x["name"])}</b>{idtag(x["id"])}<br><span class="sub">{E(x["text"])}</span>' for x in d['success_criteria']])
  b=head(d)+row('解決する課題',E(d['problem']))+row('提供価値',v)+row('範囲',sc)+row('サブドメイン',sds)+row('達成の基準',su)
  b+=fold('利害関係者と利益',len(d['stakeholders']),ul([f'<b>{E(x["who"])}</b>：{E(x["interest"])}' for x in d['stakeholders']]))
  b+=fold('品質の要求',len(d['quality']),ul([f'<b>{E(x["name"])}</b>{idtag(x["id"])}：{E(x["text"])}' for x in d['quality']]))
  b+=fold('競合',len(d['competitors']),ul([f'<b>{E(x["name"])}</b>：{E(x["difference"])}' for x in d['competitors']]))
  return b
def yn(b): return '<span class="yes">はい</span>' if b else '<span class="no">いいえ</span>'
def p_sd(d):
  bl=d['business_logic']
  b=head(d)+row('カテゴリー',f'<span class="chip strong">{E(d["category"])}</span>')+row('説明',E(d['description']))
  if d['serves_values']: b+=row('担う提供価値',' '.join(domref(x) for x in d['serves_values']))
  b+=row('業務ロジックの性質',f'<div class="qa">経緯を追う必要があるか {yn(bl["needs_tracking"])}　業務ルールが複雑か {yn(bl["complex_rules"])}　データの構造が複雑か {yn(bl["complex_data"])}</div>')
  b+=row('カテゴリーの理由',E(d['category_reason']['answer']))
  if d.get('sourcing'): b+=row('調達',E(d['sourcing']))
  if bl['rule_conditions']: b+=fold('業務ルールの根拠の条件',len(bl['rule_conditions']),ul([E(x) for x in bl['rule_conditions']]))
  return b
def p_bc(d):
  b=head(d)+row('目的',E(d['purpose']))+row('対応するサブドメイン',' '.join(f'{ref(s)} <span class="chip">{E(D[s]["category"])}</span>' for s in d['subdomains']))
  if d['relations']: b+=row('他の文脈との関係',ul([f'{ref(r["with"])}　<span class="chip">{E(r["pattern"])}</span>' for r in d['relations']]))
  if d['glossary']:
    kinds=[]
    for x in d['glossary']:
      if x['kind'] not in kinds: kinds.append(x['kind'])
    g=''.join(f'<div class="gk"><span class="mini">{E(k)}</span>'+''.join(f'<div class="term"><b>{E(x["word"])}</b><span>{E(x["definition"])}</span></div>' for x in d['glossary'] if x['kind']==k)+'</div>' for k in kinds)
    b+=row('用語集',g)
    av=[x for x in d['glossary'] if x['avoid']]
    b+=fold('使わない語',len(av),ul([f'<b>{E(x["word"])}</b> の代わりに使わない：{E("、".join(x["avoid"]))}' for x in av]))
  if d['relations']: b+=fold('関係の理由',len(d['relations']),ul([f'{ref(r["with"])}：{E(r["reason"])}' for r in d['relations']]))
  mem=[k for k,v in D.items() if v.get('context')==d['id'] or (v.get('scope',{}) or {}).get('context')==d['id']]
  b+=fold('この文脈の宣言（ツールが集める）',len(mem),ul([f'{E(KIND[D[m]["kind"]])}：{ref(m)}' for m in mem]))
  return b
def tname(d,t):
  w=term(d['context'],t); return w or t
def p_agg(d):
  names={s['id']:tname(d,s['name']) for s in d['state']}
  for c in d['commands']:
    for a in c['args']: names[a['id']]='指定された'+tname(d,a['name'])
  MUL={"1":"1つ","0..1":"なくてもよい"}
  def mul(m): return MUL.get(m, m.replace('..','〜')+'件' if '..' in m else m)
  ents={e['id']:tname(d,e['name']) for e in d['entities']}
  def ty(t): return ref(t) if t in D else (f'{E(ents[t])}<span class="id">{E(t)}</span>' if t in ents else E(t))
  st=ul([f'<b>{E(names[s["id"]])}</b>：{ty(s["type"])}（{E(mul(s["multiplicity"]))}）' for s in d['state']])
  b=head(d)+row('守る一貫性',E(d['description']))+row('状態',st)
  b+=row('不変条件',ul([E(cond(d,c,names)) for c in d['invariants']]))
  if d['commands']:
    cs=''
    for c in d['commands']:
      cs+=f'<div class="cmd"><b>{E(tname(d,c["name"]))}</b>{idtag(c["id"])}<div class="pp"><span class="mini">事前条件</span>{ul([E(cond(d,p,names)) for p in c["pre"]])}<span class="mini">事後条件</span>{ul([E(cond(d,p,names)) for p in c["post"]])}</div></div>'
    b+=row('コマンド',cs)
  if d['events']: b+=row('業務イベント',' '.join(f'<b>{E(tname(d,e["name"]))}</b>{idtag(e["id"])}' for e in d['events']))
  b+=fold('不変条件に違反する例',len(d['invariants']),ul([E(c['violation'])+' → 拒否' for c in d['invariants']]))
  for c in d['commands']:
    b+=fold(f'「{tname(d,c["name"])}」の引数と例',len(c['args'])+len(c['accepts'])+len(c['rejects']),
      '<span class="mini">引数</span>'+ul([f'{E(tname(d,a["name"]))}：{ref(a["type"]) if a["type"] in D else E(a["type"])}' for a in c['args']])+
      '<span class="mini">受け付ける例</span>'+ul([f'{E(x["before"])} に {E(x["args"])} → {E(x["after"])}' for x in c['accepts']])+
      '<span class="mini">拒否の例</span>'+ul([f'{E(x["before"])} → 「{E(tname(d,x["reason"]))}」' for x in c['rejects']]))
  for e in d['events']:
    b+=fold(f'「{tname(d,e["name"])}」の項目',len(e['fields']),ul([f'{E(f["name"])}（{E(names.get(f["from"],f["from"]))} から）' for f in e['fields']]))
  if d['entities']: b+=fold('内側のエンティティ',len(d['entities']),ul([f'<b>{E(tname(d,x["name"]))}</b>：{E("、".join(x["state"]))}' for x in d['entities']]))
  need={}
  for c in d['invariants']+[p for cmd in d['commands'] for p in cmd['pre']]:
    for t in [c['target']]+([c['if']['target']] if c.get('if') else []):
      need.setdefault(t,[]).append(c['id'])
  b+=fold('一貫性が要る状態（ツールが導く）',len(need),ul([f'{E(names.get(k,k))}：{E("、".join(sorted(set(v))))}' for k,v in need.items()]))
  return b
def p_vo(d):
  names={c['id']:c['name'] for c in d['components']}
  gdef=[x['definition'] for x in D[d['context']]['glossary'] if x['id']==d['name']]
  b=head(d)+(f'<p class="lead">{E(gdef[0])}</p>' if gdef else '')
  b+=row('構成する値',ul([f'<b>{E(c["name"])}</b>：{E(c["kind"])}（{E(c["digits"])}）・ 単位 {E(c["unit"])}' for c in d['components']]))
  if d['invariants']: b+=row('不変条件',ul([E(cond(d,c,names)) for c in d['invariants']]))
  if d['operations']: b+=row('操作',ul([f'<b>{E(o["name"])}</b>{idtag(o["id"])}：{E(o["text"])}' for o in d['operations']]))
  if d['invariants']: b+=fold('作れない値の例',len(d['invariants']),ul([E(c['violation'])+' → 作れない' for c in d['invariants']]))
  return b
def p_ds(d):
  b=head(d)+row('置く理由',f'<span class="chip strong">{E(d["reason"])}</span>')+row('参照する集約（変更しない）',' '.join(ref(x) for x in d['reads']))
  ops=''
  for o in d['operations']:
    ops+=f'<div class="cmd"><b>{E(o["name"])}</b>{idtag(o["id"])} <span class="sub">→ {ref(o["output"]) if o["output"] in D else E(o["output"])}</span><span class="mini">事後条件</span>{ul([E(cond(d,p,{})) for p in o["post"]])}</div>'
  b+=row('操作',ops)
  b+=fold('置く理由の説明',1,E(d['reason_text']))
  b+=fold('操作の入力',sum(len(o['inputs']) for o in d['operations']),ul([E(x) for o in d['operations'] for x in o['inputs']]))
  return b
def p_uc(d):
  sh={x['id']:x['who'] for x in d['stakeholders']}
  sc=d['scope']
  tags=f' ・ <span class="chip">{E(d["level"])}</span> ・ スコープ {ref(sc.get("context") or sc.get("domain"))}'
  b=f'<p class="eyebrow">ユースケース{tags}</p><h1>{E(d["name"])}{idtag(d["id"])}</h1>'
  b+=row('主アクター',E(d['primary_actor']['role']))
  b+=row('利害関係者と利益',ul([f'<b>{E(x["who"])}</b>：{E(x["interest"])}' for x in d['stakeholders']]))
  if d['preconditions']: b+=row('事前条件',ul([f'{E(x["text"])} <span class="sub">（{ref(x["established_by"])} で成り立つ）</span>' for x in d['preconditions']]))
  b+=row('最低保証',ul([f'{E(x["text"])} <span class="sub">守る相手：{E("、".join(sh[i] for i in x["protects"]))}</span>' for x in d['minimal_guarantees']]))
  b+=row('成功時保証',ul([f'{E(x["text"])} <span class="sub">満たす相手：{E("、".join(sh[i] for i in x["satisfies"]))}</span>' for x in d['success_guarantees']]))
  b+=row('トリガー',E(d['trigger']['event']))
  b+=row('主成功シナリオ','<ol class="steps">'+''.join(f'<li>{E(s["text"])}</li>' for s in d['main_success_scenario'])+'</ol>')
  b+=row('拡張',ul([f'<span class="lbl">{E(x["label"])}</span>{E(x["condition"])} <span class="chip">{E(x["ending"])}</span>' for x in d['extensions']]))
  b+=fold('拡張の処理',len(d['extensions']),''.join(f'<div class="ext"><span class="lbl">{E(x["label"])}</span>{E(x["condition"])}<ol>'+''.join(f'<li>{E(s)}</li>' for s in x['steps'])+f'</ol><span class="sub">→ {E(x["ending"])}</span></div>' for x in d['extensions']))
  b+=fold('支援アクター',len(d['supporting_actors']),ul([E(x['name']) for x in d['supporting_actors']]))
  b+=fold('寄与する達成の基準',len(d['contributes_to']),ul([domref(x) for x in d['contributes_to']]))
  aggs=[s['aggregate'] for s in d['main_success_scenario'] if s.get('aggregate')]
  b+=fold('変更する集約（ツールが手順から導く）',len(aggs),ul([ref(a) for a in dict.fromkeys(aggs)]))
  b+=fold('技術およびデータのバリエーション',len(d['variations']),ul([f'手順{x["step"]}：{E(x["varies"])}（{E("、".join(x["values"]))}）' for x in d['variations']]))
  lk=d['links']; b+=fold('関連情報',len(lk.get('quality',[]))+len(lk.get('open_issues',[])),ul([domref(x) for x in lk.get('quality',[])]+['未決定：'+E(x) for x in lk.get('open_issues',[])]))
  return b
R={"domain":p_domain,"subdomain":p_sd,"context":p_bc,"aggregate":p_agg,"value_object":p_vo,"domain_service":p_ds,"use_case":p_uc}
nav=''
for k in ORDER:
  items=[i for i,v in D.items() if v['kind']==k]
  nav+=f'<div class="ng"><span class="nk">{KIND[k]}</span>'+''.join(f'<a href="#{i}" data-id="{i}">{E(dname(i))}</a>' for i in items)+'</div>'
pages=''.join(f'<article class="page" id="{i}">{R[v["kind"]](v)}</article>' for i,v in D.items())
css=open(os.path.join(os.path.dirname(__file__),'sample.css')).read()
page=f'''<title>来店前注文の宣言</title>
<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;600;700&family=JetBrains+Mono:wght@400&display=swap">
<style>{css}</style>
<div class="wrap">
<header class="top"><p class="kicker">ユースケース駆動の宣言 ・ 描画の試し</p><p class="intro">論点3で決めた項目と見せ方で、架空の「来店前注文」の宣言を描画した。正しさを決める項目を先に見せ、残りは件数付きで畳んでいる。</p>
<nav class="nav">{nav}</nav></header>
<main>{pages}</main></div>
<script>
const show=()=>{{const id=(location.hash||'#DOM-1').slice(1);let hit=false;document.querySelectorAll('.page').forEach(p=>{{const on=p.id===id;p.hidden=!on;hit=hit||on}});if(!hit){{document.getElementById('DOM-1').hidden=false}}document.querySelectorAll('.nav a').forEach(a=>a.classList.toggle('on',a.dataset.id===(hit?id:'DOM-1')));window.scrollTo(0,0)}};
addEventListener('hashchange',show);show();
</script>'''
open(f'{OUT}/viewer.html','w').write(page)
print(len(page))
