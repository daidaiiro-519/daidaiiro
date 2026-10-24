"""abstract-draft/abstract.json に、2回目のレビューから決まる直しを適用して abstract.json を作る。
直しは S の一覧（id ・ 何を ・ なぜ ・ 出どころ）。手で abstract.json を直さない。"""
import copy, json, os
HERE = os.path.dirname(os.path.abspath(__file__))
src = json.load(open(os.path.join(HERE, '..', '..', 'abstract-draft', 'abstract.json')))
a = copy.deepcopy(src)
D = a['$defs']; X = a['x-extensions']; P = a['x-prompt']['write']
S = []
def log(i, what, why, source): S.append(dict(id=i, what=what, why=why, source=source))
def drop_rule(rules, rid):
    n = len(rules); rules[:] = [r for r in rules if r['id'] != rid]; assert len(rules) == n - 1, rid

# S01 入れ子の欄にも本体の形を掛ける（$defs/subschema で再帰）
D['subschema'] = {
  "type": "object",
  "description": "入れ子の欄（items ・ properties ・ allOf などの中）の形。x- の値の形を、根の欄と同じに検査する。説明と案内は求めない（葉には書かない）。キーワードの並びは allOf の中に置く（$defs/shape が欄として扱わないように）",
  "allOf": [{"properties": {
    "x-prompt": {"$ref": "#/$defs/prompt"}, "x-relationship": {"$ref": "#/$defs/relationship"},
    "x-derived-from": {"$ref": "#/$defs/derived"}, "x-decided-by": {"$ref": "#/$defs/decided"}, "x-published": {"$ref": "#/$defs/published"},
    "properties": {"type": "object", "additionalProperties": {"$ref": "#/$defs/subschema-or-bool"}},
    "items": {"$ref": "#/$defs/subschema-or-bool"},
    "prefixItems": {"type": "array", "items": {"$ref": "#/$defs/subschema-or-bool"}},
    "allOf": {"type": "array", "items": {"$ref": "#/$defs/subschema-or-bool"}},
    "anyOf": {"type": "array", "items": {"$ref": "#/$defs/subschema-or-bool"}},
    "oneOf": {"type": "array", "items": {"$ref": "#/$defs/subschema-or-bool"}},
    "additionalProperties": {"$ref": "#/$defs/subschema-or-bool"}
  }}]
}
D['subschema-or-bool'] = {"anyOf": [{"type": "boolean"}, {"$ref": "#/$defs/subschema"}]}
field = D['field']; field['allOf'] = [{"$ref": "#/$defs/subschema"}]
for k in ['x-prompt', 'x-relationship', 'x-derived-from', 'x-decided-by', 'x-published']: field['properties'].pop(k)
drop_rule(P, 'nested-fields')
log('S01', '$defs/subschema を足し、$defs/field から再帰で掛ける。行 nested-fields を外す',
    '入れ子の欄の x- が検査されず、壊れた x-prompt が通った', 'reviews2/adversarial 3-1')

# S02 値だけで決まる行を JSON Schema へ
dlg = D['delegate']
dlg['allOf'] = [{"if": {"required": ["run"], "properties": {"run": {"required": ["argv"]}}}, "then": {"not": {"required": ["answer"]}}}]
dlg['properties']['answer']['x-prompt']['write'] = []
argv = dlg['properties']['run']['properties']['argv']; argv['items']['pattern'] = '^[^|;&`$<>]*$'
dlg['properties']['run']['x-prompt']['write'] = []
r = D['rule']; cond = {"if": r.pop('if'), "then": r.pop('then')}
r['allOf'] = [cond, {"if": {"properties": {"by": {"const": "human"}}, "required": ["by"]}, "then": {"not": {"required": ["agent"]}}}]
log('S02', 'answer-for-ai ・ argv-no-shell を if ・ then と pattern にし、by が human なら agent を書かない を足す',
    '値だけで決まるのに行に残り、壊した入力が通った', 'reviews2/adversarial 3-4 ・ 利用者「フルに活用したほうがいい」')

