# -*- coding: utf-8 -*-
"""変更した文書が複数あるとき、タブ1枚にまとめて提示する。

  python3 tabs.py <spec.json> <出力.html>

spec.json の形。
  {
    "title": "03章 整合",
    "lede":  "<b>結論を1文。</b>…",
    "intro": "<h2>1. 何を動かしたか</h2>…",   任意。タブの上に置く
    "docs": [
      {"key":"m", "tab":"03-4 測定", "file":"…/03-4-measurement.md",
       "marks":[{"find":"…","before":"…","why":"…"}]},
      {"key":"d", "tab":"設計ノート", "file":"…/design.html", "marks":[…]}
    ]
  }

**タブの中身は、拡張子で決まる。**

  .md    render.py で描画する
  .html  そのままの見た目で置く。iframe に流し込み、届かなければ Shadow DOM へ落とす

**どちらの入れ方でも、印の付け方と開閉は同じである。**
印は `mark.chg` で、押すと変更前と理由が開く。

確かめるのは8つ。どれも、以前に実際にやらかしたものである。
  1 <script> が入っているか            —— 繋ぎ忘れて、押しても何も起きなかった
  2 印ごとに data-b と data-w が在るか
  3 .pop[hidden]{display:none} が在るか —— 無いと開いたまま閉じない
  4 表の中に <div> を置いていないか     —— <tbody> 直下の <div> は表の外へ運ばれる
  5 印の数が、渡した数と合うか
  6 タブとパネルの数が合うか
  7 波括弧が閉じているか               —— 3つ未閉鎖でスタイルが全滅した
  8 HTML を入れた面に、雛形が在るか
"""
import html
import io
import json
import re
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from render import render, mark, outside_pre  # noqa: E402

# ── HTML を、そのままの見た目で置くために取り出す ─────────────────

def strip_document(src):
    """<head> の <style> と <body> の中身だけを取り出す。

    `<html>` と `<body>` を落とすのは、入れ子にすると
    親の文書構造が壊れるためである。`<style>` は落とさない。
    落とすと、見た目がそのままでなくなる。
    """
    styles = re.findall(r"<style\b[^>]*>.*?</style>", src, re.S | re.I)
    m = re.search(r"<body\b[^>]*>(.*?)</body>", src, re.S | re.I)
    if m:
        body = m.group(1)
    else:
        # <body> を持たない断片は、<head> の中身を外して本文とみなす
        body = re.sub(r"<head\b[^>]*>.*?</head>", "", src, flags=re.S | re.I)
        body = re.sub(r"</?(?:html|body)\b[^>]*>", "", body, flags=re.I)
        for s in styles:
            body = body.replace(s, "")
    return "".join(styles) + body


def scope_for_shadow(chunk):
    """Shadow DOM に入れるとき、:root と body を :host へ寄せる。

    Shadow の中に `:root` は無い。寄せないと、そこで定めた
    カスタムプロパティが1つも効かず、色が全部落ちる。
    """
    chunk = re.sub(r"(?<![\w-]):root(?![\w-])", ":host", chunk)
    chunk = re.sub(r"(?m)^(\s*)body(\s*[,{])", r"\1:host\2", chunk)
    return chunk


# ── 印を付ける ────────────────────────────────────────────────

def mark_html(src, marks):
    """描画済みの HTML に印を付ける。render.mark と同じ規則である。"""
    return mark(src, marks)


# ── 組む ──────────────────────────────────────────────────────

