"""inventory.json の決まりを、v0.3 の形（Vocabulary ・ Concrete ・ Interpreter の要求と応答 ・ Projection Report）で書き直す。

これは v0.3 の examples/ の形にならった試案であり、正式な仕様ではない。
使い方（リポジトリの根から）:
    python3 -I .brainstorming/semantic-contract-core/measured/v03-experiment/build_v03.py
書くもの: v03/ の下の JSON。決まり（形の決まりと「説明のとおりに書く」を除く）が、
dataSchema ・ constraints ・ protocol ・ unplaced のどれか1か所にだけ入っていることを確かめてから書く。
"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "v03")
TRIAL = "試案。v0.3 の examples の形にならったもので、正式な仕様ではない。"

inv = json.load(open(os.path.join(HERE, "inventory.json"), encoding="utf-8"))
rows = {r["id"]: r for r in inv["rules"]}
targets = [r for r in inv["rules"] if r["group"] != "形" and r["checked"] != "D"]

V_TEXT = "urn:daidaiiro:vocab:text-ja:0-trial"
V_CORE = "urn:daidaiiro:vocab:constraint-core:0-trial"
V_BS = "urn:daidaiiro:vocab:brainstorming:0-trial"
V_JUDGE = "urn:daidaiiro:vocab:judgment:0-trial"

# ---- Vocabulary ------------------------------------------------------------------
# 側面は docs/11 の「Semanticの4つの側面（分析のための仮分類）」の名前
vocabularies = {
    "text-ja": {
        "kind": "vocabulary", "id": V_TEXT, "_note": TRIAL,
        "terms": {
            "OneSentence": {"description": "文が1つだけ。文は 。！？ と改行で切り、「」『』の中では切らない",
                            "aspect": "Formal", "params": {}, "requires": ["validate-ja-sentence"],
                            "definedBy": "src/brainstorming/crates/core/src/domain/text.rs:7"},
            "SentenceAtMost": {"description": "どの1文も、「」『』の中を除いて max 字まで",
                               "aspect": "Formal", "params": {"max": {"type": "integer", "minimum": 1}},
                               "requires": ["validate-ja-sentence"],
                               "definedBy": "src/brainstorming/crates/core/src/domain/text.rs:39"},
            "CharsAtMost": {"description": "欄全体が max 字まで。approx が true なら線は目安で、超えても誤りにしない",
                            "aspect": "Structural",
                            "params": {"max": {"type": "integer"}, "approx": {"type": "boolean"}},
                            "requires": ["validate-structure"]},
            "LinesAtMost": {"description": "改行で数えた行が max 行まで。approx は CharsAtMost と同じ",
                            "aspect": "Structural",
                            "params": {"max": {"type": "integer"}, "approx": {"type": "boolean"}},
                            "requires": ["validate-lines"]},
        },
    },
    "constraint-core": {
        "kind": "vocabulary", "id": V_CORE, "_note": TRIAL + " どのボードにも依らない汎用の制約。どの Interpreter もまだ持たない。",
        "terms": {
            "Count": {"description": "path の配列のうち where に当たる要素の数が min 以上 max 以下",
                      "aspect": "Formal",
                      "params": {"where": {"type": "object"}, "min": {"type": "integer"}, "max": {"type": "integer"}},
                      "requires": ["validate-count"]},
            "Implies": {"description": "if の条件がインスタンスで成り立つなら then の条件も成り立つ",
                        "aspect": "Formal", "params": {"if": {}, "then": {}}, "requires": ["validate-implies"]},
            "RefersTo": {"description": "値が in の要素の id を指し、when が成り立つとき、指す先の at の値が accept のどれか",
                         "aspect": "Formal",
                         "params": {"in": {"type": "string"}, "when": {"type": "object"},
                                    "at": {"type": "string"}, "accept": {"type": "array"}},
                         "requires": ["validate-reference"]},
            "UniqueKey": {"description": "path の配列の要素のあいだで key の値が重ならない",
                          "aspect": "Formal", "params": {"key": {"type": "string"}}, "requires": ["validate-unique-key"]},
            "OrderedBy": {"description": "path の配列が key の値で order の順に並ぶ",
                          "aspect": "Formal", "params": {"key": {"type": "string"}, "order": {"enum": ["asc", "desc"]}},
                          "requires": ["validate-order"]},
            "NotIn": {"description": "when が成り立つとき、値（小文字にしてから）が values のどれでもない",
                      "aspect": "Structural", "params": {"values": {"type": "array"}, "when": {"type": "object"}},
                      "requires": ["validate-structure"]},
            "InList": {"description": "値が list の名前の一覧のどれか。一覧はこの語彙の外に置く",
                       "aspect": "Structural", "params": {"list": {"type": "string"}}, "requires": ["validate-structure"]},
        },
    },
    "brainstorming": {
        "kind": "vocabulary", "id": V_BS, "_note": TRIAL + " このボードにしか無い意味。",
        "terms": {
            "TopicId": {"description": "ボードの中の論点の名前（Q と数字）", "aspect": "Descriptive",
                        "valueSchema": {"type": "string", "pattern": "^Q[0-9]+$"}, "requires": ["validate-structure"]},
            "TopicStatus": {"description": "論点を開いたか（open）・ 決着したか（settled）・ どちらでもないか（waiting）。保存する事実",
                            "aspect": "Operational", "valueSchema": {"enum": ["waiting", "open", "settled"]},
                            "requires": ["validate-structure"]},
            "MentionedTopicsDeclared": {"description": "本文に出てくる論点の id（前後が英数字でない Q と数字）は、自分 ・ depends ・ 下流のどれか",
                                        "aspect": "Formal", "requires": ["extract-topic-mentions"],
                                        "definedBy": "src/brainstorming/crates/core/src/domain/inspect.rs:12"},
            "UsedDownstreamOrOutside": {"description": "下流の論点の depends に現れるか、verification.out_of_scope の item に「外の作業」がある",
                                        "aspect": "Formal", "requires": ["validate-downstream-or-outer"],
                                        "definedBy": "src/brainstorming/crates/core/src/domain/inspect.rs:126"},
            "TreeDepthStep": {"description": "tree の rows の depth は、前の行の depth + 1 を超えない",
                              "aspect": "Formal", "requires": ["validate-tree-depth"]},
            "CheckedSvg": {"description": "design-svg の検査を通した SVG", "aspect": "Operational",
                           "requires": ["check-svg"]},
            "StandaloneHtml": {"description": "外のファイルや URL を読まない1枚の HTML", "aspect": "Formal",
                               "requires": ["check-html-no-external"]},
            "NotRendered": {"description": "書き手の検証の記録で、画面に描かない", "aspect": "Operational",
                            "requires": ["check-rendered-page"]},
        },
    },
    "judgment": {
        "kind": "vocabulary", "id": V_JUDGE,
        "_note": TRIAL + " いまは x-prompt の rule と test に文で書き、AI か人に意味を見させている決まり。宣言は問いの文だけで、検査の中身は Interpreter（AI ・ 人）が持つ。",
        "terms": {
            "Judgment": {"description": "question に yes と答えられるか", "aspect": "Descriptive",
                         "params": {"question": {"type": "string"}}, "requires": ["judge-meaning"]},
            "VisualJudgment": {"description": "描いて目で見て、question に yes と答えられるか", "aspect": "Descriptive",
                               "params": {"question": {"type": "string"}}, "requires": ["judge-visual"]},
        },
    },
}

# ---- Concrete Contract ------------------------------------------------------------
data_schema = ["P3", "P4", "P9", "P10", "P14", "P40", "P43", "P65", "P69", "P74", "P84", "P90", "P92",
               "P100", "P105", "I5", "G8", "G11", "I3", "I7", "P51"]
# I3 ・ I7 ・ P51 はいまの x-ref のまま（dataSchema の注釈として持つ）
protocol = ["G7"]

def c(cid, inventory, path, term, params=None, note=""):
    d = {"id": cid, "inventory": inventory, "path": path, "term": term}
    if params is not None:
        d["params"] = params
    if note:
        d["note"] = note
    return d

OPEN = {"status": "open"}
constraints = [
    c("rejected-at-least-one", ["P2", "P48"], "/topics/*/rejected", f"{V_CORE}#Count", {"min": 1}),
    c("title-40", ["P6"], "/title", f"{V_TEXT}#CharsAtMost", {"max": 40, "approx": True}),
    c("intro-one-sentence", ["P7"], "/intro", f"{V_TEXT}#OneSentence"),
    c("queue-points-open", ["P15", "I4"], "/queue/*/topic", f"{V_CORE}#RefersTo",
      {"in": "/topics", "at": "status", "accept": ["open"]}),
    c("open-depends-settled", ["I8"], "/topics/*/depends/*", f"{V_CORE}#RefersTo",
      {"in": "/topics", "when": OPEN, "at": "status", "accept": ["settled"]}),
    c("log-newest-first", ["P23"], "/log", f"{V_CORE}#OrderedBy", {"key": "date", "order": "desc"}),
    c("topics-3-to-5", ["P26"], "/topics", f"{V_CORE}#Count", {"min": 3, "max": 5}),
    c("topic-id-unique", ["P28"], "/topics", f"{V_CORE}#UniqueKey", {"key": "id"}),
    c("topic-name-10", ["P29"], "/topics/*/name", f"{V_TEXT}#CharsAtMost", {"max": 10, "approx": True},
      "「10字くらい」。approx のため、誤りにはならず注意にとどまる"),
    c("open-at-most-5", ["I1"], "/topics", f"{V_CORE}#Count", {"where": OPEN, "max": 5}),
    c("open-needs-queue", ["I2", "P30"], "/", f"{V_CORE}#Implies",
      {"if": {"count": "/topics", "where": OPEN, "min": 1}, "then": {"count": "/queue", "min": 1}}),
    c("no-open-means-all-settled", ["I6"], "/", f"{V_CORE}#Implies",
      {"if": {"count": "/topics", "where": OPEN, "max": 0}, "then": {"all": "/topics", "where": {"status": "settled"}}}),
    c("open-has-image", ["I12", "P36"], "/topics/*", f"{V_CORE}#Implies",
      {"if": OPEN, "then": {"count": "images", "min": 1}}),
    c("code-language-not-text", ["I11", "P78"], "/topics/*/images/*/language", f"{V_CORE}#NotIn",
      {"values": ["text", "plain", "txt", ""], "when": {"kind": "code"}}),
    c("language-known", ["P77", "P87"], "/topics/*/images/*/language", f"{V_CORE}#InList",
      {"list": "languages"}, "一覧はまだ無い。test は「知っている名前か」"),
    c("mentions-declared", ["I9"], "/topics/*", f"{V_BS}#MentionedTopicsDeclared"),
    c("sentence-120", ["I10", "P32"], "/topics/*/{answer/text,answer/details/*,rejected/*/reason}",
      f"{V_TEXT}#SentenceAtMost", {"max": 120},
      "P32 の test は欄全体の120字だが、実際に検査しているのは1文の長さ（inspect.rs:105）。試案は実際の検査に合わせた"),
    c("open-used-somewhere", ["I13"], "/topics/*", f"{V_BS}#UsedDownstreamOrOutside",
      {"when": OPEN}),
    c("tree-depth-step", ["P94"], "/topics/*/images/*/rows", f"{V_BS}#TreeDepthStep"),
    c("svg-checked", ["P67"], "/topics/*/images/*/svg", f"{V_BS}#CheckedSvg"),
    c("html-standalone", ["P71"], "/topics/*/images/*/html", f"{V_BS}#StandaloneHtml"),
    c("verification-hidden", ["P52"], "/topics/*/verification", f"{V_BS}#NotRendered"),
    c("decisions-only", ["P1", "G2"], "/topics/*/question", f"{V_JUDGE}#Judgment",
      {"question": "問いに、調べれば分かることが入っていないか"}),
    c("not-make-user-choose", ["G1"], "/topics/*", f"{V_JUDGE}#Judgment",
      {"question": "案を並べて利用者に選ばせていないか（答えは1つか）"}),
    c("flag-rare", ["P11"], "/flag", f"{V_JUDGE}#Judgment",
      {"question": "ほかの作業を止めているときだけ true か"}),
    c("one-decision", ["P27"], "/topics/*/answer/text", f"{V_JUDGE}#Judgment",
      {"question": "答えの1文が1つのことを決めているか"}),
    c("decidable", ["P31"], "/topics/*", f"{V_JUDGE}#Judgment",
      {"question": "答えの1文が問いに答えているか"}),
    c("image-kind-by-content", ["P37"], "/topics/*/images/*", f"{V_JUDGE}#Judgment",
      {"question": "code の完成イメージが、ファイルとして成り立つ本文か"}),
    c("assumption-one-each", ["P38", "P56"], "/topics/*/{assumptions,verification/assumptions}/*",
      f"{V_JUDGE}#Judgment", {"question": "1件に1つのことか"}),
    c("screen-height-fits", ["P72"], "/topics/*/images/*/height", f"{V_JUDGE}#VisualJudgment",
      {"question": "中身が収まる高さか"}),
    c("code-is-real-file", ["P80"], "/topics/*/images/*/text", f"{V_JUDGE}#Judgment",
      {"question": "path に書いたファイルの本文として成り立つか"}),
    c("highlight-few", ["P81"], "/topics/*/images/*/highlight", f"{V_JUDGE}#Judgment",
      {"question": "全体の行の数より十分少ないか"}),
]
# 「見せたい部分だけ」（60行くらいまで）
only_needed = [r["id"] for r in targets if r["rule"].startswith("only-needed：")]
constraints.append(c("lines-60", only_needed, "（各欄。inventory の行の where）", f"{V_TEXT}#LinesAtMost",
                     {"max": 60, "approx": True}))
# caption の1文（one-thing と one-sentence は同じ caption に同じ test を持つ）と、note の1文
one_sentence = [r["id"] for r in targets
                if r["rule"].split("：")[0] in ("one-thing", "one-sentence", "why-highlight") and r["id"] != "P7"]
constraints.append(c("caption-and-note-one-sentence", one_sentence,
                     "/topics/*/images/*/{caption,note,rows/*/note}", f"{V_TEXT}#OneSentence",
                     note="one-thing と one-sentence は同じ caption に同じ test（1文か）を持つので、1つの制約にまとめた"))

UNPLACED_REASON = {
    "ほかのファイル": "検査にインスタンスの外のファイル（ファイル名 ・ リポジトリ ・ ほかのボード）を必要とする。v0.3 の要求の例は instance を1件しか渡さない",
    "前の回": "検査に前の回の記録を必要とする。v0.3 の試案には、版や履歴を渡す置き場所が無い",
    "誰が書いたか": "誰が書いたか ・ どう書いたかはインスタンスに残らない",
    "実行と時間": "プロセス ・ 時間 ・ 実行の結果の決まり。docs/14 は時間の制約を「そのまま表現できることを目指す（形式は未決）」とする",
    "導く決まり": "値を導く決まり（保存しない値）。v0.3 の試案の operation は validate だけが例にあり、導く操作が無い",
}
unplaced = [
    ("P5", "ほかのファイル"), ("P12", "ほかのファイル"), ("P76", "ほかのファイル"), ("P86", "ほかのファイル"),
    ("P95", "ほかのファイル"), ("P88", "ほかのファイル"), ("P89", "ほかのファイル"), ("P102", "ほかのファイル"),
    ("P13", "前の回"), ("P18", "前の回"), ("G3", "前の回"),
    ("P8", "誰が書いたか"), ("G4", "誰が書いたか"), ("G5", "誰が書いたか"),
    ("P103", "実行と時間"), ("P104", "実行と時間"), ("L1", "実行と時間"), ("L2", "実行と時間"), ("L3", "実行と時間"),
    ("G6", "実行と時間"), ("G9", "実行と時間"), ("G10", "実行と時間"), ("D2", "実行と時間"),
    ("D1", "導く決まり"), ("D3", "導く決まり"), ("D4", "導く決まり"), ("D5", "導く決まり"),
]
other_contracts = ["L4"]

# ---- どの決まりも1か所にだけ入っているかを確かめる ------------------------------------
placed = {}
def put(rid, where):
    if rid in placed:
        raise SystemExit(f"{rid} が2か所にある: {placed[rid]} と {where}")
    placed[rid] = where
for rid in data_schema: put(rid, "dataSchema")
for rid in protocol: put(rid, "protocol")
for k in constraints:
    for rid in k["inventory"]: put(rid, "constraints:" + k["id"])
for rid, _ in unplaced: put(rid, "unplaced")
for rid in other_contracts: put(rid, "concrete.lock")
missing = [r["id"] for r in targets if r["id"] not in placed]
extra = [rid for rid in placed if rid not in {r["id"] for r in targets}]
if missing or extra:
    raise SystemExit(f"置き忘れ {missing} ／ 余分 {extra}")

for k in constraints:
    k["source"] = [rows[rid]["source"] for rid in k["inventory"]]

concrete = {
    "kind": "contract",
    "id": "urn:daidaiiro:contract:brainstorming-board:0-trial",
    "_note": TRIAL + " 形（JSON Schema）はいまのスキーマを名前で引き、写さない。",
    "uses": [V_TEXT, V_CORE, V_BS, V_JUDGE],
    "dataSchema": {"$ref": "brainstorming/board@v1"},
    "coveredByDataSchema": {
        "inventory": data_schema,
        "note": "いまのスキーマの形（const ・ pattern ・ enum ・ required ・ additionalProperties ・ maxItems）と x-ref がそのまま持つ決まり",
    },
    "semanticBindings": {
        "/topics/*/id": {"term": f"{V_BS}#TopicId"},
        "/topics/*/status": {"term": f"{V_BS}#TopicStatus"},
        "/topics/*/depends/*": {"term": f"{V_BS}#TopicId"},
        "/queue/*/topic": {"term": f"{V_BS}#TopicId"},
        "/changes/*/topic": {"term": f"{V_BS}#TopicId"},
    },
    "constraints": constraints,
    "coveredByProtocol": {
        "inventory": protocol,
        "note": "「AI の検査したで合格としない」は、決まりではなく Interpreter の応答（status ・ providedCapabilities）の性質になる",
    },
    "unplaced": [{"inventory": rid, "rule": rows[rid]["rule"], "source": rows[rid]["source"],
                  "reason": UNPLACED_REASON[why], "why": why} for rid, why in unplaced],
}

concrete_lock = {
    "kind": "contract",
    "id": "urn:daidaiiro:contract:brainstorming-lock:0-trial",
    "_note": TRIAL + " いまはスキーマが無く、形が lock.rs:190 の中にだけある（inventory L4）。時間の決まり L1〜L3 は置けない。",
    "inventory": ["L4"],
    "dataSchema": {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object", "additionalProperties": False,
        "required": ["board", "owner", "what", "since", "beat"],
        "properties": {"board": {"type": "string"}, "owner": {"type": "string"},
                       "what": {"type": "string"}, "since": {"type": "string"},
                       "beat": {"type": "integer", "minimum": 0}},
    },
}

# ---- Interpreter と Capability ------------------------------------------------------
interpreters = {
    "_note": TRIAL + " present は、この実験のときに動くものがあるか。",
    "interpreters": [
        {"id": "schema-driven-validate", "implementation": ".claude/skills/brainstorming/scripts/bin/schema-driven（update ・ check）",
         "providedCapabilities": ["validate-structure"], "present": True},
        {"id": "schema-driven-check-xref", "implementation": "schema-driven check（x-ref の to ・ in ・ when ・ accept）",
         "providedCapabilities": ["validate-reference"], "present": True,
         "note": "xref-trial で queue-points-open と open-depends-settled が動いた"},
        {"id": "brainstorming-inspect", "implementation": "brainstorming inspect（inspect.rs）",
         "providedCapabilities": ["validate-ja-sentence", "extract-topic-mentions", "validate-downstream-or-outer"],
         "present": True,
         "note": "validate-ja-sentence は SentenceAtMost だけで、OneSentence は持たない。対象の欄も inspect.rs:99 の4つに決まっている"},
        {"id": "schema-driven-render", "implementation": "schema-driven render（contentMediaType image/svg+xml）",
         "providedCapabilities": ["check-svg"], "present": True},
        {"id": "constraint-core", "implementation": "無い",
         "providedCapabilities": ["validate-count", "validate-implies", "validate-unique-key", "validate-order", "validate-lines"],
         "present": False,
         "note": "count と implies の一部は、標準の JSON Schema のキーワード（minItems ・ maxItems ・ maxContains ・ if/then）へ射影して jsonschema-trial で動かした"},
        {"id": "ai-reviewer", "implementation": "AI（x-prompt の test を読ませる）", "providedCapabilities": ["judge-meaning"],
         "present": False, "note": "いまは書く AI が自分で読むだけで、検査として呼ばれていない"},
        {"id": "human", "implementation": "人", "providedCapabilities": ["judge-visual"], "present": False},
    ],
    "capabilitiesWithoutInterpreter": ["validate-ja-sentence（OneSentence）", "validate-tree-depth",
                                       "check-html-no-external", "check-rendered-page"],
}

# ---- 実物のボードへの要求と応答 ----------------------------------------------------------
board = ".brainstorming/ai-driven-process.json"
measured = json.load(open(os.path.join(HERE, "measure_unchecked.out.json"), encoding="utf-8"))["ai-driven-process"]
present_caps = {cap for i in interpreters["interpreters"] if i["present"] for cap in i["providedCapabilities"]}
vocab_terms = {}
for v in vocabularies.values():
    for name, t in v["terms"].items():
        vocab_terms[f"{v['id']}#{name}"] = t
required_caps = sorted({cap for k in constraints for cap in vocab_terms[k["term"]]["requires"]})

# この実験で実物のボード（の写し）に対して実際に動かした結果。動かしていない制約は unverified にする。
RAN = {
    "queue-points-open": ("verified", "xref-trial/boards/base.json に x-ref の accept で check（ずれ0件）"),
    "open-depends-settled": ("verified", "xref-trial/boards/base.json に x-ref の when ・ accept で check（ずれ0件）"),
    "sentence-120": ("verified", "brainstorming inspect（findings 0件）"),
    "mentions-declared": ("verified", "brainstorming inspect（findings 0件）"),
    "open-used-somewhere": ("verified", "brainstorming inspect（findings 0件）"),
    "rejected-at-least-one": ("verified", "jsonschema-trial/boards/real-as-is.json（標準のキーワードへ射影）"),
    "title-40": ("verified", "jsonschema-trial/boards/real-as-is.json（標準のキーワードへ射影）"),
    "open-at-most-5": ("verified", "jsonschema-trial/boards/real-as-is.json（標準のキーワードへ射影）"),
    "open-needs-queue": ("verified", "jsonschema-trial/boards/real-as-is.json（標準のキーワードへ射影）"),
    "open-has-image": ("verified", "jsonschema-trial/boards/real-as-is.json（標準のキーワードへ射影）"),
    "code-language-not-text": ("verified", "jsonschema-trial/boards/real-as-is.json（標準のキーワードへ射影）"),
    "topics-3-to-5": ("violated", "jsonschema-trial/boards/real-as-is.json：要素の数が maxItems の 5 より多い"),
}
NOT_RUN = {
    "language-known": ["言語の名前の一覧（languages）が無い"],
    "topic-name-10": ["目安（approx）の線を検査する Interpreter が無い"],
    "svg-checked": ["この実験では render を動かしていない"],
    "caption-and-note-one-sentence": ["validate-ja-sentence（OneSentence）"],
    "intro-one-sentence": ["validate-ja-sentence（OneSentence）"],
}
results = []
for k in constraints:
    if k["id"] in RAN:
        status, evidence = RAN[k["id"]]
        results.append({"constraint": k["id"], "status": status, "evidence": evidence})
        continue
    reason = [x for x in NOT_RUN.get(k["id"], []) if not x.startswith("validate-")]
    lacking = [x for x in NOT_RUN.get(k["id"], []) if x.startswith("validate-")] or \
        [cap for cap in vocab_terms[k["term"]]["requires"] if cap not in present_caps]
    if reason:
        results.append({"constraint": k["id"], "status": "unverified", "reason": reason})
        continue
    if not lacking:
        raise SystemExit(f"{k['id']}：動かしていないのに、足りない Capability が無い")
    results.append({"constraint": k["id"], "status": "unverified", "missingCapabilities": lacking})
# 測った値（measure_unchecked.py）を、Interpreter が無い制約の「参考」として添える
reference = {
    "topics-3-to-5": measured["topics-3-to-5（件数）"],
    "title-40": measured["title-40字まで"],
    "intro-one-sentence": measured["intro-1文"],
    "topic-id-unique": measured["topic.id-重ならない"],
    "log-newest-first": measured["log-新しい順"],
    "caption-and-note-one-sentence": measured["caption-1文"],
}
for r in results:
    if r["constraint"] in reference:
        r["measuredBy"] = {"tool": "measure_unchecked.py（この実験の書き捨て。Interpreter として登録していない）",
                           **reference[r["constraint"]]}

request = {"_note": TRIAL, "protocol": "semantic.interpreter/0-draft", "operation": "validate",
           "vocabulary": [V_TEXT, V_CORE, V_BS, V_JUDGE], "contract": concrete["id"],
           "requiredCapabilities": required_caps, "instance": {"$path": board, "_note": "本文は写さず、場所で指す"}}
unverified = [r for r in results if r["status"] == "unverified"]
violated = [r for r in results if r["status"] == "violated"]
response = {
    "_note": TRIAL + " results の evidence は、この実験で実物のボード（またはその写し）に対して実際に動かした検査。動かしていない制約は unverified にした。",
    "protocol": "semantic.interpreter/0-draft",
    "status": "violated" if violated else "unverified",
    "providedCapabilities": sorted(present_caps),
    "missingCapabilities": sorted({cap for r in unverified for cap in r.get("missingCapabilities", [])}),
    "counts": {"verified": len(results) - len(unverified) - len(violated), "unverified": len(unverified), "violated": len(violated)},
    "results": results,
    "diagnostics": [
        {"code": "CONSTRAINT_VIOLATED", "path": "/topics", "constraint": "topics-3-to-5", "severity": "error",
         "message": "論点が6件ある（3〜5件）。いまの inspect と check はこれを見ていない"},
        {"code": "CAPABILITY_UNAVAILABLE", "path": "/topics/*/question", "constraint": "decisions-only", "severity": "warning",
         "message": "judge-meaning の Interpreter が無いので未検証。必須の意味なので、全体を合格にしない"},
    ],
}

# ---- 標準の JSON Schema 2020-12 への射影 ---------------------------------------------
to_standard = {
    "rejected-at-least-one": "minItems（jsonschema-trial で動いた）",
    "title-40": "maxLength（jsonschema-trial で動いた。approx は落ちる）",
    "topics-3-to-5": "minItems ・ maxItems（jsonschema-trial で動いた）",
    "open-at-most-5": "contains ・ maxContains（一時の場所で動かした）",
    "open-needs-queue": "if/then と contains（jsonschema-trial で動いた）",
    "open-has-image": "if/then（jsonschema-trial で動いた）",
    "code-language-not-text": "not と enum（jsonschema-trial で動いた。小文字にする操作は落ちる）",
    "no-open-means-all-settled": "anyOf と contains（試していない）",
}
projection = {
    "_note": TRIAL,
    "target": "json-schema-2020-12",
    "coverage": "partial",
    "lossless": False,
    "represented": [{"constraint": k, "as": v} for k, v in to_standard.items()],
    "notRepresented": [k["id"] for k in constraints if k["id"] not in to_standard],
    "diagnostics": [
        {"code": "PROJECTION_LOSSY", "constraint": "title-40", "message": "approx（目安）は標準のキーワードに無い"},
        {"code": "PROJECTION_LOSSY", "constraint": "code-language-not-text", "message": "値を小文字にしてから比べる操作は標準のキーワードに無い"},
        {"code": "PROJECTION_UNSUPPORTED", "constraint": "queue-points-open",
         "message": "ほかの要素の値を見る参照は標準のキーワードに無い。いまの基盤では x-ref（独自の注釈）で書けた"},
    ],
}

os.makedirs(OUT, exist_ok=True)
files = {
    "vocabulary.text-ja.trial.json": vocabularies["text-ja"],
    "vocabulary.constraint-core.trial.json": vocabularies["constraint-core"],
    "vocabulary.brainstorming.trial.json": vocabularies["brainstorming"],
    "vocabulary.judgment.trial.json": vocabularies["judgment"],
    "concrete.board.trial.json": concrete,
    "concrete.lock.trial.json": concrete_lock,
    "interpreters.trial.json": interpreters,
    "interpreter-request.trial.json": request,
    "interpreter-response.trial.json": response,
    "projection-report.trial.json": projection,
}
for name, body in files.items():
    with open(os.path.join(OUT, name), "w", encoding="utf-8") as f:
        json.dump(body, f, ensure_ascii=False, indent=2)
        f.write("\n")
print(json.dumps({"targets": len(targets), "dataSchema": len(data_schema), "constraints": len(constraints),
                  "constraint_rows": sum(len(k["inventory"]) for k in constraints), "protocol": len(protocol),
                  "unplaced": len(unplaced), "lock": len(other_contracts),
                  "response": response["counts"], "missing": response["missingCapabilities"],
                  "projection_represented": len(to_standard), "projection_not": len(projection["notRepresented"])},
                 ensure_ascii=False, indent=1))
