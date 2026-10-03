# 意味の塊ごとにオブジェクトを組んだ宣言（来店前注文）。ID の形は論点4で決まるので仮である。
D={}
D['DOM-1']={"kind":"domain","id":"DOM-1",
 "header":{"name":"来店前注文","problem":"昼の混雑時、店頭で注文と会計の列に並ぶ客が多く、受け取りまで15分以上かかって帰ってしまう客がいる"},
 "value_proposition":{
  "values":[{"id":"VAL-1","name":"受け取り時刻の約束","text":"来店前に注文を確定でき、調理の空きに合わせた受け取り予定時刻を約束する","differentiator":True},
            {"id":"VAL-2","name":"現金を扱わない支払い","text":"店頭で現金を扱わずに支払える","differentiator":False}],
  "competitors":[{"name":"他チェーンのモバイルオーダー","difference":"受け取り時刻を約束せず、できあがりを通知するだけ"}]},
 "success_criteria":[{"id":"SC-1","name":"約束どおりに渡せる","text":"確定した注文の 95% 以上を、受け取り予定時刻から5分以内に渡している"},
                     {"id":"SC-2","name":"列が短い","text":"昼の時間帯の店頭の待ち列が、5人以下に収まっている"}],
 "scope":{"in":["来店前の注文と支払い","受け取り予定時刻の約束","店頭での受け取り"],"out":["配達","ポイント","メニューの管理"]},
 "subdomains":["SD-1","SD-2"],
 "stakeholders":[{"id":"SH-1","who":"顧客","interest":"待たずに、約束した時刻に受け取りたい"},
                 {"id":"SH-2","who":"店舗","interest":"調理の空きを超えて注文を受けたくない"},
                 {"id":"SH-3","who":"経理","interest":"注文と入金を後から突き合わせたい"}],
 "quality":[{"id":"QR-1","name":"確定の応答","text":"注文の確定は、2秒以内に結果を返す"}]}
D['SD-1']={"kind":"subdomain","id":"SD-1",
 "header":{"name":"受注","description":"来店前の注文を受け付け、調理の空きに合わせて受け取り予定時刻を約束する"},
 "classification":{"category":"中核","competitive_advantage":True,"answer":"受け取り時刻を約束できることが、競合店との違いになる"},
 "business_logic":{"needs_tracking":False,"complex_rules":True,"complex_data":False,"rule_conditions":["AGG-1.INV-1","AGG-1.INV-2","DS-1.OP-1.POST-2"]},
 "serves_values":["VAL-1"]}
D['SD-2']={"kind":"subdomain","id":"SD-2",
 "header":{"name":"決済","description":"注文の代金を受け取る"},
 "classification":{"category":"一般","competitive_advantage":False,"answer":"どの店でも同じやり方で、外部のサービスで足りる","sourcing":"外部サービス"},
 "business_logic":{"needs_tracking":True,"complex_rules":False,"complex_data":False,"rule_conditions":[]},
 "serves_values":[]}
T=lambda i,w,k,d,a=[]:{"id":i,"word":w,"kind":k,"definition":d,"avoid":a}
D['BC-1']={"kind":"context","id":"BC-1",
 "header":{"name":"受注","purpose":"受け取り予定時刻を約束した注文を、調理の空きを超えずに受け付ける","subdomains":["SD-1"]},
 "context_map":{"relations":[{"with":"BC-2","pattern":"モデル変換装置","reason":"外部の決済サービスのモデルを、受注の言葉に持ち込まない"}]},
 "ubiquitous_language":{"terms":[
  T("TERM-1","注文","集約","1人の顧客が1つの店舗で受け取る商品の一覧と、その受け取り予定時刻",["オーダー"]),
  T("TERM-9","調理枠","集約","店舗が一定の時間に調理できる量"),
  T("TERM-2","明細","エンティティ","注文の中の、1つの商品とその数量",["行"]),
  T("TERM-3","数量","値オブジェクト","1つの明細で注文する商品の個数"),
  T("TERM-4","受け取り予定時刻","値オブジェクト","店舗が顧客に渡すと約束した時刻",["受け取り時間","ピックアップ時刻"]),
  T("TERM-5","注文の状態","値オブジェクト","注文が下書きか確定済か",["ステータス"]),
  T("TERM-14","注文番号","値オブジェクト","注文を店舗と顧客のあいだで指す番号",["オーダー ID"]),
  T("TERM-6","確定する","コマンド","下書きの注文を、受け取り予定時刻を決めて確定済にする"),
  T("TERM-7","注文確定済","業務イベント","注文が確定済になったこと"),
  T("TERM-8","受け取り時刻を見積もる","ドメインサービス","明細と調理の空きから、受け取り予定時刻を出す"),
  T("TERM-10","下書き","状態の値","まだ確定していない注文の状態"),
  T("TERM-11","確定済","状態の値","受け取り予定時刻を約束した注文の状態"),
  T("TERM-12","確定済の注文は確定できない","拒否の理由","同じ注文を2回確定しようとした"),
  T("TERM-13","明細の無い注文は確定できない","拒否の理由","明細が1件も無い注文を確定しようとした")]},
 "published_language":[]}
