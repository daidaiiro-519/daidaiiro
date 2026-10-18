# チームで使うときの出来事の列の試験。python3 team_tests.py で全部を実行する。
# 人は名前で区別する。身元は Git のホスティングが持ち、PR のレビューを通ったことを CI が記録する（review=True）。
# チームの使い方では、効く記録を書くのは CI だけ（place="CI"）。
import copy
from model import Refused, h, merge, Core
from gen import rules1, samples_for, OK, rh
from tests import refused, T as SOLO

T = {}


def tcase(sid, u, ids):
    def wrap(f):
        T[sid] = (u, ids, f.__doc__.strip(), f)
        return f
    return wrap


POLICY = {"approvers": ["鈴木", "田中"], "judges": {"r": ["佐藤", "高橋"]}, "deputies": {"r": ["山本"]}, "days": 3}
UNV = lambda rid: "unverified" if rid == "r" else "verified"


def approve(c, vid, who="鈴木"):
    c.approve_version(vid, "人", c.version(vid)["request"]["hash"], who=who, review=True)


def team(policy=POLICY, refs=None):
    """契約の承認者は鈴木と田中、rule r（AI が判定）の担当は佐藤と高橋、代わりは山本、期限は3日。"""
    c = Core(team=True)
    c.publish_term("t@1", "語の定義1")
    vid = c.write_version(rules1(refs=refs), None, None, policy)
    c.request_version(vid, c.try_version(vid, OK), [], requester="AI")
    approve(c, vid)
    c.write_assignment({"r": "ai@1"})
    c.request_assignment(c.try_assignment(OK), [], requester="AI")
    c.approve_assignment("人", c.assignment["request"]["hash"], who="田中", review=True)
    c.create("i", {"a": "曖昧", "b": "y"}, author="AI-1")
    c.verify("i", UNV, place="CI")
    return c


def draft(c, requester="鈴木", rules=None, policy=POLICY):
    vid = c.write_version(rules or {**rules1(), "n": {"by": "機械", "reads": ["b"]}}, None, None, policy)
    t = c.try_version(vid, OK)
    c.request_version(vid, t, samples_for(c, t, OK), requester=requester)
    return vid


def J(c, who, verdict="合っている", iid="i", rid="r"):
    c.judge(iid, rid, verdict, "人", c.show(iid, rid), who=who, review=True)


@tcase("TS1", "U21", ["J28"])
def ts1():
    """契約の承認者でない人と、依頼した本人は、契約の新しい版を承認できない"""
    c = team(); vid = draft(c, requester="鈴木")
    refused(lambda: approve(c, vid, "佐藤"), "J28")
    refused(lambda: approve(c, vid, "鈴木"), "J28")
    approve(c, vid, "田中")
    assert c.version(vid)["state"] == "公開"


@tcase("TS2", "U21", ["J29"])
def ts2():
    """rule の判断の担当でない人は、その rule を判断できない"""
    c = team()
    refused(lambda: J(c, "部外者"), "J29")
    J(c, "佐藤")
    assert c.state("i") == "検証済み"


@tcase("TS3", "U22", ["J31"])
def ts3():
    """担当の判断が割れる → 判断待ちに戻り、承認者へ届く。承認者の判断で決まる"""
    c = team(); J(c, "佐藤", "合っている"); J(c, "高橋", "合っていない")
    assert c.effective("i", "r") == "割れた" and c.state("i") == "判断待ち"
    assert any("割れた" in m for m in c.inbox("鈴木")) and not any("割れた" in m for m in c.inbox("佐藤"))
    J(c, "鈴木", "合っている")
    assert c.state("i") == "検証済み"


@tcase("TS4", "U20", ["J32"])
def ts4():
    """2回の CI で AI の答えが割れる → 順番に頼らず unverified。機械の答えが割れたら承認者へ届く"""
    c = team(); c.records = [x for x in c.records if not (x["inst"] == "i" and x["rule"] == "r")]
    a = copy.deepcopy(c); a.verify("i", lambda rid: "verified", place="CI")
    b = copy.deepcopy(c); b.verify("i", lambda rid: "violated", place="CI")
    assert merge(c, a, b).current("i", "r") == "unverified" and merge(c, b, a).current("i", "r") == "unverified"
    k = team(); k.records = [x for x in k.records if not (x["inst"] == "i" and x["rule"] == "o")]
    a = copy.deepcopy(k); a.verify("i", lambda rid: "verified", place="CI")
    b = copy.deepcopy(k); b.verify("i", lambda rid: "violated", place="CI")
    m = merge(k, a, b)
    assert m.current("i", "o") == "検証待ち" and any("機械の答えが割れた" in x for x in m.inbox("鈴木"))


