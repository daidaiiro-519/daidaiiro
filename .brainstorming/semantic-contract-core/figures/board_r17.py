# 第17回：利用者の指摘で、Q4 の「HTTP」と「外へ取りに行かない」に混ざっていた3つの話を分ける。
# 基盤を呼ぶ手段（Q4）／検証中に外のスキーマを取りに行かない再現性の約束（Q4）／Concrete が許す外への通信（Q3）。
import json, pathlib

HERE = pathlib.Path(__file__).parent
BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
user = lambda supports, words: {"supports": supports, "basis": words, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
U_COMM = "あとさ、外への通信が必要かどうかはコンクレートによるんじゃなくて？もちろんスキーマ駆動も外への通信をする時もあるとは思うしねぇ。たとえばウェブサーチとかはどうするの？ファクトチェックとかは？"
U_CLI = "基本はバイナリでcliを呼び出せばいいんだろうなって思ってたから。"
T = {t["id"]: t for t in board["topics"]}

def replace(details, start, text):
    i = next(k for k, x in enumerate(details) if x.startswith(start))
    details[i] = text
    return i

q4 = T["Q4"]; d = q4["answer"]["details"]
replace(d, "HTTP：外から呼ばれる", "HTTP：基盤のバイナリを起動できない呼び手（ブラウザ ・ 別の機械）から基盤を呼ぶ手段である。同じ機械の Skill ・ AI ・ スクリプトからは CLI で足りる")
replace(d, "標準の部品の能力は", "検証の再現性：検証の途中で、$ref が指す外のスキーマを取りに行かない（同じ入力から同じ結果を返すため）。jsonschema の取りに行く機能は feature で外す。これは検証の中だけの約束で、Concrete が許す外への通信（Q3）とは別である")
q4["checks"] = [c for c in q4["checks"] if c["target"] != "HTTP"]
q4["grounds"].append(user("同じ機械からは CLI で呼ぶこと", U_CLI))
for r in q4["rejected"]:
    if r["option"].startswith("HTTP を持たない"):
        r["option"] = "HTTP を持たない（検証中に外へ取りに行かない約束を理由に）"
        r["reason"] = "その約束は検証中の $ref の話で、外から基盤を呼ぶ手段とは関係しない。ブラウザや別の機械から使う目的に届かない"

q3 = T["Q3"]; d = q3["answer"]["details"]
i = next(k for k, x in enumerate(d) if x.startswith("許可の設定は、いま brainstorming"))
d.insert(i + 1, "外への通信（WebSearch ・ 原文の取得など）も、出ていく側を通して行う。要るかどうかは Concrete が決めて許可の設定に書き、基盤は許していない通信をさせない。WebSearch や fact-check そのものは基盤に入れず、AI と fact-check の Skill が行う")
q3["grounds"].append(user("外への通信が要るかは Concrete が決めること", U_COMM))

patch = [
    {"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、Q4 の HTTP の話を3つに分けた（基盤を呼ぶ手段 ・ 検証中に外のスキーマを取りに行かない約束 ・ Concrete が許す外への通信は Q3 へ）"}},
    {"op": "replace", "path": "/topics", "value": list(T.values())},
]
print(json.dumps(patch, ensure_ascii=False))
