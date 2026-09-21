# SPDX-License-Identifier: MIT
# Copyright (c) 2026 daidaiiro
"""論点を、見て選べる1枚へ組む。

文字だけで案を説明すると、読み手が頭の中で像を作ることになり、そこで解釈がぶれる。
だから案は成果物として組む ── 読み手は像を補完せず、見えているものだけで決められる。

散文で並べない。**案は表で、帰結は表で、理由は連鎖で、図は図で置く。**
情報の型が、表現の形を決める。

使い方:
    from build_board import board, Option, Table, write
    html = board(
        theme="ADR の構造", no=1, total=3,
        question="ADR の節の並びを、どういう順にするか",
        note="論点5つのうち3つが決着し、1つ新しく出た。",
        figures=[(SVG, "図の読み方")],
        kept=[Option("決定を先頭のまま、反証の節を足す",
                     "決定 → 反証 → 実測 → 残る弱点",
                     "書き手が自分の決定を攻めることになる")],
        tables=[Table("この選択が置くもの", ["段1", "段2"],
                      {"A": ["契約が3つ増える", "増えない"]})],
        dropped=[("判断の経路をたどる形", "承認の場で読み飛ばされる")],
        found=["3案が2案になった", "軸が変わった"],
        pick=("A", [("測った", "押し返されなければ承認されていた誤りが4件あった"),
                    ("結論", "A ── 決定を先頭のまま、反証の節を足す")]))
    write(html, "/path/to/board.html", "節の同一性")
"""
from __future__ import annotations

import difflib as _dl
import html as _h
import re
import sys as _sys
from dataclasses import dataclass, field

from .template import part as _t

LETTERS = "ABCDEFGH"

# ブレストボードの色。図を描く側は、これを自分のエンジンのトークンへ複製する。
# **色の正本はここではない。**`references/tokens.json` が単独で保持し、
# `tokens.py` が CSS のカスタムプロパティへ組む ──
# ここに辞書を置いていた頃は、CSS の :root と名前も値も違うものが並び、
# しかも辞書の側は参照0件の死んだ定義だった。
#
# **この Skill は図を描かない。**描き方も、描く道具も持たない ──
# deck() は図を SVG の文字列として受け取るだけで、どう描いたかを認知しない。
# 描くのは design-svg であり、成果物は具体の側の figures/ に在る。

# 出どころの種類。**機械が分岐する値は ASCII で、画面へ出す語は LABELS が保持する** ──
# 1つの語が識別子と表示を兼ねると、表示を直した瞬間に分岐が壊れる
KINDS = {"measured": "k-fact", "primary": "k-src", "rule": "k-rule",
         "assumption": "k-given", "unverified": "k-open"}
LABELS = {"measured": "実測", "primary": "原典", "rule": "決まり",
          "assumption": "前提", "unverified": "未確認"}

# 論点の状態。**値は ASCII、札の語は STATUS_LABELS が保持する**
STATUS = ("waiting", "open", "settled")
STATUS_LABELS = {"waiting": "未", "open": "新規", "settled": "決着"}


@dataclass
class Option:
    """反証を通過した案。代償を必ず添える ── 代償が無いと選べない。

    name / gist / cost は、表の1行に収まる長さで書く。
    説明が1行に収まらないなら、それは図か、別の表になるものである。

    対話の途中で案が変わったら、before と why を添える。そうすると案の名前が
    印になり、押すと「変更前」と「なぜ変えたか」が開く ── 履歴を別の場所へ
    追い出すと、いまのブレストボードと突き合わせながら読むことになる。
    """
    name: str
    gist: str
    cost: str
    before: str | None = None
    why: str | None = None


@dataclass
class Table:
    """案ごとの帰結を並べる表。列は呼び出し側が決める。

    rows は {案の記号: [その案の各列の値]}。案の記号は kept の並び順（A・B・C…）。
    """
    caption: str
    columns: list[str]
    rows: dict[str, list[str]]
    lead: str | None = None
    plain: bool = False  # 行の見出しが案の記号でないとき（層の名前など）は True


def _plain(x: str, n: int = 46) -> str:
    """印の抜き書き。**引き出しの一覧に出すので、短くする。**"""
    t = re.sub(r"<[^>]+>", "", str(x)).strip()
    return _h.escape(t[:n] + ("…" if len(t) > n else ""))


def _mark(text: str, before: str, why: str, deleted: bool = False,
          cid: str | None = None) -> str:
    """変わった箇所の印。押すと、変更前と理由が開く。

    cid を持つ印は、**引き出しから直に跳べる**。
    """
    return (f'<mark class="chg{" del" if deleted else ""}" tabindex="0" role="button" '
            f'{f'id="{cid}" ' if cid else ""}'
            f'aria-expanded="false" data-b="{_h.escape(before, quote=True)}" '
            f'data-w="{_h.escape(why, quote=True)}">{text}</mark>')


# 印がどの節に在るかを、読み手の言葉で持つ
_WHERE = {"note": "前書き", "pick": "答え", "path": "道筋", "found": "分かったこと",
          "costs": "要求事項", "weaknesses": "扱わない範囲", "tables": "表",
          "grounds": "根拠"}


def _sec(no: int, title: str, body: str) -> str:
    return f'<h2><span class="sn">{no}</span>{_h.escape(title)}</h2>{body}'


def _table(head: list[str], rows: list[list[str]], cls: str = "") -> str:
    h = "".join(f"<th>{c}</th>" for c in head)
    b = "".join("<tr>" + "".join(f"<td>{c}</td>" for c in r) + "</tr>" for r in rows)
    return f'<div class="scroll"><table class="{cls}"><tr>{h}</tr>{b}</table></div>'


def _key(letter: str, tone: str = "") -> str:
    return f'<span class="n {tone}">{letter}</span>'