@tcase("TS5", "U19", ["J30"])
def ts5():
    """2つのブランチで別々の v2 が承認される → 先に併合したほうが公開。後の PR は CI が止め、承認し直しが届く"""
    base = team()
    a = copy.deepcopy(base); va = draft(a, rules={**rules1(), "n": {"by": "機械", "reads": ["b"]}}); approve(a, va, "田中")
    b = copy.deepcopy(base); vb = draft(b, rules={**rules1(), "m": {"by": "機械", "reads": ["a"]}}); approve(b, vb, "田中")
    m1 = merge(base, base, a)
    assert [v["state"] for v in m1.versions] == ["公開", "公開"]
    refused(lambda: merge(base, m1, b), "J30")


@tcase("TS6", "U18", ["J21"])
def ts6():
    """A のブランチで判断したあと、B のブランチで中身が変わって併合 → 判断は効かず、判断待ちに戻る"""
    base = team()
    a = copy.deepcopy(base); J(a, "佐藤")
    b = copy.deepcopy(base); b.change("i", {"a": "曖昧2"}, rh(b)); b.verify("i", UNV, place="CI")
    m = merge(base, a, b)
    assert m.instances["i"]["fields"]["a"] == "曖昧2" and m.state("i") == "判断待ち"


@tcase("TS7", "U18", ["J34"])
def ts7():
    """2つのブランチで同じ記事の同じ欄を別々に変える → 併合は断られ、人が中身を選ぶ"""
    base = team()
    a = copy.deepcopy(base); a.change("i", {"a": "A案"}, rh(a))
    b = copy.deepcopy(base); b.change("i", {"a": "B案"}, rh(b))
    refused(lambda: merge(base, a, b), "J34")


@tcase("TS8", "U21", ["J33"])
def ts8():
    """届け先：判断は担当へ、契約の承認は承認者（依頼した本人を除く）へ届き、関係の無い人には届かない"""
    c = team(); draft(c, requester="鈴木")
    assert any("判断" in m for m in c.inbox("佐藤")) and not any("承認" in m for m in c.inbox("佐藤"))
    assert any("承認" in m for m in c.inbox("田中")) and not any("版" in m for m in c.inbox("鈴木"))
    assert c.inbox("部外者") == []


@tcase("TS9", "U18", ["J34"])
def ts9():
    """記録と判断はブランチをまたいで合わせるだけで、どちらも消えない"""
    base = team()
    a = copy.deepcopy(base); a.create("j", {"a": "x", "b": "y"}); a.verify("j", OK, place="CI")
    b = copy.deepcopy(base); J(b, "高橋")
    m = merge(base, a, b)
    assert m.state("j") == "検証済み" and m.state("i") == "検証済み"


@tcase("TS10", "U19", ["J30"])
def ts10():
    """A の v2 に従う記事を作ったあと、B の v2 が先に併合される → A の PR は止まり、その記事は main に入らない"""
    base = team()
    a = copy.deepcopy(base); va = draft(a); approve(a, va, "田中"); a.create("k", {"a": "x", "b": "y"}, va); a.verify("k", OK, place="CI")
    b = copy.deepcopy(base); vb = draft(b, rules={**rules1(), "m": {"by": "機械", "reads": ["a"]}}); approve(b, vb, "田中")
    m = merge(base, base, b)
    refused(lambda: merge(base, m, a), "J30")
    assert "k" not in m.instances


@tcase("TS11", "U8", ["J35"])
def ts11():
    """手で verified の記録をファイルとして足す → CI が書いた記録ではないので効かない"""
    c = team(); c.records = [x for x in c.records if not (x["inst"] == "i" and x["rule"] == "r")]
    c.add_file("record", {"inst": "i", "rule": "r", "input": c.rule_input("i", "r"), "result": "verified", "by": "ai@1", "writer": "CI"})
    assert c.state("i") == "検証待ち"


@tcase("TS12", "U22", ["J31"])
def ts12():
    """同じ人が2つのブランチで逆の判断をする → どちらの向きで併合しても「割れた」"""
    base = team()
    a = copy.deepcopy(base); J(a, "佐藤", "合っている")
    b = copy.deepcopy(base); J(b, "佐藤", "合っていない")
    assert merge(base, a, b).effective("i", "r") == "割れた" == merge(base, b, a).effective("i", "r")


@tcase("TS13", "U21", ["J28", "J35"])
def ts13():
    """チームでは、依頼者の無い依頼、名前の無い取り消し、PR のレビューを通らない承認 ・ 判断は断られる"""
    c = team(); vid = c.write_version(rules1(), None, None, POLICY)
    refused(lambda: c.request_version(vid, c.try_version(vid, OK), []), "J28")
    J(c, "佐藤")
    refused(lambda: c.revoke("i", "r", "人"), "J35")
    refused(lambda: c.judge("i", "r", "合っている", "人", c.show("i", "r"), who="高橋"), "J35")


@tcase("TS14", "U21", ["J29"])
def ts14():
    """担当の決まっていない rule は承認者だけが判断できる。担当から外した人の判断は効かなくなる"""
    c = team(); J(c, "佐藤")
    p2 = {**POLICY, "judges": {"r": ["高橋"]}}
    vid = draft(c, requester="鈴木", rules=rules1(), policy=p2); approve(c, vid, "田中")
    assert c.state("i") == "判断待ち"  # 佐藤の判断は、担当から外れたので効かない
    k = team({**POLICY, "judges": {}})
    refused(lambda: J(k, "佐藤"), "J29")
    J(k, "田中")
    assert k.state("i") == "検証済み"


