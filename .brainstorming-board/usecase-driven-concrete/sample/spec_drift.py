# concrete のドリフト検知：宣言どうしのずれ。構造で検査できるもの・対応の欄で検査できるもの・人のレビューへ渡すものに分ける
import json,re
from data3 import D
from drift import fp
KIND={"aggregate":"集約","value_object":"値オブジェクト","domain_service":"ドメインサービス"}
def terms(ctx): return {t['id']:t for t in D[ctx]['ubiquitous_language']['terms']}
def item(ref):
  """「宣言.コマンド.事前条件」などの ID から、参照される項目を引く"""
  p=ref.split('.'); d=D[p[0]]
  if d['kind']=='aggregate':
    c=[x for x in d['commands'] if x['id']==p[1]][0]
    if len(p)==2: return c
    return [x for x in c['preconditions']+c['postconditions'] if x['id']==p[2]][0]
  return None
def expect(ref):
  """参照される項目のうち、期待する結果を決める欄だけ"""
  x=item(ref); return {k:x[k] for k in ('condition','reject','example') if k in x}
def links():
  """宣言どうしの対応の一覧（参照元・欄・参照先）"""
  out=[]
  for uk,U in [(k,v) for k,v in D.items() if v['kind']=='use_case']:
    out+=_links(uk,U)
  return out
def _links(uk,U):
  out=[]
  for p in U['preconditions']:
    for r in p.get('ensures',[]): out.append((uk+'.'+p['id'],'ensures',r))
  for g in U['guarantees']['success']:
    for r in g.get('established_by',[]): out.append((uk+'.'+g['id'],'established_by',r))
  for s in U['scenario']['steps']:
    if s.get('invokes'): out.append((uk+'.'+s['id'],'invokes',s['invokes']))
    for x in s['extensions']:
      for r in x.get('handles',[]): out.append((uk+'.'+x['id'],'handles',r))
  return out
# 確かめた時点の上流のハッシュ値。宣言とは別の記録で、道具が持ち、PR の承認で確定する
def confirmed():
  c={f'{a}→{r}':fp(expect(r)) for a,f,r in links() if f!='invokes'}
  c['UC-1.EXT-2→AGG-2.CMD-1.PRE-1']='5e0a9c21'   # 見本：承認のあとで、確保する の事前条件が書き換わった
  return c
