# fact-check の「原文を取得する」を、論点7の宣言の形へ移す（論点6 A′ の試行）。
# 材料：.claude/skills/fact-check の SKILL.md ・ source.rs ・ http.rs ・ service/lib.rs ・ tests/source.rs（読むだけ）
import json,os
H=os.path.dirname(os.path.abspath(__file__))
def T(i,w,kind,d,avoid=(),**kw):
  m={"id":"M-1","definition":d,"kind":kind}; m.update(kw)
  return {"id":f"TERM-{i}","word":w,"meanings":[m],"avoid":list(avoid)}
TERMS=[
 T(1,"取得の記録","集約","1回の取得で受け取った原文の、保存した場所 ・ 出どころ ・ 取得した日時 ・ 中身のハッシュ値 ・ 大きさ"),
 T(2,"出どころ","値オブジェクト","原文を取りに行く先の URL"),
 T(3,"頼まれた出どころ","状態","書き手が渡した出どころ"),
 T(4,"取得した日時","値オブジェクト","原文を受け取った日時。利用者の地域の時刻で書く"),
 T(5,"中身のハッシュ値","値オブジェクト","受け取った原文の sha256",avoid=["中身の要約"]),
 T(6,"バイト数","状態","受け取った原文の大きさ"),
 T(7,"行数","状態","受け取った原文の行の数"),
 T(8,"大きさ","値オブジェクト","0以上の整数で表す量"),
 T(9,"種類","値オブジェクト","受け取った原文の Content-Type"),
 T(11,"保存した場所","値オブジェクト","受け取った原文を書いたファイル。名前は出どころから作る"),
 T(12,"置き場所","情報の別名","原文を保存するフォルダ。呼び出す側が渡す"),
 T(13,"取得の依頼","情報の別名","出どころと置き場所"),
 T(14,"原文","情報の別名","情報源が返した中身。記法を剥がさず、変換しない"),
 T(15,"依頼する","動作","相手に何かを頼む",form="{to}に{data}を依頼する"),
 T(16,"知らせる","動作","相手に結果を伝える",form="{to}に{data}を知らせる"),
 T(17,"記録する","コマンド","受け取った応答から、取得の記録を作る"),
 T(18,"状態コード","値オブジェクト","情報源が返した HTTP の状態コード"),
 T(19,"誤りの頁は原文として受け取らない","拒否の理由","状態コードが200でない応答は、原文にしない"),
 T(20,"空は原文として受け取らない","拒否の理由","中身が0バイトの応答は、原文にしない"),
 T(21,"制限時間を過ぎた","失敗の種類","取得が制限時間を過ぎたので止めた"),
 T(22,"取得の道具を起動できない","失敗の種類","渡された取得の道具（curl）を起動できない"),
 T(23,"試した先","情報の別名","試した出どころごとの、状態コード ・ 大きさ ・ 種類"),
 T(25,"原文を取得する","アプリケーション層の操作","出どころから原文を受け取り、取得の記録と一緒に保存する"),
 T(27,"取得の結果","情報の別名","保存した場所 ・ 記録の場所 ・ 出どころ ・ 大きさ ・ 行数 ・ 中身のハッシュ値 ・ 種類"),
 T(28,"実際に取得した出どころ","状態","原文を返した出どころ。.md を付けた先のときがある"),
 T(29,"出どころが無いこと","情報の別名","取得の依頼に出どころが入っていないこと"),
]
D={}
D['GLO-1']={"kind":"glossary","id":"GLO-1","header":{"name":"用語集"},"terms":TERMS}
D['DOM-1']={"kind":"domain","id":"DOM-1","header":{"name":"fact-check"},
 "vision":{"problem":"識別子と主張は、要約を1回通るだけで書き換わる。書き換わった識別子で参照する実装は空を返し、壊れたことに気づけない",
  "values":[{"id":"VAL-1","name":"原文の文字列で照合できる","text":"外にある情報源の原文を取得して残し、識別子と主張を原文の文字列で照合できる","differentiator":True},
            {"id":"VAL-2","name":"いつ ・ どの版を読んだかを示せる","text":"取得した日時と中身のハッシュ値を、原文と一緒に残す","differentiator":False}],
  "competitors":[],
  "success_criteria":[{"id":"SC-1","name":"取得した原文に記録が揃う","source":"external","measure_text":"取得の記録を持たない原文の件数","threshold":{"op":"le","value":0,"unit":"件"},"window":"取得のたび"}]},
 "scope":{"out":["真偽そのものの判定","原文の要約 ・ 言い換え"]},
 "design_scopes":[{"id":"SCP-1","level":"システム","name":"fact-check","outside":["SH-1","SH-2","情報源のサーバー"]}],
 "stakeholders":[{"id":"SH-1","who":"書き手","interest":"文書へ書く識別子と主張を、原文の文字列で照合したい"},
                 {"id":"SH-2","who":"読み手","interest":"原典の記述として書かれたことが、いつ ・ どの版の原文に在るかを確かめたい"}]}
