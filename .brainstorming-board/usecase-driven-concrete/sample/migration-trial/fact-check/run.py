# 移した宣言に、sample の道具（concrete7：スキーマの注釈だけを読む）を外から当てる
import json,glob,os,sys,collections
H=os.path.dirname(os.path.abspath(__file__)); sys.path.insert(0,os.path.join(H,'..','..'))
import concrete7
D={}
for f in sorted(glob.glob(os.path.join(H,'decls','*.json'))):
  d=json.load(open(f,encoding='utf-8')); D[d['id']]=d
concrete7.use(D)
R=concrete7.checks()
print(collections.Counter(x['status'] for x in R))
for x in R:
  if x['status'] not in ('合格','確かめ直し'): print(' ',x['status'],x['check'],x['from'],x['to'],x['text'])
try:
  C=concrete7.conditions(); print('テスト条件',len(C))
  for c in C: print('  ',c['id'],c['kind'],c['required_level'],c['label'])
except Exception as e: print('テスト条件が取り出せない:',type(e).__name__,e)
