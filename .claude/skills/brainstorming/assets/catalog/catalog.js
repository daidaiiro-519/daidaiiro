// brainstorming の部品の操作。ページは基盤の render が作り、serve で開いたときだけ、ステータスの読み直し ・ 回答の送信 ・ AI のチャットが動く。
(function () {
  "use strict";
  var body = document.body;
  var page = body.getAttribute("data-page");
  var boardId = body.getAttribute("data-board") || "";
  var served = location.protocol === "http:" || location.protocol === "https:";
  var $ = function (sel, root) { return (root || document).querySelector(sel); };
  var $$ = function (sel, root) { return Array.prototype.slice.call((root || document).querySelectorAll(sel)); };
  var el = function (tag, cls, text) { var e = document.createElement(tag); if (cls) e.className = cls; if (text != null) e.textContent = text; return e; };

  // 空の値を受けた部分は出さない
  $$("[data-show-if]").forEach(function (e) { if (!e.getAttribute("data-show-if")) e.hidden = true; });

  // ── サイドバー ──
  $$("[data-board-link]").forEach(function (a) {
    if (a.getAttribute("data-board-link") === boardId) a.setAttribute("aria-current", "page");
  });
  if (page === "dashboard") { var home = $("[data-home]"); if (home) home.setAttribute("aria-current", "page"); }
  var side = $(".side"), menu = $("[data-menu]");
  if (menu && side) menu.addEventListener("click", function () {
    var open = side.classList.toggle("open"); menu.setAttribute("aria-expanded", String(open));
  });

  // ── 論点のチップと面 ──
  function openTopic(id, focusPart) {
    var chips = $$(".chip[data-topic]");
    if (!chips.length) return;
    chips.forEach(function (c) { c.setAttribute("aria-selected", String(c.getAttribute("data-topic") === id)); });
    $$("[data-topic-panel]").forEach(function (p) { p.hidden = p.getAttribute("data-topic-panel") !== id; });
    var ctx = $("[data-ctx-topic]"); if (ctx) { var chip = $('.chip[data-topic="' + id + '"]'); ctx.textContent = chip ? ($(".qn", chip) || chip).textContent : id; }
    if (focusPart) {
      var panel = $('[data-topic-panel="' + id + '"]');
      var target = panel && ($('[data-part="' + focusPart + '"]', panel) || panel);
      if (target) {
        if (target.tagName === "DETAILS") target.open = true;
        target.scrollIntoView({ block: "center", behavior: "smooth" });
        target.classList.remove("flash"); void target.offsetWidth; target.classList.add("flash");
      }
    }
    try { history.replaceState(null, "", "#" + id); } catch (e) { /* 履歴を書けない枠では何もしない */ }
  }
  $$(".chip[data-topic]").forEach(function (c) { c.addEventListener("click", function () { openTopic(c.getAttribute("data-topic")); }); });
  $$("[data-jump]").forEach(function (b) { b.addEventListener("click", function () { openTopic(b.getAttribute("data-jump"), b.getAttribute("data-part")); }); });
  (function first() {
    var chips = $$(".chip[data-topic]");
    if (!chips.length) return;
    var hash = (location.hash.match(/^#(?:topic-)?(Q[0-9]+)$/) || [])[1];
    var open = chips.filter(function (c) { return c.getAttribute("data-status") === "answer"; })[0];
    openTopic(hash || (open || chips[0]).getAttribute("data-topic"));
  })();

  // ── 前の更新から変わったところ：上は要約1行、一覧は引き出し（検索 ・ 論点で絞る）、論点の中は印 ──
  (function changesView() {
    var src = $("[data-chg-src]"), drawer = $("[data-chg-drawer]");
    if (!src || !drawer) return;
    var items = $$("li", src).map(function (li) {
      return { topic: li.getAttribute("data-chg-topic"), part: li.getAttribute("data-chg-part"), label: li.getAttribute("data-chg-label"),
        summary: li.getAttribute("data-chg-summary"), reason: li.getAttribute("data-chg-reason") || "" };
    });
    if (!items.length) return;
    var topics = [];
    items.forEach(function (c) { if (topics.indexOf(c.topic) < 0) topics.push(c.topic); });
    var count = function (t) { return items.filter(function (c) { return c.topic === t; }).length; };
    var nameOf = function (t) { var q = $('.chip[data-topic="' + t + '"] .qn'); return q ? q.textContent : t; };
    var filter = "", query = "";
    var openBtn = $("[data-open-changes]"), search = $("[data-chg-search]", drawer), list = $("[data-chg-list]", drawer);
    var fbox = $("[data-chg-filter]", drawer), counter = $("[data-chg-count]", drawer);
    function open(topic) {
      var chatBox = $("[data-chat]");
      if (chatBox && !chatBox.hidden) { var x = $("[data-close-chat]", chatBox); if (x) x.click(); } // 重ねて開かない
      filter = topic || ""; paintFilter(); render(); drawer.hidden = false; openBtn.setAttribute("aria-expanded", "true"); search.focus();
    }
    function close() { drawer.hidden = true; openBtn.setAttribute("aria-expanded", "false"); openBtn.focus(); }
    function paintFilter() {
      fbox.textContent = "";
      [""].concat(topics).forEach(function (t) {
        var b = el("button", "", t ? t + "（" + count(t) + "）" : "すべて（" + items.length + "）");
        b.type = "button"; b.setAttribute("aria-pressed", String(filter === t));
        b.addEventListener("click", function () { filter = t; paintFilter(); render(); });
        fbox.appendChild(b);
      });
    }
    function render() {
      list.textContent = "";
      var q = query.trim().toLowerCase(), shown = 0;
      topics.forEach(function (t) {
        if (filter && filter !== t) return;
        var rows = items.filter(function (c) { return c.topic === t && (!q || (c.label + " " + c.summary + " " + c.reason).toLowerCase().indexOf(q) >= 0); });
        if (!rows.length) return;
        var g = el("section", "dgroup"), h = el("h3", "", nameOf(t));
        h.appendChild(el("small", "", rows.length + "か所")); g.appendChild(h);
        rows.forEach(function (c) {
          var b = el("button", "ditem"); b.type = "button";
          b.appendChild(el("span", "dp", c.label)); b.appendChild(el("span", "", c.summary));
          if (c.reason) b.appendChild(el("span", "dr", "理由：" + c.reason));
          b.addEventListener("click", function () { openTopic(c.topic, c.part); if (window.matchMedia("(max-width: 760px)").matches) close(); });
          g.appendChild(b); shown++;
        });
        list.appendChild(g);
      });
      counter.textContent = shown + "か所を表示" + (q ? "（「" + query.trim() + "」で探した）" : "");
      if (!shown) list.appendChild(el("p", "dcount", "当てはまる変わったところは無い。"));
    }
    var sum = $("[data-chg-topics]");
    topics.forEach(function (t) {
      var b = el("button", "", t + " " + count(t) + "か所"); b.type = "button"; b.title = nameOf(t);
      b.addEventListener("click", function () { open(t); }); sum.appendChild(b);
    });
    openBtn.addEventListener("click", function () { if (drawer.hidden) open(""); else close(); });
    $("[data-close-changes]", drawer).addEventListener("click", close);
    document.addEventListener("keydown", function (e) { if (e.key === "Escape" && !drawer.hidden) close(); });
    search.addEventListener("input", function () { query = search.value; render(); });
    // タブに件数、論点の中の変わった部位に印。印を押すと、その論点で引き出しを開く
    topics.forEach(function (t) {
      var chip = $('.chip[data-topic="' + t + '"]');
      if (!chip) return;
      var n = el("span", "chgn", String(count(t)));
      n.title = "前の更新から変わったところ " + count(t) + "か所"; n.setAttribute("aria-label", n.title);
      chip.appendChild(n);
    });
    items.forEach(function (c) {
      var panel = $('[data-topic-panel="' + c.topic + '"]');
      var part = panel && $('[data-part="' + c.part + '"]', panel);
      if (!part || part.classList.contains("changed")) return;
      part.classList.add("changed");
      var prev = part.previousElementSibling;
      var at = part.tagName === "DETAILS" ? $("summary", part) : $(":scope > h2", part) || $(":scope > .lbl", part) || (prev && /^(H3|DIV)$/.test(prev.tagName) ? prev : null);
      if (!at) return;
      var m = el("button", "chg-mark", "変わった"); m.type = "button"; m.title = "前の更新から変わったところを見る";
      m.addEventListener("click", function (e) { e.preventDefault(); e.stopPropagation(); open(c.topic); });
      at.appendChild(m);
    });
  })();

  // ── 完成イメージのタブ ──
  $$("section[data-part=images]").forEach(function (block) {
    var tabs = $$(".tabs button", block), panes = $$(".pane", block);
    // タブは画像の並び順、面は種類ごとの順に並ぶので、種類と、その種類の中での順番で対応させる
    var seen = {};
    var paneOf = tabs.map(function (t) {
      var kind = t.getAttribute("data-kind"), n = seen[kind] = (seen[kind] || 0) + 1;
      return panes.filter(function (p) { return p.getAttribute("data-pane") === kind; })[n - 1] || null;
    });
    function show(i) {
      tabs.forEach(function (t, j) { t.setAttribute("aria-selected", String(i === j)); });
      panes.forEach(function (p) { p.hidden = p !== paneOf[i]; });
    }
    tabs.forEach(function (t, i) { t.addEventListener("click", function () { show(i); }); });
    if (tabs.length) show(0); else block.hidden = true;
  });
  // 同じ図が2か所にあると id が重なり、隠れた側の矢印が使われるので、図ごとに id を付け直す
  $$(".figbox svg").forEach(function (svg, n) {
    var ids = $$("[id]", svg).map(function (e) { return e.id; });
    if (!ids.length) return;
    var html = svg.innerHTML;
    ids.forEach(function (id) {
      var next = "f" + n + "-" + id;
      html = html.split('id="' + id + '"').join('id="' + next + '"').split("url(#" + id + ")").join("url(#" + next + ")").split('href="#' + id + '"').join('href="#' + next + '"');
    });
    svg.innerHTML = html;
  });
  // フォルダ構成：名前が / で終わるものはフォルダ。アイコンを付けて見分ける
  var FOLDER = '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>';
  var FILE = '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/>';
  $$("ul.tree li:not(.thead)").forEach(function (li) {
    var name = ($(".tp", li) || {}).textContent || "", dir = /\/$/.test(name.trim());
    if (dir) li.setAttribute("data-dir", "");
    var ti = $(".ti", li); if (ti) ti.innerHTML = '<svg viewBox="0 0 24 24">' + (dir ? FOLDER : FILE) + "</svg>";
    // 狭い幅では / の後ろで折る（単語の途中で折らない）
    var tp = $(".tp", li);
    if (tp) { var parts = tp.textContent.split("/"); tp.textContent = ""; parts.forEach(function (part, i) { tp.appendChild(document.createTextNode(part + (i < parts.length - 1 ? "/" : ""))); if (i < parts.length - 1) tp.appendChild(document.createElement("wbr")); }); }
  });
  $$("iframe.screen").forEach(function (f) { var h = parseInt(f.getAttribute("data-height"), 10); if (h > 0) f.style.height = h + "px"; });

  // ── コード：行番号 ・ 強調する行 ・ 色付け（近似） ──
  var KEYWORDS = {
    rust: "as async await break const continue crate dyn else enum extern false fn for if impl in let loop match mod move mut pub ref return self Self static struct super trait true type unsafe use where while",
    python: "and as assert async await break class continue def del elif else except False finally for from global if import in is lambda None nonlocal not or pass raise return True try while with yield",
    javascript: "async await break case catch class const continue default delete do else export extends false finally for function if import in instanceof let new null of return super switch this throw true try typeof var void while yield",
    shell: "case do done elif else esac exit export fi for function if in local return set shift then until while",
    json: "true false null"
  };
  KEYWORDS.js = KEYWORDS.javascript; KEYWORDS.ts = KEYWORDS.typescript = KEYWORDS.javascript; KEYWORDS.sh = KEYWORDS.bash = KEYWORDS.shell;
  var COMMENT = { python: "#", shell: "#", sh: "#", bash: "#", toml: "#", yaml: "#", rust: "//", javascript: "//", js: "//", ts: "//", typescript: "//", go: "//", c: "//", java: "//" };
  function colorLine(line, lang) {
    var frag = document.createDocumentFragment();
    var words = (KEYWORDS[lang] || "").split(" ").filter(Boolean);
    var mark = COMMENT[lang];
    var re = /("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*')|([A-Za-z_][A-Za-z0-9_]*)|([^"'A-Za-z_]+)/g, m, rest = line, comment = "";
    if (mark) {
      var at = indexOutsideString(line, mark);
      if (at >= 0) { rest = line.slice(0, at); comment = line.slice(at); }
    }
    while ((m = re.exec(rest))) {
      if (m[1]) frag.appendChild(el("span", "tk-s", m[1]));
      else if (m[2] && words.indexOf(m[2]) >= 0) frag.appendChild(el("span", "tk-k", m[2]));
      else frag.appendChild(document.createTextNode(m[0]));
    }
    if (comment) frag.appendChild(el("span", "tk-c", comment));
    return frag;
  }
  function indexOutsideString(line, mark) {
    var q = null;
    for (var i = 0; i < line.length; i++) {
      var ch = line[i];
      if (q) { if (ch === "\\") i++; else if (ch === q) q = null; continue; }
      if (ch === '"' || ch === "'") { q = ch; continue; }
      if (line.substr(i, mark.length) === mark) return i;
    }
    return -1;
  }
  $$("[data-code]").forEach(function (box) {
    var pre = $("pre.src", box); if (!pre) return;
    var lang = (box.getAttribute("data-language") || "").toLowerCase();
    var hl = (box.getAttribute("data-highlight") || "").split(/[^0-9]+/).filter(Boolean).map(Number);
    var lines = pre.textContent.replace(/\n$/, "").split("\n");
    var table = el("table", "code");
    lines.forEach(function (line, i) {
      var tr = el("tr", hl.indexOf(i + 1) >= 0 ? "hl" : "");
      tr.appendChild(el("td", "ln", String(i + 1)));
      var td = el("td", "tx"); td.appendChild(colorLine(line, lang)); tr.appendChild(td);
      table.appendChild(tr);
    });
    pre.replaceWith(table);
    var copy = $("[data-copy]", box);
    if (copy) copy.addEventListener("click", function () {
      var text = lines.join("\n");
      var done = function () { copy.textContent = "コピーした"; };
      if (navigator.clipboard) navigator.clipboard.writeText(text).then(done, function () {});
    });
  });

  // ── 差分：行の最長共通部分列で、統合か並べる形に作る ──
  function lcs(a, b) {
    var n = a.length, m = b.length, t = [], i, j;
    for (i = 0; i <= n; i++) { t.push(new Array(m + 1).fill(0)); }
    for (i = n - 1; i >= 0; i--) for (j = m - 1; j >= 0; j--) t[i][j] = a[i] === b[j] ? t[i + 1][j + 1] + 1 : Math.max(t[i + 1][j], t[i][j + 1]);
    var out = []; i = 0; j = 0;
    while (i < n && j < m) {
      if (a[i] === b[j]) { out.push([" ", a[i], i + 1, j + 1]); i++; j++; }
      else if (t[i + 1][j] >= t[i][j + 1]) { out.push(["-", a[i], i + 1, null]); i++; }
      else { out.push(["+", b[j], null, j + 1]); j++; }
    }
    for (; i < n; i++) out.push(["-", a[i], i + 1, null]);
    for (; j < m; j++) out.push(["+", b[j], null, j + 1]);
    return out;
  }
  $$("[data-diff]").forEach(function (box) {
    var a = $("[data-before]", box).textContent.replace(/\n$/, "").split("\n");
    var b = $("[data-after]", box).textContent.replace(/\n$/, "").split("\n");
    if (a.length * b.length > 400000) return; // 大きすぎるときは変更前と変更後をそのまま並べる
    var ops = lcs(a, b), add = 0, del = 0;
    ops.forEach(function (o) { if (o[0] === "+") add++; if (o[0] === "-") del++; });
    var stat = $("[data-stat]", box);
    stat.appendChild(el("span", "a", "+" + add)); stat.appendChild(document.createTextNode(" ")); stat.appendChild(el("span", "r", "−" + del));
    var split = box.getAttribute("data-layout") === "split" && window.matchMedia("(min-width: 720px)").matches;
    var table = el("table", "diff" + (split ? " split" : ""));
    var SIGN = { "+": ["da", "+", "足した行"], "-": ["dr", "−", "消した行"], " ": ["", "", "変わらない行"] };
    if (!split) {
      ops.forEach(function (o) {
        var tr = el("tr", SIGN[o[0]][0]);
        tr.appendChild(el("td", "ln", o[2] == null ? "" : String(o[2])));
        tr.appendChild(el("td", "ln", o[3] == null ? "" : String(o[3])));
        var sg = el("td", "sg", SIGN[o[0]][1]); sg.setAttribute("aria-label", SIGN[o[0]][2]); tr.appendChild(sg);
        tr.appendChild(el("td", "tx", o[1]));
        table.appendChild(tr);
      });
    } else {
      var k = 0;
      while (k < ops.length) {
        if (ops[k][0] === " ") {
          var tr = el("tr"); tr.appendChild(el("td", "ln", String(ops[k][2]))); tr.appendChild(el("td", "tx", ops[k][1]));
          tr.appendChild(el("td", "ln sx", String(ops[k][3]))); tr.appendChild(el("td", "tx", ops[k][1])); table.appendChild(tr); k++; continue;
        }
        var dels = [], adds = [];
        while (k < ops.length && ops[k][0] !== " ") { (ops[k][0] === "-" ? dels : adds).push(ops[k]); k++; }
        for (var r = 0; r < Math.max(dels.length, adds.length); r++) {
          var row = el("tr"), d = dels[r], p = adds[r];
          row.appendChild(el("td", "ln", d ? String(d[2]) : "")); row.appendChild(el("td", "tx" + (d ? " dr" : ""), d ? d[1] : ""));
          row.appendChild(el("td", "ln sx", p ? String(p[3]) : "")); row.appendChild(el("td", "tx" + (p ? " da" : ""), p ? p[1] : ""));
          table.appendChild(row);
        }
      }
    }
    $(".dsrc", box).replaceWith(table);
  });

  // ── ステータスと、更新中 ・ 保留の帯 ──
  // ステータスは serve の /api/status から導く。AI が動いているあいだ（送った ・ AI に聞いた ・ ターミナルから直している）は更新中で、回答 ・ 送る ・ AI に聞くを止める。
  function postJSON(url, data) {
    return fetch(url, { method: "POST", credentials: "same-origin", headers: { "Content-Type": "application/json" }, body: JSON.stringify(data) });
  }
  var store = {
    get: function (k) { try { return JSON.parse(sessionStorage.getItem(k) || "null"); } catch (e) { return null; } },
    set: function (k, v) { try { sessionStorage.setItem(k, JSON.stringify(v)); } catch (e) { /* 残せない枠では残さない */ } },
    drop: function (k) { try { sessionStorage.removeItem(k); } catch (e) { /* 同上 */ } }
  };
  var mmss = function (ms) { var s = Math.max(0, Math.floor(ms / 1000)); return Math.floor(s / 60) + ":" + ("0" + (s % 60)).slice(-2); };
  var LABEL = { answer: "回答待ち", updating: "更新中", hold: "保留", done: "決着", waiting: "前提待ち" };
  function paintBadge(badge, st, label) {
    if (!badge) return;
    badge.setAttribute("data-status", st);
    var use = $("use", badge); if (use) use.setAttribute("href", "#st-" + st);
    var sl = $(".sl", badge); if (sl) sl.textContent = label || LABEL[st];
  }
  var head = $("[data-board-head]");
  var boardBadge = $("[data-board-status]"), firstLabel = boardBadge ? $(".sl", boardBadge).textContent : "";
  var firstStatus = boardBadge ? boardBadge.getAttribute("data-status") : "";
  var held = !!head && head.getAttribute("data-held") === "true";
  var locked = false, sending = false, chatRunning = false;
  var banner = $("[data-banner]"), bannerTimer = null;
  function setBanner(kind, text, startMs, action) {
    if (!banner) return;
    clearInterval(bannerTimer);
    if (!kind) { banner.hidden = true; return; }
    banner.hidden = false; banner.setAttribute("data-kind", kind);
    $("[data-banner-text]", banner).textContent = text;
    var tm = $("[data-banner-time]", banner);
    if (startMs) { var tickNow = function () { tm.textContent = "経過 " + mmss(Date.now() - startMs); }; tickNow(); bannerTimer = setInterval(tickNow, 1000); } else tm.textContent = "";
    var btn = $("[data-banner-action]", banner);
    btn.hidden = !action;
    if (action) { btn.textContent = action.label; btn.onclick = action.run; }
  }
  var listeners = [];
  function changed() { listeners.forEach(function (f) { f(); }); }
  function setLocked(on, label, startMs) {
    locked = on;
    if (on) {
      paintBadge(boardBadge, "updating");
      var own = sending || chatRunning;
      setBanner("updating", (label || "AI が動いている") + "。終わるとこの画面を描き直す。", startMs || Date.now(),
        own ? { label: "止める", run: function () { postJSON("/api/chat/stop", { board: boardId }); } } : null);
    } else {
      paintBadge(boardBadge, firstStatus, firstLabel);
      showHold();
    }
    changed();
  }
  function showHold() {
    if (held) {
      setBanner("hold", "このボードは保留（" + (head.getAttribute("data-held-since") || "") + " から）。解くと、回答を選んで送れる。", 0,
        served ? { label: "保留を解く", run: function () { postJSON("/api/hold", { board: boardId, hold: false }).then(reloadKeep); } } : null);
    } else setBanner(null);
  }
  var holdToggle = $("[data-hold-toggle]");
  if (holdToggle && served && !held && firstStatus !== "done") {
    holdToggle.hidden = false; holdToggle.textContent = "保留にする";
    holdToggle.addEventListener("click", function () { postJSON("/api/hold", { board: boardId, hold: true }).then(function (r) { return r.json(); }).then(function (j) { if (j.ok) reloadKeep(); else setBanner("hold", j.reason || "保留にできなかった", 0, null); }); });
  }
  function reloadKeep() {
    var chatBox = $("[data-chat]");
    store.set("bs-chat-open-" + boardId, !!(chatBox && !chatBox.hidden));
    location.reload();
  }
  showHold();

  // ── 回答：論点ごとに選ぶだけ。送るのはボードでまとめて1回 ──
  var round = Number(body.getAttribute("data-round"));
  var PICKS = "bs-picks-" + boardId + "-" + round;
  var picks = {}, saved = store.get(PICKS) || {};
  var forms = $$("[data-answer-form]");
  function sig(topic) {
    var ans = $('[data-topic-panel="' + topic + '"] .ans'), t = ans ? ans.textContent : "", h = 0;
    for (var i = 0; i < t.length; i++) h = (h * 31 + t.charCodeAt(i)) | 0;
    return String(h);
  }
  var FIELDS = ["where", "reason", "note"];
  forms.forEach(function (form) {
    var topic = form.getAttribute("data-topic");
    var field = function (n) { return $('[data-field="' + n + '"]', form); };
    function apply() {
      var p = picks[topic], off = locked || held;
      $$("[data-decision]", form).forEach(function (b) { b.setAttribute("aria-pressed", String(!!p && p.decision === b.getAttribute("data-decision"))); b.disabled = off; });
      $$(".fld[data-for]", form).forEach(function (f) { f.hidden = !p || f.getAttribute("data-for").split(" ").indexOf(p.decision) < 0; });
      $$("textarea", form).forEach(function (t) { t.disabled = off; });
    }
    function read() {
      var p = picks[topic];
      if (p) FIELDS.forEach(function (k) { p[k] = field(k).value.trim(); });
      store.set(PICKS, picks); apply(); changed();
    }
    var before = saved[topic];
    if (before && before.sig === sig(topic)) {
      picks[topic] = before;
      FIELDS.forEach(function (k) { if (before[k]) field(k).value = before[k]; });
    } else if (before) {
      $("[data-reset]", form).hidden = false; // 選んでいたあとに答えが直った
    }
    $$("[data-decision]", form).forEach(function (b) {
      b.addEventListener("click", function () {
        var d = b.getAttribute("data-decision");
        if (picks[topic] && picks[topic].decision === d) delete picks[topic]; else picks[topic] = { decision: d, sig: sig(topic) };
        $("[data-reset]", form).hidden = true;
        read();
      });
    });
    $$("textarea", form).forEach(function (t) { t.addEventListener("input", read); });
    listeners.push(apply);
    apply();
  });
  store.set(PICKS, picks);

  var bar = $("[data-sendbar]");
  var agentOf = function () { var sel = $("[data-agent]"); return sel ? sel.value : "claude"; };
  if (bar) {
    var sendBtn = $("[data-send-all]", bar), sum = $("[data-send-sum]", bar), sendMsg = $("[data-send-msg]", bar), paste = $("[data-paste]", bar);
    var sendState = function () {
      var list = Object.keys(picks), ap = 0, rt = 0, missing = [];
      list.forEach(function (t) { if (picks[t].decision === "approve") ap++; else { rt++; if (!picks[t].where) missing.push(t); } });
      sum.textContent = list.length ? "承認 " + ap + "件 ・ 差し戻し " + rt + "件 ・ 選んでいない " + (forms.length - list.length) + "件" : "まだ選んでいない";
      var kiro = agentOf() === "kiro";
      sendBtn.disabled = !list.length || missing.length > 0 || locked || held || kiro;
      sendMsg.textContent = missing.length ? missing.join(" ・ ") + "：差し戻すときは、どこが違うかを書く"
        : held ? "保留のあいだは送れない"
        : locked && !sending ? "更新が終わると送れる"
        : kiro && list.length ? "Kiro は読むだけなので送れない。AI に聞くの窓で Claude Code か Codex を選ぶ" : "";
    };
    listeners.push(sendState); sendState();
    var sel = $("[data-agent]"); if (sel) sel.addEventListener("change", sendState);
    sendBtn.addEventListener("click", function () {
      var answers = Object.keys(picks).map(function (t) {
        var p = picks[t], a = { topic: t, decision: p.decision };
        FIELDS.forEach(function (k) { if (p[k]) a[k] = p[k]; });
        return a;
      });
      var data = { board: boardId, round: round, agent: agentOf(), answers: answers };
      if (!served) {
        paste.hidden = false; paste.textContent = JSON.stringify(data, null, 1);
        sendMsg.textContent = "serve で開いていないので送れない。下の文字列を、AI とのチャットに貼る。";
        return;
      }
      if (!window.bsStream) return;
      sending = true; setLocked(true, "送った回答をもとに、AI がボードを更新している", Date.now());
      var said = "回答を送った：" + ["approve", "return"].map(function (d) {
        var ids = answers.filter(function (a) { return a.decision === d; }).map(function (a) { return a.topic; });
        return ids.length ? (d === "approve" ? "承認 " : "差し戻し ") + ids.join(" ・ ") : "";
      }).filter(Boolean).join("、");
      window.bsStream("/api/send", data, said, function (result) {
        sending = false;
        if (result.changed) { store.drop(PICKS); reloadKeep(); return; }
        setLocked(false);
        sendMsg.textContent = result.error || "ボードは変わらなかった。";
      });
    });
  }

  // ── AI に聞く ──
  var app = $(".app"), chat = $("[data-chat]");
  if (chat) {
    var msgs = $("[data-msgs]", chat), input = $("[data-input]", chat), select = $("[data-agent]", chat);
    var submit = $("[data-submit]", chat), stop = $("[data-stop]", chat), stateDot = $("[data-agent-state]", chat), note = $("[data-agent-note]", chat);
    var ICON = {
      read: '<path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>',
      edit: '<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"/>',
      run: '<path d="m4 17 6-6-6-6"/><path d="M12 19h8"/>',
      check: '<path d="m5 12 5 5 9-10"/>',
      spark: '<path d="M12 3v4M12 17v4M3 12h4M17 12h4M6.3 6.3l2.8 2.8M14.9 14.9l2.8 2.8M6.3 17.7l2.8-2.8M14.9 9.1l2.8-2.8"/>',
      diff: '<path d="M12 3v7M8.5 6.5h7M8.5 17h7"/><rect x="3" y="3" width="18" height="18" rx="2"/>'
    };
    var ACT = { read: "読んだ", edit: "直した", run: "実行した", check: "検査" };
    var NAME = { claude: "Claude Code", codex: "Codex", kiro: "Kiro" };
    var icon = function (name) { var s = document.createElementNS("http://www.w3.org/2000/svg", "svg"); s.setAttribute("class", "ic"); s.setAttribute("viewBox", "0 0 24 24"); s.setAttribute("aria-hidden", "true"); s.innerHTML = ICON[name]; return s; };
    var time = function (at) { var d = at ? new Date(at) : new Date(); return ("0" + d.getHours()).slice(-2) + ":" + ("0" + d.getMinutes()).slice(-2); };
    var current = null, controller = null;
    function who(cls, label, at) {
      var w = el("div", "who");
      var av = el("span", "av " + cls);
      if (cls === "av-ai") av.appendChild(icon("spark")); else av.textContent = "あ";
      w.appendChild(av); w.appendChild(el("b", "", label)); var t = el("time", "", time(at)); w.appendChild(t);
      return w;
    }
    function addMine(text, at) { var m = el("div", "msg me"); m.appendChild(who("av-me", "あなた", at)); m.appendChild(el("div", "bubble", text)); msgs.appendChild(m); msgs.scrollTop = msgs.scrollHeight; }
    function aiMessage(at) { var m = el("div", "msg ai"); m.appendChild(who("av-ai", NAME[select.value], at)); msgs.appendChild(m); return { root: m, work: null, bubble: null }; }
    function addWork(m, item) {
      if (!m.work) { m.work = el("div", "work"); m.work.appendChild(el("div", "wt", "作業の記録")); m.work.appendChild(el("ul")); m.root.insertBefore(m.work, m.bubble); }
      var li = el("li", item.act === "check" ? (item.ok === false ? "ng" : "ok") : "");
      li.appendChild(icon(item.act === "check" ? "check" : item.act)); li.appendChild(el("span", "", ACT[item.act] || item.act));
      var first = String(item.target || "").split("\n")[0]; var code = el("code", "", first + (String(item.target).indexOf("\n") >= 0 ? " …" : "")); code.title = item.target; li.appendChild(code);
      $("ul", m.work).appendChild(li);
    }
    // 返事の Markdown のうち、太字 ・ `コード` ・ 箇条書き ・ 見出しだけを、文字として作る（HTML としては読まない）
    function inline(parent, line) {
      var re = /(\*\*([^*]+)\*\*|`([^`]+)`)/g, at = 0, m;
      while ((m = re.exec(line))) {
        if (m.index > at) parent.appendChild(document.createTextNode(line.slice(at, m.index)));
        parent.appendChild(m[2] ? el("b", "", m[2]) : el("code", "", m[3]));
        at = re.lastIndex;
      }
      if (at < line.length) parent.appendChild(document.createTextNode(line.slice(at)));
    }
    function markdown(box, text) {
      box.textContent = "";
      var list = null;
      text.split("\n").forEach(function (line) {
        var item = line.match(/^\s*(?:[-*]|[0-9]+\.)\s+(.*)$/);
        if (item) {
          if (!list) { list = el(/^\s*[0-9]/.test(line) ? "ol" : "ul"); box.appendChild(list); }
          var li = el("li"); inline(li, item[1]); list.appendChild(li); return;
        }
        list = null;
        if (!line.trim()) { box.appendChild(el("div", "gap")); return; }
        var p = el("p"); inline(p, line.replace(/^#{1,6}\s+/, "")); if (/^#{1,6}\s/.test(line)) p.className = "h"; box.appendChild(p);
      });
    }
    function addText(m, text) {
      if (!m.bubble) { m.bubble = el("div", "bubble"); m.root.appendChild(m.bubble); m.raw = ""; }
      m.raw += text; markdown(m.bubble, m.raw);
    }
    function addNotice(kind, title, text, actions) {
      var c = el("div", "cand"); var h = el("div", "cand-h"); h.appendChild(icon("diff")); h.appendChild(el("b", "", title)); c.appendChild(h);
      c.appendChild(el("p", "", text));
      if (actions && actions.length) { var row = el("div", "cand-b"); actions.forEach(function (a) { var b = el("button", "btn" + (a.primary ? " primary" : ""), a.label); b.type = "button"; b.addEventListener("click", a.run); row.appendChild(b); }); c.appendChild(row); }
      msgs.appendChild(c); msgs.scrollTop = msgs.scrollHeight;
    }
    // 作業中の表示：動く印 ・ いま何をしているか ・ 経過時間。終わるまで会話の末尾に置く
    var running = null, runTimer = null, runStart = 0;
    function runState(text) {
      if (!running) return;
      $("[data-run-text]", running).textContent = text;
      msgs.appendChild(running); msgs.scrollTop = msgs.scrollHeight;
    }
    function busy(on) {
      stop.hidden = !on; input.setAttribute("aria-busy", String(on));
      if (on) {
        running = el("div", "running"); running.setAttribute("role", "status");
        var dots = el("span", "rdots"); dots.setAttribute("aria-hidden", "true"); dots.appendChild(el("i")); dots.appendChild(el("i")); dots.appendChild(el("i"));
        running.appendChild(dots); running.appendChild(el("span", "rtext", "AI に依頼を送っている")); $(".rtext", running).setAttribute("data-run-text", "");
        var tm = el("time", "rtime", "0:00"); running.appendChild(tm);
        runStart = Date.now(); runTimer = setInterval(function () { tm.textContent = mmss(Date.now() - runStart); }, 1000);
        msgs.appendChild(running); msgs.scrollTop = msgs.scrollHeight;
      } else {
        clearInterval(runTimer); if (running) running.remove(); running = null;
      }
    }
    function addUsage(m, u) {
      if (!u || !u.input) return;
      var pct = Math.round((u.cached / u.input) * 100);
      var line = el("div", "usage", "入力 " + u.input.toLocaleString() + "（キャッシュから " + pct + "%）・ 出力 " + u.output.toLocaleString());
      m.root.appendChild(line);
    }
    var result = null;
    function handle(ev) {
      var WORKING = { read: "読んでいる", edit: "直している", run: "実行している" };
      if (ev.type === "text") { addText(current, ev.text); runState("返事を書いている"); }
      else if (ev.type === "work") { addWork(current, ev); runState((WORKING[ev.act] || "作業している") + "：" + String(ev.target || "").split("\n")[0].slice(0, 60)); }
      else if (ev.type === "phase") { runState(ev.text); }
      else if (ev.type === "usage") { addUsage(current, ev); }
      else if (ev.type === "check") { addWork(current, { act: "check", target: ev.target || "check", ok: ev.ok }); }
      else if (ev.type === "outside") {
        var edited = ev.edited || ev.files || [], boards = ev.boards || [], unknown = ev.unknown || [];
        if (edited.length) {
          addNotice("outside", "AI がボードの外を直した", edited.join(" ・ "), [
            { label: "起動の前に戻す", primary: true, run: function () { postJSON("/api/restore", { board: boardId, run: ev.run }).then(function () { addNotice("restored", "戻した", "起動の前の写しへ戻した。", []); }); } },
            { label: "このままにする", run: function () {} }
          ]);
        }
        if (boards.length) {
          addNotice("outside", "ほかのボードが変わった", "AI の作業の記録に無い。同じ時間に別のセッションが動かしたことが多いので、このボードではすることは無い：" + boards.join(" ・ "), []);
        }
        if (unknown.length) {
          addNotice("outside", "ボードの外が変わった", (ev.ran ? "AI の作業の記録に無い。AI が実行したコマンドか、同じ時間のほかの作業が変えた：" : "AI の作業の記録に無い。同じ時間のほかの作業が変えた：") + unknown.join(" ・ "), []);
        }
      }
      else if (ev.type === "changed") { if (result) result.changed = true; runState("ボードが書き換わった。終わったら画面を描き直す"); }
      else if (ev.type === "error") { if (result) result.error = ev.text; addNotice("error", "止まった", ev.text, []); }
      msgs.scrollTop = msgs.scrollHeight;
    }
    // AI に聞くと回答を送るの両方が使う。出来事を1行ずつ読み、終わったら done に結果を渡す
    window.bsStream = function (url, payload, mine, done) {
      chatRunning = url === "/api/chat";
      addMine(mine); current = aiMessage(); busy(true); result = { changed: false, error: "" };
      if (!chatRunning) { /* 送るは、帯で進み具合を出す */ } else setLocked(true, "AI が質問に答えている", Date.now());
      controller = window.AbortController ? new AbortController() : null;
      fetch(url, { method: "POST", credentials: "same-origin", headers: { "Content-Type": "application/json" }, signal: controller && controller.signal, body: JSON.stringify(payload) })
        .then(function (r) {
          if (!r.ok || !r.body) { return r.json().then(function (j) { handle({ type: "error", text: j.reason || "受け取られなかった" }); }); }
          var reader = r.body.getReader(), dec = new TextDecoder(), buf = "";
          function pump() {
            return reader.read().then(function (res) {
              if (res.done) return;
              buf += dec.decode(res.value, { stream: true });
              var lines = buf.split("\n"); buf = lines.pop();
              lines.forEach(function (l) { if (l.trim()) { try { handle(JSON.parse(l)); } catch (e) { /* 壊れた行は捨てる */ } } });
              return pump();
            });
          }
          return pump();
        })
        .catch(function () { handle({ type: "error", text: "serve とのつながりが切れた" }); })
        .then(function () {
          var got = result; busy(false); chatRunning = false;
          if (done) done(got);
          else if (got.changed) reloadKeep();
          else setLocked(false);
        });
    };
    function ask(text) {
      if (!text.trim() || !served || locked) return;
      var topic = ($(".chip[aria-selected=true]") || {}).getAttribute ? $(".chip[aria-selected=true]").getAttribute("data-topic") : "";
      window.bsStream("/api/chat", { board: boardId, topic: topic, agent: select.value, text: text }, text, null);
    }
    var openButtons = $$("[data-open-chat]");
    listeners.push(function () {
      if (holdToggle) holdToggle.disabled = locked;
      submit.disabled = !served || locked;
      $$("[data-say]", chat).forEach(function (b) { b.disabled = !served || locked; });
      input.placeholder = locked && !chatRunning ? "更新が終わると聞ける" : "この論点について聞く・直してほしいことを書く";
    });
    function openChat() {
      var dr = $("[data-chg-drawer]");
      if (dr && !dr.hidden) { dr.hidden = true; var ob = $("[data-open-changes]"); if (ob) ob.setAttribute("aria-expanded", "false"); } // 重ねて開かない
      chat.hidden = false; app.classList.add("with-chat"); input.focus(); store.set("bs-chat-open-" + boardId, true); }
    openButtons.forEach(function (b) { b.addEventListener("click", openChat); });
    $("[data-close-chat]", chat).addEventListener("click", function () { chat.hidden = true; app.classList.remove("with-chat"); store.set("bs-chat-open-" + boardId, false); });
    $("[data-composer]", chat).addEventListener("submit", function (e) { e.preventDefault(); if (locked) return; var t = input.value; input.value = ""; ask(t); });
    input.addEventListener("keydown", function (e) { if (e.key === "Enter" && !e.shiftKey && !e.isComposing) { e.preventDefault(); $("[data-composer]", chat).requestSubmit(); } });
    $$("[data-say]", chat).forEach(function (b) { b.addEventListener("click", function () { ask(b.getAttribute("data-say")); }); });
    stop.addEventListener("click", function () { postJSON("/api/chat/stop", { board: boardId }); if (controller) controller.abort(); });
    if (store.get("bs-chat-open-" + boardId)) { chat.hidden = false; app.classList.add("with-chat"); }
    if (served) {
      $("[data-offline]", chat).hidden = true;
      fetch("/api/agents", { credentials: "same-origin" }).then(function (r) { return r.json(); }).then(function (j) {
        var ready = (j.data && j.data.agents) || {};
        $$("option", select).forEach(function (o) { if (!ready[o.value]) o.textContent = NAME[o.value] + "（見つからない）"; });
        var mark = function () { stateDot.setAttribute("data-ready", String(!!ready[select.value])); note.textContent = select.value === "kiro" ? "読むだけ（ボードは直さない）" : "書き換えはボードのフォルダだけ"; };
        select.addEventListener("change", function () { mark(); loadHistory(); }); mark();
      }, function () {});
      var loadHistory = function () {
        fetch("/api/chat?board=" + encodeURIComponent(boardId) + "&agent=" + select.value, { credentials: "same-origin" }).then(function (r) { return r.json(); }).then(function (j) {
          $$(".msg, .cand", msgs).forEach(function (n) { n.remove(); });
          ((j.data && j.data.messages) || []).forEach(function (m) {
            if (m.role === "user") addMine(m.text, m.at);
            else if (m.role === "assistant") { var a = aiMessage(m.at); addText(a, m.text || ""); addUsage(a, m.usage); }
            else if (m.role === "work") { var w = aiMessage(m.at); (m.items || []).forEach(function (it) { addWork(w, it); }); }
            else if (m.role === "notice") addNotice("notice", "serve の知らせ", m.text || "", []);
          });
        }, function () {});
      };
      loadHistory();
    } else {
      submit.disabled = true; note.textContent = "serve で開くと使える";
    }
  }
  changed();

  // ── ステータスを serve から読み直す（3秒ごと）。サイドバーとサマリーも、ロックから並べ直す ──
  var rev = null;
  function paintSide(all) {
    var groups = {};
    $$(".side [data-group]").forEach(function (g) { groups[g.getAttribute("data-group")] = g; });
    $$("[data-board-row]").forEach(function (li) {
      var id = li.getAttribute("data-board-row"), b = all[id];
      if (!b || !groups[b.status]) return;
      var list = $("ul", groups[b.status]);
      if (li.parentNode !== list) list.appendChild(li);
      var run = $("[data-run]", li);
      if (b.status === "updating" && b.lock) {
        run.hidden = false; run.textContent = b.lock.label + " ・ " + mmss(Date.now() - (Date.parse(b.lock.since) || Date.now()));
      } else run.hidden = true;
    });
    Object.keys(groups).forEach(function (k) {
      var n = $$("[data-board-row]", groups[k]).length, gc = $("[data-gc]", groups[k]);
      if (gc) gc.textContent = n + "枚";
      var tile = $('[data-tile="' + k + '"] .tn'); if (tile) tile.firstChild.nodeValue = String(n);
    });
  }
  function paintBoard(b) {
    if (rev === null) rev = b.rev;
    var busyNow = b.status === "updating";
    if (busyNow && !locked) setLocked(true, b.lock ? b.lock.label : "", b.lock ? Date.parse(b.lock.since) : Date.now());
    if (!busyNow && locked && !sending && !chatRunning) setLocked(false);
    if (!busyNow && b.rev !== rev && !sending && !chatRunning) reloadKeep();
  }
  function poll() {
    fetch("/api/status", { credentials: "same-origin" }).then(function (r) { return r.json(); }).then(function (j) {
      var all = (j.data && j.data.boards) || {};
      paintSide(all);
      if (boardId && all[boardId]) paintBoard(all[boardId]);
    }, function () {}).then(function () { setTimeout(poll, 3000); });
  }
  if (served) poll();

  // ── ダッシュボード：先に決めるものと最近の動き ──
  if (page === "dashboard") {
    var boards = $$(".prsrc");
    var waitedBy = {};
    boards.forEach(function (b) { (b.getAttribute("data-waits") || "").split(/[^a-z0-9-]+/).filter(Boolean).forEach(function (w) { waitedBy[w] = (waitedBy[w] || 0) + 1; }); });
    var today = new Date(); today.setHours(0, 0, 0, 0);
    var rows = [];
    boards.forEach(function (b) {
      var id = b.getAttribute("data-board"), flag = b.getAttribute("data-flag") === "true", opened = b.getAttribute("data-opened");
      var days = opened ? Math.max(0, Math.round((today - new Date(opened + "T00:00:00")) / 86400000)) : 0;
      $$(".pr", b).forEach(function (r) { rows.push({ el: r, id: id, title: b.getAttribute("data-title"), flag: flag, days: days, waited: waitedBy[id] || 0 }); });
    });
    rows.sort(function (x, y) { return (y.waited - x.waited) || ((y.flag ? 1 : 0) - (x.flag ? 1 : 0)) || (y.days - x.days); });
    var list = $("[data-prio]");
    if (list) {
      list.innerHTML = "";
      rows.forEach(function (r, i) {
        var li = el("li"); li.appendChild(r.el);
        $(".pn", r.el).textContent = String(i + 1);
        $(".pbd", r.el).textContent = r.title;
        var a = $("[data-to-topic]", r.el); a.href = r.id + ".html#" + a.getAttribute("data-to-topic");
        if (r.flag) $(".flag", r.el).hidden = false;
        $(".wait", r.el).textContent = (r.waited ? "ほかの " + r.waited + " 枚がこの答えを待っている ・ " : "") + "回答を待って " + r.days + "日";
        list.appendChild(li);
      });
      if (!rows.length) $("[data-prio-empty]").hidden = false;
    }
    var log = $("[data-log]");
    if (log) {
      var evs = [];
      $$(".logsrc").forEach(function (b) { $$(".ev", b).forEach(function (e) { evs.push({ date: e.getAttribute("data-date"), text: e.textContent, board: b.getAttribute("data-board") }); }); });
      evs.sort(function (x, y) { return x.date < y.date ? 1 : x.date > y.date ? -1 : 0; });
      evs.slice(0, 10).forEach(function (e) {
        var li = el("li"); li.appendChild(el("time", "", e.date));
        var a = el("a", "", e.board); a.href = e.board + ".html"; li.appendChild(a); li.appendChild(document.createTextNode(" ・ " + e.text)); log.appendChild(li);
      });
      if (!evs.length) log.appendChild(el("li", "", "まだ無い。"));
    }
  }
})();
