# Semantic Schema Core のドメインモデル（第66回、最小の形）。
# 基盤が持つのは4つ：契約 ・ instance ・ 機械の検証（計算し直す関数）・ 判定の記録（AI と人。変更に入れて残す）。
# 版の履歴 ・ 差分と理由 ・ 変更のレビュー ・ 取り込み ・ 衝突 ・ 誰が見るかは、変更の仕組み（Git などのアダプター）が持つ。
# Refused の id は不変条件 M1〜M10 を指す。文は invariants.json が持つ。
import copy, hashlib, json

RESULTS = ("verified", "violated", "unverified", "unsupported", "error")
ERROR_LIMIT = 3  # 同じ入力で AI が答えられなかった回数がこれに届いたら、人が見るものに出す（未決 O1 の仮の値）
VERDICT = {"合っている": "verified", "合っていない": "violated"}


class Refused(Exception):
    def __init__(self, reason, *ids):
        super().__init__(reason)
        self.reason, self.ids = reason, list(ids)


def h(x):
    return hashlib.sha256(json.dumps(x, ensure_ascii=False, sort_keys=True).encode()).hexdigest()[:12]


class Contract:
    """契約（Concrete スキーマ）。rule ・ term ・ validator の割り当てを持つ。変えるのはふつうの変更である。"""

    def __init__(self, rules, terms=None, assignment=None, scope=None):
        terms = dict(terms or {})
        for rid, r in rules.items():
            if r.get("by") not in ("機械", "AI", "人"):
                raise Refused(f"rule {rid} の判定する者が無い", "M2")
            if r["by"] in ("AI", "人") and not r.get("reads"):
                raise Refused(f"rule {rid} は読む欄を持たない", "M2")
            for t in r.get("terms", []):
                if t not in terms:
                    raise Refused(f"rule {rid} の term {t} が契約に無い", "M2")
        # scope：AI が書いてよい欄（変えてよい範囲。M11）。None なら全部
        self.rules, self.terms, self.assignment, self.scope = rules, terms, dict(assignment or {}), scope


def given(c, inst, iid, rid):
    """M5：rule が読む欄 ・ term ・ 参照先の値の写し。AI に渡すのはこれだけ。"""
    x, r = inst[iid], c.rules[rid]
    return copy.deepcopy({"fields": {k: x.get(k) for k in (r.get("reads") or sorted(x))},
                          "terms": {t: c.terms[t] for t in r.get("terms", [])},
                          "refs": {f: inst.get(x.get(f)) for f in r.get("refs", [])}})


def seen(c, inst, iid, rid):
    """M7：判定の記録が結び付く入力。rule の宣言と、読む欄 ・ term ・ 参照先の値。"""
    return h({"rule": c.rules[rid], "given": given(c, inst, iid, rid)})


def ai_input(c, inst, iid, rid):
    """M7：AI の判定は、使った validator にも結び付く。"""
    return h({"seen": seen(c, inst, iid, rid), "validator": c.assignment.get(rid)})


def _evidence_ok(ev, g):
    vals = [json.dumps(v, ensure_ascii=False) for v in list(g["fields"].values()) + list(g["refs"].values())]
    return bool(ev) and any(str(ev) in v for v in vals)


def verify(c, inst, records, iid, validators):
    """M3 ・ M4：rule ごとに (結果, 根拠, 判定した者) を返す。機械は毎回計算し直し、AI と人は判定の記録を使う。
    記録が無いか入力が変わっていれば AI に聞き、新しい記録を返す（変更に入れて残すのは呼ぶ側）。"""
    out, new = {}, {}
    x = inst[iid]
    for rid, r in c.rules.items():
        if any(x.get(f) and x.get(f) not in inst for f in r.get("refs", [])):
            out[rid] = ("violated", "参照先が無い", "基盤")                     # M6
            continue
        if r["by"] == "機械":
            res, ev = validators["engine"](rid, given(c, inst, iid, rid))
            out[rid] = (res, ev, "機械")
            continue
        rec = records.get((iid, rid))
        if rec and rec["by"] == "人" and rec["input"] == seen(c, inst, iid, rid):
            out[rid] = (rec["result"], rec["evidence"], "人")                  # M7 ・ M8：人の判定が優先
            continue
        if r["by"] == "人":
            out[rid] = ("unverified", "人が判定する rule", "基盤")
            continue
        if rec and rec["by"] == "AI" and rec["input"] == ai_input(c, inst, iid, rid) and rec["result"] != "error":
            out[rid] = (rec["result"], rec["evidence"], "AI")                  # M4：入力が同じなら聞き直さない
            continue
        name = c.assignment.get(rid)
        if name not in validators:
            out[rid] = ("unsupported", "validator が割り当てられていない", "基盤")
            continue
        g = given(c, inst, iid, rid)
        answers = validators[name](rid, copy.deepcopy(g))
        checked = [(res, ev) if res not in ("verified", "violated") or _evidence_ok(ev, g)
                   else ("unverified", "根拠が読む欄に無い") for res, ev in answers]   # M5
        kinds = {res for res, _ in checked if res != "error"}
        if not kinds:
            n = (rec["errors"] + 1) if rec and rec.get("result") == "error" and rec["input"] == ai_input(c, inst, iid, rid) else 1
            if n >= ERROR_LIMIT:
                out[rid] = ("unverified", f"validator が{n}回答えられなかった", "AI")   # 人が見るものに出す
            else:
                out[rid] = ("error", "validator が答えられなかった", "AI")
            new[(iid, rid)] = {"inst": iid, "rule": rid, "input": ai_input(c, inst, iid, rid), "result": "error", "evidence": "", "by": "AI", "errors": n}
            continue
        res, ev = (("unverified", "答えが割れた") if len(kinds) > 1 else next(a for a in checked if a[0] in kinds))
        out[rid] = (res, ev, "AI")
        new[(iid, rid)] = {"inst": iid, "rule": rid, "input": ai_input(c, inst, iid, rid), "result": res, "evidence": ev, "by": "AI"}
    return out, new


