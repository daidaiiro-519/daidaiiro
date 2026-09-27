
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
      /* **選んだタブを、横の並びの中央へ送る** ── 携帯ではタブが横1行なので、
         跳んだ先のタブが画面の外に残る */
      b.scrollIntoView({inline:"center",block:"nearest"});
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
