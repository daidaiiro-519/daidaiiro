# model.py に出来事を流して、状態表を生成する。マスの中身は手で書かない。
# 出力：tables.json（状態表5つと決定表）。domain-model.json はこれを読み込む。
import copy, json
from model import Core, Refused, h, call

OK = lambda rid: "verified"


def rh(c):
    return h(c.instances["i"]["fields"]) if "i" in c.instances else "-"


def samples_for(c, trial, answer):
    """抜き取りに選ばれたものへ、人が AI と同じ判定を付けたとする。"""
    out = []
    for rid, insts in trial["to_sample"].items():
        for iid in insts:
            ai = call(answer, rid, c.given(iid, rid))[0]
            out.append({"inst": iid, "rule": rid, "ai": ai, "verdict": "合っている" if ai == "verified" else "合っていない"})
    return out


def approve_v(c, rules, answer=OK, scope=None):
    vid = c.write_version(rules, None, scope)
    t = c.try_version(vid, answer)
    c.request_version(vid, t, samples_for(c, t, answer))
    c.approve_version(vid, "人", c.version(vid)["request"]["hash"])
    return vid


def approve_a(c, mapping, answer=OK, actor="人"):
    c.write_assignment(mapping)
    t = c.try_assignment(answer)
    c.request_assignment(t, samples_for(c, t, answer))
    c.approve_assignment(actor, c.assignment["request"]["hash"])


def rules1(by="AI", refs=None):
    return {"r": {"by": by, "reads": ["a"], "terms": ["t@1"], "refs": refs or []}, "o": {"by": "機械", "reads": ["b"]}}


def base(by="AI", fields=None, assign=True, refs=None, scope=None):
    c = Core()
    c.publish_term("t@1", "語の定義1")
    approve_v(c, rules1(by, refs), scope=scope)
    if assign:
        approve_a(c, {"r": "ai@1"})
    if refs:
        c.create("p1", {"a": "参照先", "b": "y"}); c.verify("p1", OK)
    c.create("i", fields or {"a": "x", "b": "y", "p": "p1"})
    c.verify("i", OK)
    return c


def at(result):
    """rule r の効いている結果が result になった状態を作る。"""
    drop = lambda c: setattr(c, "records", [x for x in c.records if not (x["inst"] == "i" and x["rule"] == "r")])
    if result == "検証待ち":
        c = base(); drop(c); return c
    if result == "verified（機械）":
        return base(by="機械")
    if result == "violated（機械）":
        c = base(by="機械"); drop(c); c.verify("i", lambda rid: "violated"); return c
    if result in ("verified（AI）", "violated（AI）", "unverified"):
        c = base(); drop(c)
        c.verify("i", lambda rid: {"verified（AI）": "verified", "violated（AI）": "violated", "unverified": "unverified"}[result]); return c
    if result == "unsupported":
        return base(assign=False)
    if result == "未記入":
        return base(fields={"a": "", "b": "y"})
    if result in ("verified（人）", "violated（人）"):
        c = at("unverified")
        c.judge("i", "r", "合っている" if result == "verified（人）" else "合っていない", "人", c.show("i", "r")); return c
    raise KeyError(result)


def swap_validator(c, actor="人"):
    approve_a(c, {"r": "ai@2"}, actor=actor)


def tamper(c):
    recs = [x for x in c.records if x["inst"] == "i" and x["rule"] == "r"]
    if recs:
        recs[-1]["tampered"] = True


def missing(c):
    raise Refused("無い instance", "J13")


