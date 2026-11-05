"""セマンティックの操作を procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）として、事前状態 → 入力 → 適用 → 事後状態で見せる頁。"""
import os, json
H = os.path.dirname(os.path.abspath(__file__)); C = os.path.dirname(os.path.dirname(H))
src = open(os.path.join(C, 'arch.html')).read()
head = src[:src.index('</style>')] + '\n</style>'
head = head.replace('<title>基盤のアーキテクチャ</title>', '<title>コマンドのシミュレーション</title>')
def table(h, rows):
    return '<div class="tw"><table><tr>' + ''.join(f'<th>{x}</th>' for x in h) + '</tr>' + ''.join('<tr>' + ''.join(f'<td>{c}</td>' for c in r) + '</tr>' for r in rows) + '</table></div>'
J = lambda o: json.dumps(o, ensure_ascii=False, indent=1)
URN = "urn:semantic-agent-schema:acdr:decision-record:v1"
P0199 = ".acdr/0199-依存の向き/acdr.json"
WHY0 = "cargo-deny では層の並びを規則にできず、規則を手で確かめていたのと食い違っていた。規則ファイルで検査する形に合わせる。"
WHY1 = "cargo-deny では層の並びを規則にできなかった。層の並びを規則ファイルに書いて検査する形に合わせる。"
V0 = {"no":"0199","slug":"依存の向き","title":"依存の向きの検査ツール","status":"proposed","decision":"architecture-lint を作る","why":WHY0,
      "shift":[{"what":"依存の向きの検査","from":"手で確かめる","to":"architecture-lint の規則ファイルで検査する"}],
      "alternatives":[{"option":"cargo-deny を使う","why_not":"層の並びを規則にできない"}]}