CSS = """
:root{--paper:#fbfaf7;--ink:#1c1a17;--muted:#6b665d;--panel:#f2efe8;--chipbg:#e7e3d9;
--move:#9a5b2c;--key:#2f6f5e;--line:#d8d3c7;--gone:#8a3a3a}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){
--paper:#16150f;--ink:#ece7dc;--muted:#98918a;--panel:#211f18;--chipbg:#2b281f;
--move:#d99b62;--key:#6fbfa4;--line:#38342a;--gone:#d97b7b}}
:root[data-theme="dark"]{--paper:#16150f;--ink:#ece7dc;--muted:#98918a;--panel:#211f18;
--chipbg:#2b281f;--move:#d99b62;--key:#6fbfa4;--line:#38342a;--gone:#d97b7b}
*{box-sizing:border-box}
body{background:var(--paper);color:var(--ink);margin:0;
font-family:"Hiragino Kaku Gothic ProN","Yu Gothic",system-ui,sans-serif;line-height:1.85}
.wrap{max-width:92rem;margin:0 auto;padding:2rem 1.2rem 5rem}
h1{font-size:1.5rem;margin:0 0 .3rem;letter-spacing:.02em}
.lede{font-size:1.06rem;border-left:4px solid var(--key);padding:.55rem 0 .55rem .9rem;
margin:1rem 0 2rem;background:var(--panel);max-width:68rem}
h2{font-size:1.12rem;margin:2.4rem 0 .8rem;padding-bottom:.3rem;border-bottom:1px solid var(--line)}
h3{font-size:1rem;margin:1.6rem 0 .5rem;color:var(--key)}
table{border-collapse:collapse;width:100%;margin:.9rem 0;font-size:.93rem}
th,td{border:1px solid var(--line);padding:.42rem .6rem;text-align:left;vertical-align:top}
th{background:var(--chipbg);font-weight:700}
code{background:var(--chipbg);padding:.08rem .3rem;border-radius:.2rem;font-size:.88em}
pre{background:var(--chipbg);padding:.7rem .9rem;border-radius:.3rem;overflow-x:auto;font-size:.84rem}
.scroll{overflow-x:auto}
.gone{color:var(--gone)}
#bar{position:sticky;top:0;z-index:9;background:var(--panel);border-bottom:2px solid var(--move);
padding:.5rem .9rem;display:flex;gap:.7rem;align-items:center;flex-wrap:wrap;font-size:.9rem;
margin:0 -1.2rem 1rem}
#bar button{font:inherit;padding:.2rem .7rem;border-radius:.25rem;border:1px solid var(--move);
background:transparent;color:var(--move);cursor:pointer}
#bar button:hover{background:var(--move);color:var(--paper)}
#tabs{display:flex;gap:.4rem;flex-wrap:wrap;margin:1rem 0 0;border-bottom:2px solid var(--line)}
.tab{font:inherit;font-size:.93rem;padding:.45rem .95rem;border:1px solid var(--line);
border-bottom:0;border-radius:.3rem .3rem 0 0;background:var(--panel);color:var(--muted);cursor:pointer}
.tab[aria-selected="true"]{background:var(--paper);color:var(--ink);font-weight:700;
box-shadow:0 2px 0 0 var(--paper)}
.tab .n{font-size:.78rem;color:var(--move);margin-left:.35rem;font-weight:700}
.pane{border:1px solid var(--line);border-top:0;background:var(--paper)}
.pane.md{padding:1.2rem 1.8rem}
.pane.md h1{font-size:1.25rem;margin-top:0}
.pane.html{padding:0}
.pane.html iframe{display:block;width:100%;border:0;background:var(--paper)}
.lane{font-size:.8rem;color:var(--muted);padding:.35rem .9rem;border-bottom:1px dashed var(--line);
background:var(--panel);display:flex;gap:.6rem;align-items:center;flex-wrap:wrap}
.lane .how{margin-left:auto;font-style:italic}
.idx{border-bottom:1px solid var(--line);background:var(--panel);padding:0 .9rem .5rem}
.idx summary{cursor:pointer;font-size:.88rem;padding:.45rem 0;font-weight:700;color:var(--move)}
.idx ol{margin:0 0 .3rem;padding-left:1.4rem;font-size:.88rem}
.idx li{margin:.35rem 0}
.idx button.go{font:inherit;font-size:.82rem;padding:.05rem .5rem;margin-left:.4rem;
border:1px solid var(--move);border-radius:.2rem;background:transparent;color:var(--move);cursor:pointer}
.idx button.go:hover{background:var(--move);color:var(--paper)}
.idx .b{color:var(--muted);font-size:.84rem}
"""

# 印そのものの見た目。iframe と Shadow の中へも、同じものを流し込む
MARKCSS = """
mark.chg{background:transparent;color:inherit;border-bottom:2px dashed #9a5b2c;
cursor:pointer;padding:0 .1rem}
mark.chg::after{content:"変";font-size:.62rem;vertical-align:super;color:#9a5b2c;
font-weight:700;margin-left:.12rem}
mark.chg:focus-visible{outline:2px solid #2f6f5e;outline-offset:2px}
.pop{display:block;margin:.6rem 0 .9rem;padding:.7rem .9rem;border:1px solid #9a5b2c;
border-radius:.3rem;background:rgba(154,91,44,.07);font-size:.9rem;line-height:1.7;
font-family:"Hiragino Kaku Gothic ProN","Yu Gothic",system-ui,sans-serif}
.pop[hidden]{display:none!important}
.pop>b{color:#9a5b2c;display:block;margin:.5rem 0 .15rem}
.pop>b:first-child{margin-top:0}
.pop pre{white-space:pre-wrap;word-break:break-word;font-size:.82rem;margin:0;
color:inherit;opacity:.8;background:rgba(128,128,128,.12);padding:.5rem .6rem;border-radius:.2rem}
"""

