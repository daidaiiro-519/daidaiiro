# シミュレーション：承認済みの宣言を JSON Patch（RFC 6902）で1手ずつ書き換え、そのたびに道具の2つの検査を流す
import copy
import data5, gen5, drift5, spec5
def use(s):
  gen5.use(s); drift5.use(s); spec5.use(s)
def ptr(doc,path):
  ks=[k.replace('~1','/').replace('~0','~') for k in path.split('/')[1:]]
  cur=doc
  for k in ks[:-1]: cur=cur[int(k)] if isinstance(cur,list) else cur[k]
  return cur,ks[-1]
def apply(doc,ops):
  for o in ops:
    par,k=ptr(doc,o['path'])
    if o['op']=='add':
      if isinstance(par,list): par.append(o['value']) if k=='-' else par.insert(int(k),o['value'])
      else: par[k]=o['value']
    elif o['op']=='replace':
      if isinstance(par,list): par[int(k)]=o['value']
      else: par[k]=o['value']
    elif o['op']=='remove':
      o['_old']=par.pop(int(k)) if isinstance(par,list) else par.pop(k)
def run(rec,rep):
  spec=spec5.checks(rec); conds=drift5.conditions()
  judge,extra=drift5.judge(conds,{"results":list(rep.values())})
  return spec,conds,judge,extra
def tests_for(conds,ids=None):
  return {c['id']:{"condition":c['id'],"hash":c['hash'],"test":"tests/"+c['id'].lower().replace('.','_')+".rs","level":c['required_level'],"result":"pass"} for c in conds if ids is None or c['id'] in ids}
STEPS=[
 {"who":"人（業務エキスパート）","what":"業務が変わり、1回に確保できる調理の量に上限ができた。集約「調理枠」の「確保する」に事前条件を足す",
  "ops":[{"op":"add","path":"/BC-1/ubiquitous_language/terms/-","value":{"id":"TERM-51","word":"一度に確保できる量を超えている","kind":"拒否の理由","definition":"1回で確保したい量が、上限の10食を超えている","avoid":[]}},
         {"op":"add","path":"/AGG-2/commands/0/preconditions/-","value":{"id":"PRE-2","condition":{"target":"ARG-1","op":"le","value":10},"reject":"TERM-51"}}]},
 {"who":"AI","what":"道具が出した「対応の欠け」を受けて、手順4（調理枠を確保する）に拡張 4b を足し、新しい拒否を扱う",
  "ops":[{"op":"add","path":"/BC-1/ubiquitous_language/terms/-","value":{"id":"TERM-52","word":"一度に受けられる量","kind":"情報の別名","definition":"1回の注文で受けられる調理の量の上限","avoid":[]}},
         {"op":"add","path":"/UC-1/scenario/steps/3/extensions/-","value":{"id":"EXT-4","condition_kind":"コマンドの拒否","handles":["AGG-2.CMD-1.PRE-2"],"ending":"失敗","steps":[{"id":"EXT-4.S-1","kind":"相互作用","actor":"システム","to":"顧客","data":["TERM-52"],"verb":"TERM-50","extensions":[]}]}}]},
 {"who":"人（承認者）","what":"PR を承認する。道具が、参照先の項目のハッシュ値を「確かめた時点」として記録する","approve":True,"ops":[]},
 {"who":"AI","what":"道具が出した「欠け」を受けて、テストを書く","tests":True,"ops":[]},
 {"who":"人（業務エキスパート）","what":"業務が変わり、調理枠は予備の1食を残すことになった。「確保する」の事前条件1の比較する値を、値オブジェクトの操作「足す」で書き換える",
  "ops":[{"op":"replace","path":"/AGG-2/commands/0/preconditions/0/condition/value","value":{"call":"VO-5.OP-1","args":["ARG-1",1]}}]},
 {"who":"人（承認者）","what":"確かめ直しになった拡張 4a を読み、扱う拒否が今も正しいことを確かめて PR を承認する","approve":True,"ops":[]},
 {"who":"AI","what":"道具が出した「古い」を受けて、テストを直す","tests":True,"ops":[]},
 {"who":"AI","what":"手順を整理しようとして、拡張 5a の手順 5a1（調理枠を戻す）を消す",
  "ops":[{"op":"remove","path":"/UC-1/scenario/steps/4/extensions/0/steps/0"}]},
 {"who":"人（承認者）","what":"「対応の欠け」があるので PR を差し戻し、手順を元に戻す",
  "ops":[{"op":"add","path":"/UC-1/scenario/steps/4/extensions/0/steps/0","value":"__restore__"}]},
]
def simulate():
  state=copy.deepcopy(data5.D); use(state)
  rec=spec5.record(); rep=tests_for(drift5.conditions())
  spec,conds,judge,extra=run(rec,rep)
  out=[{"no":0,"who":"","what":"承認済みの宣言と、それを満たすテスト。ここから始める","ops":[],"spec":spec,"judge":judge,"conds":conds,"extra":extra,"rep":copy.deepcopy(rep),"text":{}}]
  removed=None
  for i,s in enumerate(STEPS,1):
    ops=copy.deepcopy(s['ops'])
    for o in ops:
      if o.get('value')=='__restore__': o['value']=removed
    apply(state,ops)
    for o in ops:
      if '_old' in o: removed=o.pop('_old'); o['removed']=removed
    if s.get('approve'): rec=spec5.record()
    if s.get('tests'):
      _,_,j,_=run(rec,rep); rep.update(tests_for(drift5.conditions(),[k for k,v in j.items() if v!='合格']))
    spec,conds,judge,extra=run(rec,rep)
    out.append({"no":i,"who":s['who'],"what":s['what'],"ops":ops,"spec":spec,"judge":judge,"conds":conds,"extra":extra,"rep":copy.deepcopy(rep)})
  use(data5.D)
  return out
if __name__=='__main__':
  for r in simulate():
    bad=[(x['status'],x['from'],x['to']) for x in r['spec'] if x['status']!='合格']; tb={k:v for k,v in r['judge'].items() if v!='合格'}
    print(r['no'],'|',bad,'|',tb,'|','満たす' if not bad and not tb and not r['extra'] else '満たさない')
