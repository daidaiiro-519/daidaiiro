import json, copy, os
D=os.path.dirname(os.path.abspath(__file__))
J=lambda o: json.dumps(o, ensure_ascii=False, indent=2)
ABS=open(os.path.join(D,'abstract.json')).read()

def P(means, write=None, **read):
    r={"means":means}; r.update(read)
    return {"read":r,"write":write or []}
def R(id,rule,by,why,test,agent=None,reads=None):
    o={"id":id,"rule":rule,"why":why,"test":test,"by":by}
    if reads: o["reads"]=reads
    if agent: o["agent"]=agent
    return o

# ---- Concrete の段階 ----
C1={
 "$schema":"urn:semantic-agent-schema:base:semantic-abstract-schema:v1",
 "$id":"urn:semantic-agent-schema:acdr:decision-record:v1",
 "title":"意思決定の記録",
 "description":"決定 ・ なぜ ・ どう変わるか ・ 比較した案を1件にまとめた記録。対象の文書の全文は持たない",
 "x-instance-path":".acdr/{no}-{slug}/acdr.json",
 "x-instance-label":"ACDR {no}「{title}」",
 "type":"object",
 "required":["$schema","kind"],
 "properties":{
  "$schema":{"const":"urn:semantic-agent-schema:acdr:decision-record:v1"},
  "kind":{"const":"decision-record","description":"インスタンスの種類","x-prompt":P("このインスタンスが決定の記録であること",write=[R("fixed-kind","decision-record のまま変えない","machine","基盤は kind で種類を判定する","値が decision-record かを見る")])}
 }
}
C2=copy.deepcopy(C1)
C2["required"]=["$schema","kind","no","title","status","decision","why","shift","alternatives"]
C2["properties"].update({
 "no":{"type":"string","pattern":"^[0-9]{4}$","description":"記録の番号","x-prompt":P("記録の通し番号",parse="4桁の数字の文字列。先頭の 0 も番号の一部で、数として読まない。置き場所の {no} に入る",shown="題の前に「ACDR 0197」として出る",when="書く前に、次の番号を決めるために読む",write=[R("unique-no","ほかの記録と番号を重ねない","machine","番号で置き場所が決まる","同じ番号の記録がほかに無いかを見る（記録をまたぐので基盤のエンジン）")])},
 "title":{"type":"string","minLength":1,"description":"題","x-prompt":P("決めたことの名前",shown="一覧と、ほかの記録から指されたときに「ACDR 0197「…」」として出る",write=[R("noun","決めたことを名詞句で書く","ai","題だけで何の決定かが分かる","名詞で終わるかを見る","reviewer")])},
 "status":{"oneOf":[{"const":"proposed","title":"提案。承認前で、正本に適用してはならない"},{"const":"accepted","title":"承認済み"},{"const":"superseded","title":"後の記録に置き換えられた"}],"default":"proposed","description":"状態","x-decided-by":"human","x-prompt":P("承認の状態",when="正本に適用してよいかを決めるときに読む",write=[R("human-approves","accepted にするのは人の入口からだけ","machine","承認は人が決める","変更の記録で、人の入口を通ったかを見る（変更をまたぐので基盤のエンジン）")])},
 "decision":{"type":"string","minLength":1,"description":"決めたこと","x-prompt":P("この記録で決めたこと1件",parse="何を採るかの1文として読む。コマンドや旗の名前があっても、手段ではなく決定の対象の名前として読む",shown="記録の一番上に大きく出る",write=[R("decision-no-means","決定の文に手段（コマンド ・ 旗 ・ ファイル名）を混ぜない","ai","決定は何を採るかで、手段は変更の印が示す","決定の文を読む","reviewer",["/decision"])])},
 "why":{"type":"string","minLength":1,"description":"なぜ","x-prompt":P("何と食い違っていて、何に合わせたか",parse="食い違った相手と、合わせた先の2つを読み取る。根拠の引用があれば、出どころと一緒に読む",write=[R("names-both","食い違った相手と、合わせた先の両方を書く","ai","片方だけでは、なぜ変えたかが分からない","2つが書かれているかを見る","reviewer",["/why"])])},
 "shift":{"type":"array","minItems":1,"items":{"type":"object","required":["what","from","to"],"properties":{"what":{"type":"string"},"from":{"type":"string"},"to":{"type":"string"}}},"description":"どう変わるか","x-prompt":P("抽象の側の変更前と変更後",shown="変更前と変更後を並べた表として出る",write=[R("abstract-side","what には、ファイル名ではなく抽象の側の名前を書く","ai","具体の差分は変更の印が示す","what がファイル名やコマンドでないかを見る","reviewer",["/shift"])])},
 "alternatives":{"type":"array","minItems":1,"items":{"type":"object","required":["option","why_not"],"properties":{"option":{"type":"string","minLength":1},"why_not":{"type":"string","minLength":1}}},"description":"比較した案","x-prompt":P("案ごとに、採らなかった理由",parse="どれも反証を通らなかった案で、残っている懸念ではない",write=[R("refuted-only","反証を通らなかった案だけを書き、未解決の懸念を混ぜない","ai","懸念が混ざると、承認する人が判断を止める","why_not が「通らなかった理由」になっているかを見る","reviewer",["/alternatives"])])},
 "supersedes":{"type":"string","pattern":"^[0-9]{4}$","description":"置き換えた記録の番号","x-prompt":P("この記録が置き換えた、前の記録の番号",write=[R("target-exists","置き換えた記録が在る番号を書く","machine","無い記録は置き換えられない","その番号の記録が在るかを見る（記録をまたぐので基盤のエンジン）")])},
 "docs":{"type":"array","description":"変更した文書と、変更の印","x-prompt":P("この決定で変えた文書と、その印",write=[R("why-per-change","変更の印ごとに、なぜを書く","machine","理由の無い変更は記録に含めない","git の差分のまとまりごとに理由があるかを見る（Git を読むので acdr の CLI）")])}
})
C2["allOf"]=[{"if":{"properties":{"status":{"const":"superseded"}}},"then":{"required":["supersedes"]}},
             {"if":{"properties":{"status":{"const":"accepted"}}},"then":{"required":["docs"]}}]