@dataclass
class Topic:
    """1つの論点。deck() に並べると、タブ1枚になる。

    ブレストは複数の論点が絡むので、決着した論点も同じ1枚に置く ── 別々の
    ページに散らすと、後の論点が前の決着を前提にしていることが見えなくなる。

    status は waiting ／ open ／ settled のいずれか。決着した論点は kept を持たず、
    decision（決定・理由・次にすること）と、必要なら extras（節の見出しと中身）だけを持つ。

    path は、その結論に至った道筋。何を問うて何が除外されたかを順に並べる ──
    結論と根拠だけでは「なぜ他が残らなかったか」が見えない。
    """
    no: int
    label: str
    question: str
    status: str = "waiting"
    answer: str = ""
    note: str | None = None
    figures: list[tuple[str, str]] = field(default_factory=list)
    kept: list[Option] = field(default_factory=list)
    tables: list[Table] = field(default_factory=list)
    dropped: list[tuple[str, str]] = field(default_factory=list)
    found: list[str] = field(default_factory=list)
    pick: tuple[str, str] | None = None
    path: list[str] = field(default_factory=list)
    grounds: list[tuple[str, str, str, str]] = field(default_factory=list)
    costs: list[str] = field(default_factory=list)
    # 完成イメージのうち、図で表せないもの（宣言・設定・コード・木構造の実例）。
    # **畳まない** —— 答えの直下に開いたまま置く。図と同じ扱いである。
    # 畳むと、実例を探しながら結論を読むことになる（実際に、経過の底へ埋めた）
    example: str = ""
    # 「事項」か、(事項, 扱い) の対。**承認を求める論点は、対で書く**
    #
    # **欠陥の一覧ではない。**いずれもこの答えを覆さない ──
    # 覆しうるものは反証で除外済みである。扱いは 対象外 ／ 後続で決定 ／ 解消済 の3種。
    weaknesses: list = field(default_factory=list)
    # 未修正の誤り。(誤り, 現状) の対。**適用範囲外と分離する** ── 混在すると、制約が誤りに見える
    defects: list[tuple[str, str]] = field(default_factory=list)
    decision: list[tuple[str, str]] = field(default_factory=list)
    extras: list[tuple[str, str]] = field(default_factory=list)



# ──────────────────────────────────────────────────────────────
# 見た目 ── 承認の画面を正とし、論点が増えたらタブで切り替える。
# ブレストボードと承認の画面を別ページにすると、開いている論点の中身が二重になる
# （実際になった）。決着は同じ1枚に蓄積し、後の論点はそれを前提にする。
# ──────────────────────────────────────────────────────────────

HEAD = _t("head")

