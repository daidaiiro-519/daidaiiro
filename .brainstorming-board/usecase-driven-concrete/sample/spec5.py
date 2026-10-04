# 宣言どうしのずれ。構造で検査 ・ 対応の欄で検査 ・ 上流の変更 ・ 人のレビュー（候補）に分ける。自由文は読まない
import gen5 as g
from drift5 import fp,up
D=g.D
def use(state):
  global D
  D=state; g.use(state)
KIND={"aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
def links():
  out=[]
  for uk,U in [(k,v) for k,v in D.items() if v['kind']=='use_case']:
    for p in U['preconditions']:
      for r in p.get('ensures',[]): out.append((f'{uk}.{p["id"]}','ensures',r))
    for x in U['guarantees']['success']:
      for r in x.get('established_by',[]): out.append((f'{uk}.{x["id"]}','established_by',r))
    for s in g.all_steps(U):
      if s.get('invokes'): out.append((f'{uk}.{s["id"]}','invokes',s['invokes']))
      for r in s.get('checks',[]): out.append((f'{uk}.{s["id"]}','checks',r))
      for r in s.get('quality',[]): out.append((f'{uk}.{s["id"]}','quality',r))
      if s.get('via'): out.append((f'{uk}.{s["id"]}','via',s['via']))
      for x in s.get('extensions',[]):
        for f in ('handles','fails','fails_external'):
          for r in x.get(f,[]): out.append((f'{uk}.{x["id"]}',f,r))
  return out
def record(): return {f'{a}→{r}':fp(up(r)) for a,f,r in links() if f!='invokes'}
def checks(rec=None):
  R=[]
  def put(cat,name,src,dst,text,st): R.append({"category":cat,"check":name,"from":src,"to":dst,"text":text,"status":st})
  T=g.terms()
  def kind_is(t,k,src,what):
    x=T.get(t)
    if x is None: put('structure','名前が用語を指す',src,t,f'{what}が用語集の語を指していない','ずれ')
    elif k and x['kind'] not in (k if isinstance(k,tuple) else (k,)): put('structure','名前が用語を指す',src,t,f'{what}「{x["word"]}」の語の種類が {x["kind"]}','ずれ')
  rejects={}
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
          ex=g.reject_example(d,c,p)
          if ex is None: put('structure','拒否の例が組める',f'{k}.{c["id"]}.{p["id"]}','','条件から拒否の例を組めず、宣言にも例が無い','ずれ')
        for e in c['emits']: kind_is(e['name'],'業務イベント',f'{k}.{c["id"]}.{e["id"]}','業務イベントの名前')
        posts.append((c['id'],str(c['state_changes'])))
      for i,(a,pa) in enumerate(posts):
        for b,pb in posts[i+1:]:
          if pa==pb: put('structure','別のコマンドの状態の変更が同じ',f'{k}.{a}',f'{k}.{b}','名前の違うコマンドが同じ状態の変更を持つ','ずれ')
      for inv in d['invariants']:
        if not inv.get('via'): put('structure','違反する例に至る操作がある',f'{k}.{inv["id"]}','','違反する状態に至る操作（via）が無い','ずれ')
    if d['kind']=='domain_service':
      for o in d['operations']:
        for i in o['inputs']:
          f=i['from']
          if f.get('agg')=='sum':
            a,e,es=f['target'].split('.'); ent=[x for x in D[a]['structure']['entities'] if x['id']==e][0]
            ty=[x for x in ent['state'] if x['id']==es][0]['type']
            m=[s['multiplicity']['max'] for s in D[a]['structure']['state'] if s['type']==e][0]
            hi=[x['condition']['value'] for c in D[ty]['components'] for x in c['invariants'] if x['condition']['op']=='le']
            hin=[x['condition']['value'] for c in D[i['type']]['components'] for x in c['invariants'] if x['condition']['op']=='le']
            if hi and hin and m and hi[0]*m>hin[0]: put('structure','値オブジェクトの範囲を超えない',f'{k}.{o["id"]}.{i["id"]}',i['type'],f'合計は最大 {hi[0]*m} になり、{g.dname(i["type"])}の上限 {hin[0]} を超える','ずれ')
            else: put('structure','値オブジェクトの範囲を超えない',f'{k}.{o["id"]}.{i["id"]}',i['type'],f'合計は最大 {hi[0]*m} で、{g.dname(i["type"])}の上限 {hin[0]} に収まる','合格')
    if d['kind']=='subdomain':
      c=d['classification']; dc=g.derived_category(c)
      put('structure','カテゴリーが判断基準の答えと合う',k,'',f'判断基準の答えから導いたカテゴリーは {dc}、宣言は {c["category"]}','合格' if dc==c['category'] else 'ずれ')
      bl=d['business_logic']
      m=g.impl_method(bl)
      bad=(c['category']=='中核' and 'ドメインモデル' not in m) or (c['category']!='中核' and 'ドメインモデル' in m)
      put('review' if bad else 'structure','カテゴリーと実装方法',k,'',f'{c["category"]}で、業務ロジックの性質から導いた実装方法は{m}'+('。判断を再検討する機会' if bad else ''),'レビュー' if bad else '合格')
    if d['kind']=='use_case':
      ctx=d['header']['scope'].get('context'); nums=g.number(d)
      for s in g.all_steps(d):
        src=f'{k}.{s["id"]}'
        if s['kind']=='内部の状態変化' and not s.get('invokes'): put('structure','内部の状態変化は invokes を持つ',src,'',f'手順{nums[s["id"]]}が呼ぶコマンドを指していない','ずれ')
        if s['kind']=='相互作用':
          for t in s['data']: kind_is(t,('情報の別名','値オブジェクト'),src,'渡す情報')
          kind_is(s['verb'],'動作',src,'述語')
        if ctx and s.get('invokes') and D[s['invokes'].split('.')[0]]['header']['context']!=ctx: put('structure','invokes が文脈の中',src,s['invokes'],'スコープの文脈の外を呼んでいる','ずれ')
        if s['kind']=='妥当性確認':
          f=[r for x in s.get('extensions',[]) for r in x.get('fails',[])]
          for r in s['checks']:
            put('link','確かめた条件の失敗を扱う',src,r,'拡張が扱っている' if r in f else f'手順{nums[s["id"]]}で確かめる条件が成り立たないときの拡張が無い','合格' if r in f else '対応の欠け')
  for r,owners in rejects.items():
    if len(owners)>1: put('structure','拒否の理由を1つの集約だけが使う','・'.join(sorted(owners)),r,f'同じ拒否の理由を {"・".join(sorted(owners))} が使っている','ずれ')
  for uk,U in [(k,v) for k,v in D.items() if v['kind']=='use_case']:
    nums=g.number(U); ens={r for p in U['preconditions'] for r in p.get('ensures',[])}
    for s in U['scenario']['steps']:
      if not s.get('invokes') or not s['invokes'].startswith('AGG'): continue
      a,c=s['invokes'].split('.'); cm=[x for x in D[a]['commands'] if x['id']==c][0]
      hd={r for x in s['extensions'] for r in x.get('handles',[])}
      for p in cm['business_rules']:
        r=f'{a}.{c}.{p["id"]}'
        if r in ens|hd: put('link','コマンドの拒否を扱う',f'{uk}.{s["id"]}',r,'事前条件（ensures）か拡張（handles）が扱っている','合格')
        else: put('link','コマンドの拒否を扱う',f'{uk}.{s["id"]}',r,f'手順{nums[s["id"]]}が呼ぶコマンドの拒否「{g.word(p["reject"])}」を、事前条件も拡張も扱っていない','対応の欠け')
    for s in U['scenario']['steps']:
      if not s.get('via'): continue
      op,_,_=g.item(s['via']); hd={r for x in s['extensions'] for r in x.get('fails_external',[])}
      for f in op['failures']:
        r=f'{s["via"]}.{f["id"]}'
        put('link','外部の操作の失敗を扱う',f'{uk}.{s["id"]}',r,'拡張が扱っている' if r in hd else f'手順{nums[s["id"]]}が使う外部の操作の失敗「{g.word(f["name"])}」を、どの拡張も扱っていない','合格' if r in hd else '対応の欠け')
    for x in U['guarantees']['success']:
      ok=bool(x.get('established_by'))
      put('link','成功時保証を成り立たせる状態の変更',f'{uk}.{x["id"]}','・'.join(x.get('established_by',[])),'状態の変更が成り立たせている' if ok else f'「{x["name"]}」を成り立たせる状態の変更が無い','合格' if ok else '対応の欠け')
    kept={m for t in g.all_steps(U) for m in t.get('keeps',[])}
    for m in U['guarantees']['minimal']:
      put('link','最低保証を守る手順',f'{uk}.{m["id"]}','','守る手順がある' if m['id'] in kept else f'「{m["name"]}」を守る手順が無い','合格' if m['id'] in kept else '対応の欠け')
  C=record() if rec is None else rec
  for a,f,r in links():
    if f in ('invokes',): continue
    key=f'{a}→{r}'; now=fp(up(r))
    if key not in C: put('change',f'上流の変更（{f}）',a,r,'まだ承認していない対応。PR の承認で記録する','確かめ直し')
    elif C[key]!=now: put('change',f'上流の変更（{f}）',a,r,'承認のあとで参照先が変わった。参照元を確かめ直す','確かめ直し')
    else: put('change',f'上流の変更（{f}）',a,r,'承認した時点から変わっていない','合格')
  return R
if __name__=='__main__':
  import collections
  r=checks(); print(collections.Counter(x['status'] for x in r))
  for x in r:
    if x['status']!='合格': print(x)
