import json, sys, copy, os, importlib.metadata as md
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from referencing.exceptions import NoSuchResource
from referencing.jsonschema import DRAFT202012
T=os.path.dirname(os.path.abspath(__file__))
META=sys.argv[1]; BODY=sys.argv[2]
L=lambda p: json.load(open(p))
def no_net(uri): raise NoSuchResource(ref=uri)
res={}
for f in os.listdir(META):
    d=L(os.path.join(META,f)); res[d['$id']]=d
body=L(BODY); ann=L(T+'/annotations.stub.json'); gl=L(T+'/glossary.json')
res[body['$id']]=body; res[ann['$id']]=ann
res[gl['$id']]=gl
reg=Registry(retrieve=no_net).with_resources([(k,Resource(contents=v,specification=DRAFT202012)) for k,v in res.items()])
print('python-jsonschema',md.version('jsonschema'),'referencing',md.version('referencing'),'（ネットワークに取りに行かせない Registry）')
meta=res['https://json-schema.org/draft/2020-12/schema']
errs=list(Draft202012Validator(meta,registry=reg).iter_errors(body))
print('T1 本体を 2020-12 のメタスキーマで検査：errors=',len(errs),[e.message for e in errs][:5])
V=Draft202012Validator(body,registry=reg)
good=L(T+'/good.json')
fails=0
def run(name,doc,expect):
    global fails
    e=list(V.iter_errors(doc)); mark='合' if not e else '否'
    ok=mark==expect[0]; fails+= (not ok)
    print(f"{name}\t期待={expect}\t結果={mark}\t{'一致' if ok else '★不一致'}")
    for x in e[:3]: print('    ', '/'+'/'.join(map(str,x.absolute_path)), '|', x.message[:120])
run('G0 合う例',good,'合')
def mut(f):
    d=copy.deepcopy(good); f(d); return d
K=lambda d: d['properties']['kind']['x-prompt']['write']
cases=[
 ('B1 rule の by 抜け', lambda d: K(d)[0].pop('by'),'否'),
 ('B2 by が ai の rule で agent 抜け', lambda d: K(d)[1].pop('agent'),'否'),
 ('B3 x-decided-by に human 以外（ai）', lambda d: d['properties']['name'].update({'x-decided-by':'ai'}),'否'),
 ('B4 by が tool（3つ以外）', lambda d: K(d)[0].update(by='tool'),'否'),
 ('B5 by が core（3つ以外）', lambda d: K(d)[0].update(by='core'),'否'),
 ('B6 by が human で reads も agent も無し', lambda d: None,'合'),
 ('B7 x-extensions の定義の rule で by:ai の agent 抜け', lambda d: d['x-extensions']['x-unit']['prompt']['write'][0].pop('agent'),'否'),
 ('B8 x-extensions の定義に by を書く', lambda d: d['x-extensions']['x-unit'].update(by='ai'),'否'),
 ('B9 x-delegates の concrete が URN でない（コマンド）', lambda d: d['x-delegates']['lint'].update(concrete='demo-lint --json'),'否'),
 ('B10 x-delegates の値に x- の項目', lambda d: d['x-delegates']['lint'].update({'x-model':'m'}),'否'),
 ('B11 rule に validator（外した項目）', lambda d: K(d)[1].update(validator='judge'),'否'),
 ('B18 x-extensions の定義に x-prompt（x- を付けた項目）', lambda d: d['x-extensions']['x-unit'].update({'x-prompt': d['x-extensions']['x-unit']['prompt']}),'否'),
 ('B19 x-derived-from に reads（外した項目）', lambda d: d['properties']['level']['x-derived-from'].update(reads=['/properties/risk']),'否'),
 ('B12 x-derived-from に expect（外した項目）', lambda d: d['properties']['level']['x-derived-from'].update(expect=[]),'否'),
 ('B13 $id の NID が semantic-agent-schema でない', lambda d: d.update({'$id':'urn:qa-advisor:record:v1'}),'否'),
 ('B14 x-relationship の to が URN でない', lambda d: d['properties']['owner']['x-relationship'].update(to='person'),'否'),
 ('B15 x-published が文字列', lambda d: d['properties']['name'].update({'x-published':'yes'}),'否'),
 ('B16 bad だけで good なし', lambda d: K(d)[2].pop('good'),'否'),
 ('B17 x-instance-path があり kind が無い', lambda d: d['properties'].pop('kind'),'否'),
]
for n,f,e in cases: run(n,mut(f),e)
print('不一致',fails)
# 本体の中の rule が rule の形を守るか
pv=Draft202012Validator({'$ref':body['$id']+'#/$defs/rule'},registry=reg)
n=0; bad=0; ai=[]
def walk(o,path):
    global n,bad
    if isinstance(o,dict):
        for k,v in o.items():
            if k=='x-prompt' and isinstance(v,dict) and 'write' in v:
                for i,r in enumerate(v['write']):
                    n+=1
                    for e in pv.iter_errors(r): bad+=1; print('  形の違反',path+'/x-prompt/write/'+str(i),e.message)
                    if r.get('by')=='ai': ai.append((r['id'],r.get('agent')))
            walk(v,path+'/'+k)
    elif isinstance(o,list):
        for i,v in enumerate(o): walk(v,path+'/'+str(i))