D['BC-2']={"kind":"context","id":"BC-2","header":{"name":"決済","purpose":"注文の代金を外部の決済サービスで受け取る","subdomains":["SD-2"]},
 "context_map":{"relations":[]},"ubiquitous_language":{"terms":[]},"published_language":[]}
C=lambda **k:k
D['AGG-1']={"kind":"aggregate","id":"AGG-1",
 "header":{"name":"TERM-1","context":"BC-1","description":"受け取り予定時刻の約束と、約束した明細の一貫性を守る"},
 "structure":{"state":[{"id":"ST-1","name":"TERM-14","type":"VO-2","multiplicity":"1"},
                       {"id":"ST-2","name":"TERM-5","type":"VO-3","multiplicity":"1"},
                       {"id":"ST-3","name":"TERM-2","type":"ENT-1","multiplicity":"0..20"},
                       {"id":"ST-4","name":"TERM-4","type":"VO-4","multiplicity":"0..1"}],
              "entities":[{"id":"ENT-1","name":"TERM-2","state":[{"name":"商品","type":"商品の ID"},{"name":"数量","type":"VO-1"}]}]},
 "invariants":[
  {"id":"INV-1","condition":C(**{"if":{"target":"ST-2","op":"eq","value":"TERM-11"}},target="ST-4",op="not_empty"),"violation":"確定済なのに、受け取り予定時刻が空"},
  {"id":"INV-2","condition":C(**{"if":{"target":"ST-2","op":"eq","value":"TERM-11"}},target="ST-3",agg="count",op="ge",value=1),"violation":"確定済なのに、明細が0件"},
  {"id":"INV-3","condition":C(target="ST-3",agg="count",op="le",value=20),"violation":"明細が21件"}],
 "commands":[{"id":"CMD-1","name":"TERM-6","args":[{"id":"ARG-1","name":"TERM-4","type":"VO-4"}],
   "preconditions":[{"id":"PRE-1","condition":C(target="ST-2",op="eq",value="TERM-10"),"reject":"TERM-12","example":"確定済の注文を確定する"},
                    {"id":"PRE-2","condition":C(target="ST-3",agg="count",op="ge",value=1),"reject":"TERM-13","example":"明細0件の下書きを確定する"}],
   "postconditions":[{"id":"POST-1","condition":C(target="ST-2",op="eq",value="TERM-11")},
                     {"id":"POST-2","condition":C(target="ST-4",op="eq",value="ARG-1")}],
   "emits":[{"id":"EVT-1","name":"TERM-7","fields":[{"name":"注文番号","from":"ST-1"},{"name":"受け取り予定時刻","from":"ST-4"}]}],
   "accept_example":"明細2件の下書きを、受け取り予定時刻 12:30 で確定する → 確定済になり、注文確定済を出す"}]}