CSS = """
*{box-sizing:border-box}
body{margin:0;background:var(--paper);color:var(--ink);
font-family:"Zen Kaku Gothic New","Hiragino Kaku Gothic ProN","Yu Gothic",system-ui,sans-serif;
font-size:15px;line-height:1.85;-webkit-font-smoothing:antialiased}
.wrap{max-width:62rem;margin:0 auto;padding:2.4rem 1.15rem 6rem;display:flex;flex-direction:column;gap:2rem}
h1,h2,h3{font-family:"Shippori Mincho","Yu Mincho",serif;font-weight:600;text-wrap:balance;margin:0}
h1{font-size:2rem;letter-spacing:.02em;line-height:1.4}
h2{font-size:1.22rem}h3{font-size:1rem;font-weight:700}
p{margin:0}
.eyebrow{font-family:"JetBrains Mono",ui-monospace,monospace;font-size:.7rem;letter-spacing:.16em;
text-transform:uppercase;color:var(--muted)}
.thesis{font-size:1.05rem;max-width:46rem;color:var(--muted)}
.thesis b{color:var(--ink);font-weight:700}
header{display:flex;flex-direction:column;gap:.75rem;border-bottom:1px solid var(--rule);padding-bottom:1.6rem}

/* ── タブ ── */
#tabs{display:flex;gap:.35rem;flex-wrap:wrap;border-bottom:1px solid var(--rule);
padding-bottom:.5rem;position:sticky;top:0;background:var(--paper);z-index:5;padding-top:.5rem}
#tabs button{font:inherit;font-size:.85rem;padding:.4rem .8rem;border-radius:.3rem .3rem 0 0;
border:1px solid var(--rule-soft);border-bottom:0;background:var(--card);color:var(--muted);
cursor:pointer;display:flex;gap:.4rem;align-items:center}
#tabs button[aria-selected="true"]{background:var(--key);border-color:var(--key);
color:var(--paper);font-weight:700}
#tabs .tn{font-family:"JetBrains Mono",monospace;font-size:.7rem;opacity:.85}
.st{font-size:.66rem;font-weight:700;letter-spacing:.05em;border-radius:.2rem;
padding:.02em .4em;border:1px solid currentColor;white-space:nowrap}
.st.open{color:var(--warn)}.st.done{color:var(--key)}.st.wait{color:var(--dim)}
.st.now{background:var(--key);color:var(--paper);padding:.1rem .4rem}
#tabs button.wait{opacity:.5}
#tabs button.now{border-color:var(--key);border-width:2px;font-weight:700}
.front{background:var(--accent-bg,#d5e6e3);border:2px solid var(--key);border-radius:.4rem;
  padding:.9rem 1rem;margin:1rem 0}
.front h3{margin:0 0 .5rem;font-size:.95rem;color:var(--key)}
.front ol{margin:0;padding-left:1.2rem}
.front li{margin:.35rem 0}
.front .jump{background:none;border:none;color:var(--key);font-weight:700;
  text-decoration:underline;cursor:pointer;padding:0;font-size:inherit;font-family:inherit}
.front .why{color:var(--ink-soft,#5b6b66);font-size:.84rem}
.waiting{color:var(--dim);font-size:.84rem;margin:.6rem 0 0}
#tabs button[aria-selected="true"] .st{color:var(--paper)}

/* ── 論点 ── */
.q{background:var(--card);border:1px solid var(--rule-soft);border-radius:.6rem;box-shadow:var(--shadow);
padding:1.5rem 1.4rem;display:flex;flex-direction:column;gap:1.2rem}
.qh{display:flex;gap:.7rem;align-items:baseline;flex-wrap:wrap}
.qid{font-family:"JetBrains Mono",monospace;font-size:.72rem;letter-spacing:.1em;color:var(--key);
background:var(--key-soft);padding:.12rem .5rem;border-radius:.25rem}
.ans{background:var(--key-soft);border-left:4px solid var(--key);border-radius:.35rem;padding:.9rem 1.05rem}
.ans .big{font-family:"JetBrains Mono",monospace;font-weight:700;color:var(--key);
border:2px solid var(--key);border-radius:.2rem;padding:0 .4em;margin-right:.5rem}
.note{background:var(--sunk);border-left:3px solid var(--dim);border-radius:.3rem;
padding:.7rem .95rem;font-size:.9rem;color:var(--muted)}
.note b{color:var(--ink)}
figure{margin:0;background:var(--card);border:1px solid var(--rule-soft);border-radius:.45rem;
padding:1rem;display:flex;flex-direction:column;gap:.6rem;min-width:0}
figure svg{max-width:100%;height:auto;display:block;margin:0 auto}
figcaption{font-size:.82rem;color:var(--muted)}
.figs{display:grid;gap:1rem;grid-template-columns:1fr}
.ex{margin:1rem 0;padding:.9rem 1.1rem;border:1px solid var(--rule);border-left:3px solid var(--key);border-radius:6px;background:var(--card)}
.ex h3{margin:1.1rem 0 .4rem;font-size:.95rem}
.ex h3:first-child{margin-top:0}
.ex pre{overflow-x:auto;margin:.3rem 0;padding:.7rem .9rem;background:var(--sunk);border-radius:4px}
.ex pre code{font-size:.82rem;line-height:1.6;white-space:pre}
details{border:1px solid var(--rule-soft);border-radius:.4rem;background:var(--paper)}
details+details{margin-top:.5rem}
summary{cursor:pointer;padding:.55rem .9rem;font-size:.87rem;font-weight:700;color:var(--key)}
details>div{padding:0 .9rem .9rem}
.scroll{overflow-x:auto}
table{border-collapse:collapse;width:100%;font-size:.87rem;margin:.4rem 0;
background:var(--card);border-radius:.3rem;overflow:hidden}
th,td{border:1px solid var(--rule);padding:.4rem .6rem;text-align:left;vertical-align:top}
th{background:var(--panel);font-weight:700;font-size:.78rem;letter-spacing:.03em;color:var(--muted)}
code{font-family:"JetBrains Mono",monospace;font-size:.85em;background:var(--sunk);
padding:.06rem .3rem;border-radius:.2rem}
ol.path{margin:.4rem 0;padding-left:1.3rem;font-size:.88rem}
ol.path li{margin:.3rem 0}
ul.plain{margin:.4rem 0;padding-left:1.2rem;font-size:.9rem}
ul.plain li{margin:.3rem 0}
.lead-s{display:block}
.sub-s{margin-top:.25rem;font-size:.92em;color:var(--sub)}
.part{display:inline-block;font-size:.78rem;font-weight:700;color:var(--key);
background:var(--key-soft);border-radius:.25rem;padding:.1em .5em}
.kind{display:inline-block;font-family:"JetBrains Mono",monospace;font-size:.66rem;font-weight:700;
padding:.04em .4em;border-radius:.2rem;border:1px solid currentColor;white-space:nowrap}
.k-fact{color:var(--key)}.k-given{color:var(--warn)}.k-src{color:var(--src-o)}
.k-rule{color:var(--muted)}.k-open{color:var(--dim)}
.kind+small{display:block;color:var(--muted);font-size:.78rem;line-height:1.6;margin-top:.2rem}
.n{font-family:"JetBrains Mono",monospace;font-size:.7rem;color:var(--key);border:1px solid var(--key);
border-radius:2px;padding:0 .4em;display:inline-block}
.n.out{color:var(--dim);border-color:var(--dim)}
.cost{color:var(--muted)}

/* ── 回答 ── */
.form{display:flex;flex-direction:column;gap:.8rem;border-top:1px solid var(--rule);padding-top:1.1rem}
.verdicts{display:flex;gap:.6rem;flex-wrap:wrap}
.vb{font:inherit;font-size:.92rem;padding:.5rem 1.2rem;border-radius:.35rem;cursor:pointer;
border:1px solid var(--rule);background:var(--paper);color:var(--ink)}
.vb[aria-pressed="true"]{background:var(--key);border-color:var(--key);color:var(--paper);font-weight:700}
.vb.ret[aria-pressed="true"]{background:var(--warn);border-color:var(--warn)}
.reasons{display:flex;gap:.4rem;flex-wrap:wrap;margin:.3rem 0}
.rb{font:inherit;font-size:.8rem;padding:.2rem .6rem;border-radius:1rem;cursor:pointer;
border:1px dashed var(--rule);background:transparent;color:var(--muted)}
.rb:hover{color:var(--ink);border-color:var(--key)}
label{font-size:.82rem;color:var(--muted);font-weight:700}
textarea{font:inherit;font-size:.9rem;width:100%;min-height:4.5rem;padding:.6rem .7rem;
border-radius:.35rem;border:1px solid var(--rule);background:var(--paper);color:var(--ink);resize:vertical}
.send{display:flex;flex-direction:column;gap:.7rem;background:var(--card);
border:1px solid var(--rule-soft);border-radius:.6rem;padding:1.2rem 1.3rem;box-shadow:var(--shadow)}
.sb{font:inherit;font-size:.95rem;font-weight:700;padding:.6rem 1.4rem;border-radius:.35rem;
cursor:pointer;border:1px solid var(--key);background:var(--key);color:var(--paper);align-self:flex-start}
.sb:disabled{opacity:.45;cursor:not-allowed}
.note-s{font-size:.82rem;color:var(--muted)}
.out{font-family:"JetBrains Mono",monospace;font-size:.74rem;white-space:pre-wrap;word-break:break-all;
background:var(--sunk);border:1px solid var(--rule);border-radius:.35rem;padding:.7rem;
max-height:16rem;overflow:auto}
footer{font-size:.76rem;color:var(--dim);border-top:1px solid var(--rule);padding-top:1rem}
footer a{color:inherit}

/* ── この回の変更の引き出し ── */
#dtoggle{position:fixed;right:0;top:35vh;z-index:40;font:inherit;font-size:.8rem;font-weight:700;
writing-mode:vertical-rl;padding:.9rem .45rem;border:1px solid var(--warn);border-right:0;
border-radius:.4rem 0 0 .4rem;background:var(--warn-soft);color:var(--warn);cursor:pointer;
box-shadow:var(--shadow);display:flex;align-items:center;gap:.4rem}
#dtoggle .dn{writing-mode:horizontal-tb}
.dn{display:inline-block;min-width:1.4em;text-align:center;font-family:"JetBrains Mono",monospace;
font-size:.7rem;background:var(--warn);color:var(--paper);border-radius:1rem;padding:.05em .45em;
margin-left:.35em}
/* 引き出しは、**見出しが止まり、一覧だけが動く**形にする。
   見出しを sticky で止めると、内側の余白の分だけ中身が上へ抜けて透ける（実際に透けた）。 */
#drawer{position:fixed;right:0;top:0;bottom:0;width:min(26rem,92vw);z-index:41;
display:flex;flex-direction:column;overflow:hidden;padding:0;
background:var(--card);border-left:1px solid var(--rule);box-shadow:var(--shadow)}
#drawer[hidden]{display:none}
.dhead{flex:0 0 auto;display:flex;justify-content:space-between;align-items:center;gap:.5rem;
background:var(--card);padding:1rem 1rem .65rem;border-bottom:1px solid var(--rule)}
.dbody{flex:1 1 auto;overflow:auto;padding:.7rem 1rem 3rem}
#dclose{font:inherit;font-size:.78rem;padding:.25rem .7rem;border-radius:.3rem;cursor:pointer;
border:1px solid var(--rule);background:var(--paper);color:var(--ink)}
.dlist{list-style:none;margin:.6rem 0 0;padding:0;display:flex;flex-direction:column;gap:.2rem}
.dlist .dq{font-size:.78rem;font-weight:700;color:var(--key);margin-top:.7rem;
border-bottom:1px solid var(--rule-soft);padding-bottom:.2rem}
.dgo{font:inherit;font-size:.82rem;text-align:left;width:100%;cursor:pointer;
border:1px solid transparent;border-radius:.3rem;background:none;color:var(--ink);
padding:.35rem .4rem;line-height:1.6}
.dgo:hover,.dgo:focus{border-color:var(--warn);background:var(--warn-soft)}
.dw{display:inline-block;font-size:.66rem;font-weight:700;color:var(--warn);
border:1px solid var(--warn);border-radius:.2rem;padding:0 .35em;margin-right:.45em}
mark.chg.hit{outline:3px solid var(--warn);outline-offset:2px}

/* ── 変更の印 ── */
mark.chg{background:var(--warn-soft);color:var(--ink);border-radius:.15em;cursor:pointer;
box-shadow:-.2em 0 0 var(--warn-soft),.2em 0 0 var(--warn-soft);border-bottom:2px solid var(--warn)}
mark.chg::after{content:"▸";font-size:.72em;color:var(--warn);margin-left:.3em;font-weight:700}
mark.chg[aria-expanded="true"]::after{content:"▾"}
mark.chg.del{background:none;box-shadow:none;color:var(--dim);border-bottom:1px dashed var(--dim)}
mark.chg.del b{text-decoration:line-through}
.pop[hidden]{display:none}
.pop{display:block;background:var(--pop);border:1px solid var(--popline);border-left:3px solid var(--warn);
border-radius:.3rem;padding:.7rem .9rem;margin:.6rem 0 .2rem;font-size:.82rem;line-height:1.75;font-weight:400}
.pop b{display:block;font-size:.66rem;letter-spacing:.14em;color:var(--warn);margin-bottom:.3rem}
[hidden]{display:none!important}
"""

