# 第28回：利用者の問い（Interpreter と validator は役割が違うなら分けるべきか）を ddd-advisor に聞き（advisors/ddd-interpreter.json）、解決を Resolver として分ける。
import json, pathlib, sys
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
adv = json.loads((BOARD / "advisors/ddd-interpreter.json").read_text())
SRC = "ddd-advisor の回答（2026-10-17、Interpreter と validator）。advisors/ddd-interpreter.json"
def q(text, a, b):
    i = text.find(a); j = text.find(b, i)
    if i < 0 or j < 0: sys.exit(f"原文に無い：{a}")
    return text[i:j + len(b)]
user = lambda s, w: {"supports": s, "basis": w, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
T = {t["id"]: t for t in board["topics"]}
q5 = T["Q5"]
q5["question"] = "基盤で使う名前（解決 ・ ルール1件を検証する側 ・ 検証の全体 ・ 検証の結果 ・ できることの宣言 ・ アダプター ・ 書くときの条件）を何にするか"
q5["answer"]["text"] = "名前は、標準の仕様と基盤のコードが使う語に揃え、構想 v0.3 の名前は候補として扱う。コード ・ 仕様に識別子として出る語は英語、ふつうの名詞はカタカナにする。解決は Resolver、検証の全体は Semantic Validator（中に Schema Validator）、ルール1件の合否を出す側は validator とし、Interpreter の語は使わない。"
d = q5["answer"]["details"]
d.insert(0, "Resolver：x-ref の指す先 ・ x-derive の導出値 ・ 語彙を解決する（いまの graph()）。Semantic Validator と描画の両方が使う")
d.insert(1, "解決と検証は役割として分けるが、1つのドメインの中のモジュールにする。同じ言葉（参照 ・ 導出値 ・ ルール ・ インスタンス）で話せるため")
d.insert(2, "構想 v0.3 の Interpreter は3つの意味を1語に載せていたので使わない。外から足す検証は validator、validator を選ぶ調停は Semantic Validator、実行時にスキーマを動かすことは基盤の外に振り分ける")
q5["images"] = [
 {"kind": "figure", "name": "名前の対応", "caption": "決める名前と、構想 v0.3 の候補と、出どころ。", "svg": (HERE / "q5-terms.svg").read_text()},
 {"kind": "figure", "name": "役割の分け方", "caption": "Resolver が解決し、Semantic Validator と描画が使う。Semantic Validator はルールごとに validator を選ぶ。", "svg": (HERE / "q5-roles.svg").read_text()},
]
q5["checks"] = [{"target": "Resolver", "ask": "解決を Resolver として分け、1つのドメインの中のモジュールにすることでよいか",
                 "view": "役割は分け、ドメインは分けない", "basis": ["ddd-advisor の回答", "基盤の graph()"],
                 "why": "解釈を、検証とは別の言葉で話すドメインとして扱いたいか"}]
q5["grounds"] += [
 user("Interpreter と validator を役割で分けるか確かめること", "Interpreterとvalidatorで役割が違うなら分けるべきじゃないか？\nドメイン的にも違うってことかな？"),
 {"supports": "推す形", "basis": adv["conclusion"]["headline"], "tag": "primary", "source": SRC},
 {"supports": "分ける理由と、ドメインを分けない理由", "basis": q(adv["conclusion"]["because"], "v0.3 の Interpreter は検証 ・ 調停 ・ 実行の3つを1語に載せており", "文脈は1つのままモジュールで分ける。"), "tag": "primary", "source": SRC},
]
q5["rejected"] += [
 {"option": "Interpreter の語を、解決か検証のどちらかの名前として残す", "reason": "構想 v0.3 では3つの意味を載せていた語で、1つの用語に1つの意味にならない"},
 {"option": "解決（Resolver）を、検証と別のドメインにする", "reason": "同じ言葉で話せて競合するモデルが無く、x-ref を1つ変えるたびに2つのドメインを直すことになる"},
]
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "ddd-advisor の回答で、Q5 の解決を Resolver として分け（ドメインは1つ）、Interpreter の語を使わないことにした"}},
         {"op": "replace", "path": "/topics", "value": list(T.values())}]
print(json.dumps(patch, ensure_ascii=False))
