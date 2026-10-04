# 移し終える条件を数える（ずれ ・ テストの欠け）。mig6 を外から当てる
import json,glob,os,sys
H=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,os.path.join(H,'..','..'))
import concrete7,mig6
D={}
for f in sorted(glob.glob(os.path.join(H,'decls','*.json'))):
  d=json.load(open(f,encoding='utf-8')); D[d['id']]=d
concrete7.use(D)
drift_n=sum(1 for x in concrete7.checks() if x['status'] in ('ずれ','対応の欠け'))
r=mig6.count(json.load(open(os.path.join(H,'spec','migration.json'),encoding='utf-8')))
print(json.dumps(dict(r,drift=drift_n),ensure_ascii=False,indent=1))
