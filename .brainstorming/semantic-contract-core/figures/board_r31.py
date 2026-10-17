# 第31回：Q5（決着）で決めた名前に、Q0 ・ Q1〜Q4 を揃える（words_r31.py）。Q5 ・ 原典の引用 ・ 構想 v0.3 の例のファイルは変えない。
import json, pathlib, sys
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent; S = pathlib.Path(sys.argv[1])
sys.path.insert(0, str(HERE)); from words_r31 import fix
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
SVG = {("Q0", "設計原則"): "q0-principles", ("Q2", "標準と拡張"): "q2-concepts", ("Q3", "Rust の構成"): "q3-rust",
       ("Q3", "Interpreter と結果"): "q2-verifiers", ("Q4", "入口と保存先"): "q4-adapters", ("Q1", "仕様の対応"): "q1-specs"}
SKIP = {"basis", "svg", "html", "before", "after"}
def walk(o):
    if isinstance(o, dict):
        keep = o.get("kind") == "code" and not (str(o.get("path", "")).startswith(".brainstorming/semantic-contract-core/proposals/") or o.get("name") == "AI の呼び方")
        return {k: (v if k in SKIP or (keep and k == "text") else walk(v)) for k, v in o.items()}
    if isinstance(o, list):
        return [walk(v) for v in o]
    return fix(o) if isinstance(o, str) else o
topics = []
for t in board["topics"]:
    if t["id"] == "Q5":
        topics.append(t); continue
    for im in t["images"]:
        src = SVG.get((t["id"], im.get("name")))
        if src:
            im["svg"] = (HERE / f"{src}.svg").read_text()
        if im.get("kind") == "screen":
            im["html"] = (HERE / "q0-screen.html").read_text()
        if im.get("kind") == "code" and str(im.get("path", "")).startswith(".brainstorming/semantic-contract-core/proposals/"):
            name = pathlib.Path(fix(im["path"])).name
            im["text"] = (BOARD / "proposals" / name).read_text().rstrip()
        if t["id"] == "Q0" and im.get("name") == "境界と向き":
            im["svg"] = fix(im["svg"])
            (S / "dir31.svg").write_text(im["svg"])
    topics.append(walk(t))
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "Q5（決着）で決めた名前に、Q0 ・ Q1〜Q4 の文 ・ 図 ・ 案のファイルを揃えた"}},
         {"op": "replace", "path": "/topics", "value": topics}]
print(json.dumps(patch, ensure_ascii=False))
