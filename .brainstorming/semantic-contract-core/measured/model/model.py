# Semantic Schema Core のドメインモデルを、実行できる形で書いたもの。
# 状態表と決定表はこのモデルに出来事を流して生成する（gen.py）。表を手で書かない。
# 不変条件の id（J1〜J27）は do統合先-model.json が文を持ち、ここでは断る理由と、導いた根拠として記す。
import hashlib, json, copy, inspect

ERROR_LIMIT = 3   # AI の rule で、同じ入力で error が続いたら unverified にする回数（未決 O1 の仮の値）
SAMPLES = 2       # 承認の前に、AI の rule と割り当ての組ごとに人が確かめる件数（未決 O2 の仮の値）
ENGINE = "engine@1"


class Refused(Exception):
    def __init__(self, reason, *ids):
        super().__init__(reason)
        self.reason, self.ids = reason, list(ids)


def h(x):
    return hashlib.sha256(json.dumps(x, ensure_ascii=False, sort_keys=True).encode()).hexdigest()[:12]


def call(answer, rid, given):
    """validator の答え。文字列なら、読む欄の値から根拠を引いた答えとみなす。(結果, 引用) なら引用を検査する。"""
    given = copy.deepcopy(given)  # J9：AI には値の写しだけを渡す。AI が書けるのは答えだけ
    a = answer(rid, given) if len(inspect.signature(answer).parameters) >= 2 else answer(rid)
    if isinstance(a, tuple):
        return a
    quote = next((v for v in given["fields"].values() if v), None)
    return a, quote


