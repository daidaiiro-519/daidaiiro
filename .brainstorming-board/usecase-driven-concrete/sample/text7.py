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
            n = label(cur, pool if pool is decl else decl)
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
        if v is None or v == "": return ""
        for fl in flt:
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