D['REQ-1']={"kind":"other_requirements","id":"REQ-1","header":{"name":"その他の要求"},
 "business_rules":[{"id":"BR-1","name":"出どころを渡す","condition":{"target":"TERM-13.TERM-2","op":"not_empty"},"protects":["DOM-1.SH-1"]}],
 "quality":[],
 "technology":[{"id":"TEC-1","system":"情報源のサーバー","text":"取得は curl で行い、利用者の環境のプロキシと証明書の設定をそのまま使う。curl のコマンドは tool.json の external から渡す。curl の制限時間は60秒、こちらから止めるのは90秒"}],
 "data":[],
 "open_issues":["原文を変換しない（記法を剥がさない）ことを、条件の欄で書けない"]}
D['UC-1']={"kind":"use_case","id":"UC-1",
 "header":{"name":"原文を取得する","level":"ユーザー目的","scope":{"system":"DOM-1.SCP-1"},"primary_actor":"書き手","trigger_step":"STEP-1"},
 "preconditions":[],
 "stakeholders":[{"id":"SH-1","who":"DOM-1.SH-1","interest":"受け取った原文だけが、照合の相手として残る"},
                 {"id":"SH-2","who":"DOM-1.SH-2","interest":"取得した日時と中身のハッシュ値が、原文と一緒に残る"}],
 "guarantees":{
  "minimal":[{"id":"MG-1","name":"受け取らなかったものを原文として残さない","condition":{"target":"TERM-1.TERM-6","op":"ge","value":1},"protects":["SH-1"]}],
  "success":[{"id":"SG-1","name":"原文の保存","condition":{"target":"TERM-1.TERM-11","op":"not_empty"},"satisfies":["SH-1"]},
             {"id":"SG-2","name":"取得した日時の記録","condition":{"target":"TERM-1.TERM-4","op":"not_empty"},"satisfies":["SH-2"]},
             {"id":"SG-3","name":"中身のハッシュ値の記録","condition":{"target":"TERM-1.TERM-5","op":"not_empty"},"satisfies":["SH-2"]}]},
 "scenario":{"supporting_actors":["情報源のサーバー"],"steps":[
  {"id":"STEP-1","kind":"相互作用","actor":"書き手","to":"システム","data":["TERM-13"],"verb":"TERM-15","serves":["SH-1"],"extensions":[]},
  {"id":"STEP-2","kind":"妥当性確認","actor":"システム","checks":["REQ-1.BR-1"],"serves":["SH-1"],"extensions":[
    {"id":"EXT-1","condition_kind":"妥当性確認の失敗","fails":["REQ-1.BR-1"],"ending":"失敗","steps":[
      {"id":"EXT-1.S-1","kind":"相互作用","actor":"システム","to":"書き手","data":["TERM-29"],"verb":"TERM-16","extensions":[]}]}]},
  {"id":"STEP-3","kind":"相互作用","actor":"システム","to":"情報源のサーバー","data":["TERM-2"],"verb":"TERM-15","reply":["TERM-14"],"serves":["SH-1"],
   "variations":[{"varies":"依頼する先","values":["出どころに .md を付けた先（先に試す）","出どころそのもの"]}],
   "extensions":[{"id":"EXT-2","condition_kind":"支援アクターの失敗","actor":"情報源のサーバー","reasons":["TERM-21","TERM-22"],"ending":"失敗","steps":[
      {"id":"EXT-2.S-1","kind":"相互作用","actor":"システム","to":"書き手","data":["TERM-23"],"verb":"TERM-16","extensions":[]}]}]},
  {"id":"STEP-4","kind":"内部の状態変化","actor":"システム","verb":"TERM-17","object":"TERM-1","keeps":["MG-1"],"serves":["SH-1","SH-2"],"extensions":[
    {"id":"EXT-3","condition_kind":"業務ルールの拒否","reasons":["TERM-19","TERM-20"],"ending":"失敗","steps":[
      {"id":"EXT-3.S-1","kind":"相互作用","actor":"システム","to":"書き手","data":["TERM-23"],"verb":"TERM-16","extensions":[]}]}]},
  {"id":"STEP-5","kind":"相互作用","actor":"システム","to":"書き手","data":["TERM-27"],"verb":"TERM-16","serves":["SH-1","SH-2"],"extensions":[]}]},
 "contributes_to":["SC-1"],
 "open_issues":["出どころに .md を付けた先が受け取れなかったとき、出どころそのものを試す。この「順に試す」を手順と拡張で書く形が無く、変化（variations）で持った"],
 "links":{"business_rules":["REQ-1.BR-1"],"quality":[],"technology":["REQ-1.TEC-1"]}}
