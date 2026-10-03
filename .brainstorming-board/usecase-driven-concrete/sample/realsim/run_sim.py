# 実際のリポジトリで、人と AI の手を順に打ち、手ごとにコミットして CI を流す。結果は build/sim.json に残す（記録用）
import subprocess,json,os,re
R=os.path.dirname(os.path.abspath(__file__)); os.chdir(R)
ENV=dict(os.environ,CONCRETE_ROOT=R,GIT_AUTHOR_DATE='2026-10-03T12:00:00',GIT_COMMITTER_DATE='2026-10-03T12:00:00')
def sh(c,**k): return subprocess.run(c,shell=True,capture_output=True,text=True,env=ENV,**k)
def concrete(*a): return sh('python3 tool/concrete.py '+' '.join(a))
def conds(): return {l.split()[0]:l.split()[1] for l in concrete('conditions').stdout.splitlines()}
def patch(ops):
  json.dump(ops,open('build/ops.json','w'),ensure_ascii=False); r=concrete('patch','build/ops.json'); assert r.returncode==0,r.stdout+r.stderr
def edit(path,old,new):
  s=open(path).read(); assert old in s,(path,old); open(path,'w').write(s.replace(old,new,1))
def add(path,text): open(path,'a').write(text)
def set_hash(cid,new):
  for f in ['tests/test_domain.py','systemtest/uc1_test.go']:
    s=open(f).read()
    s=re.sub(r'(trace\((?:t, )?"%s",\s*")[0-9a-f]{8}'%re.escape(cid),r'\g<1>'+new,s)
    s=re.sub(r'("%s":")[0-9a-f]{8}'%re.escape(cid),r'\g<1>'+new,s)
    open(f,'w').write(s)
LOG=[]
def step(who,what,act,files_note=''):
  before=sh('git rev-parse HEAD').stdout.strip() or None
  act()
  sh('git add -A'); sh(f'git -c user.name="{who}" -c user.email=x@example.com commit -q -m "{what}"')
  stat=sh(f'git diff --stat {before} HEAD').stdout if before else sh('git show --stat --format= HEAD').stdout
  diff=sh(f'git diff {before} HEAD -- . ":(exclude)spec/confirmed.json"').stdout if before else ''
  ci=sh('./ci.sh'); status=sh('git status --short --ignored').stdout
  tracked_build=sh('git ls-files build').stdout
  LOG.append({"no":len(LOG),"who":who,"what":what,"commit":sh('git rev-parse --short HEAD').stdout.strip(),"stat":stat,"diff":diff,"ci":ci.stdout,"ci_ok":ci.returncode==0,"ignored":status,"tracked_build":tracked_build})
  print(f'── {len(LOG)-1} {who}：{what}  CI {"通った" if ci.returncode==0 else "落ちた"}')
import shutil
for x in ['spec','src','tests','systemtest','tool','impl']:
  shutil.rmtree(x,ignore_errors=True); shutil.copytree('base/'+x,x)
for x in ['ci.sh','.gitignore']: shutil.copy('base/'+x,x)
sh('rm -rf .git build'); sh('git init -q'); os.makedirs('build',exist_ok=True)
open('.git/info/exclude','a').write('base/\nrun_sim.py\n')
step('人（承認者）','承認済みの宣言と、実装と、テスト。宣言どうしの対応を承認して記録する',lambda: concrete('approve'))
def s1():
  patch([{"op":"add","path":"/BC-1/ubiquitous_language/terms/-","value":{"id":"TERM-51","word":"一度に確保できる量を超えている","kind":"拒否の理由","definition":"1回で確保したい量が、上限の10食を超えている","avoid":[]}},
         {"op":"add","path":"/AGG-2/commands/0/preconditions/-","value":{"id":"PRE-2","condition":{"target":"ARG-1","op":"le","value":10},"reject":"TERM-51"}}])
