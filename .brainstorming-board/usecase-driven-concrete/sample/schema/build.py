# 宣言の種類ごとの JSON Schema を組む。共通の形（条件 ・ 閾値 ・ 参照）は common.schema.json に1つだけ置く
# 各項目は description と x-prompt（read と write）を持つ。x-prompt.write は、このユースケース駆動のやり方で書くための基準で、
# 助言役なしで欄を書ける内容にする（ACDR 0109）。文は書かず、道具が欄から組む
import json, os
H = os.path.dirname(os.path.abspath(__file__))
S = "https://json-schema.org/draft/2020-12/schema"
C = "common.schema.json#/$defs/"


def F(desc, read, write, schema=None, **kw):
    """項目1つ。description と x-prompt を持たせる"""
    o = dict(schema or {})
    o.update(kw)
    o["description"] = desc
    o["x-prompt"] = {"read": read, "write": write}
    return o


def ref(name): return {"$ref": C + name}
def arr(items, **kw): return dict({"type": "array", "items": items}, **kw)
def obj(props, req=None, **kw):
    o = {"type": "object", "additionalProperties": False, "properties": props}
    if req: o["required"] = req
    o.update(kw)
    return o
def pat(p): return {"type": "string", "pattern": p}
def txt(): return {"type": "string", "minLength": 1}


ID = lambda pre: pat(f"^{pre}-[0-9]+$")
HEADER_IN_CONTEXT = obj({
    "name": F("名前。用語集の語の ID", "用語集の語として読む", "用語集に語を足してから、その ID を書く。語を新しく作らない", ref("term")),
    "context": F("所属する区切られた文脈", "この宣言が属する区切られた文脈", "BC の ID を書く", ID("BC"))}, ["name", "context"])
HEADER_IN_CONTEXT_DESC = ("名前と、所属する区切られた文脈", "名前は用語集の語、文脈は BC の ID である", "名前は用語集の語の ID、文脈は BC の ID で書く")


def kind_schema(kind, title, desc, props, req, defs=None, idpre=None):
    p = {"$schema": {"type": "string"},
         "kind": F("宣言の種類", "種類で読む欄が決まる", f"{kind} と書く", {"const": kind}),
         "id": F("宣言の ID", "ほかの宣言はこの ID で指す", f"{idpre}-番号 の形で書く。番号は増える一方にし、消した ID を再び使わない", ID(idpre))}
    p.update(props)
    s = {"$schema": S, "$id": f"{kind}.schema.json", "x-generates": f"decls/{idpre}-<番号>.json",
         "title": title, "description": desc, "type": "object", "additionalProperties": False,
         "required": ["kind", "id"] + req, "properties": p}
    if defs: s["$defs"] = defs
    return s


# ── 共通の形
TERM = "^TERM-[0-9]+$"
REF = "^[A-Z]+-[0-9]+(\\.[A-Z]+-[0-9]+)*$"
common = {"$schema": S, "$id": "common.schema.json", "title": "宣言の共通の形",
          "description": "条件 ・ 閾値 ・ 参照の形。どの種類の宣言も、ここを参照する。キーは ASCII、画面へ出す語は道具が持つ",
          "$defs": {
    "term": F("用語集の語の ID", "用語集で語と意味を引く", "用語集に在る語の ID を書く。無ければ先に用語集へ足す", pat(TERM)),
    "term_path": F("用語集の語の並び（例 TERM-1.TERM-5）", "左から順に、集約 → 状態 → エンティティの状態と読む", "ユースケースとその他の要求の条件はこの形で書く。内部の ID（AGG ・ ST など）を書かない", pat("^TERM-[0-9]+(\\.TERM-[0-9]+)*$")),
    "ref": F("宣言や項目への参照（例 AGG-1.CMD-1.BR-2）", "左から宣言 → 項目と読む", "指す先が在る ID を、宣言の ID から書く", pat(REF)),
    "local": F("同じ宣言の中の項目の ID（例 ST-2 ・ ARG-1 ・ CMP-1）", "同じ宣言の中で引く", "同じ宣言に在る ID を書く", pat("^[A-Z]+-[0-9]+(\\.[A-Z]+-[0-9]+)?$")),
    "value": F("条件の値", "数 ・ 語 ・ 文字列 ・ 実行前の値 ・ 値オブジェクトの操作の結果のどれか", "語は用語集の ID、引数は ARG の ID で書く。式を書かない ── 計算は値オブジェクトの操作を1回呼ぶ形（call と args）にする",
               {"anyOf": [{"type": "number"}, {"type": "string"}, {"type": "array", "items": {"type": "string"}},
                          obj({"before": {"type": "string"}, "agg": {"enum": ["count", "sum"]}}, ["before"]),
                          obj({"call": pat("^VO-[0-9]+\\.OP-[0-9]+$"), "args": {"type": "array"}}, ["call", "args"])]}),
    "cond_one": F("条件1つ（if を持たない）", "対象 ・ 比べ方 ・ 値の3つで読む", "下の condition と同じ書き方をする",
                  obj({"target": {"type": "string"}, "op": {"enum": ["eq", "ne", "ge", "le", "gt", "not_empty", "empty", "ends_with"]},
                       "value": {"$ref": "#/$defs/value"}, "agg": {"enum": ["count", "sum"]}, "measure": {"enum": ["length"]}}, ["target", "op"])),
    "condition": F("条件。文は道具が組む", "対象 ・ 比べ方 ・ 値と、在れば前提（if）で読む",
                   "対象は、ユースケースとその他の要求では用語集の語の並び、設計の側では同じ宣言の中の ID で書く。"
                   "比べ方は eq（同じ）・ ne（違う）・ ge（以上）・ le（以下）・ gt（より大きい）・ not_empty ・ empty ・ ends_with（どれかで終わる。値は文字列の並び）。"
                   "件数と合計は agg（count ・ sum）に書く。1つの値の文字数は measure: length に書き、コードポイントで数える ── agg に入れない（agg は複数の値を1つにまとめる計算である）。"
                   "「〜してはならない」は否定の文にせず、成り立つ状態を条件で書く",
                   obj({"target": {"type": "string"}, "op": {"enum": ["eq", "ne", "ge", "le", "gt", "not_empty", "empty", "ends_with"]},
                        "value": {"$ref": "#/$defs/value"}, "agg": {"enum": ["count", "sum"]}, "measure": {"enum": ["length"]},
                        "if": {"$ref": "#/$defs/cond_one"}}, ["target", "op"])),
    "threshold": F("閾値（op ・ value ・ unit）", "比べ方と値と単位で読む", "単位を必ず書く。比べ方は le か ge",
                   obj({"op": {"enum": ["le", "ge", "lt", "gt"]}, "value": {"type": "number"}, "unit": txt()}, ["op", "value", "unit"])),
    "ids": F("ID の並び", "指す先を順に読む", "指す先が在る ID だけを書く", arr(pat(REF))),
}}