E1 = [
 ("読む欄を変える", lambda c: c.change("i", {"a": "x2"}, rh(c)), ["J11", "J15", "J17"]),
 ("読まない欄を変える", lambda c: c.change("i", {"b": "y2"}, rh(c)), ["J15", "J17"]),
 ("公開した term を書き換える", lambda c: c.publish_term("t@1", "語の定義2"), ["J1", "J2"]),
 ("検証する（verified が返る）", lambda c: c.verify("i", OK), ["J17", "J18"]),
 ("検証する（根拠の無い verified が返る）", lambda c: c.verify("i", lambda rid: ("verified", None)), ["J16"]),
 ("検証する（error が返る）", lambda c: c.verify("i", lambda rid: "error"), ["J17", "J18"]),
 ("検証する（同じ入力で error が3回続く）", lambda c: [c.verify("i", lambda rid: "error") for _ in range(3)], ["J17"]),
 ("人が「合っている」と判断する", lambda c: c.judge("i", "r", "合っている", "人", c.show("i", "r")), ["J19", "J20", "J21"]),
 ("AI が判断する", lambda c: c.judge("i", "r", "合っている", "AI", c.show("i", "r")), ["J6"]),
 ("判断を取り消す", lambda c: c.revoke("i", "r", "人"), ["J21", "J23"]),
 ("validator を替える（承認）", swap_validator, ["J5", "J7", "J15", "J21"]),
 ("記録の書き換えが見つかる", tamper, ["J26", "J17"]),
 ("基盤を通らずに読む欄を書き換える", lambda c: c.external_write("i", {"a": "x3"}), ["J15", "J17"]),
]
S1 = ["検証待ち", "verified（機械）", "violated（機械）", "verified（AI）", "violated（AI）", "unverified", "unsupported",
      "未記入", "verified（人）", "violated（人）"]


def label(c):
    e = c.effective("i", "r")
    if e in ("verified", "violated"):
        e += "（機械）" if c.current_by("i", "r") == "engine@1" else "（AI）"
    return f"{e} ／ instance は{c.state('i')}"


def run(states, events, build, show):
    cells = []
    for s in states:
        for name, f, ids in events:
            c = copy.deepcopy(build(s))
            try:
                f(c)
                to, by = show(c), ids
            except Refused as e:
                to, by = "無効：" + e.reason, e.ids
            cells.append({"state": s, "event": name, "to": to, "by": by})
    return {"states": states, "events": [e[0] for e in events], "cells": cells}


# ── T2 人の判断 ──
def judged(s):
    if s == "無い":
        return at("unverified")
    c = at("verified（人）")
    if s == "入力が違って効かない":
        c.change("i", {"a": "x2"}, rh(c)); c.verify("i", lambda rid: "unverified")
    if s == "書き換えが見つかって効かない":
        c.judgments[-1]["tampered"] = True
    if s == "取り消した":
        c.revoke("i", "r", "人")
    return c


def judge_after_change(c):
    shown = c.show("i", "r")
    c.change("i", {"a": "x9"}, rh(c)); c.verify("i", lambda rid: "unverified")
    c.judge("i", "r", "合っている", "人", shown)


E2 = [
 ("判断する", lambda c: c.judge("i", "r", "合っている", "人", c.show("i", "r")), ["J20", "J21"]),
 ("示したあとに読む欄が変わってから判断する", judge_after_change, ["J21"]),
 ("読む欄が変わる", lambda c: (c.change("i", {"a": "x5" if c.instances["i"]["fields"]["a"] == "x2" else "x2"}, rh(c)), c.verify("i", lambda rid: "unverified")), ["J21"]),
 ("読む欄が判断の時点の値に戻る", lambda c: (c.change("i", {"a": "x"}, rh(c)), c.verify("i", lambda rid: "unverified")), ["J21"]),
 ("validator を替える（承認）", swap_validator, ["J21"]),
 ("判断の書き換えが見つかる", lambda c: c.judgments[-1].__setitem__("tampered", True) if c.judgments else (_ for _ in ()).throw(Refused("判断が無い", "J21")), ["J26"]),
 ("取り消す", lambda c: c.revoke("i", "r", "人"), ["J21"]),
]
S2 = ["無い", "効く", "入力が違って効かない", "書き換えが見つかって効かない", "取り消した"]


# ── T3 Concrete の版 ──
def ver(s):
    c = base()
    if s == "引退":
        approve_v(c, {"r": {"by": "AI", "reads": ["a"]}})
        c.move("i", 2, OK)
        c.retire_version(1); c.vid = 1; return c
    if s == "公開":
        c.vid = 1; return c
    c.write_version({"r": {"by": "AI", "reads": ["a"]}, "o": {"by": "機械", "reads": ["b"]}}); c.vid = 2
    if s == "承認待ち":
        t = c.try_version(2, OK); c.request_version(2, t, samples_for(c, t, OK))
    return c


