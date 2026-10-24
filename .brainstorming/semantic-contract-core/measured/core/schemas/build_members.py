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
      "properties": {"target": {"$ref": "#/$defs/target"}, "field": JP},
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
  "target": f({"$ref": U('change') + "#/$defs/target"}, '対象', '判定したインスタンス（id と kind）か、I33 なら変更の記録である。'),
  "rule": f({"type": "string", "pattern": "^urn:semantic-agent-schema:.+#[a-z0-9]+(-[a-z0-9]+)*$"}, '判定した rule', '判定した決まりで、スキーマの URN と rule の id である。'),
  "result": f({"enum": ["verified", "violated", "unverified", "unsupported", "error"]}, '結果', '5つの値のどれか1つである。', shown='状態の導出と、変更を見せるときの例外に出る。'),
  "grounds": f({"type": "array", "items": {"type": "object", "required": ["at", "text"], "additionalProperties": False, "properties": {"at": JP, "text": {"type": "string", "minLength": 1}}}},
      '判定の理由', '結果の理由を、読んだ値の場所と1文で並べたものである。',
      [w('grounds-in-reads', '理由の場所は、判定した値で読んだ場所のどれかである。', '読んでいない値を理由にした答えは unverified にする（I14）。', 'at が judged_values.reads の at にあるかを見る。')]),
  "meaning": f({"type": "object", "required": ["whole", "per_target"], "additionalProperties": False, "properties": {
      "whole": {"type": "string", "minLength": 1}, "per_target": {"type": "array", "items": {"type": "object", "required": ["target", "text", "at"], "additionalProperties": False, "properties": {
        "target": {"$ref": U('change') + "#/$defs/target"}, "text": {"type": "string", "minLength": 1}, "at": {"type": "array", "items": JP}}}}}},
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
     "role": f({"type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$"}, '役割', '親のセッションが呼んだ相手の名前（Skill ・ Agent の名前、または Concrete の x-delegates の名前）である。'),
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
# 変更のまとまり（保存しない出力の形）は、持ち主 change の $defs に置く（Q6 #6）
ch['$defs'] = {"target": target, "view": {"type": "object", "required": ["range", "exceptions", "purposes", "unreadable"], "additionalProperties": False, "properties": {
  "range": f({"type": "object", "required": ["before", "after"], "properties": {"before": {"type": "string"}, "after": {"type": "string"}}}, '前後の参照', 'ポートに渡した前の姿と後の姿の参照で、基盤は解釈しない。'),
  "exceptions": f({"type": "array", "items": {"type": "object", "required": ["type", "count", "items"], "additionalProperties": False, "properties": {
      "type": {"enum": EXC}, "count": {"type": "integer", "minimum": 1}, "items": {"type": "array", "items": {"type": "object"}}}}},
      '例外', '例外の種類ごとの件数と中身で、先頭に出る。'),
  "purposes": f({"type": "array", "items": {"type": "object", "required": ["purpose_ref", "intents"], "properties": {
      "purpose_ref": {"type": "string"}, "intents": {"type": "array", "items": {"type": "object", "required": ["change_id", "intent", "targets"], "properties": {
        "change_id": {"type": "string"}, "intent": {"type": "string"}, "judgment": {"enum": ["verified", "violated", "unverified", "unsupported", "error", "none"]},
        "meaning": {"type": "object"}, "targets": {"type": "array", "items": {"type": "object", "required": ["target", "kind"], "properties": {
          "target": {"$ref": "#/$defs/target"}, "kind": {"enum": ["fields", "lines"]},
          "fields": {"type": "array", "items": {"type": "object", "required": ["at"], "properties": {"at": JP, "before": True, "after": True}}},
          "lines": {"type": "array", "items": {"type": "object", "required": ["mark", "text"], "properties": {"mark": {"enum": [" ", "-", "+"]}, "text": {"type": "string"}}}}}}}}}}}}},
      '目的 → 意図', '目的ごと、意図ごとの変更で、JSON は欄ごと、それ以外は行ごと（ファイルの全体に印）に成型したものである。'),
  "unreadable": f({"type": "array", "items": {"type": "string"}}, '成型できなかった対象', '文字として読めないなどで成型できなかった対象のパスである。')}}}


# ---------------- view（ビュー：kind ごとの見せ方。文脈のファイル）
PTR_EXPR = {"type": "string", "pattern": "^(/([^~/]|~[01])*)+$"}
vw = head('view', 'ビュー', 'ある kind（Concrete）のインスタンスをどう見せるかの定義。文脈ごと ・ kind ごとに0か1つ。見せ方だけを持ち、データの形と意味は持たない（Concrete が持つ）。',
          '.semantic-agent-schema/views/{id}.json', 'ビュー {id}')
vw.update({"type": "object", "required": ["$schema", "kind", "id", "for", "sections"], "additionalProperties": False, "properties": {
  "$schema": {"const": U('view')},
  "kind": const_field('view', '種類', 'ビューであること。'),
  "id": f({"type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$"}, 'ビューの id', 'このビューの名前である。'),
  "for": f({"type": "string", "pattern": "^urn:semantic-agent-schema:[a-z0-9][a-z0-9-]*:[a-z0-9][a-z0-9-]*:v[0-9]+$"}, '描く kind', 'このビューが描く Concrete の URN である。', [
      w('one-view-per-kind', '同じ文脈で、同じ kind を描くビューは1つまでである。', '2つあると、どちらで描くかが決まらない（I26）。', '同じ for を持つビューの数を数える。'),
      w('for-exists', 'for の Concrete は、閉じた一覧か公開された版にある。', '無い kind は描けない。', 'for の URN が引けるかを見る。')]),
  "sections": f({"type": "array", "minItems": 1, "items": {"type": "object", "required": ["title", "parts"], "additionalProperties": False, "properties": {
      "title": {"type": "string", "minLength": 1},
      "parts": {"type": "array", "minItems": 1, "items": {"type": "object", "required": ["part", "value"], "additionalProperties": False, "properties": {
        "part": {"type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$"}, "value": PTR_EXPR, "label": {"type": "string"}}}}}}},
      '節の並び', '見出しと、部品で値を見せる並びである。', [
      w('value-in-concrete', '部品の value（JSON Pointer）は、for の Concrete の欄を指す。', '無い欄を指すと、描けない（Q1 #23）。', 'value の先頭の欄が for の Concrete の properties にあるかを見る。'),
      w('part-in-catalog', '部品の part は、文脈の部品のカタログにある名前である。', '無い部品は描けない（Q1 #23）。', 'part の名前がカタログの parts にあるかを見る。')],
      parse='部品の value は、インスタンスの中の場所（JSON Pointer）として読む。')}})

# ---------------- catalog（部品のカタログ。デザインシステムの1つ目のファイル）
TOKEN = {"type": "string", "pattern": "^[a-z0-9]+(\\.[a-z0-9-]+)*$"}
ca = head('catalog', '部品のカタログ', 'ビューが使う部品の一覧と、部品が使うデザイントークンの名前。色や寸法の値は持たない（デザイントークンが持つ）。文脈に1つ。',
          '.semantic-agent-schema/design/catalog.json', '部品のカタログ')
ca.update({"type": "object", "required": ["$schema", "kind", "id", "parts"], "additionalProperties": False, "properties": {
  "$schema": {"const": U('catalog')},
  "kind": const_field('catalog', '種類', '部品のカタログであること。'),
  "id": f({"const": "catalog"}, 'id', '文脈に1つのカタログであること。'),
  "parts": f({"type": "object", "minProperties": 1, "propertyNames": {"pattern": "^[a-z0-9]+(-[a-z0-9]+)*$"}, "additionalProperties": {"type": "object", "required": ["description", "tokens"], "additionalProperties": False, "properties": {
      "description": {"type": "string", "minLength": 1}, "tokens": {"type": "array", "items": TOKEN}}}},
      '部品', '部品の名前から、その説明と、使うデザイントークンの名前を引く。', [
      w('tokens-exist', '部品が使うトークンの名前は、デザイントークンにある。', '無いトークンを使う部品は描けない（Q1 #23）。', 'tokens の各名前がデザイントークンの tokens にあるかを見る。')])}})

# ---------------- design-tokens（デザイントークン。デザインシステムの2つ目のファイル）
COLOR = {"type": "string", "pattern": "^#[0-9A-Fa-f]{6}$"}
dt = head('design-tokens', 'デザイントークン', '色などの値を、明るい表示と暗い表示の両方で持つ。どこに使うかは持たない（カタログが持つ）。文脈に1つ。',
          '.semantic-agent-schema/design/tokens.json', 'デザイントークン')
dt.update({"type": "object", "required": ["$schema", "kind", "id", "tokens"], "additionalProperties": False, "properties": {
  "$schema": {"const": U('design-tokens')},
  "kind": const_field('design-tokens', '種類', 'デザイントークンであること。'),
  "id": f({"const": "tokens"}, 'id', '文脈に1つのデザイントークンであること。'),
  "tokens": f({"type": "object", "minProperties": 1, "propertyNames": TOKEN, "additionalProperties": {"type": "object", "required": ["light", "dark"], "additionalProperties": False, "properties": {"light": COLOR, "dark": COLOR}}},
      'トークン', 'トークンの名前から、明るい表示と暗い表示の値を引く。'),
  "contrast": f({"type": "array", "items": {"type": "object", "required": ["fg", "bg", "min"], "additionalProperties": False, "properties": {"fg": TOKEN, "bg": TOKEN, "min": {"type": "number", "minimum": 1}}}},
      'コントラストの組', '文字と地にするトークンの組と、必要なコントラスト比である。', [
      w('contrast-meets', '組ごとに、明るい表示と暗い表示の両方でコントラスト比が min 以上である。', '読めない組み合わせが描かれる。', 'WCAG の式で両方の比を計算する。')])}})

# ---------------- glossary（term の辞書。文脈に1つ。欄から $ref で指される）
gl = head('glossary', 'glossary', '文脈の term の辞書。$defs の各 term が $anchor を持ち、Concrete の欄は urn:…:glossary:v<版>#<名前> で指す。出した版の term は変えない。',
          '.semantic-agent-schema/glossary.json', 'glossary')
gl.update({"type": "object", "required": ["$schema", "$id", "kind", "id", "$defs"], "properties": {
  "$schema": {"const": U('glossary')},
  "$id": f({"type": "string", "pattern": "^urn:semantic-agent-schema:[a-z0-9][a-z0-9-]*:glossary:v[0-9]+$"}, 'glossary の名前と版', 'Concrete の欄が $ref で指す、この辞書の URN と版である。', [
      w('never-change-published', '出した版の term の中身は、出した時と同じである。', '指している Concrete の意味が黙って変わる（I25）。', '同じ版の term が前と同じかを見る。')]),
  "kind": const_field('glossary', '種類', 'glossary であること。'),
  "id": f({"const": "glossary"}, 'id', '文脈に1つの glossary であること。'),
  "$defs": f({"type": "object", "minProperties": 1, "propertyNames": {"pattern": "^[A-Za-z_][A-Za-z0-9_]*$"}, "additionalProperties": {"type": "object", "required": ["$anchor", "description"], "properties": {
      "$anchor": {"type": "string", "pattern": "^[A-Za-z_][A-Za-z0-9_]*$"}, "description": {"type": "string", "minLength": 1}, "x-unit": {"type": "string"}}}},
      'term の並び', 'term の名前から、意味（description）・ 値の形（JSON Schema）・ 単位を引く。', [
      w('anchor-is-name', '$anchor は $defs のキーと同じ名前である。', '名前と指し先がずれると、欄が別の term を指す。', 'キーと $anchor を比べる。')])}})

# ---------------- published（公開された言葉の版。値オブジェクト）
pb = head('published', '公開された言葉の版', 'Concrete の公開する部分（x-published が true の欄と x- の定義）の版。出した版は変えない。インスタンスのファイルは作らず、Concrete と一緒に基盤が組む。')
pb['$defs'] = {"version": {"type": "object", "required": ["of", "version", "fields"], "additionalProperties": False, "properties": {
  "of": f({"type": "string", "pattern": "^urn:semantic-agent-schema:"}, '公開した Concrete', 'この版を出した Concrete の URN である。'),
  "version": f({"type": "string", "pattern": "^v[0-9]+$"}, '版', '公開された言葉の版である。', [
      w('never-change', '出した版の中身は、出した時と同じである。', '読む側が黙って壊れる（I7）。', '同じ版の fields と extensions が前と同じかを見る。')]),
  "fields": f({"type": "array", "items": JP}, '見せる欄', 'x-published が true の欄の場所の並びである。'),
  "extensions": f({"type": "array", "items": {"type": "string", "pattern": "^x-"}}, '見せる x- の定義', 'published が true の x- の名前の並びである。')}}}

for name, s in [('change', ch), ('judgment', jr), ('session', sess), ('settings', st), ('view', vw), ('catalog', ca), ('design-tokens', dt), ('glossary', gl), ('published', pb)]:
    json.dump(s, open(os.path.join(HERE, f'{name}.schema.json'), 'w'), ensure_ascii=False, indent=2)
print('ok')
