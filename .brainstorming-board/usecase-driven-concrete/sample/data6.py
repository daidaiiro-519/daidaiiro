# 来店前注文の宣言（論点7：要求の側と設計の側に分けた形）。
# 要求の側（ドメイン ・ 用語集 ・ その他の要求 ・ ユースケース）は、プロダクトの内部を指さない。
# 設計の側（サブドメイン ・ 区切られた文脈 ・ アプリケーション層の操作 ・ 集約など）が、要求の側を指す。
# 意味はすべて欄に持ち、文は道具が組む。集約 ・ 値オブジェクト ・ ドメインサービスは data5 と同じ。
import copy, data5
D={}
C=data5.C
# ── 用語集（要求の側に1つ）。語ごとに意味の一覧を持つ。kind は、その意味がどう使われるか
GL_TERMS=[]
for t in data5.D['BC-1']['ubiquitous_language']['terms']:
  m={"id":"M-1","definition":t['definition'],"kind":t['kind']}
  for k in ('form','change'):
    if k in t: m[k]=t[k]
  GL_TERMS.append({"id":t['id'],"word":t['word'],"meanings":[m],"avoid":t['avoid']})
# 同じ語の別の意味（このプロダクトの文脈は使わない）。用語集の中で意味を分けて書く
GL_TERMS[0]['meanings'].append({"id":"M-2","definition":"店舗が仕入先に出す発注","kind":"情報の別名"})
D['GLO-1']={"kind":"glossary","id":"GLO-1","header":{"name":"用語集"},"terms":GL_TERMS}

# ── ドメイン（作りたいプロダクト）。ビジョン記述にまとめる。サブドメインの一覧と品質の要求は持たない
d5=data5.D['DOM-1']
D['DOM-1']={"kind":"domain","id":"DOM-1",
 "header":{"name":d5['header']['name']},
 "vision":{"problem":d5['header']['problem'],
           "values":d5['value_proposition']['values'],
           "competitors":d5['value_proposition']['competitors'],
           "success_criteria":[
             {"id":"SC-1","name":"約束どおりに渡せる","source":"event","target":"TERM-31",
              "measure":{"diff":["TERM-31","TERM-1.TERM-4"]},"threshold":{"op":"le","value":5,"unit":"分"},
              "ratio":{"op":"ge","value":95,"unit":"%"},"window":"1日"},
             d5['success_criteria'][1]]},
 "scope":d5['scope'],
 "design_scopes":[
  {"id":"SCP-1","level":"企業","name":"店（飲食チェーン）","inside":["SH-2","SH-3","SCP-2"],"outside":["SH-1","決済代行"]},
  {"id":"SCP-2","level":"システム","name":"受け取り時刻を約束するモバイルオーダー","outside":["SH-1","SH-2","決済代行の決済サービス","メニューのサービス","顧客のサービス"]}],
 "stakeholders":d5['stakeholders']}

# ── その他の要求（要求アウトラインの章）
D['REQ-1']={"kind":"other_requirements","id":"REQ-1","header":{"name":"その他の要求"},
 "business_rules":[{"id":"BR-1","name":"提示した価格で売る","condition":C(target="TERM-1.TERM-2.TERM-39",op="eq",value="TERM-40"),"protects":["DOM-1.SH-1"]}],
 "quality":[{"id":"QR-1","target":"UC-1.STEP-5","measure":"応答時間",
   "threshold":{"op":"le","value":2,"unit":"秒"},"ratio":{"op":"ge","value":90,"unit":"%"},
   "condition":{"period":"ピーク時","load":{"value":600,"unit":"件/時"}},
   "grade":{"item":"B.2.1","level":3},"method":"非機能テスト"}],
 "technology":[{"id":"TEC-1","system":"決済代行の決済サービス","text":"請求額の承認を依頼し、承認番号を受け取る。同じ依頼を2回送っても二重に請求されない"}],
 "data":[],
 "open_issues":["顧客の識別（顧客のサービスへのログイン）を、どのユースケースが成り立たせるか"]}

