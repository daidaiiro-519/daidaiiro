# 第27回：利用者の指摘で、検証の全体を Semantic Validator（中に Schema Validator）にし、アダプターを英語の inbound ・ outbound アダプターにする。
import json, pathlib
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
user = lambda s, w: {"supports": s, "basis": w, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
T = {t["id"]: t for t in board["topics"]}
q5 = T["Q5"]
q5["question"] = "基盤で使う名前（ルール1件を検証する側 ・ 検証の全体 ・ 検証の結果 ・ できることの宣言 ・ アダプター ・ 書くときの条件）を何にするか"
q5["answer"] = {
 "text": "名前は、標準の仕様と基盤のコードが使う語に揃え、構想 v0.3 の名前は候補として扱う。コード ・ 仕様に識別子として出る語は英語、ふつうの名詞はカタカナにする。いま作るのは Semantic Validator で、その中に Schema Validator を持つ。",
 "details": [
  "validator：ルール1件を検証する側（エンジン ・ AI ・ 人）の一般名。JSON Schema と SHACL の仕様が使う語",
  "Semantic Validator：いま作る検証の全体。ルールごとにどの validator で検証するかを選び、結果を検証結果で返す。基盤のうちの検証の部分の名前で、作成 ・ 更新 ・ 承認 ・ 描画は含まない",
  "Schema Validator：Semantic Validator の中の、構造と値の検証（JSON Schema）",
  "検証結果：検証の結果（validation result）。値は verified ・ violated ・ unverified ・ unsupported ・ error の5つ",
  "capability：validator が何を検証できるかの宣言。MCP の仕様が、接続の最初に宣言し合う意味で使う",
  "inbound アダプター ・ outbound アダプター：基盤を呼ばれる側と、基盤から外を呼ぶ側。基盤のコードのフォルダ名 inbound ・ outbound と同じにする",
  "ルール：スキーマに書く条件1件（id ・ rule ・ why ・ test）。基盤のスキーマのキーが rule である",
  "承認されたら、ほかの論点の名前をこの表に揃える",
 ]
}
for im in q5["images"]:
    if im.get("name") == "名前の対応":
        im["svg"] = (HERE / "q5-terms.svg").read_text()
q5["checks"] = [{"target": "validator", "ask": "ルール1件を検証する側の名前を validator にしてよいか", "view": "validator にする。標準の仕様が使う語である",
                 "basis": ["JSON Schema の仕様", "SHACL の仕様"], "why": "構想 v0.3 の Interpreter に込めた意味（解釈 ・ 調停）を残したいか"}]
q5["grounds"] += [
 user("検証の全体を Semantic Validator にし、中に Schema Validator を持つこと", "validatorとSemantic Validatorだけど、今回作成しようとしているものってSemantic Validatorだよね？この中に通常のSchemaValidatorがあるようなイメージです。"),
 user("アダプターの名前を英語にすること", "英語でいいよ、その形で Q5 を直してお願い"),
]
q5["rejected"] += [
 {"option": "Semantic Interpreter を、ルールごとに validator を選ぶ独立した部品にする（構想 v0.3）", "reason": "選ぶのは Semantic Validator の働きの一部で、部品を分けると同じ検証の全体に2つの名前が付く"},
 {"option": "アダプターをカタカナ（インバウンドアダプター ・ アウトバウンドアダプター）で書く", "reason": "コードのフォルダ名と読み替えが要り、表のほかの識別子（validator ・ capability）と揃わない"},
]
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、Q5 の検証の全体を Semantic Validator（中に Schema Validator）にし、アダプターを英語の inbound ・ outbound アダプターにした"}},
         {"op": "replace", "path": "/topics", "value": list(T.values())}]
print(json.dumps(patch, ensure_ascii=False))
