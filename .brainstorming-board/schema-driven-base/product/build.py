# schema-driven を、ユースケース駆動の宣言でプロダクトとして書く（ボード schema-driven-base の次にすること1）。
# 材料：ボード schema-driven-base の論点1〜4の決定（出どころの種類は「決まったこと」）
# 検査：ボード usecase-driven-concrete の見本の道具（schema/check.py ・ concrete7）を外から当てる（run.py）
import json, os
H = os.path.dirname(os.path.abspath(__file__))

def T(i, w, kind, d, avoid=(), **kw):
    m = {"id": "M-1", "definition": d, "kind": kind}; m.update(kw)
    return {"id": f"TERM-{i}", "word": w, "meanings": [m], "avoid": list(avoid)}

TERMS = [
 T(1, "実体", "集約", "スキーマに従う JSON のデータ1件。正本であり、1つのファイルに置く", avoid=["インスタンス", "ドキュメント"]),
 T(2, "スキーマ", "値オブジェクト", "実体の形と注釈を書いた JSON Schema"),
 T(3, "注釈", "情報の別名", "スキーマの欄に付ける、x- で始まるキー（x-ref ・ x-derive ・ x-view ・ x-prompt ・ x-generates）"),
 T(4, "欄", "値オブジェクト", "実体の中の1つの値の場所。JSON Pointer で指す", avoid=["フィールド"]),
 T(5, "取得の式", "値オブジェクト", "実体から値を取り出す JMESPath の式", avoid=["クエリ"]),
 T(6, "更新の差分", "値オブジェクト", "実体へ適用する JSON Patch（RFC 6902）の操作の並び", avoid=["パッチ"]),
 T(7, "置き場所", "値オブジェクト", "実体を置くファイルの場所。スキーマの x-generates が決める"),
 T(8, "違反", "値オブジェクト", "実体がスキーマを満たさない欄と、その理由"),
 T(9, "検証の結果", "情報の別名", "実体ごとの違反の一覧"),
 T(10, "案内", "情報の別名", "欄ごとの x-prompt。読むときの read と、値を書くときの write"),
 T(11, "実体の集合", "情報の別名", "1つの置き場所の下にある実体すべて。参照と判定は、この集合に対して確かめる"),
 T(12, "参照", "値オブジェクト", "x-ref を付けた欄の値。ほかの実体か、その中の項目を指す"),
 T(13, "判定", "値オブジェクト", "x-derive の決まりに従って、同じ実体の答えの欄から導いた値"),
 T(14, "検査の結果", "情報の別名", "参照と判定の検査ごとの、合格 ・ ずれ ・ 確かめ直しの一覧"),
 T(15, "承認の記録", "集約", "承認した時点の、実体ごとの中身のハッシュ値"),
 T(16, "中身のハッシュ値", "値オブジェクト", "実体のファイルの sha256"),
 T(17, "頁", "情報の別名", "実体の集合から描画した HTML または Markdown"),
 T(18, "頁の型", "値オブジェクト", "種類ごとの頁の、節の並びと、節ごとの部品と差し込む欄を書いたデータ"),
 T(19, "部品", "値オブジェクト", "頁を組む HTML の雛形。差し込む場所を持つ"),
 T(20, "トークン", "値オブジェクト", "見た目の値（色 ・ 寸法 ・ 余白 ・ 文字）に付けた名前と、その値"),
 T(21, "正本", "集約", "schema-driven が持つ基盤の能力。Rust の crate ・ 部品の雛形 ・ トークン"),
 T(22, "複製", "集約", "基盤を使う道具の中へ転写した、正本の写し。基盤を使う道具は、これだけで動く"),
 T(23, "差", "値オブジェクト", "複製のうち、正本と中身が違うファイル"),
 T(24, "基盤を使う道具", "情報の別名", "schema-driven の正本の複製を持ち、それで動く Skill や道具"),
 T(25, "依頼する", "動作", "相手に何かを頼む", form="{to}に{data}を依頼する"),
 T(26, "知らせる", "動作", "相手に結果を伝える", form="{to}に{data}を知らせる"),
 T(27, "作成する", "コマンド", "スキーマから、未記入の欄を持つ実体を作る"),
 T(28, "更新する", "コマンド", "実体へ更新の差分を適用する。適用したあとの実体が検証を通過したときだけ、書き込む"),
 T(29, "削除する", "コマンド", "実体のファイルを消す。実体の中の項目は、remove の差分で更新して消す"),
 T(30, "承認を記録する", "コマンド", "実体の集合の、いまの中身のハッシュ値を承認の記録へ書く"),
 T(31, "転写する", "コマンド", "正本を、基盤を使う道具の中へ写す"),
 T(32, "検査する", "ドメインサービス", "実体の集合について、指す先がある ・ 指される数 ・ 判定と宣言した値 ・ 承認のあとの変化を確かめる"),
 T(33, "描画する", "ドメインサービス", "頁の型と部品とトークンで、実体の集合から頁を組む。頁の型が無い種類は、注釈だけから組む"),
 T(34, "比べる", "ドメインサービス", "複製と正本を、ファイルごとの中身のハッシュ値で比べる"),
 T(35, "読む", "操作", "ファイルの中身を受け取る"),
 T(36, "書く", "操作", "ファイルへ中身を置く"),
 T(37, "置き場所に実体が既にある", "拒否の理由", "作成しようとした置き場所に、実体が既にあるので作らない"),
 T(38, "差分を適用できない", "拒否の理由", "差分が指す欄が実体に無いなど、差分を適用できないので書き込まない"),
 T(39, "検証を通過しない", "拒否の理由", "差分を適用したあとの実体が検証を通過しないので、書き込まない"),
 T(40, "読めない", "失敗の種類", "ファイルが無いか、読む権限が無い"),
 T(41, "書けない", "失敗の種類", "置き場所へ書く権限が無いか、空きが無い"),
 T(42, "誤りの理由", "情報の別名", "依頼を受け付けなかった理由"),
 T(43, "取得した値", "情報の別名", "取得の式を実体に当てて得た JSON の値"),
 T(44, "残っている参照", "情報の別名", "削除した実体や項目を、まだ指している参照"),
 T(45, "頁の置き場所", "値オブジェクト", "描画した頁を書いたファイルの場所"),
 T(46, "読み出す", "動作", "相手から中身を受け取る", form="{to}から{data}を読み出す"),
 T(47, "書き出す", "動作", "相手へ中身を渡して置かせる", form="{to}へ{data}を書き出す"),
 T(48, "実体の中身", "情報の別名", "実体を JSON の文字列にしたもの"),
 T(49, "頁の中身", "情報の別名", "描画した頁の HTML または Markdown の文字列"),
 T(50, "複製の中身", "情報の別名", "複製のファイルと、その中身"),
 T(51, "JSON として読めない", "失敗の種類", "ファイルの中身が JSON の文法に沿わない"),
 T(52, "読んだあとに別の更新が書いた", "拒否の理由", "差分の test の操作が、いまの実体と合わないので書き込まない"),
 T(53, "検証を通過しない実体がある", "拒否の理由", "実体の集合に検証を通過しない実体があるので、承認を記録しない"),
]

