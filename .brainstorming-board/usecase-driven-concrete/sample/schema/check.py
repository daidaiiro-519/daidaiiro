# 宣言を種類ごとのスキーマで検査し、スキーマ自身を案内の契約（description と x-prompt）で検査する
import json,glob,os,sys
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
H=os.path.dirname(os.path.abspath(__file__))
S={os.path.basename(f):json.load(open(f,encoding='utf-8')) for f in glob.glob(os.path.join(H,'*.schema.json'))}
reg=Registry().with_resources([(k,Resource.from_contents(v)) for k,v in S.items()])
bad=0
for k,v in S.items(): Draft202012Validator.check_schema(v)
# 案内の契約：x-generates を持つスキーマは、最上位と $defs の項目に description と x-prompt（read ・ write）を持つ
meta=json.load(open(os.path.join(H,'..','..','..','..','.claude','skills','no-more-spaghetti','references','schema-meta.schema.json'),encoding='utf-8'))
for k,v in S.items():
  for e in Draft202012Validator(meta).iter_errors(v): bad+=1; print('案内',k,'/'.join(map(str,e.path)),e.message[:120])
  if k=='common.schema.json':
    for n,d in v['$defs'].items():
      if not (d.get('description') and d.get('x-prompt',{}).get('read') and d.get('x-prompt',{}).get('write')): bad+=1; print('案内',k,n)
dirs=sys.argv[1:] or [os.path.join(H,'..','decls')]
for d in dirs:
  for f in sorted(glob.glob(os.path.join(d,'*.json'))):
    x=json.load(open(f,encoding='utf-8')); s=S.get(x['kind']+'.schema.json')
    if s is None: bad+=1; print('種類が無い',f); continue
    for e in Draft202012Validator(s,registry=reg).iter_errors(x): bad+=1; print('形',os.path.relpath(f,H),'/'.join(map(str,e.absolute_path)),e.message[:160])
print('検出',bad); sys.exit(1 if bad else 0)
