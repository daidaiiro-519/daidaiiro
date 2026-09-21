# -*- coding: utf-8 -*-
"""この回で変わったところの印を、事例で検証する ── 升ごとに当たること。"""
import importlib.util, sys, pathlib
spec = importlib.util.spec_from_file_location("bb2", pathlib.Path(__file__).with_name("build_board.py"))
m = importlib.util.module_from_spec(spec); sys.modules["bb2"] = m; spec.loader.exec_module(m)

def topic(pick="A ── はじめの答え", found=("甲 ── 乙",),
          weak=(("弱いところ", "負担する"),), src="出どころ", claim="もとにしたこと"):
    return m.Topic(no=1, label="試し", question="どうするか", status="新規",
                   answer="A", note="前書きである", pick=("A", pick), found=list(found),
                   kept=[m.Option("案1", "中身", "代償"), m.Option("案2", "中身", "代償")],
                   weaknesses=list(weak),
                   grounds=[("支える先", claim, "実測", src)],
                   tables=[m.Table("表", ["左", "右"], {"行": ["甲", "乙"]}, plain=True)])

ok = 0
def check(name, cond):
    global ok
    assert cond, name
    ok += 1; print("  ok", name)

a = topic(); snap = m.snapshot([a])
check("写しは升の並びで持つ", snap["1"]["grounds"] == [["支える先", "もとにしたこと", "実測", "出どころ"]])
check("前の回が無ければ、印は付かない", m.Diff(a, None).n == 0)
check("同じものどうしなら、0か所", m.Diff(a, snap).n == 0)

b = topic(pick="A′ ── 直した答え", found=("甲 ── 乙", "丙 ── 丁"))
d = m.Diff(b, snap)
check("変わった升だけ数える（答え1・足した行1）", d.n == 2)
check("前の中身が印に入る", "はじめの答え" in d.one("pick", "A′ ── 直した答え"))
check("変えていない升は、素のまま", d.mark("found", 0, 0, "甲 ── 乙") == "甲 ── 乙")
check("足した行は「足した」と出る", "この回で足した" in d.mark("found", 1, 0, "丙 ── 丁"))

c = topic(src="別の出どころ")
d2 = m.Diff(c, snap)
check("出どころだけ変えたら、1か所", d2.n == 1)
check("中身の升には印が付かない", d2.mark("grounds", 0, 1, "もとにしたこと") == "もとにしたこと")
check("出どころの升に印が付く", "chg" in d2.mark("grounds", 0, 3, "別の出どころ"))

e = topic(weak=(("弱いところ", "次にすること"),))
check("弱いところは、行き先の升だけに付く", m.Diff(e, snap).n == 1)

html = m.deck("試し", [b], prev=snap, board="t", round_no=2)
check("印が本文に付く", 'class="chg"' in html)
check("現在地に件数が出る", "この回で変わったところ（2 か所）" in html)
check("変わっていなければ 0 と出る", "（0 か所）" in m.deck("試し", [a], prev=snap, board="t", round_no=2))
check("渡さなければ、節ごと出ない", "この回で変わったところ" not in m.deck("試し", [b], board="t", round_no=2))
check("2回組んでも同じ（冪等）", m.deck("試し", [b], prev=snap, board="t", round_no=2) == html)
print(f"\n{ok} 件すべて通った")

# ── 行を1つ挿し込んでも、以降が全部「変わった」にならないこと ──
base = topic(found=("甲 ── 乙", "丙 ── 丁"))
snap2 = m.snapshot([base])
ins = topic(found=("新しい行 ── 足した", "甲 ── 乙", "丙 ── 丁"))
d3 = m.Diff(ins, snap2)
assert d3.n == 1, f"挿し込みで {d3.n} か所と出た ── 1 のはずである"
print("  ok 行を挿し込んでも、変わったのは1か所だけ")
assert d3.mark("found", 1, 0, "甲 ── 乙") == "甲 ── 乙", "以降の行に印が付いた"
print("  ok 挿し込みの後ろの行は、素のまま")
assert "この回で足した" in d3.mark("found", 0, 0, "新しい行 ── 足した")
print("  ok 挿し込んだ行だけ「足した」と出る")
print("\n18 件すべて通った")
