# SPDX-License-Identifier: MIT
"""JSON を正本として、ブレストボードを組む。

    python3 render_board.py <ボードのディレクトリ>
    python3 render_board.py <ボードのディレクトリ> --check   冪等を検査する

**JSON が正本で、HTML は生成物である。** 具体の側は Python を1行も保持しない。

**この道具は入力を書き換えない。** 前の回の基準は `rounds/<番号>.json` から
読むだけで、書き出さない ── 書き出すと、1回目と2回目で出力が相違する
（実際に 2209 行相違した）。基準の前進は `freeze` が担当する。

**節の構造は、生成の時点で確定させる。** 閲覧する側の script で
組み直すと、保存した HTML の中に構造が存在しない。
"""
from __future__ import annotations

import html as _h
import json
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

_HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(_HERE))

import tokens as _tok                                    # noqa: E402
from board_css import CSS as _CSS, 除外の採番             # noqa: E402
from build_board import Option, Table, Topic, cell, deck, write  # noqa: E402

_英字 = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

# 出来事の種別。**札の色は、前提の表（KINDS）と同じ意味で割り当てる。**
_種別 = {"差し戻し": "k-given", "失効した点": "k-open",
        "判明事項": "k-fact", "内容": "k-rule"}
# 扱いの3値。出どころの5値とは別の系列だが、札の形は共有する。
_扱い = {"対象外": "k-rule", "後続で決定": "k-given", "解消済": "k-fact"}


# ── 宣言を組み上げる ──────────────────────────────────

def _出来事(行: list[tuple[str, str]]) -> str:
    """出来事を、種別ごとのまとまりにする。

    **同じ種別が複数あるなら、1つの見出しの下へ束ねて番号を付与する。**
    """
    if not 行:
        return ""
    体, i = [], 0
    while i < len(行):
        k = 行[i][0]
        j = i
        while j < len(行) and 行[j][0] == k:
            j += 1
        n = j - i
        件 = f"<small>{n}件</small>" if n > 1 else ""
        for m in range(n):
            頭 = (f'<th rowspan="{n}"><span class="kind {_種別[k]}">{k}</span>{件}</th>'
                  if m == 0 else "")
            番 = f'<span class="ev-n">{m + 1}</span>' if n > 1 else ""
            終 = " g-end" if m == n - 1 and j < len(行) else ""
            体.append(f'<tr class="g-{k}{終}">{頭}<td>{番}{行[i + m][1]}</td></tr>')
        i = j
    return f'<table class="ev"><tbody>{"".join(体)}</tbody></table>'


def _図(b: dict, 図の元: pathlib.Path) -> str:
    """図は `figures/<名前>.svg` を毎回読む。

    **正規化した SVG を JSON へ格納しない** ── 固定幅を外す処理は
    自身の出力へ再適用すると `max-width` が消失し、冪等でなくなる。
    """
    svg = (図の元 / f"{b['名前']}.svg").read_text(encoding="utf-8")
    m = re.search(r'\swidth="([\d.]+)"', svg)
    if m:
        svg = svg.replace(m.group(0), "", 1)
        svg = svg.replace("<svg ", f'<svg style="max-width:{m.group(1)}px" ', 1)
    注 = f"<figcaption>{b['注記']}</figcaption>" if b.get("注記") else ""
    return f'<figure class="fig-top">{svg}{注}</figure>'


