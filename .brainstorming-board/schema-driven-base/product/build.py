# schema-driven を、ユースケース駆動の宣言でプロダクトとして書く（ボード schema-driven-base の次にすること1）。
# 材料：ボード schema-driven-base の論点1〜4の決定（出どころの種類は「決まったこと」）
# 検査：ボード usecase-driven-concrete の見本の道具（schema/check.py ・ concrete7）を外から当てる（run.py）
import json, os
H = os.path.dirname(os.path.abspath(__file__))

def T(i, w, kind, d, origin, ref=None, avoid=(), **kw):
    m = {"id": "M-1", "definition": d, "kind": kind}; m.update(kw)
    t = {"id": f"TERM-{i}", "word": w, "origin": origin}
    if ref: t["origin_ref"] = ref
    t.update({"meanings": [m], "avoid": list(avoid)}); return t
JS, JM, JP, US, IT = "JSON Schema", "JMESPath", "JSON Patch（RFC 6902）", "利用者", "一般の言葉"
SPEC, BIZ = "仕様の用語", "業務の言葉"

TERMS = [
 T(1, "インスタンス", "集約", "スキーマに従う JSON のデータ1件。正本であり、1つのファイルに置く", SPEC, JS, avoid=["実体"]),
 T(2, "スキーマ", "値オブジェクト", "インスタンスの形と注釈を書いた JSON Schema", SPEC, JS),
 T(3, "注釈", "情報の別名", "スキーマに書く x- で始まるキー（x-ref ・ x-derive ・ x-view ・ x-prompt ・ x-generates）", SPEC, JS),
 T(4, "プロパティ", "値オブジェクト", "インスタンスの中の1つの値の場所。JSON Pointer で指す", SPEC, JS),
 T(5, "JMESPath 式", "値オブジェクト", "インスタンスから値を取り出す式", SPEC, JM, avoid=["取得の式"]),
 T(6, "JSON Patch", "値オブジェクト", "インスタンスへ適用する操作の並び", SPEC, JP, avoid=["更新の差分"]),
 T(7, "パス", "値オブジェクト", "インスタンスのファイルの場所。スキーマの x-generates が決める", IT, avoid=["置き場所"]),
 T(8, "検証エラー", "値オブジェクト", "インスタンスがスキーマを満たさないプロパティと、その理由", IT, avoid=["違反"]),
 T(9, "検証結果", "情報の別名", "インスタンスごとの検証エラーの一覧", IT),
 T(10, "x-prompt", "情報の別名", "プロパティごとの書き方。読むときの read と、値を書くときの write を持つ", BIZ, "schema-driven の注釈", avoid=["案内"]),
 T(11, "ディレクトリ", "情報の別名", "インスタンスを置くディレクトリ。参照と導出値は、その下のインスタンスすべてに対して確かめる", IT, avoid=["実体の集合"]),
 T(12, "参照", "値オブジェクト", "x-ref を付けたプロパティの値。ほかのインスタンスか、その中の項目を指す", IT),
 T(13, "導出値", "値オブジェクト", "x-derive の決まりに従って、同じインスタンスの答えのプロパティから導いた値", BIZ, "schema-driven の注釈", avoid=["判定"]),
 T(14, "検査結果", "値オブジェクト", "参照と導出値の検査1件ごとの結果。合格 ・ ずれ ・ 確かめ直しのどれか", IT),
 T(15, "承認記録", "集約", "承認した時点の、インスタンスごとのハッシュ値", BIZ, US),
 T(16, "ハッシュ値", "値オブジェクト", "ファイルの内容の sha256", IT),
 T(17, "ページ", "情報の別名", "ディレクトリのインスタンスから描画した HTML または Markdown", IT, avoid=["頁"]),
 T(18, "ページテンプレート", "値オブジェクト", "種類ごとのページの、節の並びと、節ごとのコンポーネントと差し込むプロパティを書いたデータ", IT, avoid=["頁の型"]),
 T(19, "コンポーネント", "値オブジェクト", "ページを組む HTML のテンプレート。差し込む場所を持つ", IT, avoid=["部品"]),
 T(20, "デザイントークン", "値オブジェクト", "見た目の値（色 ・ 寸法 ・ 余白 ・ 文字）に付けた名前と、その値", IT),
 T(21, "正本", "集約", "schema-driven が持つ基盤の能力。Rust の crate ・ コンポーネント ・ デザイントークン", BIZ, US),
 T(22, "複製", "集約", "利用側の Skill の中へ転写した、正本の写し。利用側の Skill は、これだけで動く", BIZ, US),
 T(23, "差分", "値オブジェクト", "複製のうち、正本と内容が違うファイル", IT, avoid=["差"]),
 T(24, "利用側の Skill", "情報の別名", "schema-driven の複製を持ち、それで動く Skill", IT, avoid=["基盤を使う道具"]),
 T(25, "依頼する", "動作", "相手に何かを頼む", IT, form="{to}に{data}を依頼する"),
 T(26, "知らせる", "動作", "相手に結果を伝える", IT, form="{to}に{data}を知らせる"),
 T(27, "作成する", "コマンド", "スキーマから、未記入のプロパティを持つインスタンスを作る", IT),
 T(28, "更新する", "コマンド", "インスタンスへ JSON Patch を適用する。適用したあとのインスタンスが検証を通過したときだけ書く", IT),
 T(29, "削除する", "コマンド", "インスタンスのファイルを消す。インスタンスの中の項目は、remove の JSON Patch で更新して消す", IT),
 T(30, "承認を記録する", "コマンド", "検査で使ったインスタンスのパスとハッシュ値の並びを、そのまま承認記録へ書く", BIZ, US),
 T(31, "転写する", "コマンド", "正本を、利用側の Skill の中へ写す", BIZ, US),
 T(32, "検査する", "ドメインサービス", "ディレクトリのインスタンスについて、指す先がある ・ 指される数 ・ 導出値と宣言した値 ・ 承認のあとの変化を確かめる", IT),
 T(33, "描画する", "ドメインサービス", "ページテンプレートとコンポーネントとデザイントークンで、ディレクトリのインスタンスからページを組む。ページテンプレートが無い種類は、注釈だけから組む", IT),
 T(34, "比べる", "ドメインサービス", "複製と正本を、ファイルごとのハッシュ値で比べる", IT),
 T(35, "読む", "動作", "相手からファイルの内容を受け取る", IT, form="{to}から{data}を読む"),
 T(36, "書く", "動作", "相手へファイルの内容を渡して置かせる", IT, form="{to}へ{data}を書く"),
 T(37, "パスにインスタンスが既にある", "拒否の理由", "作成しようとしたパスに、インスタンスが既にあるので作らない", IT),
 T(38, "JSON Patch を適用できない", "拒否の理由", "JSON Patch が指すプロパティがインスタンスに無いなど、適用できないので書かない", IT),
 T(39, "検証を通過しない", "拒否の理由", "JSON Patch を適用したあとのインスタンスが検証を通過しないので、書かない", IT),
 T(40, "読めない", "失敗の種類", "ファイルが無いか、読む権限が無い", IT),
 T(41, "書けない", "失敗の種類", "書く権限が無いか、ディスクに空きが無い", IT),
 T(42, "エラーメッセージ", "情報の別名", "依頼を受け付けなかった理由", IT, avoid=["誤りの理由"]),
 T(43, "取得した値", "情報の別名", "JMESPath 式をインスタンスに当てて得た JSON の値", IT),
 T(44, "残っている参照", "情報の別名", "削除したインスタンスを、まだ指している参照", IT),
 T(45, "ページのパス", "値オブジェクト", "描画したページを書いたファイルの場所", IT),
 T(48, "ファイルの内容", "情報の別名", "インスタンス ・ スキーマ ・ 承認記録 ・ ページテンプレート ・ コンポーネント ・ デザイントークン ・ ページのファイルを、文字列にしたもの", IT),
 T(50, "複製のファイル", "情報の別名", "複製を作るファイルと、その内容", IT),
 T(51, "JSON として読めない", "失敗の種類", "ファイルの内容が JSON の文法に沿わない", IT),
 T(52, "ほかの更新と競合した", "拒否の理由", "読んだ時点のハッシュ値が、いまのインスタンスのハッシュ値と違うので書かない", IT),
 T(53, "検証を通過しないインスタンスがある", "拒否の理由", "ディレクトリに検証を通過しないインスタンスがあるので、承認を記録しない", IT),
 T(54, "複製に手の変更がある", "拒否の理由", "複製のファイルが、前に転写した時点のハッシュ値と違うので転写しない。正本を直してから転写し直す", IT),
 T(55, "参照と導出値のずれがある", "拒否の理由", "検査結果にずれが残っているので、承認を記録しない", IT),
 T(56, "承認したインスタンス", "値オブジェクト", "承認記録の中の、1つのインスタンスのパスとハッシュ値の組", IT),
 T(57, "参照を確かめる", "操作", "参照ごとに、指す先があるか ・ 指す先の種類が合うか ・ 指される数が決まりの範囲かを確かめる", IT),
 T(58, "導出値を確かめる", "操作", "x-derive の決まりで導いた値と、宣言した値が同じかを確かめる", IT),
 T(59, "変化を確かめる", "操作", "インスタンスのハッシュ値と、承認記録のハッシュ値が同じかを確かめる", IT),
 T(61, "JSON の値", "値オブジェクト", "インスタンスのファイルに書いてある JSON", SPEC, "JSON（RFC 8259）"),
 T(62, "ハッシュ値を求める", "操作", "JSON の値の sha256 を求める", IT),
 T(63, "適用する", "操作", "JSON Patch を JSON の値に当てて、新しい値を求める", SPEC, JP),
 T(60, "ずれ", "値オブジェクト", "ずれと出た検査の名前と、その指す先", IT),
]

