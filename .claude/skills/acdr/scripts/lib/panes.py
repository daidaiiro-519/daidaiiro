# -*- coding: utf-8 -*-
"""変更した文書が複数あるとき、タブ1枚にまとめて提示する。

**部品である。入口を保持しない** ── 呼ぶのは `build_acdr.py` で、
外から起動するのは `cli.py render` である。

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
  .html  そのままの見た目で置く。iframe に流し込み、届かなければ Shadow DOM へ切り替える

**どちらの入れ方でも、印の付け方と開閉は同じである。**
印は `mark.chg` で、押すと変更前と理由が開く。

検査するのは8つ。どれも、以前に実際にやらかしたものである。
  1 <script> が入っているか            —— 繋ぎ忘れて、押しても何も起きなかった
  2 印ごとに data-b と data-w が在るか
  3 .pop[hidden]{display:none} が在るか —— 無いと開いたまま閉じない
  4 表の中に <div> を置いていないか     —— <tbody> 直下の <div> は表の外へ運ばれる
  5 印の数が、渡した数と合うか
  6 タブとパネルの数が合うか
  7 波括弧が閉じているか               —— 3つ未閉鎖でスタイルが全滅した
  8 HTML を入れた面に、雛形が在るか

**HTML の形は、ここが持たない** ── `references/acdr.template.html` が持つ。
"""
import html
import io
import json
import os
import re
import subprocess
import sys

from .markdown import render, mark, outside_pre  # noqa: E402
from .template import part as _t  # noqa: E402
from . import acdr_css as _css  # noqa: E402
from .code_diff import is_code, render_code, render_diff  # noqa: E402

CODECSS = _css.CODE

# ── HTML を、そのままの見た目で置くために取り出す ─────────────────

def strip_document(src):
    """<head> の <style> と <body> の中身だけを取り出す。

    `<html>` と `<body>` を除去するのは、入れ子にすると
    親の文書構造が壊れるためである。`<style>` は除去しない。
    除去すると、見た目がそのままでなくなる。
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
    カスタムプロパティが1つも適用されず、色が全部失われる。
    """
    chunk = re.sub(r"(?<![\w-]):root(?![\w-])", ":host", chunk)
    chunk = re.sub(r"(?m)^(\s*)body(\s*[,{])", r"\1:host\2", chunk)
    return chunk


# ── 印を付ける ────────────────────────────────────────────────

def mark_html(src, marks):
    """描画済みの HTML に印を付ける。render.mark と同じ規則である。"""
    return mark(src, marks)


# ── 組む ──────────────────────────────────────────────────────

CSS = _css.TOKENS + _css.BASE

MARKCSS = _css.MARK

JS = r"""
(function(){
  var MARKCSS = document.getElementById("markcss").textContent;

  /* 印を1つ、押せるようにする。iframe の中でも Shadow の中でも同じ手順で動く */
  function wire(root, doc){
    root.querySelectorAll("mark.chg[data-w]").forEach(function(m){
      var p = doc.createElement("div");
      p.className = "pop"; p.hidden = true;
      var b1 = doc.createElement("b"); b1.textContent = "変更前";
      var pre = doc.createElement("pre"); pre.textContent = m.dataset.b;
      var b2 = doc.createElement("b"); b2.textContent = "なぜ";
      var w = doc.createElement("div"); w.textContent = m.dataset.w;
      p.appendChild(b1); p.appendChild(pre); p.appendChild(b2); p.appendChild(w);
      /* コードの面は表でできている。<td> の隣へ <div> を置くと行の中に入り、
         表のレイアウトから外れて見えなくなる ── 行を1つ挿入する */
      var row = m.closest("tr");
      if (row && row.closest(".code")) {
        var tr = doc.createElement("tr");
        tr.className = "poprow";
        var td = doc.createElement("td");
        td.colSpan = row.children.length || 2; td.className = "popcell";
        td.appendChild(p);
        tr.appendChild(td);
        if (row.nextSibling) row.parentNode.insertBefore(tr, row.nextSibling);
        else row.parentNode.appendChild(tr);
      } else {
        var host = m.closest("td") || m.closest("li") || m.parentNode;
        if (host.nextSibling) host.parentNode.insertBefore(p, host.nextSibling);
        else host.parentNode.appendChild(p);
      }
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

  /* HTML の面を立てる。iframe が空のままなら Shadow DOM へ切り替える */
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
      /* 中のタブを押しても測り直す。中の作りに依らず動作する */
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
      /* 届かなかった。Shadow DOM へ切り替える */
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
     JS で切り替える作りもある。**どちらでも動くように、
     切り替えを1つずつ試し、印が出たところで止める。** */
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
  document.querySelectorAll(".pane.md, .pane.code-pane")
          .forEach(function(p){ wire(p, document); });

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

    **印が付かなかったものを一覧に載せると、押しても運べない。**
    タブの数字と一覧の件数も食い違う。**除外したものは、必ず報告する。**
    """
    keep, lost = [], []
    for c in ms:
        key = 'data-b="' + html.escape(c.get("before", ""), quote=True) + '"'
        (keep if key in body else lost).append(c)
    for c in lost:
        print("  一覧から外した（印が付かず）: " + c["find"][:50], file=sys.stderr)
    return keep


def index_html(ms):
    """面の頭に置く、変更の一覧。

    **HTML の面では、印が内側の隠れたタブに入ることがある。**
    実際に4件とも隠れた。一覧が無ければ、読み手はそこへ辿り着けない。
    """
    if not ms:
        return ""
    li = [_t("index-item", find=html.escape(c["find"][:40]), at=i,
             why=html.escape(c.get("why", "")))
          for i, c in enumerate(ms)]
    return _t("index", count=len(ms), items="".join(li))