SCRIPT = r"""<script>
(function(){
  var ROOT=document.getElementById("root");
  var tabs=document.querySelectorAll("#tabs button");
  tabs.forEach(function(b){
    b.addEventListener("click",function(){
      tabs.forEach(function(o){
        o.setAttribute("aria-selected","false");
        document.getElementById(o.dataset.t).hidden=true;
      });
      b.setAttribute("aria-selected","true");
      document.getElementById(b.dataset.t).hidden=false;
      window.scrollTo({top:0,behavior:"smooth"});
    });
  });

  /* 現在地の「いま見る論点」から、その論点のタブへ跳ぶ */
  document.querySelectorAll(".front .jump").forEach(function(j){
    j.addEventListener("click",function(){
      var t=document.querySelector('#tabs button[data-t="'+j.dataset.go+'"]');
      if(t) t.click();
    });
  });

  /* 変更の印 ── 押すと、変更前と理由が開く */
  document.querySelectorAll("mark.chg").forEach(function(m){
    m.addEventListener("click",function(){
      var open=m.getAttribute("aria-expanded")==="true";
      var nx=m.nextElementSibling;
      if(open){ if(nx&&nx.classList.contains("pop")) nx.hidden=true; }
      else{
        if(!nx||!nx.classList.contains("pop")){
          nx=document.createElement("span"); nx.className="pop";
          nx.innerHTML="<b>変更前</b>"+m.dataset.b+"<b style='margin-top:.5rem'>なぜ変えたか</b>"+m.dataset.w;
          m.parentNode.insertBefore(nx,m.nextSibling);
        }
        nx.hidden=false;
      }
      m.setAttribute("aria-expanded",open?"false":"true");
    });
    m.addEventListener("keydown",function(e){
      if(e.key==="Enter"||e.key===" "){e.preventDefault();m.click();}
    });
  });

  /* この回の変更の引き出し ── どの画面からでも開ける */
  var dt=document.getElementById("dtoggle"), dw=document.getElementById("drawer");
  function drawer(open){
    if(!dt||!dw) return;
    dw.hidden=!open; dt.setAttribute("aria-expanded",open?"true":"false");
  }
  if(dt){
    dt.addEventListener("click",function(){ drawer(dw.hidden); });
    document.getElementById("dclose").addEventListener("click",function(){ drawer(false); });
    document.addEventListener("keydown",function(e){ if(e.key==="Escape") drawer(false); });
    document.querySelectorAll(".dgo").forEach(function(b){
      b.addEventListener("click",function(){
        var tb=document.querySelector('#tabs button[data-t="'+b.dataset.go+'"]');
        if(tb) tb.click();
        var m=document.getElementById(b.dataset.cid);
        if(m){
          var d=m.closest("details"); if(d) d.open=true;
          m.scrollIntoView({block:"center",behavior:"smooth"});
          m.classList.add("hit"); setTimeout(function(){ m.classList.remove("hit"); },1800);
        }
        if(window.matchMedia("(max-width:48rem)").matches) drawer(false);
      });
    });
  }

  /* 回答 ── 論点ごとに1件だけ入る */
  var state={};
  document.querySelectorAll(".form").forEach(function(f){
    var qid=f.dataset.q;
    f.querySelectorAll(".vb").forEach(function(b){
      b.addEventListener("click",function(){
        f.querySelectorAll(".vb").forEach(function(o){o.setAttribute("aria-pressed","false")});
        b.setAttribute("aria-pressed","true");
        state[qid]=b.dataset.v; count();
      });
    });
    f.querySelectorAll(".rb").forEach(function(b){
      b.addEventListener("click",function(){
        var t=f.querySelector(".reason");
        t.value=(t.value?t.value.replace(/\s*$/,"")+"／":"")+b.dataset.r; t.focus();
      });
    });
  });
  function count(){
    var n=Object.keys(state).length;
    document.getElementById("cnt").textContent=n;
    document.getElementById("go").disabled=(n===0);
  }
  document.getElementById("go").addEventListener("click",function(){
    var answers=[];
    document.querySelectorAll(".form").forEach(function(f){
      var qid=f.dataset.q;
      if(!state[qid]) return;
      var r=f.querySelector(".reason").value.trim();
      var nt=f.querySelector(".note-in").value.trim();
      answers.push({questionId:qid, topic:ROOT.dataset.themeName,
        title:f.dataset.title, verdict:state[qid],
        returnReason:(state[qid]==="returned"&&r)?r:null, note:nt||null});
    });
    var sheet={board:ROOT.dataset.board, round:Number(ROOT.dataset.round),
      answeredAt:new Date().toISOString(), answers:answers};
    var txt=JSON.stringify(sheet,null,2);
    var out=document.getElementById("out"); out.hidden=false; out.textContent=txt;
    fetch("/answer",{method:"POST",headers:{"Content-Type":"application/json"},body:txt})
      .then(function(r){return r.json().then(function(j){
        out.textContent=(r.ok?"送りました。"+(j.next||"")+"\n\n":"受け取られませんでした。貼ってください。\n\n")+txt;})})
      .catch(function(){ /* サーバーが無ければ、貼る形へ切り替わる */ });
    out.scrollIntoView({block:"nearest",behavior:"smooth"});
  });
})();
</script>"""


