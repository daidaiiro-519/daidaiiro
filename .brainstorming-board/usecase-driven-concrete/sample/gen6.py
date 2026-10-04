# 欄から文を組み、例と境界値を導く（論点7の形）。宣言の自由文は読まない（説明として画面に出すだけ）。
# 語は用語集から引く。ユースケースは用語集の語だけで書かれ、内部の ID を持たない。
import data6
D=data6.D
def use(state):
  global D
  D=state
def glossary(): return [v for v in D.values() if v['kind']=='glossary'][0]
def terms():
  """語の ID → 語と、最初の意味の種類 ・ 定義（文を組むのに使う）"""
  out={}
  for t in glossary()['terms']:
    m=t['meanings'][0]; out[t['id']]={**{k:m[k] for k in m if k!='id'},"id":t['id'],"word":t['word'],"avoid":t['avoid'],"meanings":t['meanings']}
  return out
def word(t):
  x=terms().get(t); return x['word'] if x else t
def dname(i):
  d=D.get(i)
  if not d: return None
  return word(d['header']['name'])
def find(lst,i): return [x for x in lst if x['id']==i][0]
def number(uc):
  """手順の番号と拡張のラベルを、並び順から振る。ID は変えない"""
  out={}
  for n,s in enumerate(uc['scenario']['steps'],1):
    out[s['id']]=str(n)
    for j,x in enumerate(s['extensions']):
      lab=f'{n}{"abcdefghij"[j]}'; out[x['id']]=lab
      for m,t in enumerate(x['steps'],1): out[t['id']]=f'{lab}{m}'
  return out
# ── 参照から名前を引く
def state_name(agg,sid):
  a=D[agg]
  if sid.startswith('ENT-'):
    e,es=sid.split('.'); ent=find(a['structure']['entities'],e); return word(ent['name'])+'の'+word(find(ent['state'],es)['name'])
  return word(find(a['structure']['state'],sid)['name'])
def qname(ref,decl=None,cmd=None):
  """宣言の中の参照（ST-2 ・ ARG-1 ・ AGG-1.ST-4 ・ TERM-11 など）を、用語の語にする"""
  if isinstance(ref,(int,float)): return str(ref)
  if ref=='RESULT': return '結果'
  if ref.startswith('TERM-'):
    ws=[word(x) for x in ref.split('.')]; out=ws[0]
    for w in ws[1:]: out=w if w.startswith(out) else f'{out}の{w}'
    return out
  p=ref.split('.')
  if p[0] in D and len(p)>=2 and D[p[0]]['kind']=='aggregate':
    n=state_name(p[0],'.'.join(p[1:])); a=dname(p[0])
    return n if n.startswith(a) else f'{a}の{n}'
  if p[0] in D and len(p)==1: return dname(p[0])
  if decl and D[decl]['kind']=='aggregate':
    if ref.startswith('ST-') or ref.startswith('ENT-'): return state_name(decl,ref)
    if ref.startswith('ARG-') and cmd: return '指定された'+word(find(cmd['args'],ref)['name'])
  if decl and D[decl]['kind']=='value_object' and ref.startswith('CMP-'): return find(D[decl]['components'],ref)['name']
  return ref
OPS={"eq":"は{v}","ne":"は{v}ではない","ge":"は{v}以上","le":"は{v}以下","gt":"は{v}より大きい","not_empty":"は空でない","empty":"は空"}
def val(v,decl,cmd):
  if isinstance(v,dict) and 'before' in v: return '実行前の'+qname(v['before'],decl,cmd)+('の件数' if v.get('agg')=='count' else '')
  if isinstance(v,dict) and 'call' in v:
    vo,op=v['call'].split('.'); o=find(D[vo]['operations'],op)
    a=[val(x,decl,cmd) for x in v['args']]; return f'{a[0]}と{a[1]}を「{word(o["name"])}」で求めた値'
  return qname(v,decl,cmd)
def cond(c,decl=None,cmd=None):
  def one(c,clause=False):
    t=qname(c['target'],decl,cmd)+('の件数' if c.get('agg')=='count' else '')+('の合計' if c.get('agg')=='sum' else '')
    v=c.get('value',''); op=OPS[c['op']]
    if c['op']=='eq' and (isinstance(v,dict) or (isinstance(v,str) and (not v.startswith('TERM-') or terms().get(v,{}).get('kind')!='状態の値'))): op='は{v}と同じ'
    if clause: op='が'+op[1:]
    return t+op.format(v=val(v,decl,cmd) if v!='' else '')
  s=one(c); return (one(c['if'],True)+'なら、'+s) if c.get('if') else s
