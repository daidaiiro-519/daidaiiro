# 移した宣言に、sample の検査（spec6）とテスト条件の取り出し（drift6）を外から当てる
import json,glob,os,sys,collections
H=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,os.path.join(H,'..','..'))
import spec6,drift6
D={}
for f in sorted(glob.glob(os.path.join(H,'decls','*.json'))):
  d=json.load(open(f,encoding='utf-8')); D[d['id']]=d
spec6.use(D); drift6.use(D)
R=spec6.checks()
print(collections.Counter(x['status'] for x in R))
for x in R:
  if x['status'] not in ('合格','確かめ直し'): print(' ',x['status'],x['check'],x['from'],x['to'],x['text'])
try:
  C=drift6.conditions(); print('テスト条件',len(C))
  for c in C: print('  ',c['id'],c['kind'],c['required_level'],c['label'])
except Exception as e: print('drift6.conditions が動かない:',type(e).__name__,e)
