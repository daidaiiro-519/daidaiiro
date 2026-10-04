# 基盤の頁の組み立ての見本（ボード schema-driven-base の論点3 B″）。concrete が宣言した頁の型（pages/*.page.json）を読み、
# 基盤の部品（design/parts.html の雛形）と文の型（x-view）で組む。欄の名前は頁の型とスキーマにだけ現れ、この道具は持たない
#
# 頁の型の節（node）
#   "文字"                         そのまま書く（文字を逃がす）
#   [節, 節]                       並べる
#   {"t": 文の型, "on": 道}          文の型（{欄|絞り}）で文にする
#   {"use": 名前, "on": 道}         頁の型の defs に置いた節を使う
#   {"v": 文の型の場所, "on": 道, "drop_head": 文の型}   スキーマの x-view で文にする（common.形 ・ 種類.欄.欄 ・ 種類.形:変わり型 ・ 種類:変わり型）
#   {"val": 道}                    値をそのまま書く
#   {"p": 部品, "s": {差し込む場所: 節}}   部品の雛形に差し込む
#   {"pill": 節, "tone": 札の色}     札。色は文字か {"map": 表の名前, "on": 道}
#   {"ref": 道, "sep": 区切り}       宣言へのリンク
#   {"each": 集め方, "as": 名前, "where": 条件, "do": 節, "sep": 区切り, "empty": 節}
#   {"if": 条件, "then": 節, "else": 節} ・ {"or": [節, 節]}（最初の空でないもの）・ {"bind": {名前: 道}, "do": 節}
#   {"count": 集め方} ・ {"derive": 欄} ・ {"help": 見出し} ・ {"lab": 見出し} ・ {"sch": 欄の道, "key": 鍵}
#   {"block": 見出し, "body": 節} ・ {"kv": [{"k": 節, "v": 節, "if": 条件}]}
#   {"table": {"cols": [節], "rows": [行の組み方], "empty": 節}} ・ {"ftable": {... "groups", "label", "unit"}}
#   {"head": {"badges": 節, "lead": 節}} ・ {"json": true} ・ {"fig": 図の名前} ・ {"dev": 名前, "args": [道]}
# 行の組み方  {"cells": [節], "key": 節, "if": 条件} ・ {"each": 集め方, "as": 名前, "where": 条件, "rows": [行の組み方]} ・ {"dev": 名前, "args": [道]}
# 名付けた値 cmd は、文の型がコマンドの引数を引く先になる
# 道          a/b（いまの値から）・ ^/a（宣言から）・ $名前/a（名付けた値から）・ %種類/a（その種類の最初の宣言から）
#             {"lit": 値}（そのままの値）・ @ref（ID を指す先にする）・ @decl（宣言の ID）・ @tail（最後の ID）・ #（数）・ *（値の組の値を全部）・ {"join": [道, 道]}（つないだ ID）。並びに欄の名前を当てると、項目ごとに引いて並べる
# 集め方      道 ・ {"decls": 種類, "where": 条件} ・ {"flat": [集め方]} ・ {"pick": 道, "from": 集め方} ・ {"questions": x-derive を持つ欄}
# 条件        道（空でない）・ {"not"} ・ {"all"} ・ {"any"} ・ {"eq": [道, 値]} ・ {"ne": [道, 値]} ・ {"same": [道, 道]}
#             {"in": [道, 道]} ・ {"some": 集め方, "as": 名前, "where": 条件} ・ {"meet": [集め方, 集め方]} ・ {"starts": [道, 文字]} ・ {"contains": [道, 文字]} ・ {"gt": [道, 数]}
import json, os, glob, html, sys
import base7, text7
H = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(H, "design")); from parts import P
E = lambda s: html.escape(str(s))
TPL = {json.load(open(f))["kind"]: json.load(open(f)) for f in glob.glob(os.path.join(H, "pages", "*.page.json"))}
SCH = base7.SCH

class S:
    """いまの値と、名付けた値"""
    def __init__(self, cur, env): self.cur, self.env = cur, env
    def at(self, cur, **kw): return S(cur, dict(self.env, **kw))
    def ctx(self): return {"decl": self.env["^"], "cmd": self.env.get("cmd")}

def resolve(v, s):
    """ID を指す先の項目にする。宣言の ID ・ 宣言.項目 ・ 同じ宣言の項目 ・ どれかの宣言の項目の順に探す"""
    if not isinstance(v, str): return v
    hit = text7.resolve_ref(v, s.ctx())
    if hit is not None: return hit
    for d in base7.D.values():
        hit = next((x for x in base7.find_all(d) if x.get("id") == v and x is not d), None)
        if hit is not None: return hit
    return None

def derived(group, s, of=None):
    d = val(of, s) if of else s.env["^"]
    return base7.derived_of(d["id"], group)

