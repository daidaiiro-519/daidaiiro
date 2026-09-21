# SPDX-License-Identifier: MIT
"""ソースコードを、言語ごとのコードブロックとして組む。

**印は行の単位で付ける。** 語の単位で付けると、色付けが挿入した要素と交差して
どちらかが壊れる ── コードの変更は行の単位で読むものなので、行で足りる。

**色付けは近似である。** 構文解析を実施せず、行ごとに正規表現を適用する ──
文字列の中の予約語のような事例は取り違える。読解の補助であり、判定の根拠にしない。
"""
from __future__ import annotations

import difflib
import html
import re

# 拡張子と言語の対応。**ここに無い拡張子は、コードとして扱わない。**
LANGS: dict[str, str] = {
    ".py": "python", ".pyi": "python",
    ".js": "js", ".mjs": "js", ".cjs": "js", ".ts": "js", ".tsx": "js", ".jsx": "js",
    ".json": "json",
    ".go": "go",
    ".rs": "rust",
    ".rb": "ruby",
    ".sh": "shell", ".bash": "shell", ".zsh": "shell",
    ".sql": "sql",
    ".yaml": "yaml", ".yml": "yaml",
    ".toml": "toml",
    ".css": "css",
    ".svg": "xml", ".xml": "xml",
}

KEYWORDS: dict[str, str] = {
    "python": "False None True and as assert async await break class continue def del elif "
              "else except finally for from global if import in is lambda nonlocal not or "
              "pass raise return try while with yield match case",
    "js": "as async await break case catch class const continue debugger default delete do "
          "else export extends false finally for from function if import in instanceof let "
          "new null of return static super switch this throw true try typeof var void while "
          "with yield interface type enum implements readonly",
    "go": "break case chan const continue default defer else fallthrough for func go goto if "
          "import interface map package range return select struct switch type var nil true false",
    "rust": "as async await break const continue crate dyn else enum extern false fn for if "
            "impl in let loop match mod move mut pub ref return self static struct super "
            "trait true type unsafe use where while",
    "ruby": "def end class module if elsif else unless while until for in do then begin rescue "
            "ensure yield return self nil true false and or not require",
    "shell": "if then elif else fi for while until do done case esac function return local "
             "export readonly set unset echo exit source",
    "sql": "select from where group by having order limit offset insert into values update "
           "set delete create table drop alter index join left right inner outer on as and "
           "or not null distinct union all",
    "json": "true false null",
    "yaml": "true false null yes no",
    "toml": "true false",
    "css": "",
    "xml": "",
}

# 行の中を色付けする規則。順に適用し、先に一致したものが優先する
LINE = {
    "python": [(r"#.*$", "c"), (r"(?:'''|\"\"\").*?(?:'''|\"\"\")|'[^']*'|\"[^\"]*\"", "s")],
    "js": [(r"//.*$", "c"), (r"/\*.*?\*/", "c"), (r"'[^']*'|\"[^\"]*\"|`[^`]*`", "s")],
    "go": [(r"//.*$", "c"), (r"'[^']*'|\"[^\"]*\"|`[^`]*`", "s")],
    "rust": [(r"//.*$", "c"), (r"'[^']*'|\"[^\"]*\"", "s")],
    "ruby": [(r"#.*$", "c"), (r"'[^']*'|\"[^\"]*\"", "s")],
    "shell": [(r"#.*$", "c"), (r"'[^']*'|\"[^\"]*\"", "s")],
    "sql": [(r"--.*$", "c"), (r"'[^']*'", "s")],
    "json": [(r"\"[^\"]*\"", "s")],
    "yaml": [(r"#.*$", "c"), (r"'[^']*'|\"[^\"]*\"", "s")],
    "toml": [(r"#.*$", "c"), (r"'[^']*'|\"[^\"]*\"", "s")],
    "css": [(r"/\*.*?\*/", "c"), (r"'[^']*'|\"[^\"]*\"", "s")],
    "xml": [(r"&lt;!--.*?--&gt;", "c"), (r"\"[^\"]*\"", "s")],
}

NUM = re.compile(r"\b\d[\d_]*(?:\.\d+)?\b")


def is_code(ext: str) -> bool:
    return ext.lower() in LANGS


