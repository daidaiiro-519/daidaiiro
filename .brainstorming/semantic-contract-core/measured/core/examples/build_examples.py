"""ACDR の Concrete とインスタンスの例。abstract-draft の下書きに、核の決定を適用する。"""
import copy, json, os
HERE = os.path.dirname(os.path.abspath(__file__)); DRAFT = os.path.join(HERE, '..', '..', 'abstract-draft')
c = json.load(open(os.path.join(DRAFT, 'concrete.json')))
E = []
def log(i, what, why, source): E.append(dict(id=i, what=what, why=why, source=source))
P = c['properties']

P['id'] = {"type": "string", "pattern": "^[0-9]{4}-[a-z0-9]+(-[a-z0-9]+)*$", "readOnly": True, "description": "記録の id",
  "x-prompt": {"read": {"means": "番号と短い名詞句（英小文字）をつないだ、記録の id である。", "parse": "先頭の4桁が no と同じ。置き場所の {id} に入る。"},
               "write": [{"id": "id-starts-with-no", "rule": "id の先頭の4桁は no と同じである。", "why": "番号と id が食い違うと、置き場所と題がずれる。", "test": "id の先頭4桁と no を比べる。", "by": "machine"}]}}
c['x-instance-path'] = '.acdr/{id}/acdr.json'
c['required'] = ['$schema', 'kind', 'id'] + [k for k in c['required'] if k not in ('$schema', 'kind')]
log('E01', 'id 欄を足し、置き場所を .acdr/{id}/acdr.json にする', '1件は id と kind。{slug} が欄に無かった', 'board Q1 #35 ・ reviews2/adversarial 3-10 3-11 ・ ddd F15')

P.pop('docs'); c['x-delegates'].pop('acdr-inspect')
P['changes'] = {"type": "array", "items": {"type": "string", "pattern": "^[0-7][0-9A-HJKMNP-TV-Z]{25}$", "x-relationship": {"to": "urn:semantic-agent-schema:base:change:v1"}},
  "description": "この決定の変更の記録",
  "x-prompt": {"read": {"means": "この決定のために書いた変更の記録の id の並びである。", "shown": "ACDR のビューが、基盤の変更のまとまりを、決定の下に意図ごとに出す。"},
               "write": [{"id": "purpose-matches", "rule": "並べた変更の記録の目的の参照は、この記録（ACDR の番号）である。", "why": "ほかの目的の変更が混ざると、決定の差分を取り違える。", "test": "各変更の記録の purpose_ref が ACDR {no} かを見る。", "by": "machine"}]}}
for cond in c['allOf']:
    if cond['then'].get('required') == ['docs']:
        cond['then'] = {"required": ["changes"], "properties": {"changes": {"minItems": 1}}}
log('E02', 'docs と acdr-inspect（Git の差分を読む）を外し、変更の記録の id の並び changes を足す', 'ACDR は基盤の変更のまとまりを広げて見やすくしたもの。Git を自分で読まない', 'user-instructions「ACDRはこれをより拡張して見やすくしたもの」 ・ reviews2/adversarial 2-17 2-18')

body = c['$defs'].pop('review-body', None)
if body is not None:
    c['x-delegates']['reviewer']['answer'] = body
log('E03', 'reviewer の answer の $ref を中へ書き切る', '知らないキーワードの中の $ref は実装ごとに解決が違う（本体 S07）', 'reviews2/adversarial 3-13')

sup = P.get('supersedes')
if sup:
    sup.setdefault('x-relationship', {"to": c['$id']})
    if 'pattern' in sup: sup['pattern'] = "^[0-9]{4}-[a-z0-9]+(-[a-z0-9]+)*$"
log('E04', 'supersedes を x-relationship で ACDR を指す形にする', '指す先が在るかを基盤が検査する', 'reviews2/adversarial 3-11')

json.dump(c, open(os.path.join(HERE, 'acdr.concrete.json'), 'w'), ensure_ascii=False, indent=2)
ins = json.load(open(os.path.join(DRAFT, 'instances.json')))
out = {}
for k, v in ins.items():
    v = copy.deepcopy(v); v.pop('docs', None)
    v = {'$schema': v.pop('$schema'), 'kind': v.pop('kind'), 'id': '0197-ai-permissions', **v}
    v['changes'] = ['01JBA0C1Q4M6R2T9V5X7Y1A3C5']
    out[k] = v
json.dump(out, open(os.path.join(HERE, 'acdr.instances.json'), 'w'), ensure_ascii=False, indent=2)
json.dump(E, open(os.path.join(HERE, 'acdr.changes.json'), 'w'), ensure_ascii=False, indent=1)
print(len(E), '件')