R0 = dict({"$schema":URN,"kind":"decision-record","id":"01m5t5rgz82kj790z41b7etrkw"}, **V0)
ST_WAIT = {"name":"awaiting-judgment","label":"判断待ち"}
# 段：題 ・ 使う場面 ・ セマンティック ・ procedure ・ AI の入力 ・ 機械的 ・ 事前 ・ 事後 ・ 入力の検査 ・ 出力の検査 ・ 状態
# 出力の例は、実装の試験（test-suite/scenarios の S-INST-CREATE ・ S-INST-ETAG ・ S-INST-STALE ・ S-INST-EMPTY ・ S-INST-ACCEPTED）を基盤のコマンドで流した出力から取った
STEPS = [
 dict(t="探す", uc="U28 問いに答える", sem="semantic-search", mech="search",
  proc={"手順":"問いを where と select に直す。kind と term は候補から選ぶ","入力のスキーマ":"query#/$defs/request","出力のスキーマ":"query#/$defs/result"},
  inp={"kind":[URN],"where":"contains(title, '依存')","select":{"no":"no","decision":"decision"},"contexts":["daidaiiro"]},
  before={"記録":["0185 依存の向きの規則","0199 依存の向きの検査ツール","0150 古い決定"]}, after=None,
  out={"$schema":"urn:semantic-agent-schema:base:query:v1#/$defs/result","query":"（渡した request）",
       "items":[{"id":"<0185 の id>","kind":URN,"version":"<読んだ版（何を入れるかは未決）>","values":{"no":"0185","decision":"層の並びで検査する"}},
                {"id":R0["id"],"kind":URN,"version":"<読んだ版（何を入れるかは未決）>","values":{"no":"0199","decision":"architecture-lint を作る"}}],
       "scope":{"contexts":["daidaiiro"],"index_of":"<正本の版（何を入れるかは未決）>"},"not_searched":[]},
  state="状態は変わらない", ic="合う", oc=["形：合う（query の result）","機械の rule：出どころを評価し直すと同じ値","AI の rule：無い"]),
 dict(t="読む", uc="U6 既存のインスタンスを変える（前段）", sem="semantic-read", mech="get",
  proc={"手順":"欄の title ・ description と x-prompt の read に従って読む。reads の値も一緒に読む。書くときのために etag を控える","入力のスキーマ":"procedure#/$defs/read-request（パス ・ JSON Pointer）","出力のスキーマ":"get の出力（data.path ・ pointer ・ value ・ etag）"},
  inp={"path":P0199,"pointer":"/why"},
  before=R0, after=None, out={"ok":True,"findings":[],"data":{"path":P0199,"pointer":"/why","value":WHY0,"etag":"f5260089211aa13548b1"}},
  state="状態は変わらない", ic="合う", oc=["形：合う","機械の rule：無い","AI の rule：無い"]),
 dict(t="作る", uc="U5 インスタンスを作る", sem="semantic-create", mech="create",
  proc={"手順":"人が決める欄（status）は default（proposed）のまま。必須の欄（required）はすべて値を渡す（空の欄があると作らない）。id は create が決める","入力のスキーマ":"procedure#/$defs/create-request（作るパス ・ --schema の URN ・ --value の値。値は required から id を除いたもの。パスは x-instance-path に合わせる）","出力のスキーマ":"decision-record と create の出力（data.saved ・ results ・ state ・ etag ・ unfilled）"},
  inp={"path":P0199,"--schema":URN,"--value":V0},
  before={"記録":["0185","0150"]},
  after={"記録":["0185","0150","0199"],"0199":R0},
  out={"ok":True,"findings":[],"data":{"saved":True,"id":R0["id"],"path":P0199,"state":ST_WAIT,"etag":"b8b79df7a32c32d15601",
       "results":[{"rule":"level1-template","by":"machine","result":"verified","reason":"置き場所が x-instance-path に合う"},{"rule":"names-both","by":"ai","pointer":"/why","result":"unverified","reason":"AI の判定を依頼する（任せる先 reviewer）"},"…（ほかの rule）"],
       "unfilled":[{"name":"docs","title":"変更した文書と、変更の印","description":"この決定で変えた文書と、その印","rules":["why-per-change"]},{"name":"superseded_by","title":"置き換えた記録の番号","description":"この記録を置き換えた、新しい記録の番号","rules":[]},{"name":"supersedes","title":"置き換えた記録の番号","description":"この記録が置き換えた、前の記録の番号","rules":["target-exists"]}]}},
  state="インスタンスが1つ増える（id は create が決めた）。状態：判断待ち（AI の rule が unverified）。必須の欄が空なら作らない（exit 1、findings に「必須の欄が空：why」、data.saved:false）", ic="合う（必須の欄がそろう）", oc=["形：合う","機械の rule：level1-template が verified","AI の rule：names-both ・ noun ほかを unverified で返す（任せる先 reviewer）"]),
 dict(t="変える", uc="U6 既存のインスタンスを変える", sem="semantic-write", mech="patch",
  proc={"手順":"write の rule に従って書く。get で控えた etag を patch --if-match で渡す（読んだあとに中身が変わっていれば patch が断る）","入力のスキーマ":"JSON Patch（add ・ replace ・ remove。対象の欄だけ）","出力のスキーマ":"decision-record と patch の出力（data.saved ・ results ・ state ・ etag ・ unfilled）"},
  inp={"--patch":[{"op":"replace","path":"/why","value":WHY1}],"--if-match":"f5260089211aa13548b1"},
  before=R0, after=dict(R0, why=WHY1),
  out={"ok":True,"findings":[],"data":{"saved":True,"path":P0199,"state":ST_WAIT,"etag":"（書いたあとの中身から計算した新しい値）","results":["…（rule ごとの結果）"],"unfilled":["…"]}},
  state="欄が1つ変わる。新しく持ち込んだ機械の rule の違反が無いので保存する（data.saved:true）。状態：判断待ちのまま（names-both は unverified）", ic="合う（使える op だけ ・ ETag が合う）", oc=["形：合う","機械の rule：新しい違反は無い（あれば書き込みを断る：data.saved:false、exit 1。前からある違反は断らない）","AI の rule：names-both を unverified で返す"]),
 dict(t="判定する", uc="U10 取り直す ・ U16 判定を入れる", sem="基盤が起動する AI（coding-agents）", mech="validate（答えをまとめる）",
  proc={"手順":"基盤が、Validator の1段目が返した依頼（rule names-both の x-prompt と reads の値）を、coding-agents のアダプターで判定ごとに新しいセッションの AI に渡す。AI は rule ごとの結果 ・ 理由 ・ 根拠の場所を返す（ステップ4で実装。semantic-judge を残すかは未決）","入力のスキーマ":"Abstract の本体の $defs/output-judging（results：rule ・ result ・ reason ・ grounds）","出力のスキーマ":"judgment と検証の結果"},
  inp={"results":[{"rule":"names-both","result":"verified","reason":"食い違った相手（手で確かめる）と、合わせた先（規則ファイル）の両方がある","grounds":["/why"]}]},
  before={"0199 の判定":[],"unverified":["names-both"]},
  after={"0199 の判定":[{"rule":"names-both","結果":"verified","誰":"AI"}],"unverified":[]},
  state="判定の記録が1つ増える。names-both：unverified → verified", ic="合う", oc=["形：合う","機械の rule：合う","AI の rule：verified（この答え）"]),
 dict(t="置き換えて残す", uc="U11 不要なインスタンスを置き換えて残す", sem="semantic-write", mech="patch",
  proc={"手順":"置き換えた先を決め、status を superseded にする。指しているものはそのまま","入力のスキーマ":"JSON Patch（status ・ superseded_by）","出力のスキーマ":"decision-record と patch の出力","前提":"試験の ACDR の Concrete（S-CHANGE-APPROVE）は、acdr Skill の record.schema.json と同じ superseded_by の欄（置き換えた記録の番号）と「status が superseded なら superseded_by が必須」を持つ（コミット 9ef394f9）"},
  inp=[{"op":"replace","path":"/status","value":"superseded"},{"op":"add","path":"/superseded_by","value":"0199"}],
  before={"id":"0150","status":"accepted"}, after={"id":"0150","status":"superseded","superseded_by":"0199"},
  state="0150 は残り、置き換えた先を指す。0150 の判定は効かなくなる。0160（0150 を指す）は壊れない", ic="合う", oc=["形：合う","機械の rule：合う","AI の rule：無い"]),
 dict(t="取り込む", uc="U17 既存のファイルを取り込む", sem="semantic-import", mech="import",
  proc={"手順":"$schema が決まらないファイルの中身を見て、Concrete の候補から選ぶ","入力のスキーマ":"procedure#/$defs/import-request（パス ・ $schema）","出力のスキーマ":"選んだ Concrete（decision-record）"},
  inp={"path":"docs/old/0120.json","$schema":URN},
  before={"下書き":["docs/old/0120.json（$schema が無い）"],"記録":["0199"]},
  after={"下書き":[],"記録":["0199","0120"]},
  state="下書きがインスタンスになる。0120 の状態：unverified", ic="合う", oc=["形：合う（decision-record）","機械の rule：合う","AI の rule：unverified で返す"]),
 dict(t="移す", uc="U3 Abstract の新しい版に合わせる", sem="semantic-migrate", mech="patch",
  proc={"手順":"機械的な migrate が直せなかった欄だけを、移し方の案内に従って直す","入力のスキーマ":"JSON Patch（残った欄だけ）","出力のスキーマ":"abstract@2 に合う Concrete"},
  inp=[{"op":"replace","path":"/properties/owner/x-prompt/write/0/rule","value":"責任者を role と名前で書く"}],
  before={"$schema":"abstract@2","properties":{"owner":{"x-prompt":{"write":[{"id":"owner-format","rule":"責任者を書く"}]}}}},
  after={"$schema":"abstract@2","properties":{"owner":{"x-prompt":{"write":[{"id":"owner-format","rule":"責任者を role と名前で書く"}]}}}},
  state="この前に、機械的な migrate が $schema を abstract@1 → @2 にして、直せない欄を1件残した。この段で残りが 1件 → 0件", ic="合う", oc=["形：合う（abstract@2）","機械の rule：合う","AI の rule：無い"]),
]
for s in STEPS:
    s["before"] = J(s["before"]); s["after"] = s["before"] if s["after"] is None else J(s["after"]); s["inp"] = J(s["inp"]); s["out"] = J(s["out"]) if "out" in s else ""