C3=copy.deepcopy(C2)
C3=dict(list(C3.items())[:6]+[("x-delegates",{
 "reviewer":{"role":"ACDR の決定のレビュー役","answer":{"$ref":"#/$defs/review-body"},"run":{"kind":"judging"}},
 "acdr-inspect":{"role":"ACDR の変更の印の検査（Git の差分を読む）","run":{"kind":"judging","argv":["acdr","inspect","--json"]}}
})]+list(C3.items())[6:])
C3["properties"]["docs"]["x-prompt"]["write"][0]["agent"]="acdr-inspect"
C3["properties"]["decision"]["x-prompt"]["write"].append(R("least-privilege","決定が AI の許可を動かすなら、最小限だけを与える形にし、その根拠を書く","ai","許可を広げる決定は、あとから狭めにくい","なぜ と どう変わる の節で、許可の置き場所と根拠を見る","reviewer",["/decision","/why","/shift"]))
C3["$defs"]={"review-body":{"type":"object","properties":{"findings":{"type":"array","description":"直す箇所の並び","x-prompt":P("レビューで見つけた、直す箇所の並び",parse="各項目は at（場所）・ rule（どの行か）・ problem（何が問題か）・ fix（直し方）",write=[R("fix-per-finding","指摘ごとに直し方を1つ書く","ai","差し戻すだけでは直せない","各項目に fix があるかを見る","reviewer")])}}}}

