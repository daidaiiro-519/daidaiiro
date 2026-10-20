# 第35回：利用者の指摘（Q1 が薄い。抽象の層の概念 ・ モデル ・ スキーマを、いまの実装と構想 v0.3 の両方から整理する）で、
# 抽象の層を Q1 抽象の概念モデル ・ Q6 抽象スキーマの構成 ・ Q7 抽象スキーマの中身 に分けて組み直す。
# 材料：advisors/ddd-abstract-model.json と .data.json（ddd-advisor）、measured/v03-concepts.json、measured/base-schemas.json。
import json, pathlib, sys
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
import subprocess
# 組み直す前のボード（第34回のコミット）から組む。何度流しても同じ結果になるようにする
board = json.loads(subprocess.run(["git", "show", "a55fdade:.brainstorming/semantic-contract-core.json"], capture_output=True, text=True, check=True).stdout)
adv = json.loads((BOARD / "advisors/ddd-abstract-model.json").read_text())
data = json.loads((BOARD / "advisors/ddd-abstract-model.data.json").read_text())
v03 = json.loads((BOARD / "measured/v03-concepts.json").read_text())
base = json.loads((BOARD / "measured/base-schemas.json").read_text())
svg = lambda n: (HERE / f"{n}.svg").read_text()
user = lambda s, w: {"supports": s, "basis": w, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
ADV = "ddd-advisor の回答（2026-10-17、抽象の概念モデル）。advisors/ddd-abstract-model.json"
def vq(concept_word, at):
    for it in v03["items"]:
        for q in it["quotes"]:
            if q["at"] == at: return {"supports": f"構想 v0.3 の {concept_word}", "basis": q["text"], "tag": "assumption", "source": f"構想 v0.3（利用者が Codex でブレストした検討中の案）{at}"}
    sys.exit(f"引用が無い：{at}")
T = {t["id"]: t for t in board["topics"]}
old = T["Q1"]
old_images = {im.get("name"): im for im in old["images"]}
old_grounds = {g["supports"]: g for g in old["grounds"]}
old_details = old["answer"]["details"]

q1 = {
 "id": "Q1", "name": "抽象の概念モデル", "status": "open", "depends": ["Q0", "Q5"],
 "question": "Semantic Schema-Driven 基盤の抽象の層を、どの概念で組み立て、概念どうしはどう関係するか（いまの実装と構想 v0.3 の両方から）",
 "answer": {
  "text": "抽象の層は、宣言 ・ 使う側 ・ 検証 ・ 描画の契約 ・ 版と記録の5つの群、27の概念で組み立てる。x- キーワードは Abstract スキーマだけが定義し、Semantic Concrete Schema は値を書くだけにする。検証は Semantic Validator が rule ごとに validator を選んで行い、validation result で返す。",
  "details": [
   "宣言：Abstract schema は、domain schema（JSON Schema）に x- キーワードの定義と検証の種類（機械的 ／ セマンティック）を足した契約。x- キーワードの表を1か所に持つ",
   "使う側：Concrete schema は Skill が書き、instance ・ prompt ・ rule ・ vocabulary と term ・ relation ・ derived value を使う。x- キーワードは定義しない",
   "検証：Semantic Validator が rule ごとに validator（Schema Validator ・ 基盤のエンジン ・ AI）を選び、capability と照らして validation result にまとめる",
   "必須の rule が verified でなければ、全体を verified にしない。検証しなかったことも隠さない",
   "Resolver は参照 ・ 導出値 ・ 語を解決し、検証と描画の両方が使う",
   "描画の契約（view ・ page template ・ catalog ・ design system）も抽象の層に置く（Q0 で決めたとおり、描画も基盤の契約）",
   "「Concrete」は所有の意味で使う。基盤が持つスキーマは base contract schema（基盤の契約のスキーマ）と呼び、Concrete と呼ばない",
   "構想 v0.3 の概念は、採る ・ 採らないのどちらかに振り分けた。x- の仕組みは v0.3 に無く、いまの実装の側にだけある",
   "承認（approval）は抽象の層の外（基盤のアプリケーション）に置く",
   "スキーマの構成は Q6、x- キーワードと rule の中身は Q7 で決める",
  ]
 },
 "images": [
  {"kind": "figure", "name": "概念図", "caption": "抽象の層の5つの群と、群どうしの関係。", "svg": svg("q1-model")},
  {"kind": "figure", "name": "概念の表", "caption": "27の概念の名前（英語 ・ 日本語）と、何を表すかと、構想 v0.3 との対応。", "svg": svg("q1-concepts")},
  {"kind": "figure", "name": "採らない概念", "caption": "構想 v0.3 から採らない概念と理由。", "svg": svg("q1-notadopted")},
 ],
 "checks": [
  {"target": "Concrete の意味", "ask": "「Concrete」を Skill が書くスキーマだけに使い、基盤のものは base contract schema と呼ぶことでよいか",
   "view": "所有で分ける。準拠はどれも meta-schema", "basis": ["ddd-advisor の回答", "基盤のスキーマの洗い出し"], "why": "基盤の契約のスキーマも Concrete の一種と見たいか"},
  {"target": "注釈の呼び名", "ask": "「注釈」の呼び名を x- keyword（x- キーワード）に変えてよいか",
   "view": "変える。JSON Schema の annotation と逆の意味に読める", "basis": ["ddd-advisor の回答", "JSON Schema の仕様"], "why": "「注釈」の語に慣れていて、残したいか"},
  {"target": "vocabulary", "ask": "vocabulary は term の集まりだけを指し、JSON Schema の仕組みは $vocabulary と綴りのまま書くことでよいか",
   "view": "分ける。同じ語で2つを指さない", "basis": ["ddd-advisor の回答"], "why": "語彙という語を、JSON Schema の意味でも使いたいか"},
 ],
 "grounds": [
  user("Q1 を抽象の層の中心の論点として厚くすること", "この論点の部分がこのSemanticSchemaDriven（セマンティックスキーマ駆動基盤）の抽象概念と抽象モデリング、抽象レイヤーを記載する部分なんじゃないの？？違うの？？\nこんな薄くていいのか？？もしボリュームが大きくなりそうなら論点を分けるとかさ工夫しませんか？？"),
  user("いまの実装と Zip の構想の両方から考えること", "しかもこれは今の実装だけでなく私が送ったZipの中身の中の内容も加味して、今回のSemanticSchemaDriven（セマンティックスキーマ駆動基盤）を考えるって話ですよね？？"),
  user("x- は Abstract スキーマが定義し、Concrete は使うだけであること", "違うよ、、今の話はまだAbstructスキーマでしょ？？Abstructスキーマがx-系の項目を定義しているんでしょ？？\nConcrateスキーマはそれを使用するだけだよね？？"),
  user("x- 項目に機械的とセマンティックの検証を持たせること", "これあれだな、今の実装とZip の構想を正しく抽象モデリング、抽象概念として整理したもので改めて用語とドメインスキーマの契約を定めて、そのドメインスキーマをドメインモデルで拡張したx-系の項目に対する機械的およびセマンティック検証を行えるようなモデルを定義するなど、これうまく概念をまとめて整理します。"),
  {"supports": "推す形", "basis": adv["conclusion"]["headline"], "tag": "primary", "source": ADV},
  vq("Abstract Schema", "docs/11_semantic-layered-model.md:10"),
  vq("機械的な意味と人の解釈を要する意味の区別", "docs/02_semantic-contract-model.md:25"),
  {"supports": "構想 v0.3 に x- の記述が無いこと", "basis": next(i["note"] for i in v03["items"] if i["group"] == "extension") or "docs と examples に x- で始まる拡張の記述は無い", "tag": "measured", "source": "measured/v03-concepts.json（docs 20本 ・ examples 13本を grep）"},
  old_grounds["表現モデルを内部のモデルとすること"],
 ],
 "rejected": [
  {"option": "Semantic Concrete Schema が x- キーワードを定義できるようにする（x-annotations）", "reason": "利用者の枠組み（x- は Abstract が定義し、Concrete は使うだけ）と合わず、使っている Concrete は0件である"},
  {"option": "構想 v0.3 の Semantic Model（Entity ・ Property ・ Type）を抽象の層の正本にする", "reason": "v0.3 の docs 自身が Entity を必須にせず、抽象スキーマと別のモデルを持つと同じものに2つのモデルができる"},
  next(r for r in old["rejected"] if r["option"].startswith("表現モデルと抽象の仕様を")),
  next(r for r in old["rejected"] if r["option"].startswith("構想 v0.3 の仕様の名前")),
 ],
}
tree_rows = [
 {"path": ".claude/skills/schema-driven/assets/", "depth": 0, "note": "基盤の契約のスキーマ（Skill が従う）"},
 {"path": "meta-schema.json", "depth": 1, "mark": "change", "note": "abstract@v1。x- キーワードの表 ・ x-vocabularies ・ rule の by ・ requires ・ required を足す"},
 {"path": "annotations.schema.json", "depth": 1, "mark": "change", "note": "x-ref の label を消し、矛盾する書き方を形で止める"},
 {"path": "view.schema.json", "depth": 1, "mark": "change", "note": "hidden を単独にする"},
 {"path": "page.schema.json", "depth": 1, "note": "いまのまま（検査を足すのはエンジンの側）"},
 {"path": "catalog.schema.json", "depth": 1, "mark": "change", "note": "description と x-prompt を足し、自己検査の10件を0件にする"},
 {"path": "design-system.schema.json", "depth": 1, "mark": "change", "note": "同じく3件を0件にする"},
 {"path": "vocabulary.schema.json", "depth": 1, "mark": "add", "note": "vocabulary@v1。term の id ・ 意味 ・ 値の形 ・ 単位"},
 {"path": "published.schema.json", "depth": 1, "note": "いまのまま"},
 {"path": "published.json", "depth": 1, "mark": "add", "note": "公開の前に1回記録する"},
]
q6 = {
 "id": "Q6", "name": "抽象スキーマの構成", "status": "waiting", "depends": ["Q1"],
 "question": "基盤の契約のスキーマを、どのスキーマの組で持ち、版をどう扱い、いまの境界の崩れをどう直すか",
 "answer": {
  "text": "基盤の契約のスキーマは4つの群に分け、いまの7つに vocabulary@v1 と x-term の形を足す。足すものは abstract@v1 に足し、まだ公開していないので版は上げない。境界の崩れ6件は、準拠と所有を分けて直す。",
  "details": [
   "4つの群の1つ目と2つ目：Semantic Concrete Schema の形（abstract@v1）と、x- キーワードの形（annotations@v1 ・ view@v1 ・ x-term の形）",
   "3つ目と4つ目：Skill が書くファイルの形（page ・ catalog ・ design-system ・ vocabulary）と、基盤の記録の形（published）",
   old_details[9],
   "7つとも meta-schema に準拠させ、所有は基盤（名前の頭 schema-driven/）で示す",
   "自分の検査で出ている13件は、除外せず description と x-prompt を足して0件にする",
   "validation result と capability の形は、抽象スキーマに置かず protocol（Q4）に置く",
   "公開の記録（published.json）は、足し終えたあと公開の前に1回記録する",
  ]
 },
 "images": [
  {"kind": "figure", "name": "スキーマの一覧", "caption": "基盤の契約のスキーマと4つの群。インスタンスを書く人も並べた。", "svg": svg("q6-schemas")},
  {"kind": "tree", "name": "ディレクトリ", "caption": "基盤の契約のスキーマの置き場所の変更前後。", "rows": tree_rows},
  {"kind": "figure", "name": "境界の直し", "caption": "いまの境界の崩れ6件と直し方。", "svg": svg("q6-fixes")},
 ],
 "checks": [{"target": "vocabulary@v1", "ask": "語彙の形 vocabulary@v1 を、基盤の契約のスキーマとして足してよいか",
             "view": "足す。Skill をまたいで語を共有するため", "basis": ["ddd-advisor の回答"], "why": "語彙を Skill ごとに持つだけで足りるか"}],
 "grounds": [old_grounds[k] for k in ("abstract@v1 に足し、版を上げないこと", "版を上げる決まりは、公開した版にだけ当たること", "abstract@v1 がまだ公開されていないこと") if k in old_grounds] + [
  {"supports": "いまの境界の崩れ", "basis": base["boundary"] if isinstance(base["boundary"], str) else json.dumps(base["boundary"], ensure_ascii=False), "tag": "measured", "source": "measured/base-schemas.json（基盤のスキーマ7つとエンジンのコードを突き合わせた）"},
 ],
 "rejected": [next(r for r in old["rejected"] if "@v2" in r["option"]),
  {"option": "自己検査の13件を、基盤のスキーマだけ除外する", "reason": "catalog と design-system は Skill の AI が書くファイルの形で、書き方の案内が実際に要る"}],
}
moved_details = [d for d in old_details[10:]]
q7 = {
 "id": "Q7", "name": "抽象スキーマの中身", "status": "waiting", "depends": ["Q1", "Q6"],
 "question": "x- キーワードと rule を、どう定義し、どの validator がどう検証するか",
 "answer": {
  "text": "x- キーワードは Abstract スキーマの x- キーワードの表1か所で定義し、1行に置く場所 ・ 形 ・ 検証の種類 ・ validator を持たせる。いま検証していないものは、機械的にする ・ AI の rule にする ・ unsupported と返す ・ 消す のどれかにする。rule には by ・ requires ・ required を足す。",
  "details": moved_details + [
   "機械的：x-abstract ・ x-generates ・ x-ref ・ x-derive ・ x-vocabularies ・ rule の by ・ requires ・ required",
   "両方：x-prompt の write（rule ごとに by で決める）・ x-view ・ x-states ・ x-term",
   "x-prompt の read は案内であって rule ではないので、中身は検証しない",
   "x-annotations と x-ref の label は消す",
   "どの rule も検証結果を1件返し、検証しなかったことを隠さない",
  ]
 },
 "images": [
  {"kind": "figure", "name": "x- キーワードの表", "caption": "x- キーワードごとの置く場所 ・ 検証の種類 ・ validator ・ 扱い。", "svg": svg("q7-keywords")},
  old_images["abstract@v1 の全体"],
  old_images["Zip の語彙の例"],
  {"kind": "code", "name": "検証結果の例", "caption": "実物のボードに34件のルールを当てた実験の数を、検証結果の5つの値で数え直した例。", "path": ".brainstorming/semantic-contract-core/proposals/validation-result-example.json", "language": "json", "text": (BOARD / "proposals/validation-result-example.json").read_text().rstrip()},
 ],
 "checks": old["checks"],
 "grounds": [old_grounds[k] for k in ("足すものに x-prompt を持たせること", "書く単位ごとに x-prompt が要ること", "意味の判断が要るルールが、いま検査されていないこと", "書くルールが、すでに1件ずつ id を持つこと", "関係と指される数を、いまの基盤が検証していること") if k in old_grounds] + [
  {"supports": "x- の検証の振り分け", "basis": adv["conclusion"]["because"], "tag": "primary", "source": ADV}],
 "rejected": [{"option": "いま検証していないものを、そのまま残す", "reason": "検証しなかったことが見えず、合格と区別できない"}],
}
topics = []
for t in board["topics"]:
    if t["id"] == "Q1":
        topics += [q1, q6, q7]; continue
    if t["id"] in ("Q3", "Q4"):
        t["depends"] = list(dict.fromkeys(t.get("depends", []) + ["Q6", "Q7"]))
    topics.append(t)
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、抽象の層を Q1 抽象の概念モデル ・ Q6 抽象スキーマの構成 ・ Q7 抽象スキーマの中身 に分けて組み直した（ddd-advisor の概念モデル）"}},
         {"op": "replace", "path": "/topics", "value": topics}]
print(json.dumps(patch, ensure_ascii=False))