# ── 文の型（ボード schema-driven-base の論点3を試す）。条件と値を、注釈 x-view の文の型だけで文にする
common["x-view-name"] = ["word", "name", "header/name"]
common["x-view-arg"] = "指定された"  # コマンドの引数を指す名前の前に付ける
common["x-view-words"] = {"RESULT": "結果"}  # 欄の値に書く決まった語
common["x-view-glossary"] = {"kind": "glossary", "in": "terms", "label": "word", "meanings": "meanings", "meaning_key": "kind"}  # 語を引く先
OPS = {"eq": {"then_if": {"number": True, "meaning": ["状態の値"]}, "then": "は{value|text:value}{measure|map:unit}", "else": "は{value|text:value}と同じ"},
       "ne": "は{value|text:value}ではない", "ge": "は{value|text:value}{measure|map:unit}以上", "le": "は{value|text:value}{measure|map:unit}以下",
       "gt": "は{value|text:value}より大きい", "not_empty": "は空でない", "empty": "は空", "ends_with": "は{value|text:value}のどれかで終わる"}
VIEW_ONE = {"text": "{target|name}{agg|map:agg}{measure|map:measure}{op|ops}", "ops": OPS,
            "maps": {"agg": {"count": "の件数", "sum": "の合計"}, "measure": {"length": "の文字数"}, "unit": {"length": "字"}}}
common["$defs"]["cond_one"]["x-view"] = VIEW_ONE
common["$defs"]["condition"]["x-view"] = dict(VIEW_ONE, text="{if|text:cond_one|clause|suffix:なら、}" + VIEW_ONE["text"])
common["$defs"]["value"]["x-view"] = {"by_type": {"number": "{v}", "list": "{v|quote}", "string": "{v|name}"},
    "by_key": {"before": "実行前の{before|name}{agg|map:agg}", "call": "{args.0|text:value}と{args.1|text:value}を「{call|bare}」で求めた値"},
    "maps": {"agg": {"count": "の件数", "sum": "の合計"}}}

# ── ドメイン（作りたいプロダクト）
domain = kind_schema("domain", "ドメイン", "作りたいプロダクト。ビジョン記述 ・ 利害関係者と利益 ・ スコープの外 ・ 設計スコープを持つ。業務領域の一覧と品質の要求は持たない（設計の側とその他の要求が持つ）", {
    "header": F("名前", "プロダクトの名前", "利用者に通じるプロダクトの名前を書く", obj({"name": txt()}, ["name"])),
    "vision": F("ビジョン記述", "課題 ・ 提供価値 ・ 競合 ・ 達成の基準を1か所で読む",
                "課題は誰が何に困っているかを1文で書く。提供価値は利用者が受け取る結果で書き、競合との違いになるものに differentiator: true を付ける。"
                "競合は、どの提供価値を欠くか（lacks）で書く。達成の基準は、測る対象 ・ 閾値 ・ 割合 ・ 期間を欄に分ける",
                obj({"problem": txt(),
                     "values": arr(obj({"id": ID("VAL"), "name": txt(), "text": txt(), "differentiator": {"type": "boolean"}}, ["id", "name", "text", "differentiator"])),
                     "competitors": arr(obj({"id": ID("CMP"), "name": txt(), "lacks": arr(ID("VAL"))}, ["id", "name", "lacks"])),
                     "success_criteria": arr(obj({"id": ID("SC"), "name": txt(), "source": {"enum": ["event", "external"]}, "target": pat(TERM),
                                                   "measure": obj({"diff": arr({"type": "string"})}), "measure_text": txt(),
                                                   "threshold": ref("threshold"), "ratio": ref("threshold"), "window": txt()},
                                                  ["id", "name", "source", "threshold", "window"]))},
                    ["problem", "values", "competitors", "success_criteria"])),
    "scope": F("スコープの外（Out）", "プロダクトが扱わないもの", "扱わないものを名詞で並べる。まだ移していないものとは分けて書く（移していないものは区切られた文脈の外の相手として書く）。In は道具がユーザー目的のユースケースから組むので書かない",
               obj({"out": arr(txt())}, ["out"])),
    "design_scopes": F("設計スコープ", "企業の高さとシステムの高さで、何が内で何が外かを読む",
                       "システムの高さを1つ必ず書く。ユースケースのスコープはここを指す。内と外は利害関係者の ID かシステムの名前で書く",
                       arr(obj({"id": ID("SCP"), "level": {"enum": ["企業", "システム"]}, "name": txt(), "inside": arr({"type": "string"}), "outside": arr({"type": "string"})}, ["id", "level", "name", "outside"]))),
    "stakeholders": F("利害関係者と利益", "誰が何を守りたいか", "利益は、その人が守りたいことを1文で書く。ユースケースの手順と保証は、ここの ID を指す",
                      arr(obj({"id": ID("SH"), "who": txt(), "interest": txt()}, ["id", "who", "interest"])))},
    ["header", "vision", "scope", "design_scopes", "stakeholders"], idpre="DOM")

# ── 用語集
glossary = kind_schema("glossary", "用語集", "プロダクトに1つ。語ごとに意味の一覧と、使わない語を持つ。宣言の名前と条件は、ここの語を指す", {
    "header": F("名前", "用語集の名前", "「用語集」と書く", obj({"name": txt()}, ["name"])),
    "terms": F("語の一覧", "語 → 意味の一覧 → 意味ごとの種類の順に読む",
               "1つの語に意味が2つ以上あれば、意味を分けて書き、語の頭に文脈名を付けない。区切られた文脈は、どの意味を使うかを指す。"
               "意味の kind は、その語が宣言のどこで使われるか（集約 ・ 状態 ・ 状態の値 ・ 値オブジェクト ・ コマンド ・ 拒否の理由 ・ 情報の別名 ・ 動作 など）を書く。"
               "避けたい言い換えは avoid に並べる。語を新しく作らない ── 業務の人がふだん使う語を書く",
               arr(obj({"id": pat(TERM), "word": txt(),
                        "meanings": arr(obj({"id": ID("M"), "definition": txt(), "kind": txt(), "form": txt(), "change": txt()}, ["id", "definition", "kind"]), minItems=1),
                        "avoid": arr(txt())}, ["id", "word", "meanings", "avoid"])))},
    ["header", "terms"], idpre="GLO")

