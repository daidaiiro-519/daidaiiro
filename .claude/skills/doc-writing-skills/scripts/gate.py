# -*- coding: utf-8 -*-
"""ゲート1を当てる。**0件にできるものだけを見る。**

  python3 gate.py <ファイル...>
  python3 gate.py --list
  python3 gate.py --synonyms 用語.tsv <ファイル...>

**この道具は3つの型でできている。**

  単位        文書を切ったもの。種別（本文・見出し・箇条書き・表のセル・引用・コード）と行番号を持つ
  検査        名前・拠って立つ概念・当てる単位の種別を持つ
  見つけたもの  検査の名前・概念・位置・抜粋

**検査は、概念から導けるものだけを置く。**
0件にできないもの（1文の長さ・段落の行数など）は、書き手が判定として当てる。

**1つだけ、概念から導けない検査がある。**
「書いた強調が描画されない」は媒体の決めであり、CommonMark の記法に拠る。
概念ではなく媒体に拠ることを、名簿に書いてある。
"""
from __future__ import annotations

import re
import sys
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

KINDS = frozenset({"本文", "見出し", "箇条書き", "表のセル", "引用", "コード"})
RENDERED = KINDS - {"コード"}
PROSE = frozenset({"本文", "箇条書き", "引用"})
EXEMPT_MARK = "<!-- doc-writing-skills: exempt -->"


@dataclass
class Unit:
    kind: str
    line: int
    raw: str
    text: str
    level: int = 0      # 見出しの深さ。見出し以外は0
    indent: int = 0     # 箇条書きの字下げ


@dataclass
class Finding:
    check: str
    basis: str          # 拠って立つもの。概念の番号、または「媒体の決め」
    line: int
    excerpt: str


@dataclass
class Check:
    name: str
    basis: str
    fn: Callable
    kinds: frozenset[str] | None = None
    note: str = ""


# ── 文書を単位へ切る ────────────────────────────────────────
_HEADING = re.compile(r"^(\s*)(#{1,6})(\s|$)")
_LIST = re.compile(r"^(\s*)([-*+]|\d+[.)])(\s|$)")
_QUOTE = re.compile(r"^\s*>")
_FENCE = re.compile(r"^\s*(`{3,}|~{3,})")
_TABLE = re.compile(r"^\s*\|")
_TABLE_SEP = re.compile(r"^\s*\|?[\s:|-]+\|[\s:|-]*\|?\s*$")
_INLINE = re.compile(r"\*\*|`|\[|\]\([^)]*\)|<br\s*/?>")


def _plain(s: str) -> str:
    return _INLINE.sub("", s).strip()


def split_units(text: str) -> list[Unit]:
    """文書を、位置を保ったまま単位へ切る。"""
    units: list[Unit] = []
    fence: tuple[str, int] | None = None
    in_front = False
    for i, raw in enumerate(text.split("\n"), 1):
        if i == 1 and raw.strip() == "---":
            in_front = True
            continue
        if in_front:
            if raw.strip() == "---":
                in_front = False
            continue
        m = _FENCE.match(raw)
        if m:
            run = m.group(1)
            if fence is None:
                fence = (run[0], len(run))
            elif run[0] == fence[0] and len(run) >= fence[1]:
                fence = None
            continue
        if fence is not None:
            units.append(Unit("コード", i, raw, raw.strip()))
            continue
        if not raw.strip():
            continue
        h = _HEADING.match(raw)
        if h:
            units.append(Unit("見出し", i, raw,
                              _plain(re.sub(r"^\s*#{1,6}\s*", "", raw)), level=len(h.group(2))))
            continue
        s = raw.lstrip()
        if _QUOTE.match(s):
            units.append(Unit("引用", i, raw, _plain(re.sub(r"^\s*>\s*", "", s))))
        elif _TABLE.match(s):
            if _TABLE_SEP.match(s):
                continue
            for cell in s.strip().strip("|").split("|"):
                if cell.strip():
                    units.append(Unit("表のセル", i, raw, _plain(cell)))
        elif _LIST.match(s):
            units.append(Unit("箇条書き", i, raw, _plain(_LIST.sub("", s, count=1)),
                              indent=len(raw) - len(s)))
        else:
            units.append(Unit("本文", i, raw, _plain(raw)))
    return units


# ── 文末の型 ────────────────────────────────────────────────
KEITAI = re.compile(r"(です|ます|ません|でした|ましょう|でしょう|ください)。?$")
JOTAI = re.compile(r"(である|だ|た|ない|る|い|う|く|す|つ|ぬ|ぶ|む)。$")