walk(body,'')
pp=Draft202012Validator({'$ref':body['$id']+'#/$defs/prompt'},registry=reg)
pbad=0
def walkp(o,path):
    global pbad
    if isinstance(o,dict):
        for k,v in o.items():
            if k=='x-prompt' and isinstance(v,dict) and 'read' in v:
                for e in pp.iter_errors(v): pbad+=1; print('  x-prompt の違反',path,e.message)
            walkp(v,path+'/'+k)
    elif isinstance(o,list):
        for i,v in enumerate(o): walkp(v,path+'/'+str(i))
walkp(body,'')
print(f'本体の中の rule：{n}件、形の違反 {bad}件 ／ x-prompt の形の違反 {pbad}件 ／ by: ai の rule と agent：{ai}')
from collections import Counter
cnt=Counter()
def walkb(o):
    if isinstance(o,dict):
        if isinstance(o.get('by'),str) and 'test' in o: cnt[o['by']]+=1
        for v in o.values(): walkb(v)
    elif isinstance(o,list):
        for v in o: walkb(v)
walkb(body); print('by の内訳',dict(cnt))
# Concrete を本体で検査し、Concrete で instance を検査する
reg2=reg.with_resource(good['$id'],Resource(contents=good,specification=DRAFT202012))
CV=Draft202012Validator(good,registry=reg2)
for inst,exp in [({'$schema':good['$id'],'kind':'record','id':'1','name':'a','elapsed':3},'合'),({'$schema':good['$id'],'kind':'record','elapsed':-1},'否')]:
    e=list(CV.iter_errors(inst)); print('I instance',json.dumps(inst,ensure_ascii=False),'期待',exp,'結果','合' if not e else '否',[x.message for x in e])

# 本体が自分の rule（機械で見られるもの）に違反しないか
import re
CORE9={'x-instance-path','x-instance-label','x-delegates','x-extensions','x-prompt','x-relationship','x-derived-from','x-decided-by','x-published'}
viol=[]
URN=re.compile(r'^urn:semantic-agent-schema:[a-z0-9][a-z0-9-]*:[a-z0-9][a-z0-9-]*:v[0-9]+$')
own=body['$id'].split(':')[2]
if not URN.match(body['$id']) or own not in ('core',): viol.append(('owner-kind-version',body['$id']))
dele=set(body.get('x-delegates',{}))
def w2(o,path,inside_x=False):
    if isinstance(o,dict):
        for k,v in o.items():
            if k.startswith('x-') and k not in CORE9: viol.append(('core-keywords-closed',path+'/'+k))
            if isinstance(v,dict) and v.get('by')=='ai' and 'rule' in v and v.get('agent') not in dele|{'reviewer'}: viol.append(('rule-agent',path+'/'+k))
            w2(v,path+'/'+k)
    elif isinstance(o,list):
        for i,v in enumerate(o):
            if isinstance(v,dict) and v.get('by')=='ai' and 'rule' in v and v.get('agent') not in dele|{'reviewer'}: viol.append(('rule-agent',path+'/'+str(i)))
            w2(v,path+'/'+str(i))
w2(body,'')
ext=body['properties']['x-extensions']['additionalProperties']['properties']
viol+= [('no-x-inside-values','x-extensions/'+k) for k in ext if k.startswith('x-')]
viol+= [('no-x-inside-values','x-delegates/'+k) for k in body['properties']['x-delegates']['additionalProperties']['properties'] if k.startswith('x-')]
print('本体の自己検査（owner-kind-version ・ core-keywords-closed ・ rule-agent ・ 値の中の x-）：違反',len(viol),viol)
# prompt-per-unit：根の properties のうち、入れ物（x-prompt ・ properties ・ $defs）を除いた各項目
units=[k for k in body['properties'] if k not in ('x-prompt','properties','$defs')]
pv2=[k for k in units if not ('description' in body['properties'][k] and 'x-prompt' in body['properties'][k])]
print('本体の自己検査（prompt-per-unit）：書く単位',len(units),'件、違反',len(pv2),pv2)