def clause(c,decl=None,cmd=None):
  """「XがYと同じ」の形の節（妥当性確認の文 ・ 拡張の条件に使う）"""
  t=cond(dict(c,**{}),decl,cmd)
  i=t.find('は'); return t[:i]+'が'+t[i+1:] if i>0 else t
def item(ref):
  """「AGG-1.CMD-1.BR-2」「BC-1.BR-1」「BC-1.QR-1」などの項目と、その宣言 ・ コマンドを返す"""
  p=ref.split('.'); d=D[p[0]]
  if d['kind']=='aggregate':
    c=find(d['commands'],p[1])
    if len(p)==2: return c,p[0],None
    return find(c['business_rules']+c['state_changes'],p[2]),p[0],c
  if d['kind']=='context':
    if p[1].startswith('X-'):
      x=find(d['context_map']['relations'],p[1])
      if len(p)==2: return x,p[0],None
      o=find(x['operations'],p[2])
      if len(p)==3: return o,p[0],x
      return find(o['failures'],p[3]),p[0],o
    return find(d['business_rules'] if p[1].startswith('BR') else d['quality'],p[1]),p[0],None
  if d['kind']=='domain_service': return find(d['operations'],p[1]),p[0],None
  if d['kind']=='other_requirements':
    return find(d['business_rules']+d['quality']+d['technology'],p[1]),p[0],None
  if d['kind']=='use_case':
    if len(p)==1: return d,p[0],None
    return uc_part(d,'.'.join(p[1:])),p[0],None
  return None,p[0],None
def uc_part(uc,rid):
  for x in uc['preconditions']+uc['guarantees']['minimal']+uc['guarantees']['success']:
    if x['id']==rid: return x
  for s in all_steps(uc):
    if s['id']==rid: return s
  for s in uc['scenario']['steps']:
    for x in s['extensions']:
      if x['id']==rid: return x
  return None
def g_item(ref):
  x,_,_=item(ref); return x['condition']
def item_cond(ref):
  x,decl,cmd=item(ref); return cond(x['condition'],decl,cmd)
def obj_verb(obj,verb):
  w=word(verb); o=word(obj)
  return f'{o}の{w}' if 'を' in w else f'{o}を{w}'
def cmd_label(ref):
  p=ref.split('.')
  if D[p[0]]['kind']=='domain_service': return dname(p[0])
  c=find(D[p[0]]['commands'],p[1]); w=word(c['name']); a=dname(p[0])
  return f'{a}の{w}' if 'を' in w else f'{a}を{w}'
def joinw(ids): return 'と'.join(word(x) for x in ids)
# ── ユースケースの文
def step_text(s):
  a=s['actor']
  if s['kind']=='相互作用':
    f=terms()[s['verb']].get('form','{to}に{data}を'+word(s['verb']))
    return f'{a}は、'+f.format(to=s['to'],data=joinw(s['data']))
  if s['kind']=='妥当性確認': return f'{a}は、'+'、'.join(clause(g_item(r)) for r in s['checks'])+'であることを確かめる'
  if s['kind']=='内部の状態変化': return f'{a}は、{obj_verb(s["object"],s["verb"])}'
  if s['kind']=='サブユースケースの呼び出し': return f'{a}は、「{dname(s["calls"])}」を行う'
  return a
def step_short(s):
  if s['kind']=='相互作用': return terms()[s['verb']].get('form','{data}を'+word(s['verb'])).replace('{to}に','').format(data=joinw(s['data']))
  if s['kind']=='妥当性確認': return '確かめる'
  if s['kind']=='内部の状態変化': return obj_verb(s['object'],s['verb'])
  if s['kind']=='サブユースケースの呼び出し': return f'「{dname(s["calls"])}」を行う'
  return ''
def reply_text(s): return joinw(s['reply']) if s.get('reply') else ''
def ext_text(x):
  k=x['condition_kind']
  if k=='業務ルールの拒否':
    return '、'.join('「'+word(r)+'」' for r in x['reasons'])+'で受け付けられなかった：'
  if k=='妥当性確認の失敗': return '、'.join(clause(g_item(r)) for r in x['fails'])+'でなかった：'
  if k=='支援アクターの失敗':
    if x.get('reasons'): return '、または'.join(word(r) for r in x['reasons'])+'：'
    return f'{x["actor"]}が応答しなかった、または誤った応答を返した：'
  return ''
def ending_text(x,nums): return '失敗' if x['ending']=='失敗' else f'{nums[x["ending"]]}へ戻る'
def pre_text(p): return cond(p['condition'])
def sg_text(g): return post_text(g['condition'])
def mg_text(g): return cond(g['condition'])
def all_steps(uc):
  for s in uc['scenario']['steps']:
    yield s
    for x in s['extensions']:
      for t in x['steps']: yield t
