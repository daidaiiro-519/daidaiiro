# 基盤の描画の見本（ボード schema-driven-base の論点3 B′ の完成イメージ）。欄の名前を持たず、スキーマと注釈だけで頁を組む
#   見出し ── 欄の description ・ 並び ── スキーマの properties の順
#   値     ── x-ref は名前にして、その頁へのリンク ・ x-view の文の型を持つ形は文 ・ 項目の並びは表 ・ 入れ子の項目は見出しと中身の枠
#   集合   ── 種類ごとの目次 ・ この宣言を指すもの（逆参照）
import json, os, html, glob, sys
import base7, text7
H = os.path.dirname(os.path.abspath(__file__))
E = lambda s: html.escape(str(s))
SCH = base7.SCH

def title_of(d):
    for k in SCH["common"]["x-view-name"]:
        v = base7.get(d, k)
        if isinstance(v, str): return text7.word(v) if v.startswith("TERM-") else v
    return d["id"]

def link(ref):
    head = ref.split(".")[0]
    if head in base7.D: return f'<a class="ref" href="#{E(head)}">{E(text7.name_of(ref, {}))}</a>'
    return E(text7.name_of(ref, {}))

def value(v, sch, root, ctx):
    """1つの値を描く。注釈を見て、参照 ・ 文 ・ 表 ・ 枠のどれで描くかを決める"""
    sch, root = base7.deref(sch, root)
    if v is None or v == [] or v == "": return '<span class="txt">なし</span>'
    xv = sch.get("x-view") or {}
    if isinstance(v, dict) and ("cases" in xv or ("text" in xv and not any(xv is d.get("x-view") for d in SCH["common"]["$defs"].values()))): return E(text7.view_text(v, xv, ctx))
    xr = sch.get("x-ref")
    if xr:
        vs = v if isinstance(v, list) else [v]
        if xr["to"] == "self": return " ・ ".join(E(text7.name_of(x, ctx)) for x in vs)
        return " ・ ".join(link(x) if xr["to"] != "glossary" else E(text7.word(x)) for x in vs)
    for k, sub in [("condition", None)]:
        pass
    if isinstance(v, dict) and "x-view" in sch and "text" in sch["x-view"]:
        name = next((n for n, d in SCH["common"]["$defs"].items() if d.get("x-view") is sch.get("x-view") or d.get("x-view") == sch.get("x-view")), None)
        if name: return E(text7.text(v, name, ctx))
    if isinstance(v, list):
        it = sch.get("items", {})
        it2, r2 = base7.deref(it, root)
        if v and isinstance(v[0], dict) and it2.get("properties"):
            cols = [k for k in it2["properties"] if any(k in x for x in v) and k != "id" and not (base7.deref(it2["properties"][k], r2)[0].get("x-view") or {}).get("hidden")]
            head = "".join(f'<th>{E(lab(base7.deref(it2["properties"][k], r2)[0], k))}</th>' for k in cols)
            rows = ""
            for x in v:
                c2 = dict(ctx); c2["item"] = x; c2.setdefault("item_parent", ctx.get("item"))
                rows += "<tr>" + "".join(f'<td>{value(x.get(k), it2["properties"][k], r2, c2)}</td>' for k in cols) + "</tr>"
            return f'<div class="tw"><table class="st"><thead><tr>{head}</tr></thead><tbody>{rows}</tbody></table></div>'
        return " ・ ".join(value(x, it, root, ctx) for x in v)
    if isinstance(v, dict) and sch.get("properties"):
        return kv(v, sch, root, ctx)
    if isinstance(v, dict):  # ID を鍵にした値の組（例の引数など）は「名前＝値」で描く
        return " ・ ".join(f'{E(text7.name_of(k, dict(ctx, cmd=ctx.get("item_parent"))))}＝{E(v2 if not isinstance(v2, dict) else json.dumps(v2, ensure_ascii=False))}' for k, v2 in v.items())
    if isinstance(v, bool): return "はい" if v else "いいえ"
    if isinstance(v, str) and v.startswith("TERM-"): return E(text7.word(v))
    return E(v)

def lab(p, k): return p.get("title") or p.get("description") or k
def kv(obj, sch, root, ctx):
    rows = ""
    dv = sch.get("x-derive")
    if dv and ctx.get("prop"):
        import gen6
        rows += f'<div class="kvr"><dt>{E(dv["title"])}</dt><dd><b>{E(gen6.derive(ctx["prop"], obj))}</b></dd></div>'
    for k, ps in sch.get("properties", {}).items():
        if k in ("kind", "id", "$schema") or k not in obj: continue
        p2, r2 = base7.deref(ps, root)
        if (p2.get("x-view") or {}).get("hidden"): continue
        rows += f'<div class="kvr"><dt>{E(lab(p2, k))}</dt><dd>{value(obj[k], ps, root, ctx)}</dd></div>'
    return f'<dl class="kv">{rows}</dl>'

def page(d, L):
    sch = SCH[d["kind"]]
    ctx = {"decl": d}
    body = f'<header class="ph"><p class="kind">{E(sch["title"])}</p><h1>{E(title_of(d))}<span class="id">{E(d["id"])}</span></h1></header><p class="lead">{E(sch["description"])}</p>'
    for k, ps in sch["properties"].items():
        if k in ("kind", "id", "$schema") or k not in d: continue
        p2, r2 = base7.deref(ps, sch)
        body += f'<section class="blk"><h2>{E(p2.get("description", k))}</h2>{value(d[k], ps, sch, dict(ctx, prop=k))}</section>'
    back = sorted({l["decl"] for l in L if l["ref"].split(".")[0] == d["id"] and l["decl"] != d["id"] and l["ann"]["to"] not in ("glossary", "self")})
    if back: body += f'<section class="blk"><h2>この宣言を指すもの</h2><p>{" ・ ".join(link(x) for x in back)}</p></section>'
    return f'<article class="page" id="{E(d["id"])}">{body}</article>'

def build(D, out):
    base7.use(D)
    L = base7.links()
    order = [k for k in SCH if k in {d["kind"] for d in D.values()}]
    nav = "".join(f'<div class="ng"><span class="nk">{E(SCH[k]["title"])}</span>' + "".join(f'<a href="#{E(i)}">{E(title_of(d))}</a>' for i, d in D.items() if d["kind"] == k) + "</div>" for k in order)
    css = open(os.path.join(H, "tokens.css")).read() + open(os.path.join(H, "sample4.css")).read()
    pages = "".join(page(d, L) for d in D.values())
    first = next(iter(D))
    doc = f'''<title>基盤の描画の見本</title><style>{css}</style>
<div class="shell"><aside class="side"><p class="brand">基盤の描画（欄の名前を持たない）</p><nav class="nav">{nav}</nav></aside><main class="main">{pages}</main></div>
<script>const show=()=>{{const id=(location.hash||'#{first}').slice(1);document.querySelectorAll('.page').forEach(p=>p.hidden=p.id!==id)}};addEventListener('hashchange',show);show();</script>'''
    open(out, "w").write(doc); return len(doc)

if __name__ == "__main__":
    D = {json.load(open(f))["id"]: json.load(open(f)) for f in sorted(glob.glob(os.path.join(H, "decls", "*.json")))}
    print(build(D, os.path.join(H, "page7.html")))
