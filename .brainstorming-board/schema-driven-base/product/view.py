# schema-driven の宣言を、見本の頁の型と部品（render6）で描画する。見本のファイルは書き換えない
import json, glob, os, sys
H = os.path.dirname(os.path.abspath(__file__))
S = os.path.abspath(os.path.join(H, "..", "..", "usecase-driven-concrete", "sample"))
sys.path.insert(0, S); sys.path.insert(0, os.path.join(S, "design"))
import data6
D = {}
for f in glob.glob(os.path.join(H, "decls", "*.json")):
    d = json.load(open(f, encoding="utf-8")); D[d["id"]] = d
order = {"domain": 0, "glossary": 1, "other_requirements": 2, "use_case": 3, "subdomain": 4, "context": 5, "aggregate": 6, "value_object": 7, "domain_service": 8}
key = lambda i: (order[D[i]["kind"]], int(i.split("-")[1]))
data6.D.clear(); data6.D.update({i: D[i] for i in sorted(D, key=key)})
import base7; base7.use(data6.D)
src = open(os.path.join(S, "render6.py"), encoding="utf-8").read()
src = src.replace("OUT='/home/daidaiiro/workspace/daidaiiro/.brainstorming-board/usecase-driven-concrete/sample'", f"OUT={os.path.join(H, 'view')!r}")
src = src.replace("json.load(open(os.path.join(HERE,'approved-record.json'),encoding='utf-8'))", "{}")
src = src.replace('<a href="https://claude.ai/artifact/MGX9MpSk6MtnCF8QpWqDdh" target="_blank" rel="noopener">突き合わせ ↗</a><a href="https://claude.ai/artifact/HwxXEHAKFcGig7f7UamAxY" target="_blank" rel="noopener">実行の記録 ↗</a>', "")
src = src.replace("<title>モバイルオーダーの宣言</title>", "<title>schema-driven の宣言</title>")
os.makedirs(os.path.join(H, "view"), exist_ok=True)
sys.argv = [os.path.join(S, "render6.py")]
exec(compile(src, os.path.join(S, "render6.py"), "exec"), {"__name__": "__main__", "__file__": os.path.join(S, "render6.py")})