# ── ユースケース（内部を指さない）。手順は用語集の語で書き、集約 ・ 外部の操作の ID を持たない
S=lambda i,kind,actor,**k:{"id":i,"kind":kind,"actor":actor,**k,"extensions":k.get("extensions",[])}
CH=lambda i,verb,obj,**k:S(i,"内部の状態変化","システム",verb=verb,object=obj,**k)
D['UC-0']={"kind":"use_case","id":"UC-0",
 "header":{"name":"来店前に注文して受け取る","level":"要約","scope":{"system":"DOM-1.SCP-1"},"primary_actor":"顧客","trigger_step":"STEP-1"},
 "preconditions":[],
 "stakeholders":[{"id":"SH-1","who":"DOM-1.SH-1","interest":"待たずに、約束した時刻に受け取る"},{"id":"SH-2","who":"DOM-1.SH-2","interest":"調理の空きを超えずに注文を受け、支払い済みの注文だけを渡す"}],
 "guarantees":{"minimal":[{"id":"MG-1","name":"承認の無い注文は渡さない","condition":C(if_=C(target="TERM-1.TERM-5",op="eq",value="TERM-42"),target="TERM-1.TERM-21",op="not_empty"),"protects":["SH-2"]}],
               "success":[{"id":"SG-1","name":"受け取り","condition":C(target="TERM-1.TERM-5",op="eq",value="TERM-42"),"satisfies":["SH-1","SH-2"]}]},
 "scenario":{"supporting_actors":[],"steps":[
  S("STEP-1","サブユースケースの呼び出し","顧客",calls="UC-1",serves=["SH-1"]),
  S("STEP-2","相互作用","顧客",to="店舗",data=["TERM-14"],verb="TERM-49",serves=["SH-1"]),
  S("STEP-3","サブユースケースの呼び出し","店舗",calls="UC-3",keeps=["MG-1"],serves=["SH-1","SH-2"])]},
 "contributes_to":["SC-1","SC-2"],"open_issues":[],"links":{}}
