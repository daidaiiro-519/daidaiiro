# 宣言どうしのずれ（論点5を論点7で改めた形）。構造で検査 ・ 対応の欄で検査 ・ 上流の変更 ・ 人のレビューに分ける。
# 対応は設計の側（アプリケーション層の操作）にだけ書かれ、ユースケースの側への向きは道具が計算する。自由文は読まない
import gen6 as g
from drift6 import fp
D=g.D
import json,os,re
APPROVED=os.path.join(os.path.dirname(os.path.abspath(__file__)),'approved-record.json')
def use(state):
  global D
  D=state; g.use(state)
KIND={"aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
REQ_SIDE=('domain','glossary','other_requirements','use_case')
def resolve(ref):
  """参照（DOM-1.VAL-1 ・ AGG-1.CMD-1.BR-2 など）が指す宣言か項目。無ければ None"""
  p=ref.split('.'); cur=D.get(p[0])
  def find(o,i):
    if isinstance(o,dict):
      if o.get('id')==i: return o
      for v in o.values():
        r=find(v,i)
        if r is not None: return r
    elif isinstance(o,list):
      for v in o:
        r=find(v,i)
        if r is not None: return r
    return None
  for i in p[1:]:
    if cur is None: return None
    cur=find({k:v for k,v in cur.items() if k!='id'},i)
  return cur
def links():
  """宣言の要素どうしの従属関係（参照元 ・ 欄 ・ 参照先）。仕様と仕様は、この関係だけでつながる"""
  out=[]
  for k,v in D.items():
    t=v['kind']
    if t=='subdomain':
      for u in v.get('use_cases',[]): out.append((k,'use_cases',u))
      for x in v.get('serves_values',[]): out.append((k,'serves_values',f'DOM-1.{x}'))
    if t=='context':
      for x in v['header']['subdomains']: out.append((k,'subdomains',x))
      for r in v['context_map']['relations']:
        if r.get('fulfills'): out.append((f'{k}.{r["id"]}','fulfills',r['fulfills']))
      for b in v.get('business_rules',[]):
        if b.get('implements'): out.append((f'{k}.{b["id"]}','implements',b['implements']))
    if t in ('aggregate','value_object','domain_service'): out.append((k,'context',v['header']['context']))
    if t=='aggregate':
      for x in v['structure']['state']+[y for e in v['structure']['entities'] for y in e['state']]+[a for c in v['commands'] for a in c['args']]:
        if str(x['type']).startswith('VO-'): out.append((k,'type',x['type']))
      for c in v['commands']:
        for b in c['business_rules']:
          if b.get('implements'): out.append((f'{k}.{c["id"]}.{b["id"]}','implements',b['implements']))
    if t=='domain_service':
      for a in v['reads']: out.append((k,'reads',a))
    if t=='use_case':
      out.append((k,'scope',v['header']['scope']['system']))
      for x in v['stakeholders']: out.append((f'{k}.{x["id"]}','who',x['who']))
      for f,xs in v.get('links',{}).items():
        for x in xs: out.append((k,f'links.{f}',x))
      for x in v['preconditions']:
        if x.get('established_by'): out.append((f'{k}.{x["id"]}','established_by',x['established_by']))
      for x in v.get('contributes_to',[]): out.append((k,'contributes_to',f'DOM-1.{x}'))
      for st in g.all_steps(v):
        if st.get('calls'): out.append((f'{k}.{st["id"]}','calls',st['calls']))
        for x in st.get('checks',[]): out.append((f'{k}.{st["id"]}','checks',x))
  return out
def record(): return {f'{a}→{r}':fp(resolve(r)) for a,f,r in links()}
def checks(rec=None):
  """rec は承認した時点の記録。渡さなければ、どの対応もまだ承認していないものとして扱う（自分自身と比べない）"""
  R=[]
  def put(cat,name,src,dst,text,st): R.append({"category":cat,"check":name,"from":src,"to":dst,"text":text,"status":st})
  T=g.terms()
  def kind_is(t,k,src,what):
    x=T.get(t)
    if x is None: put('structure','名前が用語を指す',src,t,f'{what}が用語集の語を指していない','ずれ')
    elif k and x['kind'] not in (k if isinstance(k,tuple) else (k,)): put('structure','名前が用語を指す',src,t,f'{what}「{x["word"]}」の語の種類が {x["kind"]}','ずれ')
  rejects={}
  # ── 設計の側の構造（集約 ・ ドメインサービス ・ サブドメイン。論点5と同じ）
  for k,d in D.items():
    if d['kind'] in KIND: kind_is(d['header']['name'],KIND[d['kind']],k,'名前')
    if d['kind']=='aggregate':
      for s in d['structure']['state']: kind_is(s['name'],None,f'{k}.{s["id"]}','状態の名前')
      for e in d['structure']['entities']:
        for s in e['state']: kind_is(s['name'],None,f'{k}.{e["id"]}.{s["id"]}','状態の名前')
      posts=[]
      for c in d['commands']:
        kind_is(c['name'],'コマンド',f'{k}.{c["id"]}','コマンドの名前')
        for p in c['business_rules']:
          kind_is(p['reject'],'拒否の理由',f'{k}.{c["id"]}.{p["id"]}','拒否の理由')
          rejects.setdefault(p['reject'],set()).add(k)
          if g.reject_example(d,c,p) is None: put('structure','拒否の例が組める',f'{k}.{c["id"]}.{p["id"]}','','条件から拒否の例を組めず、宣言にも例が無い','ずれ')
        for e in c['emits']: kind_is(e['name'],'業務イベント',f'{k}.{c["id"]}.{e["id"]}','業務イベントの名前')
        posts.append((c['id'],str(c['state_changes'])))
      for i,(a,pa) in enumerate(posts):
        for b,pb in posts[i+1:]:
          if pa==pb: put('structure','別のコマンドの状態の変更が同じ',f'{k}.{a}',f'{k}.{b}','名前の違うコマンドが同じ状態の変更を持つ','ずれ')
      for inv in d['invariants']:
        if not inv.get('via'): put('structure','違反する例に至る操作がある',f'{k}.{inv["id"]}','','違反する状態に至る操作（via）が無い','ずれ')
    if d['kind']=='subdomain':
      c=d['classification']; dc=g.derived_category(c)
      put('structure','カテゴリーが判断基準の答えと合う',k,'',f'判断基準の答えから導いたカテゴリーは {dc}、宣言は {c["category"]}','合格' if dc==c['category'] else 'ずれ')
      m=g.impl_method(d['business_logic'])
      bad=(c['category']=='中核' and 'ドメインモデル' not in m) or (c['category']!='中核' and 'ドメインモデル' in m)
      put('review' if bad else 'structure','カテゴリーと実装方法',k,'',f'{c["category"]}で、業務ロジックの性質から導いた実装方法は{m}'+('。判断を再検討する機会' if bad else ''),'レビュー' if bad else '合格')
  for r,owners in rejects.items():
    if len(owners)>1: put('structure','拒否の理由を1つの集約だけが使う','・'.join(sorted(owners)),r,f'同じ拒否の理由を {"・".join(sorted(owners))} が使っている','ずれ')
  # ── 用語集：1つの文脈が同じ語の2つの意味を指さない
  for k,v in D.items():
    if v['kind']!='context': continue
    by={}
    for u in v['uses']: by.setdefault(u['term'],set()).add(u['meaning'])
    two=[t for t,ms in by.items() if len(ms)>1]
    for t in two: put('structure','1つの文脈が1つの語に1つの意味',k,t,f'「{g.word(t)}」の意味を2つ使っている。語を2つに分ける','ずれ')
    if not two: put('structure','1つの文脈が1つの語に1つの意味',k,'',f'{len(by)}語とも、使う意味は1つ','合格')
    unk=[u['term'] for u in v['uses'] if u['term'] not in T]
    for t in unk: put('structure','使う意味が用語集にある',k,t,'用語集に無い語を指している','ずれ')
  # ── その他の要求：1件のビジネスルールを2つの文脈で実装しない
  impl={}
  for k,v in D.items():
    if v['kind']=='context':
      for b in v.get('business_rules',[]):
        if b.get('implements'): impl.setdefault(b['implements'],[]).append(k)
  for k,v in D.items():
    if v['kind']!='other_requirements': continue
    for b in v['business_rules']:
      r=f'{k}.{b["id"]}'; who=impl.get(r,[])
      if len(who)>1: put('structure','ビジネスルールを実装する文脈は1つ',r,'・'.join(who),f'「{b["name"]}」を{len(who)}つの文脈で実装している','ずれ')
      elif who: put('structure','ビジネスルールを実装する文脈は1つ',r,who[0],f'「{b["name"]}」は {g.dname(who[0])} が実装する','合格')
      else: put('structure','ビジネスルールを実装する文脈は1つ',r,'',f'「{b["name"]}」を実装する文脈が無い','欠け')
  # ── ユースケースの中（語が用語集にあるか ・ 確かめた条件と最低保証が、ユースケースの中で扱われているか）
  for k,d in D.items():
    if d['kind']!='use_case': continue
    nums=g.number(d)
    for s in g.all_steps(d):
      src=f'{k}.{s["id"]}'
      if s['kind']=='相互作用':
        for t in s['data']: kind_is(t,('情報の別名','値オブジェクト','識別子'),src,'渡す情報')
        kind_is(s['verb'],'動作',src,'述語')
      if s['kind']=='内部の状態変化':
        kind_is(s['object'],None,src,'対象'); kind_is(s['verb'],None,src,'述語')
      if s['kind']=='妥当性確認':
        f=[r for x in s.get('extensions',[]) for r in x.get('fails',[])]
        for r in s['checks']:
          put('structure','確かめた条件の失敗を扱う',src,r,'拡張が扱っている' if r in f else f'手順{nums[s["id"]]}で確かめる条件が成り立たないときの拡張が無い','合格' if r in f else '欠け')
    kept={m for t in g.all_steps(d) for m in t.get('keeps',[])}
    for m in d['guarantees']['minimal']:
      put('structure','最低保証を守る手順',f'{k}.{m["id"]}','','守る手順がある' if m['id'] in kept else f'「{m["name"]}」を守る手順が無い','合格' if m['id'] in kept else '欠け')
  # ── 従属関係：指す先がある
  L=links()
  for a,f,r in L:
    if resolve(r) is None: put('structure','指す先がある',a,r,f'{f} が指す {r} が無い','ずれ')
  put('structure','指す先がある','','',f'{len(L)}本の従属関係のうち、指す先が無いものは {sum(1 for a,f,r in L if resolve(r) is None)}本','合格' if all(resolve(r) is not None for a,f,r in L) else 'ずれ')
  # ── 依存の向き：要求の側は設計の側を指さない
  DESIGN_ID=re.compile(r'\b(SD|BC|AGG|VO|DS|APP|ENT|ST|CMD|CHG|X)-[0-9]+')
  for k,d in D.items():
    if d['kind'] not in REQ_SIDE: continue
    hits=sorted({m.group(0) for m in DESIGN_ID.finditer(json.dumps(d,ensure_ascii=False))})
    put('structure','要求の側は設計の側を指さない',k,'・'.join(hits),(f'設計の側の ID（{"・".join(hits)}）を指している' if hits else '設計の側の ID を指していない'),'ずれ' if hits else '合格')
  # ── 書かれていないもの
  sd_of={}
  for k,v in D.items():
    if v['kind']=='subdomain':
      for u in v.get('use_cases',[]): sd_of.setdefault(u,[]).append(k)
  for k,d in D.items():
    if d['kind']=='use_case' and d['header']['level']!='要約':
      w=sd_of.get(k,[])
      put('structure','ユースケースを束ねるサブドメインは1つ',k,'・'.join(w),(f'{g.dname(w[0])}が束ねる' if len(w)==1 else 'どのサブドメインにも束ねられていない' if not w else f'{len(w)}つのサブドメインに束ねられている'),'合格' if len(w)==1 else '欠け' if not w else 'ずれ')
  for k,v in D.items():
    if v['kind']!='subdomain': continue
    inn=[c for c,x in D.items() if x['kind']=='context' and k in x['header']['subdomains']]
    ful=[f'{c}.{r["id"]}' for c,x in D.items() if x['kind']=='context' for r in x['context_map']['relations'] if r.get('fulfills')==k]
    put('structure','サブドメインを担う文脈がある',k,'・'.join(inn+ful),('区切られた文脈が対象とする' if inn else '文脈の地図の外の相手が担う' if ful else 'どの区切られた文脈にも、外の相手にも担われていない'),'合格' if inn or ful else '欠け')
    m=g.impl_method(v['business_logic'])
    if 'ドメインモデル' in m and inn:
      aggs=[a for a,x in D.items() if x['kind']=='aggregate' and x['header']['context'] in inn]
      put('structure','ドメインモデルの文脈に集約がある',k,'・'.join(aggs),(f'集約 {len(aggs)}つ' if aggs else f'実装方法は{m}だが、文脈に集約が無い'),'合格' if aggs else '欠け')
  used={r for a,f,r in L if f=='type'}|{x.get('type') for v in D.values() if v['kind']=='domain_service' for o in v['operations'] for x in o['inputs']}|{o.get('result') for v in D.values() if v['kind']=='value_object' for o in v['operations']}|{a for v in D.values() if v['kind']=='value_object' for o in v['operations'] for a in o['args']}|{o.get('output') for v in D.values() if v['kind']=='domain_service' for o in v['operations']}
  for k,v in D.items():
    if v['kind']=='value_object' and k not in used: put('review','使われていない値オブジェクト',k,'','どの集約 ・ ドメインサービス ・ 値オブジェクトからも使われていない','レビュー')
  # ── 上流の変更（承認した時点のハッシュ値と比べる）
  Cr={} if rec is None else rec
  for a,f,r in L:
    key=f'{a}→{r}'; now=fp(resolve(r))
    if key not in Cr: put('change',f'上流の変更（{f}）',a,r,'まだ承認していない対応。PR の承認で記録する','確かめ直し')
    elif Cr[key]!=now: put('change',f'上流の変更（{f}）',a,r,'承認のあとで参照先が変わった。参照元を確かめ直す','確かめ直し')
    else: put('change',f'上流の変更（{f}）',a,r,'承認した時点から変わっていない','合格')
  return R
def cond_targets(c):
  out=[c['target']]
  if c.get('if'): out.append(c['if']['target'])
  return [t for t in out if isinstance(t,str) and t.startswith('TERM-')]
if __name__=='__main__':
  import collections
  r=checks(json.load(open(APPROVED,encoding='utf-8'))); print(collections.Counter(x['status'] for x in r))
  for x in r:
    if x['status']!='合格': print(x)
