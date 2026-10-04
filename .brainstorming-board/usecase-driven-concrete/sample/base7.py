# 基盤の道具の見本（ボード schema-driven-base の論点1 ・ 2）。開発に限らない。欄の名前を持たず、注釈だけを読む
#   x-ref    ── 従属関係。指す先がある ・ 指す先の種類 ・ 用語集の語の種類 ・ 扱う欄 ・ 重なり ・ 指される数 ・ 承認のあとの変化
#   x-derive ── 決まりに従って、答えから判定を出す
import json, os, re, glob, hashlib
import gen6 as g  # 判定（x-derive）を当てる derive と、語を引く word だけを使う
H = os.path.dirname(os.path.abspath(__file__))
SCH = {os.path.basename(f)[:-12]: json.load(open(f, encoding="utf-8")) for f in glob.glob(os.path.join(H, "schema", "*.schema.json"))}
D = g.D
def use(state):
    global D
    D = state; g.use(state)
def fp(o): return hashlib.sha256(json.dumps(o, ensure_ascii=False, sort_keys=True).encode()).hexdigest()[:8]

def deref(sch, root):
    """$ref と allOf をたどり、注釈と形を1つにまとめる。外のファイルへ移ったら、その根を返す"""
    out = {}
    while True:
        if "allOf" in sch:
            for x in sch["allOf"]:
                o2, root2 = deref(x, root); out.update(o2); root = root2
            out.update({k: v for k, v in sch.items() if k != "allOf"}); return out, root
        if "$ref" in sch:
            r = sch["$ref"]
            if r.startswith("#/$defs/"): base = root["$defs"][r.split("/")[-1]]
            else:
                f, frag = r.split("#"); root = SCH[f[:-12]]; base = root["$defs"][frag.split("/")[-1]]
            out.update({k: v for k, v in sch.items() if k != "$ref"}); sch = base; continue
        return dict(sch, **out), root

def get(o, path):
    for k in path.split("/"):
        if not isinstance(o, dict) or k not in o: return None
        o = o[k]
    return o

def walk(decl, item_ann=()):
    """宣言を、その種類のスキーマと並べて歩き、注釈の付いた場所を返す。item_ann は、項目に付く注釈の名前（基盤は持たず、使う側が渡す）"""
    out = []
    def visit(sch, val, chain, root):
        sch, root = deref(sch, root)
        if "x-ref" in sch: out.append(("ref", sch["x-ref"], val, chain))
        if isinstance(val, dict):
            ch = chain + [val] if "id" in val or not chain else chain
            for a in item_ann:
                if a in sch: out.append((a, sch[a], val, chain))
            for k, sub in sch.get("properties", {}).items():
                if k in val: visit(sub, val[k], ch, root)
        elif isinstance(val, list) and "items" in sch and not ("x-ref" in sch):
            for x in val: visit(sch["items"], x, chain, root)
    visit(SCH[decl["kind"]], decl, [], SCH[decl["kind"]])
    return out

def src_of(decl, chain):
    ids = [c["id"] for c in chain[1:] if isinstance(c, dict) and "id" in c]
    return ".".join([decl["id"]] + ids)

def target(ann, decl, v):
    """指す先を引く。(指す先の文字列, 指す先, 種類が合うか)"""
    to = ann["to"]
    if to == "self":
        xs = get(decl, ann["in"]) or []
        hit = [x for x in xs if x.get("id") == v]
        return f'{decl["id"]}.{v}', (hit[0] if hit else None), True
    if to == "glossary":
        for d in D.values():
            if d["kind"] == "glossary":
                hit = [t for t in (get(d, ann["in"]) or []) if t["id"] == v]
                if hit: return v, hit[0], True
        return v, None, True
    if ann.get("bare"):
        for d in D.values():
            if d["kind"] == to:
                for x in find_all(d):
                    if x.get("id") == v: return f'{d["id"]}.{v}', x, True
        return v, None, True
    head = v.split(".")[0]; d = D.get(head)
    if d is None: return v, None, True
    if d["kind"] != to: return v, d, False
    cur = d
    for i in v.split(".")[1:]:
        cur = next((x for x in find_all(cur) if x.get("id") == i), None)
        if cur is None: break
    return v, cur, True

def find_all(o):
    if isinstance(o, dict):
        if "id" in o: yield o
        for v in o.values(): yield from find_all(v)
    elif isinstance(o, list):
        for v in o: yield from find_all(v)

def links():
    """従属関係の一覧（参照元 ・ 指す先 ・ 注釈 ・ 引けた先 ・ 種類が合うか ・ 宣言）"""
    out = []
    for k, d in D.items():
        if d["kind"] not in SCH: continue
        for typ, ann, val, chain in walk(d):
            if typ != "ref" or val in (None, "", []): continue
            item = chain[-1] if chain else d
            for v in (val if isinstance(val, list) else [val]):
                if not isinstance(v, str) or (ann.get("only") and not re.match(ann["only"], v)): continue
                ref, hit, kind_ok = target(ann, d, v)
                out.append({"src": src_of(d, chain), "ann": ann, "v": v, "ref": ref, "hit": hit, "kind_ok": kind_ok, "decl": k, "item": item, "chain": chain})
    return out

def record(): return {f'{l["src"]}→{l["ref"]}': fp(l["hit"]) for l in links() if l["ann"]["to"] not in ("glossary", "self") and l["hit"] is not None}

