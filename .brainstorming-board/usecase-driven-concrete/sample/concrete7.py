# 開発の concrete の道具の見本（ユースケース駆動）。基盤の道具（base7）の上で、x-test-spec からテスト条件を取り出す
#   x-test-spec ── テスト条件の取り出し。ID ・ 確かめること ・ 求めるレベル ・ 期待する結果のハッシュ値
#   テストの記録（trace.schema.json）との突き合わせ（欠け ・ 余り ・ 古い ・ レベル違い）も、この側が持つ
import base7
from base7 import fp, get, walk, links, record, checks, SCH
import gen6 as g
def use(state):
    base7.use(state)
def impl_of(k, key, seen=None):
    D = base7.D
    """前向きの従属関係をたどって、問いの答えから判定を持つ宣言に着いたら、その判定を返す"""
    seen = seen or set()
    if k in seen or k not in D: return None
    seen.add(k); d = D[k]
    for p, ps in SCH.get(d["kind"], {}).get("properties", {}).items():
        if ps.get("x-derive") and p == key and isinstance(d.get(p), dict): return g.derive(p, d[p])
    for l in LINKS_BY.get(k, []):
        head = l["ref"].split(".")[0]
        if l["ann"]["to"] not in ("glossary", "self"):
            r = impl_of(head, key, seen)
            if r: return r
    return None
LINKS_BY = {}

def conditions():
    global LINKS_BY
    D = base7.D
    L = links(); LINKS_BY = {}
    for l in L: LINKS_BY.setdefault(l["decl"], []).append(l)
    out = []
    for k, d in D.items():
        if d["kind"] not in SCH: continue
        for typ, sp, item, chain in walk(d, ("x-test-spec",)):
            if typ != "x-test-spec": continue
            w = sp.get("when")
            if w and any((get(item, p) in c["not"]) if isinstance(c, dict) else (get(item, p) not in c) for p, c in w.items()): continue
            parent = next((c for c in reversed(chain) if isinstance(c, dict) and "id" in c and c is not item and c is not d), None)
            iid = item.get("id")
            cid = f'{k}.{sp["id"]}' if sp["id"] not in ("decl", "parent") else f'{k}.{parent["id"]}.{iid}' if sp["id"] == "parent" and parent else f'{k}.{iid}'
            if sp.get("suffix"): cid += "." + sp["suffix"]
            chk = sp.get("checks") or sp["checks_by"]["map"].get(item.get(sp["checks_by"]["field"]), sp["checks_by"]["map"]["*"])
            lv = sp["level"]
            if isinstance(lv, dict): lv = lv["map"].get(impl_of(k, lv["derive"]) or "ドメインモデル", "component")
            withs = list(sp.get("with", [])) + list((sp.get("with_by") or {}).get("map", {}).get(item.get((sp.get("with_by") or {}).get("field", "")), []))
            refs = [fp(l["hit"]) for l in L if l["decl"] == k and l["hit"] is not None and l["ann"]["to"] not in ("glossary",) and any(c is item for c in l["chain"] + [l["item"]])]
            expect = {"item": item, "with": [get(d, p) for p in withs], "parent": [parent.get(x) for x in sp.get("with_parent", [])] if parent else [], "refs": sorted(refs)}
            out.append({"id": cid, "label": iid or sp["id"], "kind": d["kind"], "checks": chk, "hash": fp(expect), "required_level": lv, "decl": k, "anchor": (parent["id"] if sp["id"] == "parent" and parent else iid or sp["id"])})
    return out

if __name__ == "__main__":
    import collections
    r = checks(); print(collections.Counter(x["status"] for x in r))
    for x in r:
        if x["status"] not in ("合格", "確かめ直し"): print(x)
    c = conditions(); print("テスト条件", len(c), collections.Counter(x["kind"] for x in c))
