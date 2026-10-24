import json,os
D=os.path.dirname(os.path.abspath(__file__))
settled=open(os.path.join(D,'abstract.settled.json')).read()
after=open(os.path.join(D,'abstract.json')).read()
conc=open(os.path.join(D,'concrete.json')).read()
tpl=open(os.path.join(D,'tpl.html')).read()
script=tpl[tpl.index('function esc'):tpl.index('let i=0;')]
css=tpl[tpl.index('<style>'):tpl.index('</style>')+8]
data={'files':[
 dict(f='semantic-abstract-schema.json（Abstract の本体）',why='決着済みの Q7 の本体に、今回決めたことを入れた',before=settled,after=after),
 dict(f='acdr の Semantic Concrete Schema（urn:semantic-agent-schema:acdr:decision-record:v1）',why='上の Abstract に従って、ACDR の Concrete を書いた',before='',after=conc)]}
changes=[
 ('URN の持ち主','core → base（$id ・ $vocabulary ・ annotations の参照）','Q6 #18：Abstract は基盤が持ち、基盤だけが変える'),
 ('根の x-prompt の read','parse と shown を足した','Abstract 自身も read の観点を使う'),
 ('根の x-prompt の write','read-perspectives を足した。rule-agent を「by と kind の対応」に書き直した。core を含む id を abstract に','read の観点を要る欄に書かせる ／ x-delegates の中身を変えたため'),
 ('根の x-delegates（Abstract 自身）','judge を置いた','基盤自身の AI の行（read-perspectives など）を判定する任せる先'),
 ('properties.x-delegates','中身を「名前 → Concrete の URN」から「名前 → $defs/delegate（自分の任せる先の定義）」に変えた。x-prompt を付け直した','ほかの Concrete を名指ししない。定義の形を JSON Schema で検査できる'),
 ('$defs/delegate','足した：role ・ answer ・ run（kind は目的で judging ・ working、argv があれば CLI で起動）','任せる先1件の形。目的と起動のしかたを別の軸にした'),
 ('Abstract 自身の AI の行（5つ）','agent を reviewer から judge に','予約した reviewer は無くなり、Abstract 自身の任せる先は judge'),
 ('$defs/published','x-prompt を足した','x- のキーワードは書く単位なので x-prompt が要る（Q7 #7）。Abstract 自身が決まりを破っていた'),
 ('$defs/rule の agent','説明を「任せる先（x-delegates）の名前」に','by: machine でも CLI の任せる先を指せる'),
 ('本体の $schema','標準の 2020-12 から、本体自身の $id に（自分で自分を検査する）','本体に書いた x-delegates と x-extensions を、本体の決まりで検査するため（Q6 #14）'),
 ('Abstract の9つの x-','根の x-extensions の一覧に、形（$defs を $ref）・ 説明 ・ prompt を書く形にそろえた。形は $defs に移した','Concrete が足す x- と同じ契約で定義する。契約の外の x- を無くす'),
 ('本体の部品の欄（23）','rule ・ prompt ・ reading ・ delegate ・ derived ・ field の欄に description と x-prompt を書いた。$defs/rule の欄 rule の write に「状態で書く」、欄 by の write に「値だけで決まる決まりは JSON Schema で書く」','本体が自分を検査するようにしたら、自分の決まり（部品の欄も書く単位）に59件違反していた'),
 ('$defs/fields','$schema ・ 入れ物（properties ・ $defs）・ x- のキーを、書く単位から外した','Q7 #7（入れ物には書かない）と、x- は x-extensions が説明するため'),
 ('AI を呼ぶ書式','Abstract に置かない（protocol の call に置く）','受け渡しの形は protocol（Q6 #16）'),
]
rows=''.join(f'<tr><td>{a}</td><td>{b}</td><td>{c}</td></tr>' for a,b,c in changes)
html=f'''<title>Abstract と Concrete の完成イメージ</title>
<meta name="claude:viewport-fit-sides" content="cover">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;700&family=JetBrains+Mono:wght@400&display=swap">
{css}
<style>table.t{{border-collapse:collapse;width:100%;font-size:13.5px;background:var(--card)}} .t td,.t th{{border:1px solid var(--line);padding:6px 8px;text-align:left;vertical-align:top}} .t th{{background:var(--code)}} .tw{{overflow-x:auto}}</style>
<main>
<header><h1>Abstract と Concrete の完成イメージ</h1><p class="meta">JSON Schema の実装（jsonschema 4.19）で、本体を本体自身で、Concrete を本体で、インスタンスを Concrete で検査し、どれも0件（実測）。Abstract は決着済みの Q7 の本体との差分。Concrete は、その Abstract に従って書いた ACDR の全体 ・ どちらも案で、ボードにはまだ当てていない</p></header>
<h2>Abstract の本体で変えたところ（14）</h2>
<div class="tw"><table class="t"><tr><th>どこ</th><th>何を</th><th>なぜ</th></tr>{rows}</table></div>
<section class="card" id="sc"></section>
<h2>ACDR の Concrete の読みどころ</h2>
<div class="tw"><table class="t">
<tr><th>どこ</th><th>中身</th><th>Abstract のどこに従っているか</th></tr>
<tr><td>根</td><td>$schema ・ $id ・ title ・ description ・ x-instance-path ・ x-instance-label</td><td>properties（Concrete の根に書けるもの）と required</td></tr>
<tr><td>x-delegates</td><td>reviewer（judging）と acdr-inspect（program）</td><td>properties.x-delegates → $defs/delegate</td></tr>
<tr><td>properties</td><td>欄ごとに description と x-prompt（read と write）</td><td>$defs/fields → $defs/field、x-prompt は $defs/prompt</td></tr>
<tr><td>欄の値の決まり</td><td>pattern ・ items ・ required ・ minItems ・ oneOf ・ default</td><td>標準の JSON Schema（allOf で取り込んだメタスキーマ）</td></tr>
<tr><td>allOf の if ・ then</td><td>superseded なら supersedes、accepted なら docs が要る</td><td>同上</td></tr>
<tr><td>x-prompt の write の行</td><td>JSON Schema で書けないものだけ（記録をまたぐ ・ Git を読む ・ 意味の判定）</td><td>$defs/rule。by と agent の対応は根の rule-agent</td></tr>
</table></div>
</main>
<script>
const DATA={json.dumps(data,ensure_ascii=False)};
{script}
const S=[DATA];let i=0;
document.querySelector('#sc').innerHTML='<h2>ファイル</h2>'+DATA.files.map(file).join('');
document.querySelector('#sc').addEventListener('click',e=>{{const bt=e.target.closest('.tg button');if(!bt)return;const fe=bt.closest('.file'),f=DATA.files[+fe.dataset.i],v=bt.dataset.v,d=fe.querySelector('.diff');
 const a=f.before?f.before.split('\\n'):[],b=f.after.split('\\n');
 if(bt.classList.contains('on')&&(v==='after'||v==='before')&&(f.ref||!f.before)){{d.hidden=!d.hidden;bt.classList.toggle('on');return}}
 fe.querySelectorAll('.tg button').forEach(x=>x.classList.remove('on'));bt.classList.add('on');d.hidden=false;
 if(v==='diff')d.innerHTML=render(lcs(a,b),2);else if(v==='all')d.innerHTML=render(lcs(a,b),-1);
 else if(v==='before')d.innerHTML=a.map(l=>'<div>'+esc('  '+l)+'</div>').join('');
 else d.innerHTML=(f.before||f.ref?b.map(l=>'<div>'+esc('  '+l)+'</div>'):b.map(l=>'<div class="a">'+esc('+ '+l)+'</div>')).join('')}});
</script>
'''
open(os.path.join(D,'abstract-and-concrete.html'),'w').write(html)
print('ok')
