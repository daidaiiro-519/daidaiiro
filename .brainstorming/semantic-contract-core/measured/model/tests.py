# 出来事の列の試験（第66回、最小の形）。python3 tests.py で全部を実行する。
# チームで使う場面も、同じ関数で書ける：変更 ・ 取り込みのあとの姿（契約 ・ instance ・ 判定の記録）を渡すだけ。
from model import Contract, Refused, verify, confirm, state, report, seen, h

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


def engine(rid, g):
    """機械の rule：必須の欄（JSON Schema の required にあたる）と、タイトルの長さ。"""
    f = g["fields"]
    if any(v in (None, "") for v in f.values()):
        return ("violated", "必須の欄が空")
    t = f.get("タイトル", "")
    return ("verified", f"{len(t)}字") if len(t) <= 20 else ("violated", f"{len(t)}字")


def ai(rid, g):
    """要約と本文を読む AI。本文に「曖昧」があれば判定できない。要約に「矛盾」があれば違反。根拠は読んだ値から引く。"""
    f = g["fields"]
    if "曖昧" in f["本文"]:
        return [("unverified", "判定できない")]
    if "矛盾" in f["要約"]:
        return [("violated", f["要約"])]
    return [("verified", f["本文"])]


V = {"engine": engine, "ai@1": ai}
RULES = {"required": {"by": "機械", "reads": ["タイトル", "要約", "本文"]},
         "title": {"by": "機械", "reads": ["タイトル"]},
         "summary": {"by": "AI", "reads": ["要約", "本文"], "terms": ["記事@1"]}}
TERMS = {"記事@1": "社外に出す文章"}


def world(**arts):
    c = Contract(RULES, TERMS, {"summary": "ai@1"})
    inst = {k: dict(v) for k, v in arts.items()} or {"a1": {"タイトル": "新製品", "要約": "出す", "本文": "10月に出す"}}
    rec = {}
    for iid in inst:
        rec.update(verify(c, inst, rec, iid, V)[1])
    return c, inst, rec


def st(c, i, k, iid="a1", v=V):
    return state(c, i, k, iid, v)


def human(c, i, k, verdict="合っている", iid="a1", rid="summary"):
    k[(iid, rid)] = confirm(c, i, verify(c, i, k, iid, V)[0], iid, rid, verdict, seen(c, i, iid, rid))


VAGUE = {"タイトル": "窓口", "要約": "変わる", "本文": "場所は曖昧"}


@case("S1", "U5", ["M3", "M9"])
def s1():
    """AI が書いた記事を検証する → 機械と AI がどちらも verified なら、人には何も出ず検証済み"""
    c, i, k = world()
    assert st(c, i, k) == "検証済み" and report((c, {}, {}), (c, i, k), V)["look"] == []


@case("S2", "U16", ["M7", "M9"])
def s2():
    """AI が判定できない → 判断待ちで人が見るものに出る → 人が判定すると検証済み"""
    c, i, k = world(a1=VAGUE)
    r = report((c, {}, {}), (c, i, k), V)
    assert st(c, i, k) == "判断待ち" and r["look"][0][:2] == ("a1", "summary")
    human(c, i, k)
    assert st(c, i, k) == "検証済み" and verify(c, i, k, "a1", V)[0]["summary"][2] == "人"


@case("S3", "U6", ["M7"])
def s3():
    """人が判定したあとに読む欄が変わる → 判定は効かず、AI に聞き直す"""
    c, i, k = world(a1=VAGUE); human(c, i, k)
    i["a1"]["本文"] = "場所はまだ曖昧"
    assert st(c, i, k) == "判断待ち"


@case("S4", "U6", ["M7"])
def s4():
    """読まない欄だけが変わる → 人の判定は効いたまま"""
    c, i, k = world(a1=VAGUE); human(c, i, k)
    i["a1"]["タイトル"] = "窓口の変更"
    assert st(c, i, k) == "検証済み"


@case("S5", "U16", ["M7"])
def s5():
    """人に見せたあとに中身が変わる → 見せた中身では判定できない"""
    c, i, k = world(a1=VAGUE); shown = seen(c, i, "a1", "summary")
    i["a1"]["本文"] = "場所はかなり曖昧"
    refused(lambda: confirm(c, i, verify(c, i, k, "a1", V)[0], "a1", "summary", "合っている", shown), "M7")


@case("S6", "U5", ["M5"])
def s6():
    """AI の根拠が読む欄に無い ・ 根拠が空 ・ 答えが割れる → どれも unverified"""
    c, i, _ = world()
    for bad in ([("verified", "読んでいない語")], [("verified", "")], [("verified", "10月に出す"), ("violated", "出す")]):
        assert verify(c, i, {}, "a1", {**V, "ai@1": lambda rid, g, b=bad: b})[0]["summary"][0] == "unverified"


@case("S7", "U5", ["M5"])
def s7():
    """AI が渡された値を書き換えても、記事は変わらない（AI に渡すのは写しだけ）"""
    c, i, _ = world(); before = h(i)
    verify(c, i, {}, "a1", {**V, "ai@1": lambda rid, g: (g["fields"].__setitem__("要約", "書き換え"), [("verified", "10月に出す")])[1]})
    assert h(i) == before