# ── その他の要求
other = kind_schema("other_requirements", "その他の要求", "ユースケースの外に書く要求。ユースケースは関連情報（links）で、ここの項目を指す", {
    "header": F("名前", "名前", "「その他の要求」と書く", obj({"name": txt()}, ["name"])),
    "business_rules": F("ビジネスルール", "シナリオの形に収まらない業務の決まり", "条件は用語集の語の並びで書き、守る利害関係者の ID を protects に書く。集約の業務ルールが実装するときは、集約の側から implements で指す",
                        arr(obj({"id": ID("BR"), "name": txt(), "condition": ref("condition"), "protects": ref("ids")}, ["id", "name", "condition", "protects"]))),
    "quality": F("品質の要求", "どの手順の、何を、どの条件で測るか", "対象の手順（UC-n.STEP-n）・ 測るもの ・ 閾値 ・ 割合 ・ 期間と負荷 ・ 測り方を欄に分ける",
                 arr(obj({"id": ID("QR"), "target": ref("ref"), "measure": txt(), "threshold": ref("threshold"), "ratio": ref("threshold"),
                          "condition": obj({"period": txt(), "load": obj({"value": {"type": "number"}, "unit": txt()}, ["value", "unit"])}, ["period", "load"]),
                          "grade": obj({"item": txt(), "level": {"type": "integer"}}), "method": txt()}, ["id", "target", "measure", "threshold", "ratio", "condition", "method"]))),
    "technology": F("使われる技術", "相互作用するシステムと、その要求", "相手のシステムの名前と、こちらに求められることを書く。製品名やコマンドの細部は書かない",
                    arr(obj({"id": ID("TEC"), "system": txt(), "text": txt()}, ["id", "system", "text"]))),
    "data": F("データ要求", "項目ごとの長さ ・ 妥当性の制約", "何文字までか ・ どんな値なら受け付けるかを、条件で書く（文字数は measure: length）。ユースケースの手順には書かず、ユースケースの links.data から結ぶ",
              arr(obj({"id": ID("DAT"), "name": txt(), "condition": ref("condition")}, ["id", "name", "condition"]))),
    "open_issues": F("未決定事項", "まだ決めていないこと", "決めていないことを1件1文で書く。決めたら消し、決めた欄へ移す", arr(txt()))},
    ["header", "business_rules", "quality", "technology", "data", "open_issues"], idpre="REQ")

# ── ユースケース
STEP_KINDS = ["相互作用", "妥当性確認", "内部の状態変化", "サブユースケースの呼び出し"]
step_props = {
    "id": ID("STEP"), "kind": {"enum": STEP_KINDS}, "actor": txt(), "to": txt(),
    "data": arr(pat(TERM)), "verb": pat(TERM), "object": pat(TERM), "reply": arr(pat(TERM)),
    "checks": ref("ids"), "calls": ID("UC"), "keeps": arr(ID("MG")), "serves": arr(ID("SH")),
    "variations": arr(obj({"varies": txt(), "values": arr(txt())}, ["varies", "values"])),
    "extensions": arr({"$ref": "#/$defs/extension"})}
STEP_GUIDE = {
    "id": ("手順の ID", "番号は道具が並び順から振る", "STEP-番号で書く。並べ替えても ID は変えない"),
    "kind": ("手順の種類", "種類で読む欄が決まる", "相互作用 ・ 妥当性確認 ・ 内部の状態変化 ・ サブユースケースの呼び出しのどれか"),
    "actor": ("主語", "誰が動くか", "アクターの名前か「システム」を書く。主語を省かない"),
    "to": ("相手", "相互作用の相手", "相互作用のときだけ書く"),
    "data": ("渡す情報", "相手に渡す情報の別名", "項目を並べず、用語集の情報の別名の語で書く。項目の一覧はデータ要求に書く"),
    "verb": ("動作", "何をするか", "用語集の動作の語で書く。画面の操作（押す ・ 選ぶ）を書かない"),
    "object": ("対象", "内部の状態変化で変わるもの", "内部の状態変化のときだけ、用語集の語で書く"),
    "reply": ("返る情報", "相手から返る情報の別名", "支援アクターとの相互作用で返るものを書く"),
    "checks": ("確かめるビジネスルール", "妥当性確認で確かめること", "妥当性確認のときだけ、その他の要求のビジネスルールの ID を書く。条件の中身は手順に書かない"),
    "calls": ("呼ぶユースケース", "サブユースケースの呼び出し先", "サブユースケースの呼び出しのときだけ、UC の ID を書く"),
    "keeps": ("守る最低保証", "この手順が守る最低保証", "失敗しても守ることを作る手順に、MG の ID を書く"),
    "serves": ("役立つ利害関係者の利益", "誰の利益のための手順か", "このユースケースの利害関係者の ID を書く。どの利益にも役立たない手順は書かない"),
    "variations": ("技術およびデータのバリエーション", "同じことを、どのように行うかの違い", "支払い手段のような、同じことのやり方の違いだけを書く。条件や「システムは…する」を持つものは拡張に書く"),
    "extensions": ("この手順の拡張", "この手順で違う振る舞いになる状況", "拡張ごとに条件の種類と終わり方を書く。無ければ空にする")}
