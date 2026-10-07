# 使い方: python3 assemble.py  → gallery.html を作る
import re,glob,os
D=os.path.dirname(os.path.abspath(__file__))
head=open(f'{D}/head.html').read()
order=['ux','ddd','usecase','qa','platform','meta-thinking']
names={'ux':'ux-advisor','ddd':'ddd-advisor','usecase':'usecase-advisor','qa':'qa-advisor','platform':'platform-advisor','meta-thinking':'meta-thinking-advisor'}
nav=['<nav aria-label="完成イメージの一覧">']; panels=[]; ids=[]
for a in order:
    f=f'{D}/frag-{a}.html'
    if not os.path.exists(f): continue
    t=open(f).read()
    secs=re.findall(r'<section class="panel"[^>]*data-p="([^"]+)"[^>]*data-kind="([^"]+)"[^>]*data-label="([^"]+)"',t)
    nav.append(f'<div class="grp">{names[a]}（{len(secs)}件）</div>')
    for pid,kind,label in secs:
        nav.append(f'<a href="#{pid}" data-p="{pid}"><span class="kd">{kind}</span><span class="lb">{label}</span></a>'); ids.append(pid)
    panels.append(t)
nav.append('</nav>')
body=f'''<div class="app">
<aside class="side">
<div class="title">advisor の完成イメージ</div>
<span class="sim">UI 案 ── 各 advisor が設計相談 ・ 判断相談で返す完成イメージを、種類ごとに1つずつ描いた見本。中身は想定の例</span>
{''.join(nav)}
</aside>
<main class="content">
{''.join(panels)}
<div class="pager"><a href="#" id="prev">← 前へ</a><a href="#" id="next">次へ →</a></div>
</main>
</div>
<script>
(function(){{
 var ids={ids!r};
 var links=document.querySelectorAll('.side nav a');
 function show(id,noScroll){{
  if(ids.indexOf(id)<0) id=ids[0];
  document.querySelectorAll('.panel').forEach(function(p){{p.hidden=p.dataset.p!==id;}});
  links.forEach(function(a){{ if(a.dataset.p===id) {{a.setAttribute('aria-current','page'); var n=a.parentNode; if(n.scrollWidth>n.clientWidth) n.scrollLeft=a.offsetLeft-16;}} else a.removeAttribute('aria-current'); }});
  var i=ids.indexOf(id), pv=document.getElementById('prev'), nx=document.getElementById('next');
  pv.hidden=i===0; nx.hidden=i===ids.length-1; pv.dataset.to=ids[i-1]||''; nx.dataset.to=ids[i+1]||'';
  if(!noScroll) window.scrollTo(0,0);
 }}
 links.forEach(function(a){{a.addEventListener('click',function(e){{e.preventDefault();show(a.dataset.p);}});}});
 ['prev','next'].forEach(function(k){{document.getElementById(k).addEventListener('click',function(e){{e.preventDefault();show(this.dataset.to);}});}});
 var h=(location.hash||'').slice(1); show(ids.indexOf(h)>=0?h:ids[0],true);
}})();
</script>'''
open(f'{D}/gallery.html','w').write(head+body)
print(len(ids),'panels')
