# シミュレーション：承認済みの宣言を JSON Patch（RFC 6902）で1手ずつ書き換え、そのたびに道具の2つの検査を流す
import copy,json
import data3, drift, spec_drift
def baseline():
  """見本の13件のずれを直した、承認済みの宣言"""
  B=copy.deepcopy(data3.D)
  T=B['BC-1']['ubiquitous_language']['terms']
  T+= [{"id":"TERM-17","word":"開始時刻","kind":"状態","definition":"調理枠が始まる時刻","avoid":[]},
       {"id":"TERM-18","word":"空き","kind":"状態","definition":"調理枠でまだ受けられる調理の量","avoid":[]},
       {"id":"TERM-19","word":"調理の空きが足りない","kind":"拒否の理由","definition":"確保したい量が、調理枠の空きより多い","avoid":[]},
       {"id":"TERM-20","word":"調理の量","kind":"値オブジェクト","definition":"店舗が一定の時間に調理する量","avoid":[]},
       {"id":"TERM-21","word":"承認番号","kind":"値オブジェクト","definition":"決済代行が支払いを承認したときに返す番号","avoid":[]}]
  B['VO-5']={"kind":"value_object","id":"VO-5","header":{"name":"TERM-20","context":"BC-1"},"components":[{"id":"CMP-1","name":"量","kind":"数","digits":"整数","unit":"食","invariants":[{"id":"INV-1","condition":{"target":"CMP-1","op":"ge","value":0},"impossible":"-1食"},{"id":"INV-2","condition":{"target":"CMP-1","op":"le","value":999},"impossible":"1000食"}]}],"operations":[]}
  A2=B['AGG-2']; A2['structure']['state'][0]['name']='TERM-17'; A2['structure']['state'][1].update(name='TERM-18',type='VO-5')
  for c in A2['commands']: c['args'][0].update(name='TERM-20',type='VO-5')
  A2['commands'][0]['preconditions'][0]['reject']='TERM-19'
  A2['commands'][1]['postconditions'][0]['condition']={"target":"ST-2","op":"ge","value":"ARG-1"}
  B['DS-1']['operations'][0]['inputs'][0]['type']='VO-5'; B['DS-1']['operations'][0]['inputs'][1]['type']='VO-5'
  B['SD-2']['business_logic']['needs_tracking']=False
  A1=B['AGG-1']; A1['structure']['state'].append({"id":"ST-5","name":"TERM-21","type":"決済代行の番号","multiplicity":"0..1"})
  A1['commands'][0]['args'].append({"id":"ARG-2","name":"TERM-21","type":"決済代行の番号"})
  A1['commands'][0]['postconditions'].append({"id":"POST-3","condition":{"target":"ST-5","op":"eq","value":"ARG-2"}})
  U=B['UC-1']; U['preconditions'].append({"id":"PRE-3","text":"注文は下書きのまま、確定を待っている","established_by":"UC-2","ensures":["AGG-1.CMD-1.PRE-1"]})
  [g for g in U['guarantees']['success'] if g['id']=='SG-3'][0]['established_by']=['AGG-1.CMD-1.POST-3']
  return B
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
      if isinstance(par,list): o['_old']=par.pop(int(k))
      else: o['_old']=par.pop(k)
def run(state,rec,rep):
  drift.D=state; spec_drift.D=state
  spec=spec_drift.checks(rec)
  conds=drift.conditions()
  report={"schema":"report.schema.json","producer":"cargo test（来店前注文）","spec_revision":"","results":list(rep.values())}
  judge,extra=drift.judge(conds,report)
  return spec,conds,judge,extra
def record():
  return {f'{a}→{r}':spec_drift.fp(spec_drift.expect(r)) for a,f,r in spec_drift.links() if f!='invokes'}
def tests_for(conds,ids=None):
  return {c['id']:{"condition":c['id'],"hash":c['hash'],"test":"tests/"+c['id'].lower().replace('.','_')+".rs","level":c['required_level'],"result":"pass"} for c in conds if ids is None or c['id'] in ids}
