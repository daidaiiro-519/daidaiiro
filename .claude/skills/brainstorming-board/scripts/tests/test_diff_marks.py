# -*- coding: utf-8 -*-
"""この回で変わったところの印を、事例で検証する ── 欄ごとに当たること。"""
import pathlib, sys
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import build_board as m  # noqa: E402

def topic(pick="A ── はじめの答え", found=("甲 ── 乙",),
          weak=(("弱いところ", "負担する"),), src="出どころ", claim="もとにしたこと"):
    return m.Topic(no=1, label="試し", question="どうするか", status="open",
                   answer="A", note="前書きである", pick=("A", pick), found=list(found),
                   kept=[m.Option("案1", "中身", "代償"), m.Option("案2", "中身", "代償")],
                   weaknesses=list(weak),
                   grounds=[("支える先", claim, "measured", src)],
                   tables=[m.Table("表", ["左", "右"], {"行": ["甲", "乙"]}, plain=True)])

ok = 0
def check(name, cond):
    global ok
    assert cond, name
    ok += 1; print("  ok", name)

a = topic(); snap = m.snapshot([a])
check("記録は欄の並びで持つ", snap["1"]["grounds"] == [["支える先", "もとにしたこと", "measured", "出どころ"]])
check("前の回が無ければ、印は付かない", m.Diff(a, None).n == 0)
check("同じものどうしなら、0か所", m.Diff(a, snap).n == 0)

b = topic(pick="A′ ── 直した答え", found=("甲 ── 乙", "丙 ── 丁"))
d = m.Diff(b, snap)
check("変わった欄だけ数える（答え1・足した行1）", d.n == 2)
check("前の中身が印に入る", "はじめの答え" in d.one("pick", "A′ ── 直した答え"))
check("変えていない欄は、素のまま", d.mark("found", 0, 0, "甲 ── 乙") == "甲 ── 乙")
check("足した行は「足した」と出る", "この回で足した" in d.mark("found", 1, 0, "丙 ── 丁"))

c = topic(src="別の出どころ")
d2 = m.Diff(c, snap)
check("出どころだけ変えたら、1か所", d2.n == 1)
check("中身の欄には印が付かない", d2.mark("grounds", 0, 1, "もとにしたこと") == "もとにしたこと")
check("出どころの欄に印が付く", "chg" in d2.mark("grounds", 0, 3, "別の出どころ"))

e = topic(weak=(("弱いところ", "次にすること"),))
check("弱いところは、行き先の欄だけに付く", m.Diff(e, snap).n == 1)

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

# ── どの画面からでも開ける引き出し ──────────────────────────
html2 = m.deck("試し", [b], prev=snap2, board="t", round_no=2)
assert 'id="dtoggle"' in html2, "つまみが出ない"
print("  ok どの画面からでも開くつまみが出る")
assert 'id="drawer"' in html2 and 'class="dgo"' in html2, "引き出しの中身が無い"
print("  ok 引き出しに、跳ぶ行が並ぶ")
assert html2.count('class="dgo"') <= m.Diff(b, snap2).n, "一覧が欄ごとに出ている"
print("  ok 一覧は行ごと（欄ごとではない）")
none2 = m.deck("試し", [b], board="t", round_no=2)
assert 'id="dtoggle"' not in none2, "前の回が無いのに、つまみが出た"
print("  ok 変更が無ければ、つまみごと出ない")
assert m.deck("試し", [b], prev=snap2, board="t", round_no=2) == html2
print("  ok 2回組んでも同じ（冪等）")

# ── 基準と組み立てが、同じものを見る ────────────────────────
# **片方だけが並べ替えていると、動かしていない欄が毎回「変わった」と出る**
# （実測で、根拠の欄が 140 か所出た）
import json as _json
import tempfile as _tmp

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import render_board as _rb  # noqa: E402
from lib.build_board import snapshot as _snap  # noqa: E402

_src = {"topics": [{
    "no": 1, "name": "試し", "question": "問い", "status": "settled",
    "answer": "答え", "decision": {"letter": "A", "text": [{"kind": "para", "text": "決定"}]},
    "grounds": [
        {"supports": "甲", "basis": "も", "tag": "前提", "source": "出"},
        {"supports": "乙", "basis": "と", "tag": "実測", "source": "所"},
        {"supports": "丙", "basis": "に", "tag": "原典", "source": "先"}]}]}
with _tmp.TemporaryDirectory() as _d:
    _dir = pathlib.Path(_d)
    _src["round"] = 1
    (_dir / "board.json").write_text(_json.dumps(_src, ensure_ascii=False), encoding="utf-8")
    _rb.freeze(_dir)          # 保存する側
    _base = _json.loads((_dir / "rounds" / "1.json").read_text(encoding="utf-8"))["snap"]
    _now = _rb.prepare(_src, _dir / "figures")      # 組み立てる側
    assert sum(m.Diff(t, _base).n for t in _now) == 0, "動かしていないのに、変わったと出る"
print("  ok 保存した基準と、組み立てが見るものが一致する")

print("\n24 件すべて通った")