def _tokens(line: str, lang: str) -> str:
    """1行を色付けする。**すでに逃がした文字列に対して適用する。**"""
    spans: list[tuple[int, int, str]] = []
    for pat, kind in LINE.get(lang, []):
        for m in re.finditer(pat, line):
            if not any(a <= m.start() < b for a, b, _ in spans):
                spans.append((m.start(), m.end(), kind))
    words = KEYWORDS.get(lang, "").split()
    if words:
        for m in re.finditer(r"\b(?:" + "|".join(map(re.escape, words)) + r")\b", line):
            if not any(a <= m.start() < b for a, b, _ in spans):
                spans.append((m.start(), m.end(), "k"))
    for m in NUM.finditer(line):
        if not any(a <= m.start() < b for a, b, _ in spans):
            spans.append((m.start(), m.end(), "n"))
    out, at = [], 0
    for a, b, kind in sorted(spans):
        if a < at:
            continue
        out.append(line[at:a])
        out.append(f'<span class="t-{kind}">{line[a:b]}</span>')
        at = b
    out.append(line[at:])
    return "".join(out)


def render_code(src: str, ext: str, marks: list[dict]) -> str:
    """コードを、行番号を備えたコードブロックへ組む。印は行の単位で付く。"""
    lang = LANGS[ext.lower()]
    lines = src.split("\n")
    # 印を行へ割り当てる。**同じ行に2件は付けない** ── 入れ子になるためである
    at: dict[int, dict] = {}
    for c in marks:
        needle = c.get("find", "")
        if not needle:
            continue
        for i, raw in enumerate(lines):
            if needle in raw and i not in at:
                at[i] = c
                break
    rows = []
    for i, raw in enumerate(lines):
        body = _tokens(html.escape(raw), lang) or "&nbsp;"
        c = at.get(i)
        if c:
            body = (f'<mark class="chg" tabindex="0" role="button" aria-expanded="false"'
                    f' data-b="{html.escape(c.get("before", ""), quote=True)}"'
                    f' data-w="{html.escape(c.get("why", ""), quote=True)}">{body}</mark>')
        rows.append(f'<tr id="L{i + 1}"><td class="ln">{i + 1}</td><td class="cd">{body}</td></tr>')
    return (f'<div class="code" data-lang="{lang}"><table><tbody>'
            + "".join(rows) + "</tbody></table></div>")


def hunks(old: list[str], new: list[str], ctx: int = 3) -> list[list[tuple]]:
    """統合差分のまとまりを組む。1件が (旧の行番号, 新の行番号, 印, 本文) の並びである。

    印は " "（変化なし）・ "-"（旧）・ "+"（新）である。
    **全行を先に組み、変化した箇所の周りだけを切り出す** ── 途中で切ると文脈が落ちる。
    """
    sm = difflib.SequenceMatcher(None, old, new, autojunk=False)
    rows: list[tuple] = []
    for kind, a1, a2, b1, b2 in sm.get_opcodes():
        if kind == "equal":
            for k in range(a2 - a1):
                rows.append((a1 + k + 1, b1 + k + 1, " ", old[a1 + k]))
            continue
        if kind in ("replace", "delete"):
            for k in range(a1, a2):
                rows.append((k + 1, None, "-", old[k]))
        if kind in ("replace", "insert"):
            for k in range(b1, b2):
                rows.append((None, k + 1, "+", new[k]))

    moved = [i for i, r in enumerate(rows) if r[2] != " "]
    if not moved:
        return []
    groups: list[list[int]] = [[moved[0]]]
    for i in moved[1:]:
        if i - groups[-1][-1] <= ctx * 2:
            groups[-1].append(i)
        else:
            groups.append([i])
    out = []
    for g in groups:
        a = max(0, g[0] - ctx)
        b = min(len(rows), g[-1] + ctx + 1)
        out.append(rows[a:b])
    return out


