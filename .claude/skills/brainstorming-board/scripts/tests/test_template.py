"""型（テンプレート）が満たすこと。

**形はテンプレートが持ち、組み立ては値を差し込むだけである** ──
形をコードの中の文字列に散らすと、板ごとに違う形が出る（実際に出た）。
"""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))
from lib import template  # noqa: E402

ok = 0


def check(name, cond):
    global ok
    assert cond, name
    ok += 1
    print("  ok", name)


# ── 差し込みの過不足は、その場で例外になる ──────────────────
check("部品を組める", template.part("fold", summary="見出し", body="中身")
      == "<details><summary>見出し</summary><div>中身</div></details>")

for bad, why in (
    (dict(summary="見出し"), "足りない値を渡すと例外になる"),
    (dict(summary="見", body="中", extra="余"), "余分な値を渡すと例外になる"),
):
    try:
        template.part("fold", **bad)
        raise AssertionError(why)
    except KeyError:
        ok += 1
        print("  ok", why)

try:
    template.part("無い部品")
    raise AssertionError("知らない部品は例外になる")
except KeyError:
    ok += 1
    print("  ok", "知らない部品は例外になる")

# ── 形の正本は1枚である ────────────────────────────────
check("テンプレートは1枚である", template.TEMPLATE.name == "board.template.html")
check("部品が揃っている", {"page", "fold", "table", "card", "front", "tab"} <= set(template.names()))

# ── 組み立ての側に、構造を作る文字列が残っていないか ──────────
src = (pathlib.Path(__file__).resolve().parents[1] / "lib" / "build_board.py").read_text(encoding="utf-8")
body = src[src.index("def _fold"):]          # CSS と JS より後ろだけを見る
for tag in ("<details>", "<section", "<figure>", '<div class="card"', '<div class="ans"'):
    check(f"組み立てが {tag} を直に書いていない", tag not in body)

print(f"\n{ok} 件すべて通った")
