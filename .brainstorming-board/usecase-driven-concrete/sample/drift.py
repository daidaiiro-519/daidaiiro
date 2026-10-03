# concrete のドリフト検知：宣言からテスト条件を取り出し、指紋を計算し、テスト実装の報告と突き合わせる
import json,hashlib
from data3 import D,IMPL
LEVEL={"ドメインモデル":{"use_case":"system","quality":"system","aggregate":"component","value_object":"component","domain_service":"component"},
       "アクティブレコード":{"use_case":"system","quality":"system","aggregate":"component-integration","value_object":"component","domain_service":"component-integration"},
       "トランザクションスクリプト":{"use_case":"system","quality":"system","aggregate":"system","value_object":"component","domain_service":"system"}}
def fp(obj): return hashlib.sha256(json.dumps(obj,ensure_ascii=False,sort_keys=True).encode()).hexdigest()[:8]
def ctx_of(d): return d['header'].get('context') or (d['header'].get('scope') or {}).get('context')
def conditions():
  out=[]
  def add(cid,label,kind,checks,expect,decl,anchor):
    lv=LEVEL[IMPL.get(ctx_of(D[decl]),'ドメインモデル')][kind]
    out.append({"id":cid,"label":label,"kind":kind,"checks":checks,"fingerprint":fp(expect),"required_level":lv,"decl":decl,"anchor":anchor})
  for k,d in D.items():
    if d['kind']=='use_case':
      sc=d['scenario']; g=d['guarantees']
      add(f'{k}.M','主成功シナリオ','use_case','成功時保証がすべて成り立つ',{"steps":[(s['actor'],s['text']) for s in sc['steps']],"success":[x['text'] for x in g['success']]},k,'M')
      for s in sc['steps']:
        for x in s['extensions']:
          fail=x['ending']=='失敗'
          add(f'{k}.{x["id"]}',x['label'],'use_case','最低保証がすべて成り立ち、成功時保証は成り立たない' if fail else '元の手順に戻り、成功時保証が成り立つ',{"cond":x['condition'],"steps":[(t['actor'],t['text']) for t in x['steps']],"ending":x['ending'],"hold":[m['text'] for m in g['minimal'] if m['id'] in x.get('guarantees_hold',[])]},k,x['id'])
        for q in s.get('quality',[]):
          qt=[x for x in D['DOM-1']['quality'] if x['id']==q][0]
          add(f'{k}.{s["id"]}.{q}',f'手順{s["no"]}の品質の要求','quality',qt['text'],{"q":qt['text'],"step":s['text']},k,s['id'])
    if d['kind']=='aggregate':
      for i in d['invariants']: add(f'{k}.{i["id"]}',i['id'],'aggregate','違反する操作が拒否される',{"c":i['condition'],"v":i['violation']},k,i['id'])
      for c in d['commands']:
        for p in c['preconditions']: add(f'{k}.{c["id"]}.{p["id"]}',p['id'],'aggregate','その拒否の理由で拒否される',{"c":p['condition'],"r":p['reject'],"e":p['example']},k,c['id'])
        for o in c.get('accept_examples',[]): add(f'{k}.{c["id"]}.{o["id"]}',o['id'],'aggregate','事後条件と業務イベントが成り立つ',{"post":[p['condition'] for p in c['postconditions']],"emits":[e['id'] for e in c['emits']],"e":o['text']},k,c['id'])
    if d['kind']=='value_object':
      for cp in d['components']:
        for i in cp['invariants']: add(f'{k}.{i["id"]}',i['id'],'value_object','作れない値を拒む',{"c":i['condition'],"x":i['impossible']},k,i['id'])
    if d['kind']=='domain_service':
      for o in d['operations']:
        for p in o['postconditions']: add(f'{k}.{o["id"]}.{p["id"]}',p['id'],'domain_service','結果が事後条件を満たす',{"c":p['condition']},k,o['id'])
  return out
RETIRED=["AGG-1.INV-9"]
def report(conds):
  by={c['id']:c for c in conds}
  r=[]
  def put(cid,result,level=None,stale=False,reason=None):
    c=by.get(cid); f=(c['fingerprint'] if c else '00000000')
    if stale: f='9f3a01c7'
    x={"condition":cid,"fingerprint":f,"test":"tests/"+cid.lower().replace('.','_')+".rs","level":level or (c['required_level'] if c else 'component'),"result":result}
    if reason: x['reason']=reason
    r.append(x)
  for c in conds:
    cid=c['id']
    if cid=='AGG-1.INV-3': continue
    if cid=='UC-1.EXT-3': put(cid,'fail'); continue
    if cid=='AGG-1.INV-2': put(cid,'pass',stale=True); continue
    if cid=='AGG-1.CMD-1.PRE-2': put(cid,'skip',reason='明細の作り方が決まっていない'); continue
    if cid.startswith('UC-1.STEP-5'): put(cid,'pass',level='component'); continue
    if cid.startswith('AGG-2') or cid.startswith('VO-2') : continue
    put(cid,'pass')
  put('AGG-1.INV-9','pass',level='component')
  return {"schema":"report.schema.json","producer":"cargo test（来店前注文）","spec_revision":"3f2c9a1","results":r}
def judge(conds,rep,exempt=()):
  res={}
  for c in conds:
    rows=[x for x in rep['results'] if x['condition']==c['id']]
    if c['id'] in exempt: st='免除'
    elif not rows: st='欠け'
    elif any(x['result']=='fail' for x in rows): st='不合格'
    elif any(x['fingerprint']!=c['fingerprint'] for x in rows): st='古い'
    elif all(x['result']=='skip' for x in rows): st='未実行'
    elif not any(x['level']==c['required_level'] and x['result']=='pass' for x in rows): st='レベル違い'
    else: st='合格'
    res[c['id']]=st
  known={c['id'] for c in conds}
  extra=[x for x in rep['results'] if x['condition'] not in known]
  return res,extra