def _cut(x: str, sep: str = " ── ") -> tuple[str, str, str]:
    """区切りで割る。**タグの内側では割らない。**

    印を付けたあとの文字列には、属性の中にも区切りが入る ──
    そこで割ると、属性が本文へ漏れる（実際に漏れた）。
    """
    depth, i = 0, 0
    while i < len(x):
        c = x[i]
        if c == "<":
            depth += 1
        elif c == ">":
            depth = max(0, depth - 1)
        elif depth == 0 and x.startswith(sep, i):
            return x[:i], sep, x[i + len(sep):]
        i += 1
    return x, "", ""


def cell(x: str) -> str:
    """1つの欄に2つのことが入っているものを、**主張と説明の2段**にする。

    「主張 ── 説明」と書いたものは、**1つの欄に2つのことが入っている**。
    区切りで割ると、読む側が文を解きほぐさずに済む。

    次の3つは割らない ── 文字列でないもの／引用（「で始まるもの）／既に割ってあるもの。
    **引用を割らないのは、原文の形を変えないためである。**
    **既に割ってあるものを割らないので、何度当てても同じものが出る（冪等）。**
    """
    if not isinstance(x, str) or x.lstrip().startswith("「") or 'class="lead-s"' in x:
        return x
    head, sep, tail = _cut(x)
    if not sep:
        return x
    if head.startswith("<b>") and head.endswith("</b>"):
        head = head[3:-4]
    return f'<b class="lead-s">{head}</b><div class="sub-s">{tail}</div>'


def _pairs(items: list[str], left: str, right: str) -> str:
    """『主張 ── 説明』の並びを、2列の表にする。区切りが無い行は、右を空にする。"""
    rows = []
    for x in items:
        a, sep, b = _cut(x)
        rows.append([f"<b>{a}</b>", b if sep else ""])
    return _tbl([left, right], rows)


def prev(mark: str, before: str, why: str, words: str = "") -> str:
    """閉じた前の答えを、3行の表にする。extras の中身として渡す。

    **利用者の言葉は、そのまま置く** ── 差し戻された理由は、言い換えると別のものになる。
    """
    rows = [("前の答え", f"<b>{mark}</b>　{before}"), ("なぜ組み直したか", cell(why))]
    if words:
        rows.append(("利用者の言葉（そのまま）", words))
    body = "".join(f"<tr><th>{k}</th><td>{v}</td></tr>" for k, v in rows)
    return f'<div class="scroll"><table>{body}</table></div>'


def _tbl(head, rows):
    """表を組む。**見出しが全部空なら、見出しの行を出さない。**

    空の `<th>` を並べると、中身の無い帯が表の上に1本出る。
    決まりの表のように、行の名前だけで読める表では見出しが要らない。
    """
    b = "".join("<tr>" + "".join(f"<td>{c}</td>" for c in r) + "</tr>" for r in rows)
    if not any(str(c).strip() for c in head):
        return f'<div class="scroll"><table>{b}</table></div>'
    h = "".join(f"<th>{c}</th>" for c in head)
    return f'<div class="scroll"><table><tr>{h}</tr>{b}</table></div>'


# ── この回で何が変わったか ────────────────────────────────────
#
# **毎回どこを直したかを、書き手が文章で言うのは仕組みではない。**
# 前の回の生成物を記録しておき、**ブレストボードが自分で印を付ける。**

_FIELDS = ("note", "pick", "path", "found", "costs", "weaknesses", "tables", "grounds")

ADDED = "（この回で足した）"


def snapshot(topics: list["Topic"]) -> dict:
    """生成物を記録する。**次の回で、この記録と比べる。**

    1行を欄の並びとして持つ ── 行の中の**どの欄が変わったか**まで見るためである。
    """
    out = {}
    for t in topics:
        out[str(t.no)] = {
            "note": [[t.note or ""]],
            "pick": [[t.pick[1] if t.pick else ""]],
            "path": [[x] for x in t.path],
            "found": [[x] for x in t.found],
            "costs": [[x] for x in t.costs],
            "weaknesses": [list(w) if isinstance(w, (tuple, list)) else [str(w)]
                           for w in t.weaknesses],
            "tables": [[tb.caption, str(k)] + [str(x) for x in v]
                       for tb in t.tables for k, v in tb.rows.items()],
            "grounds": [list(g) for g in t.grounds],
        }
    return out


class Diff:
    """前の回との違い。**欄ごとに比べる。**

    並びの同じ位置どうしを比べる ── 入れ替えは「変わった」として出る。
    **黙って動くより、出たほうがよい。**
    """

    def __init__(self, t: "Topic", prev: dict | None):
        self.on = prev is not None and str(t.no) in (prev or {})
        self.was = (prev or {}).get(str(t.no), {}) if self.on else {}
        self.n = 0
        self.pair: dict[tuple[str, int], list | None] = {}
        self.items: list[tuple[str, str, str]] = []   # (印の番号, どの節か, 抜き書き)
        self.rows: dict[tuple[str, int], str] = {}    # 行ごとに1件だけ一覧へ出す
        self.seen: list[str] = []
        self.now: dict | None = None
        self.no = t.no
        if not self.on:
            return
        now = snapshot([t])[str(t.no)]
        self.now = now
        for f in _FIELDS:
            old, new = self.was.get(f) or [], now.get(f) or []
            # **並びを突き合わせてから、欄を比べる。**
            # 位置だけで比べると、1行足しただけで以降が全部「変わった」と出る（実際に出た）。
            key = lambda r: "\u241f".join(map(str, r))
            sm = _dl.SequenceMatcher(None, [key(r) for r in old], [key(r) for r in new],
                                     autojunk=False)
            for tag, i1, i2, j1, j2 in sm.get_opcodes():
                for k in range(j1, j2):
                    o = old[i1 + (k - j1)] if tag == "replace" and i1 + (k - j1) < i2 else (
                        old[i1 + (k - j1)] if tag == "equal" else None)
                    self.pair[(f, k)] = o
            for j, row in enumerate(new):
                o = self.pair.get((f, j))
                for c, x in enumerate(row):
                    if not x:
                        continue
                    if o is None or c >= len(o) or o[c] != x:
                        self.n += 1

    def mark(self, field: str, i: int, j: int, text: str) -> str:
        """変わっていれば印にする。**変わっていなければ、何も足さない。**

        印には番号を振る ── **引き出しから、その印まで直に跳ぶため**である。
        """
        if not self.on or not isinstance(text, str) or not text:
            return text
        row = self.pair.get((field, i))
        added = row is None or j >= len(row)
        if not added and row[j] == text:
            return text
        cid = f"c{self.no}-{len(self.seen)}"
        self.seen.append(cid)
        # **一覧は行ごとに1件にする。**欄ごとに出すと、「持つ」だけの行が並ぶ（実際に並んだ）。
        if (field, i) not in self.rows:
            self.rows[(field, i)] = cid
            self.items.append((cid, _WHERE.get(field, field), _plain(self.label(field, i, text))))
        return _mark(text, ADDED if added else row[j],
                     "この回で足した" if added else "この回で変わった", cid=cid)

    def label(self, field: str, i: int, fallback: str) -> str:
        """一覧に出す、その行の見出し。**行を見分けられる欄を選ぶ。**"""
        rows = (self.now or {}).get(field) or []
        if i >= len(rows):
            return fallback
        r = rows[i]
        if field == "tables" and len(r) >= 2:
            return f"{r[0]}／{r[1]}"
        return r[0] if r else fallback

    def one(self, field: str, text: str) -> str:
        """1つしか無い欄（前書き・答え）に当てる。"""
        return self.mark(field, 0, 0, text)


