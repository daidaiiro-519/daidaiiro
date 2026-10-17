# 第22回：利用者の指摘で、画面の API（HTTP）の話を基盤の論点から外す。API は Concrete が持ち、中で基盤のコマンドを組み合わせる。
# WASM の確かめてほしい箇所を、普通の言葉で書き直す。
import json, pathlib
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
user = lambda s, w: {"supports": s, "basis": w, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
U_API = "ローカルで使う時もciliやmcpサーバー使用するようなagents coreみたいな使い方をして、必要なAPIは別にコンクレート側で使えばよくないか？"
U_COMBINE = "そのAPIで必要な基盤のコマンドを呼び出して組み合わせて使えば？って感じなんだけど、どういう質問なん？"
U_WORDS = "ほかの言語から使う日と、担い手を足す日のどちらが先か\n上記の言い回しが意味不明だ。"
T = {t["id"]: t for t in board["topics"]}
q4 = T["Q4"]; d = q4["answer"]["details"]
q4["answer"]["text"] = "protocol を運び方から切り離して JSON Schema で定義し、基盤は CLI ・ MCP ・ WASM で運ぶ。HTTP などの API は Concrete が持つ。クラウドは優先を後にし、アダプターの差し替えで載せられる作りを守る。基盤は足すだけの版で出し、4つの測り方がどれも0件なら完成とする。"
drop = ("HTTP：基盤は待ち受けない", "画面専用の経路", "基盤が持つのは HTTP への載せ方", "手元の守り")
i = next(k for k, x in enumerate(d) if x.startswith(drop))
d[:] = [x for x in d if not x.startswith(drop)]
d.insert(i, "HTTP などの API は Concrete が持ち、中で基盤のコマンドを呼んで組み合わせる。基盤は HTTP に関わらない")
q4["checks"] = [{"target": "WASM", "ask": "WASM を、ほかの言語のプログラムから基盤を使う手段として持つことでよいか",
                 "view": "持つ。基盤を言語に依らず使える", "basis": ["Component Model の文書", "wasmtime の README"],
                 "why": "ほかの言語のプログラムから基盤を使う予定があるか"}]
gone = ("同じ protocol は載せ方の契約と試験で守ること", "コマンドの一覧をそのまま載せる形", "クラウドの中身を関数に組み込むこと", "HTTP は Concrete のコマンドを載せる形にすること")
q4["grounds"] = [g for g in q4["grounds"] if g["supports"] not in gone]
q4["grounds"] += [user("API は Concrete が持つこと", U_API), user("Concrete が基盤のコマンドを組み合わせること", U_COMBINE), user("確かめてほしい箇所を普通の言葉で書くこと", U_WORDS)]
q4["rejected"] = [r for r in q4["rejected"] if not r["option"].startswith(("HTTP を持たない", "Concrete が CLI を包み", "画面専用の経路"))]
q4["rejected"].append({"option": "基盤が HTTP への載せ方（経路 ・ 状態の対応）の契約を持つ（第18〜20回）", "reason": "画面の API は Concrete の都合で、基盤のコマンドを組み合わせて作る。基盤が載せ方を決めると、Concrete の API の形を縛る"})
for im in q4["images"]:
    if im.get("name") == "protocol":
        im["caption"] = "1つの protocol を、基盤の CLI ・ MCP ・ WASM が同じ形で運ぶ。HTTP などの API は Concrete が持つ。"
        im["svg"] = (HERE / "q4-protocol.svg").read_text()
for im in T["Q3"]["images"]:
    if im.get("name") == "Rust の構成":
        im["svg"] = (HERE / "q3-rust.svg").read_text()
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、画面の API（HTTP）の話を基盤の論点から外した。API は Concrete が持ち、基盤のコマンドを組み合わせる"}},
         {"op": "replace", "path": "/topics", "value": list(T.values())}]
print(json.dumps(patch, ensure_ascii=False))