# ── 集約
def agg_consistency(a):
  ids=[]
  for i in a['invariants']: ids+= [i['condition']['target']]+([i['condition']['if']['target']] if i['condition'].get('if') else [])
  for c in a['commands']:
    for p in c['business_rules']: ids.append(p['condition']['target'])
  ids=[x for x in dict.fromkeys(ids) if x.startswith('ST-')]
  return ' ・ '.join(state_name(a['id'],x) for x in ids)+'の一貫性を守る'
def field_name(a,f): return state_name(a['id'],f['from'])
def mul_text(m):
  lo,hi=m['min'],m['max']
  if lo==hi==1: return '1つ'
  if (lo,hi)==(0,1): return '0か1つ'
  return f'{lo}件以上' if hi is None else f'{lo}〜{hi}件'
# ── 例を導く
def other_value(state_type,v):
  vo=D.get(state_type)
  if vo and vo['kind']=='value_object':
    vals=[x for c in vo['components'] for x in c.get('values',[])]
    for x in vals:
      if x!=v: return x
  return '（別の値）'
def violate(a,c):
  t=c['target']; s=[x for x in a['structure']['state'] if x['id']==t]
  ty=s[0]['type'] if s else None
  v=c.get('value')
  if c['op']=='eq': return other_value(ty,v)
  if c['op']=='not_empty': return '空'
  if c['op']=='empty': return '（空でない）'
  if isinstance(v,(int,float)):
    n=v-1 if c['op']=='ge' else v+1
    return {"count":n} if c.get('agg')=='count' else n
  return None
def satisfy(c):
  v=c.get('value')
  if c['op']=='eq': return v
  if c['op']=='not_empty': return '（空でない）'
  if isinstance(v,(int,float)): return {"count":v} if c.get('agg')=='count' else v
  return None
def reject_example(a,cmd,p):
  """偽になる事前条件がちょうど1つになる、拒否の例。組めないときは宣言の例を使う"""
  if p.get('example'): return dict(p['example'],manual=True)
  v=violate(a,p['condition'])
  if v is None: return None
  before={p['condition']['target']:v}
  for q in cmd['business_rules']:
    if q is not p: before[q['condition']['target']]=satisfy(q['condition'])
  return {"before":before,"args":{}}
def violation_example(a,inv):
  c=inv['condition']; before={}
  if c.get('if'): before[c['if']['target']]=satisfy(c['if'])
  before[c['target']]=violate(a,c)
  return {"before":before,"via":inv.get('via',[])}
def bounds(c):
  v=c.get('value')
  if not isinstance(v,(int,float)): return []
  if c['op']=='ge': return [(v,'有効'),(v-1,'無効')]
  if c['op']=='le': return [(v,'有効'),(v+1,'無効')]
  return []
def impossible(c):
  b=[x for x,k in bounds(c) if k=='無効']; return b[0] if b else None
def call(v,before,args):
  vo,op=v['call'].split('.'); o=find(D[vo]['operations'],op); w=word(o['name'])
  xs=[before.get(x['before']) if isinstance(x,dict) else args.get(x,x) for x in v['args']]
  if None in xs or not all(isinstance(x,(int,float)) for x in xs): return None
  return xs[0]+xs[1] if w=='足す' else xs[0]-xs[1] if w=='引く' else None
def after(cmd,ex):
  """状態の変更から、後の状態を導く（Derived Values）"""
  out={}
  for p in cmd['state_changes']:
    c=p['condition']; v=c.get('value')
    if c['op']=='eq':
      if isinstance(v,dict) and 'call' in v: out[c['target']]=call(v,ex['before'],ex['args'])
      elif isinstance(v,str) and v.startswith('ARG-'): out[c['target']]=ex['args'].get(v)
      else: out[c['target']]=v
    elif c['op']=='gt' and isinstance(v,dict) and 'before' in v:
      b=ex['before'].get(v['before']); n=b.get('count') if isinstance(b,dict) else b
      out[c['target']]={"count":n+1} if v.get('agg')=='count' else f'{n} より大きい'
    elif c['op']=='not_empty': out[c['target']]='（空でない）'
  return out
def show(v,a=None):
  if isinstance(v,dict) and 'count' in v: return f'{v["count"]}件'
  if isinstance(v,str) and v.startswith('TERM-'): return word(v)
  return str(v)