# よく使う差し戻しの理由。**値そのものは限定しない** ── 差し込みの見本である
REASONS = ("もっと単純に", "前提が違う", "別の道も見たい")


def _fold(summary, body):
    return _t("fold", summary=summary, body=body)


def _panel(t: Topic, theme: str, ask: bool = True, prev: dict | None = None,
           diff: "Diff | None" = None) -> str:
    """論点1つぶん。開いているものは答えと裏づけと回答欄、まだのものは問いだけ。

    prev を渡すと、**この回で変わった欄に印が付く** ── 押すと前の回の中身が開く。
    """
    qid = f"Q{t.no}"
    d = diff if diff is not None else Diff(t, prev)
    out = [_t("topic-head", qid=qid, question=_h.escape(t.question))]
    if t.note:
        out.append(_t("note", body=d.one("note", t.note)))

    if t.pick:
        letter, concl = t.pick
        out.append(_t("answer", letter=_h.escape(letter), body=d.one("pick", concl)))
    elif t.decision:
        out.append(_tbl(["", ""], [[f'<span class="st done">{_h.escape(k)}</span>', v]
                                   for k, v in t.decision]))

    folds = []
    # 完成イメージ ── **見出しは道具が作る。** 手で書かせると板ごとに違う形になる。
    # **畳まない。** 畳むと、読み手は答えを文章だけで受け取ることになる
    # （実際に「図と完成イメージから全くイメージがわかない」と差し戻された）。
    image = ""
    if t.figures:
        # 図は縦方向へ並べる。横に並べると、縦横比の違う図が幅に合わせて縮み、
        # 文字が読めなくなる（742×100 の図が 380px で潰れた）
        image += _t("figures", figures="".join(
            _t("figure", svg=svg, caption=cap) for svg, cap in t.figures))
    if t.example:
        image += _t("example", body=t.example)
    if image:
        what = []
        if t.figures:
            what.append(f"図{len(t.figures)}枚" if len(t.figures) > 1 else "図")
        if t.example:
            what.append("実例")
        folds.append(_t("fold-open",
                        summary="この答えの完成イメージ ── " + "と、".join(what),
                        body=image))

    for part, claim, kind, src in t.grounds:
        if kind not in KINDS:
            raise ValueError(f"論点{t.no}: 出どころの種類が「{kind}」。"
                             f"使えるのは {'／'.join(KINDS)} である")
        if not src.strip() or not part.strip():
            raise ValueError(f"論点{t.no}: 根拠に、支える先か出どころが無い ── 「{claim[:20]}…」")
    # 節は4つだけにする。**同じことを3か所に書かない** ──
    # 論証（残った理由）・前提・残る危険・経過。この順は、読み手が判定に使う順である。
    #   論証  ── 反証を通過した案と、除外した案。**結論はここだけから出る**
    #   前提  ── 根拠。論証が乗っているもの
    #   危険  ── まだ弱いところ。承認の判断に要る
    #   経過  ── 道筋・反証で分かったこと・負担すること・前の答え。読みたい人だけが開く
    argue = ""
    if t.kept:
        rows = []
        for i, o in enumerate(t.kept):
            name = f"<b>{o.name}</b>"
            if o.before and o.why:
                name = _mark(name, o.before, o.why)
            rows.append([f'<span class="n">{LETTERS[i]}</span>', name, cell(o.gist),
                         f'<span class="cost">{cell(o.cost)}</span>'])  # 案は印を持つ
        argue += (f'<p class="note-s">反証を通過した案 {len(t.kept)}件。'
                  "このうち1つを残し、他は代償が重いか、前提を壊す。</p>"
                  + _tbl(["", "案", "中身", "代償"], rows))
    if t.dropped:
        argue += (f'<p class="note-s">除外した案 {len(t.dropped)}件 ── 何が壊れるか。</p>'
                  + _tbl(["", "除外した案", "何が壊れるか"],
                         [['<span class="n out">×</span>',
                           _mark(f"<b>{d}</b>", "この案は残っていた", w, deleted=True), cell(w)]
                          for d, w in t.dropped]))
    ti = 0
    for tb in t.tables:
        body = []
        for k, v in tb.rows.items():
            body.append([(f"<b>{k}</b>" if tb.plain else f'<span class="n">{k}</span>')]
                        + [cell(d.mark("tables", ti, 2 + j, str(x))) for j, x in enumerate(v)])
            ti += 1
        argue += ((f'<p class="note-s">{tb.lead}</p>' if tb.lead else "")
                  + f"<p class=\"note-s\"><b>{_h.escape(tb.caption)}</b></p>"
                  + _tbl([""] + tb.columns, body))
    if argue:
        folds.append(_fold("この答えが残った理由 ── 反証を通過した案と、除外した案", argue))

    if t.grounds:
        folds.append(_fold(f"前提 ── この論証が乗っているもの（{len(t.grounds)}件）",
                           _tbl(["結論のどこを支えるか", "もとにしたこと", "その出どころ"],
                                [[f'<span class="part">{d.mark("grounds", gi, 0, p)}</span>',
                                  d.mark("grounds", gi, 1, c),
                                  f'<span class="kind {KINDS[k]}">{LABELS[k]}</span>'
                                  f'<small>{d.mark("grounds", gi, 3, src)}</small>']
                                 for gi, (p, c, k, src) in enumerate(t.grounds)])))

    if t.weaknesses:
        rows, naked = [], 0
        for wi, w in enumerate(t.weaknesses):
            if isinstance(w, (tuple, list)) and len(w) >= 2:
                rows.append([cell(d.mark("weaknesses", wi, 0, w[0])),
                             cell(d.mark("weaknesses", wi, len(w) - 1, w[-1]))])
            else:
                naked += 1
                rows.append([cell(d.mark("weaknesses", wi, 0, w)),
                             '<span class="cost">扱いが未記載である</span>'])
        lead = ('<p class="note-s"><b>いずれも、この答えを覆さない。</b>'
                "覆しうるものは反証で除外済みであり、ここには残存しない ── "
                "<b>承認を保留する理由にはならない。</b></p>"
                '<p class="note-s">扱いは3種である ── '
                "<b>対象外</b>（この答えでは解決しない。解決する手段が別に要る）／ "
                "<b>後続で決定</b>（この答えの内側で、どこで決めるかが定まっている）／ "
                "<b>解消済</b>（既に解決した）。</p>")
        if naked:
            lead += f'<p class="note-s"><b>{naked}件に扱いが無い。</b></p>'
        folds.append(_fold(f"この答えが扱わない範囲（{len(t.weaknesses)}件）",
                           lead + _tbl(["事項", "扱い"], rows)))

    # **欠陥は、適用範囲外と分離する。**同じ節に混ぜると、制約が欠陥に見える。
    if t.defects:
        folds.append(_fold(
            f"未修正の誤り（{len(t.defects)}件）",
            '<p class="note-s"><b>この答えの中で、まだ修正していない誤りである。</b>'
            "適用範囲外とは別に記載する ── 混在させると、制約が誤りに見える。</p>"
            + _tbl(["誤り", "現状"], [[cell(a), cell(b)] for a, b in t.defects])))

    hist = ""
    if t.path:
        hist += (f'<p class="note-s">そう判断するまで（道筋 {len(t.path)}手）</p>'
                 + _t("path", items="".join(
                     _t("path-item", body=d.mark("path", i, 0, x))
                     for i, x in enumerate(t.path))))
    if t.found:
        hist += (f'<p class="note-s">反証で分かったこと（{len(t.found)}件）</p>'
                 + _pairs([d.mark("found", i, 0, x) for i, x in enumerate(t.found)],
                          "何が分かったか", "だから何が決まったか"))
    if t.costs:
        hist += (f'<p class="note-s">この答えが要求する事項（{len(t.costs)}件）</p>'
                 + _pairs([d.mark("costs", i, 0, x) for i, x in enumerate(t.costs)],
                          "要求する事項", "理由"))
    for title, body in t.extras:
        hist += f'<p class="note-s">{_h.escape(title)}</p>' + body
    if hist:
        folds.append(_fold("経過 ── 道筋と、そこで分かったこと", hist))

    if folds:
        out.append(_t("folds", body="".join(folds)))

    # 決着した論点は回答欄を持たない。決まったことを、もう一度聞かない
    if ask and t.status != "settled" and (t.pick or t.decision):
        out.append(_t(
            "form", qid=qid, title=_h.escape(t.question, quote=True),
            reasons="".join(_t("reason-button", text=r) for r in REASONS)))
    return _t("topic", body="".join(out))