JS = r"""
(function(){
  var MARKCSS = document.getElementById("markcss").textContent;

  /* 印を1つ、押せるようにする。iframe の中でも Shadow の中でも同じ手が効く */
  function wire(root, doc){
    root.querySelectorAll("mark.chg[data-w]").forEach(function(m){
      var p = doc.createElement("div");
      p.className = "pop"; p.hidden = true;
      var b1 = doc.createElement("b"); b1.textContent = "変更前";
      var pre = doc.createElement("pre"); pre.textContent = m.dataset.b;
      var b2 = doc.createElement("b"); b2.textContent = "なぜ";
      var w = doc.createElement("div"); w.textContent = m.dataset.w;
      p.appendChild(b1); p.appendChild(pre); p.appendChild(b2); p.appendChild(w);
      var host = m.closest("td") || m.closest("li") || m.parentNode;
      if (host.nextSibling) host.parentNode.insertBefore(p, host.nextSibling);
      else host.parentNode.appendChild(p);
      function tg(){
        var opening = p.hidden;
        p.hidden = !opening;
        m.setAttribute("aria-expanded", opening ? "true" : "false");
        resizeAll();
      }
      m.addEventListener("click", tg);
      m.addEventListener("keydown", function(e){
        if (e.key === "Enter" || e.key === " ") { e.preventDefault(); tg(); }
      });
    });
  }

  var frames = [];
  function resizeAll(){
    frames.forEach(function(f){
      try {
        var d = f.contentDocument;
        if (d && d.body) f.style.height = (d.documentElement.scrollHeight + 8) + "px";
      } catch(e){}
    });
  }

  /* HTML の面を立てる。iframe が空のままなら Shadow DOM へ落とす */
  document.querySelectorAll(".pane.html").forEach(function(pane){
    var tpl = pane.querySelector("template");
    var src = tpl.innerHTML;
    var fr = document.createElement("iframe");
    fr.setAttribute("title", pane.dataset.tab || "");
    fr.srcdoc = '<!doctype html><meta charset="utf-8">'
              + '<style>' + MARKCSS + '</style>' + src;
    pane.appendChild(fr);
    frames.push(fr);
    var done = false;
    fr.addEventListener("load", function(){
      var d = fr.contentDocument;
      if (!d || !d.body || !d.body.firstChild) return;
      done = true;
      wire(d, d);
      resizeAll();
      /* 中の高さは、字が載ってから決まる。変わるたびに測り直す */
      if (window.ResizeObserver) {
        new ResizeObserver(resizeAll).observe(d.documentElement);
      }
      /* 中のタブを押しても測り直す。中の作りに依らず効く */
      d.addEventListener("click", function(){ setTimeout(resizeAll, 0); });
      [80, 300, 900].forEach(function(t){ setTimeout(resizeAll, t); });
      pane.querySelector(".how").textContent = "iframe で置いた";
    });
    setTimeout(function(){
      if (done) return;
      /* 載ったのに load を取り逃していないかを、もう一度見る */
      try {
        var d2 = fr.contentDocument;
        if (d2 && d2.body && d2.body.firstChild) {
          done = true; wire(d2, d2); resizeAll();
          if (window.ResizeObserver) new ResizeObserver(resizeAll).observe(d2.documentElement);
          d2.addEventListener("click", function(){ setTimeout(resizeAll, 0); });
          pane.querySelector(".how").textContent = "iframe で置いた";
          return;
        }
      } catch(e){}
      /* 届かなかった。Shadow DOM へ落とす */
      fr.remove();
      frames = frames.filter(function(x){ return x !== fr; });
      var holder = document.createElement("div");
      pane.appendChild(holder);
      var sh = holder.attachShadow({mode:"open"});
      var st = document.createElement("style");
      st.textContent = MARKCSS + "\n" + (tpl.dataset.shadowcss || "");
      sh.appendChild(st);
      var box = document.createElement("div");
      box.innerHTML = src;
      sh.appendChild(box);
      wire(sh, document);
      pane.querySelector(".how").textContent = "Shadow DOM で置いた";
    }, 2500);
  });

  /* 印を、見える状態にする。

     **内側のタブの作り方を、こちらは知らない。**
     設計ノートは <input type="radio"> と :checked ~ #panel で切り替えていた。
     JS で切り替える作りもある。**どちらでも効くように、
     切り替えを1つずつ当てて、印が出たところで止める。** */
  function reveal(m){
    if (m.offsetParent) return true;
    var n = m;
    while (n && n.nodeType === 1) { if (n.hidden) n.hidden = false; n = n.parentNode; }
    /* **印が住んでいる根から探す。**
       Shadow の中の印に対して document を探すと、切り替えが1つも見つからない */
    var scope = m.getRootNode ? m.getRootNode() : m.ownerDocument;
    for (var e = m.parentNode; e && e.nodeType === 1; e = e.parentNode) {
      if (e.tagName === "DETAILS") e.open = true;
    }
    if (m.offsetParent) return true;
    var ins = scope.querySelectorAll('input[type="radio"],input[type="checkbox"]');
    for (var i = 0; i < ins.length; i++) {
      var was = ins[i].checked;
      ins[i].checked = true;
      if (m.offsetParent) return true;
      ins[i].checked = was;
    }
    return !!m.offsetParent;
  }

  /* 一覧の「その場所へ」。印を開いて、そこまで運ぶ */
  document.querySelectorAll(".idx button.go").forEach(function(b){
    b.onclick = function(){
      var pane = b.closest(".pane");
      var i = +b.dataset.i;
      var fr = pane.querySelector("iframe");
      var root = fr ? fr.contentDocument : null;
      if (!root) {
        var h = Array.prototype.find.call(pane.querySelectorAll("div"),
          function(x){ return x.shadowRoot; });
        root = h ? h.shadowRoot : pane;
      }
      var m = root.querySelectorAll("mark.chg")[i];
      if (!m) return;
      var shown = reveal(m);
      if (m.getAttribute("aria-expanded") !== "true") m.click();
      b.textContent = shown ? "その場所へ" : "隠れたまま";
      setTimeout(function(){
        var y = 0, e = m;
        while (e && e.offsetParent) { y += e.offsetTop; e = e.offsetParent; }
        if (fr) window.scrollTo({top: fr.getBoundingClientRect().top + window.scrollY + y - 60,
                                 behavior: "smooth"});
        else m.scrollIntoView({block: "center", behavior: "smooth"});
      }, 60);
    };
  });

  /* マークダウンの面 */
  document.querySelectorAll(".pane.md").forEach(function(p){ wire(p, document); });

  function allPops(){
    var out = Array.prototype.slice.call(document.querySelectorAll(".pop"));
    frames.forEach(function(f){
      try { out = out.concat(Array.prototype.slice.call(f.contentDocument.querySelectorAll(".pop"))); }
      catch(e){}
    });
    document.querySelectorAll(".pane.html div").forEach(function(h){
      if (h.shadowRoot) out = out.concat(Array.prototype.slice.call(h.shadowRoot.querySelectorAll(".pop")));
    });
    return out;
  }
  document.getElementById("oa").onclick = function(){
    allPops().forEach(function(p){ p.hidden = false; }); resizeAll(); };
  document.getElementById("ca").onclick = function(){
    allPops().forEach(function(p){ p.hidden = true; }); resizeAll(); };

  var tabs = document.querySelectorAll(".tab");
  function sel(k){
    tabs.forEach(function(t){ t.setAttribute("aria-selected", t.dataset.t === k ? "true" : "false"); });
    document.querySelectorAll(".pane").forEach(function(p){ p.hidden = (p.dataset.k !== k); });
    resizeAll();
  }
  tabs.forEach(function(t){ t.onclick = function(){ sel(t.dataset.t); }; });
  if (tabs.length) sel(tabs[0].dataset.t);
  window.addEventListener("resize", resizeAll);
})();
"""


