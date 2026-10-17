# 第18回：platform-advisor の回答（advisors/platform-http.json、案C）で、HTTP の持ち主を決める。
# 基盤は HTTP で待ち受けず、Concrete が CLI を包んで受ける。基盤は HTTP への載せ方の契約と適合の組だけを持つ。
# あわせて、HTTP を基盤のインターフェースとして描いていた図（Q0 ・ Q3 ・ Q4）とファイル案、Q3 の図の json-ld、Q2 の案の「写して相互運用」を揃える。
import json, pathlib, sys

HERE = pathlib.Path(__file__).parent
BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
adv = json.loads((BOARD / "advisors/platform-http.json").read_text())
SRC = "platform-advisor の回答（2026-10-17、HTTP の持ち主）。advisors/platform-http.json"
user = lambda supports, words: {"supports": supports, "basis": words, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}

def quote(text, start, end):
    i = text.find(start); j = text.find(end, i)
    if i < 0 or j < 0:
        sys.exit(f"原文に無い：{start}")
    return text[i:j + len(end)]

g = lambda supports, text, start, end: {"supports": supports, "basis": quote(text, start, end), "tag": "primary", "source": SRC}
paths = {p["ask"]: p["answer"] for p in adv["path"]}
comp = {c["option"][0]: c["effect"] for c in adv["comparison"]}
U_WHEN = "仮にこれ基盤をHTTPで呼ばなければいけなくなる時って何だろうなっていうのもきになる。"
T = {t["id"]: t for t in board["topics"]}

def replace(details, start, text):
    i = next(k for k, x in enumerate(details) if x.startswith(start)); details[i] = text; return i

q4 = T["Q4"]; d = q4["answer"]["details"]
q4["answer"]["text"] = "protocol を運び方から切り離して JSON Schema で定義し、基盤は CLI ・ MCP ・ WASM で運ぶ。HTTP は基盤が待ち受けず、Concrete が CLI を包んで受け、基盤は HTTP への載せ方と適合の組を protocol の一部として持つ。基盤は足すだけの版で出し、4つの測り方がどれも0件なら完成とする。"
replace(d, "HTTP：基盤のバイナリを起動できない", "HTTP：基盤は待ち受けない。ブラウザの画面のように基盤のバイナリを起動できない呼び手は、Concrete が CLI を包んで受ける（いまの brainstorming の serve の形）")
i = replace(d, "HTTP で受けるときは", "基盤が持つのは HTTP への載せ方：終了コード 0 ・ 1 ・ 2 と HTTP の状態の対応と、本文は CLI の JSON をそのまま返すこと。入力と期待する出力の組で、どの Concrete の HTTP も同じ形で返すかを試験する")
d.insert(i + 1, "Concrete が HTTP で受けるときの守り（127.0.0.1 だけで待ち受ける ・ 起動ごとの合言葉 ・ Host と Origin の確認 ・ 画面に要る操作だけを受ける）は、Concrete の約束として基盤の文書に書く")
d.insert(i + 2, "基盤が HTTP で受ける形を検討し直すのは、クラウドの AI や別の機械からバイナリを起動せずに呼ぶとき、正本を1か所に置いて複数の人が同時に書き承認を一元にするとき、HTTP を持つ Concrete が増えて守り方がずれたときである。そのときは TLS と利用者の認証を基盤の1か所に持つ")
j = next(k for k, x in enumerate(d) if x.startswith("WASM の部品は"))
d.insert(j + 1, "ブラウザの中で読むだけの操作（check ・ render）を WASM で動かす使い道もある。書く操作は代えられないので、HTTP を置き換えるものではない（採否は決めていない）")
q4["checks"] = [{"target": "HTTP", "ask": "HTTP は基盤が待ち受けず Concrete が CLI を包んで受け、基盤は載せ方の契約と適合の組だけを持つことでよいか"}] + [c for c in q4["checks"] if c["target"] != "HTTP"]
q4["grounds"] += [
    user("HTTP で呼ぶ場面を確かめること", U_WHEN),
    g("推す案と理由", adv["conclusion"]["because"], "いまの使い方で HTTP でしか解けない場面は", "運用の負担だけが増える。"),
    g("受ける範囲は Concrete が持つほうが小さいこと", paths[next(k for k in paths if k.startswith("認証と受ける範囲"))], "Concrete が持つほうが小さい。", "Concrete が使わない操作まで受けることになる"),
    g("同じ protocol は載せ方の契約と試験で守ること", paths[next(k for k in paths if k.startswith("同じ protocol"))], "この差は、待ち受けの置き場所ではなく", "埋められる。"),
    {"supports": "不要なアクセス経路を開かないこと", "basis": next(x["quote"] for x in adv["grade_grounds"] if x["grade"] == "e-5"), "tag": "primary", "source": SRC + "（非機能要求グレード E.5.2.1 の引用）"},
]
q4["rejected"] += [
    {"option": "基盤が HTTP で待ち受ける（CLI ・ MCP と同じツールの一覧から HTTP の API を作る）", "reason": "基盤の8つの操作すべてが外から呼べる範囲に入り、画面のための serve も残るので待ち受けと合言葉が2組になる。いまは使う呼び手が居ない"},
    {"option": "Concrete が CLI を包み、基盤は何も足さない", "reason": "状態の対応や本文の形が Concrete ごとに決まり、2つ目の Concrete が HTTP を持った時点で同じ protocol を返す保証が無くなる"},
]
for im in q4["images"]:
    if im.get("name") == "protocol":
        im["caption"] = "1つの protocol を、基盤の CLI ・ MCP ・ WASM と、Concrete が CLI を包む HTTP が同じ形で運ぶ。"
        im["svg"] = (HERE / "q4-protocol.svg").read_text()

q3 = T["Q3"]
for im in q3["images"]:
    if im.get("name") == "Rust の構成":
        im["svg"] = (HERE / "q3-rust.svg").read_text()
    if im.get("kind") == "tree":
        im["rows"] = [r for r in im["rows"] if r["path"] != "http.rs"]
q0 = T["Q0"]
for im in q0["images"]:
    if im.get("name") == "境界と向き":
        a = ">CLI ・ MCP ・ HTTP ・ WASM Component<"; assert a in im["svg"]
        im["svg"] = im["svg"].replace(a, ">CLI ・ MCP ・ WASM Component（HTTP は Concrete）<", 1)
q2 = T["Q2"]
for im in q2["images"]:
    if im.get("name") == "自前の語彙（案）":
        im["text"] = (BOARD / "proposals/vocabulary.schema.json").read_text().rstrip()

patch = [
    {"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "platform-advisor の回答で、HTTP は基盤が待ち受けず Concrete が CLI を包んで受ける形にした。基盤は載せ方の契約と適合の組を持つ。図とファイル案を同じ前提に揃えた"}},
    {"op": "replace", "path": "/topics", "value": list(T.values())},
]
print(json.dumps(patch, ensure_ascii=False))