D = {}
D["GLO-1"] = {"kind": "glossary", "id": "GLO-1", "header": {"name": "用語集"}, "terms": TERMS}

D["DOM-1"] = {"kind": "domain", "id": "DOM-1", "header": {"name": "schema-driven"},
 "vision": {
  "problem": "AI が扱うデータを Markdown の文書で持つと、欄の場所が決まらず、AI は文字列を探して読むことになる。指す先が消えたことや、答えと宣言した値の食い違いに、書いた時点で気づけない。同じ読み書きの能力を道具ごとに書くので、道具どうしでずれる",
  "values": [
   {"id": "VAL-1", "name": "欄を式で指して読み書きできる", "text": "正本を構造化データで持ち、取得の式で値を取り出し、更新の差分で書き換える", "differentiator": False},
   {"id": "VAL-2", "name": "参照と判定のずれを道具が出す", "text": "実体どうしの参照と、答えから導いた判定を、スキーマの注釈だけを読んで確かめる", "differentiator": True},
   {"id": "VAL-3", "name": "人が読む頁を正本から描画する", "text": "頁の型と部品で、実体の集合から人が読む頁を組む", "differentiator": False},
   {"id": "VAL-4", "name": "基盤の能力を複製して配れる", "text": "基盤を使う道具へ正本の複製を転写し、正本との差を道具が出す", "differentiator": False}],
  "competitors": [
   {"id": "CMP-1", "name": "Markdown の文書を正本にするやり方", "lacks": ["VAL-1", "VAL-2"]},
   {"id": "CMP-2", "name": "JSON Schema の検証器だけを使うやり方", "lacks": ["VAL-2", "VAL-3"]}],
  "success_criteria": [
   {"id": "SC-1", "name": "文字列を探さずに読める", "source": "external", "measure_text": "AI エージェントが正本を読むときに、文字列の検索を使った回数", "threshold": {"op": "le", "value": 0, "unit": "回"}, "window": "作業1件"},
   {"id": "SC-2", "name": "検証を通過しない更新が書き込まれない", "source": "external", "measure_text": "更新で書き込んだ実体のうち、検証を通過しないものの件数", "threshold": {"op": "le", "value": 0, "unit": "件"}, "window": "更新のたび"},
   {"id": "SC-3", "name": "ずれを承認の前に見つける", "source": "external", "measure_text": "承認したあとに見つかった、参照と判定のずれの件数", "threshold": {"op": "le", "value": 0, "unit": "件"}, "window": "承認1回"},
   {"id": "SC-4", "name": "複製が正本と同じ", "source": "external", "measure_text": "転写し直したあとの、複製と正本の差の件数", "threshold": {"op": "le", "value": 0, "unit": "件"}, "window": "転写1回"}]},
 "scope": {"out": ["宣言の種類（ドメイン ・ ユースケースなど）の形", "テスト条件とテストの記録の突き合わせ", "図の描画"]},
 "design_scopes": [
  {"id": "SCP-1", "level": "システム", "name": "schema-driven", "outside": ["SH-1", "SH-2", "SH-3", "ファイルシステム"]}],
 "stakeholders": [
  {"id": "SH-1", "who": "AI エージェント", "interest": "欄を式で指して読み書きし、書いたものがその場で確かめられる"},
  {"id": "SH-2", "who": "データの持ち主", "interest": "正本が形と参照と判定を満たし、承認したあとの変化に気づける。正本を頁で読める"},
  {"id": "SH-3", "who": "道具の作り手", "interest": "基盤の能力を書き直さずに使え、正本が新しくなったことに気づける"}]}