@case("S8", "U16", ["M8"])
def s8():
    """AI の violated は人が覆せる。機械の violated と、基盤の violated（参照先が無い）は覆せない"""
    c, i, k = world(a1={"タイトル": "窓口", "要約": "続ける（矛盾）", "本文": "締め切った"})
    assert st(c, i, k) == "下書き"
    human(c, i, k)
    assert st(c, i, k) == "検証済み"
    c2, i2, k2 = world(a1={"タイトル": "とても長いタイトルの社内イベント報告とまとめ", "要約": "出す", "本文": "出す"})
    refused(lambda: human(c2, i2, k2, rid="title"), "M8")
    rules = {**RULES, "link": {"by": "AI", "reads": ["要約"], "refs": ["関連"]}}
    c3 = Contract(rules, TERMS, {"summary": "ai@1", "link": "ai@1"})
    i3 = {"a1": {"タイトル": "新製品", "要約": "出す", "本文": "10月に出す", "関連": "消えた記事"}}
    refused(lambda: human(c3, i3, {}, rid="link"), "M8")


@case("S9", "U22", ["M7"])
def s9():
    """判定し直す → 判定の記録は instance × rule ごとに1つで、新しい判定に置き換わる"""
    c, i, k = world(a1=VAGUE)
    human(c, i, k, "合っていない"); assert st(c, i, k) == "下書き"
    human(c, i, k, "合っている"); assert st(c, i, k) == "検証済み"


@case("S10", "U11", ["M6"])
def s10():
    """参照先の記事が消える（別の変更を取り込んだ結果でも同じ）→ 指していた記事は基盤が violated にする"""
    rules = {**RULES, "link": {"by": "AI", "reads": ["要約"], "refs": ["関連"]}}
    c = Contract(rules, TERMS, {"summary": "ai@1", "link": "ai@1"})
    i = {"a1": {"タイトル": "新製品", "要約": "出す", "本文": "10月に出す", "関連": "a2"}, "a2": {"タイトル": "旧製品", "要約": "x", "本文": "x"}}
    k = {}
    assert state(c, i, k, "a1", {**V, "ai@1": lambda rid, g: [("verified", "出す")]}) == "検証済み"
    del i["a2"]
    assert verify(c, i, k, "a1", V)[0]["link"] == ("violated", "参照先が無い", "基盤")


@case("S11", "U12", ["M5", "M6"])
def s11():
    """AI には参照先の値が渡り、関係を意味で判定できる。参照先が下書きでも、参照元には伝えない"""
    rules = {**RULES, "link": {"by": "AI", "reads": ["要約"], "refs": ["関連"]}}
    c = Contract(rules, TERMS, {"summary": "ai@1", "link": "ai@1"})
    i = {"a1": {"タイトル": "新製品", "要約": "旧製品の後継", "本文": "10月に出す", "関連": "a2"},
         "a2": {"タイトル": "旧製品", "要約": "続ける（矛盾）", "本文": "終わる"}}
    seen_refs = []
    rel = lambda rid, g: (seen_refs.append(g["refs"]), [("verified", "旧製品")] if rid == "link" else ai(rid, g))[1]
    assert state(c, i, {}, "a1", {**V, "ai@1": rel}) == "検証済み" and seen_refs and seen_refs[-1]["関連"]["タイトル"] == "旧製品"
    assert state(c, i, {}, "a2", {**V, "ai@1": rel}) == "下書き"


@case("S12", "U2", ["M10"])
def s12():
    """契約の rule を消す変更 → 状態が変わる記事が出る（差分と理由は変更の仕組みが持つ）"""
    c, i, k = world(a1={"タイトル": "採用", "要約": "続ける（矛盾）", "本文": "締め切った"})
    c2 = Contract({"title": RULES["title"]}, TERMS, c.assignment)
    r = report((c, i, k), (c2, i, k), V)
    assert r["changed"] == ["a1"] and r["states"]["a1"] == "検証済み"


@case("S13", "U14", ["M10", "M7"])
def s13():
    """validator を替える変更 → AI の判定は聞き直して記録が変更に入り、結果が変わる記事が出る。人の判定は効いたまま"""
    c, i, k = world(a1=VAGUE, a2={"タイトル": "価格", "要約": "上げる", "本文": "上げる"})
    human(c, i, k)
    strict = lambda rid, g: [("violated", g["fields"]["要約"])] if g["fields"]["要約"] == g["fields"]["本文"] else ai(rid, g)
    c2 = Contract(c.rules, c.terms, {"summary": "ai@2"})
    r = report((c, i, k), (c2, i, k), {**V, "ai@2": strict})
    assert r["changed"] == ["a2"] and r["states"]["a1"] == "検証済み" and ("a2", "summary") in r["records"]


