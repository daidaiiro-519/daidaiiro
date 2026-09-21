# -*- coding: utf-8 -*-
import re
"""欄の整形を、事例で検証する ── **冪等であること**と、**引用を変えないこと**。\n\npython3 tests/test_build_board.py で走る。"""
import pathlib, sys
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import build_board as m  # noqa: E402

ok = 0
def check(name, cond):
    global ok
    assert cond, name
    ok += 1
    print("  ok", name)

x = "<b>主張である</b> ── これは説明である"
once = m.cell(x)
check("2段に割れる", 'class="lead-s"' in once and 'class="sub-s"' in once)
check("冪等である（2回当てても同じ）", m.cell(once) == once)
check("3回当てても同じ", m.cell(m.cell(once)) == once)
check("引用は割らない", m.cell("「原文のことば ── これも原文」") == "「原文のことば ── これも原文」")
check("区切りが無ければ触らない", m.cell("ただの一文である") == "ただの一文である")
check("文字列でなければ触らない", m.cell(None) is None)
check("同じ入力から同じものが出る", m.cell(x) == once)
check("二重の太字にならない", once.count("<b") == 1)

rows = m._pairs(["甲 ── 乙", "丙"], "左", "右")
check("表に組める", "<table" in rows and "<th>左</th>" in rows)
check("区切りが無い行は右が空", "<td></td>" in rows)

p = m.prev("B′", "前の答えである", "理由である ── くわしく", "「利用者の言葉 ── そのまま」")
check("閉じた答えが3行の表になる", p.count("<tr>") == 3)
check("利用者の言葉は割らない", "「利用者の言葉 ── そのまま」" in p)
print(f"\n{ok} 件すべて通った")

# ── 印を付けたあとでも、属性の中では割らない ──────────────
marked = m.cell(m._mark("主張である ── 説明である", "前の中身 ── その続き", "この回で変わった"))
assert "data-w=" not in re.sub(r"<[^>]*>", "", marked), "属性が本文へ漏れた"
print("  ok 印の属性が、本文へ漏れない")
rows2 = m._pairs([m._mark("甲 ── 乙", "丙 ── 丁", "この回で変わった")], "左", "右")
assert "data-b=" not in re.sub(r"<[^>]*>", "", rows2), "属性が欄へ漏れた"
print("  ok 表の欄にも漏れない")

# ── 完成イメージは畳まない ────────────────────────────────
# **畳むと、答えは文章だけで届く**（実際に「図と完成イメージから全く
# イメージがわかない」と差し戻された）
img = m.Topic(no=9, label="試し", question="問い", status="open",
              pick=("A", "答え"), example="<b>できあがるもの</b>",
              figures=[("<svg></svg>", "図の説明")],
              grounds=[("甲", "根拠", "assumption", "出どころ")])
html = m.deck("試し", [img], board="t", round_no=1)


def _opened(mark):
    i = html.index(mark)
    return "open" in html[html.rfind("<details", 0, i):i]


assert _opened("この答えの完成イメージ"), "完成イメージが畳まれている"
print("  ok 完成イメージは開いたまま出る")
assert not _opened("この答えの前提"), "裏づけまで開いている"
print("  ok 裏づけは畳んだまま出る")


# ── 図は、入れ物からはみ出さない ────────────────────────────
# **上限は2つ同時に当てる** ── 元の幅と、入れ物の幅である。片方だけを
# 行内の style へ書くと、それが板の CSS に勝ち、狭い入れ物からはみ出す
import tempfile as _tmp2
from lib import render_board as _rb2  # noqa: E402

with _tmp2.TemporaryDirectory() as _d:
    _f = pathlib.Path(_d)
    (_f / "w.svg").write_text('<svg viewBox="0 0 968 340" width="968" height="340"></svg>',
                              encoding="utf-8")
    out = _rb2._figure({"name": "w", "caption": "図"}, _f)
    assert "min(100%,968px)" in out, f"上限が1つしか当たっていない: {out[:120]}"
    assert ' width="968"' not in out, "固定幅が残っている"
print("  ok 図の上限は、元の幅と入れ物の幅の両方である")

print("\n17 件すべて通った")