D['SD-1']={"kind":"subdomain","id":"SD-1","header":{"name":"原文の取得","description":"外にある情報源の原文を受け取り、取得の記録と一緒に残す"},
 "classification":{"category":"補完","competitive_advantage":False,"external_available":False,"cheaper_to_build":True,"sourcing":"自分たちで作る"},
 "business_logic":{"needs_tracking":False,"complex_rules":False,"complex_data":False,"rule_conditions":[]},
 "serves_values":["VAL-2"],"use_cases":["UC-1"]}
U=["TERM-1","TERM-2","TERM-3","TERM-4","TERM-5","TERM-6","TERM-7","TERM-8","TERM-9","TERM-11","TERM-12","TERM-13","TERM-14","TERM-15","TERM-16","TERM-17","TERM-18","TERM-19","TERM-20","TERM-21","TERM-22","TERM-23","TERM-25","TERM-27","TERM-28","TERM-29"]
D['BC-1']={"kind":"context","id":"BC-1","header":{"name":"原文の取得","purpose":"情報源の応答を原文として受け取るかを決め、取得の記録を作る。通信は curl に任せ、判定を置かない","subdomains":["SD-1"]},
 "context_map":{"relations":[{"id":"X-1","external":"情報源のサーバー","owner":"情報源の運営者","pattern":"従属","direction":"下流",
   "operations":[{"id":"OP-1","name":"TERM-15","sends":["TERM-2"],"receives":["TERM-14","TERM-18","TERM-9"],"retry":{"safe":True,"key":"TERM-2"},
     "failures":[{"id":"F-1","name":"TERM-21"},{"id":"F-2","name":"TERM-22"}]}]}]},
 "boundary":{"kind":"Skill の道具","owner":"fact-check"},
 "uses":[{"term":t,"meaning":"M-1"} for t in U],
 "business_rules":[{"id":"BR-1","condition":{"target":"AGG-1.ST-3","op":"not_empty"},"implements":"REQ-1.BR-1"}],
 "published_language":[]}
D['APP-1']={"kind":"application_operation","id":"APP-1","header":{"name":"TERM-25","context":"BC-1"},"satisfies":["UC-1"],
 "steps":[{"step":"UC-1.STEP-3","calls":"BC-1.X-1.OP-1"},{"step":"UC-1.STEP-4","calls":"AGG-1.CMD-1"}],
 "extensions":[{"extension":"UC-1.EXT-2","raised_by":["BC-1.X-1.OP-1.F-1","BC-1.X-1.OP-1.F-2"]},
               {"extension":"UC-1.EXT-3","raised_by":["AGG-1.CMD-1.BR-1","AGG-1.CMD-1.BR-2"]}],
 "preconditions":[],
 "guarantees":[{"guarantee":"UC-1.SG-1","established_by":["AGG-1.CMD-1.CHG-1"]},
               {"guarantee":"UC-1.SG-2","established_by":["AGG-1.CMD-1.CHG-3"]},
               {"guarantee":"UC-1.SG-3","established_by":["AGG-1.CMD-1.CHG-4"]}]}