def val(p, s):
    if isinstance(p, dict) and "derive" in p: return derived(p["derive"], s, p.get("of"))
    if isinstance(p, dict) and "lit" in p: return p["lit"]
    if isinstance(p, dict) and "join" in p: return p.get("sep", ".").join(str(val(x, s)) for x in p["join"])
    if not isinstance(p, str): return p
    segs = p.split("/")
    h = segs[0]
    if h in ("", "."): v, segs = s.cur, segs[1:]
    elif h == "^": v, segs = s.env["^"], segs[1:]
    elif h.startswith("$"): v, segs = s.env.get(h[1:]), segs[1:]
    elif h.startswith("%"): v, segs = next((d for d in base7.D.values() if d["kind"] == h[1:]), None), segs[1:]
    else: v = s.cur
    for k in segs:
        if v is None: return None
        if k == "#": v = len(v); continue
        if k == "*" and isinstance(v, dict):
            v = [y for x in v.values() for y in (x if isinstance(x, list) else [x])]; continue
        if k in ("@decl", "@tail"):
            f = (lambda x: x.split(".")[0]) if k == "@decl" else (lambda x: x.split(".")[-1])
            v = [f(x) for x in v] if isinstance(v, list) else f(v); continue
        if k == "@ref":
            v = [resolve(x, s) for x in v] if isinstance(v, list) else resolve(v, s); continue
        if isinstance(v, list):
            if k.isdigit(): v = v[int(k)] if int(k) < len(v) else None
            else:
                out = []
                for x in v:
                    y = x.get(k) if isinstance(x, dict) else None
                    if y is None: continue
                    out.extend(y if isinstance(y, list) else [y])
                v = out
        else: v = v.get(k) if isinstance(v, dict) else None
    return v

def src(q, s):
    """集め方から並びを作る"""
    if isinstance(q, str):
        v = val(q, s); return [] if v is None else v if isinstance(v, list) else [v]
    if "decls" in q:
        ks = q["decls"] if isinstance(q["decls"], list) else [q["decls"]]
        return [d for d in base7.D.values() if d["kind"] in ks and (not q.get("where") or cond(q["where"], s.at(d, **{q.get("as", "it"): d})))]
    if "flat" in q: return [x for y in q["flat"] for x in src(y, s)]
    if "pick" in q: return [val(q["pick"], s.at(x)) for x in src(q["from"], s)]
    if "questions" in q:  # x-derive の問い：決まりに出てくる欄の順に、問い（スキーマの title）と、この宣言の答え
        sc = base7.deref(SCH[s.env["^"]["kind"]]["properties"][q["questions"]], SCH[s.env["^"]["kind"]])[0]
        ans = val(q["questions"], s)
        return [{"key": k, "title": sc["properties"][k]["title"], "answer": ans[k]} for k in dict.fromkeys(k for r in sc["x-derive"]["rules"] for k in r["when"])]
    raise KeyError(q)

def cond(c, s):
    if c is None: return True
    if isinstance(c, str):
        v = val(c, s); return v not in (None, "", [], {}, False)
    if "not" in c: return not cond(c["not"], s)
    if "all" in c: return all(cond(x, s) for x in c["all"])
    if "any" in c: return any(cond(x, s) for x in c["any"])
    if "eq" in c: return val(c["eq"][0], s) == c["eq"][1]
    if "ne" in c: return val(c["ne"][0], s) != c["ne"][1]
    if "same" in c: return val(c["same"][0], s) == val(c["same"][1], s)
    if "in" in c: return val(c["in"][0], s) in (val(c["in"][1], s) or [])
    if "some" in c: return any(cond(c["where"], s2) for s2 in each({"each": c["some"], "as": c.get("as")}, s))
    if "meet" in c: return bool(set(map(str, src(c["meet"][0], s))) & set(map(str, src(c["meet"][1], s))))
    if "contains" in c:
        v = val(c["contains"][0], s); return isinstance(v, (str, list)) and c["contains"][1] in v
    if "starts" in c:
        v = val(c["starts"][0], s); return isinstance(v, str) and v.startswith(c["starts"][1])
    if "gt" in c:
        v = val(c["gt"][0], s); return isinstance(v, (int, float)) and v > c["gt"][1]
    raise KeyError(c)

def each(n, s):
    """並びの項目ごとに、名付けた値を足した状態を返す"""
    out = []
    for x in src(n["each"], s):
        s2 = s.at(x, **({n["as"]: x} if n.get("as") else {}))
        if cond(n.get("where"), s2): out.append(s2)
    return out

def plain(n, s): return html.unescape(ev(n, s))