@case("S14", "U1", ["M2"])
def s14():
    """判定する者 ・ 読む欄の無い rule と、契約に無い term を使う rule は断られる"""
    refused(lambda: Contract({"x": {"by": "AI"}}), "M2")
    refused(lambda: Contract({"x": {"reads": ["a"]}}), "M2")
    refused(lambda: Contract({"x": {"by": "AI", "reads": ["a"], "terms": ["記事@9"]}}, TERMS), "M2")


@case("S15", "U2", ["M7"])
def s15():
    """term の定義を変える変更 → 断らない。その term を使う rule の AI と人の判定は効かなくなり、聞き直す"""
    c, i, k = world(a1=VAGUE); human(c, i, k)
    c2 = Contract(c.rules, {"記事@1": "社内だけの文章"}, c.assignment)
    assert st(c2, i, k) == "判断待ち" and report((c, i, k), (c2, i, k), V)["changed"] == ["a1"]


@case("S16", "U0", ["M4"])
def s16():
    """何も変えずに何度検証し直しても、AI の判定は記録を使うので状態は変わらない（AI が毎回違う答えを返しても）"""
    c, i, k = world()
    flaky = iter([[("violated", "出す")], [("unverified", "x")]] * 5)
    v = {**V, "ai@1": lambda rid, g: next(flaky)}
    assert [st(c, i, k, v=v) for _ in range(3)] == ["検証済み"] * 3


@case("S17", "U8", ["M3", "M9"])
def s17():
    """validator が答えられない（error）→ 回数を記録し、確かめ直しを待つ。必須の欄が空なら機械の rule で下書き"""
    c, i, _ = world()
    assert state(c, i, {}, "a1", {**V, "ai@1": lambda rid, g: [("error", "時間切れ")]}) == "確かめ直しを待つ"
    assert verify(c, i, {}, "a1", {**V, "ai@1": lambda rid, g: [("error", "時間切れ")]})[1][("a1", "summary")]["errors"] == 1
    c2, i2, k2 = world(a1={"タイトル": "窓口", "要約": "", "本文": "出す"})
    assert st(c2, i2, k2) == "下書き"


@case("S18", "U14", ["M3"])
def s18():
    """validator の割り当てが無い AI の rule → unsupported で人が見るものに出る。判定は「合っている」「合っていない」だけ"""
    c, i, _ = world(); c = Contract(c.rules, c.terms, {})
    assert verify(c, i, {}, "a1", V)[0]["summary"][0] == "unsupported" and st(c, i, {}) == "判断待ち"
    refused(lambda: human(c, i, {}, "たぶん"), "M3")


@case("S19", "U5", ["M11"])
def s19():
    """AI の変更が、変えてよい範囲の外の欄を変えた → 取り込めない理由として出る。人の変更なら出ない"""
    c, i, k = world()
    c = Contract(c.rules, c.terms, c.assignment, scope=["要約", "本文"])
    i2 = {"a1": {**i["a1"], "タイトル": "書き換えたタイトル"}}
    assert report((c, i, k), (c, i2, k), V)["blocked"][0][0] == "a1"
    assert report((c, i, k), (c, i2, k), V, author="人")["blocked"] == []
    i3 = {"a1": {**i["a1"], "要約": "10月に出す"}}
    assert report((c, i, k), (c, i3, k), V)["blocked"] == []


@case("S20", "U9", ["M12"])
def s20():
    """AI の変更が「人が判定した」記録を書いた → 取り込めない理由として出て、その記録は効かない"""
    c, i, k = world(a1=VAGUE)
    k2 = dict(k); k2[("a1", "summary")] = {"inst": "a1", "rule": "summary", "input": seen(c, i, "a1", "summary"), "result": "verified", "evidence": "偽", "by": "人"}
    r = report((c, i, k), (c, i, k2), V)
    assert r["blocked"] and r["states"]["a1"] == "判断待ち"
    assert report((c, i, k), (c, i, k2), V, author="人")["states"]["a1"] == "検証済み"


@case("S21", "U11", ["M13"])
def s21():
    """消した ID で別の記事を作り直す → 取り込めない理由として出る（古い参照が別の記事を黙って指さないように）"""
    c, i, k = world()
    i2 = {**i, "a9": {"タイトル": "別の記事", "要約": "x", "本文": "x"}}
    assert report((c, i, k), (c, i2, k), V, deleted_ids={"a9"})["blocked"] == [("a9", "消した ID を使い直した")]


@case("S22", "U8", ["M4"])
def s22():
    """AI が同じ入力で3回答えられない → 人が見るもの（unverified）に出て、確かめ直しを待ったまま止まらない"""
    c, i, _ = world(); k = {}
    v = {**V, "ai@1": lambda rid, g: [("error", "時間切れ")]}
    for _ in range(3):
        res, n = verify(c, i, k, "a1", v); k.update(n)
    assert res["summary"][0] == "unverified" and state(c, i, k, "a1", v) == "判断待ち"


if __name__ == "__main__":
    ok = 0
    for sid, (u, ids, doc, f) in T.items():
        try:
            f(); ok += 1; print("ok  ", sid, doc)
        except Exception as e:
            print("FAIL", sid, doc, "──", repr(e)[:200])
    print(f"{ok}/{len(T)} 通過")