for k, v in list(step_props.items()): step_props[k] = F(*STEP_GUIDE[k], v)
sub_props = dict(step_props); sub_props["id"] = F("拡張の中の手順の ID", "番号は道具が振る", "EXT-番号.S-番号 で書く", pat("^EXT-[0-9]+\\.S-[0-9]+$"))
use_case = kind_schema("use_case", "ユースケース", "主アクターが目的を果たすまでの、システムとのやり取り。プロダクトの内部（集約 ・ 文脈）を指さない。設計の側のサブドメインが、このユースケースを束ねる", {
    "header": F("名前 ・ 目的レベル ・ スコープ ・ 主アクター ・ トリガー", "目的レベルとスコープで、どこまでを書くかが決まる",
                "名前は主アクターの目的を「〜を〜する」の形で書く。目的レベルは要約 ・ ユーザー目的 ・ サブ機能のどれか。スコープは設計スコープのシステムの高さ（DOM-n.SCP-n）を指す。トリガーは最初の手順を指す",
                obj({"name": txt(), "level": {"enum": ["要約", "ユーザー目的", "サブ機能"]}, "scope": obj({"system": ref("ref")}, ["system"]),
                     "primary_actor": txt(), "trigger_step": ID("STEP")}, ["name", "level", "scope", "primary_actor", "trigger_step"])),
    "stakeholders": F("このユースケースの利害関係者と利益", "誰の何をこのユースケースが守るか", "ドメインの利害関係者（DOM-n.SH-n）を who に指し、このユースケースで守る利益を1文で書く",
                      arr(obj({"id": ID("SH"), "who": ref("ref"), "interest": txt()}, ["id", "who", "interest"]))),
    "preconditions": F("事前条件", "始まる前に成り立っていること", "条件は用語集の語の並びで書く。どのユースケースが成り立たせるかを established_by に書く。事前条件で防いでいる拒否は、拡張に書かない",
                       arr(obj({"id": ID("PRE"), "condition": ref("condition"), "established_by": {"anyOf": [ID("UC"), {"type": "null"}]}}, ["id", "condition"]))),
    "guarantees": F("保証", "成功時保証は主成功シナリオの行き着く先、最低保証はどの終わり方でも守ること",
                    "成功時保証は「成功したのにこの利害関係者が不満を持つのはどんな場合か」の逆を条件で書く。「〜してはならない」も、成り立つ状態として成功時保証に書き、その状態を作る内部の状態変化の手順を主成功シナリオに置く。"
                    "最低保証には、処理がどこまで進んだかを残すことを必ず候補に入れる",
                    obj({"minimal": arr(obj({"id": ID("MG"), "name": txt(), "condition": ref("condition"), "protects": arr(ID("SH"))}, ["id", "name", "condition", "protects"])),
                         "success": arr(obj({"id": ID("SG"), "name": txt(), "condition": ref("condition"), "satisfies": arr(ID("SH"))}, ["id", "name", "condition", "satisfies"]))}, ["minimal", "success"])),
    "scenario": F("主成功シナリオと拡張", "手順を上から読み、各手順の拡張をその下で読む",
                  "手順は4種類（相互作用 ・ 妥当性確認 ・ 内部の状態変化 ・ サブユースケースの呼び出し）で、文は道具が組む。"
                  "相互作用は主語 ・ 相手 ・ 渡す情報（用語集の情報の別名）・ 動作の語で書く。妥当性確認は確かめるビジネスルールを指す。内部の状態変化は動作と対象の語で書く。"
                  "画面の操作や集約の名前を書かない。手順ごとに、どの利害関係者の利益に役立つかを serves に書く",
                  obj({"supporting_actors": arr(txt()), "steps": arr({"$ref": "#/$defs/step"}, minItems=1)}, ["supporting_actors", "steps"])),
    "contributes_to": F("寄与する達成の基準", "どの達成の基準に効くか", "ドメインの達成の基準の ID を書く", arr(ID("SC"))),
    "open_issues": F("未決定事項", "このユースケースで決めていないこと", "1件1文で書く。決めたら消す", arr(txt())),
    "links": F("関連情報", "その他の要求のどの項目に結ぶか", "ビジネスルール ・ 品質 ・ 技術 ・ データ要求を、その他の要求の ID（REQ-n.BR-n など）で指す",
               obj({"business_rules": ref("ids"), "quality": ref("ids"), "technology": ref("ids"), "data": ref("ids")}))},
    ["header", "stakeholders", "preconditions", "guarantees", "scenario", "contributes_to", "open_issues", "links"], idpre="UC",
    defs={
        "step": F("手順1つ", "種類で読む欄が決まる", "種類ごとに必要な欄だけを書く。1つの手順に動作を2つ入れない", obj(step_props, ["id", "kind", "actor", "extensions"])),
        "sub_step": F("拡張の中の手順1つ", "主成功シナリオの手順と同じ形", "主成功シナリオの手順と同じ書き方をする。ID は EXT-n.S-n", obj(sub_props, ["id", "kind", "actor", "extensions"])),
        "extension": F("拡張1つ", "条件の種類 → 条件 → 手順 → 終わり方の順に読む",
                       "拡張は失敗だけでなく、別の道筋での成功も含む。条件の種類は、妥当性確認の失敗（fails に確かめたビジネスルール）・ 業務ルールの拒否（reasons に拒否の理由の語）・ "
                       "支援アクターの失敗（actor と、在れば reasons に失敗の種類の語）・ 別の道筋での成功（condition に用語集の語の並びの条件）。"
                       "条件や「システムは…する」を持つものは、変化（variations）ではなく拡張に書く。"
                       "終わり方（ending）は4つ ── STEP-n（その手順へ戻る）・ 成功（分岐した手順が直り、元の手順が成功した状態になる。拡張の最後には何も書かない）・ "
                       "終了（別の道筋で成功し、ユースケースは終了する）・ 失敗（最低保証が成り立って終わる）。だめなら次を試す順番は、先に試すものを主成功シナリオに書き、次に試すものを終わり方「成功」の拡張に書く",
                       obj({"id": F("拡張の ID", "ラベル（3a など）は道具が振る", "EXT-番号で書く", ID("EXT")),
                            "condition_kind": F("条件の種類", "どの種類の状況か", "妥当性確認の失敗 ・ 業務ルールの拒否 ・ 支援アクターの失敗 ・ 別の道筋での成功のどれか", {"enum": ["妥当性確認の失敗", "業務ルールの拒否", "支援アクターの失敗", "別の道筋での成功"]}),
                            "fails": F("満たされなかったビジネスルール", "妥当性確認の失敗の中身", "妥当性確認の失敗のときだけ、確かめたビジネスルールの ID を書く", ref("ids")),
                            "reasons": F("理由の語", "拒否の理由か、失敗の種類", "業務ルールの拒否では拒否の理由の語、支援アクターの失敗では失敗の種類の語を書く", arr(pat(TERM))),
                            "actor": F("失敗した支援アクター", "誰の応答が無いか、誤っているか", "支援アクターの失敗のときだけ書く", txt()),
                            "condition": F("別の道筋になる条件", "どんなときに別の道筋で成功するか", "別の道筋での成功のときだけ、用語集の語の並びの条件で書く", ref("condition")),
                            "ending": F("終わり方", "拡張のあと、どうなるか", "STEP-n（その手順へ戻る）・ 成功（元の手順が成功した状態になる）・ 終了（ユースケースは終了する）・ 失敗 のどれか", {"anyOf": [ID("STEP"), {"enum": ["成功", "終了", "失敗"]}]}),
                            "steps": F("拡張の手順", "拡張の中で、誰が何をするか", "主成功シナリオの手順と同じ書き方をする", arr({"$ref": "#/$defs/sub_step"}))}, ["id", "condition_kind", "ending", "steps"],
                           allOf=[{"if": {"properties": {"condition_kind": {"const": "別の道筋での成功"}}}, "then": {"required": ["condition"]}},
                                  {"if": {"properties": {"condition_kind": {"const": "妥当性確認の失敗"}}}, "then": {"required": ["fails"]}},
                                  {"if": {"properties": {"condition_kind": {"const": "業務ルールの拒否"}}}, "then": {"required": ["reasons"]}},
                                  {"if": {"properties": {"condition_kind": {"const": "支援アクターの失敗"}}}, "then": {"required": ["actor"]}}]))})

# ── サブドメイン（問いは各欄の title、判定の決まりは x-derive）
sd_old = json.load(open(os.path.join(H, "subdomain.schema.json"), encoding="utf-8"))
cls = sd_old["properties"]["classification"]; bl = sd_old["properties"]["business_logic"]
for g in (cls, bl): g["additionalProperties"] = False
subdomain = kind_schema("subdomain", "サブドメイン", "業務領域。ユースケースを、同じアクター ・ 外部システム ・ 密接なデータで束ねる。カテゴリーと実装方法は、問いへの答えから道具が導く", {
    "header": F("名前と説明", "業務領域の名前と、何をする領域か", "業務の言葉で名前を付け、説明を1文で書く", obj({"name": txt(), "description": txt()}, ["name", "description"])),
    "classification": F("カテゴリー", "答えから導いたカテゴリーと、宣言したカテゴリーを比べる",
                        "問いはスキーマの各欄の title が持ち、どのサブドメインでも同じである。この宣言は答え（true ・ false）だけを書く。判定は x-derive の決まりで道具が出し、宣言したカテゴリーと違えば検査が止める",
                        cls),
    "business_logic": F("業務ロジックの性質", "答えから導いた実装方法で、設計の側の宣言の形が決まる",
                        "問いには上から順に答える。業務ルールが複雑と答えたら、根拠の条件（集約の不変条件やドメインサービスの結果）の ID を rule_conditions に並べる。実装方法は書かない ── 道具が x-derive で導く",
                        bl),
    "serves_values": F("担う提供価値", "ドメインのどの提供価値を担うか", "ドメインの提供価値の ID を書く", arr(ID("VAL"))),
    "use_cases": F("束ねるユースケース", "この業務領域に入るユースケース", "同じアクター ・ 外部システム ・ 密接なデータを持つユースケースを束ねる。外部のサービスで満たす領域は空にしてよい", arr(ID("UC")))},
    ["header", "classification", "business_logic", "serves_values", "use_cases"], idpre="SD")