def audit(topics: list["Topic"], extras=None, queue=None) -> list[str]:
    """出す前に、機械で見られるものだけを検査する。**見つけるが、直さない。**

    見るのは4つ。どれも、実際にやらかしたものである。
      1 一度に開く論点が多すぎないか  —— 8件を同時に出し、どれを確認すればよいか判定できなくなった。
                                      数えるのは**いま開いている論点**であって、決着した論点を含む総数ではない
      2 いま見る論点を示しているか    —— 8件を同時に出し、順番の管理を承認する側へ渡した
      3 宣言されていない依存が無いか  —— 答えの本文だけが他の論点を前提にし、根拠の欄に出てこなかった
      4 試す相手が在るか            —— 下流にも外の作業にも使われない答えを、承認へ出そうとした
      5 扱わない範囲に扱いが在るか  —— 制約を並べたまま承認を求め、覆すか否かを示さなかった
      6 答えに完成イメージが在るか  —— 図も実例も無いまま承認を求めた。人は文章だけで決定を受け取れない

    **見えないもの**が4つある —— 答えの中身が正しいか、反証が十分か、図が主張を運べているか、
    そして**「論点N」と書かずに他の論点を前提にしている依存**である。3つ目の検査が拾うのは、
    文字列として「論点N」が現れる場合だけで、語だけを借りた依存は見えない。
    """
    out: list[str] = []
    front = {n for n, _ in (queue or [])}
    open_now = front or {t.no for t in topics if t.status != "settled"}
    if len(open_now) > 5:
        out.append(f"一度に開いている論点が{len(open_now)}件ある。3〜5件に絞る ── "
                   "足すのは、既存の答えを直しても収まらないと確認し、利用者へ確認してからである")

    titles = " ".join(t for t, _ in (extras or []))
    if len(topics) >= 4 and "いま見る論点" not in titles:
        out.append("現在地に「いま見る論点」が無い。依存を自分で持つだけでは足りない ── "
                   "示さなければ、順番の管理が承認する側の仕事になる")

    for t in topics:
        if t.answer and not (t.figures or t.example):
            out.append(f"論点{t.no}: 答えを持つのに、完成イメージ（図か実例）が無い ── "
                       "文章だけでは、読み手が頭の中で像を作り、そこで解釈がぶれる")

    ref = re.compile(r"論点(\d+)")
    dep: dict[int, tuple[set[int], set[int]]] = {}
    for t in topics:
        src = " ".join(s for _, _, _, s in t.grounds)
        body = (t.answer or "") + " " + (t.pick[1] if t.pick else "")
        dep[t.no] = ({int(m) for m in ref.findall(src)} - {t.no},
                     {int(m) for m in ref.findall(body)} - {t.no})
    used: dict[int, set[int]] = {t.no: set() for t in topics}
    for n, (g, b) in dep.items():
        for x in g | b:
            used.setdefault(x, set()).add(n)

    for t in topics:
        g, b = dep[t.no]
        hidden = b - g
        if hidden:
            out.append(f"論点{t.no}: 答えの本文が論点{'・'.join(map(str, sorted(hidden)))}を"
                       "前提にしているが、根拠の欄に出てこない ── 宣言されていない依存である")
        if t.no in front and t.weaknesses:
            naked = [w for w in t.weaknesses
                     if not (isinstance(w, (tuple, list)) and len(w) >= 2)]
            if naked:
                out.append(f"論点{t.no}: いま見る論点だが、扱わない範囲{len(naked)}件に扱いが無い ── "
                           "承認者は、それが答えを覆すかを判定できない")
        if t.status != "settled" and (t.pick or t.decision) and not used.get(t.no):
            # 末端の論点には下流が無い。そこでは、外の作業（試作・実装）が試す相手になる。
            # 「外の作業」は SKILL.md が使う語である。語形に頼らず、この語だけを見る
            outer = any("外の作業" in str(w) for w in t.weaknesses)
            if not outer:
                out.append(f"論点{t.no}: 答えを持つが、下流のどの論点にも使われておらず、"
                           "外の作業で試す行き先も書かれていない ── 試す相手が無い")
    return out