@tcase("TS15", "U2", ["J28"])
def ts15():
    """承認者を変えた最新の版を引退しても、承認者は前の版の名簿に戻らない"""
    c = team(); p2 = {**POLICY, "approvers": ["田中", "伊藤"]}
    vid = draft(c, requester="鈴木", policy=p2); approve(c, vid, "田中")
    c.retire_version(vid)
    assert c.policy()["approvers"] == ["田中", "伊藤"]


@tcase("TS16", "U11", ["J12"])
def ts16():
    """A で記事 p1 を消し、B で p1 を指す記事を作る → 併合の前の検査が止める"""
    base = team(refs=["p"]); base.create("p1", {"a": "x", "b": "y"}); base.verify("p1", OK, place="CI")
    a = copy.deepcopy(base); a.delete("p1")
    b = copy.deepcopy(base); b.create("k", {"a": "x", "b": "y", "p": "p1"})
    refused(lambda: merge(base, a, b), "J12")


@tcase("TS17", "U20", ["J35"])
def ts17():
    """手元の検証は状態に効かず、CI の検証だけが効く。手元の error も数えない"""
    c = team(); c.records = [x for x in c.records if not (x["inst"] == "i" and x["rule"] == "r")]
    c.verify("i", lambda rid: "verified", place="手元")
    assert c.state("i") == "検証待ち"
    for _ in range(2):
        c.verify("i", lambda rid: "error", place="手元")
    c.verify("i", lambda rid: "error", place="CI")
    assert c.current("i", "r") == "検証待ち"
    c.verify("i", lambda rid: "verified", place="CI")
    assert c.state("i") == "検証済み"


@tcase("TS18", "U21", ["J36"])
def ts18():
    """担当が3日取らない → 代わりの担当と承認者にも届き、代わりの担当が判断できる"""
    c = team()
    assert not any("判断" in m for m in c.inbox("山本"))
    c.today += 3
    assert any("判断" in m for m in c.inbox("山本")) and any("判断" in m for m in c.inbox("田中"))
    J(c, "山本")
    assert c.state("i") == "検証済み"


@tcase("TS19", "U21", ["J33"])
def ts19():
    """1人の担当が取ったら、ほかの担当には出さない"""
    c = team(); c.claim("i", "r", "佐藤")
    assert any("判断" in m for m in c.inbox("佐藤")) and not any("判断" in m for m in c.inbox("高橋"))


@tcase("TS20", "U12", ["J33"])
def ts20():
    """検証済み以外の状態は、必ず誰かの受信箱に出る（下書き ・ 判断待ち ・ 割れた ・ 割り当て無し ・ 承認し直し）"""
    c = team()
    c.create("v", {"a": "x（本文と矛盾）", "b": "y"}, author="AI-2"); c.verify("v", lambda rid: "violated" if rid == "r" else "verified", place="CI")
    c.create("w", {"a": "x", "b": "y"}, author="AI-1"); c.verify("w", UNV, place="CI"); J(c, "佐藤", iid="w"); J(c, "高橋", "合っていない", iid="w")
    c.write_version({**rules1(), "z": {"by": "AI", "reads": ["b"]}}, None, None, POLICY)
    people = ["鈴木", "田中", "佐藤", "高橋", "山本", "AI-1", "AI-2"]
    every = [m for p in people for m in c.inbox(p)]
    for iid in c.instances:
        if c.state(iid) != "検証済み":
            assert any(iid in m for m in every), (iid, c.state(iid), every)


@tcase("TS21", "U5", ["J9"])
def ts21():
    """AI の validator が渡された値を書き換えようとしても、記事は変わらない（AI が書けるのは答えだけ）"""
    c = team(); c.records = [x for x in c.records if not (x["inst"] == "i" and x["rule"] == "r")]
    before = h(c.instances["i"])
    c.verify("i", lambda rid, given: (given["fields"].__setitem__("a", "書き換え"), "verified")[1], place="CI")
    assert h(c.instances["i"]) == before


@tcase("TS22", "U21", ["J35"])
def ts22():
    """チームでは、PR のレビューを通らない契約の承認は断られる"""
    c = team(); vid = draft(c)
    refused(lambda: c.approve_version(vid, "人", c.version(vid)["request"]["hash"], who="田中"), "J35")


if __name__ == "__main__":
    ok = 0
    for sid, (u, ids, doc, f) in list(T.items()) + [(k, v) for k, v in SOLO.items()]:
        try:
            f(); ok += 1; print("ok  ", sid, doc)
        except Exception as e:
            print("FAIL", sid, doc, "──", repr(e)[:200])
    print(f"{ok}/{len(T) + len(SOLO)} 通過（チーム {len(T)} ・ 1人 {len(SOLO)}）")
