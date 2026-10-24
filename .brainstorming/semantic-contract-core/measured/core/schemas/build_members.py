"""Abstract のほかのファイル（本体以外のスキーマ）を書く。各欄は description と x-prompt を持つ（本体の prompt-per-unit）。
値だけで決まる決まりは JSON Schema で書き、x-prompt の write には書けないものだけを置く。"""
import json, os
HERE = os.path.dirname(os.path.abspath(__file__))
BODY = 'urn:semantic-agent-schema:base:semantic-abstract-schema:v1'
U = lambda kind: f'urn:semantic-agent-schema:base:{kind}:v1'
ULID = '^[0-7][0-9A-HJKMNP-TV-Z]{25}$'
JP = {"$ref": BODY + "#/$defs/json-pointer"}

def f(schema, desc, means, write=(), **read):
    r = {"means": means}; r.update(read)
    s = dict(schema); s['description'] = desc; s['x-prompt'] = {"read": r, "write": list(write)}
    return s
def w(i, rule, why, test, by='machine', **kw):
    d = {"id": i, "rule": rule, "why": why, "test": test, "by": by}
    if by == 'ai': d['agent'] = 'judge'
    d.update(kw); return d
def const_field(v, desc, means): return f({"const": v}, desc, means)
def head(kind, title, description, path=None, label=None):
    s = {"$schema": BODY, "$id": U(kind), "title": title, "description": description}
    if path: s['x-instance-path'] = path
    if label: s['x-instance-label'] = label
    return s

target = {"oneOf": [
  {"title": "インスタンス", "type": "object", "required": ["id", "kind"], "additionalProperties": False,
   "properties": {"id": {"type": "string", "minLength": 1}, "kind": {"type": "string", "pattern": "^urn:semantic-agent-schema:"}}},
  {"title": "それ以外のファイル", "type": "object", "required": ["path"], "additionalProperties": False,
   "properties": {"path": {"type": "string", "minLength": 1, "pattern": "^[^/]"}}}]}

# ---------------- annotations
ann = head('annotations', '複数のスキーマで使う注釈の形', 'x-relationship の値の形など、本体とほかのファイルが共有する形。インスタンスは作らない。')
ann['$defs'] = {"relationship": {"type": "object", "required": ["to"], "additionalProperties": False, "properties": {
  "to": f({"type": "string", "pattern": "^urn:semantic-agent-schema:[a-z0-9][a-z0-9-]*:[a-z0-9][a-z0-9-]*:v[0-9]+$"}, '指す先の種類', '指す先のインスタンスが従う Concrete の URN である。'),
  "version": f({"type": "string", "pattern": "^v[0-9]+$"}, '公開された版', 'ほかの文脈の公開された言葉を指すときの版である。', when='ほかの文脈を指すときだけ読む。')}},
  "target": target}

# ---------------- change
ch = head('change', '変更の記録', '意図1つぶんの変更の記録。セッション ID ・ 前後の値 ・ 変更の仕組みの値（commit など）は持たない。前後は変更の仕組みからポートで受け取る。',
          '.semantic-agent-schema/changes/{id}.json', '変更 {id}「{intent}」')
ch.update({"type": "object", "required": ["$schema", "kind", "id", "purpose_ref", "intent", "writer_kind", "items"], "additionalProperties": False, "properties": {
  "$schema": {"const": U('change')},
  "kind": const_field('change', '種類', '変更の記録であること。'),
  "id": f({"type": "string", "pattern": ULID, "readOnly": True}, '記録の id', '基盤が書いた時に1回だけ決める、打ち合わせずに一意な値（ULID）である。', parse='時刻の順に並ぶ26文字。'),
  "purpose_ref": f({"type": "string", "minLength": 1}, '目的の参照', '何の目的の変更か（ACDR の番号 ・ ボードの論点など）。基盤は指す先を解釈しない。', shown='変更を見せるとき、目的 → 意図の順の先頭に出る。'),
  "intent": f({"type": "string", "minLength": 1, "maxLength": 120}, '意図', 'この変更で何をしたかったかの1文である。', [
      w('one-intent', '意図は1つで、変更の項目のすべてがその意図のための変更である。', '意図が2つあると、意図と変更が合っているか（I33）を判定できない。', '文に2つの目的が並んでいないかと、項目に意図の外の変更が無いかを見る。', 'ai')],
      shown='変更を見せるときの見出しになる。'),
  "writer_kind": f({"enum": ["human", "ai-agent"]}, '書いた者の種類', '人か AI の Agent かである。'),
  "items": f({"type": "array", "minItems": 1, "readOnly": True, "items": {"type": "object", "required": ["target"], "additionalProperties": False,
      "properties": {"target": {"$ref": U('annotations') + "#/$defs/target"}, "field": JP},
      "allOf": [{"if": {"properties": {"target": {"required": ["path"], "properties": {"path": {"not": {"pattern": "\\.json$"}}}}}}, "then": {"not": {"required": ["field"]}}}]}},
      '変更の項目', 'どの対象のどの欄が変わったかの並びで、基盤が最後に確定した姿 → いまの前後から入れる。',
      [w('unit-field', '欄は x-prompt を持つ書く単位の JSON Pointer で、配列の添字まで下げない。', '変更の単位を判定の単位にそろえる（Q1 #30）。', 'field が対象の Concrete の書く単位を指すかを見る。')],
      parse='対象はインスタンスなら id と kind、それ以外はパス。欄は JSON の対象だけが持つ。', when='AI は書かない。基盤が「変更を記録する」で入れる。'),
  "replaces": f({"type": "string", "pattern": ULID, "x-relationship": {"to": U('change')}}, '置き換える意図の id', '差し戻した意図を直すときに、前の変更の記録を指す。', when='差し戻しを直すときだけ書く。')}})

