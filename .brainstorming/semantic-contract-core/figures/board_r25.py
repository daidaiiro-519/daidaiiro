# 第25回：和語への言い換えをやめ、原典の用語（Interpreter ・ 検証 ・ 投影 ・ Capability ・ outcome）と技術用語（inbound ・ outbound アダプター ・ ルール）に戻す（words_r25.py）。
# 原典の引用 ・ 実物のファイルの差分と抜粋 ・ 原典の例は変えない。
import json, pathlib, sys
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent; S = pathlib.Path(sys.argv[1])
sys.path.insert(0, str(HERE)); from words_r25 import fix
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
U = "確かめるものって表現も何でそんな変な言いかえをしているんですかね？？\nいつになったらこの変な言い回しが撲滅するんだろう。。"
SVG = {("Q0", "設計原則"): "q0-principles", ("Q2", "標準と拡張"): "q2-concepts", ("Q3", "Rust の構成"): "q3-rust",
       ("Q3", "確かめるものと結果"): "q2-verifiers", ("Q4", "入口と保存先"): "q4-adapters", ("Q1", "仕様の対応"): "q1-specs"}
SKIP = {"basis", "svg", "html", "before", "after"}
def walk(o):
    if isinstance(o, dict):
        keep_text = o.get("kind") == "code" and not (str(o.get("path", "")).startswith(".brainstorming/semantic-contract-core/proposals/") or o.get("name") == "AI の呼び方")
        return {k: (v if k in SKIP or (keep_text and k == "text") else walk(v)) for k, v in o.items()}
    if isinstance(o, list):
        return [walk(v) for v in o]
    return fix(o) if isinstance(o, str) else o
topics = []
for t in board["topics"]:
    for im in t["images"]:
        src = SVG.get((t["id"], im.get("name")))
        if src:
            im["svg"] = (HERE / f"{src}.svg").read_text()
        if im.get("kind") == "screen":
            im["html"] = (HERE / "q0-screen.html").read_text()
        if im.get("kind") == "code" and str(im.get("path", "")).startswith(".brainstorming/semantic-contract-core/proposals/"):
            im["text"] = (BOARD / "proposals" / pathlib.Path(im["path"]).name).read_text().rstrip()
        if t["id"] == "Q0" and im.get("name") == "境界と向き":
            im["svg"] = fix(im["svg"])
    topics.append(walk(t))
next(t for t in topics if t["id"] == "Q0")["grounds"].append({"supports": "原典の用語と技術用語をそのまま使うこと", "basis": U, "tag": "primary", "source": "利用者の言葉（2026-10-17）"})
(S / "dir25.svg").write_text(next(im for im in next(t for t in topics if t["id"] == "Q0")["images"] if im.get("name") == "境界と向き")["svg"])
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、和語への言い換えをやめ、原典の用語（Interpreter ・ 検証 ・ 投影 ・ Capability ・ outcome）と技術用語（inbound ・ outbound アダプター ・ ルール）に戻した"}},
         {"op": "replace", "path": "/topics", "value": topics}]
print(json.dumps(patch, ensure_ascii=False))
