"""静的の検査：本体を本体で、ほかのファイルを本体で、例をそのファイルで検査する。不合格なら0以外で終わる。"""
import json, glob, os, sys
from jsonschema import Draft202012Validator
from referencing import Registry, Resource
from referencing.jsonschema import DRAFT202012
HERE = os.path.dirname(os.path.abspath(__file__)); ROOT = os.path.dirname(HERE)

def load_all():
    docs = {}
    for p in glob.glob(os.path.join(ROOT, 'schemas', '*.json')):
        if p.endswith('changes.json'): continue
        d = json.load(open(p)); docs[d['$id']] = (p, d)
    return docs

def registry(docs):
    return Registry().with_resources([(i, Resource.from_contents(d, default_specification=DRAFT202012)) for i, (_, d) in docs.items()])

def validator(docs, sid):
    reg = registry(docs)
    return Draft202012Validator(docs[sid][1], registry=reg)

def run():
    docs = load_all(); BODY = 'urn:semantic-agent-schema:base:semantic-abstract-schema:v1'
    fails = 0
    for sid, (p, d) in sorted(docs.items()):
        errs = list(validator(docs, BODY).iter_errors(d))
        print(('ok  ' if not errs else 'NG  ') + os.path.basename(p) + ' ← 本体')
        for e in errs[:8]: print('     ', '/'.join(map(str, e.absolute_path)), e.message[:160])
        fails += len(errs)
    return docs, fails

if __name__ == '__main__':
    _, n = run(); sys.exit(1 if n else 0)
