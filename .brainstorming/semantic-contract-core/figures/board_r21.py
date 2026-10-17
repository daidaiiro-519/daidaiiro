# 第21回：利用者の指摘で、クラウドは優先を後にし、「ローカルでできることはアダプターの差し替えでクラウドでもできる」作りを守る前提に置き直す。
import json, pathlib
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
user = lambda s, w: {"supports": s, "basis": w, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
U_MCP = "基本的にクラウドのことを考えた時にもし基盤の操作が必要になっても多分mcpサーバーとかになるのでは？"
U_LIFT = "これは優先事項は後だけどクラウドにそのままリフトできる作りにはしといてねって感じ。"
U_PORT = "そのためのポートとアダプターのがいねんでもあるんだよぉ"
T = {t["id"]: t for t in board["topics"]}
q4 = T["Q4"]; d = q4["answer"]["details"]
q4["answer"]["text"] = "protocol を運び方から切り離して JSON Schema で定義し、基盤は CLI ・ MCP ・ WASM で運ぶ。画面のための HTTP は Concrete が自分のコマンドの一覧をそのまま載せる。クラウドは優先を後にし、アダプターの差し替えで載せられる作りを守る。基盤は足すだけの版で出し、4つの測り方がどれも0件なら完成とする。"
drop = ("クラウド：Lambda", "クラウドの中身：", "中身を考え直すのは：")
i = next(k for k, x in enumerate(d) if x.startswith("クラウド：Lambda"))
d[:] = [x for x in d if not x.startswith(drop)]
d.insert(i, "クラウド：優先は後。ローカルでできることはクラウドでもできる作りにし、載せるときはアダプターの差し替えで済ませる（入る側は MCP などの手段、出る側は保存先）")
d.insert(i + 1, "そのために守る境界：core は入出力に触らず、ファイル ・ プログラム ・ AI の起動はポートの向こうのアダプターに置く（Q3）。クラウドの入口の形と中身の動かし方は、載せるときに決める")
q4["checks"] = [c for c in q4["checks"] if c["target"] != "クラウドの中身"]
q4["grounds"] += [user("クラウドの入口は MCP などになりうること", U_MCP), user("クラウドは優先を後にし、そのまま載せられる作りにすること", U_LIFT), user("そのための境界がポートとアダプターであること", U_PORT)]
q4["rejected"].append({"option": "クラウドの入口と中身の形を、いま決める（第20回）", "reason": "優先は後で、入口は MCP などになりうる。いまは境界を守れば、載せるときにアダプターの差し替えで済む"})
q4["rejected"] = [r for r in q4["rejected"] if not r["option"].startswith(("クラウドでも、同梱したバイナリ", "クラウドで WASM"))]
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、クラウドは優先を後にし、アダプターの差し替えで載せられる作りを守る前提に置き直した"}},
         {"op": "replace", "path": "/topics", "value": list(T.values())}]
print(json.dumps(patch, ensure_ascii=False))
