# -*- coding: utf-8 -*-
"""調査対象の原文を落とし、書こうとしているものが原文に在るかを確かめる。

  python3 source.py fetch  <URL> [--dir <保存先>]
  python3 source.py verify <原文> --as identifier|quote|text <当てるもの>...
                           [--from <1行1個で書いたファイル>] [--near <アンカー>] [--within <行数>]
  python3 source.py list   [--dir <保存先>]

**要約を経由して原典を読んではいけない。**
項目名は説明ではなく鍵である。1文字違えば、その鍵で引く実装は必ず空を返す。
空が返ることと、その事象が起きなかったことは、あとから区別できない。

**この道具が返すのは、真偽ではない。**
見つかった位置と、**読めなかった範囲**である。
0件は「無い」ではなく「**読めた範囲には**無い」としか言えない ──
読めなかったファイルを黙って落とせば、道具の側が推測で断定することになる。

実際に起きたこと（2026-09-05）──
公式ページを要約させて読み、要約したモデルが項目名を言い換えた。
`config_source` ・ `setup_type` ・ `expanded_prompt` ・ `user_input` の4つは
原文に1度も出てこない名前だった。それを根拠に「公式と実物が食い違う」と結論した。
原文で照合し直すと、15事象すべてが一致した。誤っていたのは読み方だった。
"""
from __future__ import annotations

import bisect
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field

DEFAULT_DIR = "sources"   # 置き場所は呼び出す側が決める。--dir で渡す
UA = "Mozilla/5.0"
MAX_BYTES = 4_000_000
SKIP = {".git", "node_modules", "target", "dist", "build", ".venv", "__pycache__"}

HOWS = ("identifier", "quote", "text")

# 語の文字。英数と、識別子で必ず語の内側になる区切り。
# それ以外の区切り（. / : ~ など）は、**当てる語自身が含んでいるときだけ**内側とする
#   ── 語は、自分の記法を自分の中に持っている。
ALWAYS_WORD = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-")


def word_chars(needle: str) -> set[str]:
    return ALWAYS_WORD | {c for c in needle if not c.isspace() and not c.isalnum()}


@dataclass
class Doc:
    """原文1つ。**連結しない。**位置をたどれる形のまま持つ。"""
    name: str
    text: str
    starts: list[int] = field(default_factory=list)

    def __post_init__(self):
        if not self.starts:
            self.starts = [0] + [m.end() for m in re.finditer("\n", self.text)]

    def line_of(self, offset: int) -> int:
        return bisect.bisect_right(self.starts, offset)

    def line_text(self, line: int) -> str:
        a = self.starts[line - 1]
        b = self.starts[line] if line < len(self.starts) else len(self.text)
        return self.text[a:b].rstrip("\n")


@dataclass
class Hit:
    doc: str
    line: int
    excerpt: str


@dataclass
class Result:
    needle: str
    how: str
    hits: list[Hit]


@dataclass
class Scan:
    """照合の結果。**見つかった位置と、読めなかった範囲を必ず一緒に持つ。**"""
    results: list[Result]
    unreadable: list[tuple[str, str]]
    docs_read: int

    @property
    def can_conclude_absent(self) -> bool:
        """0件を「無い」と結論してよいか。読めなかった範囲が在るなら、よくない。"""
        return not self.unreadable

    @property
    def missing(self) -> list[Result]:
        return [r for r in self.results if not r.hits]


# ── 原文を読む ────────────────────────────────────────────────

def read_docs(path: str, max_bytes: int = MAX_BYTES) -> tuple[list[Doc], list[tuple[str, str]]]:
    """原文を読む。**読めなかったものは、黙って落とさず返す。**"""
    docs: list[Doc] = []
    bad: list[tuple[str, str]] = []
    if os.path.isfile(path):
        files = [(path, os.path.basename(path))]
    elif os.path.isdir(path):
        files = []
        for root, dirs, names in os.walk(path):
            dirs[:] = [d for d in dirs if d not in SKIP]
            for n in sorted(names):
                if n.endswith(".meta.json"):
                    continue
                fp = os.path.join(root, n)
                files.append((fp, os.path.relpath(fp, path)))
    else:
        return [], [(path, "在らない")]
    for fp, name in files:
        try:
            size = os.path.getsize(fp)
            if size > max_bytes:
                bad.append((name, f"大きすぎる（{size:,} バイト）"))
                continue
            with open(fp, encoding="utf-8", errors="strict") as f:
                docs.append(Doc(name, f.read()))
        except UnicodeDecodeError:
            bad.append((name, "文字として読めない"))
        except OSError as e:
            bad.append((name, f"開けない（{e.strerror}）"))
    return docs, bad


# ── 当て方は3つ。呼ぶ側が名前で選ぶ ──────────────────────────────