def ending(text: str) -> str:
    """文末を、敬体か、そうでないかに分ける。

    **体言止めと常体は、品詞を見ないと分けられない。**
    「扱い」は体言で「短い」は常体だが、字面は同じ形をしている。
    分けられないものを分けたことにせず、**敬体かどうかだけ**を見る。
    """
    t = text.strip().rstrip("。")
    return "敬体" if KEITAI.search(t + "。") or KEITAI.search(t) else "非敬体"


# ── 検査 ────────────────────────────────────────────────────
def _heading_skip(units: list[Unit]) -> list[Finding]:
    """概念4。見出しの階層を飛ばすと、何がどこにあるかが掴めない。"""
    out, prev = [], 0
    for u in units:
        if u.kind != "見出し":
            continue
        if prev and u.level > prev + 1:
            out.append(Finding("見出しの階層が飛んでいる", "概念4", u.line,
                               f"見出し{prev} の次に 見出し{u.level}：{u.text[:24]}"))
        prev = u.level
    return out


def _mixed_style(units: list[Unit]) -> list[Finding]:
    """概念6。文体が混ざると、読み手は書き分けに意味があると読む。

    **引用は数えない。**引用は書き手の文体ではなく、他人の文である。
    揃えようとすれば原文を書き換えることになり、引用でなくなる。

    引用の印は2つ ── 引用として置かれた単位か、鉤括弧の中か。
    表のセルに引いた他人の言葉は前者にならないので、後者で拾う。
    鉤括弧は文をまたぐので、深さを持って追う。
    """
    kinds = Counter()
    where: dict[str, tuple[int, str]] = {}
    for u in units:
        if u.kind not in PROSE or u.kind == "引用":
            continue
        depth = 0  # 鉤括弧の深さ。引用は文をまたぐので、1文ずつでは判定できない
        for s in re.split(r"(?<=。)", u.text):
            s = s.strip()
            inside = depth > 0 or s.startswith("「")
            depth += s.count("「") - s.count("」")
            if not s.endswith("。"):
                continue
            if inside:
                continue  # 鉤括弧の中は、他人の言葉である
            e = ending(s)
            if e == "非敬体" and not JOTAI.search(s):
                continue          # 体言止めは、常体とも敬体とも決められない
            kinds[e] += 1
            where.setdefault(e, (u.line, s[:26]))
    if len(kinds) > 1:
        a, b = where["敬体"], where["非敬体"]
        return [Finding("文体が混ざっている", "概念6", min(a[0], b[0]),
                        f"敬体 {kinds['敬体']} 文（{a[0]}行「{a[1]}」）と "
                        f"常体 {kinds['非敬体']} 文（{b[0]}行「{b[1]}」）")]
    return []


def _unparallel_items(units: list[Unit]) -> list[Finding]:
    """概念6。並んだ項目の形が揃っていないと、対応が読めない。"""
    out: list[Finding] = []
    group: list[Unit] = []

    def flush(g: list[Unit]) -> None:
        if len(g) < 2:
            return
        seen = {}
        for u in g:
            seen.setdefault(ending(u.text), u)
        if len(seen) > 1:
            names = " と ".join(f"{k}（{v.line}行）" for k, v in seen.items())
            out.append(Finding("並んだ項目の語尾が揃っていない", "概念6", g[0].line,
                               f"{len(g)} 項目に {names} が混ざる"))

    prev_line = -9
    for u in units:
        if u.kind == "箇条書き" and (not group or u.line - prev_line <= 1) \
           and (not group or u.indent == group[0].indent):
            group.append(u)
        else:
            flush(group)
            group = [u] if u.kind == "箇条書き" else []
        if u.kind == "箇条書き":
            prev_line = u.line
    flush(group)
    return out


BOX_CORNER = re.compile(r"[┌┐┘┏┓┛╭╮╯┬┴┼┤]")
BOX_RULE = re.compile(r"[─━]{4,}")


def _drawn_figure(u: Unit) -> list[str]:
    """概念2。図と表は、文字で描かず、図と表の形で置く。"""
    if BOX_CORNER.search(u.raw) or BOX_RULE.search(u.raw):
        return [u.raw.strip()[:28]]
    return []


SYNONYM_PAIRS: list[tuple[str, str]] = []


