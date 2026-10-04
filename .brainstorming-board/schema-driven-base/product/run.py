# schema-driven の宣言に、見本の道具（スキーマ検証 ・ concrete7）を外から当てる
import json, glob, os, sys, collections, subprocess
H = os.path.dirname(os.path.abspath(__file__))
S = os.path.join(H, "..", "..", "usecase-driven-concrete", "sample")
sys.path.insert(0, S)
print(subprocess.run([sys.executable, os.path.join(S, "schema", "check.py"), os.path.join(H, "decls")], capture_output=True, text=True).stdout.strip().splitlines()[-1])
import concrete7
D = {}
for f in sorted(glob.glob(os.path.join(H, "decls", "*.json"))):
    d = json.load(open(f, encoding="utf-8")); D[d["id"]] = d
concrete7.use(D)
R = concrete7.checks()
print(collections.Counter(x["status"] for x in R))
for x in R:
    if x["status"] not in ("合格", "確かめ直し"): print(" ", x["status"], x["check"], x["from"], x["to"], x["text"])
C = concrete7.conditions(); print("テスト条件", len(C), collections.Counter(c["kind"] for c in C))