def _find_identifier(text: str, needle: str) -> list[int]:
    """語として当てる。前後が語の文字なら、それは別の名前である。"""
    w = word_chars(needle)
    out, i = [], text.find(needle)
    while i >= 0:
        before = text[i - 1] if i > 0 else ""
        after = text[i + len(needle)] if i + len(needle) < len(text) else ""
        if before not in w and after not in w:
            out.append(i)
        i = text.find(needle, i + 1)
    return out


def cjk(ch: str) -> bool:
    """日本語の文字か。**日本語は、行の折り返しに空白を持たない。**"""
    o = ord(ch)
    return (0x3000 <= o <= 0x30FF or 0x3400 <= o <= 0x4DBF or 0x4E00 <= o <= 0x9FFF
            or 0xF900 <= o <= 0xFAFF or 0xFF00 <= o <= 0xFF60)


def _fold(text: str) -> tuple[str, list[int]]:
    """空白の連なりを畳む。畳んだ先から元の位置へ戻れるようにする。

    **畳んだ先は、前後の文字で決まる。**
    日本語どうしの間なら**無**に、そうでなければ**空白1つ**に畳む ──
    原文が行で折り返されているだけの箇所に、空白を作り出さないためである。
    英語は語の切れ目に空白を持つので、逆に空白1つが要る。
    """
    out: list[str] = []
    idx: list[int] = []
    i, n = 0, len(text)
    while i < n:
        ch = text[i]
        if not ch.isspace():
            out.append(ch)
            idx.append(i)
            i += 1
            continue
        j = i
        while j < n and text[j].isspace():
            j += 1
        before = out[-1] if out else ""
        after = text[j] if j < n else ""
        both_cjk = bool(before) and bool(after) and cjk(before) and cjk(after)
        if before and after and not both_cjk:
            out.append(" ")
            idx.append(i)
        i = j
    return "".join(out), idx


def _find_quote(text: str, needle: str) -> list[int]:
    """引用として当てる。**空白と改行の畳み方だけを揃え、語は1文字も変えない。**"""
    folded, idx = _fold(text)
    n = _fold(needle)[0].strip()
    if not n:
        return []
    out, i = [], folded.find(n)
    while i >= 0:
        out.append(idx[i])
        i = folded.find(n, i + 1)
    return out


def _find_text(text: str, needle: str) -> list[int]:
    """そのまま探す。**探索のための種類である。**照合した証しにはならない。"""
    out, i = [], text.find(needle)
    while i >= 0:
        out.append(i)
        i = text.find(needle, i + 1)
    return out


FINDERS = {"identifier": _find_identifier, "quote": _find_quote, "text": _find_text}


def scan(path: str, needles: list[str], how: str = "", near: str | None = None,
         within: int = 40, max_bytes: int = MAX_BYTES) -> Scan:
    """原文に当てる。**種類を渡さなければ止まる ── 道具の側で決めない。**"""
    if how not in FINDERS:
        raise ValueError(
            f"当て方を渡していない（--as {' / '.join(HOWS)}）。受け取ったもの: {how!r}")
    docs, bad = read_docs(path, max_bytes)
    find = FINDERS[how]
    results = []
    for n in needles:
        hits = []
        for d in docs:
            anchors = [d.line_of(o) for o in _find_text(d.text, near)] if near else None
            for off in find(d.text, n):
                line = d.line_of(off)
                if anchors is not None and not any(abs(line - a) <= within for a in anchors):
                    continue   # 名前が在ることと、その名前がそこで使われることは別である
                hits.append(Hit(d.name, line, d.line_text(line).strip()))
        results.append(Result(n, how, hits))
    return Scan(results, bad, len(docs))


# ── 原文を落とす ──────────────────────────────────────────────

def slug(url: str) -> str:
    s = re.sub(r"^https?://", "", url)
    s = re.sub(r"[^A-Za-z0-9._-]+", "_", s).strip("_")
    return s[:120]


def acceptable(code: str, ctype: str, size: int) -> bool:
    """落としたものを原文として受け取るか。**HTML も原文である。**

    タグを剥がしたり Markdown へ変換したりはしない ── 変換した時点で、
    「原文と1文字ずつ同じ」が言えなくなる。
    """
    return code == "200" and size > 0