def _synonym(units: list[Unit]) -> list[Finding]:
    """概念7。同じ文脈では、1つの意味に1つの語だけを当てる。

    **インラインコードの中は見ない。**そこに在るのは識別子と原文の引用であり、
    書き手の言葉づかいではない。**言い換えれば、その鍵で引くものが壊れる。**
    """
    body = "\n".join(_without_code(u.raw) for u in units if u.kind != "コード")
    return [Finding("同じ意味の語が2つある", "概念7", 0, f"「{a}」と「{b}」")
            for a, b in SYNONYM_PAIRS if a in body and b in body]


def _without_code(text: str) -> str:
    """インラインコードを外す。**閉じの無いものは、そのまま残す。**"""
    return re.sub(r"`[^`\n]+`", " ", text)


JA_PUNCT = "。、）」・：；！？"


def _broken_emphasis(u: Unit) -> list[str]:
    """媒体の決め。閉じの ** が約物の直後にあると、CommonMark では閉じ記号にならない。"""
    parts, idx = [], 0
    while True:
        j = u.raw.find("**", idx)
        if j < 0:
            parts.append(u.raw[idx:])
            break
        parts.append(u.raw[idx:j])
        parts.append("\x00")
        idx = j + 2
    depth, hits = 0, []
    for k, v in enumerate(parts):
        if v != "\x00":
            continue
        depth ^= 1
        if depth:
            continue
        prev = parts[k - 1] if k else ""
        nxt = parts[k + 1] if k + 1 < len(parts) else ""
        if prev and prev[-1] in JA_PUNCT and nxt and nxt[0] not in JA_PUNCT and not nxt[0].isspace():
            hits.append(f"…{prev[-14:]}**{nxt[:10]}…")
    return hits


CHECKS: list[Check] = [
    Check("見出しの階層が飛んでいる", "概念4", _heading_skip, None,
          "見出し2の次に見出し4が来ると、間に何が在るはずだったかが分からない"),
    Check("文体が混ざっている", "概念6", _mixed_style, None,
          "敬体と常体を、同じ文書の中で混ぜない。体言止めは数えない"),
    Check("並んだ項目の語尾が揃っていない", "概念6", _unparallel_items, None,
          "同じ立場のものは、同じ形で書く。敬体かどうかだけを見る"),
    Check("文字で図や表を描いている", "概念2", _drawn_figure, KINDS,
          "箱の角と、横罫の連なりを見る。二倍ダッシュと木構造は図ではない"),
    Check("同じ意味の語が2つある", "概念7", _synonym, None,
          "--synonyms で対を与える。対を渡さなければ何も出ない"),
    Check("強調が描画されない", "媒体の決め", _broken_emphasis, RENDERED,
          "概念からは導けない。CommonMark の記法に拠る"),
]


def all_checks() -> list[Check]:
    return CHECKS


def inspect(path: str | Path) -> list[Finding]:
    raw = Path(path).read_text(encoding="utf-8")
    if EXEMPT_MARK in raw[:400]:
        return []
    units = split_units(raw)
    out: list[Finding] = []
    for c in CHECKS:
        if c.kinds is None:
            out += c.fn(units)
            continue
        for u in units:
            if u.kind in c.kinds:
                out += [Finding(c.name, c.basis, u.line, ex) for ex in c.fn(u)]
    seen, uniq = set(), []
    for f in out:
        key = (f.check, f.line, f.excerpt)
        if key not in seen:
            seen.add(key)
            uniq.append(f)
    return uniq


def print_checks() -> int:
    print(f"{'検査':30}{'拠って立つもの':16}{'当てる単位'}")
    for c in CHECKS:
        kinds = "文書全体" if c.kinds is None else " ・ ".join(sorted(c.kinds))
        print(f"{c.name:30}{c.basis:16}{kinds}")
        print(f"{'':46}{c.note}")
    return 0


def main(argv: list[str]) -> int:
    global SYNONYM_PAIRS
    args = list(argv)
    if "--synonyms" in args:
        i = args.index("--synonyms")
        p = Path(args[i + 1])
        SYNONYM_PAIRS = [tuple(x.split("\t")[:2]) for x in p.read_text(encoding="utf-8").splitlines()
                         if x.strip() and not x.startswith("#") and "\t" in x]
        del args[i:i + 2]
    if "--list" in args:
        return print_checks()
    if not args:
        print(__doc__)
        return 2
    n = 0
    for a in args:
        for f in inspect(a):
            print(f"× {Path(a).name}:{f.line} [{f.basis}] {f.check}：{f.excerpt}")
            n += 1
    print(f"\nゲート1　{'通った' if n == 0 else f'通っていない（{n} 件）'}"
          f"　／　対象 {len(args)} ファイル")
    if n:
        print("**0件にできるものだけを見ている。直してから出す。**")
    return 1 if n else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
