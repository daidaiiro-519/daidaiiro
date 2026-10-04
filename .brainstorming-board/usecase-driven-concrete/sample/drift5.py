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
  x,_,_=g.item(ref); return {k:x[k] for k in ('condition','reject','example','threshold','ratio','measure','name','sends','receives','failures','retry') if k in x}
def sem(s):
  """手順の意味の欄（文は道具が組むので入れない）"""
  return {k:s[k] for k in ('kind','actor','to','data','verb','checks','invokes','calls','reply','via') if k in s}
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
              {"kind":x['condition_kind'],"refs":{r:x.get(r) for r in ('handles','fails','actor','fails_external')},"ending":x['ending'],"steps":[sem(t) for t in x['steps']],
               "minimal":[m['condition'] for m in gu['minimal']] if fail else [],"up":[up(r) for r in x.get('handles',[])+x.get('fails',[])+x.get('fails_external',[])]},k,x['id'])
        for q in s.get('quality',[]):
          qt,_,_=g.item(q); qid=q.split('.')[-1]
          add(f'{k}.{s["id"]}.{qid}',f'手順{nums[s["id"]]}の品質の要求','quality',g.qr_text(qt),{"q":up(q),"step":sem(s)},k,s['id'])
    if d['kind']=='aggregate':
      for i in d['invariants']: add(f'{k}.{i["id"]}',i['id'],'aggregate','違反する操作が拒否される',{"c":i['condition'],"via":i.get('via')},k,i['id'])
      for s in d['structure']['state']:
        hi=s['multiplicity']['max']
        if hi not in (None,1): add(f'{k}.{s["id"]}.MAX',f'{g.state_name(k,s["id"])}の上限','aggregate',f'{g.state_name(k,s["id"])}が{hi}件を超える操作が拒否される',{"max":hi},k,s['id'])
      for c in d['commands']:
        for p in c['business_rules']: add(f'{k}.{c["id"]}.{p["id"]}',p['id'],'aggregate','その拒否の理由で拒否される',{"c":p['condition'],"r":p['reject'],"ex":g.reject_example(d,c,p)},k,c['id'])
        for o in c.get('accept_examples',[]): add(f'{k}.{c["id"]}.{o["id"]}',o['id'],'aggregate','状態の変更と業務イベントが成り立つ',{"post":[p['condition'] for p in c['state_changes']],"emits":[e['id'] for e in c['emits']],"ex":{x:o[x] for x in ('before','args')}},k,c['id'])
    if d['kind']=='value_object':
      for cp in d['components']:
        for i in cp['invariants']: add(f'{k}.{i["id"]}',i['id'],'value_object','作れない値を拒む',{"c":i['condition']},k,i['id'])
      for o in d['operations']:
        for x in o.get('accept_examples',[]): add(f'{k}.{o["id"]}.{x["id"]}',f'{g.word(o["name"])}の例','value_object','操作の結果が例と同じ',{"op":o['name'],"ex":x},k,o['id'])
    if d['kind']=='domain_service':
      for o in d['operations']:
        for p in o['results']: add(f'{k}.{o["id"]}.{p["id"]}',p['id'],'domain_service','計算の結果が宣言どおりである',{"c":p['condition'],"in":o['inputs']},k,o['id'])
  return out
if __name__=='__main__':
  cs=conditions(); print(len(cs))
  for c in cs: print(c['id'],c['label'],c['checks'],c['required_level'])