def checks(rec=None):
    R = []
    def put(cat, name, src, dst, text, st): R.append({"category": cat, "check": name, "from": src, "to": dst, "text": text, "status": st})
    L = links()
    bad = [l for l in L if l["hit"] is None]
    for l in bad: put("structure", "指す先がある", l["src"], l["v"], f'{l["v"]} が無い', "ずれ")
    put("structure", "指す先がある", "", "", f'{len(L)}本の従属関係のうち、指す先が無いものは {len(bad)}本', "合格" if not bad else "ずれ")
    wrong = [l for l in L if not l["kind_ok"]]
    for l in wrong: put("structure", "指す先の種類が注釈どおり", l["src"], l["v"], f'{l["ann"]["to"]} を指すはずの欄が、{l["hit"]["kind"]} を指している', "ずれ")
    put("structure", "指す先の種類が注釈どおり", "", "", f'種類が違う指す先は {len(wrong)}本（要求の側が設計の側を指す ・ 依存の向きの違反もここに出る）', "合格" if not wrong else "ずれ")
    # 用語集の語の種類
    for l in L:
        m = l["ann"].get("meaning")
        if not m or l["hit"] is None: continue
        w = l["ann"].get("when")
        if w and any(l["item"].get(f) not in vs for f, vs in w.items()): continue
        a = l["ann"]; kinds = {x[a["meaning_key"]] for x in l["hit"][a["meanings"]]}
        if not kinds & set(m): put("structure", "名前が用語を指す", l["src"], l["v"], f'「{l["hit"][a["label"]]}」の語の種類が {"・".join(sorted(kinds))}（{"・".join(m)} のどれかのはず）', "ずれ")
    # 同じ並びに重ねない
    seen = {}
    for l in L:
        if l["ann"].get("unique"): seen.setdefault((l["decl"], id(l["ann"])), []).append(l["v"])
    for (k, _), vs in seen.items():
        dup = sorted({v for v in vs if vs.count(v) > 1})
        put("structure", "同じ並びに同じ語を重ねない", k, "・".join(dup), (f'{len(set(vs))}語とも、1つだけ使う' if not dup else f'{"・".join(g.word(x) for x in dup)} を2回使っている'), "合格" if not dup else "ずれ")
    # 扱う欄
    for l in L:
        cb = l["ann"].get("covered_by")
        if not cb: continue
        a, b = cb.split("/")
        got = {x for e in (l["item"].get(a) or []) for x in (e.get(b) or [])}
        ok = l["v"] in got
        put("structure", "確かめた条件の失敗を扱う", l["src"], l["v"], "拡張が扱っている" if ok else "確かめる条件が成り立たないときの拡張が無い", "合格" if ok else "欠け")
    # 指される数
    groups = {}
    for l in L:
        inv = l["ann"].get("inverse")
        if inv: groups.setdefault(inv["group"], {"inv": inv, "ann": l["ann"], "refs": []})["refs"].append(l)
    for name, gr in groups.items():
        inv, ann = gr["inv"], gr["ann"]
        if "max_decls" in inv:
            by = {}
            for l in gr["refs"]: by.setdefault(l["ref"], set()).add(l["decl"])
            for t, ds in by.items():
                put("structure", name, "・".join(sorted(ds)), t, f'{len(ds)}つの宣言が使う', "合格" if len(ds) <= inv["max_decls"] else "ずれ")
            continue
        cands = candidates(ann, gr["refs"])
        for key, obj in cands:
            wh = inv.get("where")
            if wh and any(get(obj, p) not in vs for p, vs in wh.items()): continue
            n = sum(1 for l in gr["refs"] if l["ref"] == key)
            st = "欠け" if n < inv.get("min", 0) else "ずれ" if "max" in inv and n > inv["max"] else "合格"
            who = "・".join(sorted({l["src"] for l in gr["refs"] if l["ref"] == key}))
            put("structure", name, key, who, f'指しているのは {n}つ', st)
    # 問いの答えからの判定と、宣言した値
    for k, d in D.items():
        sch = SCH.get(d["kind"], {})
        for p, ps in sch.get("properties", {}).items():
            dv = ps.get("x-derive")
            if dv and dv.get("declared") and isinstance(d.get(p), dict):
                got = g.derive(p, d[p]); dec = d[p].get(dv["declared"])
                put("structure", f'{dv["title"]}と宣言が合う', k, "", f'{dv["title"]}は {got}、宣言は {dec}', "合格" if got == dec else "ずれ")
    # 承認のあとの変化
    Cr = {} if rec is None else rec
    for l in L:
        if l["ann"]["to"] in ("glossary", "self") or l["hit"] is None: continue
        key = f'{l["src"]}→{l["ref"]}'; now = fp(l["hit"])
        st = "確かめ直し" if key not in Cr or Cr[key] != now else "合格"
        put("change", "上流の変更", l["src"], l["ref"], ("まだ承認していない対応" if key not in Cr else "承認のあとで参照先が変わった" if Cr[key] != now else "承認した時点から変わっていない"), st)
    return R

def candidates(ann, refs):
    """指される側になりうるものの一覧（ID ・ 中身）"""
    to = ann["to"]
    if to == "self":
        out = []
        for k in {l["decl"] for l in refs}:
            out += [(f'{k}.{x["id"]}', x) for x in (get(D[k], ann["in"]) or [])]
        return out
    ds = [d for d in D.values() if d["kind"] == to]
    if ann.get("in"): return [(f'{d["id"]}.{x["id"]}', x) for d in ds for x in (get(d, ann["in"]) or [])]
    return [(d["id"], d) for d in ds]

