(function(){
  var ROOT=document.getElementById("root");
  var tabs=document.querySelectorAll("#tabs button");

  /* UI の完成イメージは、中身の高さに合わせる ── 隠れている間は高さが測れないので、見えたときに測り直す */
  function fit(){
    document.querySelectorAll("iframe.ix-ui").forEach(function(f){
      try{
        var d=f.contentDocument;
        if(!d||!d.documentElement||f.offsetParent===null) return;
        var h=d.documentElement.scrollHeight;
        if(h>0) f.style.height=(h+2)+"px";
      }catch(e){}
    });
  }
  document.querySelectorAll("iframe.ix-ui").forEach(function(f){f.addEventListener("load",fit)});
  tabs.forEach(function(b){
    b.addEventListener("click",function(){
      tabs.forEach(function(o){
        o.setAttribute("aria-selected","false");
        document.getElementById(o.dataset.t).hidden=true;
      });
      b.setAttribute("aria-selected","true");
      document.getElementById(b.dataset.t).hidden=false;
      fit();
      /* 携帯ではタブが横1行なので、選んだタブを並びの中央へ送る */
      b.scrollIntoView({inline:"center",block:"nearest"});
      window.scrollTo({top:0,behavior:"smooth"});
    });
  });

  /* 現在地の「いま見る論点」から、その論点のタブへ移る */
  document.querySelectorAll(".jump").forEach(function(j){
    j.addEventListener("click",function(){
      var t=document.querySelector('#tabs button[data-t="'+j.dataset.go+'"]');
      if(t) t.click();
    });
  });

  /* 完成イメージの切り替え（図 ・ コード ・ UI ・ 差分） */
  document.querySelectorAll(".ix-tab").forEach(function(b){
    b.addEventListener("click",function(){
      var g=b.dataset.g;
      document.querySelectorAll('.ix-tab[data-g="'+g+'"]').forEach(function(x){
        x.setAttribute("aria-selected",x===b?"true":"false");
      });
      document.querySelectorAll('[data-ixp][data-g="'+g+'"]').forEach(function(p){
        p.hidden=p.dataset.ixp!==b.dataset.ix;
      });
      fit();
    });
  });

  /* 回答 ── 論点ごとに1件だけ入る */
  var state={};
  document.querySelectorAll(".form").forEach(function(f){
    var topic=f.dataset.topic;
    f.querySelectorAll(".vb").forEach(function(b){
      b.addEventListener("click",function(){
        f.querySelectorAll(".vb").forEach(function(o){o.setAttribute("aria-pressed","false")});
        b.setAttribute("aria-pressed","true");
        state[topic]=b.dataset.v; count();
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
  /* 送る ── answer.schema.json の形のインスタンスを作る */
  document.getElementById("go").addEventListener("click",function(){
    var answers=[];
    document.querySelectorAll(".form").forEach(function(f){
      var topic=f.dataset.topic;
      if(!state[topic]) return;
      var a={topic:ROOT.dataset.board+"."+topic, verdict:state[topic]};
      var r=f.querySelector(".reason").value.trim();
      var nt=f.querySelector(".note-in").value.trim();
      if(state[topic]==="returned"&&r) a.reason=r;
      if(nt) a.note=nt;
      answers.push(a);
    });
    var round=Number(ROOT.dataset.round);
    var sheet={kind:"answer", id:round+"-"+Date.now(), board:ROOT.dataset.board, round:round, answers:answers};
    var txt=JSON.stringify(sheet,null,2);
    var out=document.getElementById("out"); out.hidden=false; out.textContent=txt;
    fetch("/answer",{method:"POST",headers:{"Content-Type":"application/json"},body:txt})
      .then(function(r){return r.json().then(function(j){
        out.textContent=(r.ok?"送りました。"+(j.next||"")+"\n\n":"受け取られませんでした。貼ってください。\n\n")+txt;})})
      .catch(function(){ /* 配るサーバーが無ければ、貼る形のまま */ });
    out.scrollIntoView({block:"nearest",behavior:"smooth"});
  });
})();