def deck(theme: str, topics: list[Topic], intro: str | None = None,
         extras: list[tuple[str, str]] | None = None,
         board: str = "board", round_no: int = 1,
         queue: list[tuple[int, str]] | None = None,
         prev: dict | None = None, style: str = "") -> str:
    """論点をタブ1枚にまとめ、開いている論点に回答欄を付ける。

    先頭のタブは「現在地」── どれが決着し、どれが開いているかの一覧である。
    決着した論点も同じ1枚に残す ── 後の論点は、前の決着を前提にしている。

    **queue を渡すと、いま見る論点だけが回答欄を持つ。** 渡すのは (論点の番号, なぜ今か)
    の並びで、現在地の先頭へ畳まずに出し、タブにも印を付ける。待ちの論点は薄くなり、
    回答欄を持たない ── **どれに答えるかを利用者に判定させない。**
    渡さなければ、開いている論点すべてが回答欄を持つ（従来の振る舞い）。
    """
    for t in topics:
        if len(t.kept) == 1:
            raise ValueError(f"論点{t.no}: 反証を通過した案が1つしかない。論点の立て方を再確認する "
                             "── 1つしか残らないなら、それは選択ではない。"
                             "まだ案を出していない論点は、案を空にして置く")

    for line in audit(topics, extras, queue):
        print("  △ " + line, file=_sys.stderr)

    front = {n: why for n, why in (queue or [])}
    order = {t.no: i for i, t in enumerate(topics, start=1)}
    dmap = {t.no: Diff(t, prev) for t in topics}

    def chip(t):
        if t.status == "settled":
            return '<span class="st done">決着</span>'
        if queue is None:
            cls = {"open": "open"}.get(t.status, "wait")
            return f'<span class="st {cls}">{STATUS_LABELS[t.status]}</span>'
        if t.no in front:
            return '<span class="st now">いま見る</span>'
        return '<span class="st wait">待ち</span>'

    rows = [[f'<span class="n">{t.no}</span>', _h.escape(t.question), chip(t),
             t.answer or "<span style='color:var(--dim)'>──</span>"] for t in topics]

    lead = ""
    if front:
        items = "".join(
            _t("queue-item", at=order[n], no=n,
               label=_h.escape(next(t.label for t in topics if t.no == n)), why=why)
            for n, why in (queue or []) if n in order)
        waiting = [t for t in topics if t.status != "settled" and t.no not in front]
        tail = (f'<p class="waiting">残り {len(waiting)} 件は、上流が決まるまで動く。'
                '回答欄は持たない。</p>' if waiting else "")
        lead = _t("queue", count=len(front), items=items, tail=tail)

    diffs = [(t, dmap[t.no]) for t in topics] if prev is not None else []
    n_chg = sum(d.n for _, d in diffs)
    chg_fold = ""
    if prev is not None:
        if n_chg:
            chg_fold = _fold(
                f"この回で変わったところ（{n_chg} か所）",
                '<p class="note-s">本文の中で、<b>色の付いた欄が今回の変更である</b> ── '
                "押すと前の回の中身が開く。</p>"
                + _tbl(["論点", "変わった欄"],
                       [[_t("jump", at=order[t.no], no=t.no, label=_h.escape(t.label)),
                         f"{d.n} か所"]
                        for t, d in diffs if d.n]))
        else:
            chg_fold = _fold("この回で変わったところ（0 か所）",
                             '<p class="note-s">前の回から、中身は1つも変わっていない。</p>')

    now = _t("front", style=style,
             intro=_t("note", body=intro) if intro else "",
             lead=lead,
             list=_tbl(["#", "論点", "状態", "いまの答え"], rows),
             changes=chg_fold,
             panels="".join(_fold(ti, bo) for ti, bo in (extras or [])))

    tabs = [_t("tab-front")]
    panels = [_t("pane-front", body=now)]
    for i, t in enumerate(topics, start=1):
        cls = "" if queue is None or t.status == "settled" else (
            " class=\"now\"" if t.no in front else " class=\"wait\"")
        tabs.append(_t("tab", at=i, no=t.no, cls=cls, label=_h.escape(t.label), chip=chip(t)))
        panels.append(_t("pane", at=i, body=_panel(
            t, theme, ask=queue is None or t.no in front, prev=prev, diff=dmap[t.no])))

    n_open = sum(1 for t in topics if t.status != "settled" and (t.pick or t.decision)
                 and (queue is None or t.no in front))
    send = _t("send", open=n_open)

    # この回の変更を、**どの画面からでも開ける引き出し**にする。
    # 現在地の節だけに置くと、他の論点を見ているあいだは何も見えない（実際にそうだった）。
    drawer = ""
    if prev is not None and n_chg:
        lis = ""
        for t in topics:
            d = dmap[t.no]
            if not d.items:
                continue
            lis += _t("drawer-topic", no=t.no, label=_h.escape(t.label), count=d.n)
            for cid, where, excerpt in d.items:
                lis += _t("drawer-item", at=order[t.no], cid=cid,
                          where=where, excerpt=excerpt)
        drawer = (_t("drawer-toggle", count=n_chg)
                  + _t("drawer", items=lis))

    # <body> は公開時の器が用意する。ここで書くと入れ子になるので、器は div で持つ
    return _t("board", board=_h.escape(board, quote=True), round=round_no,
              theme=_h.escape(theme, quote=True), theme_text=_h.escape(theme),
              tabs="".join(tabs), panes="".join(panels), send=send,
              drawer=drawer, script=SCRIPT)


def _css() -> str:
    """トークンを先に置いた CSS を返す。**定義は1か所からしか出ない。**"""
    from . import tokens
    t = tokens.load()
    err = tokens.validate(t)
    if err:
        raise ValueError("トークンの検査が通っていない:\n  " + "\n  ".join(err))
    return tokens.css(t) + CSS


def write(body: str, path: str, title: str) -> str:
    """1枚を、そのまま公開できるHTMLとして書き出す。"""
    import pathlib
    page = _t("page", title=_h.escape(title), head=HEAD, style=_css(), body=body)
    pathlib.Path(path).write_text(page, encoding="utf-8")
    return page