STEPS=[
 {"who":"人（業務エキスパート）","what":"業務が変わり、1回に確保できる調理の量に上限ができた。集約「調理枠」の「確保する」に事前条件を足す",
  "ops":[{"op":"add","path":"/BC-1/ubiquitous_language/terms/-","value":{"id":"TERM-22","word":"一度に確保できる量を超えている","kind":"拒否の理由","definition":"1回で確保したい量が、上限の10食を超えている","avoid":[]}},
         {"op":"add","path":"/AGG-2/commands/0/preconditions/-","value":{"id":"PRE-2","condition":{"target":"ARG-1","op":"le","value":10},"reject":"TERM-22","example":"11食を1回で確保する"}}]},
 {"who":"AI","what":"道具が出した「対応の欠け」を受けて、ユースケースに拡張 3b を足し、新しい拒否を扱う",
  "ops":[{"op":"add","path":"/UC-1/scenario/steps/2/extensions/-","value":{"id":"EXT-4","label":"3b","name":"一度に確保できる量を超える","condition_kind":"確認の失敗","condition":"注文の量が、一度に確保できる量を超えた：","ending":"失敗","handles":["AGG-2.CMD-1.PRE-2"],"steps":[{"no":"3b1","name":"量を超えたと知らせる","actor":"システム","kind":"相互作用","to":"顧客","text":"システムは、顧客に一度に受けられる量を超えたことを知らせる","extensions":[],"variations":[]}]}}]},
 {"who":"人（承認者）","what":"PR を承認する。道具が、参照先の項目のハッシュ値を「確かめた時点」として記録する","approve":True,"ops":[]},
 {"who":"AI","what":"道具が出した「欠け」を受けて、テストを書く","tests":True,"ops":[]},
 {"who":"人（業務エキスパート）","what":"業務が変わり、調理枠は予備の1食を残すことになった。「確保する」の事前条件1を書き換える",
  "ops":[{"op":"replace","path":"/AGG-2/commands/0/preconditions/0/condition/value","value":"ARG-1 と予備の1食の和"}]},
 {"who":"人（承認者）","what":"確かめ直しになった拡張 3a を読み、条件の文を直して PR を承認する","approve":True,
  "ops":[{"op":"replace","path":"/UC-1/scenario/steps/2/extensions/0/condition","value":"希望の時間帯に、予備を除いた調理の空きがなかった："}]},
 {"who":"AI","what":"道具が出した「古い」を受けて、テストを直す","tests":True,"ops":[]},
 {"who":"AI","what":"手順を整理しようとして、拡張 4a の手順 4a1「調理枠を戻す」を消す",
  "ops":[{"op":"remove","path":"/UC-1/scenario/steps/3/extensions/0/steps/0"}]},
 {"who":"人（承認者）","what":"「対応の欠け」があるので PR を差し戻し、手順を元に戻す",
  "ops":[{"op":"add","path":"/UC-1/scenario/steps/3/extensions/0/steps/0","value":"__restore__"}]},
]
def simulate():
  state=baseline()
  drift.D=state; spec_drift.D=state
  rec=record(); conds=drift.conditions(); rep=tests_for(conds)
  out=[]
  spec,conds,judge,extra=run(state,rec,rep)
  out.append({"no":0,"who":"","what":"承認済みの宣言と、それを満たすテスト。ここから始める","ops":[],"spec":spec,"judge":judge,"conds":conds,"extra":extra,"rep":copy.deepcopy(rep)})
  removed=None
  for i,s in enumerate(STEPS,1):
    ops=copy.deepcopy(s['ops'])
    for o in ops:
      if o.get('value')=='__restore__': o['value']=removed
    apply(state,ops)
    for o in ops:
      if '_old' in o: removed=o.pop('_old'); o['removed']=removed
    drift.D=state; spec_drift.D=state
    if s.get('approve'): rec=record()
    if s.get('tests'):
      conds=drift.conditions(); _,_,j,_=run(state,rec,rep)
      rep.update(tests_for(conds,[k for k,v in j.items() if v!='合格']))
    spec,conds,judge,extra=run(state,rec,rep)
    out.append({"no":i,"who":s['who'],"what":s['what'],"ops":ops,"spec":spec,"judge":judge,"conds":conds,"extra":extra,"rep":copy.deepcopy(rep)})
  drift.D=data3.D; spec_drift.D=data3.D
  return out
if __name__=='__main__':
  for r in simulate():
    bad=[x for x in r['spec'] if x['status']!='合格']; tb={k:v for k,v in r['judge'].items() if v!='合格'}
    ok=not bad and not tb and not r['extra']
    print(r['no'],r['what'][:30],'| 宣言どうし',[(x['status'],x['from'],x['to']) for x in bad],'| テスト',tb,'| 終了基準','満たす' if ok else '満たさない')
