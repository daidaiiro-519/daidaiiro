# SPDX-License-Identifier: MIT
"""JSON を正本として、ブレストボードを組む。

    python3 scripts/cli.py render <ボードのディレクトリ>
    python3 scripts/cli.py render <ボードのディレクトリ> --check 1   冪等を検査する

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
from .template import part as _t
from . import tokens as _tok                                       # noqa: E402
from .board_css import drop_numbering                              # noqa: E402
from .build_board import Option, Table, Topic, cell, deck, write   # noqa: E402

_LETTERS = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"

# 出来事の種別。**札の色は、前提の表（KINDS）と同じ意味で割り当てる。**
# **機械が分岐する値は ASCII で、画面へ出す語はこの表が保持する** ──
# 1つの語が識別子と表示を兼ねると、表示を直した瞬間に分岐が壊れる。
_EVENT_KIND = {"returned": "k-given", "obsolete": "k-open",
               "finding": "k-fact", "content": "k-rule"}
_EVENT_LABEL = {"returned": "差し戻し", "obsolete": "失効した点",
                "finding": "判明事項", "content": "内容"}
# 扱いの3値。出どころの5値とは別の系列だが、札の形は共有する。
_TREATMENT = {"out": "k-rule", "later": "k-given", "resolved": "k-fact"}
_TREATMENT_LABEL = {"out": "対象外", "later": "後続で決定", "resolved": "解消済"}


# ── 宣言を組み上げる ──────────────────────────────────

def _events(row: list[tuple[str, str]]) -> str:
    """出来事を、種別ごとのまとまりにする。

    **同じ種別が複数あるなら、1つの見出しの下へ束ねて番号を付与する。**
    """
    if not row:
        return ""
    body, i = [], 0
    while i < len(row):
        k = row[i][0]
        j = i
        while j < len(row) and row[j][0] == k:
            j += 1
        n = j - i
        count = _t("event-count", count=n) if n > 1 else ""
        for m in range(n):
            head = (_t("event-head", span=n, kind=_EVENT_KIND[k],
                       label=_EVENT_LABEL[k], count=count) if m == 0 else "")
            num = _t("event-num", n=m + 1) if n > 1 else ""
            end = " g-end" if m == n - 1 and j < len(row) else ""
            body.append(_t("event-row", tag=k, end=end, head=head, num=num,
                           body=row[i + m][1]))
        i = j
    return _t("events", rows="".join(body))


def _figure(b: dict, figure_src: pathlib.Path) -> str:
    """図は `figures/<名前>.svg` を毎回読む。

    **正規化した SVG を JSON へ格納しない** ── 固定幅を外す処理は
    自身の出力へ再適用すると `max-width` が消失し、冪等でなくなる。
    """
    svg = (figure_src / f"{b['name']}.svg").read_text(encoding="utf-8")
    m = re.search(r'\swidth="([\d.]+)"', svg)
    if m:
        svg = svg.replace(m.group(0), "", 1)
        svg = svg.replace("<svg ", f'<svg style="max-width:{m.group(1)}px" ', 1)
    note = _t("figcaption", text=b["caption"]) if b.get("caption") else ""
    return _t("figure-top", svg=svg, caption=note)


def build(blocks, figure_src: pathlib.Path) -> str:
    """宣言の並びを HTML へ組む。**種類ごとに1つの形だけを持つ。**"""
    if isinstance(blocks, str):
        return blocks
    out = []
    for b in blocks:
        k = b["kind"]
        if k == "para":
            body_text = b["text"] + build(b.get("nested", []), figure_src)
            out.append(body_text if body_text.startswith("<b>")
                       else _t("para", body=body_text))
        elif k == "note":
            out.append(_t("note-s", body=b["text"]))
        elif k == "heading":
            out.append(_t("heading", level=b["level"], text=b["text"]))
        elif k == "html":
            out.append(_t("code", text=b["text"]))
        elif k == "list":
            t = "ol" if b.get("ordered") else "ul"
            item = "".join(
                _t("list-item",
                   body=cell(x["text"]) + build(x.get("nested", []), figure_src))
                for x in b["items"])
            out.append(_t("list", tag=t, items=item))
        elif k == "fold":
            out.append(_t("fold-why", summary=b["heading"],
                          body=build(b["body"], figure_src)))
        elif k == "card":
            mark = _t("card-mark", letter=b["letter"]) if b.get("letter") else ""
            row = [(e["tag"], _t("quote", text=e["text"])
                   if e["tag"] == "returned" else
                   (cell(e["text"]) if e["tag"] == "finding" else e["text"]))
                  for e in b.get("events", [])]
            out.append(_t("card", mark=mark, heading=b["heading"],
                          events=_events(row)))
        elif k == "table":
            row = "".join(
                _t("table-row-head", head=a,
                   cells="".join(_t("table-td", cell=cell(str(v))) for v in vs))
                for a, vs in b["rows"])
            head = "".join(_t("table-th", cell=c) for c in [""] + list(b["cols"]))
            out.append(_t("table-cls", cls="fact",
                          head=_t("table-thead", row=_t("table-head", cells=head)),
                          rows=_t("table-tbody", rows=row)))
        elif k == "grid":
            head = "".join(_t("table-th", cell=c) for c in b["cols"])
            body = "".join(
                _t("table-row", cells="".join(_t("table-td", cell=c) for c in r))
                for r in b["rows"])
            out.append(_t("table", 
                          head=_t("table-thead", row=_t("table-head", cells=head)),
                          rows=_t("table-tbody", rows=body)))
        elif k == "previous":
            row = [("前の答え", _t("lead", text=b["letter"]) + "　" + b["body"]),
                 ("なぜ組み直したか", cell(b["why"]))]
            if b.get("user_words"):
                row.append(("利用者の言葉（そのまま）", b["user_words"]))
            body = "".join(_t("table-row-head", head=a,
                              cells=_t("table-td", cell=c)) for a, c in row)
            out.append(_t("table", head="", rows=body))
        elif k == "figure":
            out.append(_figure(b, figure_src))
        else:
            raise ValueError(f"知らない宣言の種類: {k}")
    return "".join(out)


# ── 節の構造を、生成の時点で確定させる ──────────────────

def _take_container(s: str, head: str) -> tuple[int, int, int] | None:
    """入れ子を数えて `<details>` の範囲を取る。"""
    i = s.find(head)
    if i < 0:
        return None
    j, depth = i, 0
    while True:
        m = re.compile(r"<details\b|</details>").search(s, j)
        if not m:
            raise ValueError("details が閉じていない")
        depth += 1 if m.group(0) == "<details" else -1
        j = m.end()
        if depth == 0:
            return i, s.index(">", s.index("</summary>", i)) + 1, j


def _split_section(s: str, label: dict[str, str]) -> str:
    """道具が1つにまとめた3つ（道筋 ・ 要求する事項 ・ 前の答え）を、3つへ割る。

    **1つの欄に1つのことだけを入れる。** 3つは別のことなので、名前も別になる。
    """
    head = '<details><summary>経過 ── 道筋と、そこで分かったこと</summary>'
    while True:
        r = _take_container(s, head)
        if not r:
            return s
        i, body_head, j = r
        mid = s[body_head:j - len("</details>")]
        mid = mid[:mid.rindex("</div>")]
        returned = 'class="g-returned' in mid
        section, pos, table_depth = [], 0, 0
        for m in re.finditer(r'<p class="note-s">(.*?)</p>|<table\b|</table>', mid, re.S):
            if m.group(0) == "<table":
                table_depth += 1
                continue
            if m.group(0) == "</table>":
                table_depth -= 1
                continue
            if table_depth:
                continue
            if section:
                section[-1][1] = mid[pos:m.start()]
            section.append([m.group(1), ""])
            pos = m.end()
        if section:
            section[-1][1] = mid[pos:]
        group = []
        for name, body in section:
            note = ""
            mm = re.fullmatch(r"そう判断するまで（道筋 (\d+)手）", name)
            if mm:
                name = (label["history"].format(n=mm.group(1)) if returned
                      else label["progress"].format(n=mm.group(1)))
                note = _t("note-s", body="古い順")
            group.append(_t("fold", summary=name, body=note + body))
        s = s[:i] + "".join(group) + s[j:]


def _normalize_name(s: str, passed: int, dropped: int) -> str:
    """節の名前の主語を「この答え」で揃え、件数を名前へ出す。"""
    s = s.replace(
        "<summary>この答えが残った理由 ── 反証を通過した案と、除外した案</summary>",
        f"<summary>この答えが残った理由 ── 通過 {passed}件 ／ 除外 {dropped}件</summary>", 1)
    return re.sub(r"<summary>前提 ── この論証が乗っているもの（(\d+)件）</summary>",
                  lambda m: f"<summary>この答えの前提（{m.group(1)}件） ── "
                            "何に依拠しているか</summary>", s)


# ── 入力から Topic を組む ────────────────────────────

def to_topic(d: dict, figure_src: pathlib.Path) -> Topic:
    decision = d.get("decision")
    return Topic(
        no=d["no"], label=d["name"], status=d["status"], question=d["question"],
        answer=cell(d["answer"]),
        note=d.get("intro"),
        pick=(decision["letter"], build(decision["text"], figure_src)) if decision else None,
        example=build(d.get("example", []), figure_src) or None,
        figures=[(build([f], figure_src).replace('<figure class="fig-top">', "")
                  .replace("</figure>", "")
                  .replace(f'<figcaption>{f.get("caption","")}</figcaption>', ""),
                  f.get("caption", "")) for f in d.get("figures", [])],
        kept=[Option(o["name"], o["body"], o["cost"]) for o in d.get("passed", [])],
        dropped=[(x["body"], x["reason"]) for x in d.get("dropped", [])],
        path=[build([b], figure_src) for b in d.get("path", [])],
        found=[cell(x) for x in d.get("findings", [])],
        grounds=[(g["supports"], cell(g["basis"]), g["tag"], g["source"])
                 for g in d.get("grounds", [])],
        costs=[cell(x) for x in d.get("requirements", [])],
        weaknesses=[(w["item"], _tagged(w)) if w.get("treatment") else w["item"]
                    for w in d.get("out_of_scope", [])],
        extras=[(e["heading"], build(e["body"], figure_src)) for e in d.get("panels", [])],
        tables=[Table(caption=tb["caption"], columns=list(tb["cols"]),
                      rows={r[0]: list(r[1]) for r in tb["rows"]},
                      lead=tb.get("lead"), plain=tb.get("plain", False))
                for tb in d.get("tables", [])],
    )


def _tagged(w: dict) -> str:
    """扱いを札にする ── 太字だけでは、同じ意味の印が2種になる。

    **札は入力が3値で持ち、画面へ出す語はこの対応表が保持する** ──
    入力へ markup を書くと、散文の検査がそこへ当たらなくなる。
    """
    tag = w["treatment"]
    note = w.get("note", "")
    return (_t("kind", cls=_TREATMENT[tag], label=_TREATMENT_LABEL[tag])
            + (f" ── {note}" if note else ""))


def _grounds_order(gs):
    """確かさの順に並べ替える ── 外の根拠 → こちらの決まり → 実測 → 前提 → 未確認。"""
    order = {"primary": 0, "rule": 1, "measured": 2,
             "assumption": 3, "unverified": 4}
    return sorted(gs, key=lambda g: order.get(g[2], 9))


# ── 組み立て ────────────────────────────────────────

def prepare(d: dict, figure_src: pathlib.Path) -> list:
    """入力から論点を起こし、**並べ替えまで済ませる。**

    **組み立てと基準の保存が、同じものを見るようにする** ── 片方だけが
    根拠を並べ替えていたので、根拠の欄が毎回「変わった」と出ていた（実測 140 か所）。
    """
    topics = [to_topic(t, figure_src) for t in d["topics"]]
    for t in topics:
        t.grounds = _grounds_order(t.grounds)
    return topics


def render(dir: pathlib.Path, *, verify: bool = True) -> str:
    if verify:
        from . import validate_input
        bad = validate_input.check(dir)
        if bad:
            raise SystemExit("入力の検査が通っていない ── HTML は書き出さない:\n  "
                             + "\n  ".join("× " + e for e in bad))
    d = json.loads((dir / "board.json").read_text(encoding="utf-8"))
    figure_src = dir / "figures"
    topics = prepare(d, figure_src)

    # 除外した案の記号は、通過した案の次から振る
    origin = {}
    for t in topics:
        used = {o.name for o in t.kept}
        origin[t.no] = _LETTERS.index(next(
            (c for c in _LETTERS[max((_LETTERS.index(c) for c in used if c in _LETTERS),
                                 default=-1) + 1:] if c not in used), "A"))

    # **トークンはここで置かない** ── 定義は write() が1回だけ置く。
    # 2か所から出すと、どちらが勝つかを document の順序に委ねることになる。
    err = _tok.validate(_tok.load())
    if err:
        raise SystemExit("トークンの検査が通っていない:\n  " + "\n  ".join(err))
    # 見た目は write() が正本から置く。ここで置くのは、
    # 入力の中身から計算した値だけである
    style = "<style>" + drop_numbering(origin) + "</style>"

    baseline = None
    round_no = d["round"]
    before = dir / "rounds" / f"{round_no - 1}.json"
    if before.exists():
        baseline = json.loads(before.read_text(encoding="utf-8")).get("snap")

    body = deck(theme=d["title"], topics=topics,
                intro=build(d.get("intro", []), figure_src), style=style,
                extras=[(e["heading"], build(e["body"], figure_src))
                        for e in d.get("panels", [])],
                board=d["board"], round_no=round_no, prev=baseline,
                queue=[(q["no"], q["why"]) for q in d.get("queue", [])])

    passed = sum(len(t.kept) for t in topics)
    body = _split_section(body, d.get("section_names", {
        "history": "この答えの履歴（{n}件） ── 差し戻しで何が失効し、何へ変更したか",
        "progress": "この答えに至る経過（{n}手） ── 何を問い、そこで何が判明したか"}))
    for t in topics:
        body = _normalize_name(body, len(t.kept), len(t.dropped))
    return body


# ブレストボードの置き場所の親。**init と同じ既定である**
BOARDS = ".brainstorming-board"


def board_dir(board: str) -> pathlib.Path:
    """名前でも道でも、同じ1つのフォルダへ解決する。

    **解決を1か所に置く** ── 道具ごとに違う解決をしていたので、名前で渡すと
    組み立てだけが黙って何もしないことがあった（実際にそうなった）。
    """
    p = pathlib.Path(board)
    if (p / "board.json").exists():
        return p.resolve()
    alt = pathlib.Path(BOARDS) / board
    if (alt / "board.json").exists():
        return alt.resolve()
    raise SystemExit(f"board.json が無い: {board} ── {p} にも {alt} にも見つからない")


def freeze(dir: pathlib.Path) -> int:
    """いまの入力を、次の回の基準として保存する。

    **組み立てと分離する** ── 組み立てが基準を書き出すと、
    1回目と2回目で出力が相違する。
    """
    from .build_board import snapshot
    d = json.loads((dir / "board.json").read_text(encoding="utf-8"))
    topics = prepare(d, dir / "figures")
    out = dir / "rounds" / f'{d["round"]}.json'
    out.parent.mkdir(exist_ok=True)
    out.write_text(json.dumps({"round": d["round"], "snap": snapshot(topics)},
                              ensure_ascii=False, indent=1), encoding="utf-8")
    print(f"基準を保存: {out.relative_to(dir)} ── 次は board.json の「回」を "
          f'{d["round"] + 1} へ進める')
    return 0


def _check_idempotent(dir: pathlib.Path, first: str) -> int:
    """**2回の一致では不足する。** 入力が変化しないことと、
    読み取り専用でも通ることを、あわせて検査する。"""
    import hashlib
    before = {p: hashlib.sha256(p.read_bytes()).hexdigest()
         for p in sorted(dir.rglob("*")) if p.is_file() and p.name != "board.html"}
    second = render(dir)
    after = {p: hashlib.sha256(p.read_bytes()).hexdigest()
         for p in sorted(dir.rglob("*")) if p.is_file() and p.name != "board.html"}
    bad = []
    if first != second:
        bad.append(f"2回の生成物が相違する（{len(first)} と {len(second)} バイト）")
    for p in sorted(set(before) | set(after)):
        if before.get(p) != after.get(p):
            bad.append(f"入力が書き換わった: {p.relative_to(dir)}")
    with tempfile.TemporaryDirectory() as td:
        many = pathlib.Path(td) / dir.name
        shutil.copytree(dir, many)
        # 生成物は入力ではない ── 消してから、入力だけを読み取り専用にする
        (many / "board.html").unlink(missing_ok=True)
        for p in many.rglob("*"):
            if p.is_file():
                p.chmod(0o444)
        # **唯一の入口から呼ぶ** ── 部品を直接起動する形は、契約の外である
        cli = pathlib.Path(__file__).resolve().parents[1] / "cli.py"
        r = subprocess.run([sys.executable, str(cli), "render", str(many)],
                           capture_output=True, text=True)
        if r.returncode:
            bad.append("読み取り専用の複製で異常終了した: "
                      + (r.stderr.strip().splitlines() or ["(出力無し)"])[-1])
    for e in bad:
        print("  ×", e, file=sys.stderr)
    print(("冪等の検査　通った" if not bad
           else f"冪等の検査　通っていない（{len(bad)} 件）"),
          file=sys.stderr if bad else sys.stdout)
    return 1 if bad else 0
