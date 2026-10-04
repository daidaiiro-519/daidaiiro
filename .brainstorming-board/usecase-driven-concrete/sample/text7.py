# 基盤の文の組み立て器の見本（ボード schema-driven-base の論点3を試す）。欄の名前を持たず、注釈 x-view の文の型だけを読む
#   文の型 ── "{欄|絞り}" を並べた文字列。絞りは word（用語集の語）・ path（語の並びを「の」でつなぐ）・ name（同じ宣言の項目の名前）
#            map:表（注釈の表で引く）・ quote（「」で囲んで並べる）・ clause（最初の「は」を「が」にする）・ text（その欄の型の文で組む）
#   変わり方 ── by_key（値がどの欄を持つかで型を選ぶ）・ when_meaning（値が用語集のこの種類の語なら、別の型）
import re, json, os
import base7
SCH = base7.SCH
D = lambda: base7.D

GL = lambda: SCH["common"]["x-view-glossary"]
def terms():
    out = {}
    for d in D().values():
        if d["kind"] == GL()["kind"]:
            for t in base7.get(d, GL()["in"]) or []:
                out[t["id"]] = t
    return out
def word(t): x = terms().get(t); return x[GL()["label"]] if x else t
def meaning(t): x = terms().get(t); return {m[GL()["meaning_key"]] for m in x[GL()["meanings"]]} if x else set()

def defs(name): return SCH["common"]["$defs"][name]

def name_of(ref, ctx):
    """参照を名前にする。用語集の語 ・ 同じ宣言の項目 ・ 別の宣言の項目のどれでも、項目の名前の欄（x-view の name）を語にする"""
    if isinstance(ref, (int, float)): return str(ref)
    if ref.startswith("TERM-"): return path(ref)
    if ref in SCH["common"].get("x-view-words", {}): return SCH["common"]["x-view-words"][ref]
    nm = SCH["common"]["x-view-name"]
    def label(o, owner=None):
        for k in nm:
            v = base7.get(o, k)
            if isinstance(v, str): return word(v) if v.startswith("TERM-") else v
        sm = SCH["common"].get("x-view-summary")
        if sm and isinstance(o.get(sm["field"]), dict):
            return text(o[sm["field"]], sm["as"], {"decl": owner or decl})
        return None
    decl, cmd = ctx.get("decl"), ctx.get("cmd")
    p = ref.split(".")
    if p[0] in D() and len(p) == 1: return label(D()[p[0]], D()[p[0]]) or ref
    other = None
    if p[0] in D():
        other = D()[p[0]]; decl, p = other, p[1:]
    def nested_label(owner, item):
        """名前の無い項目が別の項目の中に在るときは「宣言の名前の欄の説明：条件の文」にする"""
        for parent in base7.find_all(owner):
            if parent is owner: continue
            for k, v in parent.items():
                if isinstance(v, list) and any(x is item for x in v):
                    dsc = desc_of(owner["kind"], k)
                    sm = label(item, owner)
                    return f'{label(owner, owner)}の{dsc}：{sm}' if dsc and sm else None
        return None
    if other is None and len(p) == 1 and not (cmd or decl and any(x.get("id") == p[0] for x in base7.find_all(decl))):
        for dd in D().values():
            hit = next((x for x in base7.find_all(dd) if x.get("id") == p[0] and x is not dd), None)
            if hit is not None: return label(hit, dd) or ref
    for pool in ([cmd] if cmd else []) + ([decl] if decl else []):
        cur = pool
        argp = SCH["common"].get("x-view-arg", "") if (cmd is not None and pool is cmd) else ""
        for i in p:
            cur = next((x for x in base7.find_all(cur) if x.get("id") == i), None)
            if cur is None: break
        if cur is not None:
            owner = pool if pool is decl else decl
            if other is not None and not any(base7.get(cur, k) for k in nm):
                nl = nested_label(owner, cur)
                if nl: return nl
            n = label(cur, owner)
            if n:
                # 状態の型がエンティティなら、エンティティの名前を前に付ける（ENT-1.ES-3 → 明細の提示した価格）
                if len(p) > 1:
                    head = next((x for x in base7.find_all(pool) if x.get("id") == p[0]), None)
                    hn = label(head) if head else None
                    if hn and not n.startswith(hn): n = f"{hn}の{n}"
                if argp and cur in (cmd.get("args") or []): n = argp + n
                if other is not None and not ctx.get("bare"):
                    dn = label(other)
                    if dn and not n.startswith(dn): n = f"{dn}の{n}"
                if ctx.get("prefix_decl") and decl is not None and pool is decl:
                    dn = label(decl)
                    if dn and not n.startswith(dn): n = f"{dn}の{n}"
                return n
    return ref

def path(ref):
    ws = [word(x) for x in ref.split(".")]; out = ws[0]
    for w in ws[1:]: out = w if w.startswith(out) else f"{out}の{w}"
    return out