D['UC-1']={"kind":"use_case","id":"UC-1",
 "header":{"name":"注文を確定する","level":"ユーザー目的","scope":{"system":"DOM-1.SCP-2"},"primary_actor":"顧客","trigger_step":"STEP-1"},
 "preconditions":[{"id":"PRE-2","condition":C(target="TERM-1.TERM-2",agg="count",op="ge",value=1),"established_by":"UC-2"},
                  {"id":"PRE-3","condition":C(target="TERM-1.TERM-5",op="eq",value="TERM-10"),"established_by":"UC-2"}],
 "stakeholders":data5.D['UC-1']['stakeholders'],
 "guarantees":{"minimal":[{"id":"MG-1","name":"承認なしで確定しない","condition":C(if_=C(target="TERM-1.TERM-5",op="eq",value="TERM-11"),target="TERM-1.TERM-21",op="not_empty"),"protects":["SH-2"]},
                          {"id":"MG-2","name":"失敗したら調理枠を残さない","condition":C(if_=C(target="TERM-1.TERM-5",op="eq",value="TERM-10"),target="TERM-1.TERM-38",op="empty"),"protects":["SH-1","SH-2"]},
                          {"id":"MG-3","name":"どこまで進んだかを残す","condition":C(target="TERM-1.TERM-22",agg="count",op="ge",value=1),"protects":["SH-2","SH-3"]}],
               "success":[{"id":"SG-1","name":"確定","condition":C(target="TERM-1.TERM-5",op="eq",value="TERM-11"),"satisfies":["SH-1"]},
                          {"id":"SG-2","name":"受け取り時刻と調理枠の確保","condition":C(target="TERM-1.TERM-4",op="not_empty"),"satisfies":["SH-1","SH-2"]},
                          {"id":"SG-3","name":"承認番号の記録","condition":C(target="TERM-1.TERM-21",op="not_empty"),"satisfies":["SH-3"]}]},
 "scenario":{"supporting_actors":["決済代行"],"steps":[
  S("STEP-1","相互作用","顧客",to="システム",data=["TERM-33"],verb="TERM-48",serves=["SH-1"],
    variations=[{"varies":"支払い手段","values":["クレジットカード","電子マネー"]}]),
  S("STEP-2","妥当性確認","システム",checks=["REQ-1.BR-1"],serves=["SH-1","SH-2"],extensions=[
    {"id":"EXT-1","condition_kind":"妥当性確認の失敗","fails":["REQ-1.BR-1"],"ending":"STEP-2","steps":[
      S("EXT-1.S-1","相互作用","システム",to="顧客",data=["TERM-35"],verb="TERM-49"),
      S("EXT-1.S-2","相互作用","顧客",to="システム",data=["TERM-35"],verb="TERM-54")]}]),
  CH("STEP-7","TERM-43","TERM-4",serves=["SH-1"]),
  CH("STEP-3","TERM-15","TERM-9",serves=["SH-1","SH-2"],extensions=[
    {"id":"EXT-2","condition_kind":"業務ルールの拒否","reasons":["TERM-19"],"ending":"STEP-3","steps":[
      S("EXT-2.S-1","相互作用","システム",to="顧客",data=["TERM-45"],verb="TERM-49"),
      S("EXT-2.S-2","相互作用","顧客",to="システム",data=["TERM-45"],verb="TERM-54")]}]),
  S("STEP-4","相互作用","システム",to="決済代行",data=["TERM-34"],verb="TERM-48",reply=["TERM-21"],keeps=["MG-1"],serves=["SH-2"],extensions=[
    {"id":"EXT-3","condition_kind":"支援アクターの失敗","actor":"決済代行","reasons":["TERM-55"],"ending":"失敗","steps":[
      CH("EXT-3.S-1","TERM-16","TERM-9",keeps=["MG-2"]),
      CH("EXT-3.S-2","TERM-23","TERM-1",keeps=["MG-3"]),
      S("EXT-3.S-3","相互作用","システム",to="顧客",data=["TERM-47"],verb="TERM-50")]},
    {"id":"EXT-4","condition_kind":"支援アクターの失敗","actor":"決済代行","reasons":["TERM-56","TERM-57"],"ending":"失敗","steps":[
      CH("EXT-4.S-1","TERM-16","TERM-9",keeps=["MG-2"]),
      CH("EXT-4.S-2","TERM-23","TERM-1",keeps=["MG-3"]),
      S("EXT-4.S-3","相互作用","システム",to="顧客",data=["TERM-58"],verb="TERM-50")]}]),
  CH("STEP-5","TERM-6","TERM-1",serves=["SH-1","SH-3"]),
  S("STEP-6","相互作用","システム",to="顧客",data=["TERM-14","TERM-4"],verb="TERM-50",serves=["SH-1"])]},
 "contributes_to":["SC-1","SC-2"],
 "open_issues":["受け取り予定時刻を過ぎた注文の扱い"],
 "links":{"business_rules":["REQ-1.BR-1"],"quality":["REQ-1.QR-1"],"technology":["REQ-1.TEC-1"]}}
D['UC-2']={"kind":"use_case","id":"UC-2",
 "header":{"name":"注文に商品を入れる","level":"ユーザー目的","scope":{"system":"DOM-1.SCP-2"},"primary_actor":"顧客","trigger_step":"STEP-1"},
 "preconditions":[{"id":"PRE-2","condition":C(target="TERM-1.TERM-5",op="eq",value="TERM-10"),"established_by":None}],
 "stakeholders":data5.D['UC-2']['stakeholders'],
 "guarantees":{"minimal":[{"id":"MG-1","name":"明細は20件まで","condition":C(target="TERM-1.TERM-2",agg="count",op="le",value=20),"protects":["SH-2"]}],
               "success":[{"id":"SG-1","name":"明細が増える","condition":C(target="TERM-1.TERM-2",agg="count",op="gt",value={"before":"TERM-1.TERM-2","agg":"count"}),"satisfies":["SH-1"]}]},
 "scenario":{"supporting_actors":[],"steps":[
  S("STEP-1","相互作用","顧客",to="システム",data=["TERM-65"],verb="TERM-48",serves=["SH-1"]),
  CH("STEP-2","TERM-62","TERM-1",keeps=["MG-1"],serves=["SH-1","SH-2"],extensions=[
    {"id":"EXT-1","condition_kind":"業務ルールの拒否","reasons":["TERM-63"],"ending":"失敗","steps":[
      S("EXT-1.S-1","相互作用","システム",to="顧客",data=["TERM-67"],verb="TERM-50")]}]),
  S("STEP-3","相互作用","システム",to="顧客",data=["TERM-66"],verb="TERM-49",serves=["SH-1"])]},
 "contributes_to":["SC-2"],"open_issues":[],"links":{}}
