# 図の部品（ボード schema-driven-base の論点3 B″）。宣言から design-svg の入力を組み、描画は design-svg に任せる
#   頁の型から {"fig": 名前} で呼ぶ。描くのは呼ぶ側が渡す figure ・ chart（design-svg を実行する）
import text7
U = {}
def use(**kw): U.update(kw)
def word(t): return text7.word(t)
def name(d): return text7.name_of(d["id"], {})

def context_map(d, env):
    """文脈の地図：この文脈と、外の相手"""
    rel = d["context_map"]["relations"]
    nodes = [{"id": d["id"], "label": d["header"]["name"], "role": "focus"}]; edges = []
    for n, rr in enumerate(rel, 1):
        nodes.append({"id": f"x{n}", "label": rr["external"]}); edges.append({"from": d["id"], "to": f"x{n}", "label": rr["pattern"]})
    return U["figure"]("context-" + d["id"], {"direction": "LR", "nodes": nodes, "edges": edges})

def ds_operation(o, env):
    """ドメインサービスの操作：入力 → 操作 → 出力"""
    d = env["^"]; ctx = {"decl": d}
    on = word(o["name"])
    nodes = [{"id": "op", "label": on, "role": "focus"}, {"id": "out", "label": text7.name_of(o["output"], {})}]; at = {"op": ["b", "r1"], "out": ["c", "r1"]}; edges = [{"from": "op", "to": "out"}]
    for i, x in enumerate(o["inputs"]):
        nodes.append({"id": f"in{i}", "label": text7.view_any(x, text7.view_of("domain_service.operations.inputs"), ctx)}); at[f"in{i}"] = ["a", f"r{i}"]; edges.append({"from": f"in{i}", "to": "op"})
    return U["figure"]("ds-" + d["id"] + "-" + o["id"], {"layout": "grid", "direction": "LR", "nodes": nodes, "edges": edges, "grid": {"cols": ["a", "b", "c"], "rows": sorted({v[1] for v in at.values()}), "at": at}})

def agg_command(c, env):
    """コマンドの状態の移り変わり：業務ルールの状態の値 → 状態の変更の状態の値 → 業務イベント"""
    d = env["^"]; cname = word(c["name"])
    pre = [p for p in c["business_rules"] if p["condition"]["op"] == "eq"]; post = [p for p in c["state_changes"] if p["condition"]["op"] == "eq"]
    nodes = []; edges = []; seen = set()
    for a in pre:
        for z in post:
            if a["condition"]["target"] == z["condition"]["target"] and str(a["condition"]["value"]).startswith("TERM-") and str(z["condition"]["value"]).startswith("TERM-"):
                s1 = word(a["condition"]["value"]); s2 = word(z["condition"]["value"])
                for x in (s1, s2):
                    if x not in seen: seen.add(x); nodes.append({"id": x, "label": x})
                edges.append({"from": s1, "to": s2, "label": cname})
                for ev in c["emits"]: nodes.append({"id": ev["id"], "label": word(ev["name"]), "role": "muted"}); edges.append({"from": s2, "to": ev["id"], "label": "業務イベント", "dashed": True})
    return U["figure"]("agg-" + d["id"] + "-" + c["id"], {"direction": "LR", "nodes": nodes, "edges": edges}) if nodes else ""

TH = {"font.size-small": 14, "font.size": 15, "chart.exchange-col-w": 220, "chart.pad": 6, "chart.exchange-row-h": 40}
def _seq(d, with_ext):
    sc = d["scenario"]; h = d["header"]; ctx = {"decl": d}; nums = text7.numbering(d)
    short = text7.view_of("use_case.step:short"); reply = text7.view_of("use_case.step:reply"); ending = text7.view_of("use_case.extension:ending")
    parts = list(dict.fromkeys([h["primary_actor"], "システム"] + sc["supporting_actors"] + [t["actor"] for t in _all(sc)]))
    msgs = []; groups = []
    def add(s):
        frm = s["actor"]; to = s.get("to") or ("システム" if s.get("calls") else frm)
        msgs.append({"from": frm, "to": to, "label": f"{nums[s['id']]} {text7.view_any(s, short, ctx)}"})
        if s.get("reply"): msgs.append({"from": to, "to": frm, "label": text7.view_any(s, reply, ctx), "kind": "return"})
    for s in sc["steps"]:
        add(s)
        if not with_ext: continue
        for x in s["extensions"]:
            st0 = len(msgs)
            for t in x["steps"]: add(t)
            et = text7.view_any(x, ending, ctx)
            groups.append({"label": "break" if x["ending"] == "失敗" else "opt", "cases": [{"name": f"{nums[x['id']]} → {et}" if et else nums[x["id"]], "span": [st0, len(msgs) - 1]}]})
    return parts, msgs, groups
def _all(sc):
    for s in sc["steps"]:
        yield s
        for x in s["extensions"]:
            for t in x["steps"]: yield t
def uc_main(d, env):
    """主成功シナリオのシーケンス図"""
    parts, msgs, _ = _seq(d, False)
    return U["chart"]("uc-" + d["id"] + "-main", "exchange", {"participants": parts, "steps": msgs, "groups": [], "theme": TH})
def uc_all(d, env):
    """拡張を含むシーケンス図"""
    parts, msgs, groups = _seq(d, True)
    return U["chart"]("uc-" + d["id"], "exchange", {"participants": parts, "steps": msgs, "groups": groups, "theme": dict(TH)})
FIGS = {"context_map": context_map, "ds_operation": ds_operation, "agg_command": agg_command, "uc_main": uc_main, "uc_all": uc_all}
