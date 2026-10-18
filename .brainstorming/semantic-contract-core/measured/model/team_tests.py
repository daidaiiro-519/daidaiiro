# チームで使うときの出来事の列の試験。python3 team_tests.py で全部を実行する。
# チームの使い方では、効く検証の記録は CI が書いたもの（place="CI"）。人が PR で確かめたもの（review=True）は、
# そのときの中身のハッシュに結び付く。誰が確かめるかはチームのやり方で、基盤は決めない。
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


UNV = lambda rid: "unverified" if rid == "r" else "verified"


def main(refs=None):
    """main：契約 v1 と割り当てが公開され、記事 i がある（rule r は AI が判定できない）。"""
    c = Core(team=True)
    c.publish_term("t@1", "語の定義1")
    vid = c.write_version(rules1(refs=refs))
    c.request_version(vid, c.try_version(vid, OK), [])
    c.approve_version(vid, "人", c.version(vid)["request"]["hash"], review=True)
    c.write_assignment({"r": "ai@1"})
    c.request_assignment(c.try_assignment(OK), [])
    c.approve_assignment("人", c.assignment["request"]["hash"], review=True)
    c.create("i", {"a": "曖昧", "b": "y"})
    c.verify("i", UNV, place="CI")
    return c


def check(c, iid="i", rid="r", verdict="合っている"):
    """PR で人が確かめる。"""
    c.judge(iid, rid, verdict, "人", c.show(iid, rid), review=True)


def new_version(c, rules):
    vid = c.write_version(rules)
    t = c.try_version(vid, OK)
    c.request_version(vid, t, samples_for(c, t, OK))
    c.approve_version(vid, "人", c.version(vid)["request"]["hash"], review=True)
    return vid


@tcase("TS1", "U5", ["J35"])
def ts1():
    """feature ブランチで AI が記事を書く → ブランチの CI が検証し、人が見るものが PR に出る"""
    br = copy.deepcopy(main())
    br.create("k", {"a": "曖昧", "b": "y"}); br.verify("k", UNV, place="CI")
    assert br.state("k") == "判断待ち" and any("k × r：人が見る" in m for m in br.report())


@tcase("TS2", "U16", ["J21", "J35"])
def ts2():
    """PR で人が確かめる → 検証済み。そのあと PR に push して中身が変わる → 確かめたことは効かない"""
    br = copy.deepcopy(main()); check(br)
    assert br.state("i") == "検証済み"
    br.change("i", {"a": "曖昧2"}, rh(br)); br.verify("i", UNV, place="CI")
    assert br.state("i") == "判断待ち"


@tcase("TS3", "U18", ["J34", "J21"])
def ts3():
    """main に別の人の変更が入る → 自分のブランチに取り込むと、変わった中身だけが検証し直される"""
    m0 = main(); m0.create("j", {"a": "x", "b": "y"}); m0.verify("j", OK, place="CI")
    mine = copy.deepcopy(m0); check(mine)
    other = copy.deepcopy(m0); other.change("j", {"a": "x2"}, h(other.instances["j"]["fields"])); other.verify("j", OK, place="CI")
    got = merge(m0, mine, other)
    assert got.state("i") == "検証済み" and got.state("j") == "検証済み" and got.pending("j") == []


@tcase("TS4", "U18", ["J21"])
def ts4():
    """自分が PR で確かめた記事を、main で別の人が変えていた → 取り込むと、確かめたことは効かず PR に出る"""
    m0 = main()
    mine = copy.deepcopy(m0); check(mine)
    other = copy.deepcopy(m0); other.change("i", {"a": "曖昧3"}, rh(other)); other.verify("i", UNV, place="CI")
    got = merge(m0, mine, other)
    assert got.state("i") == "判断待ち" and any("i × r：人が見る" in x for x in got.report())


