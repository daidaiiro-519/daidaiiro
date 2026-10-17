# 第32回：利用者の指摘で、Q1 の抽象の仕様を構想 v0.3 の名前でなく、採る概念と名前（instance ・ vocabulary ・ JSON Schema ・ rule ・ capability と validation result）で書き直す。v0.3 の名前は参考に下げる。Q2 の同じ名前も揃える。
import json, pathlib
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
user = lambda s, w: {"supports": s, "basis": w, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
T = {t["id"]: t for t in board["topics"]}
q1 = T["Q1"]
q1["question"] = "抽象の仕様に採る5つの概念（instance ・ vocabulary ・ JSON Schema ・ rule ・ capability と validation result）を、いまの abstract@v1 でどこまで持ち、何を足すか"
q1["answer"] = {
 "text": "JSON Schema 2020-12 は abstract@v1 で足りている。instance ・ rule ・ capability と validation result は一部あり、vocabulary は無い。足すものは abstract の新しい版に置き、abstract@v1 は変えない。",
 "details": [
  "名前は構想 v0.3 の仕様の名前を使わず、採る概念を標準の仕様と基盤のコードの語で呼ぶ（Q5）。v0.3 の名前は図の参考の列に置く",
  "JSON Schema 2020-12（JSON での書き方）：同じ。abstract@v1 が JSON Schema 2020-12 と注釈で持つ",
  "instance（インスタンス）：一部ある。kind と id、プロパティと型がこれにあたる。語の意味が通る範囲（文脈）が無い",
  "vocabulary（語彙）：無い。$ref で形は共有できるが、語の意味や単位を共有する仕組みが無い",
  "rule（ルール）：一部ある。構造と値は JSON Schema、関係と数は x-ref、導出値は x-derive が持つ",
  "rule の足りないもの：意味のルールは x-prompt の test の文だけで、validator ・ 必要な capability ・ 必須かを宣言できない",
  "capability と validation result：一部ある。結果の形（合格 ・ ずれ ・ 確かめ直し）はあるが、capability の宣言と、検証しなかったルールの報告が無い",
  "公開の仕様：標準として互換を持つのは JSON/JSON Schema だけで、ほかは独自の拡張（x-）として基盤の契約に置く",
  "構想 v0.3 の全体構成図にある表現モデルは、エンジンの中のモデルとし、正本はこの抽象の仕様の1か所に置く",
 ]
}
for im in q1["images"]:
    if im.get("name") == "仕様の対応":
        im["name"] = "概念の対応"
        im["caption"] = "抽象の仕様に採る概念と名前と、いまの基盤の対応。構想 v0.3 の名前は参考。"
        im["svg"] = (HERE / "q1-specs.svg").read_text()
    if im.get("name") == "ルールの差分":
        im["caption"] = "rule を足す例：abstract のルール1件に、validator ・ 必要な capability ・ 必須かを足す（新しい版 @v2。abstract@v1 は変えない）。"
    if im.get("name") == "Zip の語彙の例":
        im["caption"] = "vocabulary の例（構想 v0.3 の Zip）：語に値の形 ・ 意味（量と単位）・ 必要な capability を持たせる。"
for g in q1["grounds"]:
    if g["supports"] == "5つの仕様":
        g["supports"] = "構想 v0.3 の5つの仕様（参考）"
q1["grounds"].append(user("採るのは概念で、名前はそのまま使わないこと", "これ原典で書いているものに引っ張られてるけど、採用にするのは概念なわけで、まんま用語をそのまま使用するわけではないよね？"))
q1["rejected"].append({"option": "構想 v0.3 の仕様の名前（Semantic Model ・ Vocabulary Spec など）をそのまま使う", "reason": "検討中の案の名前で、標準の仕様にもコードにも無い。採るのは概念で、名前は Q5 の決め方で付ける"})
q2 = T["Q2"]; d = q2["answer"]["details"]
for k, x in enumerate(d):
    d[k] = x.replace("Vocabulary Spec と Semantic Model に定義する", "vocabulary と instance の定義に置く").replace("Constraint Spec と結果の形に定義する", "rule と validation result の形に置く")
for im in q2["images"]:
    if im.get("name") == "標準と拡張":
        im["svg"] = (HERE / "q2-concepts.svg").read_text()
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、Q1 の抽象の仕様を構想 v0.3 の名前でなく採る概念と名前で書き直し、v0.3 の名前は参考に下げた"}},
         {"op": "replace", "path": "/topics", "value": list(T.values())}]
print(json.dumps(patch, ensure_ascii=False))
