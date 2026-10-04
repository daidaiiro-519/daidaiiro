import json,os
H=os.path.dirname(os.path.abspath(__file__))
S='.claude/skills/fact-check/'
DOC=S+'SKILL.md'; SRC=S+'tool/business_logic/src/source.rs'; HTTP=S+'tool/data_access/src/http.rs'; SVC=S+'tool/service/src/lib.rs'; TST=S+'tool/business_logic/tests/source.rs'
def r(item,src,at,note=''):
  x={"item":item,"source":src,"at":at}
  if note: x["note"]=note
  return x
items=[
 r("DOM-1.vision.problem","document",f"{DOC}:13-14"),
 r("DOM-1.vision.values.VAL-1","document",f"{DOC}:9"),
 r("DOM-1.vision.values.VAL-2","document",f"{DOC}:86-87"),
 r("DOM-1.vision.values.VAL-1.differentiator","decided","","材料は、どの価値が他と違うかを書いていない"),
 r("DOM-1.vision.competitors","decided","","材料に競合が無いので空にした"),
 r("DOM-1.vision.success_criteria.SC-1","decided","","材料に達成の基準が無い"),
 r("DOM-1.scope.out","document",f"{DOC}:11"),
 r("DOM-1.stakeholders.SH-1","document",f"{DOC}:71"),
 r("DOM-1.stakeholders.SH-2","document",f"{DOC}:87"),
 r("DOM-1.design_scopes.SCP-1","decided","","スコープの段（システム）は材料に無い"),
 r("GLO-1.TERM-5（avoid：中身の要約）","code",f"{SRC}:292","コードの説明は sha256 を「中身の要約」と呼ぶ。要約ではないので、避ける語へ置いた"),
 r("GLO-1.TERM-14 原文の定義（変換しない）","document",f"{DOC}:97"),
 r("GLO-1.TERM-19","code",f"{TST}:206-209","テストの文言「誤りの頁を原文として受け取らない」から"),
 r("GLO-1.TERM-20","code",f"{TST}:205","テストの文言「空を原文として受け取らない」から"),
 r("GLO-1.TERM-21","code",f"{SRC}:323"),
 r("GLO-1.TERM-22","code",f"{SRC}:322"),
 r("GLO-1.TERM-23","code",f"{SRC}:367"),
 r("GLO-1.TERM-4 定義（利用者の地域の時刻）","code",f"{SRC}:310-317"),
 r("GLO-1 のほかの語","decided","","材料の語（出どころ ・ 置き場所 ・ 取得の記録 など）を拾い、定義は新しく書いた"),
 r("REQ-1.BR-1 出どころを渡す","code",f"{SVC}:41"),
 r("REQ-1.TEC-1","code",f"{HTTP}:14,18 / {SVC}:45","curl を使う理由は {HTTP}:4 の説明（document 相当のコードの説明）"),
 r("REQ-1.open_issues[0]","document",f"{DOC}:97"),
 r("UC-1.header","document",f"{DOC}:77-83"),
 r("UC-1.header.primary_actor","document",f"{DOC}:70","材料は「呼び出す側」と書く。アクターの名前「書き手」は新しく決めた"),
 r("UC-1.STEP-1","code",f"{SVC}:38-41"),
 r("UC-1.STEP-2 / EXT-1","code",f"{SVC}:40-42"),
 r("UC-1.STEP-3","document",f"{DOC}:83"),
 r("UC-1.STEP-3.variations","code",f"{SRC}:338-343",".txt ・ .json で終わる出どころは .md を試さない。この条件は変化の欄に書けない"),
 r("UC-1.EXT-2","code",f"{SRC}:319-325"),
 r("UC-1.STEP-4","code",f"{SRC}:345-362"),
 r("UC-1.EXT-3","code",f"{SRC}:363-369"),
 r("UC-1.STEP-5","code",f"{SVC}:49-58"),
 r("UC-1.MG-1","code",f"{SRC}:364"),
 r("UC-1.SG-1","code",f"{SRC}:354-361"),
 r("UC-1.SG-2","document",f"{DOC}:86"),
 r("UC-1.SG-3","document",f"{DOC}:86"),
 r("UC-1.stakeholders","document",f"{DOC}:87 / {DOC}:282"),
 r("SD-1.classification","decided","","問いの答えを出す業務エキスパートが居ないので、試行の側で決めた"),
 r("SD-1.business_logic","decided","","同上"),
 r("BC-1.header.purpose","code",f"{HTTP}:6","「判定を置かない」はデータアクセス層の説明から"),
 r("BC-1.context_map.X-1","code",f"{HTTP}:35-52"),
 r("BC-1.context_map.X-1.OP-1.retry","decided","","同じ出どころを2回取りに行っても害が無い、は材料に無い"),
 r("BC-1.business_rules.BR-1","code",f"{SVC}:41"),
 r("APP-1","code",f"{SVC}:38-63"),
 r("AGG-1.structure","code",f"{SRC}:280-299"),
 r("AGG-1.CMD-1.BR-1","code",f"{SRC}:276-277"),
 r("AGG-1.CMD-1.BR-2","code",f"{SRC}:276-277"),
 r("AGG-1.CMD-1.state_changes","code",f"{SRC}:350-361"),
 r("AGG-1.INV-1","code",f"{SRC}:276-277"),
 r("VO-3.INV-1","decided","","sha256 の16進は64字。材料には書いていない"),
 r("VO-5.INV-1","code",f"{SRC}:268"),
 r("VO-1 ・ VO-2 ・ VO-4 ・ VO-6 ・ VO-7","decided","","型の分け方は材料に無い"),
]
tests=[
 {"test":"a_name_is_made_from_the_source","at":f"{TST}:186-199","asserts":[
   {"line":189,"hits":[],"note":"出どころから名前を作る規則（記号を _ にする）が宣言に無い"},
   {"line":190,"hits":[],"note":"同上"},
   {"line":194,"hits":["VO-5.INV-1"]}],
  "kind":"当たる条件と、当たらない assert が混ざる","decision":"VO-5.INV-1 の記録の行を足す。189 ・ 190 の名前の作り方は、人が決める（宣言に足りない条件か、宣言の外の実装の細部か）"},
 {"test":"only_what_was_actually_received_is_kept","at":f"{TST}:202-211","asserts":[
   {"line":204,"hits":["AGG-1.CMD-1.OK-1"]},
   {"line":205,"hits":["AGG-1.CMD-1.BR-2"]},
   {"line":206,"hits":["AGG-1.CMD-1.BR-1"]},
   {"line":210,"hits":["AGG-1.CMD-1.BR-1"]}],
  "kind":"宣言の条件2つ以上に当たる","decision":"条件ごとに記録の行を出す（OK-1 ・ BR-1 ・ BR-2 の3行）。210 の 301 は BR-1 の2つ目の値"},
 {"test":"the_listing_reads_the_records","at":f"{TST}:213-227","asserts":[
   {"line":223,"hits":[]},{"line":224,"hits":[]},{"line":225,"hits":[]},{"line":226,"hits":[]}],
  "kind":"当たる条件が無い","decision":"移した範囲（原文を取得する）の外。別のユースケース（取得したものを並べる）を移すときに扱う。いまは回帰テストとして回し続ける"},
 {"test":"the_curl_command_passed_in_is_the_one_run","at":f"{TST}:229-235","asserts":[
   {"line":234,"hits":["BC-1.X-1.OP-1.F-2","UC-1.EXT-2"]}],
  "kind":"宣言の条件1つに当たる（設計の側の失敗と、それを扱う拡張の組）","decision":"UC-1.EXT-2 の記録の行を足す。ただし assert が見ているのは「取得できない」までで、書き手へ試した先を知らせることは見ていない"}]
