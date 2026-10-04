# 論点7の欄の直し（7′）を、移した宣言に当てる。build.py が出した宣言を読み、直した形で書き戻す
# 1 試す順番と 2 条件の付いた変化 ── 拡張で書く（別の道筋での成功 ・ 終わり方「成功」）
# 3 してはならない ── 成功時保証の条件で書く（受け取ったまま残る）
# 4 文字数 ── その他の要求のデータ要求に書き、手順から結ぶ（measure: length）
import json,os
H=os.path.dirname(os.path.abspath(__file__))
def rd(n): return json.load(open(os.path.join(H,'decls',n+'.json'),encoding='utf-8'))
def wr(n,d): json.dump(d,open(os.path.join(H,'decls',n+'.json'),'w',encoding='utf-8'),ensure_ascii=False,indent=1)
G=rd('GLO-1'); G['terms']=[t for t in G['terms'] if t['id'] not in ('TERM-30','TERM-31','TERM-32')]; have=set()
for i,w,d,k in [("TERM-30","「.md」を付けた先","出どころの末尾に .md を付けた先","情報の別名"),
                ("TERM-31","保存した原文","取得の記録が指す、保存した場所に置いた中身","状態"),
                ("TERM-32","受け取った中身","情報源のサーバーが返した応答の本体","値オブジェクト")]:
  if i not in have: G['terms'].append({"id":i,"word":w,"meanings":[{"id":"M-1","definition":d,"kind":k}],"avoid":[]})
wr('GLO-1',G)
U=rd('UC-1'); st=U['scenario']['steps']; s3=[s for s in st if s['id']=='STEP-3'][0]
s3.pop('variations',None); s3['data']=["TERM-30"]
again=lambda p:{"id":p,"kind":"相互作用","actor":"システム","to":"情報源のサーバー","data":["TERM-2"],"verb":"TERM-15","reply":["TERM-14"],"extensions":[]}
ext2=[x for x in s3['extensions'] if x['id']=='EXT-2'][0]
s3['extensions']=[
 {"id":"EXT-4","condition_kind":"別の道筋での成功","condition":{"target":"TERM-13.TERM-2","op":"ends_with","value":[".md",".txt",".json"]},"ending":"成功","steps":[again("EXT-4.S-1")]},
 {"id":"EXT-5","condition_kind":"業務ルールの拒否","reasons":["TERM-19","TERM-20"],"ending":"成功","steps":[again("EXT-5.S-1")]},
 ext2]
U['guarantees']['success']=[g for g in U['guarantees']['success'] if g['id']!='SG-4']+[{"id":"SG-4","name":"受け取ったまま残る","condition":{"target":"TERM-1.TERM-31","op":"eq","value":"TERM-32"},"satisfies":["SH-1"]}]
U['links']['data']=["REQ-1.DAT-1"]
U['open_issues']=[x for x in U['open_issues'] if '順に試す' not in x]
wr('UC-1',U)
A=rd('AGG-1'); S=A['structure']['state']
if not [x for x in S if x['id']=='ST-9']: S.append({"id":"ST-9","name":"TERM-31","type":"VO-8","multiplicity":{"min":1,"max":1}})
c=A['commands'][0]
if not [x for x in c['args'] if x['id']=='ARG-7']: c['args'].append({"id":"ARG-7","name":"TERM-32","type":"VO-8"})
if not [x for x in c['state_changes'] if x['id']=='CHG-9']: c['state_changes'].append({"id":"CHG-9","condition":{"target":"ST-9","op":"eq","value":"ARG-7"}})
wr('AGG-1',A)
wr('VO-8',{"kind":"value_object","id":"VO-8","header":{"name":"TERM-32","context":"BC-1"},"components":[{"id":"CMP-1","name":"本体","kind":"バイト列","invariants":[]}],"operations":[]})
V=rd('VO-5'); V['components'][0]['invariants']=[]; wr('VO-5',V)
R=rd('REQ-1'); R['data']=[{"id":"DAT-1","name":"保存する名前","condition":{"target":"TERM-11","measure":"length","op":"le","value":120}}]
R['open_issues']=[x for x in R['open_issues'] if '変換しない' not in x]; wr('REQ-1',R)
P=rd('APP-1')
P['extensions']=[x for x in P['extensions'] if x['extension']!='UC-1.EXT-5']+[{"extension":"UC-1.EXT-5","raised_by":["AGG-1.CMD-1.BR-1","AGG-1.CMD-1.BR-2"]}]
P['guarantees']=[x for x in P['guarantees'] if x['guarantee']!='UC-1.SG-4']+[{"guarantee":"UC-1.SG-4","established_by":["AGG-1.CMD-1.CHG-9"]}]
wr('APP-1',P)
B=rd('BC-1')
for t in ("TERM-30","TERM-31","TERM-32"):
  if not [u for u in B['uses'] if u['term']==t]: B['uses'].append({"term":t,"meaning":"M-1"})
wr('BC-1',B)
