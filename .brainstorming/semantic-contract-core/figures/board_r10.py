# 第10回：利用者の指摘で、Q0 の目的を全体構成図の「Schema駆動基盤」の定義に合わせて書き直す。
# 意味 ・ 構造 ・ 関係 ・ 制約を1つの契約に宣言し、機械で決まる検証も意味の判断が要る検証も同じ契約から行う。
import json, pathlib

HERE = pathlib.Path(__file__).parent
BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
IMG = "構想 v0.3 の全体構成図（images/semantic-contract-core-overview-illustrated.png。画像の文字を書き写した）"
img = lambda supports, words: {"supports": supports, "basis": words, "tag": "primary", "source": IMG}
user = lambda supports, words: {"supports": supports, "basis": words, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
U_SEM = "これはセマンティックな検証もできるSchemaなんでしょ？？機械的にも意味的にもどちらにも対応できるものなんでしょ？？"
U_BASE = "その目的をベースにして私たちはセマンティックに考えていくんでしょ？？？"
OLD = "基盤の目的は、AI が書いたインスタンスを、宣言した決まりで機械が確かめ、確かめられなかった決まりも隠さず返すことである。"

t = next(t for t in board["topics"] if t["id"] == "Q0")
t["answer"]["text"] = "基盤の目的は、意味 ・ 構造 ・ 関係 ・ 制約を1つの契約（Schema）に宣言し、その契約でインスタンスを解釈して検証することである。形や数のように機械で決まる検証も、意味の判断が要る検証も、同じ契約から行い、同じ形の結果で返す。"
d = t["answer"]["details"]
assert d[0].startswith("そのために")
d.insert(1, "検証の担い手：構造と値は JSON Schema の実装、関係と制約は基盤のエンジン、意味は Semantic Interpreter（AI も担い手の1つ）")
d.insert(2, "結果の形：どの担い手でも5つの語（verified ・ violated ・ unverified ・ unsupported ・ error）で返し、確かめられなかった決まりも隠さない")
t["grounds"] += [
    img("目的を Schema駆動基盤の定義に合わせること", "2 Schema駆動基盤（Semantic Contract Core）：意味・構造・関係・制約を統合し、解釈可能で検証可能な基盤"),
    img("基盤の中心が私たちの表現能力 ・ 抽象モデルであること", "既存技術を「部品」として活用し、私たちの表現能力・抽象モデルを中心に据えた Schema 駆動基盤と開発の全体像"),
    user("機械の検証と意味の検証の両方を目的に入れること", U_SEM),
    user("目的を、以降の論点の土台にすること", U_BASE),
]
t["rejected"].append({"option": f"目的を「{OLD[6:-1]}」とする（第9回）", "reason": "確かめ方の説明にとどまり、意味 ・ 構造 ・ 関係 ・ 制約を1つの契約に宣言して解釈と検証を行うという基盤の中心が書かれていない"})
for im in t["images"]:
    if im.get("name") == "設計原則":
        im["svg"] = (HERE / "q0-principles.svg").read_text()

patch = [
    {"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、Q0 の目的を全体構成図の Schema駆動基盤の定義に合わせ、機械で決まる検証と意味の判断が要る検証の両方を目的に書いた"}},
    {"op": "replace", "path": "/topics", "value": board["topics"]},
]
print(json.dumps(patch, ensure_ascii=False))