# ---------------- judgment
jr = head('judgment', '判定の記録', '対象 × rule ごとに1件の判定の記録。セッション ID は持たない。状態は保存せず、判定した値といまの値を比べて導く。',
          '.semantic-agent-schema/judgments/{key}.json', '判定 {key}')
jr.update({"type": "object", "required": ["$schema", "kind", "id", "key", "target", "rule", "result", "grounds", "judged_by", "judged_values"], "additionalProperties": False, "properties": {
  "$schema": {"const": U('judgment')},
  "kind": const_field('judgment', '種類', '判定の記録であること。'),
  "id": f({"type": "string", "minLength": 1, "readOnly": True}, '記録の id', '対象 × rule から基盤が決める名前で、key と同じ値である。'),
  "key": f({"type": "string", "pattern": "^[A-Za-z0-9._~-]+(/[A-Za-z0-9._~-]+)*$", "readOnly": True}, '対象 × rule の識別', '対象と rule から基盤が作る、ファイルの名前にもなる識別である。', parse='<対象>/<rule の id> の形。同じ対象 × rule は上書きされ、最後の1件だけが残る。'),
  "target": f({"$ref": U('annotations') + "#/$defs/target"}, '対象', '判定したインスタンス（id と kind）か、I33 なら変更の記録である。'),
  "rule": f({"type": "string", "pattern": "^urn:semantic-agent-schema:.+#[a-z0-9]+(-[a-z0-9]+)*$"}, '判定した rule', '判定した決まりで、スキーマの URN と rule の id である。'),
  "result": f({"enum": ["verified", "violated", "unverified", "unsupported", "error"]}, '結果', '5つの値のどれか1つである。', shown='状態の導出と、変更を見せるときの例外に出る。'),
  "grounds": f({"type": "array", "items": {"type": "object", "required": ["at", "text"], "additionalProperties": False, "properties": {"at": JP, "text": {"type": "string", "minLength": 1}}}},
      '判定の理由', '結果の理由を、読んだ値の場所と1文で並べたものである。',
      [w('grounds-in-reads', '理由の場所は、判定した値で読んだ場所のどれかである。', '読んでいない値を理由にした答えは unverified にする（I14）。', 'at が judged_values.reads の at にあるかを見る。')]),
  "meaning": f({"type": "object", "required": ["whole", "per_target"], "additionalProperties": False, "properties": {
      "whole": {"type": "string", "minLength": 1}, "per_target": {"type": "array", "items": {"type": "object", "required": ["target", "text", "at"], "additionalProperties": False, "properties": {
        "target": {"$ref": U('annotations') + "#/$defs/target"}, "text": {"type": "string", "minLength": 1}, "at": {"type": "array", "items": JP}}}}}},
      '意味（I33）', '意図全体の意味の1文と、対象ごとの意味の1文である。', when='I33 の判定の記録だけが持つ。変更を見せるとき、意図の下に出る。'),
  "question": f({"type": "string", "minLength": 1}, '問い', '何が分かれば決められるかの1文である。', when='結果が unverified のときに書く。'),
  "judged_by": f({"enum": ["machine", "tool", "ai", "human"]}, '判定した者', '機械 ・ 外のツール ・ AI ・ 人のどれかである。'),
  "judged_values": f({"type": "object", "required": ["reads"], "additionalProperties": False, "properties": {
      "reads": {"type": "array", "items": {"type": "object", "required": ["at", "value"], "additionalProperties": False, "properties": {"at": JP, "value": True}}},
      "before": {"type": "array", "items": {"type": "object", "required": ["at", "value"], "properties": {"at": JP, "value": True}}},
      "prompts": {"type": "array", "items": {"type": "string"}}, "handoff": {"type": "string"},
      "delegate": {"type": "string"}, "model": {"type": "string"}, "tool_version": {"type": "string"}}},
      '判定した値', '判定したときに読んだ値と渡したもの（読んだ欄の値 ・ I33 の前の値 ・ 読んだ x-prompt の文 ・ 渡した引き継ぎ ・ 任せる先とモデル）である。',
      parse='いまの値と JSON Schema の等しさで比べ、違えばこの判定は効かない（I16）。')},
  "allOf": [
    {"if": {"properties": {"judged_by": {"const": "ai"}}}, "then": {"properties": {"judged_values": {"required": ["delegate", "model", "prompts"]}}}},
    {"if": {"properties": {"result": {"const": "unverified"}, "judged_by": {"const": "ai"}}}, "then": {"required": ["question"]}},
    {"if": {"properties": {"rule": {"pattern": "#intent-matches-change$"}}}, "then": {"required": ["meaning"], "properties": {"judged_values": {"required": ["before"]}}}}]})