# ── 区切られた文脈
context = kind_schema("context", "区切られた文脈", "1つのモデルが通じる範囲。用語集のどの意味を使うかと、外の相手との関係を持つ", {
    "header": F("名前 ・ 目的 ・ 対象とする業務領域", "何を判定するモデルか", "目的は、このモデルが何を決めるかを1文で書き、モデルに入れないものも書く。対象とする業務領域を SD の ID で書く。業務領域は発見し、区切られた文脈は設計するので、対応は一対一とは限らない ── 1つの文脈が複数の業務領域を対象とすることも、1つの業務領域に複数の文脈を作ることもある",
                obj({"name": txt(), "purpose": txt(), "subdomains": arr(ID("SD"))}, ["name", "purpose", "subdomains"])),
    "uses": F("用語集", "この文脈で使う、用語集の語と意味", "語と意味の ID を組で書く。1つの文脈で、同じ語の2つの意味を使わない",
              arr(obj({"term": pat(TERM), "meaning": ID("M")}, ["term", "meaning"]))),
    "context_map": F("文脈の地図", "外の相手ごとの関係", "相手ごとに、パターン（従属 ・ モデル変換装置 など）・ 向き ・ 使う操作と失敗の種類を書く。中核を含む下流の文脈は、モデル変換装置を挟む。まだ移していない部分も、ここに外の相手として書く",
                     obj({"relations": arr(obj({"id": ID("X"), "external": txt(), "owner": txt(), "pattern": txt(), "direction": {"enum": ["上流", "下流"]},
                                                "fulfills": ID("SD"), "case": txt(), "uses": arr(pat(TERM)),
                                                "translates": arr(obj({"theirs": txt(), "ours": pat(TERM)}, ["theirs", "ours"])),
                                                "operations": arr(obj({"id": ID("OP"), "name": pat(TERM), "sends": arr(pat(TERM)), "receives": arr(pat(TERM)),
                                                                       "retry": obj({"safe": {"type": "boolean"}, "key": pat(TERM)}),
                                                                       "failures": arr(obj({"id": ID("F"), "name": pat(TERM)}, ["id", "name"]))}, ["id", "name", "failures"]))},
                                               ["id", "external", "owner", "pattern", "direction"]))}, ["relations"])),
    "business_rules": F("文脈の業務ルール", "集約に収まらない、この文脈の業務ルール", "その他の要求のビジネスルールを実装するなら implements で指す",
                        arr(obj({"id": ID("BR"), "condition": ref("condition"), "implements": ref("ref")}, ["id", "condition"]))),
    "boundary": F("境界", "何として作り、誰が持つか", "作り方（自社で作るサービス など）と持ち主を書く", obj({"kind": txt(), "owner": txt()}, ["kind", "owner"])),
    "published_language": F("公開された言語", "外へ出す情報の形", "在るときだけ書く", arr({"type": "object"}))},
    ["header", "uses", "context_map", "boundary"], idpre="BC")

# ── 集約
agg = kind_schema("aggregate", "集約", "一貫性を守る単位。状態 ・ 不変条件 ・ コマンド（業務ルール ・ 状態の変更 ・ 業務イベント ・ 受け付ける例）を持つ", {
    "header": F(*HEADER_IN_CONTEXT_DESC, HEADER_IN_CONTEXT),
    "structure": F("状態とエンティティ", "集約が持つ状態と、その中のエンティティ", "状態ごとに名前（用語集の語）・ 型（VO-n か ID）・ 多重度を書く",
                   obj({"state": arr(obj({"id": ID("ST"), "name": pat(TERM), "type": {"type": "string"},
                                          "multiplicity": obj({"min": {"type": "integer"}, "max": {"type": ["integer", "null"]}}, ["min", "max"])}, ["id", "name", "type", "multiplicity"])),
                        "entities": arr(obj({"id": ID("ENT"), "name": pat(TERM), "state": arr(obj({"id": ID("ES"), "name": pat(TERM), "type": {"type": "string"}}, ["id", "name", "type"]))}, ["id", "name", "state"]))},
                       ["state", "entities"])),
    "invariants": F("不変条件", "コマンドのあとのどの状態でも成り立つ決まり", "条件は同じ集約の状態の ID で書く。破りうるコマンドを via に書く。1つの操作の入力と結果の関係は、不変条件ではなく保証に書く",
                    arr(obj({"id": ID("INV"), "condition": ref("condition"), "via": arr(ID("CMD"))}, ["id", "condition"]))),
    "commands": F("コマンド", "状態を変える操作",
                  "業務ルール（BR-n）は、受け付ける条件と拒否の理由の語で書く。状態の変更（CHG-n）は、変わったあとの状態を条件で書く。"
                  "業務イベントは、渡す状態を fields に書く。受け付ける例（OK-n）は、前の状態と引数を書く ── 後の状態は道具が状態の変更から導く。"
                  "引数を対象にした業務ルールの拒否の例も道具が組むので、組めないときだけ example を書く",
                  arr({"$ref": "#/$defs/command"}))},
    ["header", "structure", "invariants", "commands"], idpre="AGG",
    defs={"command": F("コマンド1つ", "引数 → 業務ルール → 状態の変更 → 業務イベントの順に読む", "名前は用語集のコマンドの語で書く",
                       obj({"id": F("コマンドの ID", "ほかの宣言は AGG-n.CMD-n で指す", "CMD-番号で書く", ID("CMD")),
                            "name": F("コマンドの名前", "用語集のコマンドの語", "用語集のコマンドの語の ID を書く", pat(TERM)),
                            "args": F("引数", "外から渡されるもの", "名前は用語集の語、型は VO-n か ID で書く", arr(obj({"id": ID("ARG"), "name": pat(TERM), "type": {"type": "string"}}, ["id", "name", "type"]))),
                            "business_rules": F("業務ルール", "受け付ける条件と、拒否の理由", "受け付ける条件を書き、満たさないときの拒否の理由を用語集の語で書く。その他の要求のビジネスルールを実装するなら implements で指す", arr(obj({"id": ID("BR"), "condition": ref("condition"), "reject": pat(TERM), "implements": ref("ref"),
                                                       "example": obj({"before": {"type": "object"}, "args": {"type": "object"}})}, ["id", "condition", "reject"]))),
                            "state_changes": F("状態の変更", "コマンドのあとの状態", "変わったあとの状態を条件で書く。計算は値オブジェクトの操作を1回呼ぶ形にする", arr(obj({"id": ID("CHG"), "condition": ref("condition")}, ["id", "condition"]))),
                            "emits": F("業務イベント", "コマンドのあとに起きたこととして外へ出すもの", "名前は用語集の業務イベントの語で書き、渡す状態を fields に書く", arr(obj({"id": ID("EVT"), "name": pat(TERM), "fields": arr(obj({"from": {"type": "string"}}, ["from"]))}, ["id", "name", "fields"]))),
                            "accept_examples": F("受け付ける例", "業務ルールをすべて満たす入力の例", "前の状態と引数だけを書く。後の状態は道具が導く", arr(obj({"id": ID("OK"), "before": {"type": "object"}, "args": {"type": "object"}}, ["id", "before", "args"])))},
                           ["id", "name", "args", "business_rules", "state_changes", "emits", "accept_examples"]))})