def seen(c):
    v = c.version(c.vid)
    return v["request"]["hash"] if v["request"] else "-"


def request_full(c):
    t = c.try_version(c.vid, OK)
    c.request_version(c.vid, t, samples_for(c, t, OK))


def retire_after_move(c):
    if c.version(c.vid)["state"] == "公開":
        n = approve_v(c, {"r": {"by": "AI", "reads": ["a"]}})
        c.move("i", n, OK)
    c.retire_version(c.vid)


E3 = [
 ("書く", lambda c: c.write_version({"r": {"by": "AI", "reads": ["a", "b"]}}, c.vid), ["J1", "J4", "J9"]),
 ("読む欄の無い AI の rule を書く", lambda c: c.write_version({"r": {"by": "AI"}}, c.vid), ["J9"]),
 ("試す", lambda c: c.try_version(c.vid, OK), ["J5"]),
 ("承認を依頼する（試して抜き取ったあと）", request_full, ["J4", "J5"]),
 ("承認を依頼する（抜き取りが足りない）", lambda c: c.request_version(c.vid, c.try_version(c.vid, OK), []), ["J5"]),
 ("承認を依頼する（試さずに）", lambda c: c.request_version(c.vid, None), ["J5"]),
 ("人が承認する", lambda c: c.approve_version(c.vid, "人", seen(c)), ["J3", "J4"]),
 ("AI が承認する", lambda c: c.approve_version(c.vid, "AI", seen(c)), ["J6"]),
 ("差し戻す", lambda c: c.send_back_version(c.vid, "人"), ["J4", "J6"]),
 ("引退させる（従う instance がある）", lambda c: c.retire_version(c.vid), ["J8"]),
 ("引退させる（従う instance を移したあと）", retire_after_move, ["J8", "J10"]),
]
S3 = ["下書き", "承認待ち", "公開", "引退"]


# ── T4 validator の割り当て ──
def asg(s):
    c = base()
    if s in ("下書き", "承認待ち"):
        c.write_assignment({"r": "ai@2"})
    if s == "承認待ち":
        t = c.try_assignment(OK); c.request_assignment(t, samples_for(c, t, OK))
    return c


def alabel(c):
    a = c.assignment
    st = "承認待ち" if a["request"] else ("下書き" if a["draft"] else "下書き無し")
    return f"{st} ／ 公開は {(a['published'] or {}).get('r')}"


def request_a(c, full=True):
    t = c.try_assignment(OK)
    c.request_assignment(t, samples_for(c, t, OK) if full else [])


E4 = [
 ("書く", lambda c: c.write_assignment({"r": "ai@3"}), ["J4"]),
 ("承認を依頼する（試して抜き取ったあと）", request_a, ["J5"]),
 ("承認を依頼する（抜き取りが足りない）", lambda c: request_a(c, False), ["J5"]),
 ("承認を依頼する（試さずに）", lambda c: c.request_assignment(None), ["J5"]),
 ("人が承認する", lambda c: c.approve_assignment("人", c.assignment["request"]["hash"] if c.assignment["request"] else "-"), ["J3", "J4", "J7"]),
 ("AI が承認する", lambda c: c.approve_assignment("AI", "-"), ["J6"]),
 ("差し戻す", lambda c: c.send_back_assignment("人"), ["J4", "J6"]),
]
S4 = ["下書き無し", "下書き", "承認待ち"]


# ── T5 instance の有無 ──
def inst(s):
    c = base(refs=["p"], scope=["a", "p"])
    approve_v(c, rules1(refs=["p"]), scope=["a", "p"])
    if s == "無い":
        del c.instances["i"]; c.records = [x for x in c.records if x["inst"] != "i"]
    if s == "消した":
        c.delete("i")
    return c


def ilabel(c):
    if "i" not in c.instances:
        return "無い（取り込みの候補）" if "i" in c.candidates else "無い"
    return f"{c.instances['i']['state']} ／ 版{c.instances['i']['version']} ・ {c.state('i')}"


def referenced_delete(c):
    c.create("k", {"a": "x", "b": "y", "p": "i"})
    c.delete("i")