def landed(body, ms):
    """実際に印が付いたものだけを返す。

    **当たらなかったものを一覧に載せると、押しても運べない。**
    タブの数字と一覧の件数も食い違う。**落としたものは、必ず報告する。**
    """
    keep, lost = [], []
    for c in ms:
        key = 'data-b="' + html.escape(c.get("before", ""), quote=True) + '"'
        (keep if key in body else lost).append(c)
    for c in lost:
        print("  一覧から外した（印が当たらず）: " + c["find"][:50], file=sys.stderr)
    return keep


def index_html(ms):
    """面の頭に置く、変更の一覧。

    **HTML の面では、印が内側の隠れたタブに入ることがある。**
    実際に4件とも隠れた。一覧が無ければ、読み手はそこへ辿り着けない。
    """
    if not ms:
        return ""
    li = []
    for i, c in enumerate(ms):
        li.append(
            f'<li><b>{html.escape(c["find"][:40])}</b>'
            f'<button class="go" data-i="{i}">その場所へ</button>'
            f'<div class="b">{html.escape(c.get("why",""))}</div></li>')
    return (f'<details class="idx" open><summary>この面の変更 {len(ms)} 件</summary>'
            f'<ol>{"".join(li)}</ol></details>')


def build(spec):
    tabs, panes, total = [], [], 0
    for d in spec["docs"]:
        src = io.open(d["file"], encoding="utf-8").read()
        ms = d.get("marks", [])
        ext = os.path.splitext(d["file"])[1].lower()
        if ext == ".md":
            body = mark(render(src), ms)
            ms = landed(body, ms)
            panes.append(
                f'<section class="pane md" data-k="{d["key"]}" hidden>'
                f'{index_html(ms)}<div class="doc">{body}</div></section>')
        else:
            chunk = mark_html(strip_document(src), ms)
            ms = landed(chunk, ms)
            shadow = html.escape(scope_for_shadow(
                "".join(re.findall(r"<style\b[^>]*>(.*?)</style>", chunk, re.S | re.I))))
            panes.append(
                f'<section class="pane html" data-k="{d["key"]}" data-tab="{html.escape(d["tab"])}" hidden>'
                f'<div class="lane"><b>そのままの見た目で置く</b>'
                f'<span class="how">経路を確かめている…</span></div>'
                f'{index_html(ms)}'
                f'<template data-shadowcss="{shadow}">{chunk}</template>'
                f'</section>')
        n = len(re.findall(r'mark class="chg"', panes[-1]))
        total += n
        tabs.append(f'<button class="tab" role="tab" data-t="{d["key"]}">'
                    f'{html.escape(d["tab"])}<span class="n">{n}</span></button>')

    return f"""<title>{html.escape(spec["title"])}</title>
<style>{CSS}</style>
<style id="markcss-live">{MARKCSS}</style>
<script type="text/plain" id="markcss">{MARKCSS}</script>

<div class="wrap">
<h1>{html.escape(spec["title"])}</h1>
<p class="lede">{spec.get("lede","")}</p>
{spec.get("intro","")}
<div id="bar"><b>変更 {total} 件</b>
<button id="oa">すべて開く</button><button id="ca">すべて閉じる</button></div>
<div id="tabs" role="tablist">{"".join(tabs)}</div>
{"".join(panes)}
</div>
<script>{JS}</script>
""", total