def confirm(c, inst, results, iid, rid, verdict, shown):
    """M7 ・ M8：人の判定。見た入力に結び付く記録を返す（instance × rule ごとに1つ。前の記録と置き換える）。"""
    if verdict not in VERDICT:
        raise Refused("判定は「合っている」か「合っていない」", "M3")
    if shown != seen(c, inst, iid, rid):
        raise Refused("見た中身といまの中身が違う", "M7")
    if c.rules[rid]["by"] == "機械" or results[rid][2] in ("機械", "基盤") and results[rid][0] == "violated":
        raise Refused("機械と基盤が判定した結果は、人が覆せない", "M8")
    return {"inst": iid, "rule": rid, "input": shown, "result": VERDICT[verdict], "evidence": f"人が「{verdict}」と判定", "by": "人"}


def state(c, inst, records, iid, validators):
    """M9：決定表。下書き（直すのは書く側）→ 確かめ直しを待つ（error）→ 判断待ち（人が見る）→ 検証済み。"""
    res, _ = verify(c, inst, records, iid, validators)
    kinds = {v[0] for v in res.values()}
    if "violated" in kinds:
        return "下書き"
    if "error" in kinds:
        return "確かめ直しを待つ"
    if kinds & {"unverified", "unsupported"}:
        return "判断待ち"
    return "検証済み"


def report(before, after, validators, author="AI", deleted_ids=()):
    """M10 〜 M13：変更（契約でも instance でも、取り込んだあとの姿でも）について、変更後の契約で全 instance を検証した状態 ・
    状態が変わった instance ・ 人が見るもの ・ 変更に入れて残す新しい判定の記録と、取り込めない理由を出す。
    差分と理由は変更の仕組みが持つ。before ・ after は (contract, instances, records)。
    author（変更を出したのが AI か人か）と deleted_ids（これまでに消した ID）は、変更の仕組み（アダプター）が渡す。"""
    (c0, i0, k0), (c1, i1, k1) = before, after
    blocked = []
    if author == "AI":
        for iid in i1:                                                     # M11：変えてよい範囲
            changed = {f for f in set(i1[iid]) | set(i0.get(iid, {})) if i1[iid].get(f) != i0.get(iid, {}).get(f)}
            if c1.scope is not None and iid in i0 and changed - set(c1.scope):
                blocked.append((iid, "変えてよい範囲の外を変えた：" + "、".join(sorted(changed - set(c1.scope)))))
        for key, r in k1.items():                                          # M12：人の判定は人の入口からだけ
            if r["by"] == "人" and k0.get(key) != r:
                blocked.append((key[0], f"AI の変更が人の判定の記録を書いた（rule {key[1]}）"))
        k1 = {key: r for key, r in k1.items() if not (r["by"] == "人" and k0.get(key) != r)}
    for iid in i1:                                                         # M13：消した ID は使い直さない
        if iid in deleted_ids and iid not in i0:
            blocked.append((iid, "消した ID を使い直した"))
    states, look, new = {}, [], {}
    for iid in i1:
        res, n = verify(c1, i1, k1, iid, validators)
        new.update(n)
        k = {**k1, **n}
        states[iid] = state(c1, i1, k, iid, validators)
        look += [(iid, rid, v[0], v[1]) for rid, v in res.items() if v[0] in ("unverified", "unsupported")]
    changed = sorted(i for i in i1 if i in i0 and state(c0, i0, k0, i, validators) != states[i])
    return {"states": states, "changed": changed, "look": look, "records": new, "blocked": blocked}