def rows_of(specs, s, P_):
    out = []
    for r in specs:
        if "each" in r:
            for s2 in each(r, s): out += rows_of(r["rows"], s2, P_)
        elif "dev" in r: out += P_["dev"][r["dev"]](*[val(a, s) for a in r.get("args", [])])
        elif cond(r.get("if"), s):
            cells = [ev(c, s) for c in r["cells"]]
            out.append((plain(r["key"], s), cells) if "key" in r else cells)
    return out

def sep(n, s, dflt):
    x = n.get("sep", dflt); return ev(x, s)

def ev(n, s):
    PT = s.env["P"]
    if n is None: return ""
    if isinstance(n, str): return E(n)
    if isinstance(n, list): return "".join(ev(x, s) for x in n)
    on = s.at(val(n["on"], s)) if "on" in n else s
    if "t" in n: return E(text7.fill(n["t"], on.cur, {}, on.ctx()))
    if "v" in n:
        sp = n["v"]
        if sp.startswith("common.") and ":" not in sp: return E(text7.text(on.cur, sp[7:], on.ctx()))
        out = text7.view_any(on.cur, text7.view_of(sp), on.ctx())
        if "drop_head" in n:  # 文の頭がこの文の型の文なら、外す
            hd = text7.fill(n["drop_head"], on.cur, {}, on.ctx())
            if out.startswith(hd): out = out[len(hd):]
        return E(out)
    if "val" in n:
        v = val(n["val"], s); return "" if v is None else E(v)
    if "p" in n: return P(n["p"], **{k: ev(x, s) for k, x in n.get("s", {}).items()})
    if "pill" in n:
        t = n.get("tone", "")
        if isinstance(t, dict): t = PT["tones"][t["map"]].get(plain(t["of"], s) if "of" in t else val(t["on"], s), "")
        return P("pill", tone=t, text=ev(n["pill"], s))
    if "ref" in n:
        v = val(n["ref"], s); vs = v if isinstance(v, list) else [] if v is None else [v]
        return sep(n, s, " ").join(PT["ref"](x) for x in vs)
    if "each" in n:
        xs = [h for h in (ev(n["do"], s2) for s2 in each(n, s)) if h]  # 空の項目は区切りを付けない
        if not xs and "empty" in n: return ev(n["empty"], s)
        return sep(n, s, "").join(xs)
    if "if" in n: return ev(n.get("then"), s) if cond(n["if"], s) else ev(n.get("else"), s)
    if "or" in n: return next((h for h in (ev(x, s) for x in n["or"]) if h), "")
    if "use" in n: return ev(s.env["defs"][n["use"]], on)
    if "bind" in n: return ev(n["do"], s.at(s.cur, **{k: val(p, s) for k, p in n["bind"].items()}))
    if "count" in n: return E(len(src(n["count"], s)))
    if "derive" in n: return E(derived(n["derive"], s, n.get("of")))
    if "help" in n: return PT["help"](n["help"])
    if "lab" in n: return E(n["lab"]) + PT["help"](n["lab"])
    if "sch" in n:
        o, root = SCH[s.env["^"]["kind"]], SCH[s.env["^"]["kind"]]
        for k in n["sch"].split("/"): o, root = base7.deref(o, root); o = o["properties"][k]
        o = base7.deref(o, root)[0]
        for k in n["key"].split("/"): o = o[k]
        return E(o)
    if "block" in n: return PT["block"](n["block"], ev(n["body"], s))
    if "kv" in n:
        rows = ""
        for r in n["kv"]:
            if not cond(r.get("if"), s): continue
            k = ev(r["k"], s)
            rows += P("kv-row", label=k + PT["help"](html.unescape(k)), value=ev(r["v"], s))
        return P("kv", rows=rows)
    if "table" in n or "ftable" in n:
        t = n.get("table") or n.get("ftable")
        if not cond(t.get("if"), s): return ""
        rows = rows_of(t["rows"], s, PT)
        if not rows and "empty" in t: return ev(t["empty"], s)
        cols = [ev(c, s) for c in t["cols"]]
        if "table" in n: return PT["table"](cols, rows)
        gs = t.get("groups") or []
        order = list(dict.fromkeys([k for _, gk in gs for k in gk] + [k for k, _ in rows]))
        rows = [(k, r) for kk in order for k, r in rows if k == kk]
        return PT["ftable"](cols, rows, gs or None, t.get("label", "種類"), t.get("unit", "件"))
    if "head" in n:
        h = n["head"]; return PT["head"](s.env["^"], ev(h.get("badges"), s), plain(h.get("lead"), s))
    if "json" in n: return PT["raw"](s.env["^"])
    if "fig" in n: return PT["fig"][n["fig"]](s.cur, s.env)
    if "dev" in n: return PT["dev"][n["dev"]](*[val(a, s) for a in n.get("args", [])])
    raise KeyError(n)

def page(d, PARTS):
    t = TPL[d["kind"]]
    return ev(t["page"], S(d, {"^": d, "P": PARTS, "defs": t.get("defs", {})}))
