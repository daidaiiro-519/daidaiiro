# 出来事の列の試験。成功の列と失敗の列の両方を持つ。python3 tests.py で全部を実行する。
from model import Refused, h
from gen import at, base, swap_validator, approve_v, approve_a, samples_for, rules1, OK, rh

T = {}


def case(sid, u, ids):
    def wrap(f):
        T[sid] = (u, ids, f.__doc__.strip(), f)
        return f
    return wrap


def refused(f, *ids):
    try:
        f()
    except Refused as e:
        assert set(ids) <= set(e.ids), (e.ids, ids)
        return
    raise AssertionError("断られなかった")


@case("S1", "U0", ["J15", "J17", "J20", "J24"])
def s1():
    """検証済み → 参照先の instance が変わる → 読むと検証待ち。人の判断は断られる"""
    c = base(refs=["p"])
    assert c.state("i") == "検証済み"
    c.change("p1", {"a": "参照先2"}, h(c.instances["p1"]["fields"]))
    assert c.state("i") == "検証待ち"
    refused(lambda: c.judge("i", "r", "合っている", "人", c.show("i", "r")), "J20")


@case("S2", "U10", ["J3", "J5", "J15", "J21"])
def s2():
    """AI の validator を替える → 抜き取ってから承認1回 → AI の rule だけ検証し直し、人の判断は効いたまま"""
    c = at("verified（人）")
    swap_validator(c)
    assert c.effective("i", "r") == "verified（人）" and c.effective("i", "o") == "verified"
    d = at("verified（AI）"); swap_validator(d)
    assert d.pending("i") == ["r"]


@case("S3", "U9", ["J3", "J4"])
def s3():
    """AI が rule を弱めた版を依頼 → AI が書き直す → 人が見た中身では承認できない"""
    c = base()
    c.write_version({"r": {"by": "AI", "reads": ["a"]}})
    t = c.try_version(2, OK); c.request_version(2, t, samples_for(c, t, OK))
    seen = c.version(2)["request"]["hash"]
    c.write_version({"r": {"by": "AI", "reads": ["b"]}}, 2)
    refused(lambda: c.approve_version(2, "人", seen), "J4")
    refused(lambda: c.create("j", {"a": "x"}, 2), "J3")


@case("S4", "U12", ["J15", "J17"])
def s4():
    """検証済み → 基盤を通らずに読む欄を書き換える → その rule は検証待ちで、verified と示さない"""
    c = at("verified（AI）"); c.external_write("i", {"a": "x9"})
    assert c.effective("i", "r") == "検証待ち" and c.state("i") == "検証待ち"


@case("S5", "U13", ["J26", "J23"])
def s5():
    """人の判断の記録を書き換える → その判断は効かず、判断の前の結果に戻る"""
    c = at("verified（人）"); c.judgments[-1]["tampered"] = True
    assert c.effective("i", "r") == "unverified" and c.state("i") == "判断待ち"


@case("S6", "U16", ["J19", "J24"])
def s6():
    """unverified → 人が「合っていない」と判断 → violated（人）で、instance は下書き"""
    c = at("unverified"); c.judge("i", "r", "合っていない", "人", c.show("i", "r"))
    assert c.effective("i", "r") == "violated（人）" and c.state("i") == "下書き"


@case("S7", "U14", ["J17", "J18"])
def s7():
    """AI の validator が error を返す → 検証待ちのまま次の検証で再び渡し、同じ入力で3回続けば unverified"""
    c = at("検証待ち"); c.verify("i", lambda r: "error")
    assert c.pending("i") == ["r"]
    c.verify("i", lambda r: "error"); c.verify("i", lambda r: "error")
    assert c.effective("i", "r") == "unverified"


@case("S8", "U7", ["J10"])
def s8():
    """新しい版へ移すと violated → 元の版とハッシュのまま残り、理由が返る"""
    c = base(); approve_v(c, {"r": {"by": "AI", "reads": ["a"]}})
    before = h(c.instances["i"])
    refused(lambda: c.move("i", 2, lambda r: "violated"), "J10")
    assert h(c.instances["i"]) == before


@case("S9", "U11", ["J13"])
def s9():
    """消した → 消した ID で変える ・ 作る → 断られる"""
    c = base(); c.delete("i")
    refused(lambda: c.change("i", {"a": "z"}, "-"), "J13")
    refused(lambda: c.create("i", {"a": "z"}), "J13")


@case("S10", "U16", ["J20"])
def s10():
    """AI の violated を人が覆す。機械の violated は覆せない"""
    c = at("violated（AI）"); c.judge("i", "r", "合っている", "人", c.show("i", "r"))
    assert c.state("i") == "検証済み"
    m = at("violated（機械）")
    refused(lambda: m.judge("i", "r", "合っている", "人", m.show("i", "r")), "J20")


@case("S11", "U5", ["J9", "J16"])
def s11():
    """AI には読む欄だけが渡り、根拠が読む欄に無い verified は unverified になる（検証の経路で）"""
    c = at("検証待ち"); seen = []
    c.verify("i", lambda rid, given: (seen.append(given), ("verified", "読む欄に無い語"))[1])
    assert set(seen[0]["fields"]) == {"a"} and c.effective("i", "r") == "unverified"
    refused(lambda: c.write_version({"r": {"by": "AI"}}), "J9")


