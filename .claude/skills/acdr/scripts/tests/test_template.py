"""型（テンプレート）が満たすこと。

**形はテンプレートが持ち、組み立ては値を差し込むだけである** ──
形をコードの中の文字列に散らすと、記録ごとに違う形が出る。
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
check("部品を組める", template.part("sec", heading="見出し", body="中身")
      == '<div class="sec"><h4>見出し</h4>中身</div>')

for bad, why in (
    (dict(heading="見出し"), "足りない値を渡すと例外になる"),
    (dict(heading="見", body="中", extra="余"), "余分な値を渡すと例外になる"),
):
    try:
        template.part("sec", **bad)
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
check("テンプレートは1枚である", template.TEMPLATE.name == "acdr.template.html")
check("部品が揃っている",
      {"acdr", "sec", "page", "page-bare", "pane-md", "pane-code", "pane-html",
       "mark", "code-block", "diff-block", "md-table"} <= set(template.names()))

# ── 入れ子の <template> を、部品の切れ目と取り違えない ──────────
frag = template.part("pane-html", key="k", tab="面", index="", shadowcss="", body="中")
check("面の雛形が、部品の中に残っている", frag.endswith("中</template></section>"))

# ── 印は、3つの面のどれでも同じ形である ──────────────────────
check("印の形が1つである",
      template.part("mark", before="前", why="なぜ", body="後")
      == '<mark class="chg" tabindex="0" role="button" aria-expanded="false"'
         ' data-b="前" data-w="なぜ">後</mark>')

# ── 組み立ての側に、構造を作る文字列が残っていないか ──────────
lib = pathlib.Path(__file__).resolve().parents[1] / "lib"
for name, tags in (("render_acdr.py", ('<div class="sec"', "<ul>", "<figure", "<table")),
                   ("code_diff.py", ("<tr", "<span class=", "<div class=")),
                   ("markdown.py", ("<h{", "<blockquote>", "<td>", "<hr>"))):
    body = (lib / name).read_text(encoding="utf-8")
    body = "\n".join(l for l in body.splitlines() if not l.lstrip().startswith("#"))
    for tag in tags:
        check(f"{name} が {tag} を組み立てに書いていない", f"'{tag}" not in body and f'"{tag}' not in body)

print(f"\n{ok} 件すべて通った")