D = {}
D["GLO-1"] = {"kind": "glossary", "id": "GLO-1", "header": {"name": "用語集"}, "terms": TERMS}

D["DOM-1"] = {"kind": "domain", "id": "DOM-1", "header": {"name": "schema-driven"},
 "vision": {
  "problem": "AI が扱うデータを Markdown の文書で持つと、どこに何が書いてあるかが決まらず、AI は文字列を探して読むことになる。指す先が消えたことや、答えと宣言した値の食い違いに、書いた時点で気づけない。同じ読み書きの能力を道具ごとに書くので、道具どうしでずれる",
  "values": [
   {"id": "VAL-1", "name": "プロパティを式で指して読み書きできる", "text": "正本を構造化データで持ち、JMESPath 式で値を取り出し、JSON Patch で書き換える", "differentiator": False},
   {"id": "VAL-2", "name": "参照と導出値のずれを道具が出す", "text": "インスタンスどうしの参照と、答えから導いた値を、スキーマの注釈だけを読んで確かめる", "differentiator": True},
   {"id": "VAL-3", "name": "人が読むページを正本から描画する", "text": "ページテンプレートとコンポーネントで、ディレクトリのインスタンスから人が読むページを組む", "differentiator": False},
   {"id": "VAL-4", "name": "基盤の能力を複製して配れる", "text": "利用側の Skill へ正本の複製を転写し、正本との差分を道具が出す", "differentiator": False}],
  "competitors": [
   {"id": "CMP-1", "name": "Markdown の文書を正本にするやり方", "lacks": ["VAL-1", "VAL-2"]},
   {"id": "CMP-2", "name": "JSON Schema の検証器だけを使うやり方", "lacks": ["VAL-2", "VAL-3"]}],
  "success_criteria": [
   {"id": "SC-1", "name": "文字列を探さずに読める", "source": "external", "measure_text": "AI エージェントが正本を読むときに、文字列の検索を使った回数", "threshold": {"op": "le", "value": 0, "unit": "回"}, "window": "作業1件"},
   {"id": "SC-2", "name": "検証を通過しない更新が書き込まれない", "source": "external", "measure_text": "更新で書き込んだインスタンスのうち、検証を通過しないものの件数", "threshold": {"op": "le", "value": 0, "unit": "件"}, "window": "更新のたび"},
   {"id": "SC-3", "name": "ずれを承認の前に見つける", "source": "external", "measure_text": "承認したあとに見つかった、参照と導出値のずれの件数", "threshold": {"op": "le", "value": 0, "unit": "件"}, "window": "承認1回"},
   {"id": "SC-4", "name": "複製が正本と同じ", "source": "external", "measure_text": "転写し直したあとの、複製と正本の差分の件数", "threshold": {"op": "le", "value": 0, "unit": "件"}, "window": "転写1回"}]},
 "scope": {"out": ["宣言の種類（ドメイン ・ ユースケースなど）の形", "テスト条件とテストの記録の突き合わせ", "図の描画"]},
 "design_scopes": [
  {"id": "SCP-1", "level": "システム", "name": "schema-driven", "outside": ["SH-1", "SH-2", "SH-3", "ファイルシステム"]}],
 "stakeholders": [
  {"id": "SH-1", "who": "AI エージェント", "interest": "プロパティを式で指して読み書きし、書いたものがその場で確かめられる"},
  {"id": "SH-2", "who": "データの持ち主", "interest": "正本が形と参照と導出値を満たし、承認したあとの変化に気づける。正本をページで読める"},
  {"id": "SH-3", "who": "道具の作り手", "interest": "基盤の能力を書き直さずに使え、正本が新しくなったことに気づける"}]}