# ---- インスタンス ----
I1={"$schema":"urn:semantic-agent-schema:acdr:decision-record:v1","kind":"decision-record","no":"0197",
 "title":"画面の AI の許可を、各 AI ツールの設定ファイルに任せる","status":"proposed",
 "decision":"ボードの画面の AI に何を許すかを、各 AI ツール自身の設定ファイルで決める。",
 "why":"画面の AI は agents.json の一覧だけで起動され、各ツールの設定と食い違っていた。各ツールの設定ファイルに合わせる。",
 "shift":[{"what":"許可の置き場所","from":"agents.json","to":"各 AI ツールの設定ファイル"}],
 "docs":[{"path":"brainstorming/tool/agent.rs","marks":"…"},{"path":".claude/settings.json","marks":"…"}],
 "alternatives":[{"option":"agents.json に許可も書く","why_not":"ツールの設定と2か所になる"},{"option":"基盤が許可を持つ","why_not":"基盤は許可の契約を持たない"},{"option":"許可を書かない","why_not":"何を許すかが決まらない"}]}
I2=copy.deepcopy(I1)
I2["why"]="画面の AI は agents.json の一覧だけで起動され、各ツールの設定と食い違っていた。各ツールの設定ファイルに合わせる。最小権限：判定させる AI は道具なしで起動し、作業させる AI の許可は各ツールの設定ファイルに置く（platform-advisor sec-03「意図したタスクに必要な最小限だけを与える」）。"
I2["shift"][0]["to"]="各 AI ツールの設定ファイル。共有する許可は .claude/settings.json、settings.local.json は個人の追加だけ"
I3=copy.deepcopy(I2); I3["status"]="accepted"

# ---- 記録 ----
def jr(rule,res,reads,by="AI（reviewer）"): return {"対象":"ACDR 0197","rule":rule,"結果":res,"判定した者":by,"読んだ欄":reads,"効いているか":True}
J1=[jr("title/noun","verified",["/title"]),jr("decision-no-means","verified",["/decision"]),jr("why/names-both","verified",["/why"]),jr("least-privilege","unverified",["/decision","/why","/shift"])]
J2=copy.deepcopy(J1); J2[2]["効いているか"]=False; J2[3]["効いているか"]=False
J3=copy.deepcopy(J1); J3[3]["結果"]="verified"
J4=J3+[{"対象":"ACDR 0197","rule":"status/human-approves","結果":"verified","判定した者":"作業した本人（人）","読んだ欄":["/status"],"効いているか":True}]
CH1=[{"id":"c1","目的":"ACDR 0197","意図":"ACDR 0197 を起こす","書いた者の種類":"AI の Agent","欄":["/no","/title","/status","/decision","/why","/shift","/alternatives"]}]
CH2=CH1+[{"id":"c2","目的":"ACDR 0197","意図":"least-privilege の根拠を書く","書いた者の種類":"AI の Agent","欄":["/why","/shift/0/to"]}]
CH3=CH2+[{"id":"c3","目的":"ACDR 0197","意図":"ACDR 0197 を承認する","書いた者の種類":"人（人の入口）","欄":["/status"]}]
SE1=[{"ID":"0a9d…","種類":"オーケストレーターのセッション","目的":None,"親":None,"役割":None,"状態":"開いている"},{"ID":"7c1e…","種類":"目的のセッション","目的":"ACDR 0197","親":"0a9d…","役割":None,"状態":"開いている"}]
SE2=SE1+[{"ID":"b52a…","種類":"任せる先のセッション","目的":"ACDR 0197","親":"7c1e…","役割":"platform-advisor の advisor","状態":"閉じた"}]

F_ABS='semantic-abstract-schema.json（Abstract の本体）'
F_CON='acdr の Semantic Concrete Schema'
F_INS='.acdr/0197-…/acdr.json（インスタンス）'
F_JUD='判定の記録'
F_CHG='変更の記録'
F_SES='セッションの索引（手元の DB の表 sessions。コミットしない）'
COLS=['tool','id','purpose_ref','parent','role','state','last_used_at']
ROWS1=[['claude-code','0a9d…','—','—','—','open','10:02'],['claude-code','7c1e…','ACDR 0197','claude-code/0a9d…','—','open','10:05']]
ROWS2=[['claude-code','0a9d…','—','—','—','open','10:02'],['claude-code','7c1e…','ACDR 0197','claude-code/0a9d…','—','open','10:31'],['claude-code','b52a…','ACDR 0197','claude-code/7c1e…','urn:…platform-advisor:judgment:v1#advisor','closed','10:29']]
F_SCH='手元のセッションの索引（手元の DB の表 session_changes。セッション → 変更の記録）'
SCOLS=['tool','id','change_id']
SC1=[['claude-code','7c1e…','c1']]
SC2=[['claude-code','7c1e…','c1'],['claude-code','7c1e…','c2']]