def fill(tmpl, val, view, ctx):
    def rep(m):
        f, *flt = m.group(1).split("|")
        v = val
        for k in f.split("."):
            if k == "": continue
            v = (v[int(k)] if isinstance(v, list) and k.isdigit() and int(k) < len(v) else None) if isinstance(v, list) else (v.get(k) if isinstance(v, dict) else None)
        if v is None or v == "" or v == []: return ""
        for fl in flt:
            if isinstance(v, list) and fl in ("word", "name", "cond", "clause", "neg", "quote1"):
                v = [apply1(x, fl, ctx) for x in v]; continue
            if fl.startswith("join:"): v = fl[5:].join(v); continue
            if fl.startswith("contains:"):
                _, x, a, b = fl.split(":"); v = a if x in v else b; continue
            if fl.startswith("endswith:"):
                _, x, a, b = fl.split(":"); v = v + (a if v.endswith(x) else b); continue
            if fl in ("cond", "neg", "quote1"): v = apply1(v, fl, ctx); continue
            if fl == "form":
                gl = SCH["common"]["x-view-glossary"]; x = terms().get(v)
                frm = next((m.get(gl["form"]) for m in (x or {}).get(gl["meanings"], []) if m.get(gl["form"])), None) or view.get("form_default")
                item = {k: (gl["form_join"].join(word(y) for y in w) if isinstance(w, list) and all(isinstance(y, str) for y in w) else w) for k, w in val.items()}
                v = fill(frm, item, view, ctx); continue
            if fl == "word": v = word(v)
            elif fl == "path": v = path(v)
            elif fl == "name": v = name_of(v, ctx)
            elif fl == "bare": v = name_of(v, dict(ctx, bare=True))
            elif fl == "quote": v = "".join(f"「{x}」" for x in v)
            elif fl.startswith("map:"): v = view["maps"][fl[4:]].get(v, "")
            elif fl.startswith("text:"): v = text(v, fl[5:], ctx)
            elif fl == "clause": v = re.sub("は", "が", v, count=1)
            elif fl.startswith("suffix:"): v = f"{v}{fl[7:]}" if v else v
            elif fl == "ops":
                o = view["ops"][v]
                if isinstance(o, dict):
                    x = val.get("value"); ti = o["then_if"]
                    o = o["then"] if ((ti.get("number") and isinstance(x, (int, float))) or (isinstance(x, str) and meaning(x) & set(ti.get("meaning", [])))) else o["else"]
                v = fill(o, val, view, ctx)
        return str(v)
    out = re.sub(r"\{([^{}]+)\}", rep, tmpl)
    return out

def apply1(x, fl, ctx):
    if fl == "word": return word(x)
    if fl == "name": return name_of(x, ctx)
    if fl == "quote1": return f"「{x}」"
    if fl == "clause": return re.sub("は", "が", x, count=1)
    if fl == "neg": return x[:-3] + "だった" if x.endswith("でない") else x + "でなかった"
    if fl == "cond":  # 参照の先の項目の条件（x-view-summary）を文にする
        sm = SCH["common"]["x-view-summary"]
        for d in D().values():
            p = x.split(".")
            if p[0] == d["id"]:
                cur = d
                for i in p[1:]: cur = next((y for y in base7.find_all(cur) if y.get("id") == i), None)
                if cur is not None: return text(cur[sm["field"]], sm["as"], {})
        return x
    return x

def view_any(val, view, ctx):
    """x-view の by_value ・ cases ・ text のどれかで文にする"""
    bv = view.get("by_value")
    if bv:
        t = bv["map"].get(val.get(bv["field"]))
        if t is None: return ""
        if isinstance(t, dict):
            for c in t.get("cases", []):
                if c.get("when_empty") and not val.get(c["when_empty"]): return fill(c["text"], val, view, ctx)
            t = t["text"]
        return fill(t, val, view, ctx)
    return view_text(val, view, ctx)

def text(val, defname, ctx):
    """共通の形の値を、その形の x-view の文の型で文にする"""
    view = defs(defname)["x-view"]
    if not isinstance(val, dict):
        for k, t in view.get("by_type", {}).items():
            if (k == "number" and isinstance(val, (int, float))) or (k == "list" and isinstance(val, list)) or (k == "string" and isinstance(val, str)):
                return fill(t, {"v": val}, view, ctx)
        return str(val)
    for k, t in view.get("by_key", {}).items():
        if k in val: return fill(t, val, view, ctx)
    return fill(view["text"], val, view, ctx)

def view_text(val, view, ctx):
    """欄に直に付いた x-view（cases と text）で文にする"""
    for c in view.get("cases", []):
        if all(val.get(k) == v for k, v in c["when"].items()): return fill(c["text"], val, view, ctx)
    return fill(view["text"], val, view, ctx)

def desc_of(kind, key):
    """スキーマの中で、欄 key の説明を探す"""
    def scan(o):
        if isinstance(o, dict):
            pr = o.get("properties") or {}
            if key in pr and isinstance(pr[key], dict):
                d = pr[key].get("description") or pr[key].get("title")
                if d: return d
            for v in o.values():
                r = scan(v)
                if r: return r
        elif isinstance(o, list):
            for v in o:
                r = scan(v)
                if r: return r
        return None
    return scan(SCH.get(kind, {}))
