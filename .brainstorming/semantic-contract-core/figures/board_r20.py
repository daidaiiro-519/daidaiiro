# 第20回：利用者の指摘（HTTP は Concrete のコマンドを載せる CQRS 的な形。最終的にクラウドで動かす）で Q4 を組み直す。
# 根拠は platform-advisor の回答（advisors/platform-http-cloud.json）と、Lambda の原文（sources/cloud/、fact-check で照合）。
import json, pathlib, sys
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
adv = json.loads((BOARD / "advisors/platform-http-cloud.json").read_text())
SRC = "platform-advisor の回答（2026-10-17、HTTP とクラウド）。advisors/platform-http-cloud.json"
CL = BOARD / "sources/cloud"
def q(text, start, end):
    i = text.find(start); j = text.find(end, i)
    if i < 0 or j < 0: sys.exit(f"原文に無い：{start}")
    return text[i:j + len(end)]
user = lambda s, w: {"supports": s, "basis": w, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
paths = {p["ask"]: p["answer"] for p in adv["path"]}
pa = lambda k: paths[next(x for x in paths if x.startswith(k))]
g = lambda s, text, a, b: {"supports": s, "basis": q(text, a, b), "tag": "primary", "source": SRC}
def cloud(s, f, a, b, line):
    return {"supports": s, "basis": q((CL / f).read_text(), a, b), "tag": "primary", "source": f"{f}:{line}（2026-10-17 取得、fact-check で照合）"}
U_CQRS = "ここにさらにHTTPが加わるならそのコンクレートのコマンドにしたやつを呼び出すようなCQRSてきなHTTPなんじゃないのか？"
U_CLOUD = "ちなみにこれ今ローカルだけど、最終的にクラウド基盤に乗っけても動くようにしたいわけだが、その場合はどうなるんだ？"
T = {t["id"]: t for t in board["topics"]}
q4 = T["Q4"]; d = q4["answer"]["details"]
def rep(start, text):
    i = next(k for k, x in enumerate(d) if x.startswith(start)); d[i] = text; return i
q4["answer"]["text"] = "protocol を運び方から切り離して JSON Schema で定義し、基盤は CLI ・ MCP ・ WASM で運ぶ。HTTP は Concrete が自分のコマンドの一覧をそのまま載せ、手元でもクラウドでも同じ (終了コード, JSON) を返す。基盤は足すだけの版で出し、4つの測り方がどれも0件なら完成とする。"
i = rep("HTTP：基盤は待ち受けない", "HTTP：基盤は待ち受けない。Concrete が自分のコマンドの一覧をそのまま載せる。読む操作（get ・ check ・ inspect など）は GET、書く操作（create ・ update ・ approve ・ advance など）は POST")
rep("基盤が持つのは HTTP への載せ方", "基盤が持つのは HTTP への載せ方：経路はコマンドの一覧から作り、本文はコマンドが返す JSON のまま、状態は終了コードで決める（0→200、1→422、2→400）。適合の組は (終了コード, JSON) で持つ")
rep("Concrete が HTTP で受けるときの守り", "手元の守り（127.0.0.1 ・ 起動ごとの合言葉 ・ Host と Origin）は手元だけのもの。クラウドの入口では TLS と利用者の認証を持ち、読む操作と書く操作で許可を分ける")
rep("基盤が HTTP で受ける形を検討し直すのは", "クラウド：Lambda は 2025-11-14 から Rust に正式に対応し、OS のみのランタイム provided.al2023 に載せる。HTTP は関数の URL か API Gateway で受ける")
d.insert(i + 1, "画面専用の経路のうち、ボードを読み書きするもの（状態 ・ 保留 ・ 戻す ・ 回答を書く）はコマンドにする。AI の CLI を動かすチャットはコマンドにせず、手元の serve にだけ残す")
j = next(k for k, x in enumerate(d) if x.startswith("クラウド：Lambda"))
d.insert(j + 1, "クラウドの中身：core の crate を関数に組み込み、Files のポートを保存先のアダプターで満たす。update の hash による競合の検出は、保存先の条件付きの書き込みへ移す")
d.insert(j + 2, "中身を考え直すのは：保存先が条件付きの書き込みを持たないとき、サーバー側で AI の CLI を動かしたいとき（コンテナにする）、ディレクトリ全体を読む check ・ render が遅すぎるとき")
checks = [c for c in q4["checks"] if c["target"] == "WASM"]
q4["checks"] = [
    {"target": "HTTP", "ask": "HTTP は Concrete のコマンドの一覧をそのまま載せ、読む操作は GET、書く操作は POST にすることでよいか",
     "view": "コマンドの一覧をそのまま載せる。チャットだけは手元に残す", "basis": ["platform-advisor の回答", "brainstorming の cli.rs"],
     "why": "チャットのような画面専用の操作を、どこまでコマンドにしたいか"},
    {"target": "クラウドの中身", "ask": "クラウドでは core の crate を関数に組み込み、保存先のアダプターを使うことでよいか",
     "view": "関数に組み込み、状態は保存先に置く", "basis": ["platform-advisor の回答", "Lambda の Rust 対応（AWS）"],
     "why": "クラウドの保存先に何を使うか（S3 か、データベースか）"},
] + checks
q4["grounds"] += [
    user("HTTP は Concrete のコマンドを載せる形にすること", U_CQRS),
    user("最終的にクラウドで動かすこと", U_CLOUD),
    g("コマンドの一覧をそのまま載せる形", pa("コマンドの一覧をそのまま"), "採る。経路は POST /<コマンド名> と GET /<コマンド名>?<引数> の2つの形だけにし", "書く操作（create ・ update ・ delete ・ approve ・ render ・ advance ・ unlock）は POST にする。"),
    g("クラウドの中身を関数に組み込むこと", pa("クラウドの中身は"), "B。関数は要求のあいだに一時の領域のデータを当てにできない前提で作る", "その層で競合の検出が抜ける。"),
    cloud("Lambda が Rust に正式に対応したこと", "aws.amazon.com_about-aws_whats-new_2025_11_aws-lambda-rust", "With this launch, Rust support in Lambda is now Generally Available", "backed by AWS Support and the Lambda SLA.", 1625),
    cloud("Rust は OS のみのランタイムに載せること", "docs.aws.amazon.com_lambda_latest_dg_lambda-rust.html.md", "Because Rust compiles to native code", "using an [OS-only runtime](runtimes-provided.md).", 6),
    cloud("HTTP で呼ぶ手段", "docs.aws.amazon.com_lambda_latest_dg_urls-configuration.html.md", "Lambda offers two ways to invoke your function through an HTTP endpoint", "function URLs and Amazon API Gateway.", 9),
]
q4["rejected"] += [
    {"option": "クラウドでも、同梱したバイナリを関数の中で起動する", "reason": "関数は状態を持てないので、ファイルを写して戻す層が増え、そこで競合の検出が抜ける"},
    {"option": "クラウドで WASM にして動かす", "reason": "保存先のアダプターは結局要り、そのうえに WASM の実行環境の層が増えるだけになる"},
    {"option": "画面専用の経路（/api/send ・ /api/chat など）をそのまま残す", "reason": "コマンドの一覧と1対1でなく、クラウドの呼び手が同じ操作を使えない"},
]
for im in q4["images"]:
    if im.get("name") == "protocol":
        im["caption"] = "1つの protocol を、基盤の CLI ・ MCP ・ WASM と、Concrete のコマンドを載せた HTTP が同じ形で運ぶ。"
        im["svg"] = (HERE / "q4-protocol.svg").read_text()
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、HTTP を Concrete のコマンドの一覧を載せる形にし、クラウドで動かす前提で Q4 を組み直した"}},
         {"op": "replace", "path": "/topics", "value": list(T.values())}]
print(json.dumps(patch, ensure_ascii=False))
