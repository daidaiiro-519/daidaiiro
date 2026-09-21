# SPDX-License-Identifier: MIT
"""レンダラが持つ CSS。**直値を書かず、トークンを参照する。**

ここが「具体の側から注入していた104行」の移設先である。
注入していた理由は「道具の CSS を別セッションが編集中だった」という暫時の事情で、
恒久の設計ではなかった。具体の側が道具の内部セレクタを名指しする形は、
層をまたいだ迂回であり、列が1つ増減した瞬間に無言で崩壊する。
"""
from __future__ import annotations

CSS = """
/* ── 経過のタイムライン ───────────────────────────── */
ol.path{list-style:none;counter-reset:st;margin:var(--sp-3) 0 var(--sp-1);
padding:0 0 0 var(--tl-inset);position:relative}
ol.path::before{content:"";position:absolute;left:calc(var(--tl-node) / 3);
top:var(--sp-3);bottom:var(--sp-3);width:var(--tl-rail);background:var(--rule)}
ol.path>li{counter-increment:st;position:relative;margin:0 0 var(--sp-5);
padding:0 0 0 var(--sp-1)}
ol.path>li:last-child{margin-bottom:0}
ol.path>li::before{content:counter(st);position:absolute;
left:calc(-1 * var(--tl-inset));top:0;width:var(--tl-node);height:var(--tl-node);
border-radius:50%;background:var(--card);border:var(--tl-rail) solid var(--rule);
color:var(--muted);font-size:var(--fs-3xs);font-weight:700;
line-height:calc(var(--tl-node) - var(--tl-rail) * 2);text-align:center}
ol.path>li:last-child::before{background:var(--key-soft);border-color:var(--key);
color:var(--key)}

/* ── 意味の単位には境界が要る ── 答え1つを、枠を持つカードにする ── */
ol.path .card{border:var(--bw-hair) solid var(--rule);border-radius:var(--card-radius);
background:var(--card);overflow:hidden}
ol.path .card-h{padding:var(--sp-2) var(--sp-5);background:var(--panel);
font-weight:700;border-bottom:var(--bw-hair) solid var(--rule)}
ol.path .ver{display:inline-block;min-width:1.5rem;font-size:var(--fs-2xs);
color:var(--move)}

/* ── 出来事は種別ごとに束ねる ── 行ごとに繰り返すと、
      いくつ在るのかも、どこまでが同じ種別かも読めない ── */
ol.path table.ev{margin:0;width:100%;font-size:var(--fs-sm);table-layout:fixed}
ol.path table.ev th{width:var(--ev-label);text-align:left;vertical-align:top;
background:transparent;border:0;border-right:var(--bw-hair) solid var(--rule-soft);
border-bottom:var(--bw-hair) solid var(--rule-soft);padding:var(--sp-3) var(--sp-5);
font-weight:400}
ol.path table.ev th small{display:block;color:var(--muted);font-size:var(--fs-3xs);
margin-top:var(--sp-1);letter-spacing:var(--tr-wide)}
ol.path table.ev td{border:0;border-bottom:var(--bw-hair) solid var(--rule-soft);
padding:var(--sp-3) var(--sp-5)}
ol.path table.ev tr:last-child th,ol.path table.ev tr:last-child td{border-bottom:0}
ol.path table.ev tr.g-end>td{border-bottom:var(--bw-hair) solid var(--rule)}
ol.path tr.g-finding td{background:var(--key-soft)}
ol.path .ev-n{display:inline-block;min-width:1.1rem;
font-family:"JetBrains Mono",monospace;font-size:var(--fs-3xs);font-weight:700;
color:var(--key);vertical-align:.08em}
ol.path .q-in{color:var(--ink)}
ol.path .q-in::before{content:"「"}
ol.path .q-in::after{content:"」"}

/* ── 図と生成物が、周りの背景に沈む ── 段差と枠を付ける。
      図の中の箱は薄い色なので台を濃くし、文字を読む生成物は淡くする。
      **この向きの逆転は、図が暗配色で反転しないことの埋め合わせである** ──
      design-svg が箱の色をトークン参照で出力した時点で取り消す ── */
.ex figure,.figs figure,figure.fig-top{background:var(--fig-bg);
border:var(--bw-hair) solid var(--rule);border-radius:var(--rd-lg);
padding:var(--sp-6) var(--sp-6);margin:var(--sp-4) 0}
.ex pre,.figs pre,details pre{background:var(--code-bg);
border:var(--bw-hair) solid var(--rule)}

/* ── 実例も折り畳みにしたので、器の枠が二重になる ── 器の装飾を外す ── */
.ex:has(> details.why-in){margin:var(--sp-3) 0 0;padding:0;border:0;
border-radius:0;background:none}
.ex:has(> details.why-in) > details.why-in{border:var(--bw-hair) solid var(--rule-soft);
border-radius:var(--rd-md);background:var(--paper)}
.ex:has(> details.why-in) h3{margin:var(--sp-7) 0 var(--sp-2);font-size:var(--fs-sm)}
.ex:has(> details.why-in) h3:first-child{margin-top:0}
.ex:has(> details.why-in) pre{overflow-x:auto;margin:var(--sp-1) 0;
padding:var(--sp-5) var(--sp-6);border-radius:var(--rd-sm)}
.ex:has(> details.why-in) pre code{font-size:var(--fs-xs);line-height:1.6;
white-space:pre}

/* ── 入れ子の折り畳み ── 上位と同じ見た目だと、どちらが親なのかが読めない ── */
details.why-in li > details,details.why-in details.why-in{
margin:var(--sp-3) 0 var(--sp-1);border:0;
border-left:var(--nest-guide) solid var(--rule);border-radius:0;background:none}
details.why-in li > details > summary,details.why-in details.why-in > summary{
padding:var(--sp-1) var(--sp-4);font-size:var(--fs-xs);font-weight:700;
color:var(--muted)}
details.why-in li > details[open] > summary,
details.why-in details.why-in[open] > summary{color:var(--key)}
details.why-in li > details > div,details.why-in details.why-in > div{
padding:0 0 var(--sp-2) var(--sp-4)}

/* ── 除外した案の記号は、空欄だった1列目へ出す ── 列が空なら、その列は要らない ── */
td > span.n.out{font-size:0!important;border:0!important;background:none!important;
box-shadow:none!important;padding:0!important}
td > span.n.out::after{content:counter(drop, upper-alpha);font-size:var(--fs-xs);
font-weight:700;color:var(--move)}
table:has(span.n.out) tr:has(span.n.out){counter-increment:drop}
/* 除外した案の表の見出しが、列と合っていない ── 通過した案の表と揃える */
table:has(span.n.out) tr:first-child th{font-size:0}
table:has(span.n.out) tr:first-child th::after{font-size:var(--fs-xs);
font-weight:700;color:var(--ink)}
table:has(span.n.out) tr:first-child th:nth-child(1)::after{content:"案"}
table:has(span.n.out) tr:first-child th:nth-child(2)::after{content:"中身"}
table:has(span.n.out) tr:first-child th:nth-child(3)::after{content:"除外した理由"}
/* 「×」は、押して消す操作に見える ── 箱を外し、除外を示す帯に替える */
td:has(> span.n.out){width:2rem;min-width:2rem;text-align:center;
background:var(--warn-soft)}
tr:has(span.n.out) td:nth-child(2){opacity:.72}
tr:has(span.n.out) td:nth-child(2) b{font-weight:600}
tr:has(span.n.out) td:nth-child(2) mark.chg.del{text-decoration-color:var(--line)}
/* 通過した案の表に、案の列が2つ在る ── 道具が振る連番を消し、記号だけを残す */
table td:has(> span.n:not(.out)){display:none}
table tr:has(span.n:not(.out)) th:first-child,
table:has(span.n:not(.out)) tr>th:first-child{display:none}
"""


def drop_numbering(origin_by_number: dict[int, int]) -> str:
    """除外した案の記号を、通過した案の次から振る。

    **記号は候補の識別子である** ── 除外した案も候補だったのに、
    道具の側は（案, 何が壊れるか）の対しか保持しないので記号が無い。
    """
    return "".join(f"#p{n} table:has(span.n.out){{counter-reset:drop {v}}}"
                   for n, v in sorted(origin_by_number.items()))