D["REQ-1"] = {"kind": "other_requirements", "id": "REQ-1", "header": {"name": "その他の要求"},
 "business_rules": [
  {"id": "BR-1", "name": "JMESPath 式を渡す", "condition": {"target": "TERM-5", "op": "not_empty"}, "protects": ["DOM-1.SH-1"]},
  {"id": "BR-2", "name": "JSON Patchを渡す", "condition": {"target": "TERM-6", "op": "not_empty"}, "protects": ["DOM-1.SH-1"]},
  {"id": "BR-3", "name": "x-prompt を求めるプロパティがスキーマに在り、x-prompt を持つ", "condition": {"target": "TERM-2.TERM-4.TERM-10", "op": "not_empty"}, "protects": ["DOM-1.SH-1"]}],
 "quality": [],
 "technology": [
  {"id": "TEC-1", "system": "ファイルシステム", "text": "インスタンス ・ スキーマ ・ ページ ・ 承認記録 ・ 複製は、利用者の環境のファイルとして読み書きする"},
  {"id": "TEC-2", "system": "AI エージェントの実行環境", "text": "CLI と MCP のどちらからも、同じ道具の一覧を呼べる"}],
 "data": [],
 "open_issues": ["JMESPath 式が文法に沿うことを、条件で書けない", "適用したあとのインスタンスで決まる拒否（JSON Patch を適用できない ・ 検証を通過しない）を、集約の業務ルールの条件で書けない。検証を通過しないことは、更新の状態の変更（検証エラーが0件）で書いた。条件が適用したあとの状態を指す書き方（before の対）があれば、更新の業務ルールに書ける", "承認したインスタンスのパスが重ならないことを、承認記録の不変条件の条件で書けない", "条件に「含まれる」の比べ方が無いので、検査するの結果（指す先が無い ・ 種類が違う）を「違う」で近い形に書いた", "同じ入力から同じページが出ることを、品質の要求の形で書けない"]}

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
READ = lambda i, who, data, reply, eid, reasons=("TERM-40", "TERM-51"), **kw: say(i, "システム", "ファイルシステム", data, verb="TERM-35", reply=reply, extensions=[fail_fs(eid, list(reasons), who)] + kw.pop("ext", []), **kw)
WRITE = lambda i, who, data, eid, **kw: say(i, "システム", "ファイルシステム", data, verb="TERM-36", extensions=[fail_fs(eid, ["TERM-41"], who)], **kw)
def other(eid, cond, who, data, ending="終了"):
    return {"id": eid, "condition_kind": "別の道筋での成功", "condition": cond, "ending": ending,
            "steps": [sub(f"{eid}.S-1", "相互作用", "システム", to=who, data=data, verb="TERM-26")]}
TECH = {"technology": ["REQ-1.TEC-1", "REQ-1.TEC-2"]}

D["UC-0"] = UC(0, "構造化データを正本として持つ", "要約", OWN,
 [SH_OWN("承認した正本が、形と参照と導出値を満たす", "SH-1"), dict(SH_AI("プロパティを式で指して読み書きし、書いたものをその場で確かめる"), id="SH-2")],
 [{"id": "MG-1", "name": "承認した正本は検証を通過している", "condition": {"if": {"target": "TERM-15.TERM-16", "op": "not_empty"}, "target": "TERM-11.TERM-8", "agg": "count", "op": "le", "value": 0}, "protects": ["SH-1"]}],
 [{"id": "SG-1", "name": "承認した正本", "condition": {"target": "TERM-15.TERM-16", "op": "not_empty"}, "satisfies": ["SH-1", "SH-2"]}],
 [step(1, "サブユースケースの呼び出し", AI, calls="UC-1", serves=["SH-2"]),
  step(2, "サブユースケースの呼び出し", AI, calls="UC-7", serves=["SH-2"]),
  step(3, "サブユースケースの呼び出し", AI, calls="UC-3", serves=["SH-1", "SH-2"]),
  step(4, "サブユースケースの呼び出し", OWN, calls="UC-6", serves=["SH-1"]),
  step(5, "サブユースケースの呼び出し", OWN, calls="UC-8", keeps=["MG-1"], serves=["SH-1"])],
 ["SC-1", "SC-2", "SC-3"], links={}, supporting=(),
 issues=["AI エージェントが、データの持ち主の作業を代わりに進める形を前提にしている"])