# ── 値オブジェクト
vo = kind_schema("value_object", "値オブジェクト", "値で同一性が決まり、変わらないもの。成分ごとの不変条件と操作を持つ", {
    "header": F(*HEADER_IN_CONTEXT_DESC, HEADER_IN_CONTEXT),
    "components": F("成分", "成分ごとの種類 ・ 精度 ・ 単位 ・ 不変条件", "妥当性の検証は不変条件として値オブジェクト自身に書く。業務の決まりでない技術の制限（ファイルの名前の長さなど）は、ここではなくデータ要求か、書き込む側に書く。文字数は measure: length",
                    arr(obj({"id": ID("CMP"), "name": txt(), "kind": txt(), "precision": txt(), "unit": txt(), "digits": {"type": "integer"}, "values": arr(txt()),
                             "invariants": arr(obj({"id": ID("INV"), "condition": ref("condition")}, ["id", "condition"]))}, ["id", "name", "kind", "invariants"]))),
    "operations": F("操作", "値から値を求める操作", "名前は用語集の語で書き、受け付ける例を1つ以上書く",
                    arr(obj({"id": ID("OP"), "name": pat(TERM), "args": arr({"type": "string"}), "result": {"type": "string"},
                             "accept_examples": arr(obj({"id": ID("OK"), "self": {}, "args": {"type": "array"}, "result": {}}, ["id", "self", "args", "result"]))}, ["id", "name", "args", "result", "accept_examples"])))},
    ["header", "components", "operations"], idpre="VO")

# ── ドメインサービス
ds = kind_schema("domain_service", "ドメインサービス", "複数の集約にまたがる計算。どの集約も変えずに、読んで結果を返す", {
    "header": F("名前 ・ 文脈 ・ 置く理由", "なぜ集約に置かないか", "理由は「複数の集約にまたがる計算」など、どの集約にも置けない理由を書く",
                obj({"name": pat(TERM), "context": ID("BC"), "reason": txt()}, ["name", "context", "reason"])),
    "reads": F("読む集約", "計算に使う集約", "AGG の ID を書く", arr(ID("AGG"), minItems=1)),
    "operations": F("操作", "入力 → 結果の条件で読む", "入力はどの集約のどの状態から取るか（合計なら agg: sum）を書き、結果は条件で書く",
                    arr(obj({"id": ID("OP"), "name": pat(TERM), "output": {"type": "string"},
                             "inputs": arr(obj({"id": ID("IN"), "type": {"type": "string"}, "from": obj({"target": {"type": "string"}, "agg": {"enum": ["sum", "count"]}}, ["target"])}, ["id", "type", "from"])),
                             "results": arr(obj({"id": ID("RES"), "condition": ref("condition")}, ["id", "condition"]))}, ["id", "name", "output", "inputs", "results"])))},
    ["header", "reads", "operations"], idpre="DS")

ALL = {"common": common, "domain": domain, "glossary": glossary, "other_requirements": other, "use_case": use_case,
       "subdomain": subdomain, "context": context, "aggregate": agg, "value_object": vo, "domain_service": ds}

# ── 宣言の外の3つのファイル（記録の契約 ・ 承認した時点の記録 ・ 移行の記録）
record = {"$schema": S, "$id": "trace.schema.json", "x-generates": "記録ファイル（環境変数 CONCRETE_TRACE が指す。1行1件の JSON）",
  "title": "記録の1行", "description": "テストが走ったときに、記録ファイルへ1行ずつ追記する。「このテストは、このテスト条件を、このハッシュ値の版で、このレベルで確かめた」を表す。テストの合否は持たない（合否はテストの実行器が判定し、保存しない）",
  "type": "object", "additionalProperties": False, "required": ["condition", "hash", "level"], "properties": {
    "condition": F("テスト条件の ID", "道具が宣言から取り出したテスト条件のどれか", "道具が出したテスト条件の一覧から、そのテストが確かめる ID をそのまま書く。1つのテストが2つ以上を確かめるなら、1件ずつ行を分ける", pat("^[A-Z]+-[0-9]+(\\.[A-Z]+-?[0-9]*)+$")),
    "hash": F("テスト条件のハッシュ値", "テストを書いたときに読んだ版", "テストを書いたときに、テスト条件の一覧で読んだハッシュ値を書く。宣言が変わると道具は「古い」と出す", pat("^[0-9a-f]{8}$")),
    "level": F("テストレベル", "このテストがどのレベルで確かめたか", "component ・ component-integration ・ system のどれか。テスト条件が求めるレベルより低いと、道具は「レベル違い」と出す", {"enum": ["component", "component-integration", "system"]}),
    "test": F("テストの名前", "どのテストが書いた行か", "テストの実行器が使う名前をそのまま書く（任意）", txt())}}
approved = {"$schema": S, "$id": "approved.schema.json", "x-generates": "spec/approved-record.json",
  "title": "承認した時点の記録", "description": "宣言の従属関係ごとに、承認した時点の参照先のハッシュ値を持つ。道具はいまのハッシュ値と比べ、違えば「確かめ直し」と出す。手で書かない ── 承認したときに道具が書く",
  "type": "object", "propertyNames": {"pattern": "^[A-Z]+-[0-9]+[A-Z0-9.:-]*→[A-Z]+-[0-9]+[A-Z0-9.-]*$"},
  "additionalProperties": pat("^[0-9a-f]{8}$")}