one={"min":1,"max":1}
D['AGG-1']={"kind":"aggregate","id":"AGG-1","header":{"name":"TERM-1","context":"BC-1"},
 "structure":{"state":[
  {"id":"ST-1","name":"TERM-11","type":"VO-5","multiplicity":one},
  {"id":"ST-2","name":"TERM-28","type":"VO-1","multiplicity":one},
  {"id":"ST-3","name":"TERM-3","type":"VO-1","multiplicity":one},
  {"id":"ST-4","name":"TERM-4","type":"VO-2","multiplicity":one},
  {"id":"ST-5","name":"TERM-5","type":"VO-3","multiplicity":one},
  {"id":"ST-6","name":"TERM-6","type":"VO-4","multiplicity":one},
  {"id":"ST-7","name":"TERM-7","type":"VO-4","multiplicity":one},
  {"id":"ST-8","name":"TERM-9","type":"VO-6","multiplicity":one}],"entities":[]},
 "invariants":[{"id":"INV-1","condition":{"target":"ST-6","op":"ge","value":1},"via":["CMD-1"]}],
 "commands":[{"id":"CMD-1","name":"TERM-17",
   "args":[{"id":"ARG-1","name":"TERM-18","type":"VO-7"},{"id":"ARG-2","name":"TERM-6","type":"VO-4"},
           {"id":"ARG-3","name":"TERM-28","type":"VO-1"},{"id":"ARG-4","name":"TERM-5","type":"VO-3"},
           {"id":"ARG-5","name":"TERM-3","type":"VO-1"},{"id":"ARG-6","name":"TERM-9","type":"VO-6"}],
   "business_rules":[
    {"id":"BR-1","condition":{"target":"ARG-1","op":"eq","value":200},"reject":"TERM-19","example":{"before":{},"args":{"ARG-1":404,"ARG-2":900}}},
    {"id":"BR-2","condition":{"target":"ARG-2","op":"ge","value":1},"reject":"TERM-20","example":{"before":{},"args":{"ARG-1":200,"ARG-2":0}}}],
   "state_changes":[
    {"id":"CHG-1","condition":{"target":"ST-1","op":"not_empty"}},
    {"id":"CHG-2","condition":{"target":"ST-2","op":"eq","value":"ARG-3"}},
    {"id":"CHG-3","condition":{"target":"ST-4","op":"not_empty"}},
    {"id":"CHG-4","condition":{"target":"ST-5","op":"eq","value":"ARG-4"}},
    {"id":"CHG-5","condition":{"target":"ST-6","op":"eq","value":"ARG-2"}},
    {"id":"CHG-6","condition":{"target":"ST-3","op":"eq","value":"ARG-5"}},
    {"id":"CHG-7","condition":{"target":"ST-7","op":"ge","value":1}},
    {"id":"CHG-8","condition":{"target":"ST-8","op":"eq","value":"ARG-6"}}],
   "emits":[],
   "accept_examples":[{"id":"OK-1","before":{},"args":{"ARG-1":200,"ARG-2":1}}]}]}
def VO(i,name,comp,inv=()):
  return {"kind":"value_object","id":f"VO-{i}","header":{"name":name,"context":"BC-1"},"components":[dict(comp,id="CMP-1",invariants=list(inv))],"operations":[]}
D['VO-1']=VO(1,"TERM-2",{"name":"URL","kind":"文字列"})
D['VO-2']=VO(2,"TERM-4",{"name":"日時","kind":"日時","precision":"秒"})
D['VO-3']=VO(3,"TERM-5",{"name":"16進の文字列","kind":"文字列"},[{"id":"INV-1","condition":{"target":"CMP-1","op":"eq","value":64},"note":"文字数"}])
D['VO-4']=VO(4,"TERM-8",{"name":"量","kind":"数","precision":"整数"},[{"id":"INV-1","condition":{"target":"CMP-1","op":"ge","value":0}}])
D['VO-5']=VO(5,"TERM-11",{"name":"ファイルの名前","kind":"文字列"},[{"id":"INV-1","condition":{"target":"CMP-1","op":"le","value":120},"note":"文字数"}])
D['VO-6']=VO(6,"TERM-9",{"name":"種類","kind":"文字列"})
D['VO-7']=VO(7,"TERM-18",{"name":"コード","kind":"数","precision":"整数"})
for k,v in D.items():
  json.dump(v,open(os.path.join(H,'decls',f'{k}.json'),'w',encoding='utf-8'),ensure_ascii=False,indent=1)
print(len(D),'件')
