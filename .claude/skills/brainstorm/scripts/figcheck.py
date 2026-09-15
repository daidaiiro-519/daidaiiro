# SPDX-License-Identifier: MIT
# Copyright (c) 2026 daidaiiro
"""図の中の文字が、重なっていないか・枠からはみ出していないかを見る。

  python3 figcheck.py <図を持つモジュール名> ...

**目で見つける前に、機械で落とす。**実際に、矢印が箱を貫き、
注記が2つ重なって別の文になっている図を出してしまった。

見るのは5つ。文字の重なり ・ 枠からのはみ出し ・ 線が箱を貫くこと ・ 空白で字下げ ・ 箱が文字を覆うこと。
"""
import importlib
import re
import sys


def check(name, fig):
    svg = fig[0] if isinstance(fig, tuple) else fig
    items = []
    for m in re.finditer(r'<text x="([\d.]+)" y="([\d.]+)"([^>]*)>(.*?)</text>', svg):
        x, y, attr = float(m.group(1)), float(m.group(2)), m.group(3)
        txt = re.sub(r"<[^>]+>", "", m.group(4))
        size = float(re.search(r'font-size="([\d.]+)"', attr).group(1)) if "font-size" in attr else 10
        # 全角は1文字ぶん、英数字と記号は約0.55文字ぶんで数える
        w = size * sum(1.0 if ord(c) > 0x2E80 else 0.55 for c in txt)
        if "middle" in attr:
            x0 = x - w / 2
        elif 'text-anchor="end"' in attr:
            x0 = x - w
        else:
            x0 = x
        items.append((y, x0, x0 + w, txt))
    vb = [float(v) for v in re.search(r'viewBox="0 0 ([\d.]+) ([\d.]+)"', svg).groups()]
    items.sort()
    bad = [(y, t, t2) for i, (y, a, b, t) in enumerate(items)
           for (y2, a2, b2, t2) in items[i + 1:]
           if abs(y - y2) < 10 and a < b2 - 2 and a2 < b - 2]
    over = [t for (y, a, b, t) in items if b > vb[0] + 2 or y > vb[1]]

    # 線が箱を貫いていないか（縦横の線だけを見る）
    rects = [tuple(map(float, m)) for m in
             re.findall(r'<rect x="([\d.]+)" y="([\d.]+)" width="([\d.]+)" height="([\d.]+)"', svg)]
    pierced = []
    for m in re.finditer(r'<line x1="([\d.]+)" y1="([\d.]+)" x2="([\d.]+)" y2="([\d.]+)"', svg):
        x1, y1, x2, y2 = map(float, m.groups())
        for rx, ry, rw, rh in rects:
            if x1 == x2 and rx < x1 < rx + rw and min(y1, y2) < ry and ry + rh < max(y1, y2):
                pierced.append(f"縦線 x={x1} が箱（y {ry}〜{ry + rh}）を貫いている")
            if y1 == y2 and ry < y1 < ry + rh and min(x1, x2) < rx and rx + rw < max(x1, x2):
                pierced.append(f"横線 y={y1} が箱（x {rx}〜{rx + rw}）を貫いている")

    # 空白で字下げを作っていないか（SVG は行頭の空白を落とすので、階層が潰れる）
    indented = [t for t in re.findall(r'<text[^>]*>(.*?)</text>', svg)
                if re.match(r'^(?:\s|&#160;|\u3000){2,}\S', re.sub(r'<[^>]+>', '', t))]

    # 箱の枠線が、文字を横切っていないか（中に収まっているラベルは当たらない）
    covered = []
    for m in re.finditer(r'<rect x="([\d.]+)" y="([\d.]+)" width="([\d.]+)" height="([\d.]+)"', svg):
        rx, ry, rw, rh = map(float, m.groups())
        edges = [("縦", rx, ry, ry + rh), ("縦", rx + rw, ry, ry + rh)]
        hedges = [("横", ry, rx, rx + rw), ("横", ry + rh, rx, rx + rw)]
        for _, ex, y1, y2 in edges:
            for (ty, a0, b0, t) in items:
                if a0 + 2 < ex < b0 - 2 and y1 < ty - 3 and ty + 3 < y2:
                    covered.append(f"箱の縦の枠（x={ex}）が「{t[:22]}」を横切っている")
        for _, ey, x1, x2 in hedges:
            for (ty, a0, b0, t) in items:
                if x1 < a0 and b0 < x2 and ty - 8 < ey < ty + 3:
                    covered.append(f"箱の横の枠（y={ey}）が「{t[:22]}」を横切っている")

    print(f"{name}: 文字 {len(items)} 件 ／ 重なり {len(bad)} 件 ／ はみ出し {len(over)} 件"
          f" ／ 貫通 {len(pierced)} 件 ／ 空白の字下げ {len(indented)} 件 ／ 覆い {len(covered)} 件")
    for c in covered[:6]:
        print(f"   {c}")
    for t in indented[:6]:
        print(f"   空白で字下げしている: 「{re.sub(r'<[^>]+>', '', t)[:30]}」 ── x の値で表す")
    for p in pierced:
        print(f"   {p}")
    for y, t, t2 in bad:
        print(f"   重なり y={y}: 「{t[:26]}」 × 「{t2[:26]}」")
    for t in over:
        print(f"   はみ出し: 「{t[:40]}」")
    return len(bad) + len(over) + len(pierced) + len(indented) + len(covered)


def main() -> int:
    bad = 0
    for mod in sys.argv[1:]:
        m = importlib.import_module(mod)
        for attr in dir(m):
            v = getattr(m, attr)
            if attr.isupper() and isinstance(v, tuple) and v and isinstance(v[0], str) and "<svg" in v[0]:
                bad += check(f"{mod}.{attr}", v)
    print("食い違いは無い" if not bad else f"直すところが {bad} 件ある")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