D['AGG-2']={"kind":"aggregate","id":"AGG-2","header":{"name":"TERM-9","context":"BC-1","description":"一定の時間に受ける調理の量が、店舗の能力を超えないようにする"},
 "structure":{"state":[{"id":"ST-1","name":"開始時刻","type":"VO-4","multiplicity":"1"},{"id":"ST-2","name":"空き","type":"VO-1","multiplicity":"1"}],"entities":[]},
 "invariants":[{"id":"INV-1","condition":C(target="ST-2",op="ge",value=0),"violation":"空きが -1"}],"commands":[]}
D['VO-1']={"kind":"value_object","id":"VO-1","header":{"name":"TERM-3","context":"BC-1"},
 "components":[{"id":"CMP-1","name":"個数","kind":"数","digits":"整数","unit":"個",
   "invariants":[{"id":"INV-1","condition":C(target="CMP-1",op="ge",value=1),"impossible":"0個"},{"id":"INV-2","condition":C(target="CMP-1",op="le",value=99),"impossible":"100個"}]}],
 "operations":[{"id":"OP-1","name":"足す","result":"VO-1","text":"2つの数量を足した数量を返す"}]}
D['VO-2']={"kind":"value_object","id":"VO-2","header":{"name":"TERM-14","context":"BC-1"},"components":[{"id":"CMP-1","name":"番号","kind":"文字","digits":"6桁","unit":"—","invariants":[]}],"operations":[]}
D['VO-3']={"kind":"value_object","id":"VO-3","header":{"name":"TERM-5","context":"BC-1"},"components":[{"id":"CMP-1","name":"状態","kind":"列挙","digits":"下書き ・ 確定済","unit":"—","invariants":[]}],"operations":[]}
D['VO-4']={"kind":"value_object","id":"VO-4","header":{"name":"TERM-4","context":"BC-1"},"components":[{"id":"CMP-1","name":"時刻","kind":"日時","digits":"分まで","unit":"—","invariants":[]}],"operations":[]}
D['DS-1']={"kind":"domain_service","id":"DS-1",
 "header":{"name":"TERM-8","context":"BC-1","reason":"複数の集約にまたがる計算","reason_text":"注文の明細と調理枠の空きの両方を使うので、どちらの集約に置いても、もう一方の状態を持ち込むことになる"},
 "reads":["AGG-1","AGG-2"],
 "operations":[{"id":"OP-1","name":"見積もる","inputs":[{"name":"明細の数量の合計","type":"VO-1"},{"name":"調理枠の空き","type":"VO-1"}],"output":"VO-4",
  "postconditions":[{"id":"POST-1","condition":C(target="結果",op="not_empty")},{"id":"POST-2","condition":C(target="結果",op="ge",value="調理枠の開始時刻")}]}]}