# S03 JSON Pointer の pattern
D['json-pointer'] = {"type": "string", "pattern": "^(/([^~/]|~[01])*)*$", "description": "JSON Pointer（RFC 6901）"}
r['properties']['reads']['items'] = {"$ref": "#/$defs/json-pointer"}
log('S03', '$defs/json-pointer を足し、reads に使う', 'format は注釈なので検査されず「why のこと」が通った', 'reviews2/adversarial 3-3')

# S04 JSON Schema と重ねた行を外す。write は0件でもよい
pr = D['prompt']; pr['required'] = ['read']; pr['properties']['write']['minItems'] = 0
pr['properties']['read']['x-prompt']['write'] = []
drop_rule(X['x-prompt']['prompt']['write'], 'read-and-write')
X['x-prompt']['prompt']['write'] = []
log('S04', 'write を任意で0件可にし、means-first ・ read-and-write を外す', 'required と同じことを行に重ね、値だけで決まる欄にも行を書かされた', 'reviews2/adversarial 3-5')

# S05 は採らない：fields の除外（$schema ・ properties ・ $defs ・ x-）は、本体が自分を検査するために要る
#   （本体の properties のキーは Concrete の根のキー x- ・ properties ・ $defs である）。レビューの 3-7 は採らない。
log('S05', '（採らない）$defs/fields の除外はこの会話の決定のまま', '本体の自己検査で、本体の properties のキー（x- ・ properties ・ $defs）が欄として扱われ、説明と案内を求められて落ちた（実測）',
    'decisions「$defs/fields は $schema ・ 入れ物 ・ x- のキーを除く」 ・ reviews2/adversarial 3-7 は採らない')

# S06 no-abstract-names の範囲
for w in X['x-extensions']['prompt']['write']:
    if w['id'] == 'no-abstract-names':
        w['rule'] = '本体の外のスキーマでは、Abstract の9つの x- と同じ名前を使わない。'
        w['test'] = '本体以外のスキーマの x-extensions の名前が、本体の9つと重なっていないかを見る。'
log('S06', 'no-abstract-names を本体の外に限る', '本体は自分を検査するので、本体に当てると violated になった', 'reviews2/adversarial 3-6')

# S07 x-delegates の answer の中で $ref を使わない
dlg['properties']['answer']['x-prompt']['write'] = [{
  "id": "answer-inline", "rule": "答えの形は answer の中に書き切り、$ref を使わない。",
  "why": "知らないキーワードの中の $ref は、JSON Schema の実装ごとに解決が違う。",
  "test": "answer の中に $ref が無いかを見る。", "by": "machine"}]
log('S07', 'answer の中で $ref を使わない行を足す', 'x-extensions と同じ理由が x-delegates にもあった', 'reviews2/adversarial 3-13')

# S08 本体の部品にも説明と案内
ext = D['extensions']['additionalProperties']['properties']
def pp(means, rid, rule, why, test, by='ai'):
    w = {"id": rid, "rule": rule, "why": why, "test": test, "by": by}
    if by == 'ai': w['agent'] = 'judge'
    return {"read": {"means": means}, "write": [w]}
ext['schema']['x-prompt'] = pp('x- の値の形（JSON Schema）である。', 'shape-only', '値の形だけを書き、意味は description と prompt に書く。', '形と意味が混ざると、どちらを直せばよいかが決まらない。', 'schema に意味の説明が入っていないかを見る。')
ext['description'].update({"description": "x- の説明", "x-prompt": pp('この x- が何を表すかの1文である。', 'what-it-means', '何を宣言するキーワードかを書く。', '説明が無いと、作者が使いどころを判断できない。', '説明がキーワードの値と合っているかを見る。')})
ext['prompt']['x-prompt'] = pp('この x- の読み方と書き方である。', 'prompt-like-field', '欄の x-prompt と同じ形で書く。', '形が違うと、作成が案内を返せない。', '$defs/prompt の形かを見る。', 'machine')
ext['published']['x-prompt'] = pp('この x- の定義をほかの文脈へ見せるかである。', 'publish-if-read', 'ほかの文脈が使う x- だけを true にする。', '見せた定義は版を上げずに変えられない。', 'true の x- をほかの文脈が使っているかを見る。')
run = dlg['properties']['run']['properties']
run['kind']['x-prompt'] = pp('任せる目的（判定させる ・ 作業させる）である。', 'purpose', '判定だけなら judging、インスタンスを書かせるなら working にする。', '目的が違うと、基盤が答えの受け取り方を誤る。', '任せる先の role と kind が合っているかを見る。')
run['argv']['x-prompt'] = {"read": {"means": "CLI として起動するときの引数の配列である。", "when": "基盤が任せる先を起動するときに読む。"}, "write": []}
log('S08', 'x-extensions の1件の4欄と run.kind ・ run.argv に description と x-prompt を足す', '本体の部品の欄はすべて説明と案内を持つ、と決めたのに抜けていた', 'decisions「本体の部品の欄はすべて description と x-prompt を持つ」 ・ reviews2/adversarial 3-14')