gaps=[
 {"condition":"BC-1.X-1.OP-1.F-1（制限時間を過ぎた）","note":"テストが無い"},
 {"condition":"UC-1.M（主成功シナリオ）","note":"取得から記録の書き出しまでを通すテストが無い"},
 {"condition":"UC-1.EXT-1（出どころが無い）","note":"テストが無い"},
 {"condition":"UC-1.EXT-3（受け取らない応答で失敗）","note":"acceptable の単体のテストはあるが、fetch が試した先を返して失敗することは見ていない"},
 {"condition":"AGG-1.INV-1","note":"BR-2 のテストが間接に見ているだけ"},
 {"condition":"VO-3.INV-1 ・ VO-4.INV-1","note":"テストが無い"},
 {"condition":"UC-1.STEP-3.variations（.md を先に試す）","note":"テストが無く、テスト条件としても取り出されない"}]
M={"use_case":"UC-1","from":{"document":DOC,"code":[SRC,HTTP,SVC],"tests":TST,"read_at":"2026-10-04"},
 "sources_note":"expert は使わない。業務エキスパートとの会話が無い試行なので、材料に無いものはすべて decided とした",
 "items":items,"tests":tests,"gaps":gaps,
 "scope":{"unit":"UC-1（原文を取得する）","not_yet":["取得したものを並べる（list）","照合する（verify）"],"done_when":"この範囲で、宣言どうしのずれが0件、テストの欠けが0件"}}
json.dump(M,open(os.path.join(H,'spec','migration.json'),'w',encoding='utf-8'),ensure_ascii=False,indent=1)
import collections; print(collections.Counter(x['source'] for x in items))