D['UC-3']={"kind":"use_case","id":"UC-3",
 "header":{"name":"注文を渡す","level":"ユーザー目的","scope":{"system":"DOM-1.SCP-2"},"primary_actor":"店舗","trigger_step":"STEP-1"},
 "preconditions":[],
 "stakeholders":data5.D['UC-3']['stakeholders'],
 "guarantees":{"minimal":[{"id":"MG-1","name":"承認の無い注文は渡さない","condition":C(if_=C(target="TERM-1.TERM-5",op="eq",value="TERM-42"),target="TERM-1.TERM-21",op="not_empty"),"protects":["SH-2"]}],
               "success":[{"id":"SG-1","name":"受け渡し済","condition":C(target="TERM-1.TERM-5",op="eq",value="TERM-42"),"satisfies":["SH-1","SH-2"]}]},
 "scenario":{"supporting_actors":[],"steps":[
  S("STEP-1","相互作用","店舗",to="システム",data=["TERM-59"],verb="TERM-48",serves=["SH-2"]),
  CH("STEP-2","TERM-30","TERM-1",keeps=["MG-1"],serves=["SH-1","SH-2"],extensions=[
    {"id":"EXT-1","condition_kind":"業務ルールの拒否","reasons":["TERM-41"],"ending":"失敗","steps":[
      S("EXT-1.S-1","相互作用","システム",to="店舗",data=["TERM-60"],verb="TERM-50")]}]),
  S("STEP-3","相互作用","システム",to="店舗",data=["TERM-61"],verb="TERM-50",serves=["SH-2"])]},
 "contributes_to":["SC-1"],"open_issues":["受け取り予定時刻を過ぎて来た顧客への受け渡し","注文番号の注文が見つからないとき"],"links":{}}

# ── 設計の側。サブドメインは束ねるユースケースを持つ
for k in ('SD-1','SD-2'):
  D[k]=copy.deepcopy(data5.D[k])
D['SD-1']['use_cases']=["UC-1","UC-2","UC-3"]
D['SD-2']['use_cases']=[]
# 区切られた文脈は、用語集のどの語のどの意味を使うかを指す。業務ルールは要求の側のルールを実装する
b5=data5.D['BC-1']
D['BC-1']={"kind":"context","id":"BC-1","header":b5['header'],"context_map":b5['context_map'],"boundary":b5['boundary'],
 "uses":[{"term":t['id'],"meaning":"M-1"} for t in GL_TERMS],
 "business_rules":[{"id":"BR-1","condition":C(target="AGG-1.ENT-1.ES-3",op="eq",value="TERM-40"),"implements":"REQ-1.BR-1"}],
 "published_language":[]}
for k in ('AGG-1','AGG-2','VO-1','VO-2','VO-3','VO-4','VO-5','VO-6','VO-7','VO-9','DS-1'):
  D[k]=copy.deepcopy(data5.D[k])

ORDER_IDS=['DOM-1','GLO-1','REQ-1','UC-0','UC-1','UC-2','UC-3','SD-1','SD-2','BC-1','AGG-1','AGG-2','VO-1','VO-2','VO-3','VO-4','VO-5','VO-6','VO-7','VO-9','DS-1']
D={k:D[k] for k in ORDER_IDS}
IMPL=data5.IMPL
RETIRED=data5.RETIRED
