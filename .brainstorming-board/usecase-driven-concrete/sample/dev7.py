# 開発の concrete の部品（ボード schema-driven-base の論点3 B″）。例を導く関数と、テスト条件の札と表を持つ。基盤は持たない
#   例を導く  ── 拒否の例 ・ 違反する状態 ・ 後の状態 ・ 境界値 ・ 作れない値（今は gen6 の関数を使う）
#   テスト条件 ── 札（tchip）と、宣言ごとの表（tblock）
import html
import gen6 as g
from parts import P
E = html.escape
CONDS = []
U = {}  # 呼ぶ側の部品（表 ・ 見出しと中身の枠 ・ 札 ・ 宣言へのリンク）
def use(conds, **parts): CONDS[:] = conds; U.update(parts)
def tchip(cid): return P("tchip", id=E(cid))
def tblock(decl):
    cs = [c for c in CONDS if c["decl"] == decl]
    if not cs: return ""
    return P("design-only", body=U["block"]("テスト条件", U["table"](["ID", "対象", "確かめること", "求めるレベル", "ハッシュ値"],
        [[tchip(c["id"]), E(c["label"]), E(c["checks"]), U["pill"](c["required_level"]), P("no-id", text=c["hash"])] for c in cs])))
def sv(x): return E(g.show(x))
def impossible(c): return E(str(g.impossible(c)))
def bounds(c): return E(" ・ ".join(f"{x}（{k}）" for x, k in g.bounds(c)))
def violation_state(decl, inv):
    """不変条件に違反する状態（前の状態の名前＝値）"""
    ve = g.violation_example(decl, inv)
    return "、".join(f"{E(g.qname(a, decl['id']))}＝{sv(v)}" for a, v in ve["before"].items())
def violation_via(decl, inv):
    ve = g.violation_example(decl, inv)
    return " ・ ".join(E(g.word([c for c in decl["commands"] if c["id"] == x][0]["name"])) for x in ve["via"]) or P("missing", text="なし")
def reject_example(decl, cmd, p):
    ex = g.reject_example(decl, cmd, p)
    exs = "、".join(f"{E(g.qname(a, decl['id'], cmd))}＝{sv(v)}" for a, v in list(ex["before"].items()) + list(ex.get("args", {}).items())) if ex else P("missing", text="組めない")
    return exs + (" " + U["pill"]("手で書いた例") if ex and ex.get("manual") else " " + U["pill"]("道具が組む"))
def accept_rows(decl, cmd):
    """受け付ける例を、前の状態 ・ 引数 ・ 後の状態 ・ 業務イベントの行にする"""
    rows = []
    k = decl["id"]
    for o in cmd.get("accept_examples", []):
        bf = "、".join(f"{E(g.qname(a, k, cmd))}＝{sv(v)}" for a, v in o["before"].items())
        ag = "、".join(f"{E(g.word(g.find(cmd['args'], a)['name']))}＝{sv(v)}" for a, v in o.get("args", {}).items()) or "―"
        at = "、".join(f"{E(g.qname(a, k, cmd))}＝{sv(v)}" for a, v in g.after(cmd, o).items())
        ev = " ".join(U["pill"](g.word(e["name"]), "t-accent") for e in cmd["emits"]) or "―"
        rows.append([tchip(f"{k}.{cmd['id']}.{o['id']}"), bf, ag, at, ev])
    return rows
