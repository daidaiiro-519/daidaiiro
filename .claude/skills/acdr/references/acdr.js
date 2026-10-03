
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

  /* 一覧の「その箇所へ移動」。印を開いて、そこまで運ぶ */
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
      b.textContent = shown ? "その箇所へ移動" : "表示されていない箇所";
      setTimeout(function(){
        var y = 0, e = m;
        while (e && e.offsetParent) { y += e.offsetTop; e = e.offsetParent; }
        if (fr) window.scrollTo({top: fr.getBoundingClientRect().top + window.scrollY + y - 60,
                                 behavior: "smooth"});
        else m.scrollIntoView({block: "center", behavior: "smooth"});
      }, 60);
    };
  });

  /* マークダウンのファイル */
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