D["REQ-1"] = {"kind": "other_requirements", "id": "REQ-1", "header": {"name": "その他の要求"},
 "business_rules": [
  {"id": "BR-1", "name": "取得の式を渡す", "condition": {"target": "TERM-5", "op": "not_empty"}, "protects": ["DOM-1.SH-1"]},
  {"id": "BR-2", "name": "更新の差分を渡す", "condition": {"target": "TERM-6", "op": "not_empty"}, "protects": ["DOM-1.SH-1"]},
  {"id": "BR-3", "name": "案内を求める欄がスキーマに在り、案内を持つ", "condition": {"target": "TERM-2.TERM-4.TERM-10", "op": "not_empty"}, "protects": ["DOM-1.SH-1"]}],
 "quality": [],
 "technology": [
  {"id": "TEC-1", "system": "ファイルシステム", "text": "実体 ・ スキーマ ・ 頁 ・ 承認の記録 ・ 複製は、利用者の環境のファイルとして読み書きする"},
  {"id": "TEC-2", "system": "AI エージェントの実行環境", "text": "CLI と MCP のどちらからも、同じ道具の一覧を呼べる"}],
 "data": [],
 "open_issues": ["取得の式が JMESPath として読めることを、条件の欄で書けない", "同じ入力から同じ頁が出ることを、品質の要求の形で書けない"]}

def step(i, kind, actor, **kw):
    s = {"id": f"STEP-{i}", "kind": kind, "actor": actor}; s.update(kw); s.setdefault("extensions", []); return s
def say(i, actor, to, data, verb="TERM-25", **kw): return step(i, "相互作用", actor, to=to, data=data, verb=verb, **kw)
def sub(sid, kind, actor, **kw):
    s = {"id": sid, "kind": kind, "actor": actor}; s.update(kw); s.setdefault("extensions", []); return s
def fail_fs(eid, reason, who, data="TERM-42", keeps=None):
    st = [sub(f"{eid}.S-1", "相互作用", "システム", to=who, data=[data], verb="TERM-26")]
    if keeps: st[0]["keeps"] = keeps
    return {"id": eid, "condition_kind": "支援アクターの失敗", "actor": "ファイルシステム", "reasons": reason if isinstance(reason, list) else [reason], "ending": "失敗", "steps": st}
def reject(eid, reasons, who, data="TERM-42"):
    return {"id": eid, "condition_kind": "業務ルールの拒否", "reasons": reasons, "ending": "失敗",
            "steps": [sub(f"{eid}.S-1", "相互作用", "システム", to=who, data=[data], verb="TERM-26")]}