S=lambda no,name,actor,kind,text,to=None,reply=None,ext=None,var=None:{k:v for k,v in dict(no=no,name=name,actor=actor,kind=kind,to=to,reply=reply,text=text,extensions=ext or [],variations=var or []).items() if v is not None}
X=lambda label,name,ck,cond,ending,steps:{"label":label,"name":name,"condition_kind":ck,"condition":cond,"ending":ending,"steps":steps}
D['UC-1']={"kind":"use_case","id":"UC-1",
 "header":{"name":"注文を確定する","level":"ユーザー目的","scope":{"context":"BC-1"},"primary_actor":"顧客","trigger":"顧客が注文の確定を依頼する"},
 "preconditions":[{"id":"PRE-1","text":"顧客が識別されている","established_by":"UC-8"},{"id":"PRE-2","text":"カートに1品以上の商品が入っている","established_by":"UC-2"}],
 "stakeholders":[{"id":"SH-1","who":"顧客","interest":"約束した時刻に受け取り、確定した合計額だけを支払う"},
                 {"id":"SH-2","who":"店舗","interest":"調理の空きを超えて注文を受けず、支払いの承認を得た注文だけを確定する"},
                 {"id":"SH-3","who":"経理","interest":"確定と請求の記録を、後から突き合わせられる"}],
 "guarantees":{"minimal":[{"id":"MG-1","text":"決済代行の承認を受けていない注文は、確定しない","protects":["SH-2"]},
                          {"id":"MG-2","text":"確定しなかった注文には、請求も調理枠の確保も残らない","protects":["SH-1","SH-2"]},
                          {"id":"MG-3","text":"確定の試みがどこまで進んだかが、注文の経過として残る","protects":["SH-2","SH-3"]}],
               "success":[{"id":"SG-1","text":"注文が確定済になり、確定した合計額だけが顧客に請求される","satisfies":["SH-1"]},
                          {"id":"SG-2","text":"受け取り予定時刻が約束され、その時刻の調理枠が注文に確保されている","satisfies":["SH-1","SH-2"]},
                          {"id":"SG-3","text":"承認番号と確定した合計額が注文に記録される","satisfies":["SH-3"]}]},
 "scenario":{"steps":[
  S(1,"確定を依頼する","顧客","相互作用","顧客は、支払い手段を添えて、注文の確定をシステムに依頼する",to="システム",var=[{"varies":"支払い手段","values":["クレジットカード","電子マネー"]}]),
  S(2,"価格を確認する","システム","妥当性確認","システムは、カートの商品の価格が提示した額から変わっていないことを確認する",
    ext=[X("2a","価格が変わった","確認の失敗","提示した額から価格が変わった商品があった：","2へ戻る",[S("2a1","変わった価格を示す","システム","相互作用","システムは、顧客に変わった価格を示す",to="顧客"),S("2a2","変わった価格を受け入れる","顧客","相互作用","顧客は、変わった価格を受け入れる",to="システム")])]),
  S(3,"調理枠を確保する","システム","内部の状態変化","システムは、受け取り予定時刻を見積もり、その時刻の調理枠を確保する",
    ext=[X("3a","調理の空きがない","確認の失敗","希望の時間帯に調理の空きがなかった：","3へ戻る",[S("3a1","空きのある時刻を示す","システム","相互作用","システムは、空きのある最も早い受け取り予定時刻を示す",to="顧客"),S("3a2","示された時刻を受け入れる","顧客","相互作用","顧客は、示された時刻を受け入れる",to="システム")])]),
  S(4,"支払いの承認を得る","システム","相互作用","システムは、決済代行に請求額の承認を依頼し、承認番号を受け取る",to="決済代行",reply="承認番号",
    ext=[X("4a","承認されない","支援アクターの応答無しまたは誤り","決済代行が承認しなかった：","失敗",[S("4a1","調理枠を戻す","システム","内部の状態変化","システムは、確保した調理枠を戻す"),S("4a2","経過を記録する","システム","内部の状態変化","システムは、確定しなかった理由と進んだ手順を注文の経過に記録する"),S("4a3","承認されなかったと知らせる","システム","相互作用","システムは、顧客に支払いが承認されなかったことを知らせる",to="顧客")])]),
  S(5,"注文を確定する","システム","内部の状態変化","システムは、注文を確定し、承認番号と確定した合計額を注文に記録する"),
  S(6,"注文番号を知らせる","システム","相互作用","システムは、顧客に注文番号と受け取り予定時刻を知らせる",to="顧客")],
  "supporting_actors":["決済代行"]},
 "contributes_to":["SC-1","SC-2"],
 "links":{"quality":["QR-1"],"open_issues":["受け取り予定時刻を過ぎた注文の扱い"]}}
D['UC-1']['scenario']['steps'][2]['aggregate']='AGG-2'
D['UC-1']['scenario']['steps'][4]['aggregate']='AGG-1'
# 手順は、変更する集約ではなく、呼び出す集約のコマンドを指す。品質の要求は、かかる手順に付ける
D['BC-1']['ubiquitous_language']['terms']+= [{"id":"TERM-15","word":"確保する","kind":"コマンド","definition":"調理枠から、注文の分の空きを取る","avoid":[]},
                                             {"id":"TERM-16","word":"戻す","kind":"コマンド","definition":"確保した空きを、調理枠へ返す","avoid":[]}]