E5 = [
 ("作る", lambda c: c.create("i", {"a": "x", "b": "y"}), ["J3", "J27", "J13"]),
 ("取り込む（$schema が決まる）", lambda c: c.take("i", {"a": "x", "b": "y"}, schema=1), ["J10", "J27"]),
 ("取り込む（$schema が決まらない）", lambda c: c.take("i", {"a": "x", "b": "y"}), ["J6"]),
 ("AI が取り込みの版を確定する", lambda c: (c.take("i", {"a": "x"}), c.take("i", {"a": "x"}, actor="AI", vid=2)), ["J6"]),
 ("変える（範囲の中）", lambda c: c.change("i", {"a": "x2"}, rh(c)), ["J11", "J13"]),
 ("変える（範囲の外）", lambda c: c.change("i", {"b": "y2"}, rh(c)), ["J11"]),
 ("移す（新しい版で検証済み）", lambda c: c.move("i", 2, OK), ["J10"]),
 ("移す（新しい版で violated）", lambda c: c.move("i", 2, lambda rid: "violated"), ["J10"]),
 ("消す", lambda c: c.delete("i"), ["J12", "J13"]),
 ("消す（ほかの instance が指している）", referenced_delete, ["J12"]),
 ("基盤を通らずに範囲の外を書き換える", lambda c: c.external_write("i", {"b": "y9"}) if "i" in c.instances else missing(c), ["J14"]),
 ("基盤を通らずに $schema を書き換える", lambda c: c.external_write("i", schema=2) if "i" in c.instances else missing(c), ["J14"]),
]
S5 = ["無い", "在る", "消した"]


# ── 決定表 D：instance の状態を、rule の効いている結果から導く ──
def newer(c, rules):
    approve_v(c, rules)
    return c


def dt():
    rows = []
    for name, build in [
        ("violated か未記入の rule がある", lambda: at("violated（AI）")),
        ("violated が無く、検証待ちの rule がある", lambda: at("検証待ち")),
        ("上の2つでなく、unverified ・ unsupported の rule がある", lambda: at("unverified")),
        ("すべて verified（機械 ・ AI ・ 人）で、従う版が最新", lambda: at("verified（人）")),
        ("すべて verified で、従う版が最新でなく、最新の版を試すと結果が変わる",
         lambda: newer(at("verified（AI）"), {"r": {"by": "AI", "reads": ["a"]}, "o": {"by": "機械", "reads": ["b"]}, "n": {"by": "人", "reads": ["b"]}})),
        ("すべて verified で、従う版が最新でなく、試しても結果が変わらない", lambda: newer(at("verified（AI）"), rules1())),
        ("基盤を通らずに範囲の外か $schema が変わった", lambda: (lambda c: (c.external_write("i", schema=9), c)[-1])(at("verified（AI）"))),
        ("従う版が引退した", lambda: (lambda c: (c.version(1).__setitem__("state", "引退"), c)[-1])(at("verified（AI）"))),
    ]:
        rows.append({"if": name, "then": build().state("i")})
    return rows


if __name__ == "__main__":
    out = {
        "T1": {"name": "rule の結果（instance × rule 1件）", "note": "状態は rule の効いている結果。／ の右は instance の状態", "table": run(S1, E1, at, label)},
        "T2": {"name": "人の判断", "note": "判断の状態は記録から導く値で、保存しない", "table": run(S2, E2, judged, lambda c: c.judgment_state("i", "r"))},
        "T3": {"name": "Concrete の版", "note": "人が承認するものの1つ目", "table": run(S3, E3, ver, lambda c: c.version(c.vid)["state"])},
        "T4": {"name": "validator の割り当て（Concrete に1つ）", "note": "人が承認するものの2つ目。承認すると前の割り当ては履歴へ移る", "table": run(S4, E4, asg, alabel)},
        "T5": {"name": "instance の有無", "note": "従う版を変えるのは移すだけ", "table": run(S5, E5, inst, ilabel)},
        "D": dt(),
    }
    json.dump(out, open("tables.json", "w"), ensure_ascii=False, indent=1)
    for k, v in out.items():
        if k != "D":
            print(k, len(v["table"]["cells"]))
    for r in out["D"]:
        print("D", r["if"], "→", r["then"])