KINDS = ['欄', 'インスタンス', 'スキーマ', 'ドキュメント', '集まり', '変更', '判定', 'セッション', '文脈']
KIND_NOTE = {'欄':'いちばん細かい単位', '集まり':'探す範囲 ・ 条件に合うもの', '変更':'change のまとまり（意図 ・ 目的）', '判定':'judgment', 'セッション':'目的のセッション', '文脈':'settings（探す範囲 ・ validator）'}
OPS = ['探す', '読む', '作る', '変える', '取り込む', '検証', '判定', '描画', '経緯', '版を出す', '移す', '閉じる']
SS, SR, SW = 'semantic-search', 'semantic-read', 'semantic-write'
M = {
 ('欄','読む'):[SR,'get'], ('欄','変える'):[SW,'patch'],
 ('インスタンス','探す'):[SS,'search'], ('インスタンス','読む'):[SR,'get'], ('インスタンス','作る'):['semantic-create','create'],
 ('インスタンス','変える'):[SW,'patch'], ('インスタンス','検証'):['','validate'],
 ('インスタンス','判定'):['','validate（基盤が AI を起動する）'], ('インスタンス','描画'):['','render'], ('インスタンス','経緯'):['','history'],
 ('スキーマ','探す'):[SS,'search'], ('スキーマ','読む'):[SR,'get'], ('スキーマ','作る'):['semantic-create','create'],
 ('スキーマ','変える'):[SW,'patch'], ('スキーマ','検証'):['','validate'], ('スキーマ','版を出す'):['','release'], ('スキーマ','移す'):['semantic-migrate','migrate ・ patch'],
 ('ドキュメント','探す'):[SS,'search'], ('ドキュメント','読む'):[SR,''], ('ドキュメント','変える'):['','record'],
 ('ドキュメント','取り込む'):['semantic-import','import'], ('ドキュメント','経緯'):['','history'],
 ('集まり','探す'):[SS,'search'], ('集まり','検証'):['','validate'],
 ('変更','作る'):['','record'], ('変更','取り込む'):['','validate .'], ('変更','判定'):['','judge（人）'], ('変更','経緯'):['','history'],
 ('判定','検証'):['','validate'],
 ('セッション','作る'):['','session open'], ('セッション','読む'):['','session resume'], ('セッション','閉じる'):['','session close'],
 ('文脈','変える'):[SW,'patch'],
}
# 段：[題, 使う場面, 呼び出し [誰, コマンド, 対象の種類, 操作, 何を, 返す型]]
FSTEPS = [
 ["目的のセッションを立てる", ["U27"], [
   ["人","session open","セッション","作る","目的「依存の向きの検査ツールを作る」","セッション（ID）"]]],
 ["探す範囲を宣言し、validator をつなぐ", ["U26","U14"], [
   ["AI","semantic-write","文脈","変える","settings の探す範囲と validator","procedure"],
   ["基盤","patch","文脈","変える","settings への JSON Patch","data.results（判定できない rule は unsupported）"]]],
 ["過去の決定と、関係するソースを探す", ["U28"], [
   ["AI","semantic-search","集まり","探す","問い「依存の向きを決めた記録と、そこで変えたファイルは？」","procedure"],
   ["基盤","search","インスタンス","探す","記録に where（JMESPath）","query"],
   ["基盤","search","ドキュメント","探す","変更の記録から、変えたファイルのパス","query"]]],
 ["見つけた記録とソースを読む", ["U13"], [
   ["AI","semantic-read","インスタンス","読む","ACDR 0185","procedure"],
   ["基盤","get","インスタンス","読む","ACDR 0185","data.value ・ etag"],
   ["AI","semantic-read","ドキュメント","読む","judge.rs","procedure（読み方の案内）"],
   ["基盤","history","ドキュメント","経緯","judge.rs を変えた記録と意図","change の並び"],
   ["AI","（ファイルを読む）","ドキュメント","読む","judge.rs の本文","基盤は中身を解釈しない"]]],
 ["ほかの文脈の公開された言葉を読む", ["U24"], [
   ["AI","semantic-read","スキーマ","読む","published の glossary（x-relationship の先）","procedure"],
   ["基盤","get","スキーマ","読む","published の glossary のパスと /terms","data.value ・ etag"]]],
 ["記録の型を読む", ["U5"], [
   ["AI","semantic-read","スキーマ","読む","Concrete decision-record","procedure"],
   ["基盤","get","スキーマ","読む","Concrete のパス（pointer は空）","data.value ・ etag（$ref は解かない）"]]],
 ["記録を作り、欄を書く", ["U5"], [
   ["AI","semantic-create","インスタンス","作る","decision-record の新しい記録","procedure（書く順 ・ 人が決める欄）"],
   ["基盤","create","インスタンス","作る","ACDR 0199（必須の欄をそろえて渡す）","data.saved ・ results ・ state ・ etag ・ unfilled"],
   ["AI","semantic-write","欄","変える","0199 の /decision","procedure（write の rule）"],
   ["基盤","patch","欄","変える","JSON Patch と --if-match（create が返した etag）","data.saved ・ results ・ state ・ etag"],
   ["基盤","record","変更","作る","0199 の変えた欄 ・ 意図 ・ 目的（--intent ・ --purpose は必須）","変更の記録（手元の DB）"]]],
 ["検証して、AI と人が rule と意図を判定する", ["U16"], [
   ["基盤","validate","インスタンス","検証","0199","data.results ・ state（AI の rule は unverified）"],
   ["基盤","（AI の判定を依頼する）","インスタンス","判定","0199 × rule decision-no-means：coding-agents のアダプターで、判定ごとに新しいセッションの AI を起動する（ステップ4）","AI の答え"],
   ["基盤","validate","インスタンス","判定","AI の答えをまとめる（Validator の2段目）","judgment ・ 検証の結果"],
   ["人","judge","変更","判定","意図と変更が合っているか（I33）。人が決める欄を変えた記録なら、承認すると human-entry-only が verified","judgment"]]],
 ["判定を受けて、記録の2つの欄をまとめて直す", ["U6"], [
   ["AI","semantic-write","インスタンス","変える","0199（/why と /alternatives）","procedure"],
   ["基盤","patch","インスタンス","変える","2つの op を持つ JSON Patch と --if-match","data.saved ・ results ・ state ・ etag"]]],
 ["rule を直し、使っている所を探して直す", ["U2","U23","U7"], [
   ["AI","semantic-search","スキーマ","探す","問い「rule owner-required を使う Concrete は？」","procedure"],
   ["基盤","search","スキーマ","探す","Concrete に where","query"],
   ["AI","semantic-read","欄","読む","Concrete の /properties/owner/x-prompt","procedure"],
   ["基盤","get","欄","読む","同じ欄","data.value ・ etag"],
   ["AI","semantic-write","スキーマ","変える","Concrete decision-record","procedure"],
   ["基盤","patch","スキーマ","変える","JSON Patch","data.saved ・ results ・ state ・ etag"],
   ["基盤","validate","スキーマ","検証","Concrete を Abstract に照らす","data.results ・ state"],
   ["基盤","validate","集まり","検証","validate .（作業の根の下のすべて）","data.files（パスごとの状態）・ findings"],
   ["AI","semantic-write","インスタンス","変える","violated になった記録","procedure"]]],
 ["効かなくなった判定を取り直す", ["U10"], [
   ["基盤","validate","判定","検証","入力が変わった rule だけ","data.results ・ state"]]],
 ["ソースを直して、意図と結びつける", ["U13"], [
   ["AI","（ファイルを直す）","ドキュメント","変える","architecture-lint/src/cargo.rs","基盤は中身を解釈しない"],
   ["基盤","record","ドキュメント","変える","cargo.rs ・ 意図 ・ 目的（0199）（--intent ・ --purpose は必須）","変更の記録（手元の DB に1件）"]]],
 ["手元の JSON を取り込む", ["U17"], [
   ["基盤","import","ドキュメント","取り込む","docs/old-decisions/*.json","インスタンス ・ 下書きの一覧"],
   ["AI","semantic-import","ドキュメント","取り込む","$schema が決まらない下書き 0120.json","procedure（Concrete の候補 ・ 入力のスキーマ）"],
   ["基盤","import","ドキュメント","取り込む","0120.json と選んだ $schema","インスタンス"]]],
 ["不要になった記録を置き換えて残す", ["U11"], [
   ["AI","semantic-write","インスタンス","変える","0150 の status を superseded、superseded_by を 0199","procedure"],
   ["基盤","patch","インスタンス","変える","JSON Patch","data.saved ・ results ・ state（0150 の判定は効かなくなる。0160 は壊れない）"]]],
 ["翌日、同じ目的のセッションを ID で再開する", ["U27"], [
   ["人","session resume","セッション","読む","前の日のセッション ID","セッション（続きから）"]]],
 ["新しい記録の型を作る", ["U1"], [
   ["AI","semantic-create","スキーマ","作る","Abstract から Concrete tool-record","procedure"],
   ["基盤","create","スキーマ","作る","Concrete の雛形（--owner ・ --kind ・ --value に title と description）","Concrete（$id は urn:semantic-agent-schema:&lt;owner&gt;:&lt;kind&gt;:v1）"]]],
 ["Abstract の新しい版を出し、Concrete を合わせる", ["U4","U3"], [
   ["提供側","release","スキーマ","版を出す","Abstract の新しい版と移し方","版 ・ 移し方"],
   ["基盤","migrate","スキーマ","移す","Concrete decision-record","直せなかった欄の一覧"],
   ["AI","semantic-migrate","スキーマ","移す","直せなかった欄 /properties/owner","procedure（移し方の案内 ・ Patch の入力のスキーマ）"],
   ["基盤","migrate ・ patch","スキーマ","移す","残った欄への JSON Patch","data.saved ・ results ・ state"]]],
 ["人が読み、変更を確かめる", ["U12","U13","U25"], [
   ["基盤","render","インスタンス","描画","0199 と view","HTML"],
   ["基盤","history","インスタンス","経緯","0199","change ・ judgment の並び"],
   ["基盤","history","変更","経緯","0199 を変えた変更の記録と意図の判定","data.changes"],
   ["人","record --summary","変更","作る","前回のコミットからの変更（いまのワークツリーとブランチの記録）","まとめ1件（.semantic-agent-schema/changes/<ULID>.json。コミットする）"]]],
 ["ほかの変更を取り込み、CI で全部を検証する", ["U18","U8","U0"], [
   ["Git","（取り込む）","変更","取り込む","チームの変更","基盤の外"],
   ["基盤","validate .","変更","取り込む","取り込んだあとの姿（作業の根の下のすべて）","data.files（パスごとの状態）・ findings"],
   ["CI","validate .","集まり","検証","作業の根の下のすべての宣言とインスタンス","data.files ・ findings"]]],
 ["セッションを閉じる", ["U27"], [
   ["人","session close","セッション","閉じる","目的のセッション","セッション（閉じた）"]]],
]
from states import ST
assert len(ST)==len(FSTEPS)
import json as _j
UC = [[u[0], u[2]] for u in _j.load(open(os.path.join(C, 'model', 'model.json')))['use_cases']]
MECH_ONLY = [
 ['validate','状態を検査する。入力は無い（対象のパスだけ。. なら作業の根の下のすべて）','findings と data.results（rule ごとの result）・ data.state（name ・ label）。validate . は data.files（パス ・ problems ・ state）。保存しない'],
 ['render','インスタンスと view','HTML（状態は変わらない）'],
 ['history','パス','data.changes（変更の記録 ・ 意図の判定 intent_judgment の並び）'],
 ['release','Abstract（提供側が書いたもの）','版 ・ 移し方'],
 ['record','パス ・ --pointer ・ --intent ・ --purpose（意図と目的は必須。意図が空なら refused）・ --session','変更の記録1件（手元の DB）'],
 ['record --summary','（無い。前回のコミットからの変更と、いまのワークツリーとブランチの記録を使う）','まとめ1件（.semantic-agent-schema/changes/<ULID>.json）。コミットしていないまとめがあれば断る'],
 ['session','目的 ・ 役割','セッション（open ・ resume ・ close）'],
 ['judge','変更の記録の id ・ --result ・ --meaning ・ --reason（人の判定：意図 I33 と、人が決める欄の承認）','judgment（data.by が human）'],
]
data = json.dumps({"S": STEPS, "K":KINDS,"KN":KIND_NOTE,"O":OPS,"M":{f"{k}|{o}":v for (k,o),v in M.items()},"F":FSTEPS,"ST":ST,"U":UC}, ensure_ascii=False)
FLOW = '''<div class="opflow">
 <div class="opbox ost"><b>事前状態</b><span>JSON</span></div><div class="oparw">→</div>
 <div class="opbox osm"><b>semantic-○○</b><span>procedure を返す<br>手順 ・ 入力のスキーマ ・ 出力のスキーマ</span></div><div class="oparw">→</div>
 <div class="opbox oai"><b>AI が入力を生成する</b><span>入力のスキーマに合う JSON</span><em>検査①　入力が型どおりか</em></div><div class="oparw">→</div>
 <div class="opbox omc"><b>機械的に適用</b><span>search ・ get ・ create ・ patch ・ import ・ migrate ・ validate</span></div><div class="oparw">→</div>
 <div class="opbox ost"><b>事後状態 ・ 出力</b><span>変わった所だけが違う。出力は別の枠</span><em>検査②　出力が型どおりか（validate）</em></div>
</div>'''
body = f'''<main>
<h1>コマンドのシミュレーション</h1>
<p class="ans">AI が行う操作は、すべてセマンティック。セマンティックのコマンドは procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）を返す。AI が生成するのは入力だけで、出力は機械的なコマンドが作り、型どおりかを validate が見る</p>

<h2>1　1つの操作の流れ</h2>
{FLOW}
{table(['問い','答え'], [
 ['出力は機械的に作るか','作る。AI は入力（Patch ・ 条件 ・ 判定）だけを生成し、状態に適用するのは機械的なコマンド'],
 ['入力を機械的に適用したら、出力は型どおりか','そうとは限らない。だから検査が2回ある ── ①入力が入力のスキーマに合うか（適用する前）②出力が形 ・ 機械の rule ・ AI の rule に合うか（適用した後。AI の rule は unverified で返る）'],
 ['検査で外れたら','①で外れた入力は断り、状態に触れない。②で新しく持ち込んだ機械の rule の違反があれば、書き込みを断る（data.saved:false、exit 1）。前からある違反 ・ AI と人の rule は断らず、状態として返す'],
 ['状態を変える操作の入力は何か','事前状態に足す ・ 変える ・ 取り除く分だけ（Patch の add ・ replace ・ remove）。作るは新しい文書の値、判定は判定1件が入力になる'],
 ['状態を変えない操作（探す ・ 読む）は','事後状態は事前状態と同じ。出力（結果）だけが増え、それも出力のスキーマで検査する'],
])}

<h2>2　目的に沿った流れ：「依存の向きの検査ツールを作る」</h2>
<p class="q">「次 →」で1段ずつ進める。段ごとに、事前状態 → 入力 → 事後状態と、使うコマンド、表のセル、通った使う場面が出る。20段で、model.json の使う場面がすべて表に載る</p>
<div class="sim">
 <div class="ctl"><button id="fprev">← 前</button><span id="fpos"></span><button id="fnext">次 →</button><button id="fall">全部の段</button></div>
 <div class="stp" id="fstate"></div>
 <div id="fstep"></div>
 <div class="tw"><table class="mx" id="mx"></table></div>
 <div class="lg"><span><i class="sw now"></i>この段で使う</span><span><i class="sw done"></i>前の段で使った</span><span><i class="sw idle"></i>まだ使っていない</span><span><span class="c s">セマンティック</span></span><span><span class="c m">機械的</span></span><span>セマンティックが無いセル：AI が行わない操作</span></div>
 <div id="uc"></div>
</div>

<h2>3　セマンティックの操作ごとの procedure</h2>
<div class="sim">
 <div class="tabs" id="tabs"></div>
 <div id="sb"></div>
</div>

<h2>4　コマンドの一覧</h2>
<p class="meta">セマンティック（AI が行う。procedure を返す）と、それを適用する機械的なコマンドの組</p>
{table(['操作','セマンティック','適用する機械的なコマンド','入力のスキーマ','出力のスキーマ'], [[s['t'], s['sem'], s['mech'], s['proc']['入力のスキーマ'], s['proc']['出力のスキーマ']] for s in STEPS])}
<p class="meta">機械的だけのコマンド（AI が行わない操作）</p>
{table(['コマンド','入力','出力'], MECH_ONLY)}

<h2>5　決めたこと</h2>
{table(['決めたこと','理由'], [
 ['AI が行う操作は、すべてセマンティックのコマンドにする','AI が関わるかどうかで、セマンティックと機械的を分ける'],
 ['セマンティックのコマンドは procedure（手順 ・ 入力のスキーマ ・ 出力のスキーマ）を返す。procedure は Abstract のファイルに足す','AI が生成する入力と、機械的に作る出力を、どちらも型で検査できる'],
 ['セマンティックは、入力のスキーマが違う操作ごとに別のコマンドにする（7つ。semantic-judge を残すかは未決）','返す型は同じ procedure でも、入力が操作ごとに違う。意図は record が --intent で受け取り、Patch では書かない'],
 ['検査は2回。①適用する前に入力を見る ②適用した後に出力を見る','①で外れた入力は断り、状態に触れない。②で新しく持ち込んだ機械の rule の違反は書き込みを断る（data.saved:false）。前からある違反 ・ AI と人の rule は断らない'],
 ['AI の rule の判定は、基盤が coding-agents のアダプターで判定ごとに新しいセッションの AI を起動して依頼し、答えを Validator の2段目に渡す（ステップ4）。judge は人の判定だけ','domain は AI を起動しない。起動はポート「rule の判定を依頼する（AI）」のアダプターに置く（CODING.md）'],
 ['楽観ロックは ETag。get が data.etag を返し、patch --if-match で渡す。JSON Patch の op は add ・ replace ・ remove だけ','ETag は文書の SHA-256 の先頭20字で、保存しない'],
 ['消すを無くす。不要になったものは superseded にして残す。項目を消すのは変える（Patch の remove）','消すでしか済まない場面が無く、消すと指している記録が壊れる'],
 ['確定していない変更の取り消しは Git の側でする。基盤は戻った姿を validate するだけ','基盤は変更の仕組みを持たない'],
])}
<p class="meta">U11（不要になったインスタンスを置き換えて残す）・ I11（superseded のインスタンスに結び付いた判定の記録は効かない）・ U28（Semantic Search）・ boundary の query ・ procedure は、ACDR 0201 で model.json に適用済み。query ・ procedure の Abstract のファイルはまだ無い。0150 ・ 0160 ・ 0120 ・ 0210 は架空。セマンティックのコマンド（procedure を返す）・ search ・ import ・ migrate ・ render ・ session ・ release はまだ実装に無い。2節のカードの decision-record@1 ・ abstract@1 は略記で、URN は urn:semantic-agent-schema:acdr:decision-record:v1 など。3節の出力の例は、実装の試験のシナリオを基盤のコマンドで流した出力から取った</p>
</main>
<style>
main>*,.sim>*{{min-width:0;max-width:100%}}
.opflow{{display:flex;align-items:stretch;gap:6px}}
.opbox{{flex:1 1 0;min-width:0;overflow-wrap:anywhere;border-radius:10px;padding:10px;display:flex;flex-direction:column;gap:4px;font-size:13px;border:1.5px solid var(--line);background:var(--card)}}
.opbox b{{font-size:14px}} .opbox span{{color:var(--sub)}} .opbox em{{font-style:normal;font-size:12px;font-weight:700;color:var(--hot)}}
.opbox.osm{{border-color:var(--accent)}} .opbox.oai{{border-color:var(--accent);background:var(--bg)}} .opbox.omc{{background:var(--bg)}}
.oparw{{align-self:center;color:var(--accent);font-size:20px}}
@media (max-width:700px){{.opflow{{flex-direction:column}} .oparw{{transform:rotate(90deg)}}}}
.sim{{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:12px;display:flex;flex-direction:column;gap:12px}}
.tabs{{display:flex;gap:6px;flex-wrap:wrap}}
.tabs button{{font:inherit;font-size:13px;padding:4px 12px;border-radius:999px;border:1px solid var(--line);background:var(--bg);color:var(--ink);cursor:pointer}}
.tabs button.on{{background:var(--accent);color:var(--card);border-color:var(--accent)}}
.row3{{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,1.1fr) minmax(0,1fr);gap:10px}}
@media (max-width:860px){{.row3{{grid-template-columns:1fr}}}}
.pn{{border:1px solid var(--line);border-radius:10px;padding:8px;display:flex;flex-direction:column;gap:6px;min-width:0;background:var(--bg)}}
.pn h4{{margin:0;font-size:13px}}
.pn pre{{margin:0;font-family:var(--mono);font-size:11.5px;line-height:1.5;white-space:pre-wrap;overflow-wrap:anywhere;background:var(--card);border-radius:6px;padding:6px}}
.pn pre .dadd{{background:var(--hot-bg);color:var(--hot);font-weight:700;display:inline;white-space:pre-wrap}}
.pn pre .ddel{{opacity:.5;text-decoration:line-through;display:inline;white-space:pre-wrap}}
.kv{{font-size:12.5px;display:grid;grid-template-columns:auto minmax(0,1fr);gap:2px 8px}}
.kv b{{color:var(--sub);font-weight:700}}
.cmd{{font-family:var(--mono);font-weight:700}}
.ok{{font-size:12.5px;font-weight:700;color:var(--accent)}}
.stt{{font-weight:700;margin:0;padding:6px 10px;border-radius:8px;background:var(--hot-bg);color:var(--ink)}}
.yes{{font-weight:700;color:var(--hot);margin:0}} .no{{font-weight:700;color:var(--accent);margin:0}} .pn p{{margin:0;font-size:13px}} .find{{font-weight:700}}
.ctl{{display:flex;gap:8px;align-items:center;flex-wrap:wrap}}
.ctl button,.lane button{{font:inherit;font-size:12.5px;padding:2px 12px;border-radius:999px;border:1px solid var(--line);background:var(--bg);color:var(--ink);cursor:pointer}}
#fpos{{font-weight:700}}
.lane{{display:grid;grid-template-columns:3.2em minmax(0,1fr);gap:4px 10px;align-items:start;padding:6px 0;border-top:1px solid var(--line);font-size:13.5px;overflow-wrap:anywhere}}
.who{{font-weight:700;color:var(--accent)}}
.lane .tg{{color:var(--sub);font-size:12.5px;grid-column:2}}
.c{{font-family:var(--mono);font-size:12.5px;padding:1px 8px;border-radius:999px;white-space:nowrap}}
.c.s{{border:1.5px solid var(--accent);color:var(--accent);background:var(--card)}}
.c.m{{background:var(--accent);color:var(--card)}}
.c.u{{border:1.5px dashed var(--hot);color:var(--hot);background:var(--card);font-family:inherit}}
table.mx{{font-size:12px}}
table.mx td,table.mx th{{text-align:center;padding:4px;vertical-align:top}}
table.mx th:first-child{{text-align:left;white-space:nowrap}}
table.mx th small{{display:block;font-weight:400;color:var(--sub);font-size:10.5px;white-space:normal;max-width:9em}}
table.mx td.idle{{opacity:.35}}
table.mx td.done{{background:var(--bg)}}
table.mx td.now{{outline:2.5px solid var(--hot);outline-offset:-2px;background:var(--hot-bg)}}
table.mx td .c{{display:block;margin:2px 0;font-size:11px;padding:0 6px;white-space:normal;word-break:keep-all;overflow-wrap:normal}}
.lg{{display:flex;flex-wrap:wrap;gap:12px;font-size:12px;color:var(--sub);align-items:center}}
.sw{{display:inline-block;width:14px;height:14px;border-radius:3px;vertical-align:-2px;margin-right:4px;border:1px solid var(--line)}}
.sw.now{{background:var(--hot-bg);outline:2px solid var(--hot)}} .sw.done{{background:var(--bg)}} .sw.idle{{background:var(--card);opacity:.4}}
.ucs{{display:flex;flex-wrap:wrap;gap:6px}}
.ucs span{{font-size:12px;padding:2px 8px;border-radius:8px;border:1px solid var(--line);background:var(--bg);opacity:.45}}
.ucs span.hit{{opacity:1;border-color:var(--accent);color:var(--ink)}}
.ucs span.cur{{opacity:1;outline:2px solid var(--hot);background:var(--hot-bg)}}
#ucsum{{font-weight:700;margin:6px 0 0}}
.lb{{font-size:12px;font-weight:700;color:var(--sub)}}
.stp{{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,.7fr) minmax(0,1fr);gap:10px;align-items:start}}
@media (max-width:760px){{.stp{{grid-template-columns:1fr}} .sa .arr{{transform:rotate(90deg)}}}}
.sp{{display:flex;flex-direction:column;gap:8px;min-width:0}} .sp h4{{margin:0;font-size:13px}}
.sa{{display:flex;flex-direction:column;gap:6px;align-self:center;min-width:0;font-size:12px;color:var(--sub)}}
.sa code{{font-family:var(--mono);font-size:11px;background:var(--bg);border:1px dashed var(--accent);border-radius:8px;padding:6px;white-space:pre-wrap;overflow-wrap:anywhere;color:var(--ink)}}
.sa .arr{{font-size:22px;color:var(--accent);align-self:center}}
.cd{{border:1.5px solid var(--line);border-radius:10px;background:var(--bg);padding:6px 8px;display:flex;flex-direction:column;gap:3px;min-width:0}}
.cd.new{{border-color:var(--hot);background:var(--hot-bg)}}
.cd.gone{{opacity:.5;border-style:dashed}}
.cd.k-出力{{border-style:dashed;background:var(--card)}}
.ch{{display:flex;flex-wrap:wrap;gap:6px;align-items:center;font-size:13px}}
.ck2{{font-size:11px;font-weight:700;color:var(--sub);border:1px solid var(--line);border-radius:6px;padding:0 5px}}
.ct{{font-family:var(--mono);font-size:11px;color:var(--sub)}}
.bd{{font-size:11px;font-weight:700;border-radius:999px;padding:0 8px;border:1.5px solid var(--line)}}
.bd.verified,.bd.open,.bd.確定,.bd.承認済み,.bd.効く{{border-color:var(--accent);color:var(--accent)}}
.bd.violated{{border-color:var(--hot);color:var(--hot)}}
.bd.was{{opacity:.55}}
.ar{{color:var(--sub);font-size:12px}}
.fr{{display:grid;grid-template-columns:auto minmax(0,1fr);gap:8px;font-size:12px;border-radius:4px;padding:0 4px}}
.fk{{color:var(--sub);font-family:var(--mono)}} .fv{{overflow-wrap:anywhere}}
.fr.chg{{background:var(--hot-bg);font-weight:700}}
.fr s{{display:block;font-weight:400;opacity:.55}}
.nt2{{font-size:11px;font-weight:700;color:var(--hot)}}
.none{{font-size:12px;color:var(--sub);margin:0}}
.ck{{margin:0;padding-left:1.2em;font-size:12.5px}}
</style>
<script>
const D={data};
function esc(s){{return String(s).replace(/[&<>]/g,c=>({{'&':'&amp;','<':'&lt;','>':'&gt;'}}[c]))}}
function diff(a,b){{const N=l=>l.replace(/,\\s*$/,''),A=a.split('\\n'),B=b.split('\\n'),sa=new Set(A.map(N)),sb=new Set(B.map(N));
 return [A.map(l=>sb.has(N(l))?esc(l):'<span class="ddel">'+esc(l)+'</span>').join('\\n'), B.map(l=>sa.has(N(l))?esc(l):'<span class="dadd">'+esc(l)+'</span>').join('\\n')]}}
const tabs=document.getElementById('tabs');
D.S.forEach((s,i)=>{{const b=document.createElement('button');b.textContent=(i+1)+' '+s.t;b.onclick=()=>show(i);tabs.appendChild(b)}});
function show(i){{const s=D.S[i];const [bf,af]=diff(s.before,s.after);
 let p='<div class="kv">'+Object.entries(s.proc).map(([k,v])=>'<b>'+esc(k)+'</b><span>'+esc(v)+'</span>').join('')+'</div>';
 document.getElementById('sb').innerHTML='<p class="meta">使う場面　'+esc(s.uc)+'</p><p class="stt">'+esc(s.state)+'</p><div class="row3">'+
 '<div class="pn"><h4>事前状態</h4><pre>'+bf+'</pre></div>'+
 '<div class="pn"><h4><span class="cmd">'+esc(s.sem)+'</span> が返す procedure</h4>'+p+'<h4>AI が生成した入力</h4><pre>'+esc(s.inp)+'</pre><span class="ok">検査①　'+esc(s.ic)+'</span><h4>機械的に適用：<span class="cmd">'+esc(s.mech)+'</span></h4></div>'+
 '<div class="pn"><h4>事後状態</h4><pre>'+af+'</pre>'+(s.out?'<h4>出力</h4><pre>'+esc(s.out)+'</pre>':'')+'<span class="ok">検査②</span><ul class="ck">'+s.oc.map(x=>'<li>'+esc(x)+'</li>').join('')+'</ul></div></div>';
 [...tabs.children].forEach((b,j)=>b.classList.toggle('on',j===i))}}
show(3);
const DET={{'semantic-search':0,'semantic-read':1,'semantic-create':2,'semantic-write':3,'（AI の判定を依頼する）':4,'semantic-import':6,'semantic-migrate':7}};
function chip(c){{return c.startsWith('semantic')?'s':c.startsWith('（')?'u':'m'}}

const vv=v=>v===''?'（空）':v;
function cards(list, other, side){{
 if(!list.length)return '<p class="none">（まだ無い）</p>';
 return list.map(c=>{{const o=other.find(x=>x.n===c.n&&x.k===c.k);
  const isNew=side==='post'&&!o, gone=side==='pre'&&!o;
  let st=c.s?'<span class="bd '+c.s+'">'+esc(c.s)+'</span>':'';
  if(side==='post'&&o&&o.s!==c.s&&c.s)st='<span class="bd '+o.s+' was">'+esc(o.s)+'</span><span class="ar">→</span>'+st;
  let rows=Object.entries(c.f).map(([k,v])=>{{const ov=o?o.f[k]:undefined;const chg=side==='post'&&o&&ov!==v;
   return '<div class="fr'+(chg?' chg':'')+'"><span class="fk">'+esc(k)+'</span><span class="fv">'+esc(vv(v))+(chg&&ov!==undefined?'<s>'+esc(vv(ov))+'</s>':'')+'</span></div>'}}).join('');
  return '<div class="cd k-'+c.k+(isNew?' new':'')+(gone?' gone':'')+'"><div class="ch"><span class="ck2">'+esc(c.k)+'</span><b>'+esc(c.n)+'</b><span class="ct">'+esc(c.t)+'</span>'+st+'</div>'+rows+(isNew?'<div class="nt2">新しく増えた</div>':'')+(gone?'<div class="nt2">無くなった</div>':'')+'</div>'}}).join('')}}
function fstate(i){{const s=D.ST[i];
 document.getElementById('fstate').innerHTML='<div class="sp"><h4>事前状態</h4>'+cards(s.pre,s.post,'pre')+'</div><div class="sa"><span>入力</span><code>'+esc(s.inp)+'</code><span class="arr">→</span></div><div class="sp"><h4>事後状態</h4>'+cards(s.post,s.pre,'post')+'</div>'}}
let fc=0, fallOn=false;
function fdraw(){{
 const used=new Set(), now=new Set(), uHit=new Set(), uNow=new Set();
 const upto=fallOn?D.F.length-1:fc;
 D.F.forEach((s,i)=>{{if(i>upto)return;const isNow=(i===upto&&!fallOn);
  s[1].forEach(u=>(isNow?uNow:uHit).add(u));
  s[2].forEach(c=>{{const k=c[2]+'|'+c[3];if(D.M[k])(isNow?now:used).add(k)}})}});
 let h='<tr><th></th>'+D.O.map(o=>'<th>'+o+'</th>').join('')+'</tr>';
 D.K.forEach(k=>{{h+='<tr><th>'+k+(D.KN[k]?'<small>'+esc(D.KN[k])+'</small>':'')+'</th>';D.O.forEach(o=>{{const v=D.M[k+'|'+o];if(!v){{h+='<td></td>';return}}
  const key=k+'|'+o, cls=now.has(key)?'now':used.has(key)?'done':'idle';
  h+='<td class="'+cls+'">'+(v[0]?'<span class="c s">'+v[0]+'</span>':'')+(v[1]?'<span class="c m">'+v[1]+'</span>':'')+'</td>'}});h+='</tr>'}});
 document.getElementById('mx').innerHTML=h;
 if(fallOn){{document.getElementById('fpos').textContent='全部の段';document.getElementById('fstep').innerHTML='<p class="meta">'+D.F.length+'の段を全部進めたところ</p>';document.getElementById('fstate').innerHTML='';}}
 else{{fstate(fc);const s=D.F[fc];document.getElementById('fpos').textContent=(fc+1)+' / '+D.F.length+'　'+s[0];
  let t='';s[2].forEach(c=>{{const d=DET[c[1]];t+='<div class="lane"><span class="who">'+esc(c[0])+'</span><span><span class="c '+chip(c[1])+'">'+esc(c[1])+'</span>　'+esc(c[4])+(d!==undefined?'　<button data-d="'+d+'">事前と事後を見る</button>':'')+'</span><span class="tg">対象：'+esc(c[2])+' ／ '+esc(c[3])+'　→ 返す型：'+esc(c[5])+'</span></div>'}});
  document.getElementById('fstep').innerHTML=t;
  document.querySelectorAll('#fstep button[data-d]').forEach(b=>b.onclick=()=>{{show(+b.dataset.d);document.getElementById('tabs').scrollIntoView({{behavior:'smooth',block:'start'}})}});}}
 const all=new Set([...uHit,...uNow]);
 let u='<div class="lb">使う場面（model.json）── 通った場面が濃くなる</div><div class="ucs">'+D.U.map(x=>'<span class="'+(uNow.has(x[0])?'cur':all.has(x[0])?'hit':'')+'">'+x[0]+' '+esc(x[1])+'</span>').join('')+'</div>';
 const miss=D.U.filter(x=>!all.has(x[0]));
 const cells=Object.keys(D.M).length, usedN=new Set([...used,...now]).size;
 u+='<p id="ucsum">使う場面 '+D.U.length+' のうち、通ったもの '+(D.U.length-miss.length)+' ／ 表のセル '+cells+' のうち、使ったもの '+usedN+(fallOn?(miss.length?'。表に載らない場面：'+miss.map(x=>x[0]).join(' ・ '):'。全部の場面が表に載った'):'')+'</p>';
 document.getElementById('uc').innerHTML=u;
}}
document.getElementById('fprev').onclick=()=>{{fallOn=false;fc=Math.max(0,fc-1);fdraw()}};
document.getElementById('fnext').onclick=()=>{{fallOn=false;fc=Math.min(D.F.length-1,fc+1);fdraw()}};
document.getElementById('fall').onclick=()=>{{fallOn=true;fdraw()}};
fdraw();

</script>'''
open(os.path.join(H, 'index.html'), 'w').write(head + '\n' + body); print('ok')