NOWHY: list[tuple] = []


def before_of(d):
    """変更前の中身を取得する。**記録へ複製しない** ── git から取る。

    `基準` が無ければ HEAD を使う。取得できなければ None を返し、呼ぶ側が全文へ落とす。
    """
    if "変更前" in d:                      # 直に渡された場合はそれを使う
        return d["変更前"]
    rev = d.get("基準", "HEAD")
    path = os.path.abspath(d["file"])
    root = path
    while root != os.path.dirname(root):
        root = os.path.dirname(root)
        if os.path.exists(os.path.join(root, ".git")):
            break
    else:
        return None
    rel = os.path.relpath(path, root)
    try:
        r = subprocess.run(["git", "-C", root, "show", f"{rev}:{rel}"],
                           capture_output=True, stdin=subprocess.DEVNULL, timeout=20)
        return r.stdout.decode("utf-8") if r.returncode == 0 else None
    except Exception:
        return None


def build(spec):
    tabs, panes, total = [], [], 0
    global NOWHY
    NOWHY = []
    for d in spec["docs"]:
        src = io.open(d["file"], encoding="utf-8").read()
        ms = d.get("marks", [])
        ext = os.path.splitext(d["file"])[1].lower()
        if is_code(ext):
            base = before_of(d)
            if base is None:
                body = render_code(src, ext, ms)
                lane = _t("lane-code", note="変更前を取得できないので、全文を置く")
            else:
                body, nh, nw = render_diff(base, src, ext, ms)
                if not nh:
                    body = render_code(src, ext, ms)
                    lane = _t("lane-code", note="差分が無いので、全文を置く")
                else:
                    miss = nh - nw
                    if miss:
                        NOWHY.append((d["key"], miss, nh))
                    lane = _t("lane-diff", count=nh,
                              why=(_t("lane-nowhy", count=miss) if miss
                                   else " ／ 全件に理由が付いている"))
            ms = landed(body, ms)
            panes.append(_t("pane-code", key=d["key"], lane=lane,
                            index=index_html(ms), body=body))
        elif ext == ".md":
            body = mark(render(src), ms)
            ms = landed(body, ms)
            panes.append(_t("pane-md", key=d["key"], index=index_html(ms), body=body))
        else:
            chunk = mark_html(strip_document(src), ms)
            ms = landed(chunk, ms)
            shadow = html.escape(scope_for_shadow(
                "".join(re.findall(r"<style\b[^>]*>(.*?)</style>", chunk, re.S | re.I))))
            panes.append(_t("pane-html", key=d["key"], tab=html.escape(d["tab"]),
                            index=index_html(ms), shadowcss=shadow, body=chunk))
        n = len(re.findall(r'mark class="chg"', panes[-1]))
        total += n
        tabs.append(_t("tab", key=d["key"], label=html.escape(d["tab"]), count=n))

    # 見出しと、足す CSS は、呼ぶ側が差し替えられる ── 組んだあとの文字列を
    # 置換して差し込むと、置換の当て先が変わったときに黙って外れる
    return _t("page", title=html.escape(spec["title"]),
              heading=spec.get("_heading")
              or _t("md-heading", level=1, body=html.escape(spec["title"])),
              style=CSS + CODECSS + spec.get("_css", ""),
              markcss=MARKCSS, embed=_css.TOKENS_EMBED + MARKCSS,
              lede=spec.get("lede", ""), intro=spec.get("intro", ""), total=total,
              tabs="".join(tabs), panes="".join(panes), js=JS), total


def check(out, spec, total):
    """作ったあと、開閉が成立する形かを検査する。"""
    ok = []
    ok.append(("<script> が在る", "<script>" in out))
    ok.append((".pop[hidden] が在る", ".pop[hidden]{display:none!important}" in out))
    nb = len(re.findall(r'<mark class="chg"[^>]*data-b="', out))
    nw = len(re.findall(r'<mark class="chg"[^>]*data-w="', out))
    ok.append((f"印ごとに data-b と data-w（{nb}／{nw}）", nb == nw == total))
    ok.append(("表の中に <div> が無い", "<tbody><div" not in out and "<tr><div" not in out))
    nt = len(re.findall(r'<button class="tab" role="tab"', out))
    npn = len(re.findall(r'<section class="pane (?:md|html|code-pane)"', out))
    ok.append((f"タブとパネルの数が合う（{nt}／{npn}）", nt == npn == len(spec["docs"])))
    # 面の中身は対象の文書である ── 波括弧が釣り合う保証は無い。数えるのは枠だけにする
    body = re.sub(r'<section class="pane.*?</section>', "",
                  out.split("<script>")[0], flags=re.S)
    ok.append((f"波括弧が閉じている（{body.count('{')}／{body.count('}')}）",
               body.count("{") == body.count("}")))
    if NOWHY:
        for key, miss, nh in NOWHY:
            ok.append((f"{key}: まとまり {nh} 件のうち {miss} 件に理由が付いていない", False))
    else:
        ok.append(("差分のまとまりに、全件理由が付いている", True))
    nhtml = sum(1 for d in spec["docs"]
                if not d["file"].lower().endswith(".md")
                and not is_code(os.path.splitext(d["file"])[1]))
    ok.append((f"HTML の面に雛形が在る（{out.count('<template')}／{nhtml}）",
               out.count("<template") == nhtml))
    return ok
