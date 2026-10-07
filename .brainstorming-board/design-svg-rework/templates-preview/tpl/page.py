import json,re,html,os
exec(open('texts.py').read())
G='/home/daidaiiro/workspace/daidaiiro/.brainstorming-board/advisor-answer/ui-samples/gallery/'
head=open(G+'head.html').read().replace('<title>advisor 完成イメージ一覧</title>','<title>図のテンプレート</title>')
theme=json.load(open('/home/daidaiiro/workspace/daidaiiro/.claude/skills/design-svg/references/theme.json'))
dvars=';'.join(f"--dsvg-{k[6:]}:{v}" for k,v in theme['dark'].items())
META=[
('hub','ハブアンドスポーク図','1つの中心と、それにつながる要素の関係を示す','ddd-1','病院の予約システムの文脈の地図'),
('lanes-over-time','スイムレーン図','種類ごとのレーンに、出来事を時系列で並べる','ddd-3','勉強会の申し込みのイベントストーミング'),
('message-sequence','シーケンス図','参加者の間のメッセージの送受信を、時系列で示す','ddd-8','ホテル予約のサーガと補償'),
('nested-areas','構成図','要素がどの領域（ネットワーク ・ グループ）に含まれるかを示す','platform-1','病院の予約サイトのクラウド構成'),
('layer-stack','レイヤー図','上下の順に意味がある層を示す（抽象度 ・ ネットワークの層）','meta-thinking-1','テストの考え方 ・ やり方 ・ 作業の3層'),
('tree','ツリー図','1つの親が複数の子に分かれる階層を示す','usecase-3','ネットショップのユースケースの目的の階層'),
('flow-steps','フロー図','手順や画面の遷移を、1本の順序で示す','ux-1','病院の予約の画面の流れ'),
('state-transition','状態遷移図','1つの対象の状態と、状態を変えるイベントを示す','qa-2','図書館の本の貸出の状態'),
('row-column-grid','マトリクス','行と列の組み合わせごとに値や状態を示す','platform-4','問い合わせの優先度ごとの対応と目標時間'),
('quadrant','4象限図','2つの軸で4つの象限に分類する','qa-3（2つ目）','仕事の重要度と緊急度'),
('quantity-pyramid','ピラミッド図','段ごとの量の大小を、上から下へ示す','qa-3（1つ目）','サポートの問い合わせの段ごとの件数'),
('system-boundary','ユースケース図','システムの境界と、その外のアクターを示す','usecase-4','病院の予約システムの範囲'),
('wireframe','ワイヤーフレーム','画面の UI 部品の配置を示す','ux-2','病院の予約の入力画面'),
('compare-before-after','比較の図','同じ形を左右に並べ、変更前と変更後の差を示す','（追加）','レビューの手順の変更'),
('gantt','ガントチャート','作業ごとの期間を、時間の軸に帯で示す','（追加）','チームの第2四半期の計画'),
('er-diagram','ER 図','エンティティと属性と、エンティティの間の関連の数を示す','（追加）','ブログのユーザー ・ 投稿 ・ コメント ・ タグ'),
('venn','ベン図','集合の重なりで、両方に属する要素を示す','（追加）','チームのスキルの重なり'),
]
NEWIDS={'compare-before-after','gantt','er-diagram','venn'}
idx=[]
for i,n,u,f,subj in META:
    s=open(f'templates/{i}.svg').read()
    cl=sorted({c for m in re.findall(r'class="([^"]*)"',s) for c in m.split()})
    idx.append({'id':i,'name':n,'use':u,'file':f'templates/{i}.svg','classes':cl,'from':f,'added':i in NEWIDS})