step('人（業務エキスパート）','業務の変更：1回に確保できる調理の量は10食まで。調理枠の「確保する」に事前条件を足す',s1)
def s2():
  patch([{"op":"add","path":"/BC-1/ubiquitous_language/terms/-","value":{"id":"TERM-52","word":"一度に受けられる量","kind":"情報の別名","definition":"1回の注文で受けられる調理の量の上限","avoid":[]}},
         {"op":"add","path":"/UC-1/scenario/steps/3/extensions/-","value":{"id":"EXT-4","condition_kind":"コマンドの拒否","handles":["AGG-2.CMD-1.PRE-2"],"ending":"失敗","steps":[
           {"id":"EXT-4.S-1","kind":"内部の状態変化","actor":"システム","invokes":"AGG-1.CMD-2","keeps":["MG-3"],"extensions":[]},
           {"id":"EXT-4.S-2","kind":"相互作用","actor":"システム","to":"顧客","data":["TERM-52"],"verb":"TERM-50","extensions":[]}]}}])
  c=conds()
  edit('src/order/domain.py',"    if not (self.free.value>=amount.value): raise Rejected('調理の空きが足りない')",
       "    if not (self.free.value>=amount.value): raise Rejected('調理の空きが足りない')\n    if not (amount.value<=10): raise Rejected('一度に確保できる量を超えている')")
  edit('src/order/usecase.py',"        if e.reason!='調理の空きが足りない': raise",
       "        if e.reason=='一度に確保できる量を超えている':\n          o.record_progress(self.clock()); self.ui.notify('一度に受けられる量'); return Result(False,'量を超えた')\n        if e.reason!='調理の空きが足りない': raise")
  add('tests/test_domain.py',f'''def test_reserve_rejects_over_limit():
  trace("AGG-2.CMD-1.PRE-2","{c["AGG-2.CMD-1.PRE-2"]}","component")
  with pytest.raises(Rejected) as e: slot(20).reserve(CookAmount(11))
  assert e.value.reason=='一度に確保できる量を超えている'
''')
  add('systemtest/uc1_test.go',f'''
func TestOverLimit(t *testing.T) {{
	trace(t, "UC-1.EXT-4", "{c["UC-1.EXT-4"]}", "system")
	o := run(t, `{{"lines":[{{"product":"P-1","qty":11,"price":500}}],"catalog":{{"P-1":500}},"slots":[{{"id":"12:00","start":"2026-10-03T12:00","free":20}}],"payment":"approve"}}`)
	if o.OK || o.Status != "下書き" {{
		t.Fatalf("成功時保証は成り立ってはいけない: %+v", o)
	}}
	minimalGuarantees(t, o, "12:00", 20)
}}
''')
step('AI','拡張 4b を足して新しい拒否を扱い、実装とテスト（Python ・ Go）を足す',s2)
step('人（承認者）','PR を承認する。宣言どうしの対応を記録する',lambda: concrete('approve'))
def s4():
  patch([{"op":"replace","path":"/AGG-2/commands/0/preconditions/0/condition/value","value":{"call":"VO-5.OP-1","args":["ARG-1",1]}}])
step('人（業務エキスパート）','業務の変更：調理枠は予備の1食を残す。「確保する」の事前条件1の比較する値を書き換える',s4)
def s5():
  c=conds()
  edit('src/order/domain.py',"    if not (self.free.value>=amount.value): raise Rejected('調理の空きが足りない')",
       "    if not (self.free.value>=amount.value+1): raise Rejected('調理の空きが足りない')")
  edit('tests/test_domain.py',"  with pytest.raises(Rejected) as e: slot(0).reserve(CookAmount(1))\n  assert e.value.reason=='調理の空きが足りない'",
       "  with pytest.raises(Rejected) as e: slot(1).reserve(CookAmount(1))\n  assert e.value.reason=='調理の空きが足りない'")
  set_hash('AGG-2.CMD-1.PRE-1',c['AGG-2.CMD-1.PRE-1']); set_hash('UC-1.EXT-2',c['UC-1.EXT-2'])
step('AI','concrete match の「古い」を受けて、予備の1食を実装し、テストとハッシュ値を直す',s5)
step('人（承認者）','確かめ直しになった拡張 4a を読み、扱う拒否が今も正しいことを確かめて承認する',lambda: concrete('approve'))
def s7():
  patch([{"op":"remove","path":"/UC-1/scenario/steps/4/extensions/0/steps/0"}])
  edit('src/order/usecase.py',"      slot.release(total); o.record_progress(self.clock())","      o.record_progress(self.clock())")
step('AI','手順を整理しようとして、拡張 5a の手順 5a1（調理枠を戻す）を宣言と実装から消す',s7)
step('人（承認者）','CI が落ちたので PR を差し戻し、変更を戻す',lambda: sh('git revert --no-commit HEAD'))
json.dump(LOG,open('build/sim.json','w'),ensure_ascii=False,indent=1)
