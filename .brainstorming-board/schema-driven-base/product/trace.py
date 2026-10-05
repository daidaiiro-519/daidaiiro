# テスト条件とテストの実行記録を照合する（開発の concrete の道具の試作）。
# 使い方：python3 trace.py <記録ファイル（1行1件の JSON）> <回の番号>
# 1〜n 回目の宣言のテスト条件について、欠け ・ 余り ・ 古い ・ レベル違いを数え、1件でもあれば終了コード 1 を返す
import json, glob, os, sys
H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(H, "..", "..", "usecase-driven-concrete", "sample"))
import concrete7

RANK = {"component": 0, "component-integration": 1, "system": 2}

def load_conditions():
    D = {}
    for f in glob.glob(os.path.join(H, "decls", "*.json")):
        d = json.load(open(f, encoding="utf-8")); D[d["id"]] = d
    concrete7.use(D)
    return {c["id"]: c for c in concrete7.conditions()}

def scope(n):
    R = json.load(open(os.path.join(H, "rounds.json"), encoding="utf-8"))["rounds"]
    return {k for r in R if r["no"] <= n for k in r["decls"]}

def match(conds, records, decls):
    """記録とテスト条件を照合し、4種類の照合結果を返す"""
    out = {"欠け": [], "余り": [], "古い": [], "レベル違い": []}
    by = {}
    for r in records: by.setdefault(r["condition"], []).append(r)
    for cid, rs in sorted(by.items()):
        if cid not in conds: out["余り"].append(cid); continue
        c = conds[cid]
        if not any(r["hash"] == c["hash"] for r in rs): out["古い"].append(cid)
        elif not any(r["hash"] == c["hash"] and RANK[r["level"]] >= RANK[c["required_level"]] for r in rs): out["レベル違い"].append(cid)
    for cid, c in sorted(conds.items()):
        if c["decl"] in decls and cid not in by: out["欠け"].append(cid)
    return out

if __name__ == "__main__":
    path, n = sys.argv[1], int(sys.argv[2])
    records = [json.loads(l) for l in open(path, encoding="utf-8") if l.strip()] if os.path.exists(path) else []
    conds = load_conditions(); decls = scope(n)
    r = match(conds, records, decls)
    total = sum(1 for c in conds.values() if c["decl"] in decls)
    print(f"{n} 回目まで：テスト条件 {total} 件 ・ 記録 {len(records)} 行")
    for k, v in r.items():
        print(f"  {k} {len(v)} 件" + (("：" + " ・ ".join(v[:12]) + (" …" if len(v) > 12 else "")) if v else ""))
    sys.exit(1 if any(r.values()) else 0)
