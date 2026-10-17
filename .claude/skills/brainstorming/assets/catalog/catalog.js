// brainstorming の部品の操作。ページは基盤の render が作り、serve で開いたときだけ回答の送信と AI のチャットが動く。
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
    var ctx = $("[data-ctx-topic]"); if (ctx) { var chip = $('.chip[data-topic="' + id + '"]'); ctx.textContent = chip ? chip.textContent : id; }
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
    var open = chips.filter(function (c) { return c.getAttribute("data-status") === "open"; })[0];
    openTopic(hash || (open || chips[0]).getAttribute("data-topic"));
  })();

  // ── 完成イメージのタブ ──
  $$("section[data-part=images]").forEach(function (block) {
    var tabs = $$(".tabs button", block), panes = $$(".pane", block);
    function show(i) {
      tabs.forEach(function (t, j) { t.setAttribute("aria-selected", String(i === j)); });
      panes.forEach(function (p, j) { p.hidden = i !== j; });
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

  // ── 回答 ──
  function postJSON(url, data) {
    return fetch(url, { method: "POST", credentials: "same-origin", headers: { "Content-Type": "application/json" }, body: JSON.stringify(data) });
  }
  $$("[data-answer-form]").forEach(function (form) {
    var decision = null;
    var send = $("[data-send]", form), msg = $("[data-msg]", form), paste = $("[data-paste]", form);
    var field = function (name) { return $('[data-field="' + name + '"]', form).value.trim(); };
    function update() {
      $$(".fld[data-for]", form).forEach(function (f) { f.hidden = !decision || f.getAttribute("data-for").split(" ").indexOf(decision) < 0; });
      var ok = decision && (decision !== "return" || field("where"));
      send.disabled = !ok;
      msg.textContent = decision === "return" && !field("where") ? "どこが違うかを書くと送れる。" : "";
    }
    $$("[data-decision]", form).forEach(function (b) {
      b.addEventListener("click", function () {
        decision = b.getAttribute("data-decision");
        $$("[data-decision]", form).forEach(function (x) { x.setAttribute("aria-pressed", String(x === b)); });
        update();
      });
    });
    $$("textarea", form).forEach(function (t) { t.addEventListener("input", update); });
    send.addEventListener("click", function () {
      var data = { board: boardId, round: Number(body.getAttribute("data-round")), topic: form.getAttribute("data-topic"), decision: decision };
      ["where", "reason", "note"].forEach(function (k) { if (field(k)) data[k] = field(k); });
      var showPaste = function (why) {
        paste.hidden = false;
        paste.textContent = JSON.stringify(data, null, 1);
        msg.textContent = why + " 下の文字列を、AI とのチャットに貼る。";
      };
      if (!served) { showPaste("serve で開いていないので送れない。"); return; }
      send.disabled = true; msg.textContent = "送っている…";
      postJSON("/api/answer", data).then(function (r) { return r.json().then(function (j) { return [r.ok, j]; }); }).then(function (res) {
        if (res[0]) msg.textContent = "送った。AI に「回答した」と一言伝えると、次の回へ進む。";
        else showPaste("受け取られなかった（" + (res[1].reason || "") + "）。");
      }, function () { showPaste("serve に届かなかった。"); });
    });
  });

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
    function busy(on) { submit.disabled = on; stop.hidden = !on; }
    function handle(ev) {
      if (ev.type === "text") { addText(current, ev.text); }
      else if (ev.type === "work") { addWork(current, ev); }
      else if (ev.type === "check") { addWork(current, { act: "check", target: ev.target || "check", ok: ev.ok }); }
      else if (ev.type === "outside") {
        addNotice("outside", "ボードの外が変わった", ev.files.join(" ・ "), [
          { label: "起動の前に戻す", primary: true, run: function () { postJSON("/api/restore", { board: boardId, run: ev.run }).then(function () { addNotice("restored", "戻した", "起動の前の写しへ戻した。", []); }); } },
          { label: "このままにする", run: function () {} }
        ]);
      }
      else if (ev.type === "changed") {
        addNotice("changed", "ボードが書き換わった", "画面を描き直した。変わったところは次の回で確かめる。", [{ label: "読み込み直す", primary: true, run: function () { location.reload(); } }]);
      }
      else if (ev.type === "error") { addNotice("error", "止まった", ev.text, []); }
      msgs.scrollTop = msgs.scrollHeight;
    }
    function ask(text) {
      if (!text.trim() || !served) return;
      addMine(text); current = aiMessage(); busy(true);
      controller = window.AbortController ? new AbortController() : null;
      var topic = ($(".chip[aria-selected=true]") || {}).getAttribute ? $(".chip[aria-selected=true]").getAttribute("data-topic") : "";
      fetch("/api/chat", { method: "POST", credentials: "same-origin", headers: { "Content-Type": "application/json" }, signal: controller && controller.signal,
        body: JSON.stringify({ board: boardId, topic: topic, agent: select.value, text: text }) })
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
        .then(function () { busy(false); });
    }
    function openChat() { chat.hidden = false; app.classList.add("with-chat"); input.focus(); }
    $$("[data-open-chat]").forEach(function (b) { b.addEventListener("click", openChat); });
    $("[data-close-chat]", chat).addEventListener("click", function () { chat.hidden = true; app.classList.remove("with-chat"); });
    $("[data-composer]", chat).addEventListener("submit", function (e) { e.preventDefault(); var t = input.value; input.value = ""; ask(t); });
    input.addEventListener("keydown", function (e) { if (e.key === "Enter" && !e.shiftKey && !e.isComposing) { e.preventDefault(); $("[data-composer]", chat).requestSubmit(); } });
    $$("[data-say]", chat).forEach(function (b) { b.addEventListener("click", function () { ask(b.getAttribute("data-say")); }); });
    stop.addEventListener("click", function () { postJSON("/api/chat/stop", { board: boardId }); if (controller) controller.abort(); });
    if (served) {
      $("[data-offline]", chat).hidden = true;
      fetch("/api/agents", { credentials: "same-origin" }).then(function (r) { return r.json(); }).then(function (j) {
        var ready = (j.data && j.data.agents) || {};
        $$("option", select).forEach(function (o) { if (!ready[o.value]) o.textContent = NAME[o.value] + "（見つからない）"; });
        var mark = function () { stateDot.setAttribute("data-ready", String(!!ready[select.value])); };
        select.addEventListener("change", function () { mark(); loadHistory(); }); mark();
      }, function () {});
      var loadHistory = function () {
        fetch("/api/chat?board=" + encodeURIComponent(boardId) + "&agent=" + select.value, { credentials: "same-origin" }).then(function (r) { return r.json(); }).then(function (j) {
          $$(".msg, .cand", msgs).forEach(function (n) { n.remove(); });
          ((j.data && j.data.messages) || []).forEach(function (m) {
            if (m.role === "user") addMine(m.text, m.at);
            else if (m.role === "assistant") { var a = aiMessage(m.at); addText(a, m.text || ""); }
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
