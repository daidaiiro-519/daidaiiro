(function(){
var root=document.currentScript?document.currentScript.parentNode:document;
function each(sel,fn){Array.prototype.forEach.call(root.querySelectorAll(sel),fn)}
function own(el,sel){return Array.prototype.filter.call(el.children,function(c){return c.matches(sel)})}
each('[data-sd-part="tabs"]',function(t){var bar=own(t,".sd-tablist")[0],tabs=bar?own(bar,"[role=tab]"):[],panes=own(t,".sd-pane");
 tabs.forEach(function(b,i){b.addEventListener("click",function(){tabs.forEach(function(x,j){x.setAttribute("aria-selected",j===i?"true":"false");panes[j].hidden=j!==i})})})});
each('[data-sd-part="mark"] .sd-mark',function(m){m.addEventListener("click",function(){m.setAttribute("aria-expanded",m.getAttribute("aria-expanded")==="true"?"false":"true")})});
function reveal(el){for(var n=el;n&&n!==root;n=n.parentNode){if(n.tagName==="DETAILS")n.open=true;
 if(n.classList&&n.classList.contains("sd-pane")&&n.hidden){var t=n.parentNode,i=own(t,".sd-pane").indexOf(n),b=own(own(t,".sd-tablist")[0],"[role=tab]")[i];if(b)b.click()}}}
each('[data-sd-part="jump"]',function(j){j.addEventListener("click",function(){var el=document.getElementById(j.getAttribute("data-sd-to"));if(!el)return;reveal(el);el.scrollIntoView({block:"center",behavior:"smooth"})})});
each('[data-sd-part="choose"] .sd-option',function(o){o.addEventListener("click",function(){own(o.parentNode,".sd-option").forEach(function(x){x.setAttribute("aria-pressed",x===o?"true":"false")})})});
each('[data-sd-part="send"]',function(s){var out=own(s,".sd-out")[0];own(s,"button")[0].addEventListener("click",function(){var answers={};
 each('[data-sd-part="choose"]',function(c){var on=c.querySelector('[aria-pressed="true"]');if(on)answers[c.getAttribute("data-sd-name")]=on.value});
 each('[data-sd-part="write"]',function(w){var v=w.querySelector("textarea").value.trim();if(v)answers[w.getAttribute("data-sd-name")]=v});
 var text=JSON.stringify({answers:answers},null,2);out.hidden=false;out.textContent=text;
 fetch(s.getAttribute("data-sd-to"),{method:"POST",headers:{"Content-Type":"application/json"},body:text})
  .then(function(r){out.textContent=(r.ok?"送りました。":"受け取られませんでした。下の文字列を貼ってください。")+"\n\n"+text})
  .catch(function(){out.textContent="配るサーバーがありません。下の文字列を貼ってください。\n\n"+text})})});
})();