def checks(rec=None):
  R=[]
  def put(cat,name,src,dst,text,st): R.append({"category":cat,"check":name,"from":src,"to":dst,"text":text,"status":st})
  # (a) 構造だけで検査できる
  seen={}
  for k,d in D.items():
    if d['kind'] in KIND:
      T=terms(d['header']['context']); t=T.get(d['header']['name'])
      if t and t['kind']!=KIND[d['kind']]: put('structure','名前の用語の種類',k,d['header']['name'],f'{t["word"]} の種類が {t["kind"]}','ずれ')
    if d['kind']=='aggregate':
      T=terms(d['header']['context'])
      for s in d['structure']['state']:
        if not s['name'].startswith('TERM-'): put('structure','状態の名前が用語を指す',f'{k}.{s["id"]}','',f'「{s["name"]}」が用語集の語を指していない','ずれ')
      for c in d['commands']:
        for p in c['preconditions']:
          seen.setdefault(p['reject'],[]).append(f'{k}.{c["id"]}.{p["id"]}')
      posts=[(c['id'],json.dumps(c['postconditions'],sort_keys=True)) for c in d['commands']]
      for i,(a,pa) in enumerate(posts):
        for b,pb in posts[i+1:]:
          if pa==pb: put('structure','別のコマンドの事後条件が同じ',f'{k}.{a}',f'{k}.{b}','名前の違うコマンドが同じ事後条件を持つ（意味はレビューで確かめる）','ずれ')
      for inv in d['invariants']:
        s=[x for x in d['structure']['state'] if x['id']==inv['condition']['target']][0]
        vo=D.get(s['type'])
        if vo and vo['kind']=='value_object' and inv['condition']['op']=='ge':
          lo=[i['condition']['value'] for cp in vo['components'] for i in cp['invariants'] if i['condition']['op']=='ge']
          if lo and inv['condition']['value']<min(lo): put('structure','値オブジェクトと不変条件が両立する',f'{k}.{inv["id"]}',s['type'],f'状態の型は {min(lo)} 以上なのに、不変条件は {inv["condition"]["value"]} 以上を許す','ずれ')
    if d['kind']=='domain_service':
      for o in d['operations']:
        for i in o['inputs']:
          if i['name'].endswith('の合計'):
            for a in d['reads']:
              for e in D[a]['structure']['entities']:
                for es in e['state']:
                  if es['type']==i['type']:
                    hi=[x['condition']['value'] for cp in D[i['type']]['components'] for x in cp['invariants'] if x['condition']['op']=='le']
                    put('structure','値オブジェクトの範囲を超えない',f'{k}.{o["id"]}',f'{a}.{e["id"]}',f'1件ごとに {hi[0]} 以下の値を、複数件足すと {hi[0]} を超えうる','ずれ')
    if d['kind']=='use_case':
      ctx=d['header']['scope']['context']
      for s in d['scenario']['steps']:
        if s.get('invokes') and D[s['invokes'].split('.')[0]]['header']['context']!=ctx: put('structure','invokes が文脈の中',f'{k}.{s["id"]}',s['invokes'],'スコープの文脈の外の集約を呼んでいる','ずれ')
  for r,us in seen.items():
    owners={u.split('.')[0] for u in us}
    if len(owners)>1: put('structure','拒否の理由を1つの集約だけが使う',us[-1],r,f'同じ拒否の理由を {"・".join(sorted(owners))} が使っている','ずれ')
  # (b) 対応の欄で検査できる
  U=D['UC-1']; st=U['scenario']['steps']
  ens={r for p in U['preconditions'] for r in p.get('ensures',[])}
  for s in st:
    if not s.get('invokes'): continue
    a,c=s['invokes'].split('.'); cm=[x for x in D[a]['commands'] if x['id']==c][0]
    hd={r for x in s['extensions'] for r in x.get('handles',[])}
    for p in cm['preconditions']:
      r=f'{a}.{c}.{p["id"]}'
      if r not in ens|hd: put('link','コマンドの拒否を扱う',f'UC-1.{s["id"]}',r,f'手順{s["no"]}が呼ぶコマンドの拒否「{terms(D[a]["header"]["context"])[p["reject"]]["word"]}」を、事前条件も拡張も扱っていない','対応の欠け')
      else: put('link','コマンドの拒否を扱う',f'UC-1.{s["id"]}',r,'事前条件（ensures）か拡張（handles）が扱っている','合格')
  for g in U['guarantees']['success']:
    if not g.get('established_by'): put('link','成功時保証を成り立たせる事後条件','UC-1.'+g['id'],'',f'「{g["name"]}」を成り立たせる事後条件が無い','対応の欠け')
    else: put('link','成功時保証を成り立たせる事後条件','UC-1.'+g['id'],'・'.join(g['established_by']),'事後条件が成り立たせている','合格')
  kept={m for s in st for x in [s]+[t for e in s['extensions'] for t in e['steps']] for m in x.get('keeps',[])}
  for m in U['guarantees']['minimal']:
    put('link','最低保証を守る手順','UC-1.'+m['id'],'', '守る手順がある' if m['id'] in kept else f'「{m["name"]}」を守る手順が無い','合格' if m['id'] in kept else '対応の欠け')
  # 上流の変更：確かめた時点のハッシュ値と、今のハッシュ値
  C=confirmed() if rec is None else rec
  for a,f,r in links():
    if f=='invokes': continue
    key=f'{a}→{r}'; now=fp(expect(r))
    if key not in C: put('change',f'上流の変更（{f}）',a,r,'まだ承認していない対応。PR の承認で記録する','確かめ直し')
    elif C[key]!=now: put('change',f'上流の変更（{f}）',a,r,'承認のあとで参照先が変わった。参照元を確かめ直す','確かめ直し')
    else: put('change',f'上流の変更（{f}）',a,r,'承認した時点から変わっていない','合格')
  # (c) 人のレビューへ渡す
  for k,d in D.items():
    if d['kind']=='subdomain':
      c=d['classification']['category']; bl=d['business_logic']
      if c=='一般' and (bl['needs_tracking'] or bl['complex_rules']): put('review','カテゴリーと業務ロジックの性質',k,'',f'一般のサブドメインなのに、「経緯を追う必要がある」が真','レビュー')
  if 'AGG-2' in D and D['AGG-2']['commands'][0]['args'][0]['name']=='TERM-3': put('review','用語の定義が文脈に合う','AGG-2.CMD-1.ARG-1','TERM-3','引数「数量」の定義は「1つの明細で注文する商品の個数」で、調理枠の量とは意味が違う疑い','レビュー')
  return R
if __name__=='__main__':
  import collections
  r=checks(); print(collections.Counter(x['status'] for x in r))
  for x in r:
    if x['status']!='合格': print(x['status'],x['check'],x['from'],x['to'],x['text'])