def invalid(eid, br, who):
    return {"id": eid, "condition_kind": "妥当性確認の失敗", "fails": [br], "ending": "失敗",
            "steps": [sub(f"{eid}.S-1", "相互作用", "システム", to=who, data=["TERM-42"], verb="TERM-26")]}
def UC(i, name, level, actor, sh, minimal, success, steps, sc, links=None, supporting=("ファイルシステム",), pre=(), issues=()):
    return {"kind": "use_case", "id": f"UC-{i}",
            "header": {"name": name, "level": level, "scope": {"system": "DOM-1.SCP-1"}, "primary_actor": actor, "trigger_step": "STEP-1"},
            "preconditions": list(pre), "stakeholders": sh, "guarantees": {"minimal": minimal, "success": success},
            "scenario": {"supporting_actors": list(supporting), "steps": steps},
            "contributes_to": sc, "open_issues": list(issues), "links": links or {"technology": ["REQ-1.TEC-1", "REQ-1.TEC-2"]}}
AI, OWN, MK = "AI エージェント", "データの持ち主", "道具の作り手"
SH_AI = lambda t: {"id": "SH-1", "who": "DOM-1.SH-1", "interest": t}
SH_OWN = lambda t, i="SH-2": {"id": i, "who": "DOM-1.SH-2", "interest": t}
SH_MK = lambda t: {"id": "SH-1", "who": "DOM-1.SH-3", "interest": t}
SAME = lambda i, name, target, sh: {"id": i, "name": name, "condition": {"target": target, "op": "eq", "value": {"before": target}}, "protects": sh}
VALID = {"target": "TERM-1.TERM-8", "agg": "count", "op": "le", "value": 0}
READ = lambda i, who, data, reply, eid, reasons=("TERM-40", "TERM-51"), **kw: say(i, "システム", "ファイルシステム", data, verb="TERM-46", reply=reply, extensions=[fail_fs(eid, list(reasons), who)] + kw.pop("ext", []), **kw)
WRITE = lambda i, who, data, eid, **kw: say(i, "システム", "ファイルシステム", data, verb="TERM-47", extensions=[fail_fs(eid, ["TERM-41"], who)], **kw)
def other(eid, cond, who, data, ending="終了"):
    return {"id": eid, "condition_kind": "別の道筋での成功", "condition": cond, "ending": ending,
            "steps": [sub(f"{eid}.S-1", "相互作用", "システム", to=who, data=data, verb="TERM-26")]}
TECH = {"technology": ["REQ-1.TEC-1", "REQ-1.TEC-2"]}

D["UC-0"] = UC(0, "構造化データを正本として持つ", "要約", OWN,
 [SH_OWN("承認した正本が、形と参照と判定を満たす", "SH-1"), dict(SH_AI("欄を式で指して読み書きし、書いたものをその場で確かめる"), id="SH-2")],
 [{"id": "MG-1", "name": "承認した正本は検証を通過している", "condition": {"if": {"target": "TERM-15.TERM-16", "op": "not_empty"}, "target": "TERM-11.TERM-8", "agg": "count", "op": "le", "value": 0}, "protects": ["SH-1"]}],
 [{"id": "SG-1", "name": "承認した正本", "condition": {"target": "TERM-15.TERM-16", "op": "not_empty"}, "satisfies": ["SH-1", "SH-2"]}],
 [step(1, "サブユースケースの呼び出し", AI, calls="UC-1", serves=["SH-2"]),
  step(2, "サブユースケースの呼び出し", AI, calls="UC-7", serves=["SH-2"]),
  step(3, "サブユースケースの呼び出し", AI, calls="UC-3", serves=["SH-1", "SH-2"]),
  step(4, "サブユースケースの呼び出し", OWN, calls="UC-6", serves=["SH-1"]),
  step(5, "サブユースケースの呼び出し", OWN, calls="UC-8", keeps=["MG-1"], serves=["SH-1"])],
 ["SC-1", "SC-2", "SC-3"], links={}, supporting=(),
 issues=["AI エージェントが、データの持ち主の作業を代わりに進める形を前提にしている"])