# ---------------- session（手元の DB の行の形）
sess = head('session', 'セッションと手元のセッションの索引の行', '手元の DB に置く行の形。確定しない。インスタンスのファイルは作らない。DDL は SQLite のアダプターがこの形から書く。')
sid = {"type": "object", "required": ["tool", "id"], "additionalProperties": False, "properties": {
  "tool": f({"type": "string", "pattern": "^[a-z0-9][a-z0-9-]*$"}, 'AI ツールの種類', 'セッションを発行した AI ツールの名前（claude-code ・ codex など）である。'),
  "id": f({"type": "string", "minLength": 1}, 'ツールが発行した文字列', 'AI ツールが発行したセッションの文字列で、基盤は解釈しない。')}}
sess['$defs'] = {
 "session-id": f(sid, 'セッション ID', 'AI ツールの種類と、そのツールが発行した文字列の組である。基盤は文字列を解釈しない。'),
 "session": {"type": "object", "required": ["session", "state", "last_used_at"], "additionalProperties": False,
   "properties": {
     "session": f({"$ref": "#/$defs/session-id"}, 'セッション ID', 'この行のセッションである。'),
     "purpose_ref": f({"type": "string", "minLength": 1}, '目的の参照', 'このセッションが進める目的である。'),
     "parent": f({"$ref": "#/$defs/session-id"}, '親のセッション', 'このセッションを立てた親である。'),
     "role": f({"type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$"}, '役割', '親の Concrete の x-delegates にある任せる先の名前である。'),
     "state": f({"enum": ["open", "closed"]}, '状態', '開いている（再開できる）か閉じたかである。', [
        w('no-reopen', '閉じた行は open に戻らない。', '閉じたセッションを開き直すと、前の結論が持ち込まれる（I45）。', '前の値が closed で今の値が open の行が無いかを見る。')]),
     "last_used_at": f({"type": "string", "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(\\.[0-9]+)?Z$"}, '最後に使った時刻', 'このセッションを最後に使った UTC の時刻である。')},
   "oneOf": [
     {"title": "オーケストレーターのセッション", "not": {"anyOf": [{"required": ["purpose_ref"]}, {"required": ["parent"]}, {"required": ["role"]}]}},
     {"title": "目的のセッション", "required": ["purpose_ref", "parent"], "not": {"required": ["role"]}},
     {"title": "任せる先のセッション", "required": ["purpose_ref", "parent", "role"]}],
   "x-prompt": {"read": {"means": "セッション1つの行で、種類は目的 ・ 親 ・ 役割の組で決まる。"}, "write": [
     w('fixed-at-creation', 'セッション ID ・ 目的 ・ 親 ・ 役割は、立てた時の値と同じである。', '変えると、索引と記録の結び付きが崩れる（I44）。', '前の値と今の値で4つが同じかを見る。'),
     w('parent-kind', '目的のセッションの親はオーケストレーターのセッションで、任せる先のセッションの親は目的のセッションで、目的は親と同じである。', '入れ子と、子が呼ぶことを止める（I44）。', '親の行の種類と目的を見る。'),
     w('one-open', '1つの手元で、開いているオーケストレーターは1つ、同じ目的の目的のセッションは1つ、目的 ・ 親 ・ 役割が同じ任せる先は1つである。', '同じ目的に紐づくセッションを同時に開くと競合する（I46）。', '開いている行を枠ごとに数える。')]}},
 "session-change": {"type": "object", "required": ["session", "change_id"], "additionalProperties": False, "properties": {
     "session": f({"$ref": "#/$defs/session-id"}, 'セッション ID', '変更の記録を書いたセッションである。'),
     "change_id": f({"type": "string", "pattern": ULID}, '変更の記録の id', 'そのセッションで書いた変更の記録である。', [
        w('session-exists', '行のセッションは手元の DB に在る。', '無いセッションの行は、セッションで引けない。', 'session が sessions の行にあるかを見る。')])},
   "x-prompt": {"read": {"means": "手元のセッションの索引の1行で、作業した本人だけが引ける。"}, "write": []}}}