S=[
 dict(a=[1],actor='目的のセッション（目的：ACDR の Concrete を作る。使う場面の「作者」）',op='Abstract の本体から、ACDR の Concrete の雛形を作る',
  pre=['Abstract の本体（基盤が持つ。この流れでは変わらない）がある','ACDR の Concrete はまだ無い'],
  post=['Concrete の雛形ができる。インスタンスを作るので kind と $schema を const で持つ'],
  beh=['基盤の入口「意味を宣言する（作成）」が、Abstract の本体から雛形を返す（Q6 #3）','Abstract の根の x-prompt（Concrete の書き方）を、型1と同じ形で作者に見せる'],
  files=[dict(f=F_ABS,why='参照するだけで変えない。全体はここで一度だけ見せる',before=ABS,after=ABS,ref=True),
         dict(f=F_CON,why='雛形を新しく作った',before='',after=J(C1))]),
 dict(a=[1],actor='目的のセッション（作者）',op='Concrete に欄と、欄ごとの x-prompt を書く',
  pre=['Concrete は雛形だけで、欄が無い'],
  
  post=['欄ができ、それぞれが説明と x-prompt を持つ','値だけで決まる決まりは JSON Schema で書く：no は4桁（pattern）、shift と alternatives の各項目の必須（items ・ required）、status の値ごとの意味（oneOf ・ title）、superseded なら supersedes が要る（if ・ then）','x-prompt の write には、JSON Schema で書けない決まりだけが残る：記録をまたぐもの（基盤のエンジン）、Git を読むもの（CLI）、意味の判定（AI）'],
  beh=['基盤が Concrete を Abstract の本体で検査する。欄ごとに description と x-prompt があるか（$defs/field）','インスタンスを書くたびに、JSON Schema の実装が pattern ・ items ・ if/then を判定する。AI も CLI も要らない','by: ai の行の agent は、任せる先がまだ無いので未解決（次の場面で足す）'],
  files=[dict(f=F_CON,why='欄と、その読み方 ・ 書き方を書いた',before=J(C1),after=J(C2))]),
 dict(a=[1],actor='目的のセッション（作者）',op='任せる先（x-delegates）と、AI に判定させる行を足す',
  pre=['by: ai の行が指す reviewer が、まだどこにも無い'],
  post=['x-delegates に reviewer（AI ツールで判定させる）と acdr-inspect（argv の CLI で判定させる）がある。どちらも目的は judging','Git の差分を読む行（why-per-change）は acdr-inspect を、ai の行は reviewer を指す。記録をまたぐ machine の行は agent を持たず、基盤のエンジンが判定する','decision に least-privilege の行が足される'],
  beh=['基盤が Concrete を Abstract で検査する。x-delegates の各項目が $defs/delegate の形か、行の agent が x-delegates にあり、by と（kind ・ argv の有無）が合うか'],
  files=[dict(f=F_CON,why='レビューを AI に、記録の形の検査を acdr の CLI に任せるため',before=J(C2),after=J(C3))]),
 dict(a=[0,3],actor='オーケストレーターのセッション（人が始め、CLAUDE.md で動く）',op='ACDR 0197 を進める目的のセッションを、基盤に立てさせる',
  pre=['セッションの索引は空（このチャットのような最上位のセッションだけがある）'],
  post=['オーケストレーターのセッション 0a9d… と、目的のセッション 7c1e… が登録される','目的のセッションの親は 0a9d…（目的は別なので、目的は親と同じでなくてよい）'],
  beh=['入口「セッションを決める ・ 立てる」が、オーケストレーターのセッションを登録する。このリポジトリ（手元の DB）で開いているのは1つで、ほかのターミナルは同じ ID で再開する','基盤は、依頼元が役割を持たないことを確かめ、目的のセッションを起動して登録する。同じ目的で開いているセッションがあれば、新しく立てずにそれを再開する','オーケストレーターは入れ子にしない。大きな目的を分けるのは目的の側（ACDR ・ ボードの論点）で、オーケストレーターは小さな目的ごとに目的のセッションを立てる'],
  files=[dict(f=F_SES,why='オーケストレーターのセッションと、目的のセッションを登録した',table=True,cols=COLS,before=[],after=ROWS1)]),
 dict(a=[1,3],actor='目的のセッション（AI の Agent）',op='基盤から書き方を受け取り、ACDR 0197 を書く（型1）',
  pre=['ACDR 0197 はまだ無い'],
  post=['ACDR 0197 が proposed でできる','変更の記録 c1 が残る'],
  beh=['入口「案内する」が、Concrete の欄を見出しにし、write の行を「書き方」にして返す','入口「インスタンスを書く」が、セッション ID と意図を受け取り、変更の記録に目的を写す'],
  md=[('基盤が返した書き方（型1）','''### 決めたこと（decision）
> 決定の文に手段（コマンド ・ 旗 ・ ファイル名）を混ぜない
> 決定が AI の許可を動かすなら、最小限だけを与える形にし、その根拠を書く
.ここに書く
### なぜ（why）
> 食い違った相手と、合わせた先の両方を書く
.ここに書く''')],
  files=[dict(f=F_INS,why='新しく書いた',before='',after=J(I1)),dict(f=F_CHG,why='書いた変更を意図と一緒に残した（セッション ID は持たない。I34）',before='[]',after=J(CH1)),dict(f=F_SCH,why='どのセッションで書いた変更かを、手元にだけ残した（I38）',table=True,cols=SCOLS,before=[],after=SC1)]),
 dict(a=[1,3,4],actor='目的のセッション（AI の Agent）',op='ACDR 0197 の検証を頼む',
  pre=['判定の記録は無い'],
  post=['JSON Schema の判定と machine の行はすべて verified（その場で出し、記録に残さない）','AI の行の判定の記録が4件残る。least-privilege は unverified で、何が分かれば決められるかが付く'],
  beh=['形の決まり（pattern ・ items ・ if/then など）は JSON Schema の実装が判定する','記録をまたぐ行（番号が重ならない）は基盤のエンジンが、Git の差分を読む行（why-per-change）は acdr inspect を引数の配列で起動して判定する','ai の行を reviewer ごとに集め、読む欄を描いた呼び出し（型3）で、道具なしの新しいセッションで起動する','レビュー agent は行ごとに5つの値を返す。セッションの索引には登録しない'],
  md=[('レビュー agent に渡した呼び出し（型3）','''### 役割
ACDR の決定のレビュー役
### 判定する行
- title/noun ・ decision-no-means ・ why/names-both ・ least-privilege
### 読む欄（ACDR 0197）
#### 決めたこと（decision）
> 何を表すか：この記録で決めたこと1件
> 解き方：何を採るかの1文として読む。コマンドや旗の名前があっても、手段ではなく決定の対象の名前として読む
ボードの画面の AI に何を許すかを、各 AI ツール自身の設定ファイルで決める。
#### なぜ（why）
> 何を表すか：何と食い違っていて、何に合わせたか
> 解き方：食い違った相手と、合わせた先の2つを読み取る。根拠の引用があれば、出どころと一緒に読む
画面の AI は agents.json の一覧だけで起動され、…
### 答え方
行ごとに5つの値と根拠。決められない行は unverified にし、何が分かれば決められるかを書く''')],
  files=[dict(f=F_JUD,why='AI の行の結果を、行ごとに残した',before='[]',after=J(J1))]),
 dict(a=[1,3,5],actor='目的のセッション（AI の Agent）',op='platform-advisor を作業させる agent として呼び、根拠を受け取る（型2）',
  pre=['least-privilege が unverified'],
  post=['任せる先のセッション（advisor）が登録され、答えを返したあと目的のセッションが閉じた','advisor の答え（最小権限に合う、共有の許可は .claude/settings.json）を親が受け取った'],
  beh=['基盤は、依頼元 7c1e… が役割を持たないことを確かめてから、任せる先のセッションを起動する','advisor は working なので、判定の記録は作らない'],
  md=[('advisor に渡した呼び出し（型2）','''### 役割
非機能の設計の助言役
### 終えること
最小権限の考え方に合うかを、判断基準の原文を引いて答える
### 書く欄（judgment）
#### 結論（conclusion）
> 書き方：採るか採らないかと、その1文
#### 根拠（grounds）
> 書き方：判断基準の原文をそのまま引く''')],
  files=[dict(f=F_SES,why='任せる先のセッションを登録し、答えのあと閉じた',table=True,cols=COLS,before=ROWS1,after=ROWS2)]),
 dict(a=[1],actor='目的のセッション（AI の Agent）',op='advisor の根拠を ACDR 0197 の本文に書き足す',
  pre=['ACDR 0197 の why に最小権限の根拠が無い'],
  post=['why と shift が変わる','その欄を読んでいた判定2件が効かなくなる。decision だけを読んだ判定は効いたまま','変更の記録 c2 が残る'],
  beh=['基盤は、判定した値（読んだ欄の値）がいまの値と違う判定を、効かないとして扱う'],
  files=[dict(f=F_INS,why='advisor の根拠を、判定させる AI が読む欄に書いた',before=J(I1),after=J(I2)),
         dict(f=F_JUD,why='読んだ欄（why ・ shift）の値が変わったので、その2件は効かない',before=J(J1),after=J(J2)),
         dict(f=F_CHG,why='書き足した変更を残した',before=J(CH1),after=J(CH2)),
         dict(f=F_SCH,why='書き足した変更も、同じ目的のセッションに結びつけた',table=True,cols=SCOLS,before=SC1,after=SC2)]),
 dict(a=[1,3,4],actor='目的のセッション（AI の Agent）',op='もう一度検証を頼む',
  pre=['判定2件が効かない'],
  post=['効かなかった2件だけ聞き直され、verified になる'],
  beh=['基盤は、効いている判定は聞き直さない（Q10 #7）','レビュー agent は新しいセッションで起動する'],
  files=[dict(f=F_JUD,why='効かなかった2件を聞き直した。least-privilege は verified になった',before=J(J2),after=J(J3))]),
 dict(a=[2,3],actor='作業した本人（人）',op='ACDR 0197 を承認する（セッションの中の本人の承認。Q0 #3）',
  pre=['AI の行はすべて verified','status は proposed'],
  post=['status が accepted になる','本人の判定が記録に残り、変更の記録 c3 が残る'],
  beh=['status は x-decided-by: human なので、基盤は人の入口から来た値だけを受け付ける','accepted になったので、if/then により docs が必須になる。JSON Schema の実装が、docs が在ることを確かめる','AI が status を書き換えると、変更の例外として取り込めない'],
  files=[dict(f=F_INS,why='人が承認した',before=J(I2),after=J(I3)),
         dict(f=F_JUD,why='人の判定を残した',before=J(J3),after=J(J4)),
         dict(f=F_CHG,why='承認の変更を残した',before=J(CH2),after=J(CH3))]),
]
for k,sc in enumerate(S): sc['ph']='make' if k<3 else 'use'
tpl=open(os.path.join(D,'tpl.html')).read()
open(os.path.join(D,'simulation.html'),'w').write(tpl.replace('/*DATA*/','const S='+json.dumps(S,ensure_ascii=False)+';'))
print('ok',len(S))
open(os.path.join(D,'concrete.json'),'w').write(J(C3))
open(os.path.join(D,'instances.json'),'w').write(json.dumps({'I1':I1,'I2':I2,'I3':I3},ensure_ascii=False))
