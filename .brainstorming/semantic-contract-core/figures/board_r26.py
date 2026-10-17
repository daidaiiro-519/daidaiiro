# 第26回：利用者の指摘で、構想 v0.3 を原典でなく「検討中の構想（Codex でのブレスト）」として扱い、名前を決める論点 Q5 を Q0 の次に足す。
import json, pathlib, sys
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
SRC = BOARD / "sources"
def q(path, start, end):
    t = " ".join(l.strip() for l in path.read_text().split("\n"))
    i = t.find(start); j = t.find(end, i)
    if i < 0 or j < 0: sys.exit(f"原文に無い：{start}")
    return t[i:j + len(end)]
def g(sup, path, a, b, src): return {"supports": sup, "basis": q(path, a, b), "tag": "primary", "source": src}
user = lambda s, w: {"supports": s, "basis": w, "tag": "primary", "source": "利用者の言葉（2026-10-17）"}
COCK = next(p for p in (SRC / "terms").iterdir() if not p.name.endswith(".meta.json"))
DRAFT = "構想 v0.3（利用者が Codex でブレストした検討中の案）"
# 構想 v0.3 の引用は「原典」でなく「前提」にする
for t in board["topics"]:
    for gr in t["grounds"]:
        if gr["source"].startswith("構想 v0.3"):
            gr["tag"] = "assumption"
            gr["source"] = gr["source"].replace("構想 v0.3 の", DRAFT + " の", 1)
q5 = {
 "id": "Q5", "name": "用語", "status": "open",
 "question": "基盤で使う名前（検証する側 ・ 検証の結果 ・ できることの宣言 ・ アダプター ・ 書くときの条件 ・ 意味を検証する部品）を何にするか",
 "answer": {
  "text": "名前は、標準の仕様と基盤のコードが使う語に揃え、構想 v0.3 の名前は候補として扱う。検証する側は validator、結果は検証結果、宣言は capability、アダプターはインバウンド ・ アウトバウンド、書くときの条件はルール、意味を検証する部品は Semantic Validator とする。",
  "details": [
   "validator：検証する側（エンジン ・ AI ・ 人）。JSON Schema と SHACL の仕様が使う語",
   "検証結果：検証の結果（validation result）。値は verified ・ violated ・ unverified ・ unsupported ・ error の5つ",
   "capability：validator が何を検証できるかの宣言。MCP の仕様が、接続の最初に宣言し合う意味で使う",
   "インバウンドアダプター ・ アウトバウンドアダプター：基盤を呼ばれる側と、基盤から外を呼ぶ側。基盤のコードのフォルダ名 inbound ・ outbound と同じにする",
   "ルール：スキーマに書く条件1件（id ・ rule ・ why ・ test）。基盤のスキーマのキーが rule である",
   "Semantic Validator：意味のルールごとに validator を選んで検証する部品。構造を検証する Schema Validator と並べる",
   "承認されたら、ほかの論点の名前をこの表に揃える",
  ]
 },
 "images": [{"kind": "figure", "name": "名前の対応", "caption": "決める名前と、構想 v0.3 の候補と、出どころ。", "svg": (HERE / "q5-terms.svg").read_text()}],
 "checks": [
  {"target": "validator", "ask": "検証する側の名前を validator にしてよいか", "view": "validator にする。標準の仕様が使う語である",
   "basis": ["JSON Schema の仕様", "SHACL の仕様"], "why": "構想 v0.3 の Interpreter に込めた意味（解釈 ・ 調停）を残したいか"},
  {"target": "アダプター", "ask": "インバウンド ・ アウトバウンドでよいか", "view": "コードのフォルダ名と同じにする",
   "basis": ["基盤のコード", "Cockburn の原文"], "why": "原文の primary ・ secondary（driving ・ driven）に合わせたいか"},
  {"target": "部品の名前", "ask": "Semantic Interpreter を Semantic Validator にしてよいか", "view": "Schema Validator と並べて Semantic Validator にする",
   "basis": ["名前の対応"], "why": "ルールごとに validator を選ぶ役を、名前で分けたいか"},
 ],
 "grounds": [
  user("名前をこのブレストで決めること", "あのね？構想を原典としているん見たいなんだけど、それってまだ検討段階のものですよ？？\n別にそれ私がCodexの方でブレストした内容の話で、この辺の名称も今回のブレストでちゃんと決めないといけないものだからね？？"),
  user("一般的な語かで決めること", "ルールごとの Interpreterと必要な Capabilityに関してはその英語での表現が一般的かって話ですよね。"),
  user("outcome も見直すこと", "outcomeもだよね。"),
  user("アダプターの呼び方", "普通にインバウンドとアウトバウンドじゃね？"),
  g("JSON Schema の仕様が validator と呼ぶこと", SRC / "www.ietf.org_archive_id_draft-bhutton-json-schema-01.txt", "For example, an implementation that is a validator", "SHOULD assume the", "JSON Schema 2020-12 のコア（draft-bhutton-json-schema-01:1468、2026-10-17 取得）"),
  g("SHACL の仕様が validation result と呼ぶこと", SRC / "www.w3.org_TR_shacl", "Each validation result has exactly one value for the property", "<code>sh:focusNode</code>", "SHACL（www.w3.org/TR/shacl:2520、2026-10-17 取得）"),
  g("MCP の仕様が capability を宣言し合う意味で使うこと", SRC / "mcp/modelcontextprotocol.io_specification_2025-06-18_basic_lifecycle.md", "Client and server capabilities establish which optional protocol features will be", "available during the session.", "MCP の仕様（2025-06-18 basic/lifecycle:147〜148、2026-10-17 取得、fact-check で取得）"),
  g("Cockburn の原文の呼び方", COCK, "In implementation, ports and adapters show up in two", "reasons. They could be also called", "Alistair Cockburn「Hexagonal architecture」（alistair.cockburn.us/hexagonal-architecture、1975〜1979行、2026-10-17 取得）"),
 ],
 "rejected": [
  {"option": "検証する側を Interpreter と呼ぶ（構想 v0.3）", "reason": "JSON Schema と SHACL の仕様に無い語で、何をするものかが名前から分からない"},
  {"option": "検証する側を検証器と呼ぶ", "reason": "技術用語を訳し直した語で、仕様の語と対応が取れない"},
  {"option": "検証の結果を outcome と呼ぶ（構想 v0.3）", "reason": "SHACL ・ JSON Schema とも一般には使わない"},
  {"option": "アダプターを primary ・ secondary（driving ・ driven）と呼ぶ", "reason": "原文の呼び方だが、基盤のコードは inbound ・ outbound で、同じものに2つの名前が付く"},
  {"option": "書くときの条件を制約（Constraint）と呼ぶ（構想 v0.3）", "reason": "SHACL の制約は構造の条件を指す。基盤のルールは意味の検証（test）も含み、スキーマのキーも rule である"},
 ],
}
topics = board["topics"]
topics.insert(1, q5)
for t in topics:
    if t["id"] in ("Q1", "Q2", "Q3", "Q4"):
        t["depends"] = t.get("depends", []) + ["Q5"]
queue = board["queue"] + [{"topic": "Q5", "why": "ほかの論点の答えは、どれもこの名前で書くため。構想 v0.3 の名前は検討中の案なので、ここで決める"}]
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、構想 v0.3 を検討中の案として扱い直し、名前を決める論点 Q5「用語」を足した"}},
         {"op": "replace", "path": "/queue", "value": queue},
         {"op": "replace", "path": "/topics", "value": topics}]
print(json.dumps(patch, ensure_ascii=False))