# ---------------- settings
st = head('settings', '基盤の設定', '基盤の振る舞いの値。利用者（~/.semantic-agent-schema/settings.json）・ プロジェクト（.semantic-agent-schema/settings.json、確定する）・ 手元（.semantic-agent-schema/settings.local.json、確定しない）の3段を、手元 ＞ プロジェクト ＞ 利用者の順で重ねる。許可 ・ キャッシュの有効時間 ・ 料金は持たない。',
          '.semantic-agent-schema/settings.json', '設定')
st.update({"type": "object", "required": ["$schema", "kind", "id"], "additionalProperties": False, "properties": {
  "$schema": {"const": U('settings')},
  "kind": const_field('settings', '種類', '基盤の設定であること。'),
  "id": f({"enum": ["user", "project", "local"]}, '置き場所', '利用者 ・ プロジェクト ・ 手元のどの段かである。'),
  "session": f({"type": "object", "additionalProperties": False, "properties": {
      "newWhenHistoryAbovePercent": {"type": "number", "exclusiveMinimum": 0, "maximum": 100}}},
      'セッションの設定', '再開か新規かを決めるときの値である。',
      parse='newWhenHistoryAbovePercent：履歴がコンテキストの上限に占める割合がこれを超えたら、有効時間内でも新しく立てる。書かなければ使わない（既定は無い）。')}})

# ---------------- change-view（変更のまとまりの形。保存しない出力）
EXC = ["intent-missing", "intent-mismatch", "unrecorded", "no-change", "record-rewritten", "human-field-changed", "state-changed", "unconfirmed-outside-intent", "not-in-range", "unreadable"]
cv = head('change-view', '変更のまとまり', '変更を見せるときの出力の形。保存せず、同じ前後と記録からは毎回同じ形に作る。Concrete はこの形を $ref で使い、見せ方を足す。')
cv['$defs'] = {"view": {"type": "object", "required": ["range", "exceptions", "purposes", "unreadable"], "additionalProperties": False, "properties": {
  "range": f({"type": "object", "required": ["before", "after"], "properties": {"before": {"type": "string"}, "after": {"type": "string"}}}, '前後の参照', 'ポートに渡した前の姿と後の姿の参照で、基盤は解釈しない。'),
  "exceptions": f({"type": "array", "items": {"type": "object", "required": ["type", "count", "items"], "additionalProperties": False, "properties": {
      "type": {"enum": EXC}, "count": {"type": "integer", "minimum": 1}, "items": {"type": "array", "items": {"type": "object"}}}}},
      '例外', '例外の種類ごとの件数と中身で、先頭に出る。'),
  "purposes": f({"type": "array", "items": {"type": "object", "required": ["purpose_ref", "intents"], "properties": {
      "purpose_ref": {"type": "string"}, "intents": {"type": "array", "items": {"type": "object", "required": ["change_id", "intent", "targets"], "properties": {
        "change_id": {"type": "string"}, "intent": {"type": "string"}, "judgment": {"enum": ["verified", "violated", "unverified", "unsupported", "error", "none"]},
        "meaning": {"type": "object"}, "targets": {"type": "array", "items": {"type": "object", "required": ["target", "kind"], "properties": {
          "target": {"$ref": U('annotations') + "#/$defs/target"}, "kind": {"enum": ["fields", "lines"]},
          "fields": {"type": "array", "items": {"type": "object", "required": ["at"], "properties": {"at": JP, "before": True, "after": True}}},
          "lines": {"type": "array", "items": {"type": "object", "required": ["mark", "text"], "properties": {"mark": {"enum": [" ", "-", "+"]}, "text": {"type": "string"}}}}}}}}}}}}},
      '目的 → 意図', '目的ごと、意図ごとの変更で、JSON は欄ごと、それ以外は行ごと（ファイルの全体に印）に成型したものである。'),
  "unreadable": f({"type": "array", "items": {"type": "string"}}, '成型できなかった対象', '文字として読めないなどで成型できなかった対象のパスである。')}}}

for name, s in [('annotations', ann), ('change', ch), ('judgment', jr), ('session', sess), ('settings', st), ('change-view', cv)]:
    json.dump(s, open(os.path.join(HERE, f'{name}.schema.json'), 'w'), ensure_ascii=False, indent=2)
print('ok')
