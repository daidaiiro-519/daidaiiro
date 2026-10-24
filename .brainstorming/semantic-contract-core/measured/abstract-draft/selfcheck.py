import json,sys,os
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012
D=os.path.dirname(os.path.abspath(__file__))
body=json.load(open(os.path.join(D,'abstract.json')))
conc=json.load(open(os.path.join(D,'concrete.json')))
ann={"$schema":"https://json-schema.org/draft/2020-12/schema","$id":"urn:semantic-agent-schema:base:annotations:v1","$defs":{"relationship":{"type":"object","required":["to"],"properties":{"to":{"type":"string"}}}}}
def res(o): return Resource.from_contents(o,default_specification=DRAFT202012)
reg=Registry().with_resources([(body['$id'],res(body)),(ann['$id'],res(ann))])
v=Draft202012Validator(body,registry=reg)
for name,doc in [('本体自身',body),('ACDR の Concrete',conc)]:
    errs=sorted(v.iter_errors(doc),key=lambda e:list(e.absolute_path))
    print(f'== {name}: {len(errs)} 件')
    for e in errs[:40]: print('  /'+'/'.join(map(str,e.absolute_path)),'|',e.message[:110])
cv=Draft202012Validator(conc,registry=reg)
for k,doc in json.load(open(os.path.join(D,'instances.json'))).items():
    errs=list(cv.iter_errors(doc)); print(f'== インスタンス {k}: {len(errs)} 件',[e.message[:80] for e in errs[:3]])
bad=dict(json.load(open(os.path.join(D,'instances.json')))['I3']); bad.pop('docs')
print('== わざと docs を消した accepted:',[e.message[:60] for e in cv.iter_errors(bad)])