@tcase("TS5", "U11", ["J12"])
def ts5():
    """main で記事 p1 が消された → 自分のブランチで p1 を指していた記事は、取り込み時の検証で違反になる"""
    m0 = main(refs=["p"]); m0.create("p1", {"a": "x", "b": "y"}); m0.verify("p1", OK, place="CI")
    mine = copy.deepcopy(m0); mine.create("k", {"a": "x", "b": "y", "p": "p1"}); mine.verify("k", OK, place="CI")
    other = copy.deepcopy(m0); other.delete("p1")
    got = merge(m0, mine, other); got.verify("k", OK, place="CI")
    assert got.state("k") == "下書き"


@tcase("TS6", "U19", ["J30"])
def ts6():
    """main に別の契約 v2 が先に入った → 自分の v2 は取り込み時に「試し直して PR で確かめる」になる"""
    m0 = main()
    other = copy.deepcopy(m0); new_version(other, {**rules1(), "n": {"by": "機械", "reads": ["b"]}})
    mine = copy.deepcopy(m0); new_version(mine, {**rules1(), "m": {"by": "機械", "reads": ["a"]}})
    got = merge(m0, mine, other)
    assert [v["state"] for v in got.versions] == ["公開", "公開", "承認待ち"]
    assert "n" in got.version(2)["rules"] and "m" in got.version(3)["rules"] and "取り込んだら" in got.version(3)["note"]


@tcase("TS7", "U18", ["J34"])
def ts7():
    """同じ記事の同じ欄を、main と自分のブランチで別々に変えた → Git の衝突として人が中身を選ぶ"""
    m0 = main()
    a = copy.deepcopy(m0); a.change("i", {"a": "A案"}, rh(a))
    b = copy.deepcopy(m0); b.change("i", {"a": "B案"}, rh(b))
    refused(lambda: merge(m0, a, b), "J34")


@tcase("TS8", "U8", ["J35"])
def ts8():
    """手で verified の記録をファイルとして足す ・ 手元で検証する → どちらも効かない。CI の検証だけが効く"""
    c = main(); c.records = [x for x in c.records if not (x["inst"] == "i" and x["rule"] == "r")]
    c.add_file("record", {"inst": "i", "rule": "r", "input": c.rule_input("i", "r"), "result": "verified", "by": "ai@1", "writer": "CI"})
    c.verify("i", lambda rid: "verified", place="手元")
    assert c.state("i") == "検証待ち"
    c.verify("i", lambda rid: "verified", place="CI")
    assert c.state("i") == "検証済み"


@tcase("TS9", "U20", ["J32"])
def ts9():
    """2回の CI で AI の答えが割れた → 取り込みの順番によらず unverified になり、PR に人が見るものとして出る"""
    m0 = main(); m0.records = [x for x in m0.records if not (x["inst"] == "i" and x["rule"] == "r")]
    a = copy.deepcopy(m0); a.verify("i", lambda rid: "verified", place="CI")
    b = copy.deepcopy(m0); b.verify("i", lambda rid: "violated", place="CI")
    assert merge(m0, a, b).current("i", "r") == "unverified" == merge(m0, b, a).current("i", "r")


@tcase("TS10", "U16", ["J35"])
def ts10():
    """PR で確かめていない判断は、チームでは効かない"""
    c = main()
    refused(lambda: c.judge("i", "r", "合っている", "人", c.show("i", "r")), "J35")


@tcase("TS11", "U5", ["J9"])
def ts11():
    """AI の validator が渡された値を書き換えようとしても、記事は変わらない（AI が書けるのは答えだけ）"""
    c = main(); c.records = [x for x in c.records if not (x["inst"] == "i" and x["rule"] == "r")]
    before = h(c.instances["i"])
    c.verify("i", lambda rid, given: (given["fields"].__setitem__("a", "書き換え"), "verified")[1], place="CI")
    assert h(c.instances["i"]) == before


if __name__ == "__main__":
    ok = 0
    for sid, (u, ids, doc, f) in list(T.items()) + [(k, v) for k, v in SOLO.items()]:
        try:
            f(); ok += 1; print("ok  ", sid, doc)
        except Exception as e:
            print("FAIL", sid, doc, "──", repr(e)[:200])
    print(f"{ok}/{len(T) + len(SOLO)} 通過（チーム {len(T)} ・ 1人 {len(SOLO)}）")