D['AGG-2']['commands']=[
 {"id":"CMD-1","name":"TERM-15","args":[{"id":"ARG-1","name":"TERM-3","type":"VO-1"}],
  "preconditions":[{"id":"PRE-1","condition":{"target":"ST-2","op":"ge","value":"ARG-1"},"reject":"TERM-13","example":"空き0の枠から1つ確保する"}],
  "postconditions":[{"id":"POST-1","condition":{"target":"ST-2","op":"not_empty"}}],"emits":[],"accept_example":"空き5の枠から2つ確保する → 空きが3になる"},
 {"id":"CMD-2","name":"TERM-16","args":[{"id":"ARG-1","name":"TERM-3","type":"VO-1"}],"preconditions":[],"postconditions":[{"id":"POST-1","condition":{"target":"ST-2","op":"not_empty"}}],"emits":[],"accept_example":"空き3の枠へ2つ戻す → 空きが5になる"}]
st=D['UC-1']['scenario']['steps']
for s in st: s.pop('aggregate',None)
st[2]['invokes']='AGG-2.CMD-1'
st[4]['invokes']='AGG-1.CMD-1'; st[4]['quality']=['QR-1']
st[3]['extensions'][0]['steps'][0]['invokes']='AGG-2.CMD-2'
D['UC-1']['links'].pop('quality',None)
# 手順は守る利害関係者を持つ。失敗で終わる拡張は、成り立つ最低保証を持つ。主成功シナリオは成功時保証で終わる
for s,sv in zip(D['UC-1']['scenario']['steps'],[["SH-1"],["SH-1","SH-2"],["SH-1","SH-2"],["SH-2"],["SH-1","SH-3"],["SH-1"]]): s['serves']=sv
D['UC-1']['scenario']['steps'][3]['extensions'][0]['guarantees_hold']=["MG-1","MG-2","MG-3"]
# 保証も、一覧の要素として短い名前を持つ
for gid,n in {"MG-1":"承認なしで確定しない","MG-2":"失敗したら何も残さない","MG-3":"どこまで進んだかを残す"}.items():
  [x for x in D['UC-1']['guarantees']['minimal'] if x['id']==gid][0]['name']=n
for gid,n in {"SG-1":"確定と請求","SG-2":"受け取り時刻と調理枠の確保","SG-3":"承認番号と合計額の記録"}.items():
  [x for x in D['UC-1']['guarantees']['success'] if x['id']==gid][0]['name']=n
# 突き合わせに使う不変の ID。2a などのラベルは表示専用で、手順を足しても ID は変わらない
for i,s in enumerate(D['UC-1']['scenario']['steps'],1): s['id']=f'STEP-{i}'
_n=0
for s in D['UC-1']['scenario']['steps']:
  for x in s['extensions']:
    _n+=1; x['id']=f'EXT-{_n}'
D['AGG-1']['commands'][0]['accept_examples']=[{"id":"OK-1","text":D['AGG-1']['commands'][0].pop('accept_example')}]
for c in D['AGG-2']['commands']: c['accept_examples']=[{"id":"OK-1","text":c.pop('accept_example')}]
D['UC-1']['exemptions']=[]
# 実装の定義（コーディング側）が持つ、文脈ごとの業務ロジックの実装方法。ここでは受注はドメインモデル
IMPL={"BC-1":"ドメインモデル"}
# 宣言どうしの対応（論点5）。参照は呼び出し側から1方向だけ書き、逆向きは道具が計算する
U=D['UC-1']; st=U['scenario']['steps']
for s in st:
  for x in s['extensions']: x.pop('guarantees_hold',None)   # 最低保証はどう終わっても守られるので、全件と同じになり、持たない
st[2]['extensions'][0]['handles']=['AGG-2.CMD-1.PRE-1']        # 3a 調理の空きがない
[p for p in U['preconditions'] if p['id']=='PRE-2'][0]['ensures']=['AGG-1.CMD-1.PRE-2']
sg={x['id']:x for x in U['guarantees']['success']}
sg['SG-1']['established_by']=['AGG-1.CMD-1.POST-1']
sg['SG-2']['established_by']=['AGG-1.CMD-1.POST-2','AGG-2.CMD-1.POST-1']
st[3]['keeps']=['MG-1']                                        # 承認を得てから確定へ進む
ex4=st[3]['extensions'][0]['steps']
ex4[0]['keeps']=['MG-2']; ex4[1]['keeps']=['MG-3']
