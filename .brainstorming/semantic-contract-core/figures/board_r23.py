# 第23回：利用者の指摘で、基盤のインターフェースをコマンドと protocol だけにし、入る側と出る側のアダプターを差し替えて手元でもクラウドでも同じ振る舞いにする形へ直す。WASM を外す。
import json, pathlib, sys
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
S = pathlib.Path(sys.argv[1])
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
user = lambda s, w: {"supports": s, "basis": w, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
U_SAME = "Adapterの接続先が違うよっていうようなInboundとOutboundを変えれば接続先だけで振る舞いは同じで作れるよって話なのかどっちなんですかね？？"
U_IF = "APIとして呼び出すのはこの基盤を使用するインターフェースだと思っているんですがそういうことではなく？？"
U_CLOUD = "いや、クラウドで動かしますよ？\n今はまだやらないってだけでできる仕組みにはしといとねってお願いだが？"
U_WASM = "コンクレート側は別にrustだけじゃないし、別の言語でもバイナリを呼ぶだけじゃないのか？"
T = {t["id"]: t for t in board["topics"]}
q4 = T["Q4"]; d = q4["answer"]["details"]
q4["answer"]["text"] = "protocol を運び方から切り離して JSON Schema で定義し、基盤のインターフェースはコマンドと protocol だけにする。入る側と出る側のアダプターを差し替えれば、手元でもクラウドでも同じ振る舞いになる（いまは CLI ・ MCP とファイル）。HTTP などの API は Concrete が持つ。基盤は足すだけの版で出し、4つの測り方がどれも0件なら完成とする。"
drop = ("WASM Component：", "WASM の部品は", "Rust は WASM の両側", "クラウド：優先は後", "そのために守る境界")
i = next(k for k, x in enumerate(d) if x.startswith(drop))
d[:] = [x for x in d if not x.startswith(drop)]
d[i:i] = [
    "基盤のインターフェース：コマンドと protocol。どこで動かしても、同じ形で受けて同じ結果を返す",
    "入る側のアダプター：いまは CLI ・ MCP（標準入出力）。クラウドでは、その実行環境から呼び出しを受けて基盤のコマンドに直すアダプターを足す",
    "出る側のアダプター：いまはファイル。クラウドでは保存先のアダプターを足す。core は入出力に触らないので変えない（Q3）",
    "クラウドで動かすことは前提で、作るのは後にする。いまはこの境界を守り、載せるときはアダプターを足すだけで済む作りにする",
    "ほかの言語の Concrete も、バイナリを呼んで基盤を使う。言語に依らないことは、バイナリと protocol で満たす",
]
q4["checks"] = []
wasm_grounds = ("WIT は部品どうし", "部品は宣言した関数", "Component Model が閉じた", "wit-bindgen は", "wasmtime を Rust")
q4["grounds"] = [g for g in q4["grounds"] if not g["supports"].startswith(wasm_grounds) and g["supports"] != "確かめてほしい箇所を普通の言葉で書くこと"]
q4["grounds"] += [user("振る舞いは同じで、アダプターの接続先だけを変えること", U_SAME), user("基盤のインターフェースはコマンドであること", U_IF), user("クラウドで動かすことは前提であること", U_CLOUD), user("ほかの言語もバイナリを呼ぶこと", U_WASM)]
q4["rejected"].append({"option": "WASM を基盤のインターフェースとして持つ（第5〜22回）", "reason": "ほかの言語からはバイナリで呼べ、言語に依らないことは protocol で満たせる。WASM でしかできないのは、プロセスを起動できない場所で動かすことだけである"})
for im in q4["images"]:
    if im.get("name") == "protocol":
        im["name"] = "入口と保存先"
        im["caption"] = "入る側と出る側のアダプターを差し替えれば、手元でもクラウドでも基盤は同じ振る舞いになる。"
        im["svg"] = (HERE / "q4-adapters.svg").read_text()
q3 = T["Q3"]
for im in q3["images"]:
    if im.get("name") == "Rust の構成":
        im["svg"] = (HERE / "q3-rust.svg").read_text()
    if im.get("kind") == "tree":
        im["rows"] = [r for r in im["rows"] if r["path"] not in ("wasm.rs", "src/schema-driven/crates/adapters/src/inbound/")]
q0 = T["Q0"]; d0 = q0["answer"]["details"]
k = next(n for n, x in enumerate(d0) if x.startswith("原則4 の拡張には"))
d0[k] = "原則4 の拡張には、外から足す解釈の担い手も含める。担い手は Agents 契約と同じくプロセスとして呼ぶ。構想は、担い手の登録と実行の権限を管理するとしている"
for im in q0["images"]:
    if im.get("name") == "境界と向き":
        im["svg"] = (S / "dir23.svg").read_text()
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、基盤のインターフェースをコマンドと protocol にし、アダプターの差し替えで手元でもクラウドでも同じ振る舞いにする形へ直した。WASM を外した"}},
         {"op": "replace", "path": "/topics", "value": list(T.values())}]
print(json.dumps(patch, ensure_ascii=False))