def 組む(blocks, 図の元: pathlib.Path) -> str:
    """宣言の並びを HTML へ組む。**種類ごとに1つの形だけを持つ。**"""
    if isinstance(blocks, str):
        return blocks
    out = []
    for b in blocks:
        k = b["種類"]
        if k == "文":
            本文 = b["本文"] + 組む(b.get("入れ子", []), 図の元)
            out.append(本文 if 本文.startswith("<b>") else f"<p>{本文}</p>")
        elif k == "注記":
            out.append(f'<p class="note-s">{b["本文"]}</p>')
        elif k == "見出し":
            out.append(f'<h{b["段"]}>{b["本文"]}</h{b["段"]}>')
        elif k == "組み上がり":
            out.append(f"<pre><code>{b['本文']}</code></pre>")
        elif k == "箇条":
            t = "ol" if b.get("順序") else "ul"
            項 = "".join(f'<li>{cell(x["本文"])}{組む(x.get("入れ子", []), 図の元)}</li>'
                        for x in b["項目"])
            out.append(f"<{t}>{項}</{t}>")
        elif k == "開閉":
            out.append(f'<details class="why-in"><summary>{b["見出し"]}</summary>'
                       f'<div>{組む(b["中身"], 図の元)}</div></details>')
        elif k == "カード":
            記 = f'<span class="ver">{b["記号"]}</span>' if b.get("記号") else ""
            行 = [(e["札"], f'<span class="q-in">{e["本文"]}</span>'
                   if e["札"] == "差し戻し" else
                   (cell(e["本文"]) if e["札"] == "判明事項" else e["本文"]))
                  for e in b.get("出来事", [])]
            out.append(f'<div class="card"><div class="card-h">{記}{b["見出し"]}</div>'
                       f"{_出来事(行)}</div>")
        elif k == "表":
            行 = "".join(f"<tr><th>{a}</th>"
                        + "".join(f"<td>{cell(str(v))}</td>" for v in vs) + "</tr>"
                        for a, vs in b["行"])
            頭 = "".join(f"<th>{c}</th>" for c in [""] + list(b["列"]))
            out.append(f'<div class="scroll"><table class="fact"><thead><tr>{頭}</tr>'
                       f"</thead><tbody>{行}</tbody></table></div>")
        elif k == "卓":
            頭 = "".join(f"<th>{c}</th>" for c in b["列"])
            体 = "".join("<tr>" + "".join(f"<td>{c}</td>" for c in r) + "</tr>"
                        for r in b["行"])
            out.append(f'<div class="scroll"><table><thead><tr>{頭}</tr></thead>'
                       f"<tbody>{体}</tbody></table></div>")
        elif k == "前の答え":
            行 = [("前の答え", f'<b>{b["記号"]}</b>　{b["中身"]}'),
                 ("なぜ組み直したか", cell(b["なぜ"]))]
            if b.get("利用者の言葉"):
                行.append(("利用者の言葉（そのまま）", b["利用者の言葉"]))
            体 = "".join(f"<tr><th>{a}</th><td>{c}</td></tr>" for a, c in 行)
            out.append(f'<div class="scroll"><table>{体}</table></div>')
        elif k == "図":
            out.append(_図(b, 図の元))
        else:
            raise ValueError(f"知らない宣言の種類: {k}")
    return "".join(out)


# ── 節の構造を、生成の時点で確定させる ──────────────────

def _器を取る(s: str, 頭: str) -> tuple[int, int, int] | None:
    """入れ子を数えて `<details>` の範囲を取る。"""
    i = s.find(頭)
    if i < 0:
        return None
    j, 深さ = i, 0
    while True:
        m = re.compile(r"<details\b|</details>").search(s, j)
        if not m:
            raise ValueError("details が閉じていない")
        深さ += 1 if m.group(0) == "<details" else -1
        j = m.end()
        if 深さ == 0:
            return i, s.index(">", s.index("</summary>", i)) + 1, j


def _節を割る(s: str, 名前: dict[str, str]) -> str:
    """道具が1つの器へ入れた3つ（道筋 ・ 要求する事項 ・ 前の答え）を、器3つへ割る。

    **1つの器に1つのことだけを入れる。** 3つは別のことなので、名前も別になる。
    """
    頭 = '<details><summary>経過 ── 道筋と、そこで分かったこと</summary>'
    while True:
        r = _器を取る(s, 頭)
        if not r:
            return s
        i, 中身の頭, j = r
        中 = s[中身の頭:j - len("</details>")]
        中 = 中[:中.rindex("</div>")]
        差し戻し = 'class="g-差し戻し' in 中
        節, 位置, 表の深さ = [], 0, 0
        for m in re.finditer(r'<p class="note-s">(.*?)</p>|<table\b|</table>', 中, re.S):
            if m.group(0) == "<table":
                表の深さ += 1
                continue
            if m.group(0) == "</table>":
                表の深さ -= 1
                continue
            if 表の深さ:
                continue
            if 節:
                節[-1][1] = 中[位置:m.start()]
            節.append([m.group(1), ""])
            位置 = m.end()
        if 節:
            節[-1][1] = 中[位置:]
        組 = []
        for 名, 体 in 節:
            注 = ""
            mm = re.fullmatch(r"そう判断するまで（道筋 (\d+)手）", 名)
            if mm:
                名 = (名前["履歴"].format(n=mm.group(1)) if 差し戻し
                      else 名前["経過"].format(n=mm.group(1)))
                注 = '<p class="note-s">古い順</p>'
            組.append(f"<details><summary>{名}</summary><div>{注}{体}</div></details>")
        s = s[:i] + "".join(組) + s[j:]