@case("S12", "U2", ["J25"])
def s12():
    """新しい版を公開 → 試して結果が変わる instance だけが前の版で検証済み"""
    c = at("verified（AI）")
    approve_v(c, rules1() | {"n": {"by": "人", "reads": ["b"]}})
    assert c.state("i") == "前の版で検証済み"
    d = at("verified（AI）"); approve_v(d, rules1())
    assert d.state("i") == "検証済み"


@case("S13", "U16", ["J21"])
def s13():
    """人に示したあとに AI が読む欄を変える → 示した入力では判断できない"""
    c = at("unverified"); shown = c.show("i", "r")
    c.change("i", {"a": "x2"}, rh(c)); c.verify("i", lambda r: "unverified")
    refused(lambda: c.judge("i", "r", "合っている", "人", shown), "J21")


@case("S14", "U16", ["J22"])
def s14():
    """まとめて判断 → instance ごとに保存し、断ったものの一覧が返る"""
    c = at("unverified"); c.create("j", {"a": "", "b": "y"}); c.verify("j", lambda r: "unverified")
    out = c.judge_many([("i", "r", c.show("i", "r")), ("j", "r", c.show("j", "r"))], "合っている", "人")
    assert c.state("i") == "検証済み" and [x[0] for x in out] == ["j"]


@case("S15", "U9", ["J6"])
def s15():
    """AI は承認 ・ 判断 ・ 取り消しができない"""
    c = at("unverified")
    refused(lambda: c.judge("i", "r", "合っている", "AI", c.show("i", "r")), "J6")
    c.write_version({"r": {"by": "AI", "reads": ["a"]}})
    t = c.try_version(2, OK); c.request_version(2, t, samples_for(c, t, OK))
    refused(lambda: c.approve_version(2, "AI", c.version(2)["request"]["hash"]), "J6")


@case("S16", "U14", ["J5"])
def s16():
    """抜き取りの人の判断が無いまま割り当ての承認を依頼する → 断られる（qa の U0-1）"""
    c = base(); c.write_assignment({"r": "ai@2"})
    t = c.try_assignment(OK)
    assert t["to_sample"] == {"r": ["i"]}
    refused(lambda: c.request_assignment(t, []), "J5")


@case("S17", "U14", ["J5"])
def s17():
    """抜き取りで人と AI が食い違う → 承認の依頼に食い違いの数が載る"""
    c = base(); c.write_assignment({"r": "ai@2"})
    t = c.try_assignment(OK)
    c.request_assignment(t, [{"inst": "i", "rule": "r", "ai": "verified", "verdict": "合っていない"}])
    assert c.assignment["request"]["mismatch"] == 1


@case("S18", "U11", ["J12"])
def s18():
    """ほかの instance が x-ref で指している instance を消す → 断られる（qa の U0-3）"""
    c = base(refs=["p"])
    refused(lambda: c.delete("p1"), "J12")


@case("S19", "U17", ["J6"])
def s19():
    """$schema が決まらないファイルを取り込む → 候補になり、AI は確定できず、人が確定すると検証待ちから始まる（qa の U0-4）"""
    c = base()
    assert c.take("f", {"a": "x", "b": "y"}) == "候補"
    refused(lambda: c.take("f", {}, actor="AI", vid=1), "J6")
    c.take("f", {}, actor="人", vid=1)
    assert c.state("f") == "検証待ち"
    c.take("g", {"a": "x", "b": "y"}, schema=1)
    assert c.state("g") == "検証待ち"


@case("S20", "U8", ["J17", "J20"])
def s20():
    """機械の rule が error を返し続ける → 検証待ちのままで、人は判断で埋められない"""
    c = base(by="機械"); c.records = [x for x in c.records if not (x["inst"] == "i" and x["rule"] == "r")]
    for _ in range(5):
        c.verify("i", lambda r: "error")
    assert c.effective("i", "r") == "検証待ち"
    refused(lambda: c.judge("i", "r", "合っている", "人", c.show("i", "r")), "J20")


@case("S21", "U1", ["J1", "J2"])
def s21():
    """公開した term を書き換える → 断られる。新しい版は別の名前で並べる"""
    c = base()
    refused(lambda: c.publish_term("t@1", "別の定義"), "J1")
    c.publish_term("t@2", "別の定義")


@case("S22", "U6", ["J11"])
def s22():
    """変えてよい範囲は Concrete の版が決め、範囲の外の欄の変更は断られる"""
    c = base(scope=["a"])
    refused(lambda: c.change("i", {"b": "y2"}, rh(c)), "J11")
    c.change("i", {"a": "x2"}, rh(c))


if __name__ == "__main__":
    ok = 0
    for sid, (u, ids, doc, f) in T.items():
        try:
            f(); ok += 1; print("ok  ", sid, doc)
        except Exception as e:
            print("FAIL", sid, doc, "──", repr(e))
    print(f"{ok}/{len(T)} 通過")