D["UC-1"] = UC(1, "インスタンスを作成する", "ユーザー目的", AI,
 [SH_AI("未記入のプロパティと、そこへ何を書くかが分かる"), SH_OWN("パスにあるインスタンスを上書きしない")],
 [SAME("MG-1", "既にあるインスタンスを書き換えない", "TERM-1.TERM-16", ["SH-2"])],
 [{"id": "SG-1", "name": "パスのインスタンス", "condition": {"target": "TERM-1.TERM-7", "op": "not_empty"}, "satisfies": ["SH-1"]},
  {"id": "SG-2", "name": "未記入のプロパティの x-prompt", "condition": {"target": "TERM-10", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, AI, "システム", ["TERM-2", "TERM-7"], serves=["SH-1"]),
  READ(2, AI, ["TERM-2"], ["TERM-48"], "EXT-1", serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-27", object="TERM-1", keeps=["MG-1"], serves=["SH-1", "SH-2"],
       extensions=[reject("EXT-2", ["TERM-37"], AI)]),
  WRITE(4, AI, ["TERM-48"], "EXT-3", serves=["SH-1"]),
  say(5, "システム", AI, ["TERM-7", "TERM-10"], verb="TERM-26", serves=["SH-1"])],
 ["SC-1"], links=TECH,
 issues=["作成した直後のインスタンスは未記入のプロパティを持つので、検証を通過しなくてよい（ボード schema-driven-base 論点1）"])

D["UC-2"] = UC(2, "値を取得する", "サブ機能", AI,
 [SH_AI("文字列を探さずに、プロパティの値だけを受け取る")],
 [SAME("MG-1", "インスタンスを書き換えない", "TERM-1.TERM-16", ["SH-1"])],
 [{"id": "SG-1", "name": "取得した値", "condition": {"target": "TERM-43", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, AI, "システム", ["TERM-7", "TERM-5"], serves=["SH-1"]),
  step(2, "妥当性確認", "システム", checks=["REQ-1.BR-1"], serves=["SH-1"], extensions=[invalid("EXT-1", "REQ-1.BR-1", AI)]),
  READ(3, AI, ["TERM-7"], ["TERM-48"], "EXT-2", keeps=["MG-1"], serves=["SH-1"]),
  say(4, "システム", AI, ["TERM-43"], verb="TERM-26", serves=["SH-1"],
      extensions=[other("EXT-3", {"target": "TERM-43", "op": "empty"}, AI, ["TERM-43"])])],
 ["SC-1"], links={"business_rules": ["REQ-1.BR-1"], **TECH})

D["UC-3"] = UC(3, "インスタンスを更新する", "ユーザー目的", AI,
 [SH_AI("更新したインスタンスがその場で確かめられ、通らなければ理由が分かる"), SH_OWN("パスに、検証を通過しない差分が書き込まれない")],
 [SAME("MG-1", "失敗したら前のインスタンスのまま", "TERM-1.TERM-16", ["SH-2"])],
 [{"id": "SG-1", "name": "検証を通過したインスタンスの書き込み", "condition": VALID, "satisfies": ["SH-1", "SH-2"]}],
 [say(1, AI, "システム", ["TERM-7", "TERM-6"], serves=["SH-1"]),
  step(2, "妥当性確認", "システム", checks=["REQ-1.BR-2"], serves=["SH-1"], extensions=[invalid("EXT-1", "REQ-1.BR-2", AI)]),
  READ(3, AI, ["TERM-7", "TERM-2"], ["TERM-48"], "EXT-2", serves=["SH-1"]),
  step(4, "内部の状態変化", "システム", verb="TERM-28", object="TERM-1", keeps=["MG-1"], serves=["SH-1", "SH-2"],
       extensions=[reject("EXT-3", ["TERM-38", "TERM-52"], AI), reject("EXT-4", ["TERM-39"], AI, data="TERM-9"),
                   other("EXT-5", {"target": "TERM-1.TERM-16", "op": "eq", "value": {"before": "TERM-1.TERM-16"}}, AI, ["TERM-9"])]),
  WRITE(5, AI, ["TERM-48"], "EXT-6", keeps=["MG-1"], serves=["SH-1"]),
  say(6, "システム", AI, ["TERM-9"], verb="TERM-26", serves=["SH-1"])],
 ["SC-2"], links={"business_rules": ["REQ-1.BR-2"], **TECH},
 issues=["項目の削除は、remove の JSON Patch を渡すこのユースケースで扱う"])

D["UC-4"] = UC(4, "インスタンスを削除する", "ユーザー目的", AI,
 [SH_AI("消したインスタンスを、まだ指している参照が分かる")],
 [],
 [{"id": "SG-1", "name": "インスタンスのファイルが無い", "condition": {"target": "TERM-1", "op": "empty"}, "satisfies": ["SH-1"]}],
 [say(1, AI, "システム", ["TERM-7"], serves=["SH-1"]),
  READ(2, AI, ["TERM-7"], ["TERM-48"], "EXT-1", serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-29", object="TERM-1", serves=["SH-1"]),
  WRITE(4, AI, ["TERM-7"], "EXT-2", serves=["SH-1"]),
  READ(5, AI, ["TERM-7"], ["TERM-48"], "EXT-3", serves=["SH-1"]),
  step(6, "内部の状態変化", "システム", verb="TERM-32", object="TERM-11", serves=["SH-1"]),
  say(7, "システム", AI, ["TERM-44"], verb="TERM-26", serves=["SH-1"])],
 ["SC-3"], links=TECH, issues=["インスタンスの中の項目を消すときは、remove の JSON Patch でインスタンスを更新して消す（インスタンスを更新する）"])

D["UC-5"] = UC(5, "ディレクトリのインスタンスを検査する", "ユーザー目的", OWN,
 [SH_OWN("指す先の無い参照 ・ 指される数の過不足 ・ 導出値と宣言した値の食い違い ・ 承認のあとの変化が、承認の前に分かる", "SH-1")],
 [SAME("MG-1", "インスタンスを書き換えない", "TERM-11.TERM-16", ["SH-1"])],
 [{"id": "SG-1", "name": "検査の結果", "condition": {"target": "TERM-14", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, OWN, "システム", ["TERM-7"], serves=["SH-1"]),
  READ(2, OWN, ["TERM-7"], ["TERM-48"], "EXT-1", keeps=["MG-1"], serves=["SH-1"],
       ext=[other("EXT-2", {"target": "TERM-11", "op": "empty"}, OWN, ["TERM-14"])]),
  step(3, "内部の状態変化", "システム", verb="TERM-32", object="TERM-11", serves=["SH-1"],
       variations=[{"varies": "承認のあとの変化", "values": ["承認記録があれば、記録したハッシュ値と比べる", "承認記録が無ければ、変化の検査を外す"]}]),
  say(4, "システム", OWN, ["TERM-9", "TERM-14"], verb="TERM-26", serves=["SH-1"])],
 ["SC-3"], links=TECH)

D["UC-6"] = UC(6, "ページを描画する", "ユーザー目的", OWN,
 [SH_OWN("正本を、種類ごとに設計したページで読める", "SH-1")],
 [SAME("MG-1", "インスタンスを書き換えない", "TERM-11.TERM-16", ["SH-1"])],
 [{"id": "SG-1", "name": "描画したページ", "condition": {"target": "TERM-45", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, OWN, "システム", ["TERM-7"], serves=["SH-1"]),
  READ(2, OWN, ["TERM-7"], ["TERM-48"], "EXT-1", keeps=["MG-1"], serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-33", object="TERM-17", serves=["SH-1"],
       variations=[{"varies": "組み方", "values": ["ページテンプレートがある種類は、ページテンプレートとコンポーネントで組む", "ページテンプレートが無い種類は、注釈だけから組む"]}]),
  WRITE(4, OWN, ["TERM-48"], "EXT-2", serves=["SH-1"]),
  say(5, "システム", OWN, ["TERM-45"], verb="TERM-26", serves=["SH-1"])],
 [], links=TECH, issues=["ページテンプレートが指すプロパティがインスタンスに無いときは「なし」と描く。拡張にするかを決めていない"])

D["UC-7"] = UC(7, "x-prompt を受け取る", "サブ機能", AI,
 [SH_AI("プロパティごとに、何を読み何を書くかが分かる")],
 [],
 [{"id": "SG-1", "name": "x-prompt", "condition": {"target": "TERM-10", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, AI, "システム", ["TERM-2", "TERM-4"], serves=["SH-1"]),
  READ(2, AI, ["TERM-2"], ["TERM-48"], "EXT-1", serves=["SH-1"]),
  step(3, "妥当性確認", "システム", checks=["REQ-1.BR-3"], serves=["SH-1"], extensions=[invalid("EXT-2", "REQ-1.BR-3", AI)]),
  say(4, "システム", AI, ["TERM-10"], verb="TERM-26", serves=["SH-1"])],
 ["SC-1"], links={"business_rules": ["REQ-1.BR-3"], **TECH})

D["UC-8"] = UC(8, "承認を記録する", "ユーザー目的", OWN,
 [SH_OWN("承認した時点の正本が残り、そのあとの変化を検査で見つけられる", "SH-1")],
 [SAME("MG-1", "拒んだら前の承認記録のまま", "TERM-15.TERM-16", ["SH-1"])],
 [{"id": "SG-1", "name": "承認記録", "condition": {"target": "TERM-15.TERM-16", "op": "not_empty"}, "satisfies": ["SH-1"]}],
 [say(1, OWN, "システム", ["TERM-7"], serves=["SH-1"]),
  step(2, "サブユースケースの呼び出し", OWN, calls="UC-5", serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-30", object="TERM-15", keeps=["MG-1"], serves=["SH-1"],
       extensions=[reject("EXT-1", ["TERM-53"], OWN, data="TERM-9"), reject("EXT-4", ["TERM-55"], OWN, data="TERM-14"),
                   other("EXT-2", {"target": "TERM-11.TERM-16", "op": "eq", "value": {"before": "TERM-15.TERM-16"}}, OWN, ["TERM-16"])]),
  WRITE(4, OWN, ["TERM-48"], "EXT-3", keeps=["MG-1"], serves=["SH-1"]),
  say(5, "システム", OWN, ["TERM-16"], verb="TERM-26", serves=["SH-1"])],
 ["SC-3"], links=TECH)

D["UC-9"] = UC(9, "基盤の複製を転写する", "ユーザー目的", MK,
 [SH_MK("基盤の能力を書き直さずに、作る道具の中で使える")],
 [SAME("MG-1", "失敗したら前の複製のまま", "TERM-22.TERM-16", ["SH-1"])],
 [{"id": "SG-1", "name": "正本と同じ複製", "condition": {"target": "TERM-23", "agg": "count", "op": "le", "value": 0}, "satisfies": ["SH-1"]}],
 [say(1, MK, "システム", ["TERM-24"], serves=["SH-1"]),
  READ(2, MK, ["TERM-24"], ["TERM-50"], "EXT-1", reasons=("TERM-40",), serves=["SH-1"]),
  step(3, "内部の状態変化", "システム", verb="TERM-31", object="TERM-22", keeps=["MG-1"], serves=["SH-1"],
       extensions=[reject("EXT-3", ["TERM-54"], MK, data="TERM-23")]),
  WRITE(4, MK, ["TERM-50"], "EXT-2", keeps=["MG-1"], serves=["SH-1"]),
  say(5, "システム", MK, ["TERM-24"], verb="TERM-26", serves=["SH-1"])],
 ["SC-4"], links=TECH)

D["UC-10"] = UC(10, "複製と正本の差分を検査する", "ユーザー目的", MK,
 [SH_MK("正本が新しくなったことと、複製を手で書き換えたことに気づける")],
 [SAME("MG-1", "複製を書き換えない", "TERM-22", ["SH-1"])],
 [{"id": "SG-1", "name": "差分の報告", "condition": {"target": "TERM-23", "op": "not_empty"}, "satisfies": ["SH-1"]}],
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
D["SD-1"] = SD(1, "インスタンスの読み書き", "スキーマからインスタンスを作り、JMESPath 式で読み、JSON Patch で書き換え、消す。書く前に検証し、プロパティごとの x-prompt を返す",
 {"category": "一般", "competitive_advantage": False, "external_available": True, "cheaper_to_build": False, "sourcing": "JSON Schema ・ JMESPath ・ JSON Patch を実装した外のコンポーネントを組み込む。自分たちで書く部分（検証を通過したときだけ書く ・ x-prompt ・ パス）は薄く保つ"},
 BL(), ["VAL-1"], ["UC-1", "UC-2", "UC-3", "UC-4", "UC-7"])
D["SD-2"] = SD(2, "参照と導出値", "スキーマの注釈 x-ref と x-derive だけを読んで、ディレクトリのインスタンスの参照と導出値を確かめ、承認のあとの変化を見つける",
 {"category": "中核", "competitive_advantage": True, "external_available": False, "cheaper_to_build": True, "sourcing": "自分たちで作る"},
 BL(rules=True, cond=["DS-1.OP-1.RES-3", "DS-1.OP-1.RES-4", "DS-1.OP-3.RES-2", "AGG-1.INV-1", "AGG-2.CMD-1.BR-2"]), ["VAL-2"], ["UC-5", "UC-8"])
D["SD-3"] = SD(3, "描画", "ページテンプレートとコンポーネントとトークンで、ディレクトリのインスタンスから人が読むページを組む",
 {"category": "補完", "competitive_advantage": False, "external_available": False, "cheaper_to_build": True, "sourcing": "自分たちで作る"},
 BL(), ["VAL-3"], ["UC-6"])
D["SD-4"] = SD(4, "転写", "正本の複製を利用側の Skill へ写し、複製と正本の差分を出す",
 {"category": "補完", "competitive_advantage": False, "external_available": False, "cheaper_to_build": True, "sourcing": "自分たちで作る"},
 BL(), ["VAL-4"], ["UC-9", "UC-10"])

FS = lambda i, ops: {"id": f"X-{i}", "external": "ファイルシステム", "owner": "利用者の環境", "pattern": "従属", "direction": "下流",
                     "operations": [{"id": f"OP-{n+1}", "name": o, "sends": ["TERM-7"], "receives": [] if o == "TERM-36" else ["TERM-48"],
                                     "failures": [{"id": "F-1", "name": "TERM-40"}, {"id": "F-2", "name": "TERM-51"}] if o == "TERM-35" else [{"id": "F-1", "name": "TERM-41"}]} for n, o in enumerate(ops)]}
def BC(i, name, purpose, sds, rels, terms, brs=(), pl=()):
    return {"kind": "context", "id": f"BC-{i}", "header": {"name": name, "purpose": purpose, "subdomains": sds},
            "context_map": {"relations": rels}, "boundary": {"kind": "Skill の道具", "owner": "schema-driven"},
            "uses": [{"term": f"TERM-{t}", "meaning": "M-1"} for t in terms], "business_rules": list(brs), "published_language": list(pl)}
IO = [25, 26, 35, 36, 40, 41, 42, 48, 51]
D["BC-1"] = BC(1, "インスタンスの操作と検査", "インスタンスの作成 ・ 取得 ・ 更新 ・ 削除と、書き込む前の検証と x-prompt、ディレクトリのインスタンスの参照と導出値と承認のあとの変化を決める。読み書きと検査は同じインスタンスを扱うので1つの文脈に置き、内側を業務領域ごとのモジュールに分ける。ページの組み方は、このモデルに入れない",
 ["SD-1", "SD-2"], [FS(1, ["TERM-35", "TERM-36"])],
 sorted(set(IO + [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 27, 28, 29, 30, 32, 37, 38, 39, 43, 44, 52, 53, 55, 56, 57, 58, 59, 60, 61, 62, 63])),
 [{"id": "BR-1", "condition": {"target": "TERM-5", "op": "not_empty"}, "implements": "REQ-1.BR-1"},
  {"id": "BR-2", "condition": {"target": "TERM-6", "op": "not_empty"}, "implements": "REQ-1.BR-2"},
  {"id": "BR-3", "condition": {"target": "TERM-2.TERM-4.TERM-10", "op": "not_empty"}, "implements": "REQ-1.BR-3"}],
 [{"name": "描画へ渡すディレクトリのインスタンス", "terms": ["TERM-11", "TERM-3", "TERM-12", "TERM-13"]}])
D["BC-2"] = BC(2, "描画", "ページテンプレートの並びのとおりに、コンポーネントへ値を差し込んでページを組む。プロパティの名前と、参照と導出値の決まりは、このモデルに入れない",
 ["SD-3"], [FS(1, ["TERM-35", "TERM-36"]),
  {"id": "X-2", "external": "インスタンスの操作と検査（BC-1）", "owner": "schema-driven", "pattern": "公開ホストサービス", "direction": "下流", "uses": ["TERM-11", "TERM-3", "TERM-12", "TERM-13"]}],
 sorted(set(IO + [1, 3, 7, 11, 12, 13, 17, 18, 19, 20, 33, 45])))
D["BC-3"] = BC(3, "転写", "正本の複製を写し、複製と正本の差分を出す。インスタンスとページは、このモデルに入れない",
 ["SD-4"], [FS(1, ["TERM-35", "TERM-36"])],
 sorted(set(IO + [16, 21, 22, 23, 24, 31, 34, 50, 54])))

# ── 設計の側（次にすること2）：中核のサブドメイン「参照と導出値」を担う文脈 BC-1 の集約 ・ ドメインサービス ・ 値オブジェクト
one, many = {"min": 1, "max": 1}, {"min": 0, "max": None}
H0 = "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"  # sha256("{}")
H1 = "015abd7f5cc57a2dd94b7590f04ad8084273905ee33ec5cebeae62276a97f862"  # sha256('{"a":1}')
PATCH = '[{"op":"add","path":"/a","value":1}]'
D["AGG-1"] = {"kind": "aggregate", "id": "AGG-1", "header": {"name": "TERM-1", "context": "BC-1"},
 "structure": {"state": [
   {"id": "ST-1", "name": "TERM-7", "type": "VO-1", "multiplicity": one},
   {"id": "ST-2", "name": "TERM-2", "type": "VO-2", "multiplicity": one},
   {"id": "ST-7", "name": "TERM-61", "type": "VO-10", "multiplicity": one},
   {"id": "ST-3", "name": "TERM-16", "type": "VO-6", "multiplicity": one},
   {"id": "ST-4", "name": "TERM-12", "type": "VO-3", "multiplicity": many},
   {"id": "ST-5", "name": "TERM-13", "type": "VO-4", "multiplicity": many},
   {"id": "ST-6", "name": "TERM-8", "type": "VO-5", "multiplicity": many}], "entities": []},
 "invariants": [{"id": "INV-1", "condition": {"target": "ST-3", "op": "eq", "value": {"call": "VO-10.OP-1", "args": ["ST-7"]}}, "via": ["CMD-1", "CMD-2"]}],
 "commands": [
  {"id": "CMD-1", "name": "TERM-27", "args": [{"id": "ARG-1", "name": "TERM-7", "type": "VO-1"}, {"id": "ARG-2", "name": "TERM-2", "type": "VO-2"}],
   "business_rules": [{"id": "BR-1", "condition": {"target": "ST-1", "op": "empty"}, "reject": "TERM-37",
                       "example": {"before": {"ST-1": "decls/UC-1.json"}, "args": {"ARG-1": "decls/UC-1.json", "ARG-2": "use_case.schema.json"}}}],
   "state_changes": [{"id": "CHG-1", "condition": {"target": "ST-1", "op": "eq", "value": "ARG-1"}},
                     {"id": "CHG-2", "condition": {"target": "ST-2", "op": "eq", "value": "ARG-2"}}],
   "emits": [], "accept_examples": [{"id": "OK-1", "before": {}, "args": {"ARG-1": "decls/UC-1.json", "ARG-2": "use_case.schema.json"}}]},
  {"id": "CMD-2", "name": "TERM-28", "args": [{"id": "ARG-1", "name": "TERM-6", "type": "VO-7"}, {"id": "ARG-2", "name": "TERM-16", "type": "VO-6"}],
   "business_rules": [{"id": "BR-1", "condition": {"target": "ST-3", "op": "eq", "value": "ARG-2"}, "reject": "TERM-52",
                       "example": {"before": {"ST-7": "{}", "ST-3": H0}, "args": {"ARG-1": PATCH, "ARG-2": H1}}}],
   "state_changes": [{"id": "CHG-3", "condition": {"target": "ST-7", "op": "eq", "value": {"call": "VO-7.OP-1", "args": ["ARG-1", {"before": "ST-7"}]}}},
                     {"id": "CHG-2", "condition": {"target": "ST-6", "agg": "count", "op": "le", "value": 0}}],
   "emits": [], "accept_examples": [{"id": "OK-1", "before": {"ST-7": "{}", "ST-3": H0}, "args": {"ARG-1": PATCH, "ARG-2": H0}}]},
  {"id": "CMD-3", "name": "TERM-29", "args": [], "business_rules": [],
   "state_changes": [{"id": "CHG-1", "condition": {"target": "ST-1", "op": "empty"}}],
   "emits": [], "accept_examples": [{"id": "OK-1", "before": {"ST-1": "decls/UC-1.json"}, "args": {}}]}]}

D["AGG-2"] = {"kind": "aggregate", "id": "AGG-2", "header": {"name": "TERM-15", "context": "BC-1"},
 "structure": {"state": [{"id": "ST-2", "name": "TERM-11", "type": "VO-1", "multiplicity": one},
                         {"id": "ST-1", "name": "TERM-56", "type": "VO-11", "multiplicity": many}], "entities": []},
 "invariants": [{"id": "INV-1", "condition": {"target": "ST-1", "agg": "count", "op": "ge", "value": 1}, "via": ["CMD-1"]}],
 "commands": [
  {"id": "CMD-1", "name": "TERM-30",
   "args": [{"id": "ARG-1", "name": "TERM-8", "type": "VO-5"}, {"id": "ARG-2", "name": "TERM-60", "type": "VO-8"}, {"id": "ARG-3", "name": "TERM-56", "type": "VO-11"}],
   "business_rules": [
    {"id": "BR-1", "condition": {"target": "ARG-1", "agg": "count", "op": "le", "value": 0}, "reject": "TERM-53",
     "example": {"before": {}, "args": {"ARG-1": {"count": 1}, "ARG-2": {"count": 0}, "ARG-3": {"count": 3}}}},
    {"id": "BR-2", "condition": {"target": "ARG-2", "agg": "count", "op": "le", "value": 0}, "reject": "TERM-55",
     "example": {"before": {}, "args": {"ARG-1": {"count": 0}, "ARG-2": {"count": 2}, "ARG-3": {"count": 3}}}}],
   "state_changes": [{"id": "CHG-2", "condition": {"target": "ST-1", "op": "eq", "value": "ARG-3"}}],
   "emits": [], "accept_examples": [{"id": "OK-1", "before": {}, "args": {"ARG-1": {"count": 0}, "ARG-2": {"count": 0}, "ARG-3": {"count": 3}}}]}]}

D["DS-1"] = {"kind": "domain_service", "id": "DS-1", "header": {"name": "TERM-32", "context": "BC-1", "reason": "複数の集約にまたがる計算"},
 "reads": ["AGG-1", "AGG-2"],
 "operations": [
  {"id": "OP-1", "name": "TERM-57",
   "inputs": [{"id": "IN-1", "from": {"target": "AGG-1.ST-4"}, "type": "VO-3"}, {"id": "IN-2", "from": {"target": "AGG-1.ST-1"}, "type": "VO-1"},
              {"id": "IN-3", "from": {"target": "AGG-1.ST-2"}, "type": "VO-2"}],
   "output": "VO-9", "results": [
    {"id": "RES-3", "condition": {"if": {"target": "AGG-1.ST-4", "op": "ne", "value": "AGG-1.ST-1"}, "target": "RESULT", "op": "eq", "value": "ずれ"}},
    {"id": "RES-4", "condition": {"if": {"target": "AGG-1.ST-4", "op": "ne", "value": "AGG-1.ST-2"}, "target": "RESULT", "op": "eq", "value": "ずれ"}}]},
  {"id": "OP-3", "name": "TERM-59",
   "inputs": [{"id": "IN-1", "from": {"target": "AGG-1.ST-3"}, "type": "VO-6"}, {"id": "IN-3", "from": {"target": "AGG-1.ST-1"}, "type": "VO-1"},
              {"id": "IN-2", "from": {"target": "AGG-2.ST-1"}, "type": "VO-11"}],
   "output": "VO-9", "results": [
    {"id": "RES-2", "condition": {"if": {"target": "AGG-2.ST-1", "op": "ne", "value": "AGG-1.ST-3"}, "target": "RESULT", "op": "eq", "value": "確かめ直し"}},
    {"id": "RES-3", "condition": {"if": {"target": "AGG-2.ST-1", "op": "eq", "value": "AGG-1.ST-3"}, "target": "RESULT", "op": "eq", "value": "合格"}}]}]}

def VO(i, name, comps, ops=()):
    return {"kind": "value_object", "id": f"VO-{i}", "header": {"name": name, "context": "BC-1"},
            "components": [dict(c, id=f"CMP-{n+1}", invariants=c.get("invariants", [])) for n, c in enumerate(comps)], "operations": list(ops)}
NE = lambda: [{"id": "INV-1", "condition": {"target": "CMP-1", "measure": "length", "op": "ge", "value": 1}}]
D["VO-1"] = VO(1, "TERM-7", [{"name": "ファイルのパス", "kind": "文字列", "invariants": NE()}])
D["VO-2"] = VO(2, "TERM-2", [{"name": "スキーマのファイルのパス", "kind": "文字列", "invariants": NE()}])
D["VO-3"] = VO(3, "TERM-12", [{"name": "指す先", "kind": "文字列", "invariants": NE()}, {"name": "x-ref の値", "kind": "文字列"}, {"name": "指す先の種類", "kind": "文字列"}])
D["VO-4"] = VO(4, "TERM-13", [{"name": "導いた値", "kind": "文字列"}, {"name": "宣言した値", "kind": "文字列"}],
 [{"id": "OP-1", "name": "TERM-58", "args": [], "result": "VO-9",
   "accept_examples": [{"id": "OK-1", "self": {"導いた値": "中核", "宣言した値": "中核"}, "args": [], "result": "合格"},
                       {"id": "OK-2", "self": {"導いた値": "中核", "宣言した値": "補完"}, "args": [], "result": "ずれ"}]}])
D["VO-5"] = VO(5, "TERM-8", [{"name": "プロパティ", "kind": "文字列"}, {"name": "理由", "kind": "文字列", "invariants": NE()}])
D["VO-6"] = VO(6, "TERM-16", [{"name": "16進の文字列", "kind": "文字列", "invariants": [{"id": "INV-1", "condition": {"target": "CMP-1", "measure": "length", "op": "eq", "value": 64}}]}])
D["VO-7"] = VO(7, "TERM-6", [{"name": "操作の並び", "kind": "文字列", "invariants": NE()}],
 [{"id": "OP-1", "name": "TERM-63", "args": ["VO-10"], "result": "VO-10",
   "accept_examples": [{"id": "OK-1", "self": PATCH, "args": ["{}"], "result": '{"a":1}'}]}])
D["VO-8"] = VO(8, "TERM-60", [{"name": "検査の名前", "kind": "文字列"}, {"name": "指す先", "kind": "文字列"}])
D["VO-9"] = VO(9, "TERM-14", [{"name": "検査の名前", "kind": "文字列"}, {"name": "状態", "kind": "列挙", "values": ["合格", "ずれ", "確かめ直し"]}])
D["VO-10"] = VO(10, "TERM-61", [{"name": "JSON の値", "kind": "文字列"}],
 [{"id": "OP-1", "name": "TERM-62", "args": [], "result": "VO-6",
   "accept_examples": [{"id": "OK-1", "self": "{}", "args": [], "result": H0}]}])
D["VO-11"] = VO(11, "TERM-56", [{"name": "パス", "kind": "文字列", "invariants": NE()}, {"name": "ハッシュ値", "kind": "文字列"}])

os.makedirs(os.path.join(H, "decls"), exist_ok=True)
for f in os.listdir(os.path.join(H, "decls")): os.remove(os.path.join(H, "decls", f))
for k, v in D.items():
    json.dump(v, open(os.path.join(H, "decls", f"{k}.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print(len(D), "件")