D["UC-1"] = UC(1, "実体を作成する", "ユーザー目的", AI,
 [SH_AI("未記入の欄と、そこへ何を書くかが分かる"), SH_OWN("置き場所にある実体を上書きしない")],
 [SAME("MG-1", "既にある実体を書き換えない", "TERM-1.TERM-16", ["SH-2"])],
 [{"id": "SG-1", "name": "置き場所の実体", "condition": {"target": "TERM-1.TERM-7", "op": "not_empty"}, "satisfies": ["SH-1"]},
  {"id": "SG-2", "name": "未記入の欄の案内", "condition": {"target": "TERM-10", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, AI, "システム", ["TERM-2", "TERM-7"], serves=["SH-1"]),
  READ(2, AI, ["TERM-2"], ["TERM-48"], "EXT-1", serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-27", object="TERM-1", keeps=["MG-1"], serves=["SH-1", "SH-2"],
       extensions=[reject("EXT-2", ["TERM-37"], AI)]),
  WRITE(4, AI, ["TERM-48"], "EXT-3", serves=["SH-1"]),
  say(5, "システム", AI, ["TERM-7", "TERM-10"], verb="TERM-26", serves=["SH-1"])],
 ["SC-1"], links=TECH,
 issues=["作成した直後の実体は未記入の欄を持つので、検証を通過しなくてよい（ボード schema-driven-base 論点1）"])

D["UC-2"] = UC(2, "欄の値を取得する", "サブ機能", AI,
 [SH_AI("文字列を探さずに、欄の値だけを受け取る")],
 [SAME("MG-1", "実体を書き換えない", "TERM-1.TERM-16", ["SH-1"])],
 [{"id": "SG-1", "name": "取得した値", "condition": {"target": "TERM-43", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, AI, "システム", ["TERM-7", "TERM-5"], serves=["SH-1"]),
  step(2, "妥当性確認", "システム", checks=["REQ-1.BR-1"], serves=["SH-1"], extensions=[invalid("EXT-1", "REQ-1.BR-1", AI)]),
  READ(3, AI, ["TERM-7"], ["TERM-48"], "EXT-2", keeps=["MG-1"], serves=["SH-1"]),
  say(4, "システム", AI, ["TERM-43"], verb="TERM-26", serves=["SH-1"],
      extensions=[other("EXT-3", {"target": "TERM-43", "op": "empty"}, AI, ["TERM-43"])])],
 ["SC-1"], links={"business_rules": ["REQ-1.BR-1"], **TECH})

D["UC-3"] = UC(3, "実体を更新する", "ユーザー目的", AI,
 [SH_AI("更新した実体がその場で確かめられ、通らなければ理由が分かる"), SH_OWN("置き場所に、検証を通過しない差分が書き込まれない")],
 [SAME("MG-1", "失敗したら前の実体のまま", "TERM-1.TERM-16", ["SH-2"])],
 [{"id": "SG-1", "name": "検証を通過した実体の書き込み", "condition": VALID, "satisfies": ["SH-1", "SH-2"]}],
 [say(1, AI, "システム", ["TERM-7", "TERM-6"], serves=["SH-1"]),
  step(2, "妥当性確認", "システム", checks=["REQ-1.BR-2"], serves=["SH-1"], extensions=[invalid("EXT-1", "REQ-1.BR-2", AI)]),
  READ(3, AI, ["TERM-7", "TERM-2"], ["TERM-48"], "EXT-2", serves=["SH-1"]),
  step(4, "内部の状態変化", "システム", verb="TERM-28", object="TERM-1", keeps=["MG-1"], serves=["SH-1", "SH-2"],
       extensions=[reject("EXT-3", ["TERM-38", "TERM-52"], AI), reject("EXT-4", ["TERM-39"], AI, data="TERM-9"),
                   other("EXT-5", {"target": "TERM-1.TERM-16", "op": "eq", "value": {"before": "TERM-1.TERM-16"}}, AI, ["TERM-9"])]),
  WRITE(5, AI, ["TERM-48"], "EXT-6", serves=["SH-1"]),
  say(6, "システム", AI, ["TERM-9"], verb="TERM-26", serves=["SH-1"])],
 ["SC-2"], links={"business_rules": ["REQ-1.BR-2"], **TECH},
 issues=["項目の削除は、remove の差分を渡すこのユースケースで扱う", "書く途中で失敗したときに、前の中身か新しい中身のどちらかが残ることを、最低保証に置くかを決めていない"])

D["UC-4"] = UC(4, "実体を削除する", "ユーザー目的", AI,
 [SH_AI("消した実体を、まだ指している参照が分かる")],
 [],
 [{"id": "SG-1", "name": "実体のファイルが無い", "condition": {"target": "TERM-1", "op": "empty"}, "satisfies": ["SH-1"]}],
 [say(1, AI, "システム", ["TERM-7"], serves=["SH-1"]),
  READ(2, AI, ["TERM-7"], ["TERM-48"], "EXT-1", serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-29", object="TERM-1", serves=["SH-1"]),
  WRITE(4, AI, ["TERM-7"], "EXT-2", serves=["SH-1"]),
  READ(5, AI, ["TERM-7"], ["TERM-48"], "EXT-3", serves=["SH-1"]),
  step(6, "内部の状態変化", "システム", verb="TERM-32", object="TERM-11", serves=["SH-1"]),
  say(7, "システム", AI, ["TERM-44"], verb="TERM-26", serves=["SH-1"])],
 ["SC-3"], links=TECH, issues=["実体の中の項目を消すときは、remove の差分で実体を更新する（実体を更新する）"])

D["UC-5"] = UC(5, "実体の集合を検査する", "ユーザー目的", OWN,
 [SH_OWN("指す先の無い参照 ・ 指される数の過不足 ・ 判定と宣言した値の食い違い ・ 承認のあとの変化が、承認の前に分かる", "SH-1")],
 [SAME("MG-1", "実体を書き換えない", "TERM-11.TERM-16", ["SH-1"])],
 [{"id": "SG-1", "name": "検査の結果", "condition": {"target": "TERM-14", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, OWN, "システム", ["TERM-7"], serves=["SH-1"]),
  READ(2, OWN, ["TERM-7"], ["TERM-48", "TERM-2", "TERM-15"], "EXT-1", keeps=["MG-1"], serves=["SH-1"],
       ext=[other("EXT-2", {"target": "TERM-11", "op": "empty"}, OWN, ["TERM-14"])]),
  step(3, "内部の状態変化", "システム", verb="TERM-32", object="TERM-11", serves=["SH-1"],
       variations=[{"varies": "承認のあとの変化", "values": ["承認の記録があれば、記録したハッシュ値と比べる", "承認の記録が無ければ、変化の検査を外す"]}]),
  say(4, "システム", OWN, ["TERM-9", "TERM-14"], verb="TERM-26", serves=["SH-1"])],
 ["SC-3"], links=TECH)

D["UC-6"] = UC(6, "頁を描画する", "ユーザー目的", OWN,
 [SH_OWN("正本を、種類ごとに設計した頁で読める", "SH-1")],
 [SAME("MG-1", "実体を書き換えない", "TERM-11.TERM-16", ["SH-1"])],
 [{"id": "SG-1", "name": "描画した頁", "condition": {"target": "TERM-45", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, OWN, "システム", ["TERM-7"], serves=["SH-1"]),
  READ(2, OWN, ["TERM-7"], ["TERM-48", "TERM-18", "TERM-19", "TERM-20"], "EXT-1", keeps=["MG-1"], serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-33", object="TERM-17", serves=["SH-1"],
       variations=[{"varies": "組み方", "values": ["頁の型がある種類は、頁の型と部品で組む", "頁の型が無い種類は、注釈だけから組む"]}]),
  WRITE(4, OWN, ["TERM-49"], "EXT-2", serves=["SH-1"]),
  say(5, "システム", OWN, ["TERM-45"], verb="TERM-26", serves=["SH-1"])],
 [], links=TECH, issues=["頁の型が指す欄が実体に無いときは「なし」と描く。拡張にするかを決めていない"])

D["UC-7"] = UC(7, "欄の案内を受け取る", "サブ機能", AI,
 [SH_AI("欄ごとに、何を読み何を書くかが分かる")],
 [],
 [{"id": "SG-1", "name": "案内", "condition": {"target": "TERM-10", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, AI, "システム", ["TERM-2", "TERM-4"], serves=["SH-1"]),
  READ(2, AI, ["TERM-2"], ["TERM-48"], "EXT-1", serves=["SH-1"]),
  step(3, "妥当性確認", "システム", checks=["REQ-1.BR-3"], serves=["SH-1"], extensions=[invalid("EXT-2", "REQ-1.BR-3", AI)]),
  say(4, "システム", AI, ["TERM-10"], verb="TERM-26", serves=["SH-1"])],
 ["SC-1"], links={"business_rules": ["REQ-1.BR-3"], **TECH})

D["UC-8"] = UC(8, "承認を記録する", "ユーザー目的", OWN,
 [SH_OWN("承認した時点の正本が残り、そのあとの変化を検査で見つけられる", "SH-1")],
 [SAME("MG-1", "拒んだら前の承認の記録のまま", "TERM-15.TERM-16", ["SH-1"])],
 [{"id": "SG-1", "name": "承認の記録", "condition": {"target": "TERM-15.TERM-16", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, OWN, "システム", ["TERM-7"], serves=["SH-1"]),
  step(2, "サブユースケースの呼び出し", OWN, calls="UC-5", serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-30", object="TERM-15", keeps=["MG-1"], serves=["SH-1"],
       extensions=[reject("EXT-1", ["TERM-53"], OWN, data="TERM-9"),
                   other("EXT-2", {"target": "TERM-11.TERM-16", "op": "eq", "value": {"before": "TERM-15.TERM-16"}}, OWN, ["TERM-16"])]),
  WRITE(4, OWN, ["TERM-48"], "EXT-3", serves=["SH-1"]),
  say(5, "システム", OWN, ["TERM-16"], verb="TERM-26", serves=["SH-1"])],
 ["SC-3"], links=TECH, issues=["参照と判定のずれが残っている実体の集合を承認してよいかを、まだ決めていない"])

D["UC-9"] = UC(9, "基盤の複製を転写する", "ユーザー目的", MK,
 [SH_MK("基盤の能力を書き直さずに、作る道具の中で使える")],
 [],
 [{"id": "SG-1", "name": "正本と同じ複製", "condition": {"target": "TERM-23", "agg": "count", "op": "le", "value": 0}, "satisfies": ["SH-1"]}],
 [say(1, MK, "システム", ["TERM-24"], serves=["SH-1"]),
  READ(2, MK, ["TERM-24"], ["TERM-50"], "EXT-1", reasons=("TERM-40",), serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-31", object="TERM-22", serves=["SH-1"]),
  WRITE(4, MK, ["TERM-50"], "EXT-2", serves=["SH-1"]),
  say(5, "システム", MK, ["TERM-24"], verb="TERM-26", serves=["SH-1"])],
 ["SC-4"], links=TECH,
 issues=["複製に手の変更があるときに、転写を拒むかを決めていない（転写すると、差の検査で見つけるはずの変更が消える）",
         "書く途中で失敗したときに、複製が半端に残らないことを最低保証に置くかを決めていない"])

D["UC-10"] = UC(10, "複製と正本の差を検査する", "ユーザー目的", MK,
 [SH_MK("正本が新しくなったことと、複製を手で書き換えたことに気づける")],
 [SAME("MG-1", "複製を書き換えない", "TERM-22", ["SH-1"])],
 [{"id": "SG-1", "name": "差の報告", "condition": {"target": "TERM-23", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, MK, "システム", ["TERM-24"], serves=["SH-1"]),
  READ(2, MK, ["TERM-24"], ["TERM-50"], "EXT-1", reasons=("TERM-40",), keeps=["MG-1"], serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-34", object="TERM-22", serves=["SH-1"]),
  say(4, "システム", MK, ["TERM-23"], verb="TERM-26", serves=["SH-1"],
      extensions=[other("EXT-2", {"target": "TERM-23", "agg": "count", "op": "le", "value": 0}, MK, ["TERM-23"])])],
 ["SC-4"], links=TECH)

def SD(i, name, desc, cls, bl, vals, ucs):
    return {"kind": "subdomain", "id": f"SD-{i}", "header": {"name": name, "description": desc},
            "classification": cls, "business_logic": bl, "serves_values": vals, "use_cases": ucs}
def BL(tr=False, rules=False, data=False, cond=()): return {"needs_tracking": tr, "complex_rules": rules, "complex_data": data, "rule_conditions": list(cond)}
D["SD-1"] = SD(1, "実体の読み書き", "スキーマから実体を作り、取得の式で読み、更新の差分で書き換え、消す。書き込む前に検証し、欄ごとの案内を返す",
 {"category": "一般", "competitive_advantage": False, "external_available": True, "cheaper_to_build": False, "sourcing": "JSON Schema ・ JMESPath ・ JSON Patch を実装した外の部品を組み込む。自分たちで書く部分（検証を通過したときだけ書く ・ 案内 ・ 置き場所）は薄く保つ"},
 BL(), ["VAL-1"], ["UC-1", "UC-2", "UC-3", "UC-4", "UC-7"])
D["SD-2"] = SD(2, "参照と判定", "スキーマの注釈 x-ref と x-derive だけを読んで、実体の集合の参照と判定を確かめ、承認のあとの変化を見つける",
 {"category": "中核", "competitive_advantage": True, "external_available": False, "cheaper_to_build": True, "sourcing": "自分たちで作る"},
 BL(rules=True), ["VAL-2"], ["UC-5", "UC-8"])
D["SD-3"] = SD(3, "描画", "頁の型と部品とトークンで、実体の集合から人が読む頁を組む",
 {"category": "補完", "competitive_advantage": False, "external_available": False, "cheaper_to_build": True, "sourcing": "自分たちで作る"},
 BL(), ["VAL-3"], ["UC-6"])
D["SD-4"] = SD(4, "転写", "正本の複製を基盤を使う道具へ写し、複製と正本の差を出す",
 {"category": "補完", "competitive_advantage": False, "external_available": False, "cheaper_to_build": True, "sourcing": "自分たちで作る"},
 BL(), ["VAL-4"], ["UC-9", "UC-10"])

FS = lambda i, ops: {"id": f"X-{i}", "external": "ファイルシステム", "owner": "利用者の環境", "pattern": "従属", "direction": "下流",
                     "operations": [{"id": f"OP-{n+1}", "name": o, "sends": ["TERM-7"], "receives": [] if o == "TERM-36" else ["TERM-48"],
                                     "failures": [{"id": "F-1", "name": "TERM-40"}, {"id": "F-2", "name": "TERM-51"}] if o == "TERM-35" else [{"id": "F-1", "name": "TERM-41"}]} for n, o in enumerate(ops)]}
def BC(i, name, purpose, sds, rels, terms, brs=(), pl=()):
    return {"kind": "context", "id": f"BC-{i}", "header": {"name": name, "purpose": purpose, "subdomains": sds},
            "context_map": {"relations": rels}, "boundary": {"kind": "Skill の道具", "owner": "schema-driven"},
            "uses": [{"term": f"TERM-{t}", "meaning": "M-1"} for t in terms], "business_rules": list(brs), "published_language": list(pl)}
IO = [25, 26, 35, 36, 40, 41, 42, 46, 47, 48, 51]
D["BC-1"] = BC(1, "実体の操作と検査", "実体の作成 ・ 取得 ・ 更新 ・ 削除と、書き込む前の検証と案内、実体の集合の参照と判定と承認のあとの変化を決める。読み書きと検査は同じ実体を扱うので1つの文脈に置き、内側を業務領域ごとのモジュールに分ける。頁の組み方は、このモデルに入れない",
 ["SD-1", "SD-2"], [FS(1, ["TERM-35", "TERM-36"])],
 sorted(set(IO + [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 27, 28, 29, 30, 32, 37, 38, 39, 43, 44, 52, 53])),
 [{"id": "BR-1", "condition": {"target": "TERM-5", "op": "not_empty"}, "implements": "REQ-1.BR-1"},
  {"id": "BR-2", "condition": {"target": "TERM-6", "op": "not_empty"}, "implements": "REQ-1.BR-2"},
  {"id": "BR-3", "condition": {"target": "TERM-2.TERM-4.TERM-10", "op": "not_empty"}, "implements": "REQ-1.BR-3"}],
 [{"name": "描画へ渡す実体の集合", "terms": ["TERM-11", "TERM-3", "TERM-12", "TERM-13"]}])
D["BC-2"] = BC(2, "描画", "頁の型の並びのとおりに、部品へ値を差し込んで頁を組む。欄の名前と、参照と判定の決まりは、このモデルに入れない",
 ["SD-3"], [FS(1, ["TERM-35", "TERM-36"]),
  {"id": "X-2", "external": "実体の操作と検査（BC-1）", "owner": "schema-driven", "pattern": "公開ホストサービス", "direction": "下流", "uses": ["TERM-11", "TERM-3", "TERM-12", "TERM-13"]}],
 sorted(set(IO + [1, 3, 7, 11, 12, 13, 17, 18, 19, 20, 33, 45, 49])))
D["BC-3"] = BC(3, "転写", "正本の複製を写し、複製と正本の差を判定する。実体と頁は、このモデルに入れない",
 ["SD-4"], [FS(1, ["TERM-35", "TERM-36"])],
 sorted(set(IO + [16, 21, 22, 23, 24, 31, 34, 50])))

os.makedirs(os.path.join(H, "decls"), exist_ok=True)
for f in os.listdir(os.path.join(H, "decls")): os.remove(os.path.join(H, "decls", f))
for k, v in D.items():
    json.dump(v, open(os.path.join(H, "decls", f"{k}.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(len(D), "件")
