# 基盤の頁の組み立ての見本（ボード schema-driven-base の論点3）。concrete が宣言した頁の型（pages/*.page.json）を読み、
# 基盤の部品（見出しと中身の枠 ・ 表 ・ 問いと答え ・ 札 ・ リンク）で組む。欄の名前は頁の型とスキーマにだけ現れる
import json, os, glob, html
import base7, text7
import gen6 as g
H = os.path.dirname(os.path.abspath(__file__))
E = lambda s: html.escape(str(s))
TPL = {json.load(open(f))["kind"]: json.load(open(f)) for f in glob.glob(os.path.join(H, "pages", "*.page.json"))}
SCH = base7.SCH

def resolve(v):
    """参照の値を、指す先の項目にする（宣言の ID、または宣言の ID を付けない項目の ID）"""
    if v in base7.D: return base7.D[v]
    for d in base7.D.values():
        hit = next((x for x in base7.find_all(d) if x.get("id") == v and x is not d), None)
        if hit is not None: return hit
    return None

def schema_at(kind, path):
    o = SCH[kind]
    for k in path.split("/"):
        o, _ = base7.deref(o, SCH[kind]); o = o["properties"][k]
    return base7.deref(o, SCH[kind])[0]

def val(d, spec, P, item=None):
    """頁の型の値の指定を、部品で描く"""
    src = item if item is not None else d
    if "derive" in spec: v = g.derive(spec["derive"], base7.get(d, spec["derive"]))
    elif "self" in spec:
        v = item if isinstance(item, str) else (item or {}).get("id")
        if spec["self"] == "link": return P["ref"](v)
        idc = f'<span class="ln mx"><span class="no">{E(v)}</span></span>' if spec.get("show_id") else ""
        if spec["self"] == "text": return E(text7.name_of(v, {})) + idc
        return P["link"](v, text7.name_of(v, {})) + idc
    else: v = base7.get(src, spec["field"])
    if v is None or v == "": return "―"
    if isinstance(v, str) and v.startswith("TERM-"): v = text7.word(v)
    as_ = spec.get("as")
    out = P["pill"](v, P["tones"].get(spec.get("tone"), {}).get(v, "")) if as_ == "pill" else f"<b>{E(v)}</b>" if as_ == "bold" else f'<span class="txt">{E(v)}</span>' if as_ == "txt" else E(v)
    df = spec.get("differs_from_derive")
    if df and g.derive(df["group"], base7.get(d, df["group"])) != v: out += f' <span class="missing">{E(df["note"])}</span>'
    return out

def part(d, p, P):
    if p.get("when") and not base7.get(d, p["when"]): return ""
    k = d["kind"]
    if p["part"] == "kv":
        rows = []
        for r in p["rows"]:
            if r.get("when") and not base7.get(d, r["when"]): continue
            lb = r["label"]; lb = schema_at(k, lb["derive_title"])["x-derive"]["title"] if isinstance(lb, dict) else lb
            rows.append((lb, val(d, r["value"], P)))
        return P["kv"](rows)
    if p["part"] == "questions":  # 問い（スキーマの title）と、この宣言の答え。判定の決まりに出てくる欄だけ
        sc = schema_at(k, p["group"]); ks = list(dict.fromkeys(q for r in sc["x-derive"]["rules"] for q in r["when"]))
        ans = base7.get(d, p["group"])
        return P["table"](p.get("heads", ["問い", "答え"]), [[E(sc["properties"][q]["title"]), P["yn"](ans[q])] for q in ks])
    if p["part"] == "table":
        xs = base7.get(d, p["items"]["field"]) or []
        if not xs: return f'<p class="txt">{E(p.get("empty", "なし"))}</p>'
        rows = [[val(d, c["value"], P, resolve(x) if p["items"].get("resolve") else x) if "field" in c["value"] else val(d, c["value"], P, x) for c in p["cols"]] for x in xs]
        sub = ""
        if p.get("sub"): sub = f'<h3 class="sub">{E(schema_at(k, p["sub"]["title_of"]).get("title") or schema_at(k, p["sub"]["title_of"]).get("description"))}</h3>'
        return sub + P["table"]([c["head"] for c in p["cols"]], rows)
    return ""

def page(d, P):
    t = TPL[d["kind"]]; hd = t.get("head", {})
    badge = " " + val(d, hd["badge"], P) if hd.get("badge") else ""
    b = P["head"](d, badge, base7.get(d, hd["lead"]["field"]) if hd.get("lead") else "")
    for s in t["sections"]:
        if s.get("when") and not base7.get(d, s["when"]): continue
        b += P["block"](s["title"], "".join(part(d, p, P) for p in s["parts"]))
    return b + (P["raw"](d) if t.get("json") else "")