class Core:
    """1つの Concrete（版の並びと、割り当て1つ）と、その instance と記録を持つ。"""

    def __init__(self, team=False):
        self.team = team            # チームの使い方：効く記録は 信頼できる実行者が書いたものだけ（J35）
        self.today = 0
        self.revocations = []       # 判断の取り消しの記録（足すだけ）
        self.versions = []          # {id, rules, scope, state, request}
        self.assignment = {"draft": None, "published": None, "request": None, "history": []}  # J7
        self.instances = {}         # id -> {version, fields, saved, state, schema}
        self.deleted = set()
        self.records = []           # {inst, rule, input, result, by, tampered}
        self.judgments = []         # {inst, rule, input, verdict, revoked, tampered}
        self.terms = {}             # "名前@版" -> 定義（J1 ・ J2：書き換えない）
        self.candidates = {}        # 取り込みの候補

    # ── 語 ──
    def publish_term(self, name, definition):
        if name in self.terms and self.terms[name] != definition:
            raise Refused("公開した term は変えない。新しい版を出す", "J1", "J2")
        self.terms[name] = definition

    # ── 承認を通すもの：Concrete の版 ──
    def write_version(self, rules, vid=None, scope=None):
        if vid is not None and self.version(vid)["state"] in ("公開", "引退"):
            raise Refused("公開した版は変えない。新しい版を下書きで作る", "J1")
        for r in rules.values():
            if r["by"] in ("AI", "人") and not r.get("reads"):
                raise Refused("AI と人が判定する rule は読む欄を持つ", "J9")
        if vid is None:
            vid = len(self.versions) + 1
            self.versions.append({"id": vid, "rules": rules, "scope": scope, "state": "下書き", "request": None})
            return vid
        v = self.version(vid)
        v["rules"], v["scope"], v["state"], v["request"] = rules, scope, "下書き", None  # J4：書けば依頼は取り下げ
        return vid

    def version(self, vid):
        return next(v for v in self.versions if v["id"] == vid)

    def _content(self, v):
        return h({"rules": v["rules"], "scope": v["scope"]})

    def latest_hash(self):
        vid = self.latest()
        return self._content(self.version(vid)) if vid else None

    def try_version(self, vid, answer):
        """J5：下書きの版を instance に当てて検証し、効いている結果が変わる instance を返す。いまの結果にはしない。"""
        v = self.version(vid)
        if v["state"] == "引退":
            raise Refused("引退した版", "J8")
        changed, samples = [], []
        for iid, x in self.instances.items():
            if x["state"] != "在る" or iid in self.deleted:
                continue
            t = copy.deepcopy(self)
            t.version(vid)["state"] = "公開"
            t.instances[iid]["version"] = t.instances[iid]["schema"] = vid
            t.verify(iid, answer)
            if self._results(iid) != t._results(iid):
                changed.append(iid)
            samples += [(iid, rid) for rid, r in v["rules"].items() if r["by"] == "AI"]
        return {"hash": self._content(v), "changed": changed, "to_sample": self._pick(samples)}

    def _results(self, iid):
        return {rid: self.effective(iid, rid) for rid in self.rules(iid)}

    def _pick(self, pairs):
        by_rule = {}
        for iid, rid in pairs:
            by_rule.setdefault(rid, [])
            if len(by_rule[rid]) < SAMPLES:
                by_rule[rid].append(iid)
        return by_rule

    def _check_samples(self, trial, samples):
        """J5：抜き取りの件数がそろい、どれにも人の判断が付いているか。食い違いの数を返す。"""
        need = trial["to_sample"]
        got = {}
        for s in samples:
            got.setdefault(s["rule"], []).append(s)
        for rid, insts in need.items():
            have = [s for s in got.get(rid, []) if s["inst"] in insts and s.get("verdict") in ("合っている", "合っていない")]
            if len(have) < len(insts):
                raise Refused("抜き取りの人の判断がそろっていない", "J5")
        return sum(1 for s in samples if (s["verdict"] == "合っている") != (s["ai"] == "verified"))

    def request_version(self, vid, trial, samples=(), requester=None):
        v = self.version(vid)
        if v["state"] != "下書き":
            raise Refused("下書きの版だけを依頼できる", "J4")
        if trial is None or trial["hash"] != self._content(v):
            raise Refused("試した結果の無い依頼", "J5")
        mismatch = self._check_samples(trial, samples)
        v["state"], v["request"] = "承認待ち", {"hash": self._content(v), "trial": trial, "mismatch": mismatch,
                                               "requester": requester, "parent": self.latest_hash()}

    def _reviewed(self, in_changeset):
        if self.team and not in_changeset:
            raise Refused("変更セットの中で確かめたものでなければ効かない", "J35")

    def approve_version(self, vid, actor, seen_hash, who=None, in_changeset=False):
        self._human(actor)
        self._reviewed(in_changeset)
        v = self.version(vid)
        if v["state"] != "承認待ち":
            raise Refused("依頼されていない", "J4")
        if seen_hash != v["request"]["hash"] or self._content(v) != seen_hash:
            raise Refused("人が見た中身といまの中身が違う", "J4")
        if v["request"]["parent"] != self.latest_hash():
            raise Refused("親の版が変わったので承認し直す", "J30")
        v["state"], v["approved_by"] = "公開", who

    def send_back_version(self, vid, actor):
        self._human(actor)
        v = self.version(vid)
        if v["state"] != "承認待ち":
            raise Refused("依頼されていない", "J4")
        v["state"], v["request"] = "下書き", None

    def retire_version(self, vid):
        v = self.version(vid)
        if v["state"] != "公開":
            raise Refused("公開した版だけを引退できる", "J8")
        if any(x["version"] == vid and iid not in self.deleted for iid, x in self.instances.items()):
            raise Refused("従う instance がある", "J8")
        v["state"] = "引退"

    def latest(self):
        pub = [v for v in self.versions if v["state"] == "公開"]
        return pub[-1]["id"] if pub else None

    # ── 承認を通すもの：validator の割り当て（Concrete に1つ。J7） ──
    def write_assignment(self, mapping):
        self.assignment["draft"], self.assignment["request"] = mapping, None

    def try_assignment(self, answer):
        a = self.assignment
        if a["draft"] is None:
            raise Refused("下書きの割り当てが無い", "J4")
        changed, samples = [], []
        for iid in self.instances:
            if iid in self.deleted:
                continue
            t = copy.deepcopy(self)
            t.assignment["published"] = a["draft"]
            t.verify(iid, answer)
            if self._results(iid) != t._results(iid):
                changed.append(iid)
            samples += [(iid, rid) for rid, r in self.rules(iid).items()
                        if r["by"] == "AI" and (a["published"] or {}).get(rid) != a["draft"].get(rid)]
        return {"hash": h(a["draft"]), "changed": changed, "to_sample": self._pick(samples)}

    def request_assignment(self, trial, samples=(), requester=None):
        a = self.assignment
        if a["draft"] is None:
            raise Refused("下書きの割り当てが無い", "J4")
        if trial is None or trial["hash"] != h(a["draft"]):
            raise Refused("試した結果の無い依頼", "J5")
        mismatch = self._check_samples(trial, samples)
        a["request"] = {"hash": h(a["draft"]), "trial": trial, "mismatch": mismatch, "requester": requester}

    def send_back_assignment(self, actor):
        self._human(actor)
        if self.assignment["request"] is None:
            raise Refused("依頼されていない", "J4")
        self.assignment["request"] = None

    def approve_assignment(self, actor, seen_hash, who=None, in_changeset=False):
        self._human(actor)
        self._reviewed(in_changeset)
        a = self.assignment
        if a["request"] is None:
            raise Refused("依頼されていない", "J4")
        if seen_hash != a["request"]["hash"] or h(a["draft"]) != seen_hash:
            raise Refused("人が見た中身といまの中身が違う", "J4")
        if a["published"] is not None:
            a["history"].append(a["published"])
        a["published"], a["draft"], a["request"] = a["draft"], None, None

    def validator_of(self, rid, rule):
        if rule["by"] == "人":
            return "人"
        if rule["by"] == "機械":
            return ENGINE
        return (self.assignment["published"] or {}).get(rid)

    # ── instance ──
    def create(self, iid, fields, vid=None, author=None):
        if iid in self.instances or iid in self.deleted:
            raise Refused("同じ ID ・ 消した ID は使えない", "J27", "J13")
        vid = vid or self.latest()
        if vid is None or self.version(vid)["state"] != "公開":
            raise Refused("公開した版にだけ従える", "J3")
        self.instances[iid] = {"version": vid, "fields": dict(fields), "saved": dict(fields), "state": "在る", "schema": vid,
                               "author": author}

    def take(self, iid, fields, schema=None, actor="AI", vid=None):
        """取り込む：$schema が公開した版1つに決まれば、そのまま instance にする。決まらなければ人が版を確定する（J6）。"""
        if iid in self.instances or iid in self.deleted:
            raise Refused("同じ ID ・ 消した ID は使えない", "J27", "J13")
        if schema is not None and any(v["id"] == schema and v["state"] == "公開" for v in self.versions):
            return self.create(iid, fields, schema)
        if vid is None:
            self.candidates[iid] = fields
            return "候補"
        self._human(actor)
        self.create(iid, self.candidates.pop(iid, fields), vid)

    def change(self, iid, fields, read_hash):
        x = self._alive(iid)
        if read_hash != h(x["fields"]):
            raise Refused("読んだ時点のハッシュと違う", "J11")
        scope = self.version(x["version"])["scope"]
        if scope is not None and any(k not in scope for k in fields):
            raise Refused("変えてよい範囲の外", "J11")
        x["fields"].update(fields)
        x["saved"] = dict(x["fields"])

    def move(self, iid, vid, answer):
        """J10：新しい版で下書きか検証待ちになるなら、元の版に残す。"""
        x = self._alive(iid)
        if self.version(vid)["state"] != "公開":
            raise Refused("公開した版だけへ移せる", "J3", "J8")
        t = copy.deepcopy(self)
        t.instances[iid]["version"] = t.instances[iid]["schema"] = vid
        t.verify(iid, answer)
        st = t.state(iid)
        if st in ("下書き", "検証待ち"):
            raise Refused("移すと " + st + " になるので元の版に残す", "J10")
        x["version"] = x["schema"] = vid
        self.verify(iid, answer)

    def referrers(self, iid):
        """iid を x-ref で指している、消していない instance。"""
        out = []
        for jid, y in self.instances.items():
            if jid == iid or jid in self.deleted:
                continue
            for r in self.rules(jid).values():
                if any(y["fields"].get(f) == iid for f in r.get("refs", [])):
                    out.append(jid)
        return out

    def delete(self, iid):
        self._alive(iid)
        if self.referrers(iid):
            raise Refused("指されている", "J12")
        self.instances[iid]["state"] = "消した"
        self.deleted.add(iid)

    def external_write(self, iid, fields=None, schema=None):
        """基盤を通らない書き換え。保存した記録は変えない。"""
        x = self.instances[iid]
        if iid in self.deleted:
            x["ghost"] = True  # 消した ID を持つファイルが置かれた（J14）
            return
        if fields:
            x["fields"].update(fields)
        if schema is not None:
            x["schema"] = schema

    def _alive(self, iid):
        if iid in self.deleted:
            raise Refused("消した ID", "J13")
        if iid not in self.instances:
            raise Refused("無い instance", "J13")
        return self.instances[iid]

    def _human(self, actor):
        if actor != "人":
            raise Refused("人の操作の入口からだけ", "J6")

    # ── rule の入力と結果 ──
    def given(self, iid, rid):
        """J9：rule が読む欄 ・ term ・ 参照先の値だけ。AI にはこれだけを渡す。"""
        x = self.instances[iid]
        r = self.rules(iid)[rid]
        reads = r.get("reads") or sorted(x["fields"])
        refs = {}
        for f in r.get("refs", []):
            t = x["fields"].get(f)
            refs[f] = h(self.instances[t]["fields"]) if t in self.instances and t not in self.deleted else None
        return {"fields": {k: x["fields"].get(k) for k in reads},
                "terms": {t: self.terms.get(t) for t in r.get("terms", [])}, "refs": refs}

    def rule_input(self, iid, rid, with_validator=True):
        """J15：rule の宣言 ・ 読む欄 ・ term ・ 参照先 ・（その rule の validator）。版の番号は入れない。"""
        r = self.rules(iid)[rid]
        parts = {"rule": h(r), "given": self.given(iid, rid)}
        if with_validator:
            parts["validator"] = self.validator_of(rid, r)
        return h(parts)

    def record(self, iid, rid, result, quote=None, runner=None, by=None):
        r = self.rules(iid)[rid]
        if by is None and r["by"] == "AI" and result in ("verified", "violated"):
            vals = [str(v) for v in self.given(iid, rid)["fields"].values() if v]
            if not quote or not any(str(quote) in v for v in vals):
                result = "unverified"  # J16：根拠が読む欄の値に無い
        self.records.append({"inst": iid, "rule": rid, "input": self.rule_input(iid, rid), "result": result,
                             "by": by or self.validator_of(rid, r), "tampered": False, "writer": runner or "local",
                             "trusted": True, "day": self.today})

    def add_file(self, kind, obj):
        """基盤を通らずに、誰かが記録や判断のファイルを足した。中身に何が書いてあっても、基盤が書いたものではない。"""
        o = dict(obj, trusted=False, tampered=False)
        (self.records if kind == "record" else self.judgments).append(o)

    def pending(self, iid):
        return [rid for rid in self.rules(iid) if self.current(iid, rid) == "検証待ち"]

    def verify(self, iid, answer, runner=None):
        """J18：検証待ちの rule だけを渡す。"""
        for rid in self.pending(iid):
            r = self.rules(iid)[rid]
            x = self.instances[iid]
            if any(x["fields"].get(f) and (x["fields"][f] not in self.instances or x["fields"][f] in self.deleted)
                   for f in r.get("refs", [])):
                self.record(iid, rid, "violated", runner=runner, by=ENGINE)  # J12：参照先が無い（基盤が判定する）
            elif r["by"] == "人":
                self.record(iid, rid, "unverified", runner=runner)
            elif self.validator_of(rid, r) is None:
                self.record(iid, rid, "unsupported", runner=runner)
            else:
                result, quote = call(answer, rid, self.given(iid, rid))
                self.record(iid, rid, result, quote, runner)

    def rules(self, iid):
        return self.version(self.instances[iid]["version"])["rules"]

    def _recs(self, iid, rid):
        inp = self.rule_input(iid, rid)
        return [r for r in self.records if r["inst"] == iid and r["rule"] == rid and r["input"] == inp and not r["tampered"]
                and r.get("trusted", True) and (not self.team or r.get("writer") == "trusted")]

    def current(self, iid, rid):
        """J17 ・ J32：入力がいまと一致する記録の答えが1つにそろえば、その答え。記録の順番には頼らない。
        AI の答えが割れたら unverified、機械の答えが割れたら検証待ち。error だけなら検証待ちで、AI は続けば unverified。"""
        recs = self._recs(iid, rid)
        ai = self.rules(iid)[rid]["by"] == "AI"
        got = {r["result"] for r in recs if r["result"] != "error"}
        if len(got) == 1:
            return got.pop()
        if len(got) > 1:
            return "unverified" if ai else "検証待ち"
        errors = sum(1 for r in recs if r["result"] == "error")
        return "unverified" if ai and errors >= ERROR_LIMIT else "検証待ち"

    def current_by(self, iid, rid):
        recs = [r for r in self._recs(iid, rid) if r["result"] != "error"]
        return recs[-1]["by"] if recs else None

    def unfilled(self, iid, rid):
        x = self.instances[iid]
        return any(x["fields"].get(k) in (None, "") for k in (self.rules(iid)[rid].get("reads") or []))

    # ── 人の判断 ──
    def show(self, iid, rid):
        """人に示す。示した時点の入力（validator を除く）を返し、判断はそれに結び付く（J21）。"""
        return self.rule_input(iid, rid, with_validator=False)

    def judge(self, iid, rid, verdict, actor, shown, who=None, in_changeset=False):
        self._human(actor)
        self._alive(iid)
        self._reviewed(in_changeset)
        if shown != self.rule_input(iid, rid, with_validator=False):
            raise Refused("人に示した入力といまの入力が違う", "J21")
        if self.unfilled(iid, rid):
            raise Refused("未記入の欄を読む rule", "J20")
        cur = self.current(iid, rid)
        ai = self.rules(iid)[rid]["by"] == "AI" and self.current_by(iid, rid) not in (None, ENGINE, "人")
        if not (cur in ("unverified", "unsupported") or (ai and cur in ("verified", "violated"))):
            raise Refused("判断を付けられない結果（" + cur + "）", "J20")
        for j in self.live_judgments(iid, rid):
            if j.get("who") == who:
                self.revocations.append({"target": j["id"], "who": who, "trusted": True})  # 同じ人のし直し
        j = {"inst": iid, "rule": rid, "input": shown, "verdict": verdict, "tampered": False, "who": who,
             "via": "changeset" if in_changeset else "local", "trusted": True, "day": self.today}
        j["id"] = h(j)
        self.judgments.append(j)

    def judge_many(self, items, verdict, actor, who=None, in_changeset=False):
        """J22：instance ごとに別に保存し、断ったものの一覧を返す。"""
        refused = []
        for iid, rid, shown in items:
            try:
                self.judge(iid, rid, verdict, actor, shown, who, in_changeset)
            except Refused as e:
                refused.append((iid, rid, e.reason))
        return refused

    def _revoked(self):
        return {r["target"] for r in self.revocations if r.get("trusted", True)}

    def revoke(self, iid, rid, actor, who=None):
        """取り消しは、前の判断を書き換えずに、取り消しの記録を足す。"""
        self._human(actor)
        dead = self._revoked()
        js = [j for j in self.judgments if j["inst"] == iid and j["rule"] == rid and j.get("id") not in dead
              and (who is None or j.get("who") == who)]
        if not js:
            raise Refused("取り消す判断が無い", "J21")
        for j in js:
            self.revocations.append({"target": j.get("id"), "who": who, "trusted": True})

    def judgment_state(self, iid, rid):
        js = [j for j in self.judgments if j["inst"] == iid and j["rule"] == rid]
        if not js:
            return "無い"
        j = js[-1]
        if j.get("id") in self._revoked():
            return "取り消した"
        if j["tampered"]:
            return "書き換えが見つかって効かない"
        if j["input"] != self.rule_input(iid, rid, with_validator=False):
            return "入力が違って効かない"
        return "効く"

    def live_judgments(self, iid, rid):
        """効いている判断：取り消していない ・ 書き換えが見つかっていない ・ 示した入力がいまと同じ。
        チームでは、変更セットの中で確かめたものだけ。順番には頼らない。"""
        inp = self.rule_input(iid, rid, with_validator=False)
        dead = self._revoked()
        return [j for j in self.judgments
                if j["inst"] == iid and j["rule"] == rid and j.get("id") not in dead and not j["tampered"]
                and j.get("trusted", True) and j["input"] == inp
                and (not self.team or j.get("via") == "changeset")]

    def effective(self, iid, rid):
        """J23 ・ J31：未記入 → 効いている判断（割れたら「割れた」）→ いまの結果 の順で決める。"""
        if self.unfilled(iid, rid):
            return "未記入"
        verdicts = {j["verdict"] for j in self.live_judgments(iid, rid)}
        if len(verdicts) > 1:
            return "割れた"
        if verdicts:
            return {"合っている": "verified（人）", "合っていない": "violated（人）"}[verdicts.pop()]
        return self.current(iid, rid)

    # ── instance の状態（決定表 D。J24 ・ J25 ・ J14 ・ J8） ──
    def state(self, iid):
        if iid in self.deleted:
            return "消した（消した ID のファイルを違反として返す）" if self.instances[iid].get("ghost") else "消した"
        x = self.instances[iid]
        if self.version(x["version"])["state"] != "公開":
            return "下書き"  # 引退した版 ・ 公開していない版に従う instance（J8 ・ J30）
        scope = self.version(x["version"])["scope"]
        if x["schema"] != x["version"] or (scope is not None and any(
                x["fields"].get(k) != x["saved"].get(k) for k in set(x["fields"]) | set(x["saved"]) if k not in scope)):
            return "下書き"
        eff = [self.effective(iid, rid) for rid in self.rules(iid)]
        if any(e in ("violated", "violated（人）", "未記入") for e in eff):
            return "下書き"
        if "検証待ち" in eff:
            return "検証待ち"
        if any(e in ("unverified", "unsupported", "割れた") for e in eff):
            return "判断待ち"
        latest = self.latest()
        if latest != x["version"] and iid in self.version(latest)["request"]["trial"]["changed"]:
            return "前の版で検証済み"
        return "検証済み"

    # ── 変更セットに出すもの（J33）：変更の差分と、人が見るべきもの。誰が見るかはチームのやり方で、基盤は決めない ──
    def report(self):
        out = []
        for v in self.versions:
            if v["state"] == "承認待ち":
                out.append(f"契約の版 {v['id']}：差分と試した結果（状態が変わる instance {len(v['request']['trial']['changed'])}件）")
        if self.assignment["request"]:
            out.append("validator の割り当て：差分と試した結果")
        for iid in self.instances:
            if iid in self.deleted:
                continue
            st = self.state(iid)
            if st == "下書き":
                out.append(f"{iid}：違反がある（AI が直す）")
            for rid in self.rules(iid):
                e = self.effective(iid, rid)
                if e in ("unverified", "unsupported", "割れた"):
                    out.append(f"{iid} × {rid}：人が見る（{e}）")
                elif e == "検証待ち" and st != "下書き":
                    out.append(f"{iid} × {rid}：検証できていない")
        return out