def render_diff(old_src: str, new_src: str, ext: str, marks: list[dict]) -> tuple[str, int, int]:
    """Git の差分の形へ組み、まとまりごとに理由を添える。

    **返すのは (HTML, まとまりの数, 理由が付いたまとまりの数) である。**
    Git は差分を出すが、なぜ変えたかを出さない ── そこを埋めるのがこの道具である。
    """
    lang = LANGS[ext.lower()]
    old = old_src.split("\n")
    new = new_src.split("\n")
    hs = hunks(old, new)

    # 理由を、まとまりへ割り当てる。**変化した行に一致するものを先に見る**
    used: set[int] = set()
    at: dict[int, dict] = {}
    for c in marks:
        needle = c.get("find", "")
        if not needle:
            continue
        for i, h in enumerate(hs):
            if i in used:
                continue
            if any(m == "+" and needle in line for _, _, m, line in h):
                at[i] = c
                used.add(i)
                break
        else:
            for i, h in enumerate(hs):
                if i not in used and any(needle in line for _, _, _, line in h):
                    at[i] = c
                    used.add(i)
                    break

    rows = []
    for i, h in enumerate(hs):
        a = next((x for x, _, _, _ in h if x), None)
        b = next((y for _, y, _, _ in h if y), None)
        c = at.get(i)
        head = (f'<tr class="hh"><td class="ln"></td><td class="ln"></td>'
                f'<td class="mk"></td><td class="cd">'
                f'@@ 旧 {a or "-"} ／ 新 {b or "-"} @@'
                + ("" if c else ' <span class="nowhy">理由が付いていない</span>')
                + "</td></tr>")
        rows.append(head)
        for x, y, mk, line in h:
            body = _tokens(html.escape(line), lang) or "&nbsp;"
            if c and mk == "+" and c.get("find", "") in line and "chg" not in head:
                body = (f'<mark class="chg" tabindex="0" role="button" aria-expanded="false"'
                        f' data-b="{html.escape(c.get("before", "── 上の - の行が変更前である"), quote=True)}"'
                        f' data-w="{html.escape(c.get("why", ""), quote=True)}">{body}</mark>')
                head = head + "chg"
            cls = {"-": "d", "+": "a", " ": ""}[mk]
            rows.append(f'<tr class="r{cls}"><td class="ln">{x or ""}</td>'
                        f'<td class="ln">{y or ""}</td>'
                        f'<td class="mk">{html.escape(mk)}</td>'
                        f'<td class="cd">{body}</td></tr>')
    return (f'<div class="code diff" data-lang="{lang}"><table><tbody>'
            + "".join(rows) + "</tbody></table></div>", len(hs), len(at))


CSS = """
.code{overflow-x:auto;background:var(--paper);border:1px solid var(--line);border-radius:.3rem;
font-size:.82rem;line-height:1.7}
.code table{border-collapse:collapse;width:100%;margin:0;font-size:inherit}
.code td{border:0;padding:0;vertical-align:top;white-space:pre;
font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace}
.code td.ln{width:3.2rem;min-width:3.2rem;text-align:right;padding:0 .7rem 0 .5rem;
color:var(--muted);background:var(--panel);user-select:none;border-right:1px solid var(--line);
position:sticky;left:0}
.code td.cd{padding:0 .8rem}
.code tr:has(mark.chg) td.ln{background:var(--chipbg);color:var(--move);font-weight:700}
.code .t-c{color:var(--muted);font-style:italic}
.code .t-s{color:var(--key)}
.code .t-k{color:var(--move);font-weight:700}
.code .t-n{color:var(--gone)}
.code mark.chg{display:inline-block;width:100%}
.code td.popcell{white-space:normal;padding:.2rem .8rem .5rem 3.9rem}
.code .pop{white-space:normal;font-family:"Hiragino Kaku Gothic ProN","Yu Gothic",system-ui,sans-serif;max-width:62rem}
.code tr.poprow td{background:var(--paper)}
.lane.code-lane b{color:var(--move)}\n.code.diff td.mk{width:1.4rem;min-width:1.4rem;text-align:center;color:var(--muted)}\n.code.diff tr.ra td{background:rgba(47,111,94,.10)}\n.code.diff tr.ra td.mk{color:var(--key);font-weight:700}\n.code.diff tr.rd td{background:rgba(138,58,58,.10)}\n.code.diff tr.rd td.mk{color:var(--gone);font-weight:700}\n.code.diff tr.hh td{background:var(--panel);color:var(--muted);font-size:.78rem;\npadding:.25rem .8rem;border-top:1px solid var(--line);border-bottom:1px solid var(--line)}\n.code.diff tr.hh td.cd{white-space:normal}\n.code.diff .nowhy{color:var(--gone);font-weight:700}\n.code.diff td.popcell{padding-left:5.6rem}
"""
