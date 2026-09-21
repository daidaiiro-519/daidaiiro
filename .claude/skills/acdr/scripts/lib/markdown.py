#!/usr/bin/env python3
"""マークダウンを描画し、変更箇所に印を付ける。

**部品である。入口を保持しない** ── 呼ぶのは `tabs.py` で、
外から起動するのは `cli.py render` である。

marks.json の形。find は描画後のHTMLに現れる文字列。
  [{"find": "...", "before": "変更前の原文", "why": "なぜ変えたか"}, ...]

一致しなかった find は標準エラーに出す。黙って除外しない。

**HTML の形は、ここが持たない** ── `references/acdr.template.html` が持つ。
"""
import re, html, io, sys, json

from .template import part as _t

def inline(t):
    t = html.escape(t)
    t = re.sub(r'`([^`]+)`', _t("md-code", body=r"\1"), t)
    t = re.sub(r'\*\*(.+?)\*\*', _t("md-strong", body=r"\1"), t)
    return t.replace('&lt;br&gt;', '<br>').replace('&lt;br/&gt;', '<br>')

def render(md):
    # HTML のコメントは、描画すると文字として出る。除去する
    md = re.sub(r'<!--.*?-->', '', md, flags=re.S)
    out, L, i = [], md.split('\n'), 0
    while i < len(L):
        l = L[i]
        if l.startswith('```'):
            lang = l[3:].strip(); j = i + 1; b = []
            while j < len(L) and not L[j].startswith('```'):
                b.append(L[j]); j += 1
            cls = 'mermaid' if lang == 'mermaid' else 'code'
            out.append(_t("md-pre", cls=cls, body=html.escape('\n'.join(b))))
            i = j + 1; continue
        m = re.match(r'^(#{1,4})\s+(.*)', l)
        if m:
            n = len(m.group(1))
            out.append(_t("md-heading", level=n, body=inline(m.group(2)))); i += 1; continue
        if l.startswith('|'):
            rows = []
            while i < len(L) and L[i].startswith('|'):
                rows.append(L[i]); i += 1
            cells = [[c.strip() for c in r.strip().strip('|').split('|')] for r in rows]
            def sep(row):
                return all(re.match(r'^:?-{2,}:?$', c.strip()) for c in row if c.strip()) \
                       and any(c.strip() for c in row)
            body = [c for c in cells if not sep(c)]
            rows = []
            # 見出しが全部空なら、見出しの行を出さない。
            # 空の <th> を並べると、中身の無い帯が表の上に1本出る
            if body and not any(c.strip() for c in body[0]):
                body = body[1:]
            elif len(body) > 1:
                rows.append(_t("md-tr", cells=''.join(
                    _t("md-th", cell=inline(c)) for c in body[0])))
                body = body[1:]
            for r in body:
                rows.append(_t("md-tr", cells=''.join(
                    _t("md-td", cell=inline(c)) for c in r)))
            out.append(_t("md-table", rows=''.join(rows))); continue
        if l.startswith('>'):
            b = []
            while i < len(L) and L[i].startswith('>'):
                b.append(L[i][1:].strip()); i += 1
            out.append(_t("md-quote", body=inline('<br>'.join(b)))); continue
        if l.strip() == '---':
            out.append(_t("md-hr")); i += 1; continue
        if not l.strip():
            i += 1; continue
        b = []
        while i < len(L) and L[i].strip() and not L[i].startswith(('|', '>', '#', '```')) \
              and L[i].strip() != '---':
            b.append(L[i]); i += 1
        out.append(_t("para", body=inline('<br>'.join(b))))
    return '\n'.join(out)

def outside_pre(h):
    """<pre> の外だけを、印を付けてよい範囲として返す。

    コードと図の中に印を差し込むと、その中身が壊れる。
    mermaid は差し込んだ時点で描画されなくなる。
    """
    spans, i = [], 0
    while True:
        a = h.find("<pre", i)
        if a < 0:
            spans.append((i, len(h))); break
        spans.append((i, a))
        b = h.find("</pre>", a)
        i = len(h) if b < 0 else b + 6
    return spans


def mark(h, marks):
    """変更後のHTMLに、印を差し込む。

    **位置は、差し込む前のHTMLに対して先に全部決める。**
    差し込んだ `data-b` ・ `data-w` はHTMLの一部になるので、
    差し込みながら探すと、**次の印が前の印の理由文の中へ入る**。
    実際にそれで属性の中へ `<mark>` が入り、面が壊れた。
    """
    spans = outside_pre(h)
    plan = []
    for c in marks:
        f = c["find"]
        if f not in h:
            print("  一致せず: " + f[:60], file=sys.stderr)
            continue
        at = next((h.find(f, a, b) for a, b in spans if h.find(f, a, b) >= 0), None)
        if at is None:
            print("  コードか図の中にしかない: " + f[:60], file=sys.stderr)
            continue
        if not c.get("why"):
            print("  なぜが無い: " + f[:60], file=sys.stderr)
        plan.append((at, len(f), c))

    # 重なりを除外する。同じ場所へ2つ差し込むと、片方が他方の中へ入る
    plan.sort(key=lambda x: (x[0], -x[1]))
    kept, end = [], -1
    for at, ln, c in plan:
        if at < end:
            print("  位置が重なる: " + c["find"][:60], file=sys.stderr)
            continue
        kept.append((at, ln, c))
        end = at + ln

    # 後ろから差し込む。前の位置がずれない
    for at, ln, c in reversed(kept):
        a = html.escape(c.get("before", ""), quote=True)
        w = html.escape(c.get("why", ""), quote=True)
        f = c["find"]
        h = h[:at] + _t("mark", before=a, why=w, body=f) + h[at + ln:]
    print(f"  {len(kept)}/{len(marks)} 件に印を付けた", file=sys.stderr)
    return h