SRC = ["document", "expert", "code", "decided"]
migration = {"$schema": S, "$id": "migration.schema.json", "x-generates": "spec/migration.json",
  "title": "移行の記録", "description": "既存のプロダクトを宣言へ移すとき、ユースケース1件ごとに1つ書く。項目ごとの出どころ ・ 既存のテストの assert ごとの当たり先 ・ 移す範囲を持つ。テストの欠けは書かない ── 道具が当たり先と宣言のテスト条件から数える",
  "type": "object", "additionalProperties": False, "required": ["use_case", "from", "items", "tests", "scope"], "properties": {
    "use_case": F("移すユースケース", "この記録が扱う範囲の単位", "UC の ID を1つ書く。同じデータを操作するユースケースは、同じ回に移す", ID("UC")),
    "from": F("材料の置き場所", "どの文書 ・ コード ・ テストを読んだか", "読んだファイルの経路と、読んだ日を書く",
              obj({"document": txt(), "code": arr(txt()), "tests": txt(), "read_at": pat("^[0-9]{4}-[0-9]{2}-[0-9]{2}$")}, ["read_at"])),
    "sources_rule": F("出どころの分け方", "この記録で使った分け方", "実行される振る舞いは code、書かれた説明（文書 ・ コードの説明文 ・ テストの名前と文言）は document。1件に両方が混ざるなら code", txt()),
    "sources_note": F("出どころの注記", "出どころについての補足", "expert を使わなかった理由など、分け方の外のことだけを書く", txt()),
    "items": F("項目ごとの出どころ", "宣言のどの項目を、どこから取ったか",
               "宣言の項目ごとに1件書く。document は文書の該当箇所と突き合わせ、expert は業務エキスパートが業務の言葉で確かめ、code と decided は業務エキスパートが新しく決めるものとして承認する。元の位置はファイル:行で書く",
               arr(obj({"item": txt(), "source": {"enum": SRC}, "at": {"type": "string"}, "note": txt()}, ["item", "source", "at"]))),
    "tests": F("既存のテストの当たり先", "既存のテストの assert が、宣言のどのテスト条件を確かめているか",
               "テストごとではなく、assert ごとに当たるテスト条件の ID を hits に書く。当たる条件が無い assert は hits を空にし、宣言に足りない条件か、移す範囲の外のテストかを人が決める。全部を回帰テストとして回し続ける",
               arr(obj({"test": txt(), "at": txt(), "asserts": arr(obj({"line": {"type": "integer"}, "hits": arr(pat("^[A-Z]+-[0-9]+(\\.[A-Z]+-?[0-9]*)+$")), "note": txt()}, ["line", "hits"])),
                        "kind": txt(), "decision": txt()}, ["test", "at", "asserts"]))),
    "gaps_note": F("欠けの注記", "欠けをどう数えるか", "欠けは手で書かない。道具が数えることだけを書く", txt()),
    "scope": F("移す範囲と移し終える条件", "どこまで移したか、何をもって移し終えるか", "範囲の単位 ・ まだ移していない部分 ・ 移し終える条件（その範囲で、宣言どうしのずれが0件、テストの欠けが0件）を書く",
               obj({"unit": txt(), "not_yet": arr(txt()), "done_when": txt()}, ["unit", "done_when"]))}}
ALL.update({"trace": record, "approved": approved, "migration": migration})

# ── concrete の注釈（ボード schema-driven-base の論点2）。道具は欄の名前を持たず、この注釈だけを読む
#   x-ref：この欄が指す先。to＝種類（self は同じ宣言）・ item＝「宣言.項目」の形 ・ bare＝宣言の ID を付けない項目の ID ・ in＝指す先の集まりの場所
#          meaning＝用語集の語の意味の種類 ・ when＝同じ項目の欄がこの値のときだけ ・ inverse＝指される側の数（group ・ min ・ max ・ where ・ max_decls）
#          unique＝同じ並びの中で重ねない ・ covered_by＝同じ項目の中で、この値を扱う欄
#   x-test-spec：この項目がテスト条件であること。id＝decl（宣言.項目）か parent（宣言.親.項目）・ checks ・ level ・ with（期待する結果に含める同じ宣言の欄）
import copy
LV = {"derive": "business_logic", "map": {"イベント履歴式ドメインモデル": "component", "ドメインモデル": "component", "アクティブレコード": "component-integration", "トランザクションスクリプト": "system"}}
def at(sc, path):
    o = sc
    for k in path.split("/"):
        o = o[k] if k else o
        if isinstance(o, dict) and "$ref" in o and o["$ref"].startswith("#/$defs/") and k != path.split("/")[-1]:
            o = sc["$defs"][o["$ref"].split("/")[-1]]
    return o
def put(sc, path, key, val):
    o = at(sc, path)
    if "$ref" in o and len(o) <= 3:  # 共通の形を指す欄は、その場に写して注釈を付ける
        o.setdefault("allOf", [{"$ref": o.pop("$ref")}])
    o[key] = val
G = {"to": "glossary", "in": "terms", "meanings": "meanings", "meaning_key": "kind", "label": "word"}
T = lambda *k: dict(G, meaning=list(k)) if k else dict(G)
for k in ("aggregate", "value_object"):
    ALL[k]["properties"]["header"] = copy.deepcopy(ALL[k]["properties"]["header"])