def check(out, spec, total):
    """作ったあと、開閉が成立する形かを確かめる。"""
    ok = []
    ok.append(("<script> が在る", "<script>" in out))
    ok.append((".pop[hidden] が在る", ".pop[hidden]{display:none!important}" in out))
    nb = len(re.findall(r'<mark class="chg"[^>]*data-b="', out))
    nw = len(re.findall(r'<mark class="chg"[^>]*data-w="', out))
    ok.append((f"印ごとに data-b と data-w（{nb}／{nw}）", nb == nw == total))
    ok.append(("表の中に <div> が無い", "<tbody><div" not in out and "<tr><div" not in out))
    nt = len(re.findall(r'<button class="tab" role="tab"', out))
    npn = len(re.findall(r'<section class="pane (?:md|html)"', out))
    ok.append((f"タブとパネルの数が合う（{nt}／{npn}）", nt == npn == len(spec["docs"])))
    body = out.split("<script>")[0]
    ok.append((f"波括弧が閉じている（{body.count('{')}／{body.count('}')}）",
               body.count("{") == body.count("}")))
    nhtml = sum(1 for d in spec["docs"] if not d["file"].lower().endswith(".md"))
    ok.append((f"HTML の面に雛形が在る（{out.count('<template')}／{nhtml}）",
               out.count("<template") == nhtml))
    return ok


if __name__ == "__main__":
    spec = json.load(io.open(sys.argv[1], encoding="utf-8"))
    out, total = build(spec)
    for name, good in check(out, spec, total):
        print(("  OK  " if good else "  NG  ") + name, file=sys.stderr)
    io.open(sys.argv[2], "w", encoding="utf-8").write(out)
    print(f"  印 {total} 件 / {len(spec['docs'])} 面 / {len(out)} 字", file=sys.stderr)