def _名を揃える(s: str, 通過: int, 除外: int) -> str:
    """節の名前の主語を「この答え」で揃え、件数を名前へ出す。"""
    s = s.replace(
        "<summary>この答えが残った理由 ── 反証を通過した案と、除外した案</summary>",
        f"<summary>この答えが残った理由 ── 通過 {通過}件 ／ 除外 {除外}件</summary>", 1)
    return re.sub(r"<summary>前提 ── この論証が乗っているもの（(\d+)件）</summary>",
                  lambda m: f"<summary>この答えの前提（{m.group(1)}件） ── "
                            "何に依拠しているか</summary>", s)


# ── 入力から Topic を組む ────────────────────────────

def 論点へ(d: dict, 図の元: pathlib.Path) -> Topic:
    決定 = d.get("決定")
    return Topic(
        no=d["番号"], label=d["名前"], status=d["状態"], question=d["問い"],
        answer=cell(d["答え"]),
        note=d.get("前書き"),
        pick=(決定["記号"], 組む(決定["本文"], 図の元)) if 決定 else None,
        example=組む(d.get("実例", []), 図の元) or None,
        figures=[(組む([f], 図の元).replace('<figure class="fig-top">', "")
                  .replace("</figure>", "")
                  .replace(f'<figcaption>{f.get("注記","")}</figcaption>', ""),
                  f.get("注記", "")) for f in d.get("図", [])],
        kept=[Option(o["記号"], o["中身"], o["代償"]) for o in d.get("通過", [])],
        dropped=[(x["中身"], x["理由"]) for x in d.get("除外", [])],
        path=[組む([b], 図の元) for b in d.get("道筋", [])],
        found=[cell(x) for x in d.get("判明事項", [])],
        grounds=[(g["支える"], cell(g["もと"]), g["札"], g["出どころ"])
                 for g in d.get("前提", [])],
        costs=[cell(x) for x in d.get("要求", [])],
        weaknesses=[(w["事項"], _札付き(w["扱い"])) if "扱い" in w else w["事項"]
                    for w in d.get("扱わない", [])],
        extras=[(e["見出し"], 組む(e["中身"], 図の元)) for e in d.get("面", [])],
    )


def _札付き(扱い: str) -> str:
    """扱いの3値を札にする ── 太字だけでは、同じ意味の印が2種になる。"""
    for 語, cls in _扱い.items():
        if 扱い.startswith(語):
            return f"<span class='kind {cls}'>{語}</span>" + 扱い[len(語):]
    return 扱い


def _前提の順(gs):
    """確かさの順に並べ替える ── 外の根拠 → こちらの決まり → 実測 → 前提 → 未確認。"""
    札 = {"原典": 0, "決まり": 1, "実測": 2, "前提": 3, "未確認": 4}
    return sorted(gs, key=lambda g: 札.get(g[2], 9))


# ── 組み立て ────────────────────────────────────────

