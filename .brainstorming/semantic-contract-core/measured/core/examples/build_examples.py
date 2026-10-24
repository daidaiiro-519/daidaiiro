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

c['x-delegates']['acdr-inspect']['role'] = 'ACDR の変更の印の検査（基盤の変更のまとまりを受け取る）'
for w in P['docs']['x-prompt']['write']:
    if w['id'] == 'why-per-change':
        w['test'] = '基盤の変更のまとまり（最後に確定した姿からの差分）の変更ごとに、docs の marks に why があるかを見る'
        w['agent'] = 'acdr-inspect'
log('E06', 'acdr-inspect は Git を読まず、基盤の変更のまとまりを受け取って、変更ごとに why があるかを見る', 'ACDR は基盤の差分を広げる Concrete なので、変更の仕組みに直接依存しない', '利用者「少なくとも基盤の変更差分を使用するから直接的に依存しない」')
log('E02', '（取り下げ）docs と acdr-inspect は外さない', 'ACDR は基盤の差分を、より分かりやすく見せることに特化した Concrete。docs（変更1件ごとのどこ ・ 変更前 ・ なぜ）と acdr-inspect はその拡張の部分で、基盤と重ならない', 'user-instructions「ACDRはこれをより拡張して見やすくしたもの」 ・ 利用者「これははずさないよ？」')

mark = {"type": "object", "required": ["find", "why"], "additionalProperties": False, "properties": {
  "find": {"type": "string", "minLength": 1}, "before": {"type": "string"}, "why": {"type": "string", "minLength": 1}}}
P['docs'].update({"description": "変えた文書と、変更1件ごとのどこ ・ 変更前 ・ なぜ", "items": {"type": "object", "required": ["key", "tab", "file"], "additionalProperties": False, "properties": {
  "key": {"type": "string", "pattern": "^[a-z0-9-]+$"}, "tab": {"type": "string", "minLength": 1}, "file": {"type": "string", "minLength": 1},
  "rev": {"type": "string"}, "before": {"type": "string"}, "marks": {"type": "array", "items": mark}}}})
log('E05', 'docs の項目の形を、いまの acdr Skill のスキーマ（key ・ tab ・ file ・ rev ・ before ・ marks の3つ組）に合わせる', '下書きは docs の形を省いていた', '.claude/skills/acdr/references/acdr.schema.json')

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
    v = copy.deepcopy(v)
    v['docs'] = [{'key': 'tool', 'tab': '画面の AI の起動', 'file': 'brainstorming/tool/agent.rs', 'rev': 'HEAD',
                  'marks': [{'find': 'fn spawn_agent(cfg: &AgentCfg)', 'before': 'fn spawn_agent(cfg: &AgentCfg, allowed: &[Tool])', 'why': '許可の一覧を渡さず、各 AI ツールの設定ファイルに合わせる。'}]}]
    v = {'$schema': v.pop('$schema'), 'kind': v.pop('kind'), 'id': '0197-ai-permissions', **v}
    out[k] = v
json.dump(out, open(os.path.join(HERE, 'acdr.instances.json'), 'w'), ensure_ascii=False, indent=2)
json.dump(E, open(os.path.join(HERE, 'acdr.changes.json'), 'w'), ensure_ascii=False, indent=1)
print(len(E), '件')
