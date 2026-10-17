# 第29回：利用者の指摘で、用語を英語と日本語の対で持つ。
import json, pathlib
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
T = {t["id"]: t for t in board["topics"]}
q5 = T["Q5"]
q5["answer"]["text"] = "名前は、標準の仕様と基盤のコードが使う語に揃え、英語と日本語の対で持つ。構想 v0.3 の名前は候補として扱う。解決は Resolver（解決器）、検証の全体は Semantic Validator（意味検証器）、ルール1件の合否を出す側は validator（検証器）とし、Interpreter の語は使わない。"
q5["answer"]["details"] = [
 "Resolver（解決器）：x-ref の指す先 ・ x-derive の導出値 ・ 語彙を解決する（いまの graph()）。Semantic Validator と描画の両方が使う",
 "解決と検証は役割として分けるが、1つのドメインの中のモジュールにする。同じ言葉（参照 ・ 導出値 ・ ルール ・ インスタンス）で話せるため",
 "構想 v0.3 の Interpreter は3つの意味を1語に載せていたので使わない。外から足す検証は validator、validator を選ぶ調停は Semantic Validator、実行時にスキーマを動かすことは基盤の外に振り分ける",
 "validator（検証器）：ルール1件を検証する側（エンジン ・ AI ・ 人）の一般名。JSON Schema と SHACL の仕様が使う語",
 "Semantic Validator（意味検証器）：いま作る検証の全体。ルールごとにどの validator で検証するかを選び、結果を検証結果で返す。作成 ・ 更新 ・ 承認 ・ 描画は含まない",
 "Schema Validator（スキーマ検証器）：Semantic Validator の中の、構造と値の検証（JSON Schema）",
 "validation result（検証結果）：値は verified ・ violated ・ unverified ・ unsupported ・ error の5つ",
 "capability（能力）：validator が何を検証できるかの宣言。MCP の仕様が、接続の最初に宣言し合う意味で使う",
 "inbound adapter（入力アダプター）・ outbound adapter（出力アダプター）：基盤を呼ばれる側と、基盤から外を呼ぶ側。英語はコードのフォルダ名 inbound ・ outbound と同じ",
 "rule（ルール）：スキーマに書く条件1件（id ・ rule ・ why ・ test）",
 "書き方：文の中では英語で書き、初めて出すときに日本語の用語を括弧で添える。用語の表は英語と日本語の対で持つ",
 "承認されたら、ほかの論点の名前をこの表に揃える",
]
for im in q5["images"]:
    if im.get("name") == "名前の対応":
        im["caption"] = "決める名前の英語と日本語と、構想 v0.3 の候補と、出どころ。"
        im["svg"] = (HERE / "q5-terms.svg").read_text()
q5["grounds"].append({"supports": "英語と日本語の対で持つこと", "basis": "英語と日本語の用語で両方ほしいですね。", "tag": "primary", "source": "利用者の言葉（2026-10-17）"})
q5["rejected"] = [r for r in q5["rejected"] if not r["option"].startswith("検証する側を検証器と呼ぶ")]
q5["rejected"].append({"option": "英語だけ、または日本語だけで持つ", "reason": "英語だけではコードの外で読みにくく、日本語だけではコードや仕様の語と対応が取れない"})
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、Q5 の用語を英語と日本語の対で持つようにした"}},
         {"op": "replace", "path": "/topics", "value": list(T.values())}]
print(json.dumps(patch, ensure_ascii=False))