def render(dir: pathlib.Path, *, 検査する: bool = True) -> str:
    if 検査する:
        import validate_input
        悪 = validate_input.check(dir)
        if 悪:
            raise SystemExit("入力の検査が通っていない ── HTML は書き出さない:\n  "
                             + "\n  ".join("× " + e for e in 悪))
    d = json.loads((dir / "board.json").read_text(encoding="utf-8"))
    図の元 = dir / "figures"
    topics = [論点へ(t, 図の元) for t in d["論点"]]
    for t, src in zip(topics, d["論点"]):
        t.grounds = _前提の順(t.grounds)

    # 除外した案の記号は、通過した案の次から振る
    起点 = {}
    for t in topics:
        使用済 = {o.name for o in t.kept}
        起点[t.no] = _英字.index(next(
            (c for c in _英字[max((_英字.index(c) for c in 使用済 if c in _英字),
                                 default=-1) + 1:] if c not in 使用済), "A"))

    tk = _tok.load()
    err = _tok.validate(tk)
    if err:
        raise SystemExit("トークンの検査が通っていない:\n  " + "\n  ".join(err))
    style = ("<style>" + _tok.css(tk) + _CSS + 除外の採番(起点) + "</style>")

    基準 = None
    回 = d["回"]
    前 = dir / "rounds" / f"{回 - 1}.json"
    if 前.exists():
        基準 = json.loads(前.read_text(encoding="utf-8")).get("snap")

    body = deck(theme=d["題"], topics=topics,
                intro=style + 組む(d.get("前書き", []), 図の元),
                extras=[(e["見出し"], 組む(e["中身"], 図の元))
                        for e in d.get("面", [])],
                board=d["板"], round_no=回, prev=基準,
                queue=[(q["番号"], q["なぜ"]) for q in d.get("行列", [])])

    通過 = sum(len(t.kept) for t in topics)
    body = _節を割る(body, d.get("節の名前", {
        "履歴": "この答えの履歴（{n}件） ── 差し戻しで何が失効し、何へ変更したか",
        "経過": "この答えに至る経過（{n}手） ── 何を問い、そこで何が判明したか"}))
    for t in topics:
        body = _名を揃える(body, len(t.kept), len(t.dropped))
    return body


def freeze(dir: pathlib.Path) -> int:
    """いまの入力を、次の回の基準として保存する。

    **組み立てと分離する** ── 組み立てが基準を書き出すと、
    1回目と2回目で出力が相違する。
    """
    from build_board import snapshot
    d = json.loads((dir / "board.json").read_text(encoding="utf-8"))
    図の元 = dir / "figures"
    topics = [論点へ(t, 図の元) for t in d["論点"]]
    out = dir / "rounds" / f'{d["回"]}.json'
    out.parent.mkdir(exist_ok=True)
    out.write_text(json.dumps({"round": d["回"], "snap": snapshot(topics)},
                              ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"基準を保存: {out.relative_to(dir)} ── 次は board.json の「回」を "
          f'{d["回"] + 1} へ進める')
    return 0


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    dir = pathlib.Path(sys.argv[1]).resolve()
    if "--freeze" in sys.argv:
        return freeze(dir)
    body = render(dir)
    if "--check" in sys.argv:
        return _冪等を検査する(dir, body)
    write(body, str(dir / "board.html"), json.loads(
        (dir / "board.json").read_text(encoding="utf-8"))["題"])
    print(f"書き出し: {dir / 'board.html'}")
    return 0


def _冪等を検査する(dir: pathlib.Path, 一度目: str) -> int:
    """**2回の一致では不足する。** 入力が変化しないことと、
    読み取り専用でも通ることを、あわせて検査する。"""
    import hashlib
    前 = {p: hashlib.sha256(p.read_bytes()).hexdigest()
         for p in sorted(dir.rglob("*")) if p.is_file() and p.name != "board.html"}
    二度目 = render(dir)
    後 = {p: hashlib.sha256(p.read_bytes()).hexdigest()
         for p in sorted(dir.rglob("*")) if p.is_file() and p.name != "board.html"}
    悪 = []
    if 一度目 != 二度目:
        悪.append(f"2回の組み上がりが相違する（{len(一度目)} と {len(二度目)} バイト）")
    for p in sorted(set(前) | set(後)):
        if 前.get(p) != 後.get(p):
            悪.append(f"入力が書き換わった: {p.relative_to(dir)}")
    with tempfile.TemporaryDirectory() as td:
        複 = pathlib.Path(td) / dir.name
        shutil.copytree(dir, 複)
        # 組み上がりは入力ではない ── 消してから、入力だけを読み取り専用にする
        (複 / "board.html").unlink(missing_ok=True)
        for p in 複.rglob("*"):
            if p.is_file():
                p.chmod(0o444)
        r = subprocess.run([sys.executable, __file__, str(複)],
                           capture_output=True, text=True)
        if r.returncode:
            悪.append("読み取り専用の複製で異常終了した: "
                      + (r.stderr.strip().splitlines() or ["(出力無し)"])[-1])
    for e in 悪:
        print("  ×", e, file=sys.stderr)
    print(("冪等の検査　通った" if not 悪
           else f"冪等の検査　通っていない（{len(悪)} 件）"),
          file=sys.stderr if 悪 else sys.stdout)
    return 1 if 悪 else 0


if __name__ == "__main__":
    sys.exit(main())