json.dump({'templates':idx},open('templates.json','w'),ensure_ascii=False,indent=1)
nav=['<nav aria-label="テンプレートの一覧">']; panels=[]; ids=[]
for i,n,u,f,subj in META:
    t=open(f'resolved/t-{i}.svg').read(); g0=open(f'resolved/f-{i}.svg').read()
    g=g0; gd=g0
    chip='追加' if i in NEWIDS else 'テンプレート'
    kd=f'<span class="kd">追加</span>' if i in NEWIDS else ''
    nav.append(f'<a href="#{i}" data-p="{i}"><span class="lb">{n}</span>{kd}</a>'); ids.append(i)
    tag=f'<span class="tag">追加</span>' if i in NEWIDS else ''
    panels.append(f'''<section class="panel" id="p-{i}" data-p="{i}" hidden>
<h2>{n} {tag}</h2>
<div class="card"><p class="use"><b>用途</b>　{u}</p><p class="meta"><code>{i}</code>　元にした図：{f}</p></div>
<div class="grid3">
<figure class="card tpl"><figcaption>テンプレート（{html.escape(i)}）</figcaption><div class="sv">{t}</div></figure>
<figure class="card big"><figcaption>テンプレートから作った図：{subj}</figcaption><div class="sv">{g}</div></figure>
<figure class="card big dk"><figcaption>同じ図（ダークモード）</figcaption><div class="sv">{gd}</div></figure>
</div></section>''')
nav.append('</nav>')
extra=f'''<style>
.card{{background:var(--inset);border:1px solid var(--rule);border-radius:10px;box-shadow:var(--shadow);padding:12px 14px;min-width:0}}
.card p{{margin:2px 0}}.use b{{color:var(--accent)}}.meta{{font-size:.8rem;color:var(--dim)}}
.grid3{{display:grid;gap:12px;margin-top:12px}}
figure.card{{margin:0;display:grid;gap:6px}}
figcaption{{font-size:.78rem;color:var(--dim);font-weight:700}}
.sv{{overflow-x:auto}}.sv svg{{width:100%;height:auto;display:block}}
.tpl .sv{{max-width:420px}}.tpl .sv svg{{min-width:0!important}}.side nav a .lb{{white-space:nowrap}}
.dk{{background:#16211E;border-color:#354642}}.dk figcaption{{color:#9DADA8}}.dk .dsvg{{{dvars}}}
.tag{{flex:none;font-size:.72rem;font-weight:700;color:var(--accent);background:var(--accent-bg);padding:1px 8px;border-radius:9px}}
.side nav a .kd{{flex:none;font-size:.68rem;font-weight:700;color:var(--accent);background:var(--accent-bg);padding:0 7px;border-radius:9px}}
</style>'''
body=f'''{extra}<div class="app">
<aside class="side">
<div class="title">図のテンプレート（17種類）</div>
<span class="sim">ACDR 2 の下見 ── テンプレートと、そこから作った図。中身は想定の例</span>
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
  links.forEach(function(a){{ if(a.dataset.p===id) a.setAttribute('aria-current','page'); else a.removeAttribute('aria-current'); }});
  var i=ids.indexOf(id), pv=document.getElementById('prev'), nx=document.getElementById('next');
  pv.hidden=i===0; nx.hidden=i===ids.length-1; pv.dataset.to=ids[i-1]||''; nx.dataset.to=ids[i+1]||'';
  if(!noScroll) window.scrollTo(0,0);
 }}
 links.forEach(function(a){{a.addEventListener('click',function(e){{e.preventDefault();history.replaceState(null,'','#'+a.dataset.p);show(a.dataset.p);}});}});
 ['prev','next'].forEach(function(k){{document.getElementById(k).addEventListener('click',function(e){{e.preventDefault();history.replaceState(null,'','#'+this.dataset.to);show(this.dataset.to);}});}});
 var h=(location.hash||'').slice(1); show(ids.indexOf(h)>=0?h:ids[0],true);
}})();
</script>'''
open('../templates-preview.html','w').write('<!doctype html><html lang="ja"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">\n'+head+body)
print(len(ids))