A = [
 ("other_requirements", "properties/business_rules/items/properties/protects", "x-ref", {"to": "domain", "item": True}),
 ("other_requirements", "properties/quality/items/properties/target", "x-ref", {"to": "use_case", "item": True}),
 ("other_requirements", "properties/quality/items", "x-test-spec", {"id": "decl", "checks": "閾値と割合を満たす", "level": "system"}),
 ("other_requirements", "properties/data/items", "x-test-spec", {"id": "decl", "checks": "条件を満たさない値を受け付けない", "level": "system"}),
 ("use_case", "properties/header/properties/scope/properties/system", "x-ref", {"to": "domain", "item": True}),
 ("use_case", "properties/stakeholders/items/properties/who", "x-ref", {"to": "domain", "item": True}),
 ("use_case", "properties/preconditions/items/properties/established_by", "x-ref", {"to": "use_case"}),
 ("use_case", "properties/guarantees/properties/minimal/items/properties/protects", "x-ref", {"to": "self", "in": "stakeholders"}),
 ("use_case", "properties/guarantees/properties/success/items/properties/satisfies", "x-ref", {"to": "self", "in": "stakeholders"}),
 ("use_case", "properties/contributes_to", "x-ref", {"to": "domain", "bare": True}),
 ("use_case", "properties/links/properties/business_rules", "x-ref", {"to": "other_requirements", "item": True}),
 ("use_case", "properties/links/properties/quality", "x-ref", {"to": "other_requirements", "item": True}),
 ("use_case", "properties/links/properties/technology", "x-ref", {"to": "other_requirements", "item": True}),
 ("use_case", "properties/links/properties/data", "x-ref", {"to": "other_requirements", "item": True}),
 ("use_case", "properties/scenario", "x-test-spec", {"id": "M", "checks": "成功時保証がすべて成り立つ", "level": "system", "with": ["guarantees/success"]}),
 ("use_case", "$defs/step/properties/data", "x-ref", dict(T("情報の別名", "値オブジェクト", "識別子"), when={"kind": ["相互作用"]})),
 ("use_case", "$defs/step/properties/verb", "x-ref", dict(T("動作"), when={"kind": ["相互作用"]})),
 ("use_case", "$defs/step/properties/object", "x-ref", T()),
 ("use_case", "$defs/step/properties/reply", "x-ref", T()),
 ("use_case", "$defs/step/properties/checks", "x-ref", {"to": "other_requirements", "item": True, "covered_by": "extensions/fails"}),
 ("use_case", "$defs/step/properties/calls", "x-ref", {"to": "use_case"}),
 ("use_case", "$defs/step/properties/keeps", "x-ref", {"to": "self", "in": "guarantees/minimal", "inverse": {"group": "最低保証を守る手順", "min": 1}}),
 ("use_case", "$defs/step/properties/serves", "x-ref", {"to": "self", "in": "stakeholders"}),
 ("use_case", "$defs/extension/properties/fails", "x-ref", {"to": "other_requirements", "item": True}),
 ("use_case", "$defs/extension/properties/reasons", "x-ref", T("拒否の理由", "失敗の種類")),
 ("use_case", "$defs/extension", "x-test-spec", {"id": "decl", "level": "system", "checks_by": {"field": "ending", "map": {"失敗": "最低保証がすべて成り立ち、成功時保証は成り立たない", "成功": "元の手順が成功した状態で続き、成功時保証が成り立つ", "終了": "別の道筋で成功して終わる", "*": "元の手順に戻り、成功時保証が成り立つ"}}, "with_by": {"field": "ending", "map": {"失敗": ["guarantees/minimal"]}}}),
 ("subdomain", "properties/use_cases", "x-ref", {"to": "use_case", "inverse": {"group": "ユースケースを束ねるサブドメインは1つ", "min": 1, "max": 1, "where": {"header/level": ["ユーザー目的", "サブ機能"]}}}),
 ("subdomain", "properties/serves_values", "x-ref", {"to": "domain", "bare": True}),
 ("context", "properties/header/properties/subdomains", "x-ref", {"to": "subdomain", "inverse": {"group": "サブドメインを担う文脈がある", "min": 1}}),
 ("context", "properties/context_map/properties/relations/items/properties/fulfills", "x-ref", {"to": "subdomain", "inverse": {"group": "サブドメインを担う文脈がある", "min": 1}}),
 ("context", "properties/business_rules/items/properties/implements", "x-ref", {"to": "other_requirements", "item": True, "in": "business_rules", "inverse": {"group": "ビジネスルールを実装する文脈は1つ", "min": 1, "max": 1}}),
 ("context", "properties/uses/items/properties/term", "x-ref", dict(T(), unique=True)),
 ("aggregate", "properties/header/properties/name", "x-ref", T("集約")),
 ("aggregate", "properties/header/properties/context", "x-ref", {"to": "context"}),
 ("aggregate", "properties/structure/properties/state/items/properties/name", "x-ref", T()),
 ("aggregate", "properties/structure/properties/state/items/properties/type", "x-ref", {"to": "value_object", "only": "^VO-"}),
 ("aggregate", "properties/structure/properties/entities/items/properties/name", "x-ref", T("エンティティ")),
 ("aggregate", "properties/invariants/items/properties/via", "x-ref", {"to": "self", "in": "commands"}),
 ("aggregate", "properties/invariants/items", "x-test-spec", {"id": "decl", "checks": "違反する操作が拒否される", "level": LV}),
 ("aggregate", "$defs/command/properties/name", "x-ref", T("コマンド")),
 ("aggregate", "$defs/command/properties/business_rules", "x-test-spec-items", {"id": "parent", "checks": "その拒否の理由で拒否される", "level": LV}),
 ("aggregate", "$defs/command/properties/accept_examples", "x-test-spec-items", {"id": "parent", "checks": "状態の変更と業務イベントが成り立つ", "level": LV, "with_parent": ["state_changes", "emits"]}),
 ("value_object", "properties/header/properties/name", "x-ref", T("値オブジェクト")),
 ("value_object", "properties/header/properties/context", "x-ref", {"to": "context"}),
 ("value_object", "properties/components/items/properties/invariants", "x-test-spec-items", {"id": "decl", "checks": "作れない値を拒む", "level": "component"}),
 ("value_object", "properties/operations/items/properties/accept_examples", "x-test-spec-items", {"id": "parent", "checks": "操作の結果が例と同じ", "level": "component"}),
 ("domain_service", "properties/header/properties/name", "x-ref", T("ドメインサービス")),
 ("domain_service", "properties/header/properties/context", "x-ref", {"to": "context"}),
 ("domain_service", "properties/reads", "x-ref", {"to": "aggregate"}),
 ("domain_service", "properties/operations/items/properties/results", "x-test-spec-items", {"id": "parent", "checks": "計算の結果が宣言どおりである", "level": LV}),
]
for k, path, key, val in A:
    if key == "x-test-spec-items":
        o = at(ALL[k], path); o["items"]["x-test-spec"] = val
    else:
        put(ALL[k], path, key, val)
# 業務ルールの拒否の理由と実装するビジネスルールは、コマンドの業務ルールの項目の中にある
br = ALL["aggregate"]["$defs"]["command"]["properties"]["business_rules"]["items"]["properties"]
br["reject"]["x-ref"] = T("拒否の理由"); br["reject"]["x-ref"]["inverse"] = {"group": "拒否の理由を1つの集約だけが使う", "max_decls": 1}
br["implements"].setdefault("allOf", [{"$ref": br["implements"].pop("$ref")}]) if "$ref" in br["implements"] else None
br["implements"]["x-ref"] = {"to": "other_requirements", "item": True, "in": "business_rules", "inverse": {"group": "ビジネスルールを実装する文脈は1つ", "min": 1, "max": 1}}
ALL["aggregate"]["$defs"]["command"]["properties"]["emits"]["items"]["properties"]["name"]["x-ref"] = T("業務イベント")
ALL["aggregate"]["properties"]["invariants"]["items"]["required"] = ["id", "condition", "via"]
ALL["aggregate"]["properties"]["structure"]["properties"]["state"]["items"]["x-test-spec"] = {"id": "decl", "suffix": "MAX", "when": {"multiplicity/max": {"not": [None, 1]}}, "checks": "上限を超える操作が拒否される", "level": LV}
ALL["subdomain"]["properties"]["classification"]["x-derive"]["declared"] = "category"
DM = ["ドメインモデル", "イベント履歴式ドメインモデル"]
# 判定に付ける条件：同じ宣言のほかの欄の値と、判定の組み合わせ（合わなければ人のレビュー）
ALL["subdomain"]["properties"]["business_logic"]["x-derive"]["expect"] = [
    {"name": "カテゴリーと実装方法", "when": {"classification/category": ["中核"]}, "in": DM},
    {"name": "カテゴリーと実装方法", "when": {"classification/category": ["一般", "補完"]}, "not_in": DM}]
# 指される数の条件を、指される側の判定で絞る：実装方法がドメインモデルのサブドメインを対象とする文脈には、集約が必要
ALL["aggregate"]["properties"]["header"]["properties"]["context"]["x-ref"]["inverse"] = {"group": "ドメインモデルの文脈に集約がある", "min": 1, "where_derive": {"via": "header/subdomains", "derive": "business_logic", "in": DM}}
for k, s in ALL.items():
    json.dump(s, open(os.path.join(H, f"{k}.schema.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(len(ALL), "files")