# S09 インスタンスは id を持つ
a['then']['properties']['properties']['required'] = ['kind', '$schema', 'id']
for w in P:
    if w['id'] == 'instance-path-and-kind':
        w['rule'] = 'インスタンスを作るスキーマには x-instance-path を書き、根の properties に kind（const）と id を置く。'
        w['test'] = 'x-instance-path を持つスキーマの根に、kind の const と id があるかを見る。'
log('S09', 'インスタンスを作るスキーマの根に id を求める', '1件は id と kind が共通の契約なのに、ACDR は id を持たず x-relationship で指せなかった', 'board Q1 #35 ・ reviews2/adversarial 3-11')

# S10 決定権の確かめ方を、誰が書いたかを問わない形に
w = X['x-decided-by']['prompt']['write'][0]
w['rule'] = 'この欄の値が変わった変更の記録には、本人の承認（人の判定）がある。作成時の default の値はそのまま書いてよい。'
w['test'] = 'この欄が default 以外の値に変わった変更の記録に、I33 への人の判定があるかを見る。'
log('S10', 'human-entry-only を「本人の承認があるか」で確かめる形にし、作成時の default を除く',
    '① で誰が書いたかを区別しなくなった。作成時の既定値で毎回「人が決める欄の書き換え」が出た', 'two-decisions ① ・ reviews2/adversarial 1-5')

# S11 agent は本体の x-delegates も引く
for w in r['properties']['agent']['x-prompt']['write']:
    if w['id'] == 'agent-exists':
        w['rule'] = 'このスキーマか本体の x-delegates にある名前を書く。'
        w['test'] = '名前が、このスキーマか本体の x-delegates にあるかを見る。'
log('S11', 'agent-exists を「このスキーマか本体の x-delegates」にする', '本体以外のファイルが本体の judge を指すと violated になった', 'reviews2/adversarial 2-15')

# S12 x-delegates の説明から（案）を外す
X['x-delegates']['description'] = '任せる先。名前から、その任せる先の定義（立場 ・ 答えの形 ・ 起動のしかた）を引く'
dlg['description'] = '任せる先1件。目的（kind）と起動のしかた（argv の有無）を持つ'
# S13 廃語を直す（本体の文）
def words(o):
    if isinstance(o, dict): return {k: words(v) for k, v in o.items()}
    if isinstance(o, list): return [words(v) for v in o]
    if isinstance(o, str): return o.replace('判定に要る欄', '判定に必要な欄').replace('要る点が埋もれる', '必要な点が埋もれる').replace('が要る）', 'を必要とする）').replace('要るのに無い欄', '必要なのに無い欄').replace('AI もエンジンも要らず', 'AI もエンジンも使わず').replace('基盤の一員なら base', '基盤のファイルなら base').replace('base（基盤の一員）', 'base（基盤のファイル）')
    return o
a.update(words(a))
log('S13', '本体の文の廃語（要る）と、伝わらない語（一員）を直す', '利用者が廃語と決めた語が本体の文に残っていた', '.doc-writing/retired-words.json')

log('S12', 'x-delegates と $defs/delegate の説明から（案）を外す', 'この会話で決めた', 'decisions「x-delegates は残す」')

a['description'] = src['description']
json.dump(a, open(os.path.join(HERE, 'abstract.json'), 'w'), ensure_ascii=False, indent=2)
json.dump(S, open(os.path.join(HERE, 'abstract.changes.json'), 'w'), ensure_ascii=False, indent=1)
print(len(S), '件の直し')
