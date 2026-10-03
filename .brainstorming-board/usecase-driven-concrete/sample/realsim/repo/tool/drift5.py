# 宣言からテスト条件を取り出し、期待する結果を決める欄（と、それが指す上流の項目を1段）からハッシュ値を計算する
import json,hashlib
import gen5 as g
from data5 import IMPL,RETIRED
D=g.D
def use(state):
  global D
  D=state; g.use(state)
LEVEL={"ドメインモデル":{"use_case":"system","quality":"system","aggregate":"component","value_object":"component","domain_service":"component"},
       "アクティブレコード":{"use_case":"system","quality":"system","aggregate":"component-integration","value_object":"component","domain_service":"component-integration"},
       "トランザクションスクリプト":{"use_case":"system","quality":"system","aggregate":"system","value_object":"component","domain_service":"system"}}
def fp(obj): return hashlib.sha256(json.dumps(obj,ensure_ascii=False,sort_keys=True).encode()).hexdigest()[:8]
def ctx_of(d): return d['header'].get('context') or (d['header'].get('scope') or {}).get('context')
def up(ref):
  x,_,_=g.item(ref); return {k:x[k] for k in ('condition','reject','example','threshold','ratio','measure') if k in x}
def sem(s):
  """手順の意味の欄（文は道具が組むので入れない）"""
  return {k:s[k] for k in ('kind','actor','to','data','verb','checks','invokes','calls','reply') if k in s}
def conditions():
  out=[]
  def add(cid,label,kind,checks,expect,decl,anchor):
    lv=LEVEL[IMPL.get(ctx_of(D[decl]),'ドメインモデル')][kind]
    out.append({"id":cid,"label":label,"kind":kind,"checks":checks,"hash":fp(expect),"required_level":lv,"decl":decl,"anchor":anchor})
  for k,d in D.items():
    if d['kind']=='use_case':
      sc=d['scenario']; gu=d['guarantees']; nums=g.number(d)
      add(f'{k}.M','主成功シナリオ','use_case','成功時保証がすべて成り立つ',
          {"steps":[sem(s) for s in sc['steps']],"success":[x.get('established_by') for x in gu['success']],"up":[up(r) for x in gu['success'] for r in x.get('established_by',[])]},k,'M')
      for s in sc['steps']:
        for x in s['extensions']:
          fail=x['ending']=='失敗'
          add(f'{k}.{x["id"]}',nums[x['id']],'use_case','最低保証がすべて成り立ち、成功時保証は成り立たない' if fail else '元の手順に戻り、成功時保証が成り立つ',
              {"kind":x['condition_kind'],"refs":{r:x.get(r) for r in ('handles','fails','actor')},"ending":x['ending'],"steps":[sem(t) for t in x['steps']],
               "minimal":[m['condition'] for m in gu['minimal']] if fail else [],"up":[up(r) for r in x.get('handles',[])+x.get('fails',[])]},k,x['id'])
        for q in s.get('quality',[]):
          qt,_,_=g.item(q); qid=q.split('.')[-1]
          add(f'{k}.{s["id"]}.{qid}',f'手順{nums[s["id"]]}の品質の要求','quality',g.qr_text(qt),{"q":up(q),"step":sem(s)},k,s['id'])
    if d['kind']=='aggregate':
      for i in d['invariants']: add(f'{k}.{i["id"]}',i['id'],'aggregate','違反する操作が拒否される',{"c":i['condition'],"via":i.get('via')},k,i['id'])
      for s in d['structure']['state']:
        hi=s['multiplicity']['max']
        if hi not in (None,1): add(f'{k}.{s["id"]}.MAX',f'{g.state_name(k,s["id"])}の上限','aggregate',f'{g.state_name(k,s["id"])}が{hi}件を超える操作が拒否される',{"max":hi},k,s['id'])
      for c in d['commands']:
        for p in c['preconditions']: add(f'{k}.{c["id"]}.{p["id"]}',p['id'],'aggregate','その拒否の理由で拒否される',{"c":p['condition'],"r":p['reject'],"ex":g.reject_example(d,c,p)},k,c['id'])
        for o in c.get('accept_examples',[]): add(f'{k}.{c["id"]}.{o["id"]}',o['id'],'aggregate','事後条件と業務イベントが成り立つ',{"post":[p['condition'] for p in c['postconditions']],"emits":[e['id'] for e in c['emits']],"ex":{x:o[x] for x in ('before','args')}},k,c['id'])
    if d['kind']=='value_object':
      for cp in d['components']:
        for i in cp['invariants']: add(f'{k}.{i["id"]}',i['id'],'value_object','作れない値を拒む',{"c":i['condition']},k,i['id'])
      for o in d['operations']:
        for x in o.get('accept_examples',[]): add(f'{k}.{o["id"]}.{x["id"]}',f'{g.word(o["name"])}の例','value_object','操作の結果が例と同じ',{"op":o['name'],"ex":x},k,o['id'])
    if d['kind']=='domain_service':
      for o in d['operations']:
        for p in o['postconditions']: add(f'{k}.{o["id"]}.{p["id"]}',p['id'],'domain_service','結果が事後条件を満たす',{"c":p['condition'],"in":o['inputs']},k,o['id'])
  return out
def report(conds):
  """見本の報告（テスト実装が出したものとして作る）。状態がひととおり出るように選んである"""
  by={c['id']:c for c in conds}; r=[]
  def put(cid,result,level=None,stale=False,reason=None):
    c=by.get(cid); f=(c['hash'] if c else '00000000')
    if stale: f='9f3a01c7'
    x={"condition":cid,"hash":f,"test":"tests/"+cid.lower().replace('.','_')+".rs","level":level or (c['required_level'] if c else 'component'),"result":result}
    if reason: x['reason']=reason
    r.append(x)
  for c in conds:
    cid=c['id']
    if cid in ('AGG-1.ST-3.MAX','AGG-1.CMD-3.OK-1'): continue
    if cid=='UC-1.EXT-3': put(cid,'fail'); continue
    if cid=='AGG-1.INV-2': put(cid,'pass',stale=True); continue
    if cid=='AGG-1.CMD-1.PRE-2': put(cid,'skip',reason='明細の作り方が決まっていない'); continue
    if cid.startswith('UC-1.STEP-5'): put(cid,'pass',level='component'); continue
    put(cid,'pass')
  put('AGG-1.INV-3','pass',level='component')
  return {"schema":"report.schema.json","producer":"cargo test（来店前注文）","spec_revision":"3f2c9a1","results":r}
def judge(conds,rep,exempt=()):
  res={}
  for c in conds:
    rows=[x for x in rep['results'] if x['condition']==c['id']]
    if c['id'] in exempt: st='免除'
    elif not rows: st='欠け'
    elif any(x['result']=='fail' for x in rows): st='不合格'
    elif any(x['hash']!=c['hash'] for x in rows): st='古い'
    elif all(x['result']=='skip' for x in rows): st='未実行'
    elif not any(x['level']==c['required_level'] and x['result']=='pass' for x in rows): st='レベル違い'
    else: st='合格'
    res[c['id']]=st
  known={c['id'] for c in conds}
  extra=[x for x in rep['results'] if x['condition'] not in known]
  return res,extra
if __name__=='__main__':
  cs=conditions(); print(len(cs))
  for c in cs: print(c['id'],c['label'],c['checks'],c['required_level'])
  j,e=judge(cs,report(cs)); import collections; print(collections.Counter(j.values()),[x['condition'] for x in e])