# ── ドメイン ・ サブドメイン ・ 文脈 ・ ドメインサービス
OPW={"le":"以下","ge":"以上","lt":"未満","gt":"より大きい"}
def thr(t): return f'{t["value"]}{t["unit"]}{OPW[t["op"]]}'
def sc_text(s):
  if s['source']=='event':
    a,b=s['measure']['diff']; ev=word(a)
    return f'{ev}の時刻と{qname(b)}の差が{thr(s["threshold"])}の割合が、{s["window"]}ごとに{thr(s["ratio"])}'
  return f'{s["measure_text"]}が{thr(s["threshold"])}（{s["window"]}）'
def event_word(ref):
  a,e=ref.split('.')
  for c in D[a]['commands']:
    for x in c['emits']:
      if x['id']==e: return word(x['name'])
  return ref
def qr_text(q):
  c=q['condition']; u,sid=q['target'].split('.',1); s=uc_part(D[u],sid)
  return f'「{dname(u)}」の手順{number(D[u])[sid]}（{step_short(s)}）の{q["measure"]}が{thr(q["threshold"])}の割合が{thr(q["ratio"])}（{c["period"]} ・ {c["load"]["value"]}{c["load"]["unit"]}）'
def derived_category(c):
  if c['competitive_advantage']: return '中核'
  if c['external_available'] and not c['cheaper_to_build']: return '一般'
  return '補完'
def ds_reason(d):
  if d['header']['reason']=='複数の集約にまたがる計算':
    return '・'.join(dname(x) for x in d['reads'])+'の状態を両方読む計算なので、どちらか1つの集約に置くと、もう一方の状態を持ち込む'
  return d['header']['reason']
def input_name(i):
  f=i['from']; return qname(f['target'])+('の合計' if f.get('agg')=='sum' else '')
def competitor_text(c):
  vs=[find(D['DOM-1']['vision']['values'],v)['name'] for v in c['lacks']]
  return '「'+'」「'.join(vs)+'」が無い'
def scope_in(dom): return [v['header']['name'] for v in D.values() if v['kind']=='subdomain']
# ── 設計の側から要求の側への対応
def apps(): return [v for v in D.values() if v['kind']=='application_operation']
def apps_of(uc): return [a for a in apps() if uc in a['satisfies']]
def resolve_term_path(path):
  """用語集の語の並び（TERM-1.TERM-5）を、集約の状態の参照（AGG-1.ST-2）へ解く。解けなければ None"""
  ws=path.split('.')
  for k,v in D.items():
    if v['kind']=='aggregate' and v['header']['name']==ws[0]:
      if len(ws)==1: return k
      st=[x for x in v['structure']['state'] if x['name']==ws[1]]
      if not st: return None
      if len(ws)==2: return f'{k}.{st[0]["id"]}'
      ent=[e for e in v['structure']['entities'] if e['id']==st[0]['type']]
      es=[x for x in ent[0]['state'] if x['name']==ws[2]] if ent else []
      return f'{k}.{ent[0]["id"]}.{es[0]["id"]}' if es else None
  return None

def impl_method(bl):
  """業務ロジックの性質から実装方法を導く（判断基準 design-heuristics の順）"""
  if bl['needs_tracking']: return 'イベント履歴式ドメインモデル'
  if bl['complex_rules']: return 'ドメインモデル'
  if bl['complex_data']: return 'アクティブレコード'
  return 'トランザクションスクリプト'

def post_text(c,decl=None,cmd=None):
  """状態の変更を「どうなるか」の文にする。比較の文（XはYと同じ）にしない"""
  t=qname(c['target'],decl,cmd); v=c.get('value')
  if c['op']=='eq' and isinstance(v,dict) and 'call' in v:
    vo,op=v['call'].split('.'); o=find(D[vo]['operations'],op); ch=terms()[o['name']].get('change')
    other=[a for a in v['args'] if not isinstance(a,dict)]
    if ch and other: return f'{t}が{qname(other[0],decl,cmd).replace("指定された","")}だけ{ch}'
  if c['op']=='eq' and isinstance(v,str) and v.startswith('ARG-'):
    a=dname(decl) if decl else ''
    return f'{a}に{t}が記録される' if a and not t.startswith(a) else f'{t}が記録される'
  if c['op']=='eq' and isinstance(v,str) and v.startswith('TERM-'): return f'{t}が{word(v)}になる'
  if c['op']=='gt' and isinstance(v,dict) and 'before' in v: return f'{t}が1件増える' if c.get('agg')=='count' else f'{t}が増える'
  return cond(c,decl,cmd)
def item_post(ref):
  x,decl,cmd=item(ref); return post_text(x['condition'],decl,cmd)