def merge(base, ours, theirs):
    """J34：統合先（theirs）の差分を自分の変更セット（ours）へ取り込む。ファイルの併合は Git が行い、
    記録と判断は1件1ファイルなので合わせるだけ。取り込んだあとの中身は、信頼できる実行者がいつもの検証でもう一度確かめる。
    同じ instance の同じ欄を両方で変えたときだけ、衝突として人が中身を選ぶ。"""
    m = copy.deepcopy(ours)
    for name in ("records", "judgments", "revocations"):
        have = {h(x) for x in getattr(m, name)}
        for x in getattr(theirs, name):
            if h(x) not in have:
                getattr(m, name).append(copy.deepcopy(x)); have.add(h(x))
    for name, d in theirs.terms.items():
        m.publish_term(name, d)
    # 版：先に 統合先（theirs）が足した版を並べ、そのあとに自分の変更セット（ours）の版を並べる。
    # 公開した版の親が、並べた時点のいまの版と違えば、試し直して変更セットの中で確かめる（J30）
    m.versions = copy.deepcopy(base.versions)
    renum = {"ours": {}, "theirs": {}}
    for side, src in (("theirs", theirs), ("ours", ours)):
        for v in src.versions[len(base.versions):]:
            known = {m._content(w): w["id"] for w in m.versions}
            if src._content(v) in known:
                renum[side][v["id"]] = known[src._content(v)]
                continue
            nv = copy.deepcopy(v); nv["id"] = len(m.versions) + 1; renum[side][v["id"]] = nv["id"]
            if v["state"] == "公開" and v["request"]["parent"] != m.latest_hash():
                nv["state"], nv["note"] = "承認待ち", "取り込んだら親の版が変わった。試し直して変更セットの中で確かめる"
            m.versions.append(nv)
    for iid, x in m.instances.items():
        x["version"] = renum["ours"].get(x["version"], x["version"]); x["schema"] = renum["ours"].get(x["schema"], x["schema"])
    if theirs.assignment["published"] != base.assignment["published"]:
        if m.assignment["published"] not in (base.assignment["published"], theirs.assignment["published"]):
            raise Refused("割り当てを両方で別々に変えた（衝突）", "J34")
        m.assignment = copy.deepcopy(theirs.assignment)
    for iid in set(base.instances) | set(ours.instances) | set(theirs.instances):
        b, o, t = (x.instances.get(iid) for x in (base, ours, theirs))
        if o == t or t == b:
            continue
        if o == b:
            nt = copy.deepcopy(t); nt["version"] = renum["theirs"].get(nt["version"], nt["version"]); nt["schema"] = renum["theirs"].get(nt["schema"], nt["schema"])
            m.instances[iid] = nt
            continue
        raise Refused("同じ instance を両方で変えた（衝突）。人が中身を選ぶ", "J34")
    m.deleted |= (theirs.deleted - base.deleted)
    return m
