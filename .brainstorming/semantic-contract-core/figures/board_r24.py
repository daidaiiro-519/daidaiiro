# 第24回：廃語「担い手」を、ボードの文 ・ 図 ・ 案のファイルで普通の言葉へ置き換える（words_r24.py）。利用者の言葉と原典の引用（grounds の basis）は変えない。
import json, pathlib, sys
HERE = pathlib.Path(__file__).parent; BOARD = HERE.parent
sys.path.insert(0, str(HERE)); from words_r24 import fix
board = json.loads((BOARD.parent / "semantic-contract-core.json").read_text())
U = "外から足す担い手\nこの変な言い回し何とかならない？？何なん担い手って・・・・"
SVG = {("Q0", "設計原則"): "q0-principles", ("Q2", "標準と拡張"): "q2-concepts", ("Q3", "Rust の構成"): "q3-rust", ("Q3", "担い手と結果"): "q2-verifiers"}
def walk(o, key=None):
    if isinstance(o, dict):
        return {k: (v if k == "basis" else walk(v, k)) for k, v in o.items()}
    if isinstance(o, list):
        return [walk(v, key) for v in o]
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
    t = walk(t)
    topics.append(t)
q0 = next(t for t in topics if t["id"] == "Q0")
q0["grounds"].append({"supports": "説明の無い言い回しを使わないこと", "basis": U, "tag": "primary", "source": "利用者の言葉（2026-10-17）"})
left = [s for s in json.dumps(topics, ensure_ascii=False).split("担い手")][1:]
patch = [{"op": "add", "path": "/log/0", "value": {"date": "2026-10-17", "text": "利用者の指摘で、説明の無い言い回し「担い手」を、確かめるもの ・ 検査のプログラムなどの普通の言葉へ置き換えた"}},
         {"op": "replace", "path": "/topics", "value": topics}]
print(json.dumps(patch, ensure_ascii=False))
print(len(left), file=sys.stderr)