def fetch(url: str, outdir: str) -> str | None:
    """原文を落とす。まず <URL>.md を試し、無ければ本体を取る。"""
    os.makedirs(outdir, exist_ok=True)
    tried = []
    cands = [url] if url.endswith((".md", ".txt", ".json")) else [url + ".md", url]
    for cand in cands:
        path = os.path.join(outdir, slug(cand))
        r = subprocess.run(
            ["curl", "-sSL", "-m", "60", "-A", UA, cand, "-o", path,
             "-w", "%{http_code} %{content_type}"],
            capture_output=True, text=True)
        code = (r.stdout or "").split(" ")[0]
        ctype = (r.stdout or " ").split(" ", 1)[-1].strip()
        size = os.path.getsize(path) if os.path.exists(path) else 0
        tried.append((cand, code, ctype, size))
        if acceptable(code, ctype, size):
            body = open(path, "rb").read()
            meta = {"url": cand, "requested": url,
                    "fetched_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
                    "sha256": hashlib.sha256(body).hexdigest(),
                    "bytes": len(body), "lines": body.count(b"\n") + 1,
                    "content_type": ctype}
            with open(path + ".meta.json", "w", encoding="utf-8") as f:
                json.dump(meta, f, ensure_ascii=False, indent=1)
            print(f"落とした: {path}")
            print(f"  {meta['bytes']:,} バイト ・ {meta['lines']:,} 行 ・ {ctype}")
            print(f"  sha256 {meta['sha256'][:16]}…")
            return path
        if os.path.exists(path):
            os.remove(path)   # 受け取らなかったものを残さない。一覧に出ないゴミになる
    print("落とせなかった。試したもの:")
    for c, code, ctype, size in tried:
        print(f"  {code}  {size:>9,}  {ctype[:30]:32}{c}")
    return None


def lst(outdir: str) -> int:
    if not os.path.isdir(outdir):
        print(f"まだ何も落としていない: {outdir}")
        return 0
    rows = []
    for f in sorted(os.listdir(outdir)):
        if f.endswith(".meta.json"):
            m = json.load(open(os.path.join(outdir, f), encoding="utf-8"))
            rows.append((m["fetched_at"][:10], m["lines"], m["url"]))
    print(f"{'落とした日':12}{'行':>8}  出どころ")
    for d, l, u in rows:
        print(f"{d:12}{l:>8,}  {u}")
    print(f"── {len(rows)} 件")
    return 0


# ── 報告 ────────────────────────────────────────────────────

def report(path: str, s: Scan, near: str | None, within: int) -> int:
    meta_path = path + ".meta.json"
    if os.path.exists(meta_path):
        m = json.load(open(meta_path, encoding="utf-8"))
        print(f"── {m['url']}")
        print(f"   {m['bytes']:,} バイト ・ {m['lines']:,} 行 ・ 落とした日 {m['fetched_at'][:10]}")
    else:
        print(f"── {path}  {s.docs_read:,} ファイル")
    if near:
        print(f"   アンカー「{near}」の ±{within} 行の内側だけを見る")
    how = s.results[0].how if s.results else ""
    print(f"   当て方: {how}" + ("　※ 探索のための種類。照合した証しにはならない" if how == "text" else ""))
    for r in s.results:
        if r.hits:
            where = " ・ ".join(f"{h.doc}:{h.line}" for h in r.hits[:2])
            print(f"  ok   {r.needle:34} {len(r.hits)} か所　{where}")
        else:
            print(f"  ×    {r.needle:34} 読めた範囲には無い")
    print(f"── 当てた {len(s.results)} ／ 読めた範囲に無い {len(s.missing)}"
          f" ／ 読めなかった {len(s.unreadable)}")
    if s.unreadable:
        print("\n**読めなかった範囲が在る。0件を「原文に無い」と結論してはいけない。**")
        for name, why in s.unreadable:
            print(f"  読めず  {name}　{why}")
    if s.missing:
        print("\n**読めた範囲に無いものを、原典の名前として書いてはいけない。**")
        print("見つからない原因は4つ──①名前を言い換えた ②別のページに在る "
              "③読めなかった範囲に在る ④本当に無い。")
        print("①なら直す。②なら該当ページを落として照合し直す。③なら読めるようにする。"
              "④なら「無い」と書く。**推測で埋めない。**")
    return 1 if (s.missing or s.unreadable) else 0


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__)
        return 2
    cmd, args = argv[0], list(argv[1:])

    def take(flag, cast=str, default=None):
        if flag in args:
            i = args.index(flag)
            v = cast(args[i + 1])
            del args[i:i + 2]
            return v
        return default

    outdir = take("--dir", default=DEFAULT_DIR)
    if cmd == "fetch":
        return 0 if fetch(args[0], outdir) else 1
    if cmd == "list":
        return lst(outdir)
    if cmd == "verify":
        how = take("--as", default="")
        near = take("--near")
        within = take("--within", int, 40)
        src = take("--from")
        path = args[0]
        needles = ([x.strip() for x in open(src, encoding="utf-8") if x.strip()] if src
                   else [a for a in args[1:] if not a.startswith("--")])
        if not needles:
            print("当てるものを渡す")
            return 2
        try:
            s = scan(path, needles, how=how, near=near, within=within)
        except ValueError as e:
            print(f"× {e}")
            print("  identifier ── 語として当てる（前後が語の文字なら別の名前）")
            print("  quote      ── 引用として当てる（空白の畳み方だけ揃え、語は変えない）")
            print("  text       ── そのまま探す（探索用。照合した証しにはならない）")
            return 2
        return report(path, s, near, within)
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
