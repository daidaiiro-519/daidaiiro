# 移す範囲の「移し終える条件」を数える（論点6）。移行の記録（spec/migration.json）の assert ごとの当たり先と、
# 宣言から取り出したテスト条件（concrete7.conditions）を突き合わせる。欠けを手で数えない
import concrete7
def scope_of(cid,uc):
  """テスト条件が移す範囲に入るか。ユースケースはその1件、設計の側は、そのユースケースを満たす操作から届く宣言"""
  return cid.split('.')[0]==uc or not cid.startswith('UC-')
def count(mig):
  conds=[c for c in concrete7.conditions() if scope_of(c['id'],mig['use_case'])]
  hit={h for t in mig['tests'] for a in t['asserts'] for h in a['hits']}
  unhit=[(t['test'],a['line']) for t in mig['tests'] for a in t['asserts'] if not a['hits']]
  gaps=[c['id'] for c in conds if c['id'] not in hit]
  unknown=sorted(hit-{c['id'] for c in conds})
  return {"conditions":len(conds),"gaps":gaps,"asserts_without_condition":unhit,"hits_to_unknown":unknown}
