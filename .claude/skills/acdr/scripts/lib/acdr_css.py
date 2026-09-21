# SPDX-License-Identifier: MIT
"""1枚の見た目。**CSS の正本はここ1つである。**

**直値を書かない** ── 色は `references/tokens.json` から出る。
段ごとに置き場所が分かれていると、配色を替えたときに片方だけが取り残される。

| 名前 | 何を着せるか |
|---|---|
| `TOKENS` | トークンの表。本体の先頭に置く |
| `TOKENS_EMBED` | 同じ表を、`:host` でも解決する形で。iframe と Shadow の中へ流し込む |
| `BASE` | 頁 ・ タブ ・ 面 |
| `CODE` | コードと差分 |
| `MARK` | 印と、押すと開く札。**中へも流し込む** |
| `SECTION` | 節（決定 ・ なぜ ・ 形の変化 …） |
"""
from __future__ import annotations

from . import tokens as _tokens

# 色の正本は references/tokens.json である
TOKENS = _tokens.css(_tokens.load())
TOKENS_EMBED = _tokens.css(_tokens.load(), host=True)

BASE = """
*{box-sizing:border-box}
body{background:var(--paper);color:var(--ink);margin:0;
font-family:"Hiragino Kaku Gothic ProN","Yu Gothic",system-ui,sans-serif;line-height:1.85}
.wrap{max-width:92rem;margin:0 auto;padding:2rem 1.2rem 5rem}
h1{font-size:1.5rem;margin:0 0 .3rem;letter-spacing:.02em}
.lede{font-size:1.06rem;border-left:4px solid var(--move);padding:.55rem 0 .55rem .9rem;
margin:1rem 0 2rem;background:var(--panel);max-width:68rem}
h2{font-size:1.12rem;margin:2.4rem 0 .8rem;padding-bottom:.3rem;border-bottom:1px solid var(--line)}
h3{font-size:1rem;margin:1.6rem 0 .5rem;color:var(--move)}
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

CODE = """
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
.lane.code-lane b{color:var(--move)}\n.code.diff td.mk{width:1.4rem;min-width:1.4rem;text-align:center;color:var(--muted)}\n.code.diff tr.ra td{background:var(--add-bg)}\n.code.diff tr.ra td.mk{color:var(--key);font-weight:700}\n.code.diff tr.rd td{background:var(--del-bg)}\n.code.diff tr.rd td.mk{color:var(--gone);font-weight:700}\n.code.diff tr.hh td{background:var(--panel);color:var(--muted);font-size:.78rem;\npadding:.25rem .8rem;border-top:1px solid var(--line);border-bottom:1px solid var(--line)}\n.code.diff tr.hh td.cd{white-space:normal}\n.code.diff .nowhy{color:var(--gone);font-weight:700}\n.code.diff td.popcell{padding-left:5.6rem}
"""

MARK = """
mark.chg{background:transparent;color:inherit;border-bottom:2px dashed var(--move);
cursor:pointer;padding:0 .1rem}
mark.chg::after{content:"変";font-size:.62rem;vertical-align:super;color:var(--move);
font-weight:700;margin-left:.12rem}
mark.chg:focus-visible{outline:2px solid var(--move);outline-offset:2px}
.pop{display:block;margin:.6rem 0 .9rem;padding:.7rem .9rem;border:1px solid var(--move);
border-radius:.3rem;background:var(--pop-bg);font-size:.9rem;line-height:1.7;
font-family:"Hiragino Kaku Gothic ProN","Yu Gothic",system-ui,sans-serif}
.pop[hidden]{display:none!important}
.pop>b{color:var(--move);display:block;margin:.5rem 0 .15rem}
.pop>b:first-child{margin-top:0}
.pop pre{white-space:pre-wrap;word-break:break-word;font-size:.82rem;margin:0;
color:inherit;opacity:.8;background:var(--pre-bg);padding:.5rem .6rem;border-radius:.2rem}
"""

SECTION = """
.acdr{border:1px solid var(--line);background:var(--panel);border-radius:.35rem;
padding:1.1rem 1.3rem 1.2rem;margin:0 0 1.8rem;max-width:68rem}
.acdr .hd{display:flex;gap:.7rem;align-items:baseline;flex-wrap:wrap;
border-bottom:1px solid var(--line);padding-bottom:.55rem;margin-bottom:1rem}
.acdr .no{font-size:.78rem;font-weight:700;letter-spacing:.1em;color:var(--move)}
.acdr h1.t{font-size:1.35rem;margin:0;letter-spacing:.02em;border:0;padding:0}
.acdr .when{margin-left:auto;font-size:.82rem;color:var(--muted)}
.acdr .st{font-size:.76rem;font-weight:700;padding:.08rem .55rem;border-radius:999px;
border:1px solid currentColor}
.acdr .st.proposed{color:var(--move)}
.acdr .st.accepted{color:var(--key)}
.acdr .st.superseded{color:var(--muted)}
.acdr .stnote{font-size:.82rem;color:var(--muted)}
.acdr .ask{font-size:.78rem;font-weight:700;letter-spacing:.1em;color:var(--move);
margin:0 0 .35rem}
.acdr .decide{font-size:1.12rem;background:var(--paper);border-left:4px solid var(--move);
padding:.65rem .95rem;margin:0 0 1.1rem}
.acdr .sec{margin:1.1rem 0 0}
.acdr .sec h4{font-size:.78rem;font-weight:700;letter-spacing:.1em;color:var(--move);
margin:0 0 .3rem;text-transform:none}
.acdr .sec p{margin:0}
.acdr .sec ul{margin:.1rem 0;padding-left:1.2rem}
.acdr .sec table{margin:.35rem 0 0}
.acdr .none{color:var(--muted)}
.acdr .lead{font-size:.86rem;color:var(--muted);margin:0 0 .3rem}
.acdr table.shift td:nth-child(3){font-weight:700}
.acdr .fig{margin:.4rem 0 0;padding:.6rem;background:var(--paper);border:1px solid var(--line);border-radius:.3rem;overflow-x:auto}
.acdr .fig svg{display:block;max-width:100%;height:auto}
.acdr .fig figcaption{font-size:.84rem;color:var(--muted);margin-top:.4rem}
.lede{display:none}
.bridge{max-width:68rem;font-size:.9rem;color:var(--muted);margin:0 0 .6rem}
"""
